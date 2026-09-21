mod common;
use common::*;
use ui_layout::*;

fn border_box_of(n: &LayoutNode) -> Rect {
    match &n.layout_box {
        LayoutBox::BlockBox(b) => b.border_box,
        LayoutBox::InlineBox(b) => b.box_model.border_box,
        _ => panic!("expected a block or inline box"),
    }
}

fn inline_box(n: &LayoutNode) -> &InlineBox {
    match &n.layout_box {
        LayoutBox::InlineBox(b) => b,
        _ => panic!("expected inline box"),
    }
}

fn floated(width: f32, height: f32, side: Float) -> LayoutNode {
    LayoutNode::new(Style {
        float: side,
        size: SizeStyle {
            width: Length::Px(width).into(),
            height: Length::Px(height).into(),
            ..Default::default()
        },
        ..Default::default()
    })
}

fn fixed_block(width: f32, height: f32, clear: Clear) -> LayoutNode {
    LayoutNode::new(Style {
        clear,
        size: SizeStyle {
            width: Length::Px(width).into(),
            height: Length::Px(height).into(),
            ..Default::default()
        },
        ..Default::default()
    })
}

fn root_with(width: f32, children: Vec<LayoutNode>) -> LayoutNode {
    LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Px(width).into(),
                ..Default::default()
            },
            ..Default::default()
        },
        children,
    )
}

#[test]
fn left_float_is_placed_at_left_edge_and_does_not_advance_flow() {
    let mut root = root_with(
        400.0,
        vec![
            floated(100.0, 50.0, Float::Left),
            fixed_block(200.0, 30.0, Clear::None),
        ],
    );

    LayoutEngine::layout(&mut root, 400.0, 600.0);

    let float_box = border_box_of(node(&root, 0));
    assert_eq!((float_box.x, float_box.y), (0.0, 0.0));
    assert_eq!((float_box.width, float_box.height), (100.0, 50.0));

    // The following block overlaps the float instead of being pushed below it.
    let block = border_box_of(node(&root, 1));
    assert_eq!((block.x, block.y), (0.0, 0.0));
}

#[test]
fn two_left_floats_stack_side_by_side() {
    let mut root = root_with(
        400.0,
        vec![
            floated(100.0, 50.0, Float::Left),
            floated(120.0, 50.0, Float::Left),
        ],
    );

    LayoutEngine::layout(&mut root, 400.0, 600.0);

    assert_eq!(border_box_of(node(&root, 0)).x, 0.0);
    let second = border_box_of(node(&root, 1));
    assert_eq!((second.x, second.y), (100.0, 0.0));
}

#[test]
fn right_float_is_aligned_to_right_edge() {
    let mut root = root_with(400.0, vec![floated(100.0, 50.0, Float::Right)]);

    LayoutEngine::layout(&mut root, 400.0, 600.0);

    let float_box = border_box_of(node(&root, 0));
    assert_eq!((float_box.x, float_box.width), (300.0, 100.0));
}

#[test]
fn clear_moves_block_below_left_float() {
    let mut root = root_with(
        400.0,
        vec![
            floated(100.0, 50.0, Float::Left),
            fixed_block(200.0, 30.0, Clear::Both),
        ],
    );

    LayoutEngine::layout(&mut root, 400.0, 600.0);

    let block = border_box_of(node(&root, 1));
    assert_eq!(block.y, 50.0);
}

#[test]
fn clear_on_float_drops_it_below_previous_float() {
    let mut second = floated(200.0, 40.0, Float::Left);
    second.style.clear = Clear::Left;
    let mut root = root_with(400.0, vec![floated(100.0, 50.0, Float::Left), second]);

    LayoutEngine::layout(&mut root, 400.0, 600.0);

    let second_box = border_box_of(node(&root, 1));
    assert_eq!((second_box.x, second_box.y), (0.0, 50.0));
}

#[test]
fn float_contributes_to_parent_content_height() {
    let mut root = root_with(400.0, vec![floated(100.0, 70.0, Float::Left)]);

    LayoutEngine::layout(&mut root, 400.0, 600.0);

    assert!(approx_eq(block_box(&root).border_box.height, 70.0));
}

