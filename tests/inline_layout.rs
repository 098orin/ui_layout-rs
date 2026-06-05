use ui_layout::*;

fn fragment(width: f32, height: f32) -> ItemFragment {
    ItemFragment::Fragment(Fragment { width, height })
}

fn node<'a>(n: &'a LayoutNode, idx: usize) -> &'a LayoutNode {
    n.children[idx].node().expect("expected node child")
}

fn block_box(n: &LayoutNode) -> &BoxModel {
    match &n.layout_box {
        LayoutBox::BlockBox(b) => b,
        _ => panic!("expected block box"),
    }
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}

fn mock_inline_box() -> LayoutBox {
    LayoutBox::InlineBox(InlineBox {
        box_model: BoxModel {
            border_box: rect(0.0, 0.0, 114.0, 20.0),
            padding_box: rect(2.0, 0.0, 110.0, 20.0),
            content_box: rect(6.0, 0.0, 100.0, 20.0),
            children_box: rect(6.0, 0.0, 100.0, 20.0),
        },
        line_spans: vec![
            LineSpan {
                x_range: 0.0..40.0,
                line_pos: (6.0, 0.0),
                line_index: 0,
            },
            LineSpan {
                x_range: 40.0..70.0,
                line_pos: (0.0, 20.0),
                line_index: 1,
            },
            LineSpan {
                x_range: 70.0..100.0,
                line_pos: (0.0, 40.0),
                line_index: 2,
            },
        ],
    })
}

// --- ItemFragment API ---

#[test]
fn item_fragment_api_and_fragmentnode() {
    let frag = ItemFragment::Fragment(Fragment {
        width: 50.0,
        height: 20.0,
    });
    assert_eq!(frag.width(), 50.0);
    assert_eq!(frag.height(), 20.0);
    assert!(!frag.is_line_break());

    let lb = ItemFragment::LineBreak;
    assert_eq!(lb.width(), 0.0);
    assert_eq!(lb.height(), 0.0);
    assert!(lb.is_line_break());

    let fragment_node = FragmentNode {
        node: frag,
        placement: Placement {
            offset: (0.0, 0.0),
            line_index: 0,
        },
    };
    assert_eq!(fragment_node.node.width(), 50.0);
}

// --- Inline text wrapping ---

#[test]
fn inline_text_wraps_across_lines() {
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
            [
                fragment(28.0, 12.0),
                fragment(14.0, 12.0),
                fragment(7.0, 12.0),
                fragment(49.0, 12.0),
                fragment(77.0, 12.0),
                fragment(56.0, 12.0),
            ],
        )],
    );

    let mut root = LayoutNode::with_children(Style::default(), [block]);
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).content_box.height, 60.0);

    let inline_node = node(node(&root, 0), 0);
    let boxes: Vec<BoxModel> = inline_node.layout_box.iter().collect();

    assert_eq!(boxes.len(), 3);
    assert_eq!(boxes[0].content_box.width, 98.0);
    assert_eq!(boxes[1].content_box.width, 77.0);
    assert_eq!(boxes[2].content_box.width, 56.0);
    for (i, b) in boxes.iter().enumerate() {
        assert_eq!(b.border_box.y, i as f32 * 20.0);
    }
}

#[test]
fn inline_forced_line_break() {
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

#[test]
fn leading_linebreak_creates_empty_line_span() {
    let child = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(16.0),
            ..Default::default()
        },
        [ItemFragment::LineBreak, fragment(40.0, 10.0)],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let boxes: Vec<BoxModel> = node(&root, 0).layout_box.iter().collect();
    assert_eq!(boxes.len(), 2);
    assert_eq!(boxes[0].content_box.width, 0.0);
    assert_eq!(boxes[0].border_box.y, 0.0);
    assert_eq!(boxes[1].content_box.width, 40.0);
    assert_eq!(boxes[1].border_box.x, 0.0);
    assert_eq!(boxes[1].border_box.y, 16.0);
}

// --- Nested block + inline ---

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
                [fragment(35.0, 12.0)],
            )],
        )],
    );

    let mut root = LayoutNode::with_children(Style::default(), [block]);
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let inline_node = node(node(node(&root, 0), 0), 0);
    match &inline_node.layout_box {
        LayoutBox::InlineBox(inline) => {
            assert_eq!(inline.line_spans.len(), 1);
            assert_eq!(inline.line_spans[0].width(), 35.0);
        }
        _ => panic!("expected inline box"),
    }

    let outer = block_box(node(&root, 0));
    assert_eq!(outer.content_box.x, 16.0);
    assert_eq!(outer.content_box.y, 12.0);
}

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

    assert_eq!(block_box(&root).content_box.height, 20.0);
}

// --- Multiple inline siblings ---

