use ui_layout::*;

#[test]
fn nested_flex_coordinate_bug() {
    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            spacing: Spacing {
                padding_top: 5.0,
                padding_bottom: 5.0,
                padding_left: 10.0,
                padding_right: 10.0,
                ..Default::default()
            },
            row_gap: 6.0,
            ..Default::default()
        },
        vec![
            // Row child
            LayoutNode::new(Style {
                display: Display::Flex {
                    flex_direction: FlexDirection::Row,
                },
                spacing: Spacing {
                    padding_top: 2.0,
                    padding_bottom: 2.0,
                    padding_left: 3.0,
                    padding_right: 3.0,
                    ..Default::default()
                },
                align_items: AlignItems::End,
                ..Default::default()
            }),
        ],
    );

    // Row child has three children with different heights and margins
    let row = &mut root.children[0];
    row.children = vec![
        LayoutNode::new(Style {
            size: SizeStyle {
                width: Length::Px(20.0),
                height: Length::Px(10.0),
                ..Default::default()
            },
            spacing: Spacing {
                margin_top: 1.0,
                margin_bottom: 1.0,
                ..Default::default()
            },
            ..Default::default()
        }),
        LayoutNode::new(Style {
            size: SizeStyle {
                width: Length::Px(30.0),
                height: Length::Px(20.0),
                ..Default::default()
            },
            spacing: Spacing {
                margin_top: 2.0,
                margin_bottom: 2.0,
                ..Default::default()
            },
            ..Default::default()
        }),
        LayoutNode::new(Style {
            size: SizeStyle {
                width: Length::Px(40.0),
                height: Length::Px(15.0),
                ..Default::default()
            },
            spacing: Spacing {
                margin_top: 1.0,
                margin_bottom: 1.0,
                ..Default::default()
            },
            ..Default::default()
        }),
    ];

    LayoutEngine::layout(&mut root, 200.0, 100.0);

    dbg!(&root);

    let r = &root.children[0];

    let a = &r.children[0].rect;
    let b = &r.children[1].rect;
    let c = &r.children[2].rect;

    // stretched
    assert_eq!(r.rect.width, 200.0 - 10.0 - 10.0);
    // max height of the children + padding + margin
    assert_eq!(r.rect.height, 20.0 + 2.0 + 2.0 + 2.0 + 2.0);

    // padding top + margin top
    assert_eq!(a.y, 15.0);
    assert_eq!(b.y, 2.0 + 0.0 + 2.0); // b: remaining = 0
    assert_eq!(c.y, 10.0);

    assert_eq!(a.x, 3.0);
    assert_eq!(b.x, 3.0 + 20.0);
    assert_eq!(c.x, 3.0 + 20.0 + 30.0);
}

#[test]
fn flex_column_coordinates() {
    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Column,
            },
            spacing: Spacing {
                padding_top: 10.0,
                padding_bottom: 10.0,
                ..Default::default()
            },
            row_gap: 5.0,
            ..Default::default()
        },
        vec![
            LayoutNode::new(Style {
                size: SizeStyle {
                    height: Length::Px(50.0),
                    ..Default::default()
                },
                spacing: Spacing {
                    margin_top: 2.0,
                    margin_bottom: 3.0,
                    ..Default::default()
                },
                ..Default::default()
            }),
            LayoutNode::new(Style {
                size: SizeStyle {
                    height: Length::Px(30.0),
                    ..Default::default()
                },
                spacing: Spacing {
                    margin_top: 1.0,
                    margin_bottom: 1.0,
                    ..Default::default()
                },
                ..Default::default()
            }),
        ],
    );

    LayoutEngine::layout(&mut root, 100.0, 200.0);

    let a = &root.children[0].rect;
    let b = &root.children[1].rect;

    // cursor_y = padding_top (10) + margin_top(2) = 12
    assert_eq!(a.y, 12.0);
    // next cursor_y = 12 + 50 + margin_bottom(3) + gap(5) + margin_top(1) = 71
    assert_eq!(b.y, 71.0);
}

#[test]
fn flex_row_coordinates_align() {
    let mut root = LayoutNode::with_children(
        Style {
            display: Display::Flex {
                flex_direction: FlexDirection::Row,
            },
            spacing: Spacing {
                padding_left: 10.0,
                padding_right: 10.0,
                ..Default::default()
            },
            column_gap: 4.0,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        vec![
            LayoutNode::new(Style {
                size: SizeStyle {
                    height: Length::Px(20.0),
                    width: Length::Px(30.0),
                    ..Default::default()
                },
                ..Default::default()
            }),
            LayoutNode::new(Style {
                size: SizeStyle {
                    height: Length::Px(40.0),
                    width: Length::Px(50.0),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ],
    );

    LayoutEngine::layout(&mut root, 200.0, 100.0);

    let a = &root.children[0].rect;
    let b = &root.children[1].rect;

    // cross axis: container height = 100, padding_top=0, padding_bottom=0
    // a height=20, center → offset = (100-20)/2 = 40
    assert_eq!(a.y, 40.0);
    // b height=40, center → offset = (100-40)/2 = 30
    assert_eq!(b.y, 30.0);
    // main axis: cursor = padding_left=10 → first x=10
    assert_eq!(a.x, 10.0);
    // second x = 10 + 30 + gap 4 = 44
    assert_eq!(b.x, 44.0);
}
