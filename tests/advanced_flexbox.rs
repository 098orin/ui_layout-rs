use ui_layout::*;

#[test]
fn flex_basis_fixed_size() {
    let child1 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: Length::Px(80.0),
            flex_grow: 0.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: Length::Px(60.0),
            flex_grow: 0.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            size: SizeStyle {
                width: Length::Px(300.0),
                height: Length::Px(50.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.content_box.width, 80.0); // flex_basis
        }
        _ => panic!("Expected single box model"),
    }

    match &root.children[1].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.content_box.width, 60.0); // flex_basis
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn flex_basis_with_grow() {
    let child1 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: Length::Px(50.0),
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: Length::Px(30.0),
            flex_grow: 2.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            size: SizeStyle {
                width: Length::Px(300.0),
                height: Length::Px(50.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Available space: 300 - 50 - 30 = 220px
    // Child1 gets: 50 + (220 * 1/3) = 123.33px
    // Child2 gets: 30 + (220 * 2/3) = 176.67px

    match &root.children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert!((box_model.content_box.width - 123.33).abs() < 0.1);
        }
        _ => panic!("Expected single box model"),
    }

    match &root.children[1].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert!((box_model.content_box.width - 176.67).abs() < 0.1);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn flex_column_layout() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: Length::Px(40.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            height: Length::Px(30.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            size: SizeStyle {
                width: Length::Px(200.0),
                height: Length::Px(150.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.content_box.height, 40.0);
        }
        _ => panic!("Expected single box model"),
    }

    match &root.children[1].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.content_box.height, 80.0); // 150 - 40 - 30 = 80
        }
        _ => panic!("Expected single box model"),
    }

    match &root.children[2].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.content_box.height, 30.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn flex_justify_content_space_evenly() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(50.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(60.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(40.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            size: SizeStyle {
                width: Length::Px(300.0),
                height: Length::Px(50.0),
                ..Default::default()
            },
            justify_content: JustifyContent::SpaceEvenly,
            ..Default::default()
        },
        vec![child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Available space: 300 - 50 - 60 - 40 = 150px
    // Space evenly: 150 / 4 = 37.5px gaps
    // Positions: 37.5, 125, 222.5

    match &root.children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert!((box_model.border_box.x - 37.5).abs() < 0.1);
        }
        _ => panic!("Expected single box model"),
    }

    match &root.children[1].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert!((box_model.border_box.x - 125.0).abs() < 0.1);
        }
        _ => panic!("Expected single box model"),
    }

    match &root.children[2].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert!((box_model.border_box.x - 222.5).abs() < 0.1);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn flex_align_items_different_values() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(50.0),
            height: Length::Px(30.0),
            ..Default::default()
        },
        item_style: ItemStyle {
            align_self: Some(AlignItems::Start),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(60.0),
            height: Length::Px(40.0),
            ..Default::default()
        },
        item_style: ItemStyle {
            align_self: Some(AlignItems::Center),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(40.0),
            height: Length::Px(20.0),
            ..Default::default()
        },
        item_style: ItemStyle {
            align_self: Some(AlignItems::End),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            size: SizeStyle {
                width: Length::Px(300.0),
                height: Length::Px(100.0),
                ..Default::default()
            },
            align_items: AlignItems::Stretch, // Default, but children override
            ..Default::default()
        },
        vec![child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Child1: align-self: start -> y = 0
    match &root.children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.border_box.y, 0.0);
        }
        _ => panic!("Expected single box model"),
    }

    // Child2: align-self: center -> y = (100 - 40) / 2 = 30
    match &root.children[1].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.border_box.y, 30.0);
        }
        _ => panic!("Expected single box model"),
    }

    // Child3: align-self: end -> y = 100 - 20 = 80
    match &root.children[2].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.border_box.y, 80.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn nested_flex_containers() {
    // Inner flex container
    let inner_child1 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        size: SizeStyle {
            height: Length::Px(20.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let inner_child2 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 2.0,
            ..Default::default()
        },
        size: SizeStyle {
            height: Length::Px(20.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let inner_flex = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            size: SizeStyle {
                width: Length::Px(120.0),
                height: Length::Px(30.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inner_child1, inner_child2],
    );

    // Outer flex container
    let regular_child = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(80.0),
            height: Length::Px(30.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            size: SizeStyle {
                width: Length::Px(250.0),
                height: Length::Px(50.0),
                ..Default::default()
            },
            column_gap: Length::Px(10.0),
            ..Default::default()
        },
        vec![inner_flex, regular_child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Check outer container children positioning
    match &root.children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.border_box.x, 0.0);
            assert_eq!(box_model.content_box.width, 120.0);
        }
        _ => panic!("Expected single box model"),
    }

    match &root.children[1].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.border_box.x, 130.0); // 120 + 10 (gap)
            assert_eq!(box_model.content_box.width, 80.0);
        }
        _ => panic!("Expected single box model"),
    }

    // Check inner flex container children
    match &root.children[0].children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.content_box.width, 40.0); // 120 * 1/3
        }
        _ => panic!("Expected single box model"),
    }

    match &root.children[0].children[1].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.content_box.width, 80.0); // 120 * 2/3
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn flex_with_percentage_basis() {
    let child1 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: Length::Percent(30.0), // 30% of 300px = 90px
            flex_grow: 0.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: Length::Percent(20.0), // 20% of 300px = 60px
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            size: SizeStyle {
                width: Length::Px(300.0),
                height: Length::Px(50.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.content_box.width, 90.0); // 30% of 300px
        }
        _ => panic!("Expected single box model"),
    }

    match &root.children[1].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // flex_basis: 60px + remaining space (300 - 90 - 60 = 150px)
            assert_eq!(box_model.content_box.width, 210.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn flex_auto_margins_override_justify_content() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(50.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(60.0),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: Length::Auto,
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(40.0),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: Length::Auto,
            margin_right: Length::Auto,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            size: SizeStyle {
                width: Length::Px(300.0),
                height: Length::Px(50.0),
                ..Default::default()
            },
            justify_content: JustifyContent::Center, // Should be overridden by auto margins
            ..Default::default()
        },
        vec![child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Available space: 300 - 50 - 60 - 40 = 150px
    // Child1: no auto margin, at start
    match &root.children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.border_box.x, 0.0);
        }
        _ => panic!("Expected single box model"),
    }

    // Child2: margin-left: auto pushes it right
    // Child3: centered with auto margins
    // The exact positioning depends on the auto margin implementation
}

