use ui_layout::*;

#[test]
fn block_vertical_margin_collapsing_between_siblings() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_node_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let c1_box = match &root.children[0].layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model"),
    };

    let c2_box = match &root.children[1].layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model"),
    };

    // child1 is at the top so y = 0
    assert_eq!(c1_box.border_box.y, 0.0);

    // margin-bottom(30) and margin-top(10) collapse to max(30, 10) = 30
    assert_eq!(c2_box.border_box.y, c1_box.border_box.height + 30.0);
}
