mod common;
use common::*;
use ui_layout::*;

// ---------------------------------------------------------------------------
// ContentBox (default) – specified width/height is the content box
// ---------------------------------------------------------------------------

#[test]
fn content_box_no_spacing() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(120.0)),
            height: LengthOrAuto::Length(Length::Px(80.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert_eq!(b.content_box.width, 120.0);
    assert_eq!(b.content_box.height, 80.0);
    assert_eq!(b.border_box.width, 120.0);
    assert_eq!(b.border_box.height, 80.0);
}

#[test]
fn content_box_with_padding_and_border() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(150.0)),
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(20.0),
            padding_top: Length::Px(5.0),
            padding_bottom: Length::Px(15.0),
            border_left: Length::Px(2.0),
            border_right: Length::Px(3.0),
            border_top: Length::Px(1.0),
            border_bottom: Length::Px(4.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    // Specified size is the content box
    assert_eq!(b.content_box.width, 150.0);
    assert_eq!(b.content_box.height, 100.0);
    // Padding box = content + padding
    assert_eq!(b.padding_box.width, 180.0); // 150 + 10 + 20
    assert_eq!(b.padding_box.height, 120.0); // 100 + 5 + 15
    // Border box = padding box + border
    assert_eq!(b.border_box.width, 185.0); // 180 + 2 + 3
    assert_eq!(b.border_box.height, 125.0); // 120 + 1 + 4
}

#[test]
fn content_box_auto_width_stretches() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Auto,
            height: LengthOrAuto::Length(Length::Px(60.0)),
            ..Default::default()
        },
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(10.0),
            border_left: Length::Px(5.0),
            border_right: Length::Px(5.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 200.0, 600.0);

    let b = block_box(&root);
    // Auto width stretches to viewport; content = 200 - 10 - 10 - 5 - 5 = 170
    assert_eq!(b.content_box.width, 170.0);
    assert_eq!(b.border_box.width, 200.0);
}

// ---------------------------------------------------------------------------
// BorderBox – specified width/height is the border box
// ---------------------------------------------------------------------------

#[test]
fn border_box_no_spacing() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(120.0)),
            height: LengthOrAuto::Length(Length::Px(80.0)),
            ..Default::default()
        },
        box_sizing: BoxSizing::BorderBox,
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    // No padding/border → border box == content box
    assert_eq!(b.content_box.width, 120.0);
    assert_eq!(b.content_box.height, 80.0);
    assert_eq!(b.border_box.width, 120.0);
    assert_eq!(b.border_box.height, 80.0);
}

#[test]
fn border_box_with_padding_and_border() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(200.0)),
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        box_sizing: BoxSizing::BorderBox,
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(15.0),
            padding_top: Length::Px(5.0),
            padding_bottom: Length::Px(8.0),
            border_left: Length::Px(2.0),
            border_right: Length::Px(3.0),
            border_top: Length::Px(1.0),
            border_bottom: Length::Px(4.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    // Specified size IS the border box
    assert_eq!(b.border_box.width, 200.0);
    assert_eq!(b.border_box.height, 100.0);
    // Padding box = border box - border
    assert_eq!(b.padding_box.width, 195.0); // 200 - 2 - 3
    assert_eq!(b.padding_box.height, 95.0); // 100 - 1 - 4
    // Content box = padding box - padding
    assert_eq!(b.content_box.width, 170.0); // 195 - 10 - 15
    assert_eq!(b.content_box.height, 82.0); // 95 - 5 - 8
}

#[test]
fn border_box_auto_width_stretches() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Auto,
            height: LengthOrAuto::Length(Length::Px(60.0)),
            ..Default::default()
        },
        box_sizing: BoxSizing::BorderBox,
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(10.0),
            border_left: Length::Px(5.0),
            border_right: Length::Px(5.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 200.0, 600.0);

    let b = block_box(&root);
    // Auto width stretches to viewport as border box
    assert_eq!(b.border_box.width, 200.0);
    // Content = 200 - 5 - 5 - 10 - 10 = 170
    assert_eq!(b.content_box.width, 170.0);
}