#[test]
fn float_margins_offset_the_border_box() {
    let mut float = floated(80.0, 40.0, Float::Left);
    float.style.spacing = Spacing {
        margin_left: Length::Px(10.0).into(),
        margin_top: Length::Px(5.0).into(),
        margin_right: Length::Px(6.0).into(),
        margin_bottom: Length::Px(7.0).into(),
        ..Default::default()
    };
    let mut root = root_with(400.0, vec![float]);

    LayoutEngine::layout(&mut root, 400.0, 600.0);

    let float_box = border_box_of(node(&root, 0));
    assert_eq!((float_box.x, float_box.y), (10.0, 5.0));
    // The float's margin box (5 + 40 + 7) drives the parent's height.
    assert!(approx_eq(block_box(&root).border_box.height, 52.0));
}

#[test]
fn inline_content_wraps_around_left_float() {
    let inline = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        vec![
            fragment(150.0, 20.0),
            fragment(150.0, 20.0),
            fragment(150.0, 20.0),
        ],
    );
    let mut root = root_with(400.0, vec![floated(100.0, 50.0, Float::Left), inline]);

    LayoutEngine::layout(&mut root, 400.0, 600.0);

    let spans = &inline_box(node(&root, 1)).line_spans;
    assert!(!spans.is_empty());

    // Every line begins at or after the float's right edge (x = 100).
    for span in spans {
        assert!(
            span.line_pos.0 >= 100.0 - 0.01,
            "line placed left of the float: {span:?}"
        );
        assert!(span.line_pos.0 < 400.0);
    }

    // The third fragment does not fit beside the float and wraps to line 1.
    let max_line = spans.iter().map(|s| s.line_index).max().unwrap();
    assert!(max_line >= 1, "content did not wrap below the float");
}

#[test]
fn nested_flow_line_wraps_around_ancestor_float() {
    let inner_inline = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            ..Default::default()
        },
        vec![fragment(300.0, 20.0)],
    );
    let nested = LayoutNode::with_children(
        Style {
            ..Default::default()
        },
        vec![inner_inline],
    );
    let mut root = root_with(400.0, vec![floated(100.0, 80.0, Float::Left), nested]);

    LayoutEngine::layout(&mut root, 400.0, 600.0);

    let spans = &inline_box(node(&root, 1).children[0].node().unwrap()).line_spans;
    assert!(!spans.is_empty());
    assert!(
        spans[0].line_pos.0 >= 100.0 - 0.01,
        "nested line did not wrap around the ancestor float: {spans:?}"
    );
}

#[test]
fn flow_root_isolates_descendants_from_ancestor_float() {
    let inner_inline = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            ..Default::default()
        },
        vec![fragment(300.0, 20.0)],
    );
    let isolated = LayoutNode::with_children(
        Style {
            display: Display::parse("flow-root").unwrap(),
            ..Default::default()
        },
        vec![inner_inline],
    );
    let mut root = root_with(400.0, vec![floated(100.0, 80.0, Float::Left), isolated]);

    LayoutEngine::layout(&mut root, 400.0, 600.0);

    let spans = &inline_box(node(&root, 1).children[0].node().unwrap()).line_spans;
    assert!(!spans.is_empty());
    assert_eq!(
        spans[0].line_pos.0, 0.0,
        "flow-root should not wrap around the ancestor float"
    );
}

#[test]
fn float_that_does_not_fit_drops_below_earlier_float() {
    // A 300px left float in a 400px container leaves only 100px, so the second
    // 300px float must move below the first rather than overflow/overlap it.
    let mut root = root_with(
        400.0,
        vec![
            floated(300.0, 50.0, Float::Left),
            floated(300.0, 40.0, Float::Left),
        ],
    );

    LayoutEngine::layout(&mut root, 400.0, 600.0);

    let first = border_box_of(node(&root, 0));
    let second = border_box_of(node(&root, 1));
    assert_eq!((first.x, first.y), (0.0, 0.0));
    assert_eq!((second.x, second.y), (0.0, 50.0));
}

