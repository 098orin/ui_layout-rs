use ui_layout::*;

#[test]
fn padding_content_box() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(200.0),
            height: Length::Px(100.0),
            ..Default::default()
        },
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(15.0),
            padding_top: Length::Px(5.0),
            padding_bottom: Length::Px(8.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Content box should be the specified size
            assert_eq!(box_model.content_box.width, 200.0);
            assert_eq!(box_model.content_box.height, 100.0);

            // Padding box should include padding
            assert_eq!(box_model.padding_box.width, 225.0); // 200 + 10 + 15
            assert_eq!(box_model.padding_box.height, 113.0); // 100 + 5 + 8

            // Border box should be same as padding box (no border)
            assert_eq!(box_model.border_box.width, 225.0);
            assert_eq!(box_model.border_box.height, 113.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn border_box_sizing() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(200.0),
            height: Length::Px(100.0),
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

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Border box should be the specified size
            assert_eq!(box_model.border_box.width, 200.0);
            assert_eq!(box_model.border_box.height, 100.0);

            // Padding box should exclude borders
            assert_eq!(box_model.padding_box.width, 195.0); // 200 - 2 - 3
            assert_eq!(box_model.padding_box.height, 95.0); // 100 - 1 - 4

            // Content box should exclude padding and borders
            assert_eq!(box_model.content_box.width, 170.0); // 200 - 2 - 3 - 10 - 15
            assert_eq!(box_model.content_box.height, 82.0); // 100 - 1 - 4 - 5 - 8
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn margins_affect_positioning() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
            height: Length::Px(50.0),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: Length::Px(20.0),
            margin_top: Length::Px(10.0),
            margin_right: Length::Px(30.0),
            margin_bottom: Length::Px(15.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Px(300.0),
                height: Length::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Child should be positioned with margins
            assert_eq!(box_model.border_box.x, 20.0); // margin_left
            assert_eq!(box_model.border_box.y, 10.0); // margin_top
            assert_eq!(box_model.border_box.width, 100.0);
            assert_eq!(box_model.border_box.height, 50.0);
        }
        _ => panic!("Expected single box model"),
    }

    // Root should account for child margins in its height
    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.content_box.height, 75.0); // 10 + 50 + 15
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn min_width_constraint() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
            height: Length::Px(50.0),
            min_width: Length::Px(150.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Width should be constrained to minimum
            assert_eq!(box_model.content_box.width, 150.0);
            assert_eq!(box_model.content_box.height, 50.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn max_width_constraint() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(200.0),
            height: Length::Px(50.0),
            max_width: Length::Px(150.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Width should be constrained to maximum
            assert_eq!(box_model.content_box.width, 150.0);
            assert_eq!(box_model.content_box.height, 50.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn min_max_height_constraints() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
            height: Length::Px(30.0),
            min_height: Length::Px(50.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
            height: Length::Px(80.0),
            max_height: Length::Px(60.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Px(300.0),
                height: Length::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // First child height should be constrained by min_height
            assert_eq!(box_model.content_box.height, 50.0);
        }
        _ => panic!("Expected single box model"),
    }

    match &root.children[1].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Second child height should be constrained by max_height
            assert_eq!(box_model.content_box.height, 60.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn percentage_padding() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(200.0),
            height: Length::Px(100.0),
            ..Default::default()
        },
        spacing: Spacing {
            padding_left: Length::Percent(10.0),  // 10% of width = 20px
            padding_right: Length::Percent(5.0),  // 5% of width = 10px
            padding_top: Length::Percent(10.0),   // 10% of width = 20px
            padding_bottom: Length::Percent(5.0), // 5% of width = 10px
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Content box should be the specified size
            assert_eq!(box_model.content_box.width, 200.0);
            assert_eq!(box_model.content_box.height, 100.0);

            // Padding box should include percentage padding
            assert_eq!(box_model.padding_box.width, 230.0); // 200 + 20 + 10
            assert_eq!(box_model.padding_box.height, 130.0); // 100 + 20 + 10
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn viewport_relative_sizing() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Vw(50.0),  // 50% of 800px = 400px
            height: Length::Vh(25.0), // 25% of 600px = 150px
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: Length::Vw(5.0),  // 5% of 800px = 40px
            margin_top: Length::Vh(10.0),  // 10% of 600px = 60px
            padding_left: Length::Vw(2.5), // 2.5% of 800px = 20px
            padding_top: Length::Vw(2.5),  // 2.5% of 800px = 20px
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Content size should be viewport-relative
            assert_eq!(box_model.content_box.width, 400.0);
            assert_eq!(box_model.content_box.height, 150.0);

            // Position should account for viewport-relative margins
            assert_eq!(box_model.border_box.x, 40.0);
            assert_eq!(box_model.border_box.y, 60.0);

            // Padding should be viewport-relative
            assert_eq!(box_model.padding_box.width, 420.0); // 400 + 20
            assert_eq!(box_model.padding_box.height, 170.0); // 150 + 20
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn complex_spacing_calculation() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(300.0),
            height: Length::Px(200.0),
            ..Default::default()
        },
        box_sizing: BoxSizing::ContentBox,
        spacing: Spacing {
            margin_left: Length::Px(5.0),
            margin_right: Length::Px(5.0),
            margin_top: Length::Px(10.0),
            margin_bottom: Length::Px(10.0),

            border_left: Length::Px(3.0),
            border_right: Length::Px(3.0),
            border_top: Length::Px(2.0),
            border_bottom: Length::Px(2.0),

            padding_left: Length::Px(15.0),
            padding_right: Length::Px(20.0),
            padding_top: Length::Px(12.0),
            padding_bottom: Length::Px(18.0),
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Content box should be specified size
            assert_eq!(box_model.content_box.width, 300.0);
            assert_eq!(box_model.content_box.height, 200.0);

            // Padding box should include padding
            assert_eq!(box_model.padding_box.width, 335.0); // 300 + 15 + 20
            assert_eq!(box_model.padding_box.height, 230.0); // 200 + 12 + 18

            // Border box should include borders
            assert_eq!(box_model.border_box.width, 341.0); // 335 + 3 + 3
            assert_eq!(box_model.border_box.height, 234.0); // 230 + 2 + 2

            // Position should account for margins
            assert_eq!(box_model.border_box.x, 5.0);
            assert_eq!(box_model.border_box.y, 10.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn flex_with_spacing_constraints() {
    let child1 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: Length::Px(5.0),
            margin_right: Length::Px(10.0),
            padding_left: Length::Px(8.0),
            padding_right: Length::Px(12.0),
            border_left: Length::Px(2.0),
            border_right: Length::Px(2.0),
            ..Default::default()
        },
        size: SizeStyle {
            min_width: Length::Px(50.0),
            max_width: Length::Px(200.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: Length::Px(5.0),
            margin_right: Length::Px(5.0),
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
                height: Length::Px(100.0),
                ..Default::default()
            },
            column_gap: Length::Px(20.0),
            ..Default::default()
        },
        vec![child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Test that flex items respect their spacing and size constraints
    match &root.children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // First child should grow but respect constraints
            let content_width = box_model.content_box.width;
            assert!(content_width >= 50.0); // min_width constraint
            assert!(content_width <= 200.0); // max_width constraint

            // Border box should include spacing
            let expected_border_width = content_width + 8.0 + 12.0 + 2.0 + 2.0; // padding + border
            assert_eq!(box_model.border_box.width, expected_border_width);
        }
        _ => panic!("Expected single box model"),
    }

    match &root.children[1].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Second child should have fixed width
            assert_eq!(box_model.content_box.width, 100.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn auto_width_single_child() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(150.0),
            height: Length::Px(50.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Px(100.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Width should match containing block width
            assert_eq!(box_model.content_box.width, 800.0);
            assert_eq!(box_model.content_box.height, 100.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn auto_height_single_child() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
            height: Length::Px(75.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let inner = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Px(200.0),
                height: Length::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    let mut root = LayoutNode::with_children(Style::default(), vec![inner]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Parent with Auto height should adopt the child's height
            assert_eq!(box_model.content_box.width, 200.0);
            assert_eq!(box_model.content_box.height, 75.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn auto_width_multiple_children() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(120.0),
            height: Length::Px(50.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(80.0),
            height: Length::Px(50.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Px(100.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Width should match containing block width
            assert_eq!(box_model.content_box.width, 800.0);
            assert_eq!(box_model.content_box.height, 100.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn auto_height_multiple_children() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
            height: Length::Px(60.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
            height: Length::Px(90.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let inner = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Px(200.0),
                height: Length::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child1, child2],
    );

    let mut root = LayoutNode::with_children(Style::default(), vec![inner]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Parent with Auto height should adopt the largest child's height
            assert_eq!(box_model.content_box.width, 200.0);
            assert_eq!(box_model.content_box.height, 90.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn auto_width_with_padding() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
            height: Length::Px(50.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Px(100.0),
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(10.0),
                padding_right: Length::Px(15.0),
                padding_top: Length::Px(5.0),
                padding_bottom: Length::Px(8.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Width should match containing block width
            assert_eq!(box_model.content_box.width, 800.0);
            assert_eq!(box_model.content_box.height, 100.0);

            // Padding box should include padding
            assert_eq!(box_model.padding_box.width, 825.0); // 800 + 10 + 15
            assert_eq!(box_model.padding_box.height, 113.0); // 100 + 5 + 8
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn auto_height_with_padding() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
            height: Length::Px(60.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let inner = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Px(200.0),
                height: Length::Auto,
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(10.0),
                padding_right: Length::Px(15.0),
                padding_top: Length::Px(5.0),
                padding_bottom: Length::Px(8.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    let mut root = LayoutNode::with_children(Style::default(), vec![inner]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Content box should include child size
            assert_eq!(box_model.content_box.width, 200.0);
            assert_eq!(box_model.content_box.height, 60.0);

            // Padding box should include padding
            assert_eq!(box_model.padding_box.width, 225.0); // 200 + 10 + 15
            assert_eq!(box_model.padding_box.height, 73.0); // 60 + 5 + 8
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn auto_width_with_border() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
            height: Length::Px(50.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Px(100.0),
                ..Default::default()
            },
            spacing: Spacing {
                border_left: Length::Px(3.0),
                border_right: Length::Px(4.0),
                border_top: Length::Px(2.0),
                border_bottom: Length::Px(2.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Width should match containing block width
            assert_eq!(box_model.content_box.width, 800.0);
            assert_eq!(box_model.content_box.height, 100.0);

            // Padding box should be same as content box (no padding)
            assert_eq!(box_model.padding_box.width, 800.0);
            assert_eq!(box_model.padding_box.height, 100.0);

            // Border box should include borders
            assert_eq!(box_model.border_box.width, 807.0); // 800 + 3 + 4
            assert_eq!(box_model.border_box.height, 104.0); // 100 + 2 + 2
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn auto_height_with_border() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
            height: Length::Px(70.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Px(200.0),
                height: Length::Auto,
                ..Default::default()
            },
            spacing: Spacing {
                border_left: Length::Px(2.0),
                border_right: Length::Px(2.0),
                border_top: Length::Px(3.0),
                border_bottom: Length::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Note: Auto size of root follows viewport size.

            // Content box should include child size
            assert_eq!(box_model.content_box.width, 200.0);
            assert_eq!(box_model.content_box.height, 600.0 - 3.0 - 5.0);

            // Padding box should be same as content box (no padding)
            assert_eq!(box_model.content_box.width, 200.0);
            assert_eq!(box_model.content_box.height, 600.0 - 3.0 - 5.0);

            // Border box should include borders
            assert_eq!(box_model.border_box.width, 204.0);
            assert_eq!(box_model.border_box.height, 600.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn auto_sizing_with_padding_and_border() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(80.0),
            height: Length::Px(60.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Auto,
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(10.0),
                padding_right: Length::Px(10.0),
                padding_top: Length::Px(8.0),
                padding_bottom: Length::Px(8.0),
                border_left: Length::Px(2.0),
                border_right: Length::Px(2.0),
                border_top: Length::Px(1.0),
                border_bottom: Length::Px(1.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Width should match containing block width
            assert_eq!(box_model.content_box.width, 800.0);
            assert_eq!(box_model.content_box.height, 60.0);

            // Padding box should include padding
            assert_eq!(box_model.padding_box.width, 820.0); // 800 + 10 + 10
            assert_eq!(box_model.padding_box.height, 76.0); // 60 + 8 + 8

            // Border box should include borders
            assert_eq!(box_model.border_box.width, 824.0); // 820 + 2 + 2
            assert_eq!(box_model.border_box.height, 78.0); // 76 + 1 + 1
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn padding_border_size_verification() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(200.0),
            height: Length::Px(150.0),
            ..Default::default()
        },
        spacing: Spacing {
            padding_left: Length::Px(12.0),
            padding_right: Length::Px(18.0),
            padding_top: Length::Px(10.0),
            padding_bottom: Length::Px(15.0),
            border_left: Length::Px(2.0),
            border_right: Length::Px(3.0),
            border_top: Length::Px(1.0),
            border_bottom: Length::Px(4.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Content box
            assert_eq!(box_model.content_box.width, 200.0);
            assert_eq!(box_model.content_box.height, 150.0);

            // Verify padding individually
            let padding_width = box_model.padding_box.width - box_model.content_box.width;
            let padding_height = box_model.padding_box.height - box_model.content_box.height;
            assert_eq!(padding_width, 30.0); // 12 + 18
            assert_eq!(padding_height, 25.0); // 10 + 15

            // Verify border individually
            let border_width = box_model.border_box.width - box_model.padding_box.width;
            let border_height = box_model.border_box.height - box_model.padding_box.height;
            assert_eq!(border_width, 5.0); // 2 + 3
            assert_eq!(border_height, 5.0); // 1 + 4

            // Total sizes
            assert_eq!(box_model.padding_box.width, 230.0); // 200 + 30
            assert_eq!(box_model.padding_box.height, 175.0); // 150 + 25
            assert_eq!(box_model.border_box.width, 235.0); // 230 + 5
            assert_eq!(box_model.border_box.height, 180.0); // 175 + 5
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn auto_width_with_child_margins() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
            height: Length::Px(50.0),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: Length::Px(10.0),
            margin_right: Length::Px(15.0),
            margin_top: Length::Px(5.0),
            margin_bottom: Length::Px(5.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Px(100.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Width should match containing block width
            assert_eq!(box_model.content_box.width, 800.0);
            assert_eq!(box_model.content_box.height, 100.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn auto_width_with_child_padding() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
            height: Length::Px(50.0),
            ..Default::default()
        },
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(15.0),
            padding_top: Length::Px(5.0),
            padding_bottom: Length::Px(5.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Px(100.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Width should match containing block width
            assert_eq!(box_model.content_box.width, 800.0);
            assert_eq!(box_model.content_box.height, 100.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn auto_width_with_child_border() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
            height: Length::Px(50.0),
            ..Default::default()
        },
        spacing: Spacing {
            border_left: Length::Px(2.0),
            border_right: Length::Px(3.0),
            border_top: Length::Px(1.0),
            border_bottom: Length::Px(1.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Px(100.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Width should match containing block width
            assert_eq!(box_model.content_box.width, 800.0);
            assert_eq!(box_model.content_box.height, 100.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn auto_width_with_multiple_children_different_sizes() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
            height: Length::Px(50.0),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: Length::Px(5.0),
            margin_right: Length::Px(5.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(80.0),
            height: Length::Px(50.0),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: Length::Px(10.0),
            margin_right: Length::Px(10.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Px(100.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Width should match containing block width
            assert_eq!(box_model.content_box.width, 800.0);
            assert_eq!(box_model.content_box.height, 100.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn auto_width_with_parent_padding() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
            height: Length::Px(50.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Px(100.0),
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(10.0),
                padding_right: Length::Px(15.0),
                padding_top: Length::Px(5.0),
                padding_bottom: Length::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Width should match containing block width
            assert_eq!(box_model.content_box.width, 800.0);
            assert_eq!(box_model.content_box.height, 100.0);

            // Padding box should include parent's padding
            assert_eq!(box_model.padding_box.width, 825.0); // 800 + 10 + 15
            assert_eq!(box_model.padding_box.height, 110.0); // 100 + 5 + 5
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn auto_width_with_parent_border() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
            height: Length::Px(50.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Px(100.0),
                ..Default::default()
            },
            spacing: Spacing {
                border_left: Length::Px(2.0),
                border_right: Length::Px(3.0),
                border_top: Length::Px(1.0),
                border_bottom: Length::Px(1.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Width should match containing block width
            assert_eq!(box_model.content_box.width, 800.0);
            assert_eq!(box_model.content_box.height, 100.0);

            // Padding box should be same as content (no padding)
            assert_eq!(box_model.padding_box.width, 800.0);
            assert_eq!(box_model.padding_box.height, 100.0);

            // Border box should include parent's border
            assert_eq!(box_model.border_box.width, 805.0); // 800 + 2 + 3
            assert_eq!(box_model.border_box.height, 102.0); // 100 + 1 + 1
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn auto_width_nested_blocks() {
    let grandchild = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(80.0),
            height: Length::Px(30.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Px(50.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![grandchild],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Px(100.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Check grandchild
    match &root.children[0].children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.content_box.width, 80.0);
            assert_eq!(box_model.content_box.height, 30.0);
        }
        _ => panic!("Expected single box model for grandchild"),
    }

    // Check child (width uses containing block width from root)
    match &root.children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.content_box.width, 800.0);
            assert_eq!(box_model.content_box.height, 50.0);
        }
        _ => panic!("Expected single box model for child"),
    }

    // Check root (width uses viewport width)
    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            assert_eq!(box_model.content_box.width, 800.0);
            assert_eq!(box_model.content_box.height, 100.0);
        }
        _ => panic!("Expected single box model for root"),
    }
}

#[test]
fn auto_width_empty_container() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Px(100.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Width should match containing block width
            assert_eq!(box_model.content_box.width, 800.0);
            assert_eq!(box_model.content_box.height, 100.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn auto_width_with_border_box_sizing() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
            height: Length::Px(50.0),
            ..Default::default()
        },
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(10.0),
            border_left: Length::Px(2.0),
            border_right: Length::Px(2.0),
            ..Default::default()
        },
        box_sizing: BoxSizing::BorderBox,
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Px(100.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // Width should match containing block width
            assert_eq!(box_model.content_box.width, 800.0);
            assert_eq!(box_model.content_box.height, 100.0);
        }
        _ => panic!("Expected single box model"),
    }
}
