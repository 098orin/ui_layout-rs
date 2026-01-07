use ui_layout::*;

#[test]
fn flex_column_auto() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: None,
            height: Some(20.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: None,
            height: Some(40.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            row_gap: 10.0,
            ..Default::default()
        },
        vec![child1, child2],
    );

    LayoutEngine::layout(&mut root, 200.0, 200.0);

    assert_eq!(root.rect.width, 200.0);
    assert_eq!(root.rect.height, 200.0);

    // main axis
    assert_eq!(root.children[0].rect.height, 20.0);
    assert_eq!(root.children[1].rect.height, 40.0);

    // cross axis (stretch)
    assert_eq!(root.children[0].rect.width, 200.0);
    assert_eq!(root.children[1].rect.width, 200.0);

    assert_eq!(root.children[0].rect.y, 0.0);
    assert_eq!(root.children[1].rect.y, 20.0 + 10.0);
}

#[test]
fn nested_flex_auto_size() {
    let inner_child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Some(40.0),
            height: Some(10.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let inner_child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Some(60.0),
            height: Some(20.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let inner = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            column_gap: 5.0,
            ..Default::default()
        },
        vec![inner_child1, inner_child2],
    );

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            ..Default::default()
        },
        vec![inner],
    );

    LayoutEngine::layout(&mut root, 300.0, 300.0);

    let inner_rect = root.children[0].rect;

    // stretch
    assert_eq!(inner_rect.width, 300.0);
    // maximum height of children
    assert_eq!(inner_rect.height, 20.0);
}

#[test]
fn block_auto_size() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Some(50.0),
            height: Some(10.0),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: 5.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Some(30.0),
            height: Some(20.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Block,
            ..Default::default()
        },
        vec![child1, child2],
    );

    LayoutEngine::layout(&mut root, 200.0, 200.0);

    assert_eq!(root.rect.width, 200.0);
    assert_eq!(root.rect.height, 200.0);

    assert_eq!(root.children[1].rect.y, 10.0 + 5.0);
}

#[test]
fn nested_block_size() {
    let child1_1 = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle {
            width: Some(50.0),
            height: Some(20.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child1_2 = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle {
            width: Some(70.0),
            height: Some(10.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child1 = LayoutNode::with_children(
        Style {
            display: Display::Block,
            ..Default::default()
        },
        vec![child1_1, child1_2],
    );

    let child2 = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle {
            width: Some(40.0),
            height: Some(15.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let inner = LayoutNode::with_children(
        Style {
            display: Display::Block,
            ..Default::default()
        },
        vec![child1, child2],
    );

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Block,
            ..Default::default()
        },
        vec![inner],
    );

    LayoutEngine::layout(&mut root, 300.0, 200.0);

    println!("{:#?}", root);

    let inner_size = root.children[0].rect;

    assert_eq!(inner_size.height, 45.0);
}

#[test]
fn nested_too_big_flex() {
    let child = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle {
            width: Some(60.0),
            height: Some(16.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let inner1 = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            ..Default::default()
        },
        vec![child],
    );

    let inner2 = LayoutNode::new(Style {
        display: Display::Flex {
            flex_direction: FlexDirection::Row,
        },
        size: SizeStyle {
            height: Some(700.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let outer = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            ..Default::default()
        },
        vec![inner1, inner2],
    );

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            ..Default::default()
        },
        vec![outer],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    println!("{:#?}", root);

    assert!(root.rect.width > 0.0);
    assert!(root.rect.height > 0.0);

    let outer = &root.children[0];
    assert!(outer.rect.width > 0.0);
    assert!(outer.rect.height > 0.0);

    let inner1 = &outer.children[0];
    assert!(inner1.rect.width > 0.0);
    assert!(inner1.rect.height > 0.0);

    let child = &inner1.children[0];
    assert!(child.rect.width > 0.0);
    assert!(child.rect.height > 0.0);

    let inner2 = &outer.children[1];
    assert!(inner2.rect.width > 0.0);
    assert!(inner2.rect.height > 0.0);
}
