use ui_layout::*;

#[test]
fn inline_basic_flow() {
    let fragment1 = ItemFragment::Fragment(Fragment {
        width: 30.0,
        height: 20.0,
    });

    let fragment2 = ItemFragment::Fragment(Fragment {
        width: 40.0,
        height: 25.0,
    });

    let fragment3 = ItemFragment::Fragment(Fragment {
        width: 35.0,
        height: 15.0,
    });

    let mut inline_node = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Inline,
            inner: InnerDisplay::Flow,
        },
        ..Default::default()
    });
    inline_node.set_fragments(vec![fragment1, fragment2, fragment3]);

    let mut root = LayoutNode::with_node_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inline_node],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Check fragment placements
    assert_eq!(root.children[0].placements.len(), 3);

    // All fragments should be on the same line
    for placement in &root.children[0].placements {
        assert_eq!(placement.line_index, 0);
    }

    // Check horizontal positions
    assert_eq!(root.children[0].placements[0].offset.0, 0.0); // First fragment at start
    assert_eq!(root.children[0].placements[1].offset.0, 30.0); // Second after first
    assert_eq!(root.children[0].placements[2].offset.0, 70.0); // Third after second

    // Check that inline node has correct layout boxes
    let boxes: Vec<BoxModel> = (&root.children[0].layout_box).into_iter().collect();
    assert_eq!(boxes.len(), 1);

    let box_model = &boxes[0];
    assert_eq!(box_model.content_box.width, 105.0);
    assert_eq!(box_model.content_box.height, 25.0);
}

#[test]
fn inline_line_wrapping() {
    let fragment1 = ItemFragment::Fragment(Fragment {
        width: 80.0,
        height: 20.0,
    });

    let fragment2 = ItemFragment::Fragment(Fragment {
        width: 70.0,
        height: 25.0,
    });

    let fragment3 = ItemFragment::Fragment(Fragment {
        width: 60.0,
        height: 15.0,
    });

    let mut inline_node = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Inline,
            inner: InnerDisplay::Flow,
        },
        ..Default::default()
    });
    inline_node.set_fragments(vec![fragment1, fragment2, fragment3]);

    let mut root = LayoutNode::with_node_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(120.0)), // Force wrapping
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inline_node],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Check fragment placements
    assert_eq!(root.children[0].placements.len(), 3);

    // First fragment on line 0
    assert_eq!(root.children[0].placements[0].line_index, 0);
    assert_eq!(root.children[0].placements[0].offset.0, 0.0);

    // Second fragment should wrap to line 1
    assert_eq!(root.children[0].placements[1].line_index, 1);
    assert_eq!(root.children[0].placements[1].offset.0, 0.0);

    // Third fragment should wrap to line 2
    assert_eq!(root.children[0].placements[2].line_index, 2);
    assert_eq!(root.children[0].placements[2].offset.0, 0.0);

    // Check total height accounts for multiple lines
    let boxes: Vec<BoxModel> = (&root.children[0].layout_box).into_iter().collect();
    assert_eq!(boxes.len(), 3);

    assert_eq!(boxes[0].content_box.height, 20.0);
    assert_eq!(boxes[1].content_box.height, 25.0);
    assert_eq!(boxes[2].content_box.height, 15.0);

    assert_eq!(boxes[0].content_box.width, 80.0);
    assert_eq!(boxes[1].content_box.width, 70.0);
    assert_eq!(boxes[2].content_box.width, 60.0);
}

#[test]
fn inline_with_line_breaks() {
    let fragment1 = ItemFragment::Fragment(Fragment {
        width: 30.0,
        height: 20.0,
    });

    let line_break = ItemFragment::LineBreak;

    let fragment2 = ItemFragment::Fragment(Fragment {
        width: 40.0,
        height: 25.0,
    });

    let mut inline_node = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Inline,
            inner: InnerDisplay::Flow,
        },
        ..Default::default()
    });
    inline_node.set_fragments(vec![fragment1, line_break, fragment2]);

    let mut root = LayoutNode::with_node_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inline_node],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Check fragment placements (line break creates a placement too)
    assert_eq!(root.children[0].placements.len(), 3);

    // First fragment on line 0
    assert_eq!(root.children[0].placements[0].line_index, 0);
    assert_eq!(root.children[0].placements[0].offset.0, 0.0);

    // Line break on line 0 -> 1 transition
    assert_eq!(root.children[0].placements[1].line_index, 1);

    // Second fragment on line 1
    assert_eq!(root.children[0].placements[2].line_index, 1);
    assert_eq!(root.children[0].placements[2].offset.0, 0.0);

    // Check total height
    let box_models: Vec<BoxModel> = (&root.children[0].layout_box).into_iter().collect();
    assert_eq!(box_models.len(), 2);

    assert_eq!(box_models[0].content_box.height, 20.0);
    assert_eq!(box_models[1].content_box.height, 25.0);
}

