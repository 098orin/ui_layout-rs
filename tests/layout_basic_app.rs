use ui_layout::*;

#[test]
fn layout_mvp() {
    let toolbar = LayoutNode::new(Style {
        display: Display::Block,
        height: Some(40.0),
        ..Style::default()
    });

    let sidebar = LayoutNode::new(Style {
        display: Display::Block,
        width: Some(200.0),
        ..Style::default()
    });

    let editor = LayoutNode::new(Style {
        display: Display::Block,
        item_style: ItemStyle { flex_grow: 1.0 },
        ..Style::default()
    });

    let main = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            item_style: ItemStyle { flex_grow: 1.0 },
            ..Style::default()
        },
        vec![sidebar, editor],
    );

    let status = LayoutNode::new(Style {
        display: Display::Block,
        height: Some(24.0),
        ..Style::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            ..Style::default()
        },
        vec![toolbar, main, status],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let toolbar = &root.children[0].rect;
    let main = &root.children[1].rect;
    let status = &root.children[2].rect;

    assert_eq!(toolbar.height, 40.0);
    assert_eq!(main.height, 600.0 - 40.0 - 24.0);
    assert_eq!(status.height, 24.0);

    let sidebar = &root.children[1].children[0].rect;
    let editor = &root.children[1].children[1].rect;

    assert_eq!(sidebar.width, 200.0);
    assert_eq!(editor.width, 800.0 - 200.0);
}
