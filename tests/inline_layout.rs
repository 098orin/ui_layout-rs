use ui_layout::*;

#[test]
fn layout_box_into_iter_block_single_line() {
    let fragment1 = ItemFragment::Fragment(Fragment {
        width: 30.0,
        height: 20.0,
    })
    .into();

    let fragment2 = ItemFragment::Fragment(Fragment {
        width: 40.0,
        height: 25.0,
    })
    .into();

    let fragment3 = ItemFragment::Fragment(Fragment {
        width: 35.0,
        height: 15.0,
    })
    .into();

    let child = LayoutNode::with_fragment_children(
        Style {
            display: Display::parse("inline").unwrap(),
            spacing: Spacing {
                padding_left: Length::Px(5.0),
                padding_right: Length::Px(5.0),
                border_left: Length::Px(2.0),
                border_right: Length::Px(2.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![fragment1, fragment2, fragment3],
    );

    let mut root = LayoutNode::with_node_children(Style::default(), vec![child]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let boxes: Vec<BoxModel> = dbg!(&root.children[0].node().unwrap().layout_box)
        .into_iter()
        .collect();
    assert_eq!(boxes.len(), 1);
    let b = dbg!(&boxes[0]);
    assert_eq!(b.content_box.width, 105.0);
    assert_eq!(b.border_box.width, 105.0 + 14.0);
}
