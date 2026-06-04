use ui_layout::*;

fn fragment(width: f32, height: f32) -> ItemFragment {
    ItemFragment::Fragment(Fragment { width, height })
}

fn text_fragments(text: &str) -> Vec<ItemFragment> {
    let mut frags = Vec::new();
    for word in text.split_whitespace() {
        frags.push(fragment((7 * word.len()) as f32, 12.0));
    }
    frags
}

fn node<'a>(n: &'a LayoutNode, idx: usize) -> &'a LayoutNode {
    n.children[idx].node().expect("expected node child")
}

// ============================================================
// Scenario 1: Inline child line breaks are properly reflected
// ============================================================

/// Inline with fragments that wrap into multiple lines:
/// parent block should reflect the total height.
#[test]
fn inline_multi_line_height_reflected_in_block_parent() {
    let inline = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        text_fragments("Rust is a systems programming language"),
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [inline],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // 3 lines × 20px = 60px
    let root_box = match &root.layout_box {
        LayoutBox::BlockBox(b) => b,
        _ => panic!("expected block box"),
    };
    assert!(
        (root_box.content_box.height - 60.0).abs() < 0.1,
        "parent block height should be 60, got {}",
        root_box.content_box.height
    );

    let inline_boxes: Vec<BoxModel> = node(&root, 0).layout_box.iter().collect();
    assert_eq!(inline_boxes.len(), 3);
    for (i, b) in inline_boxes.iter().enumerate() {
        assert_eq!(b.border_box.y, i as f32 * 20.0, "line {} y position", i);
    }
}

/// Two separate inline siblings in a block parent:
/// second inline should start where first ends.
#[test]
fn inline_siblings_flow_horizontally() {
    let inline1 = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(40.0, 15.0)],
    );

    let inline2 = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(50.0, 15.0)],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [inline1, inline2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let c1 = node(&root, 0);
    let c2 = node(&root, 1);

    let c1_boxes: Vec<BoxModel> = c1.layout_box.iter().collect();
    let c2_boxes: Vec<BoxModel> = c2.layout_box.iter().collect();

    assert_eq!(c1_boxes.len(), 1);
    assert_eq!(c2_boxes.len(), 1);
    assert_eq!(c1_boxes[0].border_box.x, 0.0);
    assert_eq!(c2_boxes[0].border_box.x, 40.0);
}

/// Inline sibling wraps to next line when line is full:
/// second inline should appear on a new line.
#[test]
fn inline_sibling_wraps_to_next_line() {
    let inline1 = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(70.0, 15.0)],
    );

    let inline2 = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(50.0, 15.0)],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [inline1, inline2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let c1 = node(&root, 0);
    let c2 = node(&root, 1);

    let c1_boxes: Vec<BoxModel> = c1.layout_box.iter().collect();
    let c2_boxes: Vec<BoxModel> = c2.layout_box.iter().collect();

    assert_eq!(c1_boxes.len(), 1);
    assert_eq!(c1_boxes[0].border_box.y, 0.0);

    // inline1 (70px) fills most of 100px line, inline2 (50px) wraps
    assert_eq!(c2_boxes.len(), 1);
    assert_eq!(c2_boxes[0].border_box.x, 0.0);
    assert_eq!(c2_boxes[0].border_box.y, 20.0);

    // parent height should include both lines
    let root_box = match &root.layout_box {
        LayoutBox::BlockBox(b) => b,
        _ => panic!("expected block box"),
    };
    assert!(
        (root_box.content_box.height - 40.0).abs() < 0.1,
        "parent height should be 40 (2 lines), got {}",
        root_box.content_box.height
    );
}

// ============================================================
// Scenario 2: Block children heights reflected in parent height
// ============================================================

