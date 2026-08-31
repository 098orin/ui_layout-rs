mod common;
use common::*;
use ui_layout::*;

// --- Block spacing ---

#[test]
fn block_margin_left_shifts_node() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 15.0);
}

#[test]
fn block_margin_auto_left_right_centers_node() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(80.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
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
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 60.0);
}

#[test]
fn block_margin_collapse_three_siblings_with_uneven_margins() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(10.0)),
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
            margin_bottom: LengthOrAuto::Length(Length::Px(25.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(5.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(20.0)),
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

    // Margins collapse: between child1(30) and child2(10) => 30
    // Between child2(25) and child3(5) => 25
    // Parent has no border/padding => last child's bottom margin (20) extrudes
    // Parent's content height excludes the extruded margin
    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 40.0);
    assert_eq!(block_box(node(&root, 2)).border_box.y, 85.0);
    assert_eq!(block_box(&root).content_box.height, 100.0);
}

#[test]
fn block_auto_height_respects_child_margin_bottom() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: LengthOrAuto::Length(Length::Px(30.0)),
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
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Parent has no border/padding => child's margin_bottom extrudes below
    assert_eq!(block_box(&root).content_box.height, 50.0);
    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
}

#[test]
fn block_auto_height_respects_child_margin_top() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(50.0)),
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
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Parent has no border/padding => child's margin_top collapses with parent
    // Child is at y=0 and parent height excludes extruded margin
    assert_eq!(block_box(&root).content_box.height, 50.0);
    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
}

// --- Flex spacing ---

#[test]
fn flex_row_margin_on_child_affects_spacing_between_items() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_right: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::OutsideInner {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Length(Length::Px(50.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 0)).border_box.width, 50.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 70.0);
}