#[test]
fn multiple_inline_siblings_in_block() {
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
                [fragment(35.0, 12.0)],
            ),
            LayoutNode::with_children(
                Style {
                    display: Display::parse("inline").unwrap(),
                    line_height: Length::Px(20.0),
                    ..Default::default()
                },
                [fragment(35.0, 12.0)],
            ),
        ],
    );

    let mut root = LayoutNode::with_children(Style::default(), [block]);
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let hello_boxes: Vec<BoxModel> = node(node(&root, 0), 0).layout_box.iter().collect();
    let world_boxes: Vec<BoxModel> = node(node(&root, 0), 1).layout_box.iter().collect();

    assert!(!hello_boxes.is_empty());
    assert_eq!(hello_boxes[0].border_box.x, 0.0);
    assert_eq!(hello_boxes[0].border_box.y, 0.0);
    assert_eq!(hello_boxes[0].content_box.width, 35.0);
    assert_eq!(world_boxes[0].border_box.x, 35.0);
    assert_eq!(world_boxes[0].border_box.y, 0.0);
    assert_eq!(world_boxes[0].content_box.width, 35.0);
}

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

    let c1: Vec<BoxModel> = node(&root, 0).layout_box.iter().collect();
    let c2: Vec<BoxModel> = node(&root, 1).layout_box.iter().collect();

    assert_eq!(c1.len(), 1);
    assert_eq!(c1[0].border_box.y, 0.0);
    assert_eq!(c2.len(), 1);
    assert_eq!(c2[0].border_box.x, 0.0);
    assert_eq!(c2[0].border_box.y, 20.0);
    assert_eq!(block_box(&root).content_box.height, 40.0);
}

// --- Single-line inline ---

#[test]
fn inline_fragments_on_single_line() {
    let child = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            spacing: Spacing {
                padding_left: Length::Px(5.0),
                padding_right: Length::Px(5.0),
                border_left: Length::Px(2.0),
                border_right: Length::Px(2.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [
            fragment(30.0, 20.0),
            fragment(40.0, 25.0),
            fragment(35.0, 15.0),
        ],
    );

    let mut root = LayoutNode::with_children(Style::default(), [child]);
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let boxes: Vec<BoxModel> = root.children[0].node().unwrap().layout_box.iter().collect();
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0].content_box.width, 105.0);
    assert_eq!(boxes[0].border_box.width, 105.0 + 14.0);
}

// --- Multi-line inline wrapping ---

#[test]
fn inline_fragments_wrap_into_line_boxes_with_max_line_width() {
    let child = LayoutNode::with_children(
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
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let child = node(&root, 0);
    assert_eq!(child.layout_box.width_box(), 80.0);
    assert_eq!(child.layout_box.height_box(), 40.0);
    assert_eq!(child.layout_box.height_box(), 40.0);

    let boxes: Vec<BoxModel> = child.layout_box.iter().collect();
    assert_eq!(boxes.len(), 2);
    assert_eq!(boxes[0].content_box.width, 60.0);
    assert_eq!(boxes[0].border_box.y, 0.0);
    assert_eq!(boxes[1].content_box.width, 80.0);
    assert_eq!(boxes[1].border_box.y, 20.0);
}

// --- Inline padding / border ---

#[test]
fn inline_padding_and_border_are_applied_only_to_outer_edges_when_split() {
    let child = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            spacing: Spacing {
                padding_left: Length::Px(4.0),
                padding_right: Length::Px(6.0),
                border_left: Length::Px(2.0),
                border_right: Length::Px(3.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [fragment(70.0, 10.0), fragment(50.0, 10.0)],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let boxes: Vec<BoxModel> = node(&root, 0).layout_box.iter().collect();
    assert_eq!(boxes.len(), 2);
    assert_eq!(boxes[0].content_box.width, 70.0);
    assert_eq!(boxes[0].border_box.width, 76.0);
    assert_eq!(boxes[1].content_box.width, 50.0);
    assert_eq!(boxes[1].border_box.width, 59.0);
}

// --- Inline height in block parent ---

#[test]
fn inline_multi_line_height_reflected_in_block_parent() {
    let inline = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [
            fragment(28.0, 12.0),
            fragment(14.0, 12.0),
            fragment(7.0, 12.0),
            fragment(49.0, 12.0),
            fragment(77.0, 12.0),
            fragment(56.0, 12.0),
        ],
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

    assert_eq!(block_box(&root).content_box.height, 60.0);

    let inline_boxes: Vec<BoxModel> = node(&root, 0).layout_box.iter().collect();
    assert_eq!(inline_boxes.len(), 3);
    for (i, b) in inline_boxes.iter().enumerate() {
        assert_eq!(b.border_box.y, i as f32 * 20.0);
    }
}

#[test]
fn block_with_multiline_inline_then_block_child() {
    let inline = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(60.0, 10.0), fragment(50.0, 10.0), fragment(30.0, 10.0)],
    );

    let block_child = LayoutNode::new(Style {
        display: Display::parse("block").unwrap(),
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [inline, block_child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(root.layout_box.height_box(), 140.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 40.0);
}

// --- Inline with block child ---

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

    assert_eq!(block_box(&root).content_box.height, 40.0);

    let inline_boxes: Vec<BoxModel> = node(&root, 0).layout_box.iter().collect();
    assert_eq!(inline_boxes.len(), 1);
    assert_eq!(inline_boxes[0].border_box.height, 40.0);
}

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

    assert_eq!(block_box(&root).content_box.height, 30.0);
}

// --- Merge scenarios ---

#[test]
fn push_or_merge_merges_continuation_spans_from_child_inline() {
    let inline1 = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(30.0, 10.0)],
    );
    let inline2 = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(40.0, 10.0)],
    );

    let parent = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [LayoutChild::from(inline1), LayoutChild::from(inline2)],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        [parent],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let pboxes: Vec<BoxModel> = node(&root, 0).layout_box.iter().collect();
    assert_eq!(pboxes.len(), 1);
    assert_eq!(pboxes[0].content_box.width, 70.0);
    assert_eq!(pboxes[0].border_box.y, 0.0);
}

