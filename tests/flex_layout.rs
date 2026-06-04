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

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.1
}

// --- Flex grow ---

#[test]
fn flex_row_grow() {
    let child1 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 1.0,
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

    assert_eq!(block_box(node(&root, 0)).content_box.width, 150.0);
    assert_eq!(block_box(node(&root, 1)).content_box.width, 150.0);
}

// --- Gap ---

#[test]
fn flex_gap_affects_children_box() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(15.0)),
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(10.0)),
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

    assert_eq!(block_box(&root).children_box.width, 15.0 + 10.0 + 20.0);
    assert_eq!(
        block_box(node(&root, 1)).border_box.x,
        block_box(node(&root, 0)).border_box.width + 20.0
    );
}

#[test]
fn flex_row_gap_column_gap() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            height: LengthOrAuto::Length(Length::Px(40.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(70.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(35.0)),
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
                height: LengthOrAuto::Length(Length::Px(80.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            column_gap: LengthOrAuto::Length(Length::Px(15.0)),
            row_gap: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        [child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 75.0);
    assert_eq!(block_box(node(&root, 2)).border_box.x, 160.0);
}

// --- Align items ---

#[test]
fn flex_align_items_stretch() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Auto,
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
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Length(Length::Px(80.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Stretch,
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).content_box.height, 80.0);
}

#[test]
fn flex_align_items_different_values() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        item_style: ItemStyle {
            align_self: Some(AlignItems::Start),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            height: LengthOrAuto::Length(Length::Px(40.0)),
            ..Default::default()
        },
        item_style: ItemStyle {
            align_self: Some(AlignItems::Center),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(40.0)),
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        item_style: ItemStyle {
            align_self: Some(AlignItems::End),
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
                height: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            align_items: AlignItems::Stretch,
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 30.0);
    assert_eq!(block_box(node(&root, 2)).border_box.y, 80.0);
}

// --- Justify content ---

#[test]
fn flex_justify_content_space_evenly() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(40.0)),
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
            justify_content: JustifyContent::SpaceEvenly,
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert!(approx_eq(block_box(node(&root, 0)).border_box.x, 37.5));
    assert!(approx_eq(block_box(node(&root, 1)).border_box.x, 125.0));
    assert!(approx_eq(block_box(node(&root, 2)).border_box.x, 222.5));
}

// --- Column direction ---

