mod common;
use common::*;
use ui_layout::*;

// --- Basic coordinates ---

#[test]
fn child_coordinates_relative_to_parent_content_box() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut parent = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(20.0),
                padding_top: Length::Px(15.0),
                padding_right: Length::Px(10.0),
                padding_bottom: Length::Px(5.0),
                border_left: Length::Px(5.0),
                border_top: Length::Px(3.0),
                border_right: Length::Px(7.0),
                border_bottom: Length::Px(2.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut parent, 800.0, 600.0);

    let parent_box = block_box(&parent);
    let child_box = block_box(node(&parent, 0));

    assert_eq!(parent_box.content_box.x, 25.0);
    assert_eq!(parent_box.content_box.y, 18.0);
    assert_eq!(child_box.border_box.x, 0.0);
    assert_eq!(child_box.border_box.y, 0.0);
}

#[test]
fn nested_coordinate_system() {
    let grandchild = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(20.0)),
            height: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Length(Length::Px(5.0)),
            margin_top: LengthOrAuto::Length(Length::Px(3.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Length(Length::Px(60.0)),
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(10.0),
                padding_top: Length::Px(8.0),
                margin_left: LengthOrAuto::Length(Length::Px(15.0)),
                margin_top: LengthOrAuto::Length(Length::Px(12.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        [grandchild],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(20.0),
                padding_top: Length::Px(25.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let root_box = block_box(&root);
    let child_box = block_box(node(&root, 0));
    let grandchild_box = block_box(node(node(&root, 0), 0));

    assert_eq!(root_box.content_box.x, 20.0);
    assert_eq!(root_box.content_box.y, 25.0);
    assert_eq!(child_box.border_box.x, 15.0);
    assert_eq!(child_box.border_box.y, 12.0);
    assert_eq!(child_box.content_box.x, 25.0);
    assert_eq!(child_box.content_box.y, 20.0);
    assert_eq!(grandchild_box.border_box.x, 5.0);
    assert_eq!(grandchild_box.border_box.y, 3.0);
}

// --- Flex children coordinates ---

#[test]
fn flex_children_coordinates() {
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

    let mut flex_container = LayoutNode::with_children(
        Style {
            display: Display {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(80.0)),
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(15.0),
                padding_top: Length::Px(10.0),
                padding_right: Length::Px(5.0),
                padding_bottom: Length::Px(8.0),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            column_gap: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut flex_container, 800.0, 600.0);

    let container_box = block_box(&flex_container);
    let child1_box = block_box(node(&flex_container, 0));
    let child2_box = block_box(node(&flex_container, 1));

    assert_eq!(container_box.content_box.x, 15.0);
    assert_eq!(container_box.content_box.y, 10.0);
    assert_eq!(child1_box.border_box.x, 0.0);
    assert_eq!(child1_box.border_box.y, 0.0);
    assert_eq!(child2_box.border_box.x, 60.0);
    assert_eq!(child2_box.border_box.y, 0.0);
}

// --- Block children coordinates with margins ---

#[test]
fn block_children_coordinates_with_margins() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(40.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(120.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut parent = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(25.0),
                padding_top: Length::Px(20.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut parent, 800.0, 600.0);

    let parent_box = block_box(&parent);
    let child1_box = block_box(node(&parent, 0));
    let child2_box = block_box(node(&parent, 1));

    assert_eq!(parent_box.content_box.x, 25.0);
    assert_eq!(parent_box.content_box.y, 20.0);
    assert_eq!(child1_box.border_box.x, 0.0);
    assert_eq!(child1_box.border_box.y, 0.0);
    assert_eq!(child2_box.border_box.x, 0.0);
    assert_eq!(child2_box.border_box.y, 60.0);
}

// --- Auto margins ---

#[test]
fn coordinate_system_with_auto_margins() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            height: LengthOrAuto::Length(Length::Px(40.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Auto,
            margin_right: LengthOrAuto::Auto,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut parent = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(15.0),
                padding_top: Length::Px(10.0),
                padding_right: Length::Px(15.0),
                padding_bottom: Length::Px(10.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut parent, 800.0, 600.0);

    let parent_box = block_box(&parent);
    let child_box = block_box(node(&parent, 0));

    assert_eq!(parent_box.content_box.x, 15.0);
    assert_eq!(parent_box.content_box.y, 10.0);
    assert_eq!(parent_box.content_box.width, 200.0);

    let expected_child_x = (200.0 - 60.0) / 2.0;
    assert_eq!(child_box.border_box.x, expected_child_x);
    assert_eq!(child_box.border_box.y, 0.0);
}
