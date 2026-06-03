use ui_layout::*;

fn fragment(width: f32, height: f32) -> ItemFragment {
    ItemFragment::Fragment(Fragment { width, height })
}

fn node<'a>(n: &'a LayoutNode, idx: usize) -> &'a LayoutNode {
    n.children[idx].node().expect("expected node child")
}

#[test]
fn layout_box_into_iter_block_single_line() {
    let fragment1 = fragment(30.0, 20.0);
    let fragment2 = fragment(40.0, 25.0);
    let fragment3 = fragment(35.0, 15.0);

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
        [fragment1, fragment2, fragment3],
    );

    let mut root = LayoutNode::with_children(Style::default(), [child]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let boxes: Vec<BoxModel> = root.children[0].node().unwrap().layout_box.iter().collect();
    assert_eq!(boxes.len(), 1);
    let b = &boxes[0];
    assert_eq!(b.content_box.width, 105.0);
    assert_eq!(b.border_box.width, 105.0 + 14.0);
}

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
    assert_eq!(child.layout_box.height(), 40.0);

    let boxes: Vec<BoxModel> = child.layout_box.iter().collect();
    assert_eq!(boxes.len(), 2);
    assert_eq!(boxes[0].content_box.width, 60.0);
    assert_eq!(boxes[0].content_box.y, 0.0);
    assert_eq!(boxes[1].content_box.width, 80.0);
    assert_eq!(boxes[1].content_box.y, 20.0);
}

#[test]
fn inline_line_break_starts_a_new_line() {
    let child = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(16.0),
            ..Default::default()
        },
        [
            fragment(25.0, 10.0),
            ItemFragment::LineBreak,
            fragment(35.0, 10.0),
        ],
    );

    let mut root = LayoutNode::with_children(Style::default(), [child]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let boxes: Vec<BoxModel> = node(&root, 0).layout_box.iter().collect();
    assert_eq!(boxes.len(), 2);
    assert_eq!(boxes[0].content_box.width, 25.0);
    assert_eq!(boxes[0].content_box.y, 0.0);
    assert_eq!(boxes[1].content_box.width, 35.0);
    assert_eq!(boxes[1].content_box.x, 0.0);
    assert_eq!(boxes[1].content_box.y, 16.0);
}

#[test]
fn nested_inline_child_spans_are_merged_into_parent_lines() {
    let nested = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(40.0, 10.0), fragment(50.0, 10.0)],
    );

    let parent = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [
            LayoutChild::from(fragment(30.0, 10.0)),
            LayoutChild::from(nested),
            LayoutChild::from(fragment(20.0, 10.0)),
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

    let parent_boxes: Vec<BoxModel> = node(&root, 0).layout_box.iter().collect();
    assert_eq!(parent_boxes.len(), 2);
    assert_eq!(parent_boxes[0].content_box.width, 70.0);
    assert_eq!(parent_boxes[0].content_box.x, 0.0);
    assert_eq!(parent_boxes[0].content_box.y, 0.0);
    assert_eq!(parent_boxes[1].content_box.width, 70.0);
    assert_eq!(parent_boxes[1].content_box.x, 0.0);
    assert_eq!(parent_boxes[1].content_box.y, 20.0);

    let nested_boxes: Vec<BoxModel> = node(node(&root, 0), 1).layout_box.iter().collect();
    assert_eq!(nested_boxes.len(), 2);
    assert_eq!(nested_boxes[0].border_box.x, 30.0);
    assert_eq!(nested_boxes[0].content_box.width, 40.0);
    assert_eq!(nested_boxes[1].content_box.x, 0.0);
    assert_eq!(nested_boxes[1].content_box.width, 50.0);
}

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