#[test]
fn flex_column_layout() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(40.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
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
                height: LengthOrAuto::Length(Length::Px(150.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        [child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).content_box.height, 40.0);
    assert_eq!(block_box(node(&root, 0)).content_box.width, 200.0);
    assert_eq!(block_box(node(&root, 1)).content_box.height, 80.0);
    assert_eq!(block_box(node(&root, 1)).content_box.width, 200.0);
    assert_eq!(block_box(node(&root, 2)).content_box.height, 30.0);
    assert_eq!(block_box(node(&root, 2)).content_box.width, 200.0);
}

// --- Nested flex ---

#[test]
fn nested_flex_containers() {
    let inner_child1 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let inner_child2 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 2.0,
            ..Default::default()
        },
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let inner_flex = LayoutNode::with_children(
        Style {
            display: Display {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(120.0)),
                height: LengthOrAuto::Length(Length::Px(30.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [inner_child1, inner_child2],
    );

    let regular_child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(80.0)),
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
                width: LengthOrAuto::Length(Length::Px(250.0)),
                height: LengthOrAuto::Length(Length::Px(50.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            column_gap: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        [inner_flex, regular_child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 0)).content_box.width, 120.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 130.0);
    assert_eq!(block_box(node(&root, 1)).content_box.width, 80.0);
    assert_eq!(block_box(node(node(&root, 0), 0)).content_box.width, 40.0);
    assert_eq!(block_box(node(node(&root, 0), 1)).content_box.width, 80.0);
}

// --- Percentage basis ---

#[test]
fn flex_with_percentage_basis() {
    let child1 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: LengthOrAuto::Length(Length::Percent(30.0)),
            flex_grow: 0.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: LengthOrAuto::Length(Length::Percent(20.0)),
            flex_grow: 1.0,
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

    assert_eq!(block_box(node(&root, 0)).content_box.width, 90.0);
    assert_eq!(block_box(node(&root, 1)).content_box.width, 210.0);
}

// --- Auto margins ---

#[test]
fn flex_auto_margins_override_justify_content() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(60.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Auto,
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(40.0)),
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
        [child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
}

// --- Fragments in flex ---

#[test]
fn flex_row_places_consecutive_fragments_as_one_item() {
    let trailing_node = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(40.0)),
            height: LengthOrAuto::Length(Length::Px(20.0)),
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
                height: LengthOrAuto::Length(Length::Px(40.0)),
                ..Default::default()
            },
            line_height: Length::Px(20.0),
            flex_direction: FlexDirection::Row,
            column_gap: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        [
            LayoutChild::from(fragment(20.0, 10.0)),
            LayoutChild::from(fragment(30.0, 10.0)),
            LayoutChild::from(trailing_node),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(
        root.children[0].fragment().unwrap().placement.offset,
        (0.0, 0.0)
    );
    assert_eq!(
        root.children[1].fragment().unwrap().placement.offset,
        (20.0, 0.0)
    );
    assert_eq!(block_box(node(&root, 2)).border_box.x, 60.0);
}

// --- Min / max constraints ---

#[test]
fn flex_min_max_constraints_with_grow() {
    let child1 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        size: SizeStyle {
            min_width: LengthOrAuto::Length(Length::Px(80.0)),
            max_width: LengthOrAuto::Length(Length::Px(120.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 2.0,
            ..Default::default()
        },
        size: SizeStyle {
            min_width: LengthOrAuto::Length(Length::Px(60.0)),
            max_width: LengthOrAuto::Length(Length::Px(100.0)),
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
                width: LengthOrAuto::Length(Length::Px(400.0)),
                height: LengthOrAuto::Length(Length::Px(50.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let c1 = block_box(node(&root, 0));
    assert!(c1.content_box.width <= 120.0);
    assert!(c1.content_box.width >= 80.0);

    let c2 = block_box(node(&root, 1));
    assert!(c2.content_box.width <= 100.0);
    assert!(c2.content_box.width >= 60.0);
}

// --- Basis: auto ---

#[test]
fn test_flex_basis_auto_simple() {
    let mut container = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Flex,
        },
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(300.0)),
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        flex_direction: FlexDirection::Row,
        ..Default::default()
    });

    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });
    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(80.0)),
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    container.children = vec![
        LayoutChild::Node(Box::new(child1)),
        LayoutChild::Node(Box::new(child2)),
    ];

    LayoutEngine::layout(&mut container, 800.0, 600.0);

    assert_eq!(block_box(node(&container, 0)).border_box.width, 50.0);
    assert_eq!(block_box(node(&container, 1)).border_box.width, 80.0);
}

#[test]
fn test_flex_basis_grow_simple() {
    let mut container = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Flex,
        },
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(300.0)),
            ..Default::default()
        },
        flex_direction: FlexDirection::Row,
        ..Default::default()
    });

    let child = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: LengthOrAuto::Length(Length::Px(100.0)),
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    container.children = vec![LayoutChild::Node(Box::new(child))];
    LayoutEngine::layout(&mut container, 800.0, 600.0);

    assert_eq!(block_box(node(&container, 0)).border_box.width, 300.0);
}

// --- Basis: grow/shrink with explicit width/height ---

#[test]
fn flex_grow_with_explicit_width() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_grow: 1.0,
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

    assert!(approx_eq(
        block_box(node(&root, 0)).content_box.width,
        175.0
    ));
    assert!(approx_eq(
        block_box(node(&root, 1)).content_box.width,
        125.0
    ));
}

#[test]
fn flex_shrink_with_explicit_width() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(200.0)),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_shrink: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(150.0)),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_shrink: 1.0,
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
                width: LengthOrAuto::Length(Length::Px(210.0)),
                height: LengthOrAuto::Length(Length::Px(50.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert!(approx_eq(
        block_box(node(&root, 0)).content_box.width,
        120.0
    ));
    assert!(approx_eq(block_box(node(&root, 1)).content_box.width, 90.0));
}

#[test]
fn flex_column_grow_with_explicit_height() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_grow: 2.0,
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
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert!(approx_eq(
        block_box(node(&root, 0)).content_box.height,
        80.0
    ));
    assert!(approx_eq(
        block_box(node(&root, 1)).content_box.height,
        120.0
    ));
}
