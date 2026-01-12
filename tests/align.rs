use ui_layout::*;

#[test]
fn test_align_items_start() {
    let root = setup_container(&[40.0, 60.0], AlignItems::Start);

    let c1 = &root.children[0].rect;
    let c2 = &root.children[1].rect;

    assert_eq!(c1.y, 0.0, "Start: Child1 top aligned");
    assert_eq!(c2.y, 0.0, "Start: Child2 top aligned");
}

#[test]
fn test_align_items_center() {
    let root = setup_container(&[40.0, 60.0], AlignItems::Center);

    let c1 = &root.children[0].rect;
    let c2 = &root.children[1].rect;

    assert!(
        (c1.y - (100.0 - 40.0) / 2.0).abs() < 0.01,
        "Center: Child1 vertically centered"
    );
    assert!(
        (c2.y - (100.0 - 60.0) / 2.0).abs() < 0.01,
        "Center: Child2 vertically centered"
    );
}

#[test]
fn test_align_items_end() {
    let root = setup_container(&[40.0, 60.0], AlignItems::End);

    let c1 = &root.children[0].rect;
    let c2 = &root.children[1].rect;

    assert!(
        (c1.y - (100.0 - 40.0)).abs() < 0.01,
        "End: Child1 bottom aligned"
    );
    assert!(
        (c2.y - (100.0 - 60.0)).abs() < 0.01,
        "End: Child2 bottom aligned"
    );
}

#[test]
fn test_align_items_stretch() {
    let mut root = LayoutNode::new(Style {
        display: Display::Flex {
            flex_direction: FlexDirection::Row,
        },
        size: SizeStyle {
            width: Length::Px(300.0),
            height: Length::Px(100.0),
            ..Default::default()
        },
        align_items: AlignItems::Stretch,
        ..Default::default()
    });

    for _ in 0..2 {
        let child = LayoutNode::new(Style {
            size: SizeStyle {
                width: Length::Px(50.0),
                height: Length::Auto,
                ..Default::default()
            },
            ..Default::default()
        });
        root.children.push(child);
    }

    LayoutEngine::layout(&mut root, 300.0, 100.0);

    let c1 = &root.children[0].rect;
    let c2 = &root.children[1].rect;

    assert_eq!(c1.y, 0.0, "Stretch: Child1 top aligned");
    assert_eq!(c2.y, 0.0, "Stretch: Child2 top aligned");
    assert_eq!(c1.height, 100.0, "Stretch: Child1 height stretched");
    assert_eq!(c2.height, 100.0, "Stretch: Child2 height stretched");
}

fn setup_container(child_heights: &[f32], align: AlignItems) -> LayoutNode {
    let mut root = LayoutNode::new(Style {
        display: Display::Flex {
            flex_direction: FlexDirection::Row,
        },
        size: SizeStyle {
            width: Length::Px(300.0),
            height: Length::Px(100.0),
            ..Default::default()
        },
        align_items: align,
        ..Default::default()
    });

    for &h in child_heights {
        let child = LayoutNode::new(Style {
            size: SizeStyle {
                width: Length::Px(50.0),
                height: Length::Px(h),
                ..Default::default()
            },
            ..Default::default()
        });
        root.children.push(child);
    }

    LayoutEngine::layout(&mut root, 300.0, 100.0);

    root
}
