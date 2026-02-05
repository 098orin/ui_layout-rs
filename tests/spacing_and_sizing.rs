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
            padding_top: Length::Percent(10.0), // 10% of width = 20px (CSS spec: all padding % relative to width)
            padding_bottom: Length::Percent(5.0), // 5% of width = 10px (CSS spec: all padding % relative to width)
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
            assert_eq!(box_model.padding_box.height, 130.0); // 100 + 20 + 10 (all % relative to width per CSS spec)
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
            padding_top: Length::Vw(2.5), // 2.5% of 800px = 20px (CSS spec: all padding % relative to width)
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
            assert_eq!(box_model.padding_box.height, 170.0); // 150 + 20 (padding % relative to width per CSS spec)
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
            // Content box should be specified size (content box sizing)
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
