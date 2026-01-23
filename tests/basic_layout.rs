use ui_layout::*;

#[test]
fn block_basic_box_model() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(200.0),
            height: Length::Px(100.0),
            ..Default::default()
        },
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(10.0),
            padding_top: Length::Px(5.0),
            padding_bottom: Length::Px(5.0),
            border_left: Length::Px(2.0),
            border_right: Length::Px(2.0),
            border_top: Length::Px(1.0),
            border_bottom: Length::Px(1.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(root.box_model.content_box.width, 200.0);
    assert_eq!(root.box_model.content_box.height, 100.0);

    assert_eq!(root.box_model.padding_box.width, 220.0);
    assert_eq!(root.box_model.padding_box.height, 110.0);

    assert_eq!(root.box_model.border_box.width, 224.0);
    assert_eq!(root.box_model.border_box.height, 112.0);
}

#[test]
fn block_auto_height_from_children() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            height: Length::Px(40.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Px(100.0),
                height: Length::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(root.children[0].box_model.content_box.height, 40.0);
}

#[test]
fn flex_row_grow() {
    let child1 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 1.0,
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

    let c1 = &root.children[0];
    let c2 = &root.children[1];

    assert_eq!(c1.box_model.content_box.width, 150.0);
    assert_eq!(c2.box_model.content_box.width, 150.0);
}

#[test]
fn flex_gap_affects_children_box() {
    let child1 = LayoutNode::new(Style::default());
    let child2 = LayoutNode::new(Style::default());

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            size: SizeStyle {
                width: Length::Px(200.0),
                height: Length::Px(50.0),
                ..Default::default()
            },
            column_gap: Length::Px(20.0),
            ..Default::default()
        },
        vec![child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(root.box_model.children_box.width, 20.0);
    assert_eq!(
        root.children[1].box_model.border_box.x,
        root.children[0].box_model.border_box.width + 20.0
    );
}

#[test]
fn flex_align_items_stretch() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            height: Length::Auto,
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
                width: Length::Px(100.0),
                height: Length::Px(80.0),
                ..Default::default()
            },
            align_items: AlignItems::Stretch,
            ..Default::default()
        },
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(root.children[0].box_model.content_box.height, 80.0);
}

#[test]
fn block_margin_auto_centering() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(100.0),
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

    let x = root.children[0].box_model.border_box.x;
    assert_eq!(x, 100.0); // (300 - 100) / 2
}
