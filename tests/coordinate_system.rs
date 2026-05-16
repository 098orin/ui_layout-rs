use ui_layout::*;

#[test]
fn test_child_coordinates_relative_to_parent_content_box() {
    // Parent with padding and border
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut parent = LayoutNode::with_node_children(
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
        vec![child],
    );

    LayoutEngine::layout(&mut parent, 800.0, 600.0);

    let parent_box = match &parent.layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model for parent"),
    };

    let child_box = match &parent.children[0].layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model for child"),
    };

    // Parent content box should start at border + padding
    assert_eq!(parent_box.content_box.x, 25.0); // border_left(5) + padding_left(20)
    assert_eq!(parent_box.content_box.y, 18.0); // border_top(3) + padding_top(15)

    // Child should be positioned relative to parent's content box (0,0)
    // Child coordinates are relative to parent's content box origin
    assert_eq!(child_box.border_box.x, 0.0); // Relative to parent content box
    assert_eq!(child_box.border_box.y, 0.0); // Relative to parent content box
}

#[test]
fn test_nested_coordinate_system() {
    // Deeply nested structure to test coordinate propagation
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

    let child = LayoutNode::with_node_children(
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
        vec![grandchild],
    );

    let mut root = LayoutNode::with_node_children(
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
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let root_box = match &root.layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model for root"),
    };

    let child_box = match &root.children[0].layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model for child"),
    };

    let grandchild_box = match &root.children[0].children[0].layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model for grandchild"),
    };

    // Root content box
    assert_eq!(root_box.content_box.x, 20.0);
    assert_eq!(root_box.content_box.y, 25.0);

    // Child should be positioned relative to root's content box
    // Child position = child_margin (relative to root's content box origin)
    assert_eq!(child_box.border_box.x, 15.0); // child_margin_left (relative to root content box)
    assert_eq!(child_box.border_box.y, 12.0); // child_margin_top (relative to root content box)

    // Child's content box (relative to child's border box)
    assert_eq!(child_box.content_box.x, 15.0 + 10.0); // child_border_x + padding_left
    assert_eq!(child_box.content_box.y, 12.0 + 8.0); // child_border_y + padding_top

    // Grandchild should be positioned relative to child's content box
    // Grandchild position = grandchild_margin (relative to child's content box)
    assert_eq!(grandchild_box.border_box.x, 5.0); // grandchild_margin_left (relative to child content box)
    assert_eq!(grandchild_box.border_box.y, 3.0); // grandchild_margin_top (relative to child content box)
}

#[test]
fn test_flex_children_coordinates() {
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

    let mut flex_container = LayoutNode::with_node_children(
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
        vec![child1, child2],
    );

    LayoutEngine::layout(&mut flex_container, 800.0, 600.0);

    let container_box = match &flex_container.layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model for flex container"),
    };

    let child1_box = match &flex_container.children[0].layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model for child1"),
    };

    let child2_box = match &flex_container.children[1].layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model for child2"),
    };

    // Container content box
    assert_eq!(container_box.content_box.x, 15.0); // padding_left
    assert_eq!(container_box.content_box.y, 10.0); // padding_top

    // Child1 should be at the start of container's content box (relative coordinates)
    assert_eq!(child1_box.border_box.x, 0.0); // Start of container content box
    assert_eq!(child1_box.border_box.y, 0.0); // Start of container content box

    // Child2 should be positioned after child1 + gap (relative coordinates)
    assert_eq!(child2_box.border_box.x, 40.0 + 20.0); // child1_width + gap
    assert_eq!(child2_box.border_box.y, 0.0); // Same row as child1
}

#[test]
fn test_block_children_coordinates_with_margins() {
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
            margin_top: LengthOrAuto::Length(Length::Px(15.0)), // Should collapse with child1's margin_bottom
            ..Default::default()
        },
        ..Default::default()
    });

    let mut parent = LayoutNode::with_node_children(
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
        vec![child1, child2],
    );

    LayoutEngine::layout(&mut parent, 800.0, 600.0);

    let parent_box = match &parent.layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model for parent"),
    };

    let child1_box = match &parent.children[0].layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model for child1"),
    };

    let child2_box = match &parent.children[1].layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model for child2"),
    };

    // Parent content box
    assert_eq!(parent_box.content_box.x, 25.0);
    assert_eq!(parent_box.content_box.y, 20.0);

    // Child1 should be at the start of parent's content box (relative coordinates)
    assert_eq!(child1_box.border_box.x, 0.0);
    assert_eq!(child1_box.border_box.y, 0.0);

    // Child2 should be positioned after child1 + collapsed margin (max(20, 15) = 20)
    assert_eq!(child2_box.border_box.x, 0.0);
    assert_eq!(child2_box.border_box.y, 40.0 + 20.0); // child1_height + collapsed_margin
}

#[test]
fn test_coordinate_system_with_auto_margins() {
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

    let mut parent = LayoutNode::with_node_children(
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
        vec![child],
    );

    LayoutEngine::layout(&mut parent, 800.0, 600.0);

    let parent_box = match &parent.layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model for parent"),
    };

    let child_box = match &parent.children[0].layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model for child"),
    };

    // Parent content box
    assert_eq!(parent_box.content_box.x, 15.0);
    assert_eq!(parent_box.content_box.y, 10.0);
    assert_eq!(parent_box.content_box.width, 200.0);

    // Child should be centered horizontally within parent's content box (relative coordinates)
    let expected_child_x = (200.0 - 60.0) / 2.0; // (content_width - child_width) / 2
    assert_eq!(child_box.border_box.x, expected_child_x);
    assert_eq!(child_box.border_box.y, 0.0); // Top of parent content box
}
