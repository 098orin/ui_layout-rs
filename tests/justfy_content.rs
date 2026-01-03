use ui_layout::*;

#[test]
fn justify_content_center_column() {
    let child1 = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle {
            height: Some(50.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle {
            height: Some(50.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        vec![child1, child2],
    );

    LayoutEngine::layout(&mut root, 100.0, 300.0);

    let c1 = &root.children[0].rect;
    let c2 = &root.children[1].rect;

    // total children height = 100
    // remaining = 300 - 100 = 200
    // start offset = 100
    assert_eq!(c1.y, 100.0);
    assert_eq!(c2.y, 150.0);
}

#[test]
fn justify_content_space_between_column() {
    let child1 = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle {
            height: Some(50.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle {
            height: Some(50.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle {
            height: Some(50.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            justify_content: JustifyContent::SpaceBetween,
            ..Default::default()
        },
        vec![child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 100.0, 300.0);

    let c1 = &root.children[0].rect;
    let c2 = &root.children[1].rect;
    let c3 = &root.children[2].rect;

    // used = 150
    // remaining = 150
    // gap = 150 / (3 - 1) = 75
    assert_eq!(c1.y, 0.0);
    assert_eq!(c2.y, 50.0 + 75.0);
    assert_eq!(c3.y, 50.0 * 2.0 + 75.0 * 2.0);
}

#[test]
fn justify_content_space_evenly_column() {
    let children = (0..3)
        .map(|_| {
            LayoutNode::new(Style {
                display: Display::Block,
                size: SizeStyle {
                    height: Some(50.0),
                    ..Default::default()
                },
                ..Default::default()
            })
        })
        .collect();

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            justify_content: JustifyContent::SpaceEvenly,
            ..Default::default()
        },
        children,
    );

    LayoutEngine::layout(&mut root, 100.0, 300.0);

    let c1 = root.children[0].rect.y;
    let c2 = root.children[1].rect.y;
    let c3 = root.children[2].rect.y;

    // used = 150
    // remaining = 150
    // gap = 150 / (3 + 1) = 37.5

    assert_eq!(c1, 37.5);
    assert_eq!(c2, 37.5 * 2.0 + 50.0);
    assert_eq!(c3, 37.5 * 3.0 + 100.0);
}
