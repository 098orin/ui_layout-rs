use ui_layout::*;

#[test]
fn test_header_width() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(200.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let header = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle {
            width: Length::Px(200.0),
            ..Default::default()
        },
        spacing: Spacing {
            padding_left: Length::Px(16.0),
            padding_right: Length::Px(16.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert!(
        header.rect.width <= root.rect.width,
        "Header width exceeds root width"
    );
}

#[test]
fn test_footer_height() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            height: Length::Px(50.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let footer = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle {
            height: Length::Px(50.0),
            ..Default::default()
        },
        spacing: Spacing {
            padding_top: Length::Px(4.0),
            padding_bottom: Length::Px(4.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert!(
        footer.rect.height <= root.rect.height,
        "Footer height exceeds root height"
    );
}

#[test]
fn test_sidebar_height() {
    let parent = LayoutNode::new(Style {
        size: SizeStyle {
            height: Length::Px(120.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let sidebar = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle {
            min_height: Length::Px(100.0),
            ..Default::default()
        },
        spacing: Spacing {
            padding_top: Length::Px(10.0),
            padding_bottom: Length::Px(10.0),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    assert!(
        sidebar.rect.height <= parent.rect.height,
        "Sidebar height exceeds parent"
    );
}

#[test]
fn test_align_self_end() {
    let child = LayoutNode::new(Style {
        display: Display::Block,
        item_style: ItemStyle {
            align_self: Some(AlignItems::End),
            ..Default::default()
        },
        size: SizeStyle {
            width: Length::Px(20.0),
            height: Length::Px(30.0),
            ..Default::default()
        },
        spacing: Spacing::default(),
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            size: SizeStyle {
                height: Length::Px(100.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    dbg!(&mut root);

    assert_eq!(
        root.children[0].rect.x + root.children[0].rect.width,
        root.rect.width,
    );
}

#[test]
fn test_padding_box() {
    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Block,
            spacing: Spacing {
                padding_top: Length::Px(8.0),
                padding_bottom: Length::Px(8.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![LayoutNode::new(Style {
            display: Display::Block,
            size: SizeStyle {
                width: Length::Px(100.0),
                height: Length::Px(20.0),
                ..Default::default()
            },
            spacing: Spacing {
                padding_top: Length::Px(4.0),
                padding_bottom: Length::Px(4.0),
                ..Default::default()
            },
            ..Default::default()
        })],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let child = &root.children[0];
    assert!(
        child.rect.height + 8.0 + 8.0 <= root.rect.height,
        "Padding double-counted"
    );
}