#[test]
fn inline_with_margins() {
    let fragment1 = ItemFragment::Fragment(Fragment {
        width: 50.0,
        height: 20.0,
    });

    let mut inline_node = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Inline,
            inner: InnerDisplay::Flow,
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Length(Length::Px(10.0)),
            margin_right: LengthOrAuto::Length(Length::Px(15.0)),
            margin_top: LengthOrAuto::Length(Length::Px(5.0)), // ignored
            margin_bottom: LengthOrAuto::Length(Length::Px(8.0)), // ignored
            ..Default::default()
        },
        ..Default::default()
    });
    inline_node.set_fragments(vec![fragment1]);

    let inner = LayoutNode::with_node_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inline_node],
    );

    let mut root = LayoutNode::with_node_children(Style::default(), vec![inner]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Inline element layout
    let box_models: Vec<BoxModel> = (&root.children[0].children[0].layout_box)
        .into_iter()
        .collect();
    assert_eq!(box_models.len(), 1);

    let box_model = &box_models[0];

    // Horizontal margins affect x-position
    assert_eq!(box_model.border_box.x, 10.0);

    // Vertical margins do NOT affect inline positioning
    assert_eq!(box_model.border_box.y, 0.0);

    assert_eq!(box_model.content_box.width, 50.0);
    assert_eq!(box_model.content_box.height, 20.0);

    // Parent height calculation
    match &root.children[0].layout_box {
        LayoutBox::BlockBox(box_model) => {
            // Inline margins (top/bottom) do not contribute
            assert_eq!(box_model.content_box.height, 20.0);
        }
        _ => panic!("Expected single box model"),
    }
}

#[test]
fn inline_with_padding() {
    let fragment1 = ItemFragment::Fragment(Fragment {
        width: 40.0,
        height: 18.0,
    });

    let mut inline_node = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Inline,
            inner: InnerDisplay::Flow,
        },
        spacing: Spacing {
            padding_left: Length::Px(12.0),
            padding_right: Length::Px(8.0),
            padding_top: Length::Px(6.0),
            padding_bottom: Length::Px(4.0),
            ..Default::default()
        },
        ..Default::default()
    });
    inline_node.set_fragments(vec![fragment1]);

    let mut root = LayoutNode::with_node_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inline_node],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let boxes: Vec<BoxModel> = (&root.children[0].layout_box).into_iter().collect();
    assert_eq!(boxes.len(), 1);

    let box_model = &boxes[0];

    // Content box should be the fragment size
    assert_eq!(box_model.content_box.width, 40.0);
    assert_eq!(box_model.content_box.height, 18.0);

    // Padding box should include padding
    assert_eq!(box_model.padding_box.width, 60.0); // 40 + 12 + 8
    assert_eq!(box_model.padding_box.height, 28.0); // 18 + 6 + 4

    // Border box same as padding box (no border)
    assert_eq!(box_model.border_box.width, 60.0);
    assert_eq!(box_model.border_box.height, 28.0);

    // Fragment should be positioned within padding
    assert_eq!(root.children[0].placements.len(), 1);
    assert_eq!(root.children[0].placements[0].offset.0, 0.0); // Fragment offset relative to content box
}

#[test]
fn inline_with_borders() {
    let fragment1 = ItemFragment::Fragment(Fragment {
        width: 35.0,
        height: 22.0,
    });

    let mut inline_node = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Inline,
            inner: InnerDisplay::Flow,
        },
        spacing: Spacing {
            padding_left: Length::Px(8.0),
            padding_right: Length::Px(6.0),
            padding_top: Length::Px(4.0),
            padding_bottom: Length::Px(3.0),

            border_left: Length::Px(3.0),
            border_right: Length::Px(2.0),
            border_top: Length::Px(1.0),
            border_bottom: Length::Px(2.0),
            ..Default::default()
        },
        ..Default::default()
    });
    inline_node.set_fragments(vec![fragment1]);

    let mut root = LayoutNode::with_node_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inline_node],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let boxes: Vec<BoxModel> = (&root.children[0].layout_box).into_iter().collect();
    assert_eq!(boxes.len(), 1);

    let box_model = &boxes[0];

    // Content box should be fragment size
    assert_eq!(box_model.content_box.width, 35.0);
    assert_eq!(box_model.content_box.height, 22.0);

    // Padding box should include padding
    assert_eq!(box_model.padding_box.width, 49.0); // 35 + 8 + 6
    assert_eq!(box_model.padding_box.height, 29.0); // 22 + 4 + 3

    // Border box should include borders
    assert_eq!(box_model.border_box.width, 54.0); // 49 + 3 + 2
    assert_eq!(box_model.border_box.height, 32.0); // 29 + 1 + 2
}