#[test]
fn border_box_content_clamped_to_zero() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(10.0)),
            height: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        box_sizing: BoxSizing::BorderBox,
        spacing: Spacing {
            padding_left: Length::Px(8.0),
            padding_right: Length::Px(8.0),
            border_left: Length::Px(2.0),
            border_right: Length::Px(2.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert_eq!(b.border_box.width, 10.0);
    // 10 - 2 - 2 - 8 - 8 = -10 → clamped to 0
    assert_eq!(b.content_box.width, 0.0);
}

// ---------------------------------------------------------------------------
// Equivalence – both sizing modes can produce identical visual results
// ---------------------------------------------------------------------------

#[test]
fn equivalence_same_visual_result() {
    // BorderBox with explicit 200x100
    let mut root_border = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(200.0)),
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        box_sizing: BoxSizing::BorderBox,
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(15.0),
            padding_top: Length::Px(5.0),
            padding_bottom: Length::Px(8.0),
            border_left: Length::Px(2.0),
            border_right: Length::Px(3.0),
            border_top: Length::Px(1.0),
            border_bottom: Length::Px(4.0),
            ..Default::default()
        },
        ..Default::default()
    });

    // ContentBox with content sized to match: 200 - 2 - 3 - 10 - 15 = 170, 100 - 1 - 4 - 5 - 8 = 82
    let mut root_content = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(170.0)),
            height: LengthOrAuto::Length(Length::Px(82.0)),
            ..Default::default()
        },
        box_sizing: BoxSizing::ContentBox,
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(15.0),
            padding_top: Length::Px(5.0),
            padding_bottom: Length::Px(8.0),
            border_left: Length::Px(2.0),
            border_right: Length::Px(3.0),
            border_top: Length::Px(1.0),
            border_bottom: Length::Px(4.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root_border, 800.0, 600.0);
    LayoutEngine::layout(&mut root_content, 800.0, 600.0);

    let b_border = block_box(&root_border);
    let b_content = block_box(&root_content);

    assert_eq!(b_border.border_box.width, b_content.border_box.width);
    assert_eq!(b_border.border_box.height, b_content.border_box.height);
    assert_eq!(b_border.padding_box.width, b_content.padding_box.width);
    assert_eq!(b_border.padding_box.height, b_content.padding_box.height);
    assert_eq!(b_border.content_box.width, b_content.content_box.width);
    assert_eq!(b_border.content_box.height, b_content.content_box.height);
}

// ---------------------------------------------------------------------------
// Children respect parent's content area regardless of box_sizing
// ---------------------------------------------------------------------------

#[test]
fn content_box_child_fits_content_area() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(150.0)),
                ..Default::default()
            },
            box_sizing: BoxSizing::ContentBox,
            spacing: Spacing {
                padding_left: Length::Px(10.0),
                padding_right: Length::Px(10.0),
                border_left: Length::Px(5.0),
                border_right: Length::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![new_child(50.0, 100.0)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert_eq!(b.content_box.width, 200.0);
    // Child should be inside the content area
    let child = node(&root, 0);
    let cb = block_box(child);
    assert_eq!(cb.border_box.x, 0.0);
    assert_eq!(cb.border_box.width, 100.0);
}

#[test]
fn border_box_child_fits_content_area() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(150.0)),
                ..Default::default()
            },
            box_sizing: BoxSizing::BorderBox,
            spacing: Spacing {
                padding_left: Length::Px(10.0),
                padding_right: Length::Px(10.0),
                border_left: Length::Px(5.0),
                border_right: Length::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![new_child(50.0, 100.0)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    // Border box = 200, content = 200 - 5 - 5 - 10 - 10 = 170
    assert_eq!(b.border_box.width, 200.0);
    assert_eq!(b.content_box.width, 170.0);
    // Child should be inside the content area
    let child = node(&root, 0);
    let cb = block_box(child);
    assert_eq!(cb.border_box.x, 0.0);
    assert_eq!(cb.border_box.width, 100.0);
}

#[test]
fn border_box_multiple_children_stack() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            box_sizing: BoxSizing::BorderBox,
            spacing: Spacing {
                padding_top: Length::Px(10.0),
                padding_left: Length::Px(10.0),
                border_top: Length::Px(5.0),
                border_left: Length::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![new_child(40.0, 50.0), new_child(30.0, 60.0)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let child0 = node(&root, 0);
    let child1 = node(&root, 1);
    let cb0 = block_box(child0);
    let cb1 = block_box(child1);

    // Both children start at x = 0 (left edge of parent's content area)
    assert_eq!(cb0.border_box.x, 0.0);
    assert_eq!(cb1.border_box.x, 0.0);
    // First child starts at y = 0 (top edge of parent's content area)
    assert_eq!(cb0.border_box.y, 0.0);
    // Second child stacks below: 0 + 40 = 40
    assert_eq!(cb1.border_box.y, 40.0);
}
