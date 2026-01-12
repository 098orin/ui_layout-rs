use ui_layout::*;

#[test]
fn test_flex_full_behavior() {
    let mut root = LayoutNode::new(Style {
        display: Display::Flex {
            flex_direction: FlexDirection::Row,
        },
        size: SizeStyle {
            width: Length::Px(300.0),
            height: Length::Px(100.0),
            ..Default::default()
        },
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(10.0),
            padding_top: Length::Px(5.0),
            padding_bottom: Length::Px(5.0),
            ..Default::default()
        },
        column_gap: Length::Px(5.0),
        ..Default::default()
    });

    let child1 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 1.0,
            flex_basis: Length::Px(50.0),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: Length::Px(5.0),
            margin_right: Length::Px(5.0),
            ..Default::default()
        },
        size: SizeStyle {
            width: Length::Auto,
            height: Length::Px(40.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 2.0,
            flex_basis: Length::Px(30.0),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: Length::Px(2.0),
            margin_right: Length::Px(2.0),
            ..Default::default()
        },
        size: SizeStyle {
            width: Length::Auto,
            height: Length::Px(50.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(40.0),
            height: Length::Px(60.0),
            ..Default::default()
        },
        ..Default::default()
    });

    root.children.push(child1);
    root.children.push(child2);
    root.children.push(child3);

    LayoutEngine::layout(&mut root, 300.0, 100.0);

    let c1 = &root.children[0].rect;
    let c2 = &root.children[1].rect;
    let c3 = &root.children[2].rect;

    // ==========
    // child1
    // ==========
    assert_eq!(c1.y, 5.0, "Child1 y offset by parent's padding_top");
    assert_eq!(
        c1.x,
        10.0 + 5.0,
        "Child1 x offset by parent's padding_left + margin_left"
    );

    let margin_total = 5.0 + 5.0 + 2.0 + 2.0; // child1 + child2
    let gap_total = 5.0 + 5.0; // gap
    let fixed_child_width = 40.0; // child3

    let flex_basis_total = 50.0 + 30.0; // child1 + child2

    let remaining_space = 300.0 - (margin_total + gap_total + flex_basis_total + fixed_child_width);
    let total_flex = 1.0 + 2.0;

    let expected_c1_width = 50.0 + remaining_space * 1.0 / total_flex;

    assert!(
        (c1.width - expected_c1_width).abs() < 0.01,
        "Child1 width correct with flex-grow"
    );

    // ==========
    // child2
    // ==========

    // c1.x + c1.width + c1.margin_right + gap + c2.margin_left
    let expected_c2_x = c1.x + c1.width + 5.0 + 5.0 + 2.0;
    assert!(
        (c2.x - expected_c2_x).abs() < 0.01,
        "Child2 x correct with spacing and gap"
    );
    let expected_c2_width = 30.0 + remaining_space * 2.0 / total_flex;
    assert!(
        (c2.width - expected_c2_width).abs() < 0.01,
        "Child2 width correct with flex-grow"
    );

    // ==========
    // child3
    // ==========
    let expected_c3_x = c2.x + c2.width + 2.0 + 5.0; // c2.margin_right + gap + c3.margin_left(default 0)
    assert!(
        (c3.x - expected_c3_x).abs() < 0.01,
        "Child3 x correct with spacing and gap"
    );
    assert_eq!(c3.width, 40.0, "Child3 width fixed");
    assert_eq!(c3.y, 5.0, "Child3 y offset by parent's padding_top");
}

#[test]
fn test_flex_column_layout_with_grow() {
    let mut root = LayoutNode::new(Style {
        display: Display::Flex {
            flex_direction: FlexDirection::Column,
        },
        size: SizeStyle {
            width: Length::Px(100.0),
            height: Length::Px(200.0),
            ..Default::default()
        },
        row_gap: Length::Px(5.0),
        ..Default::default()
    });

    let child1 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 1.0,
            flex_basis: Length::Px(50.0),
            ..Default::default()
        },
        size: SizeStyle {
            width: Length::Px(80.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 2.0,
            flex_basis: Length::Px(30.0),
            ..Default::default()
        },
        size: SizeStyle {
            width: Length::Px(60.0),
            ..Default::default()
        },
        ..Default::default()
    });

    root.children.push(child1);
    root.children.push(child2);

    LayoutEngine::layout(&mut root, 100.0, 200.0);

    let c1 = &root.children[0].rect;
    let c2 = &root.children[1].rect;

    // child1 height
    let remaining_space = 200.0 - 50.0 - 30.0 - 5.0; // gap = 5
    let total_flex = 1.0 + 2.0;
    let expected_c1_height = 50.0 + remaining_space * 1.0 / total_flex;
    assert!(
        (c1.height - expected_c1_height).abs() < 0.01,
        "Child1 height correct with flex-grow in column"
    );

    let expected_c2_height = 30.0 + remaining_space * 2.0 / total_flex;
    assert!(
        (c2.height - expected_c2_height).abs() < 0.01,
        "Child2 height correct with flex-grow in column"
    );

    // x positions remain as width offsets (no padding/margin in this test)
    assert_eq!(c1.x, 0.0);
    assert_eq!(c2.x, 0.0);

    // y positions
    assert_eq!(c1.y, 0.0);
    let expected_c2_y = c1.y + c1.height + 5.0; // gap
    assert!((c2.y - expected_c2_y).abs() < 0.01);
}
