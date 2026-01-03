use ui_layout::*;

#[test]
fn flex_row_basic() {
    // Left sidebar (fixed width)
    let sidebar = LayoutNode::new(Style {
        display: Display::Block,
        item_style: ItemStyle {
            flex_grow: 0.0,
            ..Default::default()
        },
        size: SizeStyle {
            width: Some(200.0),
            ..Default::default()
        },
        ..Default::default()
    });

    // Editor (takes all remaining space)
    let editor = LayoutNode::new(Style {
        display: Display::Block,
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    // Parent (Row)
    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            ..Default::default()
        },
        vec![sidebar, editor],
    );

    // Run layout (Window size: 800x600)
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Check the result
    let sidebar_rect = root.children[0].rect;
    let editor_rect = root.children[1].rect;

    println!("sidebar: {:?}", sidebar_rect);
    println!("editor : {:?}", editor_rect);

    // Verify layout results
    assert_eq!(sidebar_rect.width, 200.0);
    assert_eq!(editor_rect.width, 600.0);
}

#[test]
fn layout_mvp() {
    let toolbar = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle {
            height: Some(40.0),
            ..Default::default()
        },
        ..Style::default()
    });

    let sidebar = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle {
            width: Some(200.0),
            ..Default::default()
        },
        ..Style::default()
    });

    let editor = LayoutNode::new(Style {
        display: Display::Block,
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        size: SizeStyle::default(),
        ..Style::default()
    });

    let main = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            item_style: ItemStyle {
                flex_grow: 1.0,
                ..Default::default()
            },
            size: SizeStyle::default(),
            ..Style::default()
        },
        vec![sidebar, editor],
    );

    let status = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle {
            height: Some(24.0),
            ..Default::default()
        },
        ..Style::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            size: SizeStyle::default(),
            ..Style::default()
        },
        vec![toolbar, main, status],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let toolbar_rect = &root.children[0].rect;
    let main_rect = &root.children[1].rect;
    let status_rect = &root.children[2].rect;

    assert_eq!(toolbar_rect.height, 40.0);
    assert_eq!(main_rect.height, 600.0 - 40.0 - 24.0);
    assert_eq!(status_rect.height, 24.0);

    let sidebar_rect = &root.children[1].children[0].rect;
    let editor_rect = &root.children[1].children[1].rect;

    assert_eq!(sidebar_rect.width, 200.0);
    assert_eq!(editor_rect.width, 800.0 - 200.0);
}
