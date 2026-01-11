use ui_layout::*;

#[test]
fn align_self_center_overrides_parent() {
    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            align_items: AlignItems::Start,
            spacing: Spacing {
                padding_left: 10.0,
                padding_right: 10.0,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![LayoutNode::new(Style {
            display: Display::Block,
            size: SizeStyle {
                width: Length::Px(40.0),
                height: Length::Px(20.0),
                ..Default::default()
            },
            item_style: ItemStyle {
                align_self: Some(AlignItems::Center),
                ..Default::default()
            },
            ..Default::default()
        })],
    );

    LayoutEngine::layout(&mut root, 200.0, 100.0);

    let child = root.children[0].rect;

    // (180 - 40) / 2 = 70
    assert_eq!(child.x, 10.0 + 70.0);
    assert_eq!(child.width, 40.0);
}

#[test]
fn align_self_end_overrides_parent() {
    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            align_items: AlignItems::Start,
            spacing: Spacing {
                padding_left: 10.0,
                padding_right: 10.0,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![LayoutNode::new(Style {
            display: Display::Block,
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
        })],
    );

    LayoutEngine::layout(&mut root, 200.0, 100.0);

    let child = root.children[0].rect;

    // 180 - 40 = 140
    assert_eq!(child.x, 10.0 + 140.0);
    assert_eq!(child.width, 40.0);
}

#[test]
fn align_self_stretch_overrides_parent() {
    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            align_items: AlignItems::Center,
            spacing: Spacing {
                padding_left: 10.0,
                padding_right: 10.0,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![LayoutNode::new(Style {
            display: Display::Block,
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Px(20.0),
                ..Default::default()
            },
            item_style: ItemStyle {
                align_self: Some(AlignItems::Stretch),
                ..Default::default()
            },
            ..Default::default()
        })],
    );

    LayoutEngine::layout(&mut root, 200.0, 100.0);

    let child = &root.children[0].rect;

    assert_eq!(child.x, 10.0);
    assert_eq!(child.width, 180.0);
}

#[test]
fn align_self_auto_uses_parent_align_items() {
    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            align_items: AlignItems::End,
            spacing: Spacing {
                padding_left: 10.0,
                padding_right: 10.0,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![LayoutNode::new(Style {
            display: Display::Block,
            size: SizeStyle {
                width: Length::Px(40.0),
                height: Length::Px(20.0),
                ..Default::default()
            },
            item_style: ItemStyle {
                align_self: None, // auto
                ..Default::default()
            },
            ..Default::default()
        })],
    );

    LayoutEngine::layout(&mut root, 200.0, 100.0);

    let child = root.children[0].rect;

    assert_eq!(child.x, 10.0 + 140.0);
    assert_eq!(child.width, 40.0);
}