#[test]
fn inline_percentage_spacing() {
    let fragment1 = ItemFragment::Fragment(Fragment {
        width: 30.0,
        height: 20.0,
    });

    let mut inline_node = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Inline,
            inner: InnerDisplay::Flow,
        },
        spacing: Spacing {
            padding_left: Length::Percent(5.0),   // 5% of 200px = 10px
            padding_right: Length::Percent(2.5),  // 2.5% of 200px = 5px
            padding_top: Length::Percent(7.5), // 7.5% of 200px = 15px (CSS: all padding % relative to width)
            padding_bottom: Length::Percent(1.0), // 1% of 200px = 2px

            margin_left: Length::Percent(2.0), // 2% of 200px = 4px
            margin_top: Length::Percent(0.0), // 0% - CSS spec: margin % relative to width, using 0 for simplicity
            ..Default::default()
        },
        ..Default::default()
    });
    inline_node.set_fragments(vec![fragment1]);

    let mut root = LayoutNode::with_node_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inline_node],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let boxes: Vec<BoxModel> = (&root.children[0].layout_box).into_iter().collect();
    assert_eq!(boxes.len(), 1);

    let box_model = &boxes[0];

    // Content box should be fragment size
    assert_eq!(box_model.content_box.width, 30.0);
    assert_eq!(box_model.content_box.height, 20.0);

    // Padding box with percentage padding
    assert_eq!(box_model.padding_box.width, 45.0); // 30 + 10 + 5
    assert_eq!(box_model.padding_box.height, 37.0); // 20 + 15 + 2

    // Position should account for percentage margins
    assert_eq!(box_model.border_box.x, 4.0); // 2% of 200px
    assert_eq!(box_model.border_box.y, 0.0); // 0% margin_top
}

#[test]
fn mixed_inline_and_block_children() {
    let fragment1 = ItemFragment::Fragment(Fragment {
        width: 40.0,
        height: 15.0,
    });

    let fragment2 = ItemFragment::Fragment(Fragment {
        width: 35.0,
        height: 20.0,
    });

    let mut inline_node1 = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Inline,
            inner: InnerDisplay::Flow,
        },
        ..Default::default()
    });
    inline_node1.set_fragments(vec![fragment1]);

    let block_node = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Flow,
        },
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(25.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut inline_node2 = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Inline,
            inner: InnerDisplay::Flow,
        },
        ..Default::default()
    });
    inline_node2.set_fragments(vec![fragment2]);

    let mut root = LayoutNode::with_node_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inline_node1, block_node, inline_node2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // First inline: first line
    let boxes: Vec<BoxModel> = (&root.children[0].layout_box).into_iter().collect();
    assert_eq!(boxes.len(), 1);

    let box_model = &boxes[0];

    assert_eq!(box_model.border_box.x, 0.0);
    assert_eq!(box_model.border_box.y, 0.0);

    // Block: new line after first inline line (height = 15)
    match &root.children[1].layout_box {
        LayoutBox::BlockBox(box_model) => {
            assert_eq!(box_model.border_box.x, 0.0);
            assert_eq!(box_model.border_box.y, 15.0);
        }
        _ => panic!("Expected single box model"),
    }

    // Second inline: new line after block (15 + 25 = 40)
    let boxes2: Vec<BoxModel> = (&root.children[2].layout_box).into_iter().collect();
    assert_eq!(boxes2.len(), 1);

    let box_model = &boxes2[0];

    assert_eq!(box_model.border_box.x, 0.0);
    assert_eq!(box_model.border_box.y, 40.0);
}

#[test]
fn inline_empty_fragments() {
    let inline_node = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Inline,
            inner: InnerDisplay::Flow,
        },
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(10.0),
            padding_top: Length::Px(5.0),
            padding_bottom: Length::Px(5.0),
            ..Default::default()
        },
        ..Default::default()
    });
    // No fragments set - empty inline element

    let mut root = LayoutNode::with_node_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inline_node],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Empty inline element should still have padding
    match &root.children[0].layout_box {
        LayoutBox::None => {}
        _ => panic!("Expected single box model"),
    }

    // Should have no fragment placements
    assert_eq!(root.children[0].placements.len(), 0);
}
