use ui_layout::*;

#[test]
fn block_vertical_margin_collapsing_between_siblings() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: Length::Px(20.0),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: Length::Px(30.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: Length::Px(20.0),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: Length::Px(10.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Px(100.0),
                height: Length::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let c1 = &root.children[0];
    let c2 = &root.children[1];

    // child1 は先頭なので y = 0
    assert_eq!(c1.box_model.border_box.y, 0.0);

    // margin-bottom(30) と margin-top(10) は回収されて max(30, 10) = 30
    assert_eq!(
        c2.box_model.border_box.y,
        c1.box_model.border_box.height + 30.0
    );
}
