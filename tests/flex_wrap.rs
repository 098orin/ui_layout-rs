mod common;
use common::*;
use ui_layout::*;

#[test]
fn flex_wrap_border_box_basis() {
    let item_style = || Style {
        size: SizeStyle {
            width: LengthOrAuto::Auto,
            height: LengthOrAuto::Length(Length::Px(80.0)),
            ..Default::default()
        },
        box_sizing: BoxSizing::BorderBox,
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(10.0),
            padding_top: Length::Px(10.0),
            padding_bottom: Length::Px(10.0),
            border_left: Length::Px(10.0),
            border_right: Length::Px(10.0),
            border_top: Length::Px(10.0),
            border_bottom: Length::Px(10.0),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_basis: LengthOrAuto::Length(Length::Px(120.0)),
            flex_grow: 0.0,
            flex_shrink: 0.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let children = (0..5)
        .map(|_| LayoutNode::new(item_style()))
        .collect::<Vec<_>>();

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::OutsideInner {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(500.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            spacing: Spacing {
                border_left: Length::Px(4.0),
                border_right: Length::Px(4.0),
                border_top: Length::Px(4.0),
                border_bottom: Length::Px(4.0),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..Default::default()
        },
        children,
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // The container's content box is 500px wide.
    // Four 120px border-box flex items fit on the first line.
    // The fifth item wraps onto the second line.
    assert_eq!(block_box(&root).content_box.width, 500.0);
    assert_eq!(block_box(&root).border_box.width, 508.0);

    for i in 0..4 {
        let child = block_box(node(&root, i));

        assert_eq!(child.border_box.width, 120.0);
        assert_eq!(child.border_box.height, 80.0);
        assert_eq!(child.border_box.x, i as f32 * 120.0);
        assert_eq!(child.border_box.y, 0.0);

        // 120 - 10 - 10 - 10 - 10 = 80
        assert_eq!(child.content_box.width, 80.0);

        // 80 - 10 - 10 - 10 - 10 = 40
        assert_eq!(child.content_box.height, 40.0);
    }

    let fifth = block_box(node(&root, 4));

    assert_eq!(fifth.border_box.width, 120.0);
    assert_eq!(fifth.border_box.height, 80.0);
    assert_eq!(fifth.border_box.x, 0.0);
    assert_eq!(fifth.border_box.y, 80.0);

    assert_eq!(fifth.content_box.width, 80.0);
    assert_eq!(fifth.content_box.height, 40.0);
}

#[test]
fn flex_wrap_content_box_basis() {
    let item_style = || Style {
        size: SizeStyle {
            width: LengthOrAuto::Auto,
            height: LengthOrAuto::Length(Length::Px(80.0)),
            ..Default::default()
        },
        // Default is ContentBox.
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(10.0),
            padding_top: Length::Px(10.0),
            padding_bottom: Length::Px(10.0),
            border_left: Length::Px(10.0),
            border_right: Length::Px(10.0),
            border_top: Length::Px(10.0),
            border_bottom: Length::Px(10.0),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_basis: LengthOrAuto::Length(Length::Px(120.0)),
            flex_grow: 0.0,
            flex_shrink: 0.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let children = (0..5)
        .map(|_| LayoutNode::new(item_style()))
        .collect::<Vec<_>>();

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::OutsideInner {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(500.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            spacing: Spacing {
                border_left: Length::Px(4.0),
                border_right: Length::Px(4.0),
                border_top: Length::Px(4.0),
                border_bottom: Length::Px(4.0),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..Default::default()
        },
        children,
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // The flex basis is the content-box width.
    //
    // Each item is:
    //   content: 120px
    //   padding: 20px
    //   border: 20px
    //   border-box: 160px
    //
    // Therefore only three items fit on the first line:
    //   160 * 3 = 480
    //   160 * 4 = 640 > 500

    for i in 0..3 {
        let child = block_box(node(&root, i));

        assert_eq!(child.content_box.width, 120.0);
        assert_eq!(child.border_box.width, 160.0);
        assert_eq!(child.content_box.height, 80.0);
        assert_eq!(child.border_box.height, 120.0);

        assert_eq!(child.border_box.x, i as f32 * 160.0);
        assert_eq!(child.border_box.y, 0.0);
    }

    let fourth = block_box(node(&root, 3));

    assert_eq!(fourth.content_box.width, 120.0);
    assert_eq!(fourth.border_box.width, 160.0);
    assert_eq!(fourth.content_box.height, 80.0);
    assert_eq!(fourth.border_box.height, 120.0);

    assert_eq!(fourth.border_box.x, 0.0);
    assert_eq!(fourth.border_box.y, 120.0);

    let fifth = block_box(node(&root, 4));

    assert_eq!(fifth.content_box.width, 120.0);
    assert_eq!(fifth.border_box.width, 160.0);
    assert_eq!(fifth.content_box.height, 80.0);
    assert_eq!(fifth.border_box.height, 120.0);

    assert_eq!(fifth.border_box.x, 160.0);
    assert_eq!(fifth.border_box.y, 120.0);
}

#[test]
// A wrapped second line must not be shifted to the right because its items
// happen to share a Y coordinate with the first line (zero-height first line).
fn flex_wrap_line_of_zero_height_is_not_shifted_right() {
    let children = vec![
        LayoutNode::new(Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(800.0)),
                height: LengthOrAuto::Length(Length::Px(0.0)),
                ..Default::default()
            },
            box_sizing: BoxSizing::BorderBox,
            ..Default::default()
        }),
        LayoutNode::new(Style {
            size: SizeStyle {
                width: LengthOrAuto::Auto,
                height: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            box_sizing: BoxSizing::BorderBox,
            item_style: ItemStyle {
                flex_grow: 1.0,
                flex_basis: LengthOrAuto::Length(Length::Px(800.0)),
                ..Default::default()
            },
            ..Default::default()
        }),
    ];

    let mut root = LayoutNode::with_children(
        Style {
            display: Display::OutsideInner {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(800.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            box_sizing: BoxSizing::BorderBox,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..Default::default()
        },
        children,
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Both items wrap onto their own lines starting at x == 0.
    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 0.0);
}
