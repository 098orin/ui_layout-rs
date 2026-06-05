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
fn block_then_inline() {
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
fn inline_then_block() {
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
fn inline_with_fragment_and_block() {
    let block_child1 = LayoutNode::new(Style {
        display: Display::parse("block").unwrap(),
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(80.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let block_child2 = LayoutNode::new(Style {
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
            LayoutChild::from(block_child1),
            fragment(40.0, 15.0).into(),
            fragment(30.0, 15.0).into(),
            LayoutChild::from(block_child2),
            fragment(30.0, 15.0).into(),
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

    assert_eq!(block_box(&root).content_box.height, 100.0);

    let inline_child = node(&root, 0);
    let inline_box = inline_child.layout_box.iter().collect::<Vec<_>>();
    assert_eq!(inline_box.len(), 2);

    assert_eq!(inline_child.layout_box.width_box(), 70.0);
}

#[test]
fn consecutive_inline() {
    let inline1 = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(35.0, 12.0)],
    );

    let inline2 = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(40.0, 12.0)],
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

    let boxes1: Vec<BoxModel> = node(&root, 0).layout_box.iter().collect();
    let boxes2: Vec<BoxModel> = node(&root, 1).layout_box.iter().collect();

    assert_eq!(boxes1[0].border_box.x, 0.0);
    assert_eq!(boxes1[0].border_box.y, 0.0);
    assert_eq!(boxes1[0].content_box.width, 35.0);

    assert_eq!(boxes2[0].border_box.x, 35.0);
    assert_eq!(boxes2[0].border_box.y, 0.0);
    assert_eq!(boxes2[0].content_box.width, 40.0);

    assert_eq!(block_box(&root).content_box.height, 20.0);
}
