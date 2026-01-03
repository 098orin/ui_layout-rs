use ui_layout::*;

#[test]
fn align_items_start() {
    let root = layout_with_align(AlignItems::Start);
    let child = root.children[0].rect;

    assert_eq!(child.x, 10.0);
    assert_eq!(child.width, 40.0);
}

#[test]
fn align_items_center() {
    let root = layout_with_align(AlignItems::Center);
    let child = root.children[0].rect;

    // (180 - 40) / 2 = 70
    assert_eq!(child.x, 10.0 + 70.0);
    assert_eq!(child.width, 40.0);
}

#[test]
fn align_items_end() {
    let root = layout_with_align(AlignItems::End);
    let child = root.children[0].rect;

    // 180 - 40 = 140
    assert_eq!(child.x, 10.0 + 140.0);
    assert_eq!(child.width, 40.0);
}

#[test]
fn align_items_stretch() {
    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            align_items: AlignItems::Stretch,
            spacing: Spacing {
                padding_left: 10.0,
                padding_right: 10.0,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![LayoutNode::new(Style {
            display: Display::Block,
            size: SizeStyle {
                width: None,
                height: Some(20.0),
                ..Default::default()
            },
            ..Default::default()
        })],
    );

    LayoutEngine::layout(&mut root, 200.0, 100.0);

    let child = &root.children[0].rect;

    assert_eq!(child.x, 10.0);
    assert_eq!(child.width, 180.0);
}

fn layout_with_align(align: AlignItems) -> LayoutNode {
    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            align_items: align,
            spacing: Spacing {
                padding_left: 10.0,
                padding_right: 10.0,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![LayoutNode::new(Style {
            display: Display::Block,
            size: SizeStyle {
                width: Some(40.0),
                height: Some(20.0),
                ..Default::default()
            },
            ..Default::default()
        })],
    );

    LayoutEngine::layout(&mut root, 200.0, 100.0);
    root
}