#[test]
fn push_or_merge_does_not_merge_spans_on_different_lines() {
    let inline1 = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(70.0, 10.0)],
    );
    let inline2 = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(50.0, 10.0)],
    );
    let inline3 = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(40.0, 10.0)],
    );

    let parent = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [inline1, inline2, inline3],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        [parent],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let pboxes: Vec<BoxModel> = node(&root, 0).layout_box.iter().collect();

    assert_eq!(pboxes.len(), 2);
    assert_eq!(pboxes[0].content_box.width, 70.0);
    assert_eq!(pboxes[0].border_box.y, 0.0);
    assert_eq!(pboxes[1].content_box.width, 90.0);
    assert_eq!(pboxes[1].border_box.y, 20.0);
}

// --- Line index ---

#[test]
fn line_index_is_local_within_each_inline_box() {
    let nested = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(30.0, 10.0), fragment(50.0, 10.0)],
    );

    let parent = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [
            LayoutChild::from(fragment(40.0, 10.0)),
            LayoutChild::from(nested),
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
        [parent],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let pboxes: Vec<BoxModel> = node(&root, 0).layout_box.iter().collect();
    assert_eq!(pboxes.len(), 2);
    let nboxes: Vec<BoxModel> = node(node(&root, 0), 1).layout_box.iter().collect();
    assert_eq!(nboxes.len(), 2);

    assert_eq!(pboxes[0].content_box.width, 70.0);
    assert_eq!(pboxes[0].border_box.y, 0.0);
    assert_eq!(pboxes[1].content_box.width, 50.0);
    assert_eq!(pboxes[1].border_box.y, 20.0);

    assert_eq!(nboxes[0].content_box.width, 30.0);
    assert_eq!(nboxes[0].border_box.y, 0.0);
    assert_eq!(nboxes[1].content_box.width, 50.0);
    assert_eq!(nboxes[1].border_box.y, 20.0);
}

// --- LayoutBox iterator ---

#[test]
fn borrowed_layout_box_iter_tracks_remaining_len() {
    let layout_box = mock_inline_box();
    let mut iter = layout_box.iter();

    assert_eq!(iter.len(), 3);
    assert_eq!(iter.size_hint(), (3, Some(3)));

    let first = iter.next().unwrap();
    assert_eq!(first.content_box.width, 40.0);
    assert_eq!(iter.len(), 2);

    let last = iter.next_back().unwrap();
    assert_eq!(last.content_box.width, 30.0);
    assert_eq!(iter.len(), 1);

    let middle = iter.next().unwrap();
    assert_eq!(middle.content_box.width, 30.0);
    assert_eq!(iter.len(), 0);
    assert!(iter.next().is_none());
    assert!(iter.next_back().is_none());
}

#[test]
fn owned_layout_box_into_iter_yields_line_boxes_lazily() {
    let mut iter = mock_inline_box().into_iter();

    assert_eq!(iter.len(), 3);

    let first = iter.next().unwrap();
    assert_eq!(first.border_box.y, 0.0);
    assert_eq!(first.content_box.width, 40.0);

    let second = iter.next().unwrap();
    assert_eq!(second.border_box.y, 20.0);
    assert_eq!(second.content_box.width, 30.0);

    let third = iter.next().unwrap();
    assert_eq!(third.border_box.y, 40.0);
    assert_eq!(third.content_box.width, 30.0);

    assert_eq!(iter.len(), 0);
    assert!(iter.next().is_none());
}

#[test]
fn none_and_block_iterators_report_len_and_end_correctly() {
    let mut none_iter = LayoutBox::None.iter();
    assert_eq!(none_iter.len(), 0);
    assert!(none_iter.next().is_none());

    let block = LayoutBox::BlockBox(BoxModel {
        border_box: rect(1.0, 2.0, 30.0, 40.0),
        padding_box: rect(1.0, 2.0, 30.0, 40.0),
        content_box: rect(1.0, 2.0, 30.0, 40.0),
        children_box: rect(1.0, 2.0, 30.0, 40.0),
    });

    let mut iter = block.iter();
    assert_eq!(iter.len(), 1);
    assert_eq!(
        iter.next_back().unwrap().content_box,
        rect(1.0, 2.0, 30.0, 40.0)
    );
    assert_eq!(iter.len(), 0);
    assert!(iter.next().is_none());
}
