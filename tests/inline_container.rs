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
        display: Display::Inline,
        ..Default::default()
    });
    child1.set_fragments(vec![frag1]);

    let mut child2 = LayoutNode::new(Style {
        display: Display::Inline,
        ..Default::default()
    });
    child2.set_fragments(vec![frag2]);

    let inline_container = LayoutNode::with_children(
        Style {
            display: Display::Inline,
            ..Default::default()
        },
        vec![child1, child2],
    );

    let inner = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Px(100.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inline_container],
    );

    let mut root = LayoutNode::with_children(Style::default(), vec![inner]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.children[0].children[0].children[0].layout_boxes {
        LayoutBoxes::Multiple(lines) => {
            assert_eq!(lines.len(), 1);
        }
        _ => panic!("Expected multiple line boxes"),
    }

    match &root.children[0].children[0].children[1].layout_boxes {
        LayoutBoxes::Multiple(lines) => {
            // First one is the last part of the previous line
            // so width should be 0.
            assert_eq!(lines.len(), 2);
        }
        _ => panic!("Expected multiple line boxes"),
    }

    match &root.children[0].children[0].layout_boxes {
        LayoutBoxes::Multiple(lines) => {
            assert_eq!(lines.len(), 2);
        }
        _ => panic!("Expected multiple line boxes"),
    }
}

#[test]
fn inline_container_margin_horizontal_only() {
    let frag = ItemFragment::Fragment(Fragment {
        width: 50.0,
        height: 20.0,
    });

    let mut child = LayoutNode::new(Style {
        display: Display::Inline,
        ..Default::default()
    });
    child.set_fragments(vec![frag]);

    let inline_container = LayoutNode::with_children(
        Style {
            display: Display::Inline,
            spacing: Spacing {
                margin_left: Length::Px(10.0),
                margin_top: Length::Px(5.0), // ignored
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    let mut root = LayoutNode::with_children(Style::default(), vec![inline_container]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.children[0].layout_boxes {
        LayoutBoxes::Multiple(lines) => {
            let line = &lines[0];

            // horizontal margin applies
            assert_eq!(line.border_box.x, 10.0);

            // vertical margin ignored
            assert_eq!(line.border_box.y, 0.0);
        }
        _ => panic!("Expected multiple"),
    }
}

#[test]
fn inline_container_padding_border() {
    let frag = ItemFragment::Fragment(Fragment {
        width: 50.0,
        height: 20.0,
    });

    let mut child = LayoutNode::new(Style {
        display: Display::Inline,
        ..Default::default()
    });
    child.set_fragments(vec![frag]);

    let inline_container = LayoutNode::with_children(
        Style {
            display: Display::Inline,
            spacing: Spacing {
                padding_left: Length::Px(5.0),
                border_left: Length::Px(2.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    let mut root = LayoutNode::with_children(Style::default(), vec![inline_container]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.children[0].layout_boxes {
        LayoutBoxes::Multiple(lines) => {
            let line = &lines[0];

            // content width is child width
            assert_eq!(line.content_box.width, 50.0);

            // position reflects padding + border
            assert_eq!(line.border_box.x, 0.0);
            assert_eq!(line.content_box.x, 7.0); // 5 + 2
        }
        _ => panic!("Expected multiple"),
    }
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
        display: Display::Inline,
        ..Default::default()
    });
    child1.set_fragments(vec![frag1]);

    let mut child2 = LayoutNode::new(Style {
        display: Display::Inline,
        ..Default::default()
    });
    child2.set_fragments(vec![frag2]);

    let inline_container = LayoutNode::with_children(
        Style {
            display: Display::Inline,
            ..Default::default()
        },
        vec![child1, child2],
    );

    let inner = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: Length::Px(100.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inline_container],
    );

    let mut root = LayoutNode::with_children(Style::default(), vec![inner]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.children[0].children[0].layout_boxes {
        LayoutBoxes::Multiple(box_models) => {
            assert_eq!(box_models.len(), 2);
        }
        _ => panic!("Expected multiple"),
    }

    match &root.children[0].layout_boxes {
        LayoutBoxes::Single(box_model) => {
            // 2 lines × 20px
            assert_eq!(box_model.content_box.height, 40.0);
        }
        _ => panic!("Expected single"),
    }
}
