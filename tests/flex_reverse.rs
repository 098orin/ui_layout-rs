mod common;
use common::*;
use ui_layout::*;

#[test]
fn row_reverse_positions_children_right_to_left() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });
    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            ..Default::default()
        },
        ..Default::default()
    });
    let mut root = LayoutNode::with_children(
        flex_container(300.0, 50.0, FlexDirection::RowReverse),
        [child1, child2],
    );
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // RowReverse flex-start: first child at rightmost
    // Container width 300: Child1 (50) at x=250, Child2 (60) at x=190
    assert_eq!(block_box(node(&root, 0)).border_box.x, 250.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 190.0);
}

#[test]
fn row_reverse_flex_end_positions_children_left_to_right() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });
    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
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
            flex_direction: FlexDirection::RowReverse,
            justify_content: JustifyContent::End,
            ..Default::default()
        },
        [child1, child2],
    );
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // RowReverse flex-end: items packed toward left, first child leftmost
    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 50.0);
}

#[test]
fn column_reverse_positions_children_bottom_to_top() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });
    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(60.0)),
            ..Default::default()
        },
        ..Default::default()
    });
    let mut root = LayoutNode::with_children(
        flex_container(200.0, 300.0, FlexDirection::ColumnReverse),
        [child1, child2],
    );
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // ColumnReverse flex-start: first child at bottom
    // Container height 300: Child1 (50) at y=250, Child2 (60) at y=190
    assert_eq!(block_box(node(&root, 0)).border_box.y, 250.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 190.0);
}

#[test]
fn row_reverse_center() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });
    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
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
            flex_direction: FlexDirection::RowReverse,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        [child1, child2],
    );
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // remaining = 300 - 110 = 190, start_offset = 95
    // Reversed: cursor = 300 - 95 = 205
    // Child1 at 205 - 50 = 155, Child2 at 155 - 60 = 95
    assert_eq!(block_box(node(&root, 0)).border_box.x, 155.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 95.0);
}

#[test]
fn row_reverse_column_reverse_with_gap() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });
    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
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
            flex_direction: FlexDirection::RowReverse,
            column_gap: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        [child1, child2],
    );
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // RowReverse flex-start with gap 10:
    // Container 300, items 50+10+60=120, remaining=180, start_offset=0
    // Reversed cursor = 300 - 0 = 300
    // Child1: cursor = 300-50 = 250
    // cursor = 250 - 10 = 240
    // Child2: cursor = 240-60 = 180
    assert_eq!(block_box(node(&root, 0)).border_box.x, 250.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 180.0);
}

#[test]
fn column_reverse_flex_end() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });
    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(60.0)),
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
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(300.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::End,
            ..Default::default()
        },
        [child1, child2],
    );
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // ColumnReverse flex-end: items at top, first child at top
    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 50.0);
}
