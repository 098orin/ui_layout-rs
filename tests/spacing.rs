use ui_layout::*;

fn node<'a>(n: &'a LayoutNode, idx: usize) -> &'a LayoutNode {
    n.children[idx].node().expect("expected node child")
}

fn block_box(n: &LayoutNode) -> &BoxModel {
    match &n.layout_box {
        LayoutBox::BlockBox(b) => b,
        _ => panic!("expected block box"),
    }
}

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.01
}

// ============================================
// Block spacing tests
// ============================================

#[test]
fn block_margin_left_shifts_child() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 15.0);
}

#[test]
fn block_margin_auto_left_right_centers_child() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(80.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Auto,
            margin_right: LengthOrAuto::Auto,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 60.0);
}

#[test]
fn block_margin_collapse_three_siblings_with_uneven_margins() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(10.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(25.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(5.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Margins collapse: between child1(30) and child2(10) => 30
    // Between child2(25) and child3(5) => 25
    // Last child's bottom margin (20) should be included in parent height
    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 40.0);
    assert_eq!(block_box(node(&root, 2)).border_box.y, 85.0);
    assert_eq!(block_box(&root).content_box.height, 120.0);
}

#[test]
fn block_auto_height_respects_child_margin_bottom() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // 50 (child height) + 30 (child margin-bottom) = 80
    assert_eq!(block_box(&root).content_box.height, 80.0);
    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
}

#[test]
fn block_auto_height_respects_child_margin_top() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // 50 (child height) + 20 (margin top) = 70
    assert_eq!(block_box(&root).content_box.height, 70.0);
    assert_eq!(block_box(node(&root, 0)).border_box.y, 20.0);
}

// ============================================
// Flex spacing tests
// ============================================

#[test]
fn flex_row_margin_on_child_affects_spacing_between_items() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_right: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
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
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 0)).border_box.width, 50.0);
    // child2 should start after child1 + child1's margin_right
    assert_eq!(block_box(node(&root, 1)).border_box.x, 70.0);
}

#[test]
fn flex_row_margin_left_shifts_child() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Length(Length::Px(15.0)),
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
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    // child2 should be shifted right by its margin_left
    assert_eq!(block_box(node(&root, 1)).border_box.x, 65.0);
}

#[test]
fn flex_children_with_margins_justify_center() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_right: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
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
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Total space used: 50 (child1) + 20 (child1 margin) + 60 (child2) = 130
    // Center: (300 - 130) / 2 = 85
    // Child1 at x = 85
    assert!(approx_eq(block_box(node(&root, 0)).border_box.x, 85.0));
    assert!(approx_eq(block_box(node(&root, 1)).border_box.x, 155.0));
}

#[test]
fn flex_children_with_margins_justify_space_between() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_right: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
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
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceBetween,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Total space used: 50 + 10 + 60 = 120
    // SpaceBetween with 2 items: remaining = 300 - 120 = 180
    // gap = 180 / 1 = 180
    // Child1 at x = 0
    // Child2 at x = 50 + 10 + 180 = 240
    assert!(approx_eq(block_box(node(&root, 0)).border_box.x, 0.0));
    assert!(approx_eq(block_box(node(&root, 1)).border_box.x, 240.0));
}

#[test]
fn flex_children_with_margins_justify_space_evenly() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(40.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_right: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
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
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceEvenly,
            ..Default::default()
        },
        [child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Total space used: 40 + 10 + 50 + 60 = 160
    // SpaceEvenly: gaps = 4 (items+1)
    // gap = (300 - 160) / 4 = 35
    // Child1 at x = 35
    // Child2 at x = 35 + 40 + 10 + 35 = 120
    // Child3 at x = 120 + 50 + 35 = 205
    assert!(approx_eq(block_box(node(&root, 0)).border_box.x, 35.0));
    assert!(approx_eq(block_box(node(&root, 1)).border_box.x, 120.0));
    assert!(approx_eq(block_box(node(&root, 2)).border_box.x, 205.0));
}

#[test]
fn flex_auto_margin_with_fixed_margin_on_sibling() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_right: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Auto,
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
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Total space used: 50 (border_box) + 20 (fixed margin right) + 60 (border_box) = 130
    // Remaining: 300 - 130 = 170
    // Auto margin on child2's left: 170
    // Child1 at x = 0
    // Child2 at x = 50 + 20 + 170 = 240
    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 0)).border_box.width, 50.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 240.0);
    assert_eq!(block_box(node(&root, 1)).border_box.width, 60.0);
}

#[test]
fn flex_column_margin_between_items() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(40.0)),
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
                height: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    // child2 should be below child1 + child1's margin_bottom
    assert_eq!(block_box(node(&root, 1)).border_box.y, 45.0);
}

#[test]
fn flex_column_margin_top_shifts_child() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(40.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(20.0)),
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
                height: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    // child2 should be at child1's bottom + child2's margin_top
    assert_eq!(block_box(node(&root, 1)).border_box.y, 50.0);
}

#[test]
fn flex_margin_with_gap_interaction() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(40.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_right: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
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
            flex_direction: FlexDirection::Row,
            column_gap: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // child1 at x=0, width=40
    // margin_right=10, gap=20
    // child2 at x = 40 + 10 + 20 = 70
    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 0)).border_box.width, 40.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 70.0);
}

