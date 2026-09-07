mod common;
use common::*;
use std::cell::Cell;
use std::rc::Rc;
use ui_layout::*;

/// Mimics orinium's `TextFlowLayouter`: centers every line within
/// `containing_block_width` (falling back to a huge value when the containing
/// block width is unknown) and wraps clusters once the running width exceeds
/// `available_inline_size`. A regression shows up as astronomical span/box
/// coordinates, exactly like the translate.google.co.jp header bug
/// (`available_inline_size == 0` because the parent line position leaked into
/// the width computation).
#[derive(Debug)]
struct CenteredWrappingText {
    recorded: Rc<Cell<f32>>,
    text_width: f32,
}

impl CenteredWrappingText {
    fn new(text_width: f32, recorded: Rc<Cell<f32>>) -> Self {
        Self {
            recorded,
            text_width,
        }
    }
}

impl CustomLayouter for CenteredWrappingText {
    fn layout(&mut self, ctx: &LayoutContext) -> LayoutBox {
        let lh = ctx.line_height.max(16.0);
        let cluster_w = 14.0;
        let clusters = (self.text_width / cluster_w).round() as usize;

        self.recorded.set(ctx.available_inline_size);

        let mut spans = Vec::new();
        let mut line_x = 0.0;
        let mut line_y = 0.0;
        let mut line_idx = 0;
        for _ in 0..clusters {
            if line_x > 0.0 && line_x + cluster_w > ctx.available_inline_size {
                line_idx += 1;
                line_x = 0.0;
                line_y += lh;
            }
            // orinium: `containing_block_width.unwrap_or(f32::MAX)`.
            let cb_w = ctx.containing_block_width.unwrap_or(f32::MAX);
            let aligned = (cb_w - cluster_w) / 2.0 + line_x;
            spans.push(LineSpan {
                x_range: aligned..(aligned + cluster_w),
                line_pos: (aligned, line_y),
                line_index: line_idx,
            });
            line_x += cluster_w;
        }

        let (start_x, start_y) = ctx.start_pos;
        let total_width = spans
            .iter()
            .map(|s| s.line_pos.0 + s.width())
            .filter(|x| !x.is_nan())
            .max_by(f32::total_cmp)
            .map(|max_x| (max_x - start_x).max(0.0))
            .unwrap_or(0.0);
        let total_height = spans
            .iter()
            .map(|s| s.line_index)
            .max()
            .map(|line| (line as f32 + 1.0) * lh)
            .unwrap_or(0.0);

        let box_model = BoxModel::from(rect(start_x, start_y, total_width, total_height));
        LayoutBox::InlineBox(InlineBox {
            box_model,
            line_spans: spans,
        })
    }

    fn measure(&self, _ctx: &LayoutContext) -> MeasureResult {
        MeasureResult {
            width: self.text_width,
            height: 16.0,
        }
    }
}

#[test]
fn inline_block_nested_in_midline_flow_gets_positive_available_space() {
    // Reproduces the header of translate.google.co.jp:
    //   root(800) > [frame(700 wide), section]  (frame pushes the line to x=700)
    //   section (inline flow) > a               (inline-block, explicit 508 wide)
    //   a (flow-root) > wrapper (inline flow) > text (CenteredWrappingText)
    //
    // Before the fix, the flows inside `a` started at the parent's line
    // position (x=700), so `available_inline_size` became 0 and the 56px text
    // wrapped onto a second line using a gigantic unknown containing block
    // width — blowing the box up to ~8.5e37.
    let recorded = Rc::new(Cell::new(-1.0));
    let text = custom_inline(CenteredWrappingText::new(56.0, recorded.clone()));

    let wrapper = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            ..Default::default()
        },
        vec![text],
    );
    let a = LayoutNode::with_children(
        Style {
            display: Display::parse("inline-block").unwrap(),
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(508.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![wrapper],
    );
    let section = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            ..Default::default()
        },
        vec![a],
    );
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(800.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![
            LayoutChild::from(fragment(700.0, 40.0)),
            LayoutChild::from(section),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // The text layouter must have seen a positive, finite amount of space:
    // the leaked line position (700) must not be subtracted from the 508 box.
    let available = recorded.get();
    assert!(
        available.is_finite() && available > 0.0,
        "available_inline_size must be positive and finite, got {available:?}"
    );

    // And no span/box coordinate may approach f32::MAX.
    let boxes: Vec<_> = root.layout_box.iter().collect();
    for b in &boxes {
        assert!(
            b.border_box.x.abs() < 100_000.0 && b.border_box.width.abs() < 100_000.0,
            "layout produced an astronomical box: {:?}",
            b.border_box
        );
    }
}

/// An auto-width inline-block whose contents are *aligned* (here: centered)
/// must shrink-wrap to its content, not to the parent's available width.
///
/// Root cause pairing with the giga bug: centered text lays its glyphs out
/// against the containing width, and the resulting line advance used to widen
/// the shrink-to-fit inline-block (e.g. the ログイン `<a>` inflating from 85px
/// to fit a 486px line). The engine re-lays-out the inner flow with an
/// ever-tighter available width until the box stops shrinking.
#[test]
fn auto_width_inline_block_with_centered_text_shrinks_to_content() {
    let text = custom_inline(CenteredWrappingText::new(56.0, Rc::new(Cell::new(0.0))));
    let wrapper = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            ..Default::default()
        },
        vec![text],
    );
    let a = LayoutNode::with_children(
        Style {
            display: Display::parse("inline-block").unwrap(),
            size: SizeStyle {
                min_width: LengthOrAuto::Length(Length::Px(85.0)),
                ..Default::default()
            },
            box_sizing: BoxSizing::BorderBox,
            spacing: Spacing {
                padding_left: Length::Px(12.0),
                padding_right: Length::Px(12.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![wrapper],
    );
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(800.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![a],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let boxes: Vec<_> = root.layout_box.iter().collect();
    for b in &boxes {
        assert!(
            b.border_box.x.abs() < 100_000.0 && b.border_box.width.abs() < 100_000.0,
            "layout produced an astronomical box: {:?}",
            b.border_box
        );
    }

    // The inline-block must be 85px (min-width), i.e. its content (56px text)
    // must NOT have been inflated by centering against the 800px line — and the
    // centered text must sit inside that box, not at (800 - 56) / 2 = 372.
    let a_box = boxes
        .iter()
        .find(|b| (b.border_box.width - 85.0).abs() < 4.0)
        .expect("the inline-block should shrink-wrap to its min-width");
    assert!(
        a_box.border_box.width <= 90.0,
        "auto-width inline-block inflated by alignment offset: {:?}",
        a_box.border_box
    );
}