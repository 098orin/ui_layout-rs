mod common;
use common::*;
use ui_layout::*;

// --- Box model ---

#[test]
fn block_basic_box_model() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(200.0)),
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(10.0),
            padding_top: Length::Px(5.0),
            padding_bottom: Length::Px(5.0),
            border_left: Length::Px(2.0),
            border_right: Length::Px(2.0),
            border_top: Length::Px(1.0),
            border_bottom: Length::Px(1.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert_eq!(b.content_box.width, 200.0);
    assert_eq!(b.content_box.height, 100.0);
    assert_eq!(b.padding_box.width, 220.0);
    assert_eq!(b.padding_box.height, 110.0);
    assert_eq!(b.border_box.width, 224.0);
    assert_eq!(b.border_box.height, 112.0);
}

#[test]
fn border_box_sizing() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(200.0)),
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        box_sizing: BoxSizing::BorderBox,
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(15.0),
            padding_top: Length::Px(5.0),
            padding_bottom: Length::Px(8.0),
            border_left: Length::Px(2.0),
            border_right: Length::Px(3.0),
            border_top: Length::Px(1.0),
            border_bottom: Length::Px(4.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert_eq!(b.border_box.width, 200.0);
    assert_eq!(b.border_box.height, 100.0);
    assert_eq!(b.padding_box.width, 195.0);
    assert_eq!(b.padding_box.height, 95.0);
    assert_eq!(b.content_box.width, 170.0);
    assert_eq!(b.content_box.height, 82.0);
}

// --- Auto height ---

#[test]
fn block_auto_height_from_children() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [new_child(40.0, 0.0)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).content_box.height, 40.0);
    assert_eq!(block_box(&root).content_box.height, 40.0);
}

#[test]
fn auto_height_with_multiple_children() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [
            new_child(30.0, 0.0),
            new_child(50.0, 0.0),
            new_child(20.0, 0.0),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(&root).content_box.height, 100.0);
    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 30.0);
    assert_eq!(block_box(node(&root, 2)).border_box.y, 80.0);
}

#[test]
fn auto_height_with_padding_and_margins() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(5.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(5.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(10.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            spacing: Spacing {
                padding_top: Length::Px(8.0),
                padding_bottom: Length::Px(8.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let root_box = block_box(&root);
    assert_eq!(root_box.content_box.height, 75.0);
    assert_eq!(root_box.border_box.height, 91.0);
    assert_eq!(root_box.content_box.y, 8.0);
}

#[test]
fn block_child_with_padding_followed_by_block_node() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Auto,
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            padding_top: Length::Px(10.0),
            padding_bottom: Length::Px(10.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Auto,
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 50.0);
    assert_eq!(block_box(&root).content_box.height, 100.0);
}

// --- Fragment-based height ---

#[test]
fn block_auto_height_from_direct_fragments_single_line() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(800.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            line_height: Length::Px(38.4),
            ..Default::default()
        },
        [fragment(79.578125, 38.4)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert_eq!(b.content_box.height, 38.4);
    assert!(b.content_box.width >= 79.578125);
}

#[test]
fn block_auto_height_from_direct_fragments_multi_line() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [
            fragment(60.0, 10.0),
            fragment(50.0, 10.0),
            fragment(30.0, 10.0),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // 60px wraps, then 50+30 fit on second line = 2 lines * 20px
    let b = block_box(&root);
    assert_eq!(b.content_box.height, 40.0);
}

// --- Margins ---

#[test]
fn block_margin_auto_centering() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Auto,
            margin_right: LengthOrAuto::Auto,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 100.0);
}

#[test]
fn margins_affect_positioning() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Length(Length::Px(20.0)),
            margin_top: LengthOrAuto::Length(Length::Px(10.0)),
            margin_right: LengthOrAuto::Length(Length::Px(30.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let inner = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    let mut root = LayoutNode::with_children(Style::default(), [inner]);
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let c = node(&root.children[0].node().unwrap(), 0);
    let b = block_box(c);
    assert_eq!(b.border_box.x, 20.0);
    // margin-top collapses with parent (no border/padding) => child at y=0
    assert_eq!(b.border_box.y, 0.0);
}

// --- Margin collapsing ---

#[test]
fn block_vertical_margin_collapsing_between_siblings() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let c1 = block_box(node(&root, 0));
    let c2 = block_box(node(&root, 1));

    assert_eq!(c1.border_box.y, 0.0);
    assert_eq!(c2.border_box.y, c1.border_box.height + 30.0);
}

#[test]
fn three_siblings_with_margin_collapse() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });
    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(5.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        ..Default::default()
    });
    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(8.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 30.0);
    assert_eq!(block_box(node(&root, 2)).border_box.y, 75.0);
    assert_eq!(block_box(&root).content_box.height, 85.0);
}

// --- Parent-child margin collapsing ---

#[test]
fn parent_child_top_margin_collapse() {
    // Parent has no border/padding, child has margin-top
    // Child should be at y=0 and parent's effective margin absorbs child's margin
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut parent = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut parent, 800.0, 600.0);

    // Child should be at y=0 (margin-top collapsed)
    assert_eq!(block_box(node(&parent, 0)).border_box.y, 0.0);
    // Parent height should exclude the child's collapsed margin-top
    assert_eq!(block_box(&parent).content_box.height, 50.0);
}

#[test]
fn parent_child_top_margin_collapse_blocked_by_border() {
    // Parent has border-top, so child's margin-top should NOT collapse
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut parent = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            spacing: Spacing {
                border_top: Length::Px(1.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut parent, 800.0, 600.0);

    // Child should be shifted by its margin-top (no collapse)
    assert_eq!(block_box(node(&parent, 0)).border_box.y, 30.0);
    // Parent height should include child's margin-top
    assert_eq!(block_box(&parent).content_box.height, 80.0);
}

#[test]
fn parent_child_top_margin_collapse_blocked_by_padding() {
    // Parent has padding-top, so child's margin-top should NOT collapse
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut parent = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            spacing: Spacing {
                padding_top: Length::Px(1.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut parent, 800.0, 600.0);

    // Child should be shifted by its margin-top (no collapse)
    assert_eq!(block_box(node(&parent, 0)).border_box.y, 30.0);
}

#[test]
fn parent_child_bottom_margin_collapse() {
    // Parent has no border/padding, child has margin-bottom
    // Child's margin-bottom should extrude below parent
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut parent = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut parent, 800.0, 600.0);

    // Parent height should NOT include child's margin-bottom
    assert_eq!(block_box(&parent).content_box.height, 50.0);
}

#[test]
fn parent_child_bottom_margin_collapse_blocked_by_border() {
    // Parent has border-bottom, child's margin-bottom should be included in height
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut parent = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            spacing: Spacing {
                border_bottom: Length::Px(1.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut parent, 800.0, 600.0);

    // Parent height SHOULD include child's margin-bottom (border blocks collapse)
    assert_eq!(block_box(&parent).content_box.height, 70.0);
}

#[test]
fn nested_margin_collapse_top() {
    // Grandparent -> parent -> child, all without border/padding.
    // Child's margin-top should propagate through parent to grandparent.
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(40.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let parent = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    let mut grandparent = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [parent],
    );

    LayoutEngine::layout(&mut grandparent, 800.0, 600.0);

    // All margins collapsed: child at y=0 inside parent, parent at y=0 inside grandparent
    let parent_node = node(&grandparent, 0);
    let child_node = node(parent_node, 0);

    assert_eq!(block_box(child_node).border_box.y, 0.0);
    assert_eq!(block_box(parent_node).border_box.y, 0.0);
    // Grandparent content height = child height (margins extruded above)
    assert_eq!(block_box(&grandparent).content_box.height, 30.0);
}

#[test]
fn parent_child_collapse_with_sibling_margins() {
    // First child's margin-top collapses with parent; sibling collapsing still works
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(30.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut parent = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut parent, 800.0, 600.0);

    // child1: margin-top (30) collapsed with parent => at y=0
    assert_eq!(block_box(node(&parent, 0)).border_box.y, 0.0);
    // child2: sibling margin collapsed (max(child1.mb=10, child2.mt=0) = 10)
    assert_eq!(block_box(node(&parent, 1)).border_box.y, 30.0);
    // Parent height: child2 bottom (60) + child2 margin-bottom (15) but bottom collapse
    // => 60 (child2 bottom) because margin-bottom extruded
    assert_eq!(block_box(&parent).content_box.height, 60.0);
}

#[test]
fn parent_own_margin_collapses_with_child_margin() {
    // Parent has its own margin-top. Child's margin-top should collapse with parent's.
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(40.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let parent = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            spacing: Spacing {
                margin_top: LengthOrAuto::Length(Length::Px(20.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    // Give root a border so it blocks margin collapsing upward
    let mut root = LayoutNode::with_children(
        Style {
            spacing: Spacing {
                border_top: Length::Px(1.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [parent],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Parent's effective margin-top = max(20, 40) = 40
    // Parent should be positioned at y=40 (collapsed margin)
    let parent_node = node(&root, 0);
    assert_eq!(block_box(parent_node).border_box.y, 40.0);
    // Child at y=0 inside parent (margin collapsed)
    assert_eq!(block_box(node(parent_node, 0)).border_box.y, 0.0);
    // Parent content height = 50 (child height only)
    assert_eq!(block_box(parent_node).content_box.height, 50.0);
}

// --- BoxModel direct accessors ---

#[test]
fn box_model_width_height_accessors() {
    let bm = BoxModel {
        sticky_edges: None,
        border_box: Rect {
            x: 5.0,
            y: 10.0,
            width: 200.0,
            height: 100.0,
        },
        padding_box: Rect {
            x: 7.0,
            y: 12.0,
            width: 196.0,
            height: 96.0,
        },
        content_box: Rect {
            x: 9.0,
            y: 14.0,
            width: 192.0,
            height: 92.0,
        },
        children_box: Rect {
            x: 9.0,
            y: 14.0,
            width: 192.0,
            height: 92.0,
        },
    };
    assert_eq!(bm.width(), 200.0);
    assert_eq!(bm.height(), 100.0);
}

// --- Edge cases ---

#[test]
fn block_empty_node() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert_eq!(b.content_box.width, 100.0);
    assert_eq!(b.content_box.height, 50.0);
}

#[test]
fn block_with_flex_child_followed_by_block_with_margin() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Flex,
        },
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        flex_direction: FlexDirection::Row,
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(40.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let c1 = block_box(node(&root, 0));
    let c2 = block_box(node(&root, 1));
    let c3 = block_box(node(&root, 2));

    assert_eq!(c1.border_box.y, 0.0);
    // child2: collapsed margin max(10, 0) = 10, so y = child1_bottom + 10 = 20 + 10 = 30
    assert_eq!(c2.border_box.y, 30.0);
    // child3: flex bottom = 30 + 30 = 60, collapsed margin max(0, 15) = 15, so y = 60 + 15 = 75
    assert_eq!(c3.border_box.y, 75.0);
    // Parent height: child3 bottom = 75 + 40 = 115
    assert_eq!(block_box(&root).content_box.height, 115.0);
}

#[test]
fn block_sibling_with_margin_after_marginless_sibling() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let c1 = block_box(node(&root, 0));
    let c2 = block_box(node(&root, 1));
    let c3 = block_box(node(&root, 2));

    assert_eq!(c1.border_box.y, 0.0);
    // child2: margin collapsed between child1(10) and child2(0) => 10
    assert_eq!(c2.border_box.y, 20.0);
    // child3: margin collapsed between child2(0) and child3(20) => 20, NOT with parent
    assert_eq!(c3.border_box.y, 50.0);
}

#[test]
fn block_display_none() {
    let mut root = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::None,
            inner: InnerDisplay::Flow,
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert!(root.layout_box.is_empty());
    assert_eq!(root.layout_box.width(), 0.0);
    assert_eq!(root.layout_box.height(), 0.0);
}
