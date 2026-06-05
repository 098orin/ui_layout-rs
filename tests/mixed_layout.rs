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

fn fragment(width: f32, height: f32) -> ItemFragment {
    ItemFragment::Fragment(Fragment { width, height })
}

#[test]
fn block_with_inline_then_block_siblings() {
    let inline_child = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(35.0, 15.0)],
    );

    let block_child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(40.0)),
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
        [inline_child, block_child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let inline_boxes: Vec<BoxModel> = node(&root, 0).layout_box.iter().collect();
    assert_eq!(inline_boxes.len(), 1);
    assert_eq!(inline_boxes[0].border_box.y, 0.0);

    assert_eq!(block_box(node(&root, 1)).border_box.y, 20.0);
    assert_eq!(block_box(&root).content_box.height, 60.0);
}

#[test]
fn block_with_block_then_inline_siblings() {
    let block_child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(40.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let inline_child = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(35.0, 15.0)],
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
        [block_child, inline_child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);

    let inline_boxes: Vec<BoxModel> = node(&root, 1).layout_box.iter().collect();
    assert_eq!(inline_boxes.len(), 1);
    assert_eq!(inline_boxes[0].border_box.y, 40.0);
    assert_eq!(block_box(&root).content_box.height, 60.0);
}

#[test]
fn block_with_interleaved_inline_and_block() {
    let inline1 = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(30.0, 10.0)],
    );

    let block1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let inline2 = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(40.0, 10.0)],
    );

    let block2 = LayoutNode::new(Style {
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
        [inline1, block1, inline2, block2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 1)).border_box.y, 20.0);

    let inline2_boxes: Vec<BoxModel> = node(&root, 2).layout_box.iter().collect();
    assert_eq!(inline2_boxes[0].border_box.y, 50.0);

    assert_eq!(block_box(node(&root, 3)).border_box.y, 70.0);
    assert_eq!(block_box(&root).content_box.height, 90.0);
}

#[test]
fn inline_fragments_between_block_children() {
    let block1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let middle_fragments = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(16.0),
            ..Default::default()
        },
        [fragment(50.0, 10.0), fragment(30.0, 10.0)],
    );

    let block2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(30.0)),
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
        [block1, middle_fragments, block2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);

    let middle_boxes: Vec<BoxModel> = node(&root, 1).layout_box.iter().collect();
    assert_eq!(middle_boxes.len(), 1);
    assert_eq!(middle_boxes[0].border_box.y, 20.0);

    assert_eq!(block_box(node(&root, 2)).border_box.y, 36.0);
    assert_eq!(block_box(&root).content_box.height, 66.0);
}

#[test]
fn inline_fragments_then_block_then_more_fragments() {
    let first_fragments = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(60.0, 10.0), fragment(50.0, 10.0)],
    );

    let block_child = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let more_fragments = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(40.0, 10.0)],
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
        [first_fragments, block_child, more_fragments],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let first_boxes: Vec<BoxModel> = node(&root, 0).layout_box.iter().collect();
    assert_eq!(first_boxes.len(), 2);
    assert_eq!(first_boxes[0].border_box.y, 0.0);
    assert_eq!(first_boxes[1].border_box.y, 20.0);

    assert_eq!(block_box(node(&root, 1)).border_box.y, 40.0);

    let more_boxes: Vec<BoxModel> = node(&root, 2).layout_box.iter().collect();
    assert_eq!(more_boxes.len(), 1);
    assert_eq!(more_boxes[0].border_box.y, 70.0);

    assert_eq!(block_box(&root).content_box.height, 90.0);
}