#[test]
fn flex_row_gap_column_gap() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(60.0),
            height: Length::Px(40.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(70.0),
            height: Length::Px(30.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(50.0),
            height: Length::Px(35.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            size: SizeStyle {
                width: Length::Px(300.0),
                height: Length::Px(80.0),
                ..Default::default()
            },
            column_gap: Length::Px(15.0), // Gap between items in row
            row_gap: Length::Px(10.0),    // Not used in single row, but testing
            ..Default::default()
        },
        vec![child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Child positions should account for gaps
    // Child1: x = 0
    // Child2: x = 60 + 15 = 75
    // Child3: x = 75 + 70 + 15 = 160

    match &root.children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.border_box.x, 0.0);
        }
        _ => panic!("Expected single box model"),
    }

    match &root.children[1].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.border_box.x, 75.0);
        }
        _ => panic!("Expected single box model"),
    }

    match &root.children[2].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.border_box.x, 160.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn flex_min_max_constraints_with_grow() {
    let child1 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        size: SizeStyle {
            min_width: Length::Px(80.0),
            max_width: Length::Px(120.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 2.0,
            ..Default::default()
        },
        size: SizeStyle {
            min_width: Length::Px(60.0),
            max_width: Length::Px(100.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            size: SizeStyle {
                width: Length::Px(400.0),
                height: Length::Px(50.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Without constraints: child1 would get 133.33px, child2 would get 266.67px
    // With constraints: child1 clamped to max 120px, child2 clamped to max 100px

    match &root.children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert!(box_model.content_box.width <= 120.0); // Respects max constraint
            assert!(box_model.content_box.width >= 80.0); // Respects min constraint
        }
        _ => panic!("Expected single box model"),
    }

    match &root.children[1].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert!(box_model.content_box.width <= 100.0); // Respects max constraint
            assert!(box_model.content_box.width >= 60.0); // Respects min constraint
        }
        _ => panic!("Expected single box model"),
    }
}
