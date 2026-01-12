use ui_layout::*;

#[test]
fn test_spacing_and_gap() {
    let mut root = LayoutNode::new(Style {
        display: Display::Flex {
            flex_direction: FlexDirection::Row,
        },
        size: SizeStyle {
            width: Length::Px(200.0),
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
        spacing: Spacing {
            margin_left: Length::Px(5.0),
            margin_right: Length::Px(5.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(40.0),
            height: Length::Px(60.0),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: Length::Px(3.0),
            margin_right: Length::Px(2.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(30.0),
            height: Length::Px(40.0),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: Length::Px(2.0),
            margin_right: Length::Px(1.0),
            ..Default::default()
        },
        ..Default::default()
    });

    root.children.push(child1);
    root.children.push(child2);
    root.children.push(child3);

    LayoutEngine::layout(&mut root, 200.0, 100.0);

    dbg!(&root);

    let c1 = &root.children[0].rect;
    let c2 = &root.children[1].rect;
    let c3 = &root.children[2].rect;

    // child1: margin_left = 5
    assert!((c1.x - 5.0).abs() < 0.01, "Child1 x correct with margin");
    assert_eq!(c1.y, 0.0, "Child1 y top aligned");

    // child2: c1.x + c1.width + c1.margin_right + gap + c2.margin_left
    let expected_c2_x = 5.0 + 50.0 + 5.0 + 10.0 + 3.0;
    assert!(
        (c2.x - expected_c2_x).abs() < 0.01,
        "Child2 x correct with spacing and gap"
    );
    assert_eq!(c2.y, 0.0, "Child2 y top aligned");

    // child3: c2.x + c2.width + c2.margin_right + gap + c3.margin_left
    let expected_c3_x = expected_c2_x + 40.0 + 2.0 + 10.0 + 2.0;
    assert!(
        (c3.x - expected_c3_x).abs() < 0.01,
        "Child3 x correct with spacing and gap"
    );
    assert_eq!(c3.y, 0.0, "Child3 y top aligned");
}

#[test]
fn test_positioning_with_padding() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(150.0),
            height: Length::Px(100.0),
            ..Default::default()
        },
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(10.0),
            padding_top: Length::Px(5.0),
            padding_bottom: Length::Px(5.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(50.0),
            height: Length::Px(40.0),
            ..Default::default()
        },
        ..Default::default()
    });

    root.children.push(child);

    LayoutEngine::layout(&mut root, 150.0, 100.0);

    let c = &root.children[0].rect;

    assert_eq!(c.x, 10.0, "Child x offset by parent's padding_left");
    assert_eq!(c.y, 5.0, "Child y offset by parent's padding_top");
}
