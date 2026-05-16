use ui_layout::*;

fn node<'a>(n: &'a LayoutNode, idx: usize) -> &'a LayoutNode {
    n.children[idx].node().expect("expected node child")
}

#[test]
fn block_basic_box_model() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(200.0)),
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(10.0),
            padding_top: Length::Px(5.0),
            padding_bottom: Length::Px(5.0),
            border_left: Length::Px(2.0),
            border_right: Length::Px(2.0),
            border_top: Length::Px(1.0),
            border_bottom: Length::Px(1.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_box {
        LayoutBox::BlockBox(box_model) => {
            assert_eq!(box_model.content_box.width, 200.0);
            assert_eq!(box_model.content_box.height, 100.0);
            assert_eq!(box_model.padding_box.width, 220.0);
            assert_eq!(box_model.padding_box.height, 110.0);
            assert_eq!(box_model.border_box.width, 224.0);
            assert_eq!(box_model.border_box.height, 112.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn block_auto_height_from_children() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(40.0)),
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
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &node(&root, 0).layout_box {
        LayoutBox::BlockBox(box_model) => {
            assert_eq!(box_model.content_box.height, 40.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn flex_row_grow() {
    let child1 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_node_children(
        Style {
            display: Display {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Length(Length::Px(50.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        vec![child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let c1_box = match &node(&root, 0).layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model"),
    };

    let c2_box = match &node(&root, 1).layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model"),
    };

    assert_eq!(c1_box.content_box.width, 150.0);
    assert_eq!(c2_box.content_box.width, 150.0);
}

#[test]
fn flex_gap_affects_children_box() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(15.0)),
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });
    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(10.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_node_children(
        Style {
            display: Display {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(50.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            column_gap: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        vec![child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let root_box = match &root.layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model"),
    };

    let child0_box = match &node(&root, 0).layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model"),
    };

    let child1_box = match &node(&root, 1).layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model"),
    };

    assert_eq!(root_box.children_box.width, 15.0 + 10.0 + 20.0);
    assert_eq!(child1_box.border_box.x, child0_box.border_box.width + 20.0);
}

#[test]
fn flex_align_items_stretch() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Auto,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_node_children(
        Style {
            display: Display {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Length(Length::Px(80.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Stretch,
            ..Default::default()
        },
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    dbg!(root.layout_box.height());

    match &node(&root, 0).layout_box {
        LayoutBox::BlockBox(box_model) => {
            assert_eq!(box_model.content_box.height, 80.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn block_margin_auto_centering() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Auto,
            margin_right: LengthOrAuto::Auto,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_node_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let child_box = match &node(&root, 0).layout_box {
        LayoutBox::BlockBox(box_model) => box_model,
        _ => panic!("Expected single box model"),
    };

    assert_eq!(child_box.border_box.x, 100.0); // (300 - 100) / 2
}
