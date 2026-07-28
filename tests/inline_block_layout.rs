mod common;
use common::*;
use ui_layout::*;

fn inline_block_style() -> Style {
    Style {
        display: Display::parse("inline-block").unwrap(),
        ..Default::default()
    }
}

// --- Parsing & formatting ---

#[test]
fn display_parsing() {
    let d = Display::parse("inline-block").unwrap();
    assert_eq!(d.outer, OuterDisplay::Inline);
    assert_eq!(d.inner, InnerDisplay::FlowRoot);

    let d2 = Display::parse("inline flow-root").unwrap();
    assert_eq!(d2.outer, OuterDisplay::Inline);
    assert_eq!(d2.inner, InnerDisplay::FlowRoot);
}

#[test]
fn display_formatting() {
    let d = Display {
        outer: OuterDisplay::Inline,
        inner: InnerDisplay::FlowRoot,
    };
    assert_eq!(format!("{}", d), "inline-block");
}

// --- Basic layout: inline-block in block parent ---

#[test]
fn basic_block_children() {
    let inline_block =
        LayoutNode::with_children(inline_block_style(), vec![new_child(30.0), new_child(40.0)]);

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inline_block],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // The inline-block should be an InlineBox
    let ib = inline_box_model(node(&root, 0));
    // Children stack vertically: 30 + 40 = 70
    assert_eq!(ib.content_box.height, 70.0);
}

// --- No margin collapsing (flow-root behavior) ---

struct InlineBlockMarginCase {
    name: &'static str,
    child_mt: f32,
    child_mb: f32,
    expected_height: f32,
}

