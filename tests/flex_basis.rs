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
