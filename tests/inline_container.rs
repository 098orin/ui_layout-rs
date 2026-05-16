use ui_layout::*;

#[test]
fn inline_container_line_break() {
    let frag1 = ItemFragment::Fragment(Fragment {
        width: 80.0,
        height: 20.0,
    });
    let frag2 = ItemFragment::Fragment(Fragment {
        width: 90.0,
        height: 20.0,
    });

    let mut child1 = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Inline,
            inner: InnerDisplay::Flow,
        },
        ..Default::default()
    });
    child1.set_fragments(vec![frag1]);

    let mut child2 = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Inline,
            inner: InnerDisplay::Flow,
        },
        ..Default::default()
    });
    child2.set_fragments(vec![frag2]);

    let inline_container = LayoutNode::with_node_children(
        Style {
            display: Display {
                outer: OuterDisplay::Inline,
                inner: InnerDisplay::Flow,
            },
            ..Default::default()
        },
        vec![child1, child2],
    );

    let inner = LayoutNode::with_node_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inline_container],
    );

    let mut root = LayoutNode::with_node_children(Style::default(), vec![inner]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let lines: Vec<BoxModel> = (&root.children[0].children[0].children[0].layout_box)
        .into_iter()
        .collect();
    assert_eq!(lines.len(), 1);

    let lines2: Vec<BoxModel> = (&root.children[0].children[0].children[1].layout_box)
        .into_iter()
        .collect();
    assert_eq!(lines2.len(), 2);

    let lines3: Vec<BoxModel> = (&root.children[0].children[0].layout_box)
        .into_iter()
        .collect();
    assert_eq!(lines3.len(), 2);
}

#[test]
fn inline_container_margin_horizontal_only() {
    let frag = ItemFragment::Fragment(Fragment {
        width: 50.0,
        height: 20.0,
    });

    let mut child = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Inline,
            inner: InnerDisplay::Flow,
        },
        ..Default::default()
    });
    child.set_fragments(vec![frag]);

    let inline_container = LayoutNode::with_node_children(
        Style {
            display: Display {
                outer: OuterDisplay::Inline,
                inner: InnerDisplay::Flow,
            },
            spacing: Spacing {
                margin_left: LengthOrAuto::Length(Length::Px(10.0)),
                margin_top: LengthOrAuto::Length(Length::Px(5.0)), // ignored
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    let mut root = LayoutNode::with_node_children(Style::default(), vec![inline_container]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let lines: Vec<BoxModel> = (&root.children[0].layout_box).into_iter().collect();
    let line = &lines[0];

    // horizontal margin applies
    assert_eq!(line.border_box.x, 10.0);

    // vertical margin ignored
    assert_eq!(line.border_box.y, 0.0);
}

#[test]
fn inline_container_padding_border() {
    let frag = ItemFragment::Fragment(Fragment {
        width: 50.0,
        height: 20.0,
    });

    let mut child = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Inline,
            inner: InnerDisplay::Flow,
        },
        ..Default::default()
    });
    child.set_fragments(vec![frag]);

    let inline_container = LayoutNode::with_node_children(
        Style {
            display: Display {
                outer: OuterDisplay::Inline,
                inner: InnerDisplay::Flow,
            },
            spacing: Spacing {
                padding_left: Length::Px(5.0),
                border_left: Length::Px(2.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    let mut root = LayoutNode::with_node_children(Style::default(), vec![inline_container]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let lines: Vec<BoxModel> = (&root.children[0].layout_box).into_iter().collect();
    let line = &lines[0];

    // content width is child width
    assert_eq!(line.content_box.width, 50.0);

    // position reflects padding + border
    assert_eq!(line.border_box.x, 0.0);
    assert_eq!(line.content_box.x, 7.0); // 5 + 2
}

#[test]
fn inline_container_parent_height() {
    let frag1 = ItemFragment::Fragment(Fragment {
        width: 80.0,
        height: 20.0,
    });
    let frag2 = ItemFragment::Fragment(Fragment {
        width: 90.0,
        height: 20.0,
    });

    let mut child1 = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Inline,
            inner: InnerDisplay::Flow,
        },
        ..Default::default()
    });
    child1.set_fragments(vec![frag1]);

    let mut child2 = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Inline,
            inner: InnerDisplay::Flow,
        },
        ..Default::default()
    });
    child2.set_fragments(vec![frag2]);

    let inline_container = LayoutNode::with_node_children(
        Style {
            display: Display {
                outer: OuterDisplay::Inline,
                inner: InnerDisplay::Flow,
            },
            ..Default::default()
        },
        vec![child1, child2],
    );

    let inner = LayoutNode::with_node_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inline_container],
    );

    let mut root = LayoutNode::with_node_children(Style::default(), vec![inner]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let box_models: Vec<BoxModel> = (&root.children[0].children[0].layout_box)
        .into_iter()
        .collect();
    assert_eq!(box_models.len(), 2);

    match &root.children[0].layout_box {
        LayoutBox::BlockBox(box_model) => {
            // 2 lines × 20px
            assert_eq!(box_model.content_box.height, 40.0);
        }
        _ => panic!("Expected single"),
    }
}
