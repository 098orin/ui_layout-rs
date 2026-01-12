use ui_layout::*;

#[test]
fn test_row_gap() {
    // Flex container
    let mut root = LayoutNode::new(Style {
        display: Display::Flex {
            flex_direction: FlexDirection::Row,
        },
        size: SizeStyle {
            width: Length::Px(300.0),
            height: Length::Px(100.0),
            ..Default::default()
        },
        column_gap: Length::Px(10.0),
        ..Default::default()
    });

    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(50.0),
            height: Length::Px(50.0),
            ..Default::default()
        },
        ..Default::default()
    });
    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(50.0),
            height: Length::Px(50.0),
            ..Default::default()
        },
        ..Default::default()
    });
    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(50.0),
            height: Length::Px(50.0),
            ..Default::default()
        },
        ..Default::default()
    });

    root.children.push(child1);
    root.children.push(child2);
    root.children.push(child3);

    LayoutEngine::layout(&mut root, 300.0, 100.0);

    dbg!(&root);

    let c1 = &root.children[0].rect;
    let c2 = &root.children[1].rect;
    let c3 = &root.children[2].rect;

    assert!(
        (c2.x - c1.x - c1.width - 10.0).abs() < 0.01,
        "Gap between Child1 and Child2 correct"
    );
    assert!(
        (c3.x - c2.x - c2.width - 10.0).abs() < 0.01,
        "Gap between Child2 and Child3 correct"
    );

    assert_eq!(c1.x, 0.0, "First child x = 0");
}
