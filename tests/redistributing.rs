use ui_layout::*;

#[test]
fn flex_grow_redistribute_max() {
    // grow1: max_width
    let grow1 = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle {
            max_width: Some(250.0),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    // grow2: no max_width
    let grow2 = LayoutNode::new(Style {
        display: Display::Block,
        size: SizeStyle::default(),
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

    let r_grow1 = &root.children[0].rect;
    let r_grow2 = &root.children[1].rect;

    println!("grow1: {:?}", r_grow1);
    println!("grow2: {:?}", r_grow2);

    // max_width
    assert_eq!(r_grow1.width, 250.0);

    // else
    let expected_grow2_width = 800.0 - r_grow1.width;
    assert_eq!(r_grow2.width, expected_grow2_width);

    // total
    let total: f32 = r_grow1.width + r_grow2.width;
    assert_eq!(total, 800.0);
}
