use layout::*;

#[test]
fn flex_row_basic() {
    // 左サイドバー（固定幅）
    let sidebar = LayoutNode::new(Style {
        display: Display::Block,
        item_style: ItemStyle { flex_grow: 0.0 },
        width: Some(200.0),
        height: None,
        padding: 0.0,
    });

    // エディタ（残り全部）
    let editor = LayoutNode::new(Style {
        display: Display::Block,
        item_style: ItemStyle { flex_grow: 1.0 },
        width: None,
        height: None,
        padding: 0.0,
    });

    // 親（Row）
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

    // レイアウト実行（ウィンドウサイズ 800x600）
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // 結果確認
    let sidebar_rect = root.children[0].rect;
    let editor_rect = root.children[1].rect;

    println!("sidebar: {:?}", sidebar_rect);
    println!("editor : {:?}", editor_rect);

    // 最低限のアサーション
    assert_eq!(sidebar_rect.width, 200.0);
    assert_eq!(editor_rect.width, 600.0);
}
