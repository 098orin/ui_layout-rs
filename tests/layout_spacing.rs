use ui_layout::*;

#[test]
fn layout_with_spacing() {
    // Root node with padding
    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            spacing: Spacing {
                padding_top: 10.0,
                padding_bottom: 10.0,
                padding_left: 5.0,
                padding_right: 5.0,
                ..Default::default()
            },
            ..Style::default()
        },
        vec![
            // Child 1 with margin
            LayoutNode::new(Style {
                display: Display::Block,
                size: SizeStyle {
                    height: Some(50.0),
                    ..Default::default()
                },
                spacing: Spacing {
                    margin_top: 2.0,
                    margin_bottom: 3.0,
                    margin_left: 4.0,
                    margin_right: 5.0,
                    ..Default::default()
                },
                ..Style::default()
            }),
            // Child 2 with margin
            LayoutNode::new(Style {
                display: Display::Block,
                size: SizeStyle {
                    height: Some(30.0),
                    ..Default::default()
                },
                spacing: Spacing {
                    margin_top: 1.0,
                    margin_bottom: 2.0,
                    margin_left: 3.0,
                    margin_right: 4.0,
                    ..Default::default()
                },
                ..Style::default()
            }),
        ],
    );

    // Layout with root size 100x200
    LayoutEngine::layout(&mut root, 100.0, 200.0);

    let child1 = &root.children[0].rect;
    let child2 = &root.children[1].rect;

    // Root padding affects child x/y
    assert_eq!(child1.x, 5.0 + 4.0); // root.padding_left + child.margin_left
    assert_eq!(child1.y, 10.0 + 2.0); // root.padding_top + child.margin_top
    assert_eq!(child1.width, 100.0 - 5.0 - 5.0); // root width minus horizontal padding
    assert_eq!(child1.height, 50.0);

    // Child2 position accounts for child1 height + margins
    assert_eq!(child2.x, 5.0 + 3.0); // root.padding_left + child2.margin_left
    assert_eq!(child2.y, 10.0 + 2.0 + 50.0 + 3.0 + 1.0);
    // root.padding_top + child1.margin_top + child1.height + child1.margin_bottom + child2.margin_top
    assert_eq!(child2.width, 100.0 - 5.0 - 5.0); // root width minus horizontal padding
    assert_eq!(child2.height, 30.0);
}
