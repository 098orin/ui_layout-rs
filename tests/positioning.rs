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
            display: Display::OutsideInner {
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

#[test]
fn absolute_auto_width_shrink_wraps_content() {
    let absolute = LayoutNode::with_children(
        Style {
            position: PositionStyle {
                kind: Position::Absolute,
                top: Length::Px(10.0).into(),
                left: Length::Px(15.0).into(),
                ..Default::default()
            },
            ..Default::default()
        },
        [LayoutNode::new(Style {
            size: size(40.0, 20.0),
            ..Default::default()
        })],
    );
    let mut root = LayoutNode::with_children(Style::default(), [absolute]);

    LayoutEngine::layout(&mut root, 300.0, 200.0);

    let absolute_box = block_box(node(&root, 0));
    assert_eq!(absolute_box.border_box.x, 15.0);
    assert_eq!(absolute_box.border_box.y, 10.0);
    assert_eq!(absolute_box.border_box.width, 40.0);
    assert_eq!(absolute_box.border_box.height, 20.0);
}

#[test]
fn fixed_auto_width_shrink_wraps_content() {
    let fixed = LayoutNode::with_children(
        Style {
            position: PositionStyle {
                kind: Position::Fixed,
                right: Length::Px(20.0).into(),
                bottom: Length::Px(20.0).into(),
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(20.0),
                padding_right: Length::Px(20.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [LayoutNode::new(Style {
            size: size(50.0, 30.0),
            ..Default::default()
        })],
    );
    let mut root = LayoutNode::with_children(Style::default(), [fixed]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let fixed_box = block_box(node(&root, 0));
    assert_eq!(fixed_box.border_box.width, 90.0);
    assert_eq!(fixed_box.border_box.x, 690.0);
    assert_eq!(fixed_box.border_box.y, 550.0);
}

// --- Sticky positioning ---

fn sticky_style(
    top: LengthOrAuto,
    right: LengthOrAuto,
    bottom: LengthOrAuto,
    left: LengthOrAuto,
) -> Style {
    Style {
        position: PositionStyle {
            kind: Position::Sticky,
            top,
            right,
            bottom,
            left,
        },
        size: size(50.0, 20.0),
        ..Default::default()
    }
}

#[test]
fn sticky_records_resolved_edges_without_moving_the_box() {
    let sticky = LayoutNode::new(sticky_style(
        Length::Px(10.0).into(),
        LengthOrAuto::Auto,
        LengthOrAuto::Auto,
        Length::Px(5.0).into(),
    ));
    let following = LayoutNode::new(Style {
        size: size(50.0, 30.0),
        ..Default::default()
    });
    let mut root = LayoutNode::with_children(Style::default(), [sticky, following]);

    LayoutEngine::layout(&mut root, 300.0, 200.0);

    let sticky_box = block_box(node(&root, 0));
    assert_eq!(
        (sticky_box.border_box.x, sticky_box.border_box.y),
        (0.0, 0.0)
    );
    let edges = sticky_box
        .sticky_edges
        .expect("sticky edges should be recorded");
    assert_eq!(
        (edges.left, edges.top, edges.right, edges.bottom),
        (Some(5.0), Some(10.0), None, None)
    );
    // Sticky participates in normal flow, so the sibling still follows it.
    assert_eq!(block_box(node(&root, 1)).border_box.y, 20.0);
    assert!(
        format!("{:#}", node(&root, 0)).contains("sticky(left: 5, top: 10)"),
        "layout box formatting should include the sticky edges"
    );
}

#[test]
fn sticky_percentage_offsets_resolve_against_containing_block() {
    let sticky = LayoutNode::new(sticky_style(
        Length::Percent(10.0).into(),
        Length::Percent(5.0).into(),
        LengthOrAuto::Auto,
        Length::Percent(25.0).into(),
    ));
    let mut root = LayoutNode::with_children(
        Style {
            position: PositionStyle {
                kind: Position::Relative,
                ..Default::default()
            },
            size: size(200.0, 100.0),
            ..Default::default()
        },
        [sticky],
    );

    LayoutEngine::layout(&mut root, 300.0, 200.0);

    let edges = block_box(node(&root, 0))
        .sticky_edges
        .expect("sticky edges should be recorded");
    assert_eq!(edges.left, Some(50.0));
    assert_eq!(edges.top, Some(10.0));
    assert_eq!(edges.right, Some(10.0));
    assert_eq!(edges.bottom, None);
}

#[test]
fn sticky_without_offsets_records_all_none() {
    let sticky = LayoutNode::new(sticky_style(
        LengthOrAuto::Auto,
        LengthOrAuto::Auto,
        LengthOrAuto::Auto,
        LengthOrAuto::Auto,
    ));
    let mut root = LayoutNode::with_children(Style::default(), [sticky]);

    LayoutEngine::layout(&mut root, 300.0, 200.0);

    let edges = block_box(node(&root, 0))
        .sticky_edges
        .expect("sticky edges should be recorded");
    assert_eq!(
        (edges.left, edges.top, edges.right, edges.bottom),
        (None, None, None, None)
    );
}

#[test]
fn sticky_inline_box_records_edges() {
    let inline = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            position: PositionStyle {
                kind: Position::Sticky,
                top: Length::Px(3.0).into(),
                ..Default::default()
            },
            ..Default::default()
        },
        [LayoutChild::from(fragment(20.0, 10.0))],
    );
    let mut root = LayoutNode::with_children(Style::default(), [inline]);

    LayoutEngine::layout(&mut root, 300.0, 200.0);

    let inline_box = inline_box_model(node(&root, 0));
    let edges = inline_box
        .sticky_edges
        .expect("sticky edges should be recorded");
    assert_eq!(
        (edges.left, edges.top, edges.right, edges.bottom),
        (None, Some(3.0), None, None)
    );
}

// --- Out-of-flow stretching between opposite offsets ---

#[test]
fn fixed_inset_zero_fills_the_viewport() {
    let fixed = LayoutNode::new(Style {
        position: PositionStyle {
            kind: Position::Fixed,
            top: Length::Px(0.0).into(),
            right: Length::Px(0.0).into(),
            bottom: Length::Px(0.0).into(),
            left: Length::Px(0.0).into(),
        },
        ..Default::default()
    });
    let mut root = LayoutNode::with_children(Style::default(), [fixed]);

    LayoutEngine::layout(&mut root, 300.0, 200.0);

    let fixed_box = block_box(node(&root, 0));
    assert_eq!(fixed_box.border_box, rect(0.0, 0.0, 300.0, 200.0));
}

#[test]
fn absolute_auto_width_stretches_between_left_and_right() {
    let absolute = LayoutNode::new(Style {
        position: PositionStyle {
            kind: Position::Absolute,
            top: Length::Px(30.0).into(),
            right: Length::Px(20.0).into(),
            left: Length::Px(10.0).into(),
            ..Default::default()
        },
        size: SizeStyle {
            height: Length::Px(40.0).into(),
            ..Default::default()
        },
        ..Default::default()
    });
    let mut root = LayoutNode::with_children(
        Style {
            size: size(300.0, 200.0),
            ..Default::default()
        },
        [absolute],
    );

    LayoutEngine::layout(&mut root, 300.0, 200.0);

    let absolute_box = block_box(node(&root, 0));
    assert_eq!(absolute_box.border_box, rect(10.0, 30.0, 270.0, 40.0));
}

#[test]
fn absolute_auto_height_stretches_between_top_and_bottom() {
    let absolute = LayoutNode::new(Style {
        position: PositionStyle {
            kind: Position::Absolute,
            top: Length::Px(10.0).into(),
            bottom: Length::Px(20.0).into(),
            left: Length::Px(15.0).into(),
            ..Default::default()
        },
        size: SizeStyle {
            width: Length::Px(50.0).into(),
            ..Default::default()
        },
        ..Default::default()
    });
    let mut root = LayoutNode::with_children(
        Style {
            size: size(300.0, 200.0),
            ..Default::default()
        },
        [absolute],
    );

    LayoutEngine::layout(&mut root, 300.0, 200.0);

    let absolute_box = block_box(node(&root, 0));
    assert_eq!(absolute_box.border_box, rect(15.0, 10.0, 50.0, 170.0));
}

#[test]
fn absolute_stretch_subtracts_margins_border_and_padding() {
    let absolute = LayoutNode::new(Style {
        position: PositionStyle {
            kind: Position::Absolute,
            top: Length::Px(10.0).into(),
            right: Length::Px(10.0).into(),
            bottom: Length::Px(10.0).into(),
            left: Length::Px(10.0).into(),
        },
        spacing: Spacing {
            margin_left: Length::Px(5.0).into(),
            margin_right: Length::Px(5.0).into(),
            border_left: Length::Px(2.0),
            border_right: Length::Px(2.0),
            padding_left: Length::Px(3.0),
            padding_right: Length::Px(3.0),
            ..Default::default()
        },
        ..Default::default()
    });
    let mut root = LayoutNode::with_children(
        Style {
            size: size(300.0, 200.0),
            ..Default::default()
        },
        [absolute],
    );

    LayoutEngine::layout(&mut root, 300.0, 200.0);

    let absolute_box = block_box(node(&root, 0));
    // 300 - left(10) - right(10) - margins(10) = 270 border-box width.
    assert_eq!(absolute_box.border_box.width, 270.0);
    // 270 - border(4) - padding(6) = 260 content-box width.
    assert_eq!(absolute_box.content_box.width, 260.0);
}

#[test]
fn explicit_size_wins_over_opposite_offset_stretch() {
    let absolute = LayoutNode::new(Style {
        position: PositionStyle {
            kind: Position::Absolute,
            top: Length::Px(0.0).into(),
            right: Length::Px(10.0).into(),
            bottom: Length::Px(0.0).into(),
            left: Length::Px(10.0).into(),
        },
        size: size(40.0, 30.0),
        ..Default::default()
    });
    let mut root = LayoutNode::with_children(
        Style {
            size: size(300.0, 200.0),
            ..Default::default()
        },
        [absolute],
    );

    LayoutEngine::layout(&mut root, 300.0, 200.0);

    let absolute_box = block_box(node(&root, 0));
    assert_eq!(absolute_box.border_box, rect(10.0, 0.0, 40.0, 30.0));
}
