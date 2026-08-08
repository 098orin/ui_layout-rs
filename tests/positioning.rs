mod common;

use common::*;
use ui_layout::*;

fn size(width: f32, height: f32) -> SizeStyle {
    SizeStyle {
        width: Length::Px(width).into(),
        height: Length::Px(height).into(),
        ..Default::default()
    }
}

#[test]
fn relative_offset_does_not_move_following_sibling() {
    let relative = LayoutNode::new(Style {
        position: PositionStyle {
            kind: Position::Relative,
            top: Length::Px(10.0).into(),
            left: Length::Px(5.0).into(),
            ..Default::default()
        },
        size: size(50.0, 20.0),
        ..Default::default()
    });
    let following = LayoutNode::new(Style {
        size: size(50.0, 30.0),
        ..Default::default()
    });
    let mut root = LayoutNode::with_children(Style::default(), [relative, following]);

    LayoutEngine::layout(&mut root, 300.0, 200.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 5.0);
    assert_eq!(block_box(node(&root, 0)).border_box.y, 10.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 20.0);
    assert_eq!(block_box(&root).content_box.height, 50.0);
}

#[test]
fn absolute_box_is_removed_from_block_flow() {
    let absolute = LayoutNode::new(Style {
        position: PositionStyle {
            kind: Position::Absolute,
            top: Length::Px(10.0).into(),
            left: Length::Px(15.0).into(),
            ..Default::default()
        },
        size: size(50.0, 20.0),
        ..Default::default()
    });
    let in_flow = LayoutNode::new(Style {
        size: size(60.0, 30.0),
        ..Default::default()
    });
    let mut root = LayoutNode::with_children(Style::default(), [absolute, in_flow]);

    LayoutEngine::layout(&mut root, 300.0, 200.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 15.0);
    assert_eq!(block_box(node(&root, 0)).border_box.y, 10.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 0.0);
    assert_eq!(block_box(&root).content_box.height, 30.0);
}

#[test]
fn absolute_descendant_uses_nearest_positioned_ancestor() {
    let absolute = LayoutNode::new(Style {
        position: PositionStyle {
            kind: Position::Absolute,
            top: Length::Px(12.0).into(),
            left: Length::Px(18.0).into(),
            ..Default::default()
        },
        size: size(20.0, 20.0),
        ..Default::default()
    });
    let wrapper = LayoutNode::with_children(
        Style {
            size: size(100.0, 80.0),
            spacing: Spacing {
                margin_left: Length::Px(40.0).into(),
                margin_top: Length::Px(30.0).into(),
                ..Default::default()
            },
            ..Default::default()
        },
        [absolute],
    );
    let mut root = LayoutNode::with_children(
        Style {
            position: PositionStyle {
                kind: Position::Relative,
                ..Default::default()
            },
            size: size(300.0, 200.0),
            ..Default::default()
        },
        [wrapper],
    );

    LayoutEngine::layout(&mut root, 500.0, 400.0);

    let wrapper_box = block_box(node(&root, 0));
    let absolute_box = block_box(node(node(&root, 0), 0));
    assert_eq!(wrapper_box.content_box.x + absolute_box.border_box.x, 18.0);
    assert_eq!(wrapper_box.content_box.y + absolute_box.border_box.y, 12.0);
}

#[test]
fn absolute_box_is_not_a_flex_item() {
    let absolute = LayoutNode::new(Style {
        position: PositionStyle {
            kind: Position::Absolute,
            right: Length::Px(10.0).into(),
            bottom: Length::Px(5.0).into(),
            ..Default::default()
        },
        size: size(30.0, 20.0),
        ..Default::default()
    });
    let in_flow = LayoutNode::new(Style {
        size: size(60.0, 30.0),
        ..Default::default()
    });
    let mut root = LayoutNode::with_children(
        Style {
            display: Display {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            position: PositionStyle {
                kind: Position::Relative,
                ..Default::default()
            },
            size: size(200.0, 100.0),
            ..Default::default()
        },
        [absolute, in_flow],
    );

    LayoutEngine::layout(&mut root, 300.0, 200.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 160.0);
    assert_eq!(block_box(node(&root, 0)).border_box.y, 75.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 0.0);
    assert_eq!(block_box(&root).children_box.width, 60.0);
}

#[test]
fn fixed_box_uses_viewport() {
    let fixed = LayoutNode::new(Style {
        position: PositionStyle {
            kind: Position::Fixed,
            right: Length::Percent(10.0).into(),
            bottom: Length::Percent(10.0).into(),
            ..Default::default()
        },
        size: size(30.0, 40.0),
        ..Default::default()
    });
    let mut root = LayoutNode::with_children(
        Style {
            position: PositionStyle {
                kind: Position::Relative,
                ..Default::default()
            },
            size: size(100.0, 100.0),
            ..Default::default()
        },
        [fixed],
    );

    LayoutEngine::layout(&mut root, 300.0, 200.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 240.0);
    assert_eq!(block_box(node(&root, 0)).border_box.y, 140.0);
}