#[test]
fn no_parent_child_margin_collapse() {
    let cases = [
        InlineBlockMarginCase {
            name: "mt only",
            child_mt: 20.0,
            child_mb: 0.0,
            expected_height: 20.0 + 50.0,
        },
        InlineBlockMarginCase {
            name: "mb only",
            child_mt: 0.0,
            child_mb: 20.0,
            expected_height: 50.0 + 20.0,
        },
        InlineBlockMarginCase {
            name: "mt + mb",
            child_mt: 10.0,
            child_mb: 15.0,
            expected_height: 10.0 + 50.0 + 15.0,
        },
    ];

    for case in &cases {
        let inline_block = LayoutNode::with_children(
            inline_block_style(),
            vec![block_child(50.0, case.child_mt, case.child_mb)],
        );

        let mut root = LayoutNode::with_children(
            Style {
                size: SizeStyle {
                    width: LengthOrAuto::Length(Length::Px(200.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            vec![inline_block],
        );

        LayoutEngine::layout(&mut root, 800.0, 600.0);

        let ib = inline_box_model(node(&root, 0));
        assert_eq!(
            ib.content_box.height, case.expected_height,
            "[{}] content height",
            case.name
        );
    }
}

#[test]
fn no_sibling_margin_collapse() {
    let inline_block = LayoutNode::with_children(
        inline_block_style(),
        vec![block_child(30.0, 0.0, 20.0), block_child(40.0, 15.0, 0.0)],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inline_block],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let ib = inline_box_model(node(&root, 0));
    // Both margins preserved: 30 + 20 + 15 + 40 = 105
    assert_eq!(ib.content_box.height, 30.0 + 20.0 + 15.0 + 40.0);
}

// --- Inline positioning within block ---

#[test]
fn sits_inline_in_text_flow() {
    let inline_block = LayoutNode::with_children(
        Style {
            display: Display::parse("inline-block").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        vec![new_child(30.0)],
    );

    let frag_before = fragment(40.0, 10.0);

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![
            LayoutChild::from(frag_before),
            LayoutChild::from(inline_block),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let ib = node(&root, 1);
    // Should be an InlineBox positioned after the fragment
    let boxes: Vec<BoxModel> = ib.layout_box.iter().collect();
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0].border_box.x, 40.0);
    assert_eq!(boxes[0].border_box.height, 30.0);
}

// --- Explicit sizing ---

#[test]
fn respects_explicit_width() {
    let inline_block = LayoutNode::with_children(
        Style {
            display: Display::parse("inline-block").unwrap(),
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![new_child(30.0)],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inline_block],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let ib = inline_box_model(node(&root, 0));
    assert_eq!(ib.content_box.width, 100.0);
}

// --- Padding and border ---

#[test]
fn padding_and_border_applied() {
    let inline_block = LayoutNode::with_children(
        Style {
            display: Display::parse("inline-block").unwrap(),
            spacing: Spacing {
                padding_top: Length::Px(5.0),
                padding_bottom: Length::Px(5.0),
                border_top: Length::Px(2.0),
                border_bottom: Length::Px(2.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![new_child(30.0)],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inline_block],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let ib = inline_box_model(node(&root, 0));
    // border(2) + padding(5) + child(30) + padding(5) + border(2) = 44
    assert_eq!(ib.content_box.height, 30.0);
    assert_eq!(ib.border_box.height, 2.0 + 5.0 + 30.0 + 5.0 + 2.0);
}

// --- Multiple inline-block siblings flow inline ---

#[test]
fn multiple_inline_blocks_flow_inline() {
    let ib1 = LayoutNode::with_children(
        Style {
            display: Display::parse("inline-block").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        vec![new_child(30.0)],
    );

    let ib2 = LayoutNode::with_children(
        Style {
            display: Display::parse("inline-block").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        vec![new_child(25.0)],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(400.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![ib1, ib2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let boxes1: Vec<BoxModel> = node(&root, 0).layout_box.iter().collect();
    let boxes2: Vec<BoxModel> = node(&root, 1).layout_box.iter().collect();

    assert_eq!(boxes1.len(), 1);
    assert_eq!(boxes2.len(), 1);
    assert_eq!(boxes1[0].border_box.x, 0.0);
    assert_eq!(boxes2[0].border_box.x, 0.0);
    assert_eq!(boxes2[0].border_box.y, 0.0);
}

// --- Empty inline-block ---

#[test]
fn empty() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![LayoutNode::new(inline_block_style())],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);
    let ib = inline_box_model(node(&root, 0));
    assert_eq!(ib.content_box.height, 0.0);
}

// --- Comparison with inline (flow) ---

#[test]
fn inline_block_vs_inline_margin_difference() {
    // inline-block (flow-root): child margin stays inside, height = mt(20) + child(50) = 70
    let ib_parent = LayoutNode::with_children(
        Style {
            display: Display::parse("inline-block").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        vec![block_child(50.0, 20.0, 0.0)],
    );

    // flow-root (block): same behavior, child margin stays inside
    let fr_parent = LayoutNode::with_children(
        Style {
            display: Display::parse("flow-root").unwrap(),
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![block_child(50.0, 20.0, 0.0)],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(400.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![ib_parent, fr_parent],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // inline-block: no collapse, height = 20 + 50 = 70
    let ib_boxes: Vec<BoxModel> = node(&root, 0).layout_box.iter().collect();
    assert_eq!(ib_boxes.len(), 1);
    assert_eq!(ib_boxes[0].border_box.height, 70.0);

    // flow-root (block): same, height = 20 + 50 = 70
    assert_eq!(block_box(node(&root, 1)).content_box.height, 70.0);
}

// --- Fragments inside inline-block ---

#[test]
fn with_fragments() {
    let inline_block = LayoutNode::with_children(
        Style {
            display: Display::parse("inline-block").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        vec![
            LayoutChild::from(fragment(50.0, 10.0)),
            LayoutChild::from(fragment(60.0, 10.0)),
        ],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inline_block],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let ib = inline_box_model(node(&root, 0));
    // Both on same line (50 + 60 = 110 < 200)
    assert_eq!(ib.content_box.height, 20.0);
}
