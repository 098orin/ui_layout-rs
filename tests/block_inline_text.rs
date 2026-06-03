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

/// Block container with a single inline child containing text fragments.
/// Verifies the inline box is created and has positive content dimensions.
#[test]
fn block_contains_inline_text() {
    let block = LayoutNode::with_children(
        Style {
            display: Display::parse("block").unwrap(),
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [LayoutNode::with_children(
            Style {
                display: Display::parse("inline").unwrap(),
                line_height: Length::Px(20.0),
                ..Default::default()
            },
            text_fragments("Hello World"),
        )],
    );

    let mut root = LayoutNode::with_children(Style::default(), [block]);
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // "Hello" (35px) + "World" (35px) = 70px on one line
    let inline_node = node(node(&root, 0), 0);
    match &inline_node.layout_box {
        LayoutBox::InlineBox(inline) => {
            assert_eq!(inline.line_spans.len(), 1);
            assert!((inline.line_spans[0].width() - 70.0).abs() < 0.1);
        }
        _ => panic!("expected inline box"),
    }
}

/// Inline text wraps across multiple lines when the container is narrow.
#[test]
fn inline_text_wraps_across_lines() {
    // Word widths: "Rust"=28, "is"=14, "a"=7, "systems"=49, "programming"=77, "language"=56
    // Container 100px wide:
    //   Line 0: Rust(28) + is(14) + a(7) + systems(49) = 98 (fits)
    //   Line 1: programming(77) (doesn't fit line0, starts line1)
    //   Line 1: programming(77) + language(56) = 133 > 100 → language wraps
    //   Line 2: language(56)
    let block = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [LayoutNode::with_children(
            Style {
                display: Display::parse("inline").unwrap(),
                line_height: Length::Px(20.0),
                ..Default::default()
            },
            text_fragments("Rust is a systems programming language"),
        )],
    );

    let mut root = LayoutNode::with_children(Style::default(), [block]);
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let block_node = node(&root, 0);

    match &block_node.layout_box {
        LayoutBox::BlockBox(b) => {
            // 3 lines × 20px line_height
            assert!((b.content_box.height - 60.0).abs() < 0.1);
        }
        _ => panic!("expected block box"),
    }

    let inline_node = node(block_node, 0);
    let boxes: Vec<BoxModel> = inline_node.layout_box.iter().collect();

    assert_eq!(boxes.len(), 3);
    assert!((boxes[0].content_box.width - 98.0).abs() < 0.1);
    assert!((boxes[1].content_box.width - 77.0).abs() < 0.1);
    assert!((boxes[2].content_box.width - 56.0).abs() < 0.1);
    for (i, b) in boxes.iter().enumerate() {
        assert_eq!(b.border_box.y, i as f32 * 20.0);
    }
}

/// Explicit line break via ItemFragment::LineBreak inside an inline element.
#[test]
fn inline_text_forced_line_break() {
    let block = LayoutNode::with_children(
        Style::default(),
        [LayoutNode::with_children(
            Style {
                display: Display::parse("inline").unwrap(),
                line_height: Length::Px(16.0),
                ..Default::default()
            },
            [
                fragment(30.0, 10.0),
                ItemFragment::LineBreak,
                fragment(40.0, 10.0),
            ],
        )],
    );

    let mut root = LayoutNode::with_children(Style::default(), [block]);
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let boxes: Vec<BoxModel> = node(node(&root, 0), 0).layout_box.iter().collect();
    assert_eq!(boxes.len(), 2);
    assert_eq!(boxes[0].content_box.width, 30.0);
    assert_eq!(boxes[0].border_box.y, 0.0);
    assert_eq!(boxes[1].content_box.width, 40.0);
    assert_eq!(boxes[1].border_box.x, 0.0);
    assert_eq!(boxes[1].border_box.y, 16.0);
}

/// Block > block > inline(text) nesting with padding at each level.
#[test]
fn nested_block_contains_inline_text() {
    let block = LayoutNode::with_children(
        Style {
            display: Display::parse("block").unwrap(),
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            spacing: Spacing {
                padding_left: Length::Px(16.0),
                padding_top: Length::Px(12.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [LayoutNode::with_children(
            Style {
                display: Display::parse("block").unwrap(),
                size: SizeStyle {
                    width: LengthOrAuto::Length(Length::Px(150.0)),
                    height: LengthOrAuto::Auto,
                    ..Default::default()
                },
                spacing: Spacing {
                    padding_left: Length::Px(8.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            [LayoutNode::with_children(
                Style {
                    display: Display::parse("inline").unwrap(),
                    line_height: Length::Px(20.0),
                    ..Default::default()
                },
                text_fragments("Hello"),
            )],
        )],
    );

    let mut root = LayoutNode::with_children(Style::default(), [block]);
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let outer = node(&root, 0);
    let inner = node(outer, 0);
    let inline_node = node(inner, 0);

    // Inline "Hello" = 35px on a single line
    match &inline_node.layout_box {
        LayoutBox::InlineBox(inline) => {
            assert_eq!(inline.line_spans.len(), 1);
            assert!((inline.line_spans[0].width() - 35.0).abs() < 0.1);
        }
        _ => panic!("expected inline box"),
    }

    // Outer block: content box inset by padding
    match &outer.layout_box {
        LayoutBox::BlockBox(b) => {
            assert_eq!(b.content_box.x, 16.0);
            assert_eq!(b.content_box.y, 12.0);
        }
        _ => panic!("expected block box"),
    }

    // Inner block: positioned at outer's content-box origin, content box inset by its own padding
    match &inner.layout_box {
        LayoutBox::BlockBox(b) => {
            assert_eq!(b.border_box.x, 0.0);
            assert_eq!(b.border_box.y, 0.0);
            assert_eq!(b.content_box.x, 8.0);
            assert_eq!(b.content_box.y, 0.0);
        }
        _ => panic!("expected block box"),
    }
}

/// Two inline siblings are placed side by side in the same inline formatting context.
#[test]
fn multiple_inline_siblings_in_block() {
    // "Hello" = 35px, "World" = 35px → 70px total, fits in 200px
    let block = LayoutNode::with_children(
        Style {
            display: Display::parse("block").unwrap(),
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [
            LayoutNode::with_children(
                Style {
                    display: Display::parse("inline").unwrap(),
                    line_height: Length::Px(20.0),
                    ..Default::default()
                },
                text_fragments("Hello"),
            ),
            LayoutNode::with_children(
                Style {
                    display: Display::parse("inline").unwrap(),
                    line_height: Length::Px(20.0),
                    ..Default::default()
                },
                text_fragments("World"),
            ),
        ],
    );

    let mut root = LayoutNode::with_children(Style::default(), [block]);
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // "Hello" starts at x=0, "World" starts right after "Hello"
    let hello_boxes: Vec<BoxModel> = node(node(&root, 0), 0).layout_box.iter().collect();
    let world_boxes: Vec<BoxModel> = node(node(&root, 0), 1).layout_box.iter().collect();

    assert!(!hello_boxes.is_empty());
    assert!(!world_boxes.is_empty());

    assert_eq!(hello_boxes[0].border_box.x, 0.0);
    assert_eq!(hello_boxes[0].border_box.y, 0.0);
    assert_eq!(hello_boxes[0].content_box.width, 35.0);

    assert_eq!(world_boxes[0].border_box.x, 35.0);
    assert_eq!(world_boxes[0].border_box.y, 0.0);
    assert_eq!(world_boxes[0].content_box.width, 35.0);
}