#[test]
fn flex_padding_on_container_affects_children_box() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
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
                height: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(20.0),
                padding_top: Length::Px(15.0),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(&root).content_box.x, 20.0);
    assert_eq!(block_box(&root).content_box.y, 15.0);
    // children_box should reflect children extent relative to content box
    assert_eq!(block_box(&root).children_box.width, 50.0);
    assert_eq!(block_box(&root).children_box.height, 30.0);
}

#[test]
fn flex_padding_with_multiple_children_justify_center() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(40.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
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
            spacing: Spacing {
                padding_left: Length::Px(30.0),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // content_box.x = padding_left = 30
    // content width = 300 (explicit ContentBox)
    // Children are positioned relative to border box (not content box)
    // Children: total = 40+60=100, remaining = 300-100=200, center offset = 100
    // cursor starts at offset 100 from border box origin
    // Child1 at x = 100, Child2 at x = 140
    assert!(approx_eq(block_box(node(&root, 0)).border_box.x, 100.0));
    assert!(approx_eq(block_box(node(&root, 1)).border_box.x, 140.0));
}

// ============================================
// Combined block + flex spacing tests
// ============================================

#[test]
fn nested_padding_margin() {
    let inner = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Length(Length::Px(10.0)),
            margin_top: LengthOrAuto::Length(Length::Px(5.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let outer = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(20.0),
                padding_top: Length::Px(15.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [inner],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(400.0)),
                height: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(30.0),
                padding_top: Length::Px(25.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [outer],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let root_box = block_box(&root);
    let outer_box = block_box(node(&root, 0));
    let inner_box = block_box(node(node(&root, 0), 0));

    // Root: border(0) + padding(30) = content_box.x=30
    assert_eq!(root_box.content_box.x, 30.0);
    assert_eq!(root_box.content_box.y, 25.0);

    // Outer: block child at (0,0) relative to root's border box
    // Then shifted by margin (none) and position (0,0)
    // Outer border_box.x and y should be relative to root's border box
    assert_eq!(outer_box.border_box.x, 0.0);
    assert_eq!(outer_box.border_box.y, 0.0);

    // Outer's content box: border(0)+padding(20) = content_box.x=20
    // But outer is at x=0, so content_box absolute x = 20
    assert_eq!(outer_box.content_box.x, 20.0);
    assert_eq!(outer_box.content_box.y, 15.0);

    // Inner: block child within outer, at (0,0) + margin offsets
    assert_eq!(inner_box.border_box.x, 10.0);
    assert_eq!(inner_box.border_box.y, 5.0);
}

// ============================================
// Percentage-based spacing tests
// ============================================

#[test]
fn percentage_padding_on_container() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(200.0)),
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        spacing: Spacing {
            padding_left: Length::Percent(10.0),
            padding_top: Length::Percent(5.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Percentage padding resolves against the containing block width (viewport = 800)
    // padding_left = 10% of 800 = 80
    // padding_top = 5% of 800 = 40
    assert_eq!(block_box(&root).content_box.x, 80.0);
    assert_eq!(block_box(&root).content_box.y, 40.0);
}

// ============================================
// Border-box sizing spacing tests
// ============================================

#[test]
fn border_box_sizing_with_padding_and_margin() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(30.0)),
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            box_sizing: BoxSizing::BorderBox,
            spacing: Spacing {
                padding_left: Length::Px(20.0),
                padding_right: Length::Px(20.0),
                border_left: Length::Px(5.0),
                border_right: Length::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // With BorderBox: width=200 is border box width
    // Content width = 200 - 20 - 20 - 5 - 5 = 150
    let b = block_box(&root);
    assert_eq!(b.border_box.width, 200.0);
    assert_eq!(b.content_box.width, 150.0);
    assert_eq!(b.content_box.x, 25.0);

    // Child should be positioned correctly
    let child_box = block_box(node(&root, 0));
    assert_eq!(child_box.border_box.x, 10.0);
}

// ============================================
// Edge cases
// ============================================

#[test]
fn all_margins_auto_on_block() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Auto,
            margin_right: LengthOrAuto::Auto,
            margin_top: LengthOrAuto::Auto,
            margin_bottom: LengthOrAuto::Auto,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let child_box = block_box(node(&root, 0));

    // Auto left+right margins center the block horizontally
    assert_eq!(child_box.border_box.x, 100.0);

    // Auto top+bottom collapse to 0 (no previous sibling to collapse with)
    assert_eq!(child_box.border_box.y, 0.0);
}

#[test]
fn zero_padding_margin() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Length(Length::Px(0.0)),
            margin_right: LengthOrAuto::Length(Length::Px(0.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(0.0),
                padding_right: Length::Px(0.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Zero padding/margin should result in child at (0,0)
    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    assert_eq!(block_box(&root).content_box.x, 0.0);
    assert_eq!(block_box(&root).content_box.y, 0.0);
}

#[test]
fn multiple_flex_children_with_various_margins() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(30.0)),
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_right: LengthOrAuto::Length(Length::Px(5.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(40.0)),
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Length(Length::Px(10.0)),
            margin_right: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Length(Length::Px(8.0)),
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
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // child1 at x=0, margin_right=5
    // child2: margin_left=10, so x = 30+5+10 = 45
    // child2: border width=40, margin_right=15
    // child3: margin_left=8, so x = 45+40+15+8 = 108
    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 45.0);
    assert_eq!(block_box(node(&root, 2)).border_box.x, 108.0);
}
