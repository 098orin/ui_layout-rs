use ui_layout::*;

#[test]
fn flex_column_row_gap_applied() {
    let root = layout_with_gap(12.0);

    let first = &root.children[0].rect;
    let second = &root.children[1].rect;

    // padding_top = 10
    assert_eq!(first.y, 10.0);

    // 10 (padding)
    // + 20 (height)
    // + 12 (row_gap)
    assert_eq!(second.y, 10.0 + 20.0 + 12.0);
}

#[test]
fn flex_column_zero_gap() {
    let root = layout_with_gap(0.0);

    let first = &root.children[0].rect;
    let second = &root.children[1].rect;

    assert_eq!(second.y, first.y + first.height);
}

fn layout_with_gap(row_gap: f32) -> LayoutNode {
    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            row_gap,
            spacing: Spacing {
                padding_top: 10.0,
                padding_bottom: 10.0,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![
            LayoutNode::new(Style {
                display: Display::Block,
                size: SizeStyle {
                    width: Length::Px(40.0),
                    height: Length::Px(20.0),
                    ..Default::default()
                },
                ..Default::default()
            }),
            LayoutNode::new(Style {
                display: Display::Block,
                size: SizeStyle {
                    width: Length::Px(40.0),
                    height: Length::Px(20.0),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ],
    );

    LayoutEngine::layout(&mut root, 200.0, 100.0);
    root
}
