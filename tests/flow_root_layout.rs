mod common;
use common::*;
use ui_layout::*;

fn flow_root_style() -> Style {
    Style {
        display: Display::parse("flow-root").unwrap(),
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(200.0)),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn display_parsing() {
    let d = Display::parse("flow-root").unwrap();
    assert_eq!(d.outer, OuterDisplay::Block);
    assert_eq!(d.inner, InnerDisplay::FlowRoot);

    let d2 = Display::parse("block flow-root").unwrap();
    assert_eq!(d2.outer, OuterDisplay::Block);
    assert_eq!(d2.inner, InnerDisplay::FlowRoot);
}

#[test]
fn display_formatting() {
    let d = Display {
        outer: OuterDisplay::Block,
        inner: InnerDisplay::FlowRoot,
    };
    assert_eq!(format!("{}", d), "flow-root");
    assert_eq!(format!("{}", InnerDisplay::FlowRoot), "flow-root");
}

#[test]
fn outer_is_block() {
    let mut root = LayoutNode::with_children(flow_root_style(), vec![new_child(30.0, 0.0)]);
    LayoutEngine::layout(&mut root, 800.0, 600.0);
    assert_eq!(block_box(&root).content_box.width, 200.0);
}

#[test]
fn basic_block_children() {
    let mut root = LayoutNode::with_children(
        flow_root_style(),
        vec![new_child(30.0, 0.0), new_child(40.0, 0.0)],
    );
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert_eq!(b.content_box.width, 200.0);
    assert_eq!(b.content_box.height, 30.0 + 40.0);

    let c0 = node(&root, 0);
    let c1 = node(&root, 1);
    assert_eq!(block_box(c0).border_box.y, 0.0);
    assert_eq!(block_box(c0).border_box.height, 30.0);
    assert_eq!(block_box(c1).border_box.y, 30.0);
    assert_eq!(block_box(c1).border_box.height, 40.0);
}

#[test]
fn empty() {
    let mut root = LayoutNode::new(flow_root_style());
    LayoutEngine::layout(&mut root, 800.0, 600.0);
    assert_eq!(block_box(&root).content_box.height, 0.0);
}

// --- Parent-child margin collapsing: flow vs flow-root ---

struct ParentChildCase {
    name: &'static str,
    child_mt: f32,
    child_mb: f32,
    flow_height: f32,
    fr_height: f32,
    fr_child_y: f32,
}

#[test]
fn parent_child_margin_collapse() {
    // flow: child margin collapses with parent (no border/padding)
    // flow-root: child margin stays inside
    let cases = [
        ParentChildCase {
            name: "mt only",
            child_mt: 20.0,
            child_mb: 0.0,
            flow_height: 50.0,
            fr_height: 20.0 + 50.0,
            fr_child_y: 20.0,
        },
        ParentChildCase {
            name: "mb only",
            child_mt: 0.0,
            child_mb: 20.0,
            flow_height: 50.0,
            fr_height: 50.0 + 20.0,
            fr_child_y: 0.0,
        },
        ParentChildCase {
            name: "mt + mb",
            child_mt: 10.0,
            child_mb: 15.0,
            flow_height: 50.0,
            fr_height: 10.0 + 50.0 + 15.0,
            fr_child_y: 10.0,
        },
    ];

    for case in &cases {
        // --- flow ---
        let mut flow_root = LayoutNode::with_children(
            Style {
                size: SizeStyle {
                    width: LengthOrAuto::Length(Length::Px(200.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            vec![block_child(50.0, case.child_mt, case.child_mb)],
        );
        LayoutEngine::layout(&mut flow_root, 800.0, 600.0);
        let flow_h = block_box(&flow_root).content_box.height;

        // --- flow-root ---
        let mut fr_root = LayoutNode::with_children(
            flow_root_style(),
            vec![block_child(50.0, case.child_mt, case.child_mb)],
        );
        LayoutEngine::layout(&mut fr_root, 800.0, 600.0);
        let fr_h = block_box(&fr_root).content_box.height;
        let fr_child = node(&fr_root, 0);

        assert_eq!(
            flow_h, case.flow_height,
            "[{}] flow content_height",
            case.name
        );
        assert_eq!(
            fr_h, case.fr_height,
            "[{}] flow-root content_height",
            case.name
        );
        assert_eq!(
            block_box(fr_child).border_box.y,
            case.fr_child_y,
            "[{}] flow-root child y",
            case.name
        );
    }
}

// --- Sibling margin collapsing: flow vs flow-root ---

struct SiblingCase {
    name: &'static str,
    h1: f32,
    mb1: f32,
    h2: f32,
    mt2: f32,
    flow_height: f32,
    fr_height: f32,
    fr_c1_y: f32,
    fr_c2_y: f32,
}

#[test]
fn sibling_margin_collapse() {
    // flow: adjacent margins collapse (max of two)
    // flow-root: both margins preserved (sum)
    let cases = [
        SiblingCase {
            name: "equal margins",
            h1: 30.0,
            mb1: 20.0,
            h2: 40.0,
            mt2: 20.0,
            flow_height: 30.0 + 20.0 + 40.0,
            fr_height: 30.0 + 20.0 + 20.0 + 40.0,
            fr_c1_y: 0.0,
            fr_c2_y: 30.0 + 20.0 + 20.0,
        },
        SiblingCase {
            name: "unequal margins",
            h1: 30.0,
            mb1: 20.0,
            h2: 40.0,
            mt2: 15.0,
            flow_height: 30.0 + 20.0 + 40.0,
            fr_height: 30.0 + 20.0 + 15.0 + 40.0,
            fr_c1_y: 0.0,
            fr_c2_y: 30.0 + 20.0 + 15.0,
        },
        SiblingCase {
            name: "second larger",
            h1: 30.0,
            mb1: 10.0,
            h2: 40.0,
            mt2: 25.0,
            flow_height: 30.0 + 25.0 + 40.0,
            fr_height: 30.0 + 10.0 + 25.0 + 40.0,
            fr_c1_y: 0.0,
            fr_c2_y: 30.0 + 10.0 + 25.0,
        },
    ];

    for case in &cases {
        // --- flow ---
        let mut flow_root = LayoutNode::with_children(
            Style {
                size: SizeStyle {
                    width: LengthOrAuto::Length(Length::Px(200.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            vec![
                block_child(case.h1, 0.0, case.mb1),
                block_child(case.h2, case.mt2, 0.0),
            ],
        );
        LayoutEngine::layout(&mut flow_root, 800.0, 600.0);
        let flow_h = block_box(&flow_root).content_box.height;

        // --- flow-root ---
        let mut fr_root = LayoutNode::with_children(
            flow_root_style(),
            vec![
                block_child(case.h1, 0.0, case.mb1),
                block_child(case.h2, case.mt2, 0.0),
            ],
        );
        LayoutEngine::layout(&mut fr_root, 800.0, 600.0);
        let fr_h = block_box(&fr_root).content_box.height;
        let fr_c1 = node(&fr_root, 0);
        let fr_c2 = node(&fr_root, 1);

        assert_eq!(
            flow_h, case.flow_height,
            "[{}] flow content_height",
            case.name
        );
        assert_eq!(
            fr_h, case.fr_height,
            "[{}] flow-root content_height",
            case.name
        );
        assert_eq!(
            block_box(fr_c1).border_box.y,
            case.fr_c1_y,
            "[{}] flow-root child1 y",
            case.name
        );
        assert_eq!(
            block_box(fr_c2).border_box.y,
            case.fr_c2_y,
            "[{}] flow-root child2 y",
            case.name
        );
    }
}

// --- Border/padding blocks collapsing (same as regular flow) ---

#[test]
fn border_blocks_collapse_like_flow() {
    // When parent has border/padding, margin collapsing is already blocked
    // in regular flow. flow-root behaves the same.
    let root = LayoutNode::with_children(
        Style {
            display: Display::parse("flow-root").unwrap(),
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            spacing: Spacing {
                border_top: Length::Px(5.0),
                padding_top: Length::Px(10.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![block_child(50.0, 20.0, 0.0)],
    );

    let mut outer = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![root],
    );

    LayoutEngine::layout(&mut outer, 800.0, 600.0);

    let fr = node(&outer, 0);
    let fr_box = block_box(fr);

    // border(5) + padding(10) + margin_top(20) + child(50) = 85
    assert_eq!(fr_box.border_box.height, 5.0 + 10.0 + 20.0 + 50.0);
    // content_box = margin_top(20) + child(50) = 70
    assert_eq!(fr_box.content_box.height, 20.0 + 50.0);

    let c = node(fr, 0);
    assert_eq!(block_box(c).border_box.y, 20.0);
}

// --- Nested flow-root isolates margin collapsing ---

#[test]
fn nested_isolates_margin_collapse() {
    let inner = LayoutNode::with_children(
        Style {
            display: Display::parse("flow-root").unwrap(),
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![block_child(40.0, 10.0, 10.0)],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![new_child(20.0, 0.0), inner, new_child(20.0, 0.0)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let c0 = node(&root, 0);
    let fr = node(&root, 1);
    let c2 = node(&root, 2);

    assert_eq!(block_box(c0).border_box.y, 0.0);
    assert_eq!(block_box(c0).border_box.height, 20.0);

    // inner: 10 + 40 + 10 = 60
    assert_eq!(block_box(fr).border_box.y, 20.0);
    assert_eq!(block_box(fr).content_box.height, 10.0 + 40.0 + 10.0);

    assert_eq!(block_box(c2).border_box.y, 20.0 + 60.0);
    assert_eq!(block_box(c2).border_box.height, 20.0);

    assert_eq!(block_box(&root).content_box.height, 20.0 + 60.0 + 20.0);
}

// --- Inline children ---

#[test]
fn with_fragments() {
    let mut root = LayoutNode::with_children(
        Style {
            display: Display::parse("flow-root").unwrap(),
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        vec![
            LayoutChild::Fragment(FragmentNode::new(ItemFragment::Fragment(Fragment {
                width: 50.0,
                height: 20.0,
            }))),
            LayoutChild::Fragment(FragmentNode::new(ItemFragment::Fragment(Fragment {
                width: 60.0,
                height: 20.0,
            }))),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    // Both on same line (50 + 60 = 110 < 200)
    assert_eq!(b.content_box.height, 20.0);
}

// --- flow-root as child of block ---

#[test]
fn as_child_of_block() {
    let fr = LayoutNode::with_children(flow_root_style(), vec![block_child(40.0, 10.0, 0.0)]);

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![fr],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let fr_node = node(&root, 0);
    assert_eq!(block_box(fr_node).content_box.height, 10.0 + 40.0);

    let c = node(fr_node, 0);
    assert_eq!(block_box(c).border_box.y, 10.0);
}

// --- flow-root with padding and border ---

#[test]
fn with_padding_and_border() {
    let mut root = LayoutNode::with_children(
        Style {
            display: Display::parse("flow-root").unwrap(),
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            spacing: Spacing {
                padding_top: Length::Px(10.0),
                padding_bottom: Length::Px(10.0),
                border_top: Length::Px(2.0),
                border_bottom: Length::Px(2.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![new_child(30.0, 0.0), new_child(40.0, 0.0)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    // border(2) + padding(10) + child1(30) + child2(40) + padding(10) + border(2) = 94
    assert_eq!(b.border_box.height, 2.0 + 10.0 + 30.0 + 40.0 + 10.0 + 2.0);
    assert_eq!(b.content_box.height, 30.0 + 40.0);
}