#[test]
fn opposite_side_floats_do_not_overlap() {
    // A right float leaves only 150px on the left; the 250px left float drops
    // below it instead of overlapping.
    let mut root = root_with(
        400.0,
        vec![
            floated(250.0, 50.0, Float::Right),
            floated(250.0, 40.0, Float::Left),
        ],
    );

    LayoutEngine::layout(&mut root, 400.0, 600.0);

    let right = border_box_of(node(&root, 0));
    let left = border_box_of(node(&root, 1));
    assert_eq!((right.x, right.y), (150.0, 0.0));
    assert_eq!(left.y, 50.0);
    assert!(
        left.y >= right.y + right.height - 0.01,
        "left float overlaps right float: {left:?} vs {right:?}"
    );
}

#[test]
fn later_float_is_not_placed_above_earlier_dropped_float() {
    let mut root = root_with(
        400.0,
        vec![
            floated(100.0, 50.0, Float::Left),
            floated(400.0, 20.0, Float::Left),
            floated(50.0, 10.0, Float::Left),
        ],
    );

    LayoutEngine::layout(&mut root, 400.0, 600.0);

    let b = border_box_of(node(&root, 1));
    let c = border_box_of(node(&root, 2));
    assert_eq!(b.y, 50.0);
    assert!(
        c.y >= b.y - 0.01,
        "third float ({}) was placed above the second ({})",
        c.y,
        b.y
    );
}

#[test]
fn inline_content_after_dropped_float_still_wraps_from_the_top() {
    // A dropped float must not push following inline content down: the first
    // line wraps around the first float and stays at the top of the flow.
    let inline = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        vec![fragment(50.0, 20.0)],
    );
    let mut root = root_with(
        400.0,
        vec![
            floated(300.0, 40.0, Float::Left),
            floated(300.0, 30.0, Float::Left),
            inline,
        ],
    );

    LayoutEngine::layout(&mut root, 400.0, 600.0);

    let spans = &inline_box(node(&root, 2)).line_spans;
    assert!(!spans.is_empty());
    assert_eq!(spans[0].line_pos.1, 0.0, "line was pushed below the floats");
    assert!(spans[0].line_pos.0 >= 300.0 - 0.01);
}

#[derive(Debug)]
struct RecordingCustom {
    recorded: std::rc::Rc<std::cell::Cell<(f32, f32, f32, f32)>>,
}

impl RecordingCustom {
    fn new(recorded: std::rc::Rc<std::cell::Cell<(f32, f32, f32, f32)>>) -> Self {
        Self { recorded }
    }
}

impl CustomLayouter for RecordingCustom {
    fn layout(&mut self, ctx: &LayoutContext) -> LayoutBox {
        let (avail_start, avail_end) = ctx
            .float_space
            .as_ref()
            .map(|space| space.avail_at(0.0))
            .unwrap_or((0.0, 0.0));
        self.recorded.set((
            ctx.start_pos.0,
            ctx.available_inline_size,
            avail_start,
            avail_end,
        ));
        let (x, y) = ctx.start_pos;
        let width = 20.0;
        let box_model = BoxModel::from(rect(x, y, width, 10.0));
        LayoutBox::InlineBox(InlineBox {
            box_model,
            line_spans: vec![LineSpan {
                x_range: x..x + width,
                line_pos: (x, y),
                line_index: 0,
            }],
        })
    }

    fn measure(&self, _ctx: &LayoutContext) -> MeasureResult {
        MeasureResult {
            width: 20.0,
            height: 10.0,
        }
    }
}

#[test]
fn custom_inline_layouter_is_handed_the_float_free_span() {
    let recorded = std::rc::Rc::new(std::cell::Cell::new((0.0, 0.0, 0.0, 0.0)));
    let custom = custom_inline(RecordingCustom::new(recorded.clone()));
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Px(400.0).into(),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![LayoutChild::from(floated(100.0, 50.0, Float::Left)), custom],
    );

    LayoutEngine::layout(&mut root, 400.0, 600.0);

    let (start_x, available, avail_start, avail_end) = recorded.get();
    // The line begins after the float (x = 100) and only 300px remain.
    assert_eq!(start_x, 100.0);
    assert_eq!(available, 300.0);
    // The float-space query is relative to the containing block, not the line
    // start: the float's right edge is at x = 100, not x = 0.
    assert_eq!(avail_start, 100.0);
    assert_eq!(avail_end, 400.0);
}
