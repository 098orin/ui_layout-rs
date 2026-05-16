use ui_layout::*;

#[test]
fn layout_box_into_iter_block_single() {
    let mut child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            padding_left: Length::Px(5.0),
            padding_right: Length::Px(5.0),
            border_left: Length::Px(2.0),
            border_right: Length::Px(2.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_node_children(Style::default(), vec![child]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let boxes: Vec<BoxModel> = (&root.layout_box).into_iter().collect();
    assert_eq!(boxes.len(), 1);
    let b = &boxes[0];
    // border box width = content width + padding + border: 60 + 5+5 + 2+2 = 74
    assert_eq!(b.border_box.width, 74.0);
    assert_eq!(b.content_box.width, 60.0);
}
