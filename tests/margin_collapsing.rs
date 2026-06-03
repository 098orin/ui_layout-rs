use ui_layout::*;

fn node<'a>(n: &'a LayoutNode, idx: usize) -> &'a LayoutNode {
    n.children[idx].node().expect("expected node child")
}

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

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let c1_box = match &node(&root, 0).layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected block box model"),
    };

    let c2_box = match &node(&root, 1).layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected block box model"),
    };

    // child1 is at the top so y = 0
    assert_eq!(c1_box.border_box.y, 0.0);

    // margin-bottom(30) and margin-top(10) collapse to max(30, 10) = 30
    assert_eq!(c2_box.border_box.y, c1_box.border_box.height + 30.0);
}

#[test]
fn auto_height_covers_deeply_nested_block_children_with_margins() {
    let def_style = Style {
        spacing: Spacing {
            margin_top: Length::Px(10.0).into(),
            margin_bottom: Length::Px(10.0).into(),
            margin_left: Length::Px(10.0).into(),
            margin_right: Length::Px(10.0).into(),
            ..Default::default()
        },
        ..Default::default()
    };

    let mut root = LayoutNode::new(Style {
        spacing: Spacing {
            ..Default::default()
        },
        ..def_style.clone()
    });

    fn push_child(parent: &mut LayoutNode, style: Style, max: usize, current: usize) {
        if current + 1 < max {
            parent.children.push(LayoutNode::new(style.clone()).into());
            push_child(
                parent.children[0].node_mut().unwrap(),
                style,
                max,
                current + 1,
            );
        } else {
            parent.children.push(
                LayoutNode::new(Style {
                    size: SizeStyle {
                        height: Length::Px(50.0).into(),
                        ..Default::default()
                    },
                    ..style
                })
                .into(),
            );
        }
    }

    for i in 0..10 {
        root.children
            .push(LayoutNode::new(def_style.clone()).into());
        push_child(
            root.children[i].node_mut().unwrap(),
            def_style.clone(),
            5,
            0,
        );
    }

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let root_box = match &root.layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected block box model"),
    };

    let first_child = node(&root, 0);
    let first_child_box = match &first_child.layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected block box model"),
    };

    let leaf = node(node(node(node(node(first_child, 0), 0), 0), 0), 0);
    let leaf_box = match &leaf.layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected block box model"),
    };

    assert_eq!(leaf_box.border_box.y + leaf_box.border_box.height, 60.0);
    assert_eq!(first_child_box.border_box.height, 150.0);
    assert_eq!(root_box.border_box.height, 1610.0);
}