#[test]
fn flex_row_margin_left_shifts_node() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::OutsideInner {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Length(Length::Px(50.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 65.0);
}

#[test]
fn flex_children_with_margins_justify_center() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_right: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::OutsideInner {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Length(Length::Px(50.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert!(approx_eq(block_box(node(&root, 0)).border_box.x, 85.0));
    assert!(approx_eq(block_box(node(&root, 1)).border_box.x, 155.0));
}

#[test]
fn flex_children_with_margins_justify_space_between() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_right: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::OutsideInner {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Length(Length::Px(50.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert!(approx_eq(block_box(node(&root, 0)).border_box.x, 0.0));
    assert!(approx_eq(block_box(node(&root, 1)).border_box.x, 240.0));
}

#[test]
fn flex_children_with_margins_justify_space_evenly() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(40.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_right: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::OutsideInner {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Length(Length::Px(50.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceEvenly,
            ..Default::default()
        },
        [child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert!(approx_eq(block_box(node(&root, 0)).border_box.x, 35.0));
    assert!(approx_eq(block_box(node(&root, 1)).border_box.x, 120.0));
    assert!(approx_eq(block_box(node(&root, 2)).border_box.x, 205.0));
}

#[test]
fn flex_auto_margin_with_fixed_margin_on_sibling() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_right: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Auto,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::OutsideInner {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Length(Length::Px(50.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 0)).border_box.width, 50.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 240.0);
    assert_eq!(block_box(node(&root, 1)).border_box.width, 60.0);
}

#[test]
fn flex_column_margin_between_items() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(40.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::OutsideInner {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 45.0);
}

#[test]
fn flex_column_margin_top_shifts_node() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(40.0)),
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
            display: Display::OutsideInner {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 50.0);
}

// --- Gap + margin interaction ---

#[test]
fn flex_margin_with_gap_interaction() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(40.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_right: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::OutsideInner {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Length(Length::Px(50.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            column_gap: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 0)).border_box.width, 40.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 70.0);
}

#[test]
fn flex_padding_on_container_affects_children_box() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::OutsideInner {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(20.0),
                padding_top: Length::Px(15.0),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(&root).content_box.x, 20.0);
    assert_eq!(block_box(&root).content_box.y, 15.0);
    assert_eq!(block_box(&root).children_box.width, 50.0);
    assert_eq!(block_box(&root).children_box.height, 30.0);
}

#[test]
fn flex_padding_with_multiple_children_justify_center() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(40.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::OutsideInner {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Length(Length::Px(50.0)),
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(30.0),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert!(approx_eq(block_box(node(&root, 0)).border_box.x, 100.0));
    assert!(approx_eq(block_box(node(&root, 1)).border_box.x, 140.0));
}

// --- Nested spacing ---

#[test]
fn nested_padding_margin() {
    let inner = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Length(Length::Px(10.0)),
            margin_top: LengthOrAuto::Length(Length::Px(5.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let outer = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(20.0),
                padding_top: Length::Px(15.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [inner],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(400.0)),
                height: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(30.0),
                padding_top: Length::Px(25.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [outer],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let root_box = block_box(&root);
    let outer_box = block_box(node(&root, 0));
    let inner_box = block_box(node(node(&root, 0), 0));

    assert_eq!(root_box.content_box.x, 30.0);
    assert_eq!(root_box.content_box.y, 25.0);
    assert_eq!(outer_box.border_box.x, 0.0);
    assert_eq!(outer_box.border_box.y, 0.0);
    assert_eq!(outer_box.content_box.x, 20.0);
    assert_eq!(outer_box.content_box.y, 15.0);
    assert_eq!(inner_box.border_box.x, 10.0);
    assert_eq!(inner_box.border_box.y, 5.0);
}

// --- Percentage-based spacing ---

#[test]
fn percentage_padding_on_container() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(200.0)),
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        spacing: Spacing {
            padding_left: Length::Percent(10.0),
            padding_top: Length::Percent(5.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Percentage padding resolves against the containing block width (viewport = 800)
    assert_eq!(block_box(&root).content_box.x, 80.0);
    assert_eq!(block_box(&root).content_box.y, 40.0);
}

// --- Border-box sizing ---

#[test]
fn border_box_sizing_with_padding_and_margin() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(30.0)),
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            box_sizing: BoxSizing::BorderBox,
            spacing: Spacing {
                padding_left: Length::Px(20.0),
                padding_right: Length::Px(20.0),
                border_left: Length::Px(5.0),
                border_right: Length::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert_eq!(b.border_box.width, 200.0);
    assert_eq!(b.content_box.width, 150.0);
    assert_eq!(b.content_box.x, 25.0);

    let child_box = block_box(node(&root, 0));
    assert_eq!(child_box.border_box.x, 10.0);
}

// --- Edge cases ---

#[test]
fn all_margins_auto_on_block() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Auto,
            margin_right: LengthOrAuto::Auto,
            margin_top: LengthOrAuto::Auto,
            margin_bottom: LengthOrAuto::Auto,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let child_box = block_box(node(&root, 0));

    assert_eq!(child_box.border_box.x, 100.0);
    assert_eq!(child_box.border_box.y, 0.0);
}

#[test]
fn zero_padding_margin() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Length(Length::Px(0.0)),
            margin_right: LengthOrAuto::Length(Length::Px(0.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(0.0),
                padding_right: Length::Px(0.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    assert_eq!(block_box(&root).content_box.x, 0.0);
    assert_eq!(block_box(&root).content_box.y, 0.0);
}

#[test]
fn multiple_flex_children_with_various_margins() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(30.0)),
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_right: LengthOrAuto::Length(Length::Px(5.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(40.0)),
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Length(Length::Px(10.0)),
            margin_right: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Length(Length::Px(8.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::OutsideInner {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Length(Length::Px(50.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 45.0);
    assert_eq!(block_box(node(&root, 2)).border_box.x, 108.0);
}

#[test]
fn flex_column_auto_height_includes_child_margins() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(10.0)),
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
            margin_top: LengthOrAuto::Length(Length::Px(10.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(40.0)),
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
            display: Display::OutsideInner {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        [child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let container_height = block_box(&root).content_box.height;
    let expected = (10.0 + 20.0 + 10.0) + (10.0 + 30.0 + 10.0) + (10.0 + 40.0 + 10.0);
    assert!(
        (container_height - expected).abs() < 0.01,
        "expected height {}, got {}",
        expected,
        container_height
    );
}

#[test]
fn flex_row_auto_height_includes_child_cross_margins() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(20.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
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
            width: LengthOrAuto::Length(Length::Px(30.0)),
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(5.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::OutsideInner {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let container_height = block_box(&root).content_box.height;
    let expected = f32::max(5.0 + 30.0 + 5.0, 5.0 + 20.0 + 15.0);
    assert!(
        (container_height - expected).abs() < 0.01,
        "expected height {}, got {}",
        expected,
        container_height
    );
}
