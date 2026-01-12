use ui_layout::*;

#[test]
fn test_block_layout() {
    let child_style = Style {
        size: SizeStyle {
            width: Length::Auto,
            height: Length::Auto,
            ..Default::default()
        },
        ..Default::default()
    };
    let root_style = Style {
        size: SizeStyle {
            width: Length::Px(200.0),
            height: Length::Auto,
            ..Default::default()
        },
        ..Default::default()
    };

    let child = LayoutNode::new(child_style);
    let mut root = LayoutNode::with_children(root_style, vec![child]);

    LayoutEngine::layout(&mut root, 200.0, 100.0);

    let child_rect = &root.children[0].rect;
    assert_eq!(child_rect.width, 200.0, "Child should fill parent width");
    assert_eq!(child_rect.height, 0.0, "Child height auto -> 0 by default");
}

#[test]
fn test_flex_layout_row() {
    // Flex container
    let mut root = LayoutNode::new(Style {
        display: Display::Flex {
            flex_direction: FlexDirection::Row,
        },
        size: SizeStyle {
            width: Length::Px(300.0),
            height: Length::Px(100.0),
            ..Default::default()
        },
        justify_content: JustifyContent::Start,
        ..Default::default()
    });

    // Flex children
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Auto,
            height: Length::Auto,
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_grow: 1.0,
            flex_basis: Length::Px(50.0),
            ..Default::default()
        },
        ..Default::default()
    });
    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Auto,
            height: Length::Auto,
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_grow: 2.0,
            flex_basis: Length::Px(50.0),
            ..Default::default()
        },
        ..Default::default()
    });

    root.children.push(child1);
    root.children.push(child2);

    LayoutEngine::layout(&mut root, 300.0, 100.0);

    dbg!(&root);

    let c1 = &root.children[0].rect;
    let c2 = &root.children[1].rect;

    // 200px [1:2] → 50 + 66.66... : 50 + 133.33...
    let total_grow = 1.0 + 2.0;
    let remaining = 300.0 - 50.0 - 50.0; // 200
    let expected_c1_width = 50.0 + remaining * 1.0 / total_grow; // 50 + 66.666...
    let expected_c2_width = 50.0 + remaining * 2.0 / total_grow; // 50 + 133.333...

    assert!(
        (c1.width - expected_c1_width).abs() < 0.01,
        "Child1 width correct"
    );
    assert!(
        (c2.width - expected_c2_width).abs() < 0.01,
        "Child2 width correct"
    );
}
