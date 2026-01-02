use layout::*;

#[test]
fn flex_row_basic() {
    // Left sidebar (fixed width)
    let sidebar = LayoutNode::new(Style {
        display: Display::Block,
        item_style: ItemStyle { flex_grow: 0.0 },
        width: Some(200.0),
        height: None,
        padding: 0.0,
    });

    // Editor (takes all remaining space)
    let editor = LayoutNode::new(Style {
        display: Display::Block,
        item_style: ItemStyle { flex_grow: 1.0 },
        width: None,
        height: None,
        padding: 0.0,
    });

    // Parent (Row)
    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            width: None,
            height: None,
            item_style: ItemStyle::default(),
            padding: 0.0,
        },
        vec![sidebar, editor],
    );

    // Run layout (Windows size: 800x600)
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
