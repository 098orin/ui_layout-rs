use ui_layout::*;

fn node<'a>(n: &'a LayoutNode, idx: usize) -> &'a LayoutNode {
    n.children[idx].node().expect("expected node child")
}

#[test]
fn test_flex_basis_auto_simple() {
    let mut container = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Flex,
        },
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(300.0)),
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        flex_direction: FlexDirection::Row,
        ..Default::default()
    });

    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });
    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(80.0)),
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    container.children = vec![
        LayoutChild::Node(Box::new(child1)),
        LayoutChild::Node(Box::new(child2)),
    ];

    LayoutEngine::layout(&mut container, 800.0, 600.0);

    if let LayoutBox::BlockBox(ref b) = node(&container, 0).layout_box {
        assert_eq!(b.border_box.width, 50.0);
    } else {
        panic!("expected block box for child 0")
    }

    if let LayoutBox::BlockBox(ref b) = node(&container, 1).layout_box {
        assert_eq!(b.border_box.width, 80.0);
    } else {
        panic!("expected block box for child 1")
    }
}

#[test]
fn test_flex_basis_grow_simple() {
    let mut container = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Flex,
        },
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(300.0)),
            ..Default::default()
        },
        flex_direction: FlexDirection::Row,
        ..Default::default()
    });

    let child = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: LengthOrAuto::Length(Length::Px(100.0)),
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    container.children = vec![LayoutChild::Node(Box::new(child))];
    LayoutEngine::layout(&mut container, 800.0, 600.0);

    if let LayoutBox::BlockBox(ref b) = node(&container, 0).layout_box {
        assert_eq!(b.border_box.width, 300.0);
    } else {
        panic!("expected block box for child")
    }
}

#[test]
fn flex_grow_with_explicit_width() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
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
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // flex_basis: auto → falls back to explicit width
    // Child1 basis = 100, Child2 basis = 50
    // Remaining = 300 - 100 - 50 = 150
    // Each grows by 150/2 = 75
    // Child1 = 175, Child2 = 125

    match &node(&root, 0).layout_box {
        LayoutBox::BlockBox(box_model) => {
            assert!((box_model.content_box.width - 175.0).abs() < 0.1);
        }
        _ => panic!("Expected block box model"),
    }

    match &node(&root, 1).layout_box {
        LayoutBox::BlockBox(box_model) => {
            assert!((box_model.content_box.width - 125.0).abs() < 0.1);
        }
        _ => panic!("Expected block box model"),
    }
}

#[test]
fn flex_shrink_with_explicit_width() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(200.0)),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_shrink: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(150.0)),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_shrink: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(210.0)),
                height: LengthOrAuto::Length(Length::Px(50.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // flex_basis: auto → falls back to explicit width
    // Child1 basis = 200, Child2 basis = 150
    // Overflow = 210 - 200 - 150 = -140
    // Shrink ratios: 200*1 : 150*1 → 200:150 → 4:3
    // Child1 shrinks by 140 * (200/(200+150)) = 80 → 120
    // Child2 shrinks by 140 * (150/(200+150)) = 60 → 90

    match &node(&root, 0).layout_box {
        LayoutBox::BlockBox(box_model) => {
            assert!((box_model.content_box.width - 120.0).abs() < 0.1);
        }
        _ => panic!("Expected block box model"),
    }

    match &node(&root, 1).layout_box {
        LayoutBox::BlockBox(box_model) => {
            assert!((box_model.content_box.width - 90.0).abs() < 0.1);
        }
        _ => panic!("Expected block box model"),
    }
}

#[test]
fn flex_column_grow_with_explicit_height() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_grow: 2.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // flex_basis: auto → falls back to explicit height
    // Child1 basis = 30, Child2 basis = 20
    // Remaining = 200 - 30 - 20 = 150
    // Total grow = 3
    // Child1 grows by 150 * 1/3 = 50 → 80
    // Child2 grows by 150 * 2/3 = 100 → 120

    match &node(&root, 0).layout_box {
        LayoutBox::BlockBox(box_model) => {
            assert!((box_model.content_box.height - 80.0).abs() < 0.1);
        }
        _ => panic!("Expected block box model"),
    }

    match &node(&root, 1).layout_box {
        LayoutBox::BlockBox(box_model) => {
            assert!((box_model.content_box.height - 120.0).abs() < 0.1);
        }
        _ => panic!("Expected block box model"),
    }
}
