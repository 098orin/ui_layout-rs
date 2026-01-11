use ui_layout::*;

#[test]
fn flex_grow_redistribute_max() {
    let grow1 = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle {
            max_width: Length::Px(250.0),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let grow2 = LayoutNode::new(Style {
        display: Display::Block,
        item_style: ItemStyle {
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
            ..Default::default()
        },
        vec![grow1, grow2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let r1 = &root.children[0].rect;
    let r2 = &root.children[1].rect;

    assert_eq!(r1.width, 250.0);
    assert_eq!(r2.width, 550.0);
    assert_eq!(r1.width + r2.width, 800.0);
}

#[test]
fn flex_grow_redistribute_min() {
    let min_item = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle {
            min_width: Length::Px(300.0),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let grow_item = LayoutNode::new(Style {
        display: Display::Block,
        item_style: ItemStyle {
            flex_grow: 3.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            ..Default::default()
        },
        vec![min_item, grow_item],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let r1 = &root.children[0].rect;
    let r2 = &root.children[1].rect;

    assert!(r1.width >= 300.0);
    assert_eq!(r1.width + r2.width, 800.0);
}

#[test]
fn flex_basis_and_max() {
    let a = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle {
            max_width: Length::Px(200.0),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_basis: Some(150.0),
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let b = LayoutNode::new(Style {
        display: Display::Block,
        item_style: ItemStyle {
            flex_basis: Some(100.0),
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
            ..Default::default()
        },
        vec![a, b],
    );

    LayoutEngine::layout(&mut root, 500.0, 600.0);

    let r1 = &root.children[0].rect;
    let r2 = &root.children[1].rect;

    assert!(r1.width <= 200.0);
    assert_eq!(r1.width + r2.width, 500.0);
}