/// Multiple block children: parent auto-height should sum all children.
#[test]
fn multiple_block_children_auto_height() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
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

    let root_box = match &root.layout_box {
        LayoutBox::BlockBox(b) => b,
        _ => panic!("expected block box"),
    };
    // 30 + 50 + 20 = 100
    assert!(
        (root_box.content_box.height - 100.0).abs() < 0.1,
        "parent height should be 100, got {}",
        root_box.content_box.height
    );

    // Check each child's position
    let c1 = node(&root, 0);
    let c2 = node(&root, 1);
    let c3 = node(&root, 2);

    let c1_box = match &c1.layout_box {
        LayoutBox::BlockBox(b) => b,
        _ => panic!("expected block box"),
    };
    let c2_box = match &c2.layout_box {
        LayoutBox::BlockBox(b) => b,
        _ => panic!("expected block box"),
    };
    let c3_box = match &c3.layout_box {
        LayoutBox::BlockBox(b) => b,
        _ => panic!("expected block box"),
    };

    assert_eq!(c1_box.border_box.y, 0.0);
    assert_eq!(c2_box.border_box.y, 30.0);
    assert_eq!(c3_box.border_box.y, 80.0);
}

/// Block children with padding in parent: parent height should include them.
#[test]
fn block_children_with_padding_in_parent_auto_height() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(5.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(5.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(10.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(10.0)),
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
            spacing: Spacing {
                padding_top: Length::Px(8.0),
                padding_bottom: Length::Px(8.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let root_box = match &root.layout_box {
        LayoutBox::BlockBox(b) => b,
        _ => panic!("expected block box"),
    };

    // children_height = child1(20) + collapse(max(5,10)=10) + child2(30) + child2_margin_bottom(10)
    //                 = 20 + 10 + 30 + 10 = 70
    // But also child1's margin_bottom(5) adds: 70 + 5 = 75
    // Actually: child1_bottom(5+20=25) + child1_margin_bottom(5) = 30
    //   child2_bottom(25+10+30=65) + child2_margin_bottom(10) = 75
    //   max(30, 75) = 75
    // content_box.height = children_height = 75
    // border_box.height = 75 + padding_top(8) + padding_bottom(8) = 91
    assert!(
        (root_box.content_box.height - 75.0).abs() < 0.1,
        "parent content height should be 75, got {}",
        root_box.content_box.height
    );
    assert!(
        (root_box.border_box.height - 91.0).abs() < 0.1,
        "parent border height should be 91, got {}",
        root_box.border_box.height
    );

    // Children should be positioned relative to content box origin,
    // which includes padding offset.
    assert_eq!(root_box.content_box.y, 8.0); // border_top(0) + padding_top(8)
}

// ============================================================
// Scenario 3: Inline containing block children
// ============================================================

/// Inline node containing a block child:
/// the inline should still contribute its block child's height to the parent.
#[test]
fn inline_with_block_child_height_reflected() {
    let inner_block = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(40.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let inline_wrapper = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [inner_block],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [inline_wrapper],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let root_box = match &root.layout_box {
        LayoutBox::BlockBox(b) => b,
        _ => panic!("expected block box"),
    };

    // The inline wrapper's block child (40px) should contribute to parent height
    assert!(
        (root_box.content_box.height - 40.0).abs() < 0.1,
        "parent height should include block child inside inline (expected 40, got {})",
        root_box.content_box.height
    );

    // The inline wrapper should have the block child's dimensions
    let inline_node = node(&root, 0);
    let inline_boxes: Vec<BoxModel> = inline_node.layout_box.iter().collect();

    // The inline should yield the base box model reflecting the block child
    assert_eq!(inline_boxes.len(), 1);
    assert!(
        (inline_boxes[0].border_box.height - 40.0).abs() < 0.1,
        "inline containing block should have height 40, got {}",
        inline_boxes[0].border_box.height
    );
}

/// Inline with fragments AND a block child:
/// fragments on first line, block below.
#[test]
fn inline_with_fragment_then_block_child() {
    let inner_block = LayoutNode::new(Style {
        display: Display::parse("block").unwrap(),
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(80.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let inline_wrapper = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [
            LayoutChild::from(fragment(40.0, 15.0)),
            LayoutChild::from(inner_block),
        ],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [inline_wrapper],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let root_box = match &root.layout_box {
        LayoutBox::BlockBox(b) => b,
        _ => panic!("expected block box"),
    };

    // Fragment line: line_height=20, block child: 30px. Both start at y=0.
    // Total extent = max(20, 30) = 30.
    assert!(
        (root_box.content_box.height - 30.0).abs() < 0.1,
        "parent height should be 30 (fragment line 20px, block child 30px), got {}",
        root_box.content_box.height
    );
}

/// Nested inline > block > inline:
/// the inner inline should be positioned correctly within the block.
#[test]
fn inline_contains_block_contains_inline() {
    let inner_inline = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(50.0, 15.0)],
    );

    let middle_block = LayoutNode::with_children(
        Style {
            display: Display::parse("block").unwrap(),
            size: SizeStyle {
                width: LengthOrAuto::Auto,
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [inner_inline],
    );

    let outer_inline = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [middle_block],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [outer_inline],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let root_box = match &root.layout_box {
        LayoutBox::BlockBox(b) => b,
        _ => panic!("expected block box"),
    };

    // outer_inline > middle_block(40) > inner_inline(fragment 50px, line_height=20)
    // middle_block has auto height, gets 20px from inner_inline's line
    // outer_inline's InlineBox reflects middle_block's 20px
    // Root sees outer_inline as 20px
    assert!(
        (root_box.content_box.height - 20.0).abs() < 0.1,
        "root height should be 20 (inner_inline line_height), got {}",
        root_box.content_box.height
    );
}

// ============================================================
// Edge cases: margin collapse between block siblings in flow
// ============================================================

/// Three block children with margins:
/// - child1 margin-bottom = 20
/// - child2 margin-top = 15, margin-bottom = 10
/// - child3 margin-top = 5
/// Collapse: max(20,15)=20 between 1-2, max(10,5)=10 between 2-3
#[test]
fn three_block_siblings_margin_collapse_auto_height() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: LengthOrAuto::Length(Length::Px(20.0)),
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
            margin_top: LengthOrAuto::Length(Length::Px(15.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(5.0)),
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

    let root_box = match &root.layout_box {
        LayoutBox::BlockBox(b) => b,
        _ => panic!("expected block box"),
    };

    // child1(20) + max(20,15)=20 + child2(20) + max(10,5)=10 + child3(20) = 90
    assert!(
        (root_box.content_box.height - 90.0).abs() < 0.1,
        "parent height should be 90 after margin collapse, got {}",
        root_box.content_box.height
    );

    let c1 = node(&root, 0);
    let c2 = node(&root, 1);
    let c3 = node(&root, 2);

    let c1_box = match &c1.layout_box {
        LayoutBox::BlockBox(b) => b,
        _ => panic!("expected block box"),
    };
    let c2_box = match &c2.layout_box {
        LayoutBox::BlockBox(b) => b,
        _ => panic!("expected block box"),
    };
    let c3_box = match &c3.layout_box {
        LayoutBox::BlockBox(b) => b,
        _ => panic!("expected block box"),
    };

    assert_eq!(c1_box.border_box.y, 0.0);
    assert_eq!(c2_box.border_box.y, 40.0); // 20 + max(20,15) = 20+20 = 40
    assert_eq!(c3_box.border_box.y, 70.0); // 40+20 + max(10,5) = 60+10 = 70
}

// ============================================================
// Edge case: inline box height_box() and width_box() correctness
// ============================================================

/// Inline with multiple lines: height_box() should return total height.
#[test]
fn inline_height_box_multi_line() {
    let inline = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [
            fragment(60.0, 10.0),
            fragment(50.0, 10.0),
            fragment(30.0, 10.0),
        ],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        [inline],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let inline_node = node(&root, 0);
    // 2 lines × 20px = 40
    assert_eq!(inline_node.layout_box.height_box(), 40.0);
    // max span width = 80 (second line: 50 + 30)
    assert_eq!(inline_node.layout_box.width_box(), 80.0);
}

// ============================================================
// Edge case: block containing only inline children
// track cursor correctly after inline children
// ============================================================

/// Block parent with two inline children that fit on one line.
/// The cursor after the second inline should be at combined width.
#[test]
fn block_with_inline_children_cursor_tracks_correctly() {
    let inline1 = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(25.0, 10.0)],
    );

    let inline2 = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(35.0, 10.0)],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [inline1, inline2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let root_box = match &root.layout_box {
        LayoutBox::BlockBox(b) => b,
        _ => panic!("expected block box"),
    };

    // One line of content, line_height=20
    assert!(
        (root_box.content_box.height - 20.0).abs() < 0.1,
        "parent height should be 20 for single line, got {}",
        root_box.content_box.height
    );
}
