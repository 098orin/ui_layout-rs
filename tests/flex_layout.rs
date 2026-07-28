mod common;
use common::*;
use ui_layout::*;

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
        flex_container(300.0, 50.0, FlexDirection::Row),
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).content_box.width, 150.0);
    assert_eq!(block_box(node(&root, 1)).content_box.width, 150.0);
}

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
        flex_container(300.0, 50.0, FlexDirection::Row),
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
        flex_container(210.0, 50.0, FlexDirection::Row),
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert!(approx_eq(
        block_box(node(&root, 0)).content_box.width,
        120.0
    ));
    assert!(approx_eq(block_box(node(&root, 1)).content_box.width, 90.0));
}

// --- Flex basis ---

#[test]
fn flex_basis_grow_simple() {
    let child = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: LengthOrAuto::Length(Length::Px(100.0)),
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root =
        LayoutNode::with_children(flex_container(300.0, 50.0, FlexDirection::Row), [child]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.width, 300.0);
}

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
            flex_grow: 0.0,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        flex_container(300.0, 50.0, FlexDirection::Row),
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).content_box.width, 90.0);
    assert_eq!(block_box(node(&root, 1)).content_box.width, 60.0);
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
        flex_container(200.0, 150.0, FlexDirection::Column),
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
        flex_container(100.0, 200.0, FlexDirection::Column),
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
fn flex_align_self_start_and_end() {
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
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 80.0);
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
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
}

// --- Nested flex ---

#[test]
fn nested_flex_containers() {
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
        [
            LayoutNode::new(Style {
                size: SizeStyle {
                    width: LengthOrAuto::Length(Length::Px(50.0)),
                    ..Default::default()
                },
                ..Default::default()
            }),
            LayoutNode::new(Style {
                size: SizeStyle {
                    width: LengthOrAuto::Length(Length::Px(70.0)),
                    ..Default::default()
                },
                ..Default::default()
            }),
        ],
    );

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
            ..Default::default()
        },
        [inner_flex],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 0)).content_box.width, 120.0);
    assert_eq!(block_box(node(node(&root, 0), 0)).content_box.width, 50.0);
    assert_eq!(block_box(node(node(&root, 0), 1)).content_box.width, 70.0);
}

// --- Min / max constraints ---

#[test]
fn flex_min_max_constraints() {
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
        flex_container(400.0, 50.0, FlexDirection::Row),
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

// --- Fragments in flex ---

#[test]
fn flex_row_places_fragments_and_nodes() {
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
    assert_eq!(block_box(node(&root, 2)).border_box.x, 50.0);
}

#[test]
fn flex_row_with_only_fragments() {
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
            line_height: Length::Px(20.0),
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [
            LayoutChild::from(fragment(50.0, 10.0)),
            LayoutChild::from(fragment(70.0, 10.0)),
            LayoutChild::from(fragment(30.0, 10.0)),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(
        root.children[0].fragment().unwrap().placement.offset,
        (0.0, 0.0)
    );
    assert_eq!(
        root.children[1].fragment().unwrap().placement.offset,
        (50.0, 0.0)
    );
    assert_eq!(
        root.children[2].fragment().unwrap().placement.offset,
        (120.0, 0.0)
    );
}

#[test]
fn flex_column_with_only_fragments() {
    let mut root = LayoutNode::with_children(
        Style {
            display: Display {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            line_height: Length::Px(20.0),
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        [
            LayoutChild::from(fragment(30.0, 40.0)),
            LayoutChild::from(fragment(30.0, 60.0)),
            LayoutChild::from(fragment(30.0, 30.0)),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // In a column, fragments within the single flex item flow left-to-right
    // since fragments are placed by flow_fragment_range (always horizontal).
    assert_eq!(
        root.children[0].fragment().unwrap().placement.offset,
        (0.0, 0.0)
    );
    assert_eq!(
        root.children[1].fragment().unwrap().placement.offset,
        (30.0, 0.0)
    );
    assert_eq!(
        root.children[2].fragment().unwrap().placement.offset,
        (60.0, 0.0)
    );
}

// --- Inline children in flex ---

#[test]
fn flex_row_with_inline_children() {
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
        [fragment(60.0, 15.0)],
    );

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
        [inline1, inline2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(inline_box_model(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(inline_box_model(node(&root, 1)).border_box.x, 40.0);
}

#[test]
fn flex_column_with_inline_children() {
    let inline1 = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(50.0, 30.0)],
    );

    let inline2 = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(50.0, 50.0)],
    );

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
        [inline1, inline2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(inline_box_model(node(&root, 0)).border_box.y, 0.0);
    // Inline box height is determined by line_height (20px), not fragment height
    assert_eq!(inline_box_model(node(&root, 1)).border_box.y, 20.0);
}

// --- Mixed fragments and inline children ---

#[test]
fn flex_row_with_fragments_and_inline_children() {
    let inline_child = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(50.0, 15.0)],
    );

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
            line_height: Length::Px(20.0),
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [
            LayoutChild::from(fragment(30.0, 10.0)),
            LayoutChild::from(inline_child),
            LayoutChild::from(fragment(40.0, 10.0)),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(
        root.children[0].fragment().unwrap().placement.offset,
        (0.0, 0.0)
    );
    assert_eq!(inline_box_model(node(&root, 1)).border_box.x, 30.0);
    assert_eq!(
        root.children[2].fragment().unwrap().placement.offset,
        (80.0, 0.0)
    );
}

// --- Gap with fragments ---

#[test]
fn flex_gap_with_fragments() {
    let middle_node = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(10.0)),
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
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Length(Length::Px(40.0)),
                ..Default::default()
            },
            line_height: Length::Px(20.0),
            flex_direction: FlexDirection::Row,
            column_gap: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        [
            LayoutChild::from(fragment(30.0, 10.0)),
            LayoutChild::from(fragment(20.0, 10.0)),
            LayoutChild::from(middle_node),
            LayoutChild::from(fragment(40.0, 10.0)),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(
        root.children[0].fragment().unwrap().placement.offset,
        (0.0, 0.0)
    );
    assert_eq!(
        root.children[1].fragment().unwrap().placement.offset,
        (30.0, 0.0)
    );
    // Gap of 15 between the fragment group and node
    assert_eq!(block_box(node(&root, 2)).border_box.x, 65.0);
    // Gap of 15 between node and second fragment group
    assert_eq!(
        root.children[3].fragment().unwrap().placement.offset,
        (90.0, 0.0)
    );
}

#[test]
fn flex_multiple_fragment_groups() {
    let middle_node = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
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
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Length(Length::Px(40.0)),
                ..Default::default()
            },
            line_height: Length::Px(20.0),
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [
            LayoutChild::from(fragment(20.0, 10.0)),
            LayoutChild::from(fragment(30.0, 10.0)),
            LayoutChild::from(middle_node),
            LayoutChild::from(fragment(40.0, 10.0)),
            LayoutChild::from(fragment(10.0, 10.0)),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // First fragment group (2 fragments)
    assert_eq!(
        root.children[0].fragment().unwrap().placement.offset,
        (0.0, 0.0)
    );
    assert_eq!(
        root.children[1].fragment().unwrap().placement.offset,
        (20.0, 0.0)
    );
    // Middle node
    assert_eq!(block_box(node(&root, 2)).border_box.x, 50.0);
    // Second fragment group (2 fragments)
    assert_eq!(
        root.children[3].fragment().unwrap().placement.offset,
        (100.0, 0.0)
    );
    assert_eq!(
        root.children[4].fragment().unwrap().placement.offset,
        (140.0, 0.0)
    );
}

// --- JustifyContent variants ---

#[test]
fn flex_justify_content_space_around() {
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
            justify_content: JustifyContent::SpaceAround,
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // remaining = 300 - (50 + 60 + 40) = 150, gap = 150/3 = 50, start_offset = 25, gap_between = 50
    assert!(approx_eq(block_box(node(&root, 0)).border_box.x, 25.0));
    assert!(approx_eq(block_box(node(&root, 1)).border_box.x, 125.0));
    assert!(approx_eq(block_box(node(&root, 2)).border_box.x, 235.0));
}

#[test]
fn flex_justify_content_end() {
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
            justify_content: JustifyContent::End,
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // remaining = 300 - (50 + 60) = 190, start_offset = 190
    assert_eq!(block_box(node(&root, 0)).border_box.x, 190.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 240.0);
}

// --- AlignItems variants ---

#[test]
fn flex_align_items_center() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
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
                height: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // cross offset = (100 - 30) / 2 = 35
    assert_eq!(block_box(node(&root, 0)).border_box.y, 35.0);
}

#[test]
fn flex_align_items_end() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
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
                height: LengthOrAuto::Length(Length::Px(100.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::End,
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // cross offset = 100 - 30 = 70
    assert_eq!(block_box(node(&root, 0)).border_box.y, 70.0);
}

// --- Row gap in column ---

#[test]
fn flex_column_row_gap() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(40.0)),
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
                height: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Column,
            row_gap: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 50.0);
}

// --- Min/max height constraints ---

#[test]
fn auto_sized_flex_row_uses_flow_child_min_height() {
    let child = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 1.0,
            flex_basis: LengthOrAuto::Length(Length::Px(80.0)),
            ..Default::default()
        },
        size: SizeStyle {
            min_height: LengthOrAuto::Length(Length::Px(70.0)),
            ..Default::default()
        },
        spacing: Spacing {
            padding_top: Length::Px(8.0),
            padding_bottom: Length::Px(8.0),
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
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 300.0, 200.0);

    assert_eq!(block_box(node(&root, 0)).content_box.height, 70.0);
    assert_eq!(block_box(&root).content_box.height, 86.0);
}

#[test]
fn flex_min_height_constraint() {
    let child = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        size: SizeStyle {
            min_height: LengthOrAuto::Length(Length::Px(60.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root =
        LayoutNode::with_children(flex_container(200.0, 100.0, FlexDirection::Column), [child]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert!(block_box(node(&root, 0)).content_box.height >= 60.0);
}

#[test]
fn flex_max_height_constraint() {
    let child = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 1.0,
            ..Default::default()
        },
        size: SizeStyle {
            max_height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root =
        LayoutNode::with_children(flex_container(200.0, 200.0, FlexDirection::Column), [child]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert!(block_box(node(&root, 0)).content_box.height <= 30.0);
}

// --- Empty container ---

#[test]
fn flex_empty_container() {
    let mut root = LayoutNode::new(flex_container(200.0, 100.0, FlexDirection::Row));

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert_eq!(b.content_box.width, 200.0);
    assert_eq!(b.content_box.height, 100.0);
}

// --- Display none child ---

#[test]
fn flex_display_none_child() {
    let visible = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let hidden = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::None,
            inner: InnerDisplay::Flow,
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        flex_container(300.0, 50.0, FlexDirection::Row),
        [visible, hidden],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(node(&root, 0).layout_box.width(), 50.0);
    assert!(node(&root, 1).layout_box.is_empty());
}

// --- inline-flex display ---

#[test]
fn flex_inline_flex_layout() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            display: Display {
                outer: OuterDisplay::Inline,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(50.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::Row,
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert_eq!(b.content_box.width, 200.0);
    assert_eq!(b.content_box.height, 50.0);
    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
}

// --- Nested flex stress ---

fn block_flex() -> Display {
    Display {
        outer: OuterDisplay::Block,
        inner: InnerDisplay::Flex,
    }
}

fn panel(width_scale: f32, w: f32, h: f32) -> LayoutNode {
    let width = if width_scale > 0.0 {
        LengthOrAuto::Length(Length::Percent(width_scale * 100.0))
    } else {
        LengthOrAuto::Length(Length::Px(w))
    };
    LayoutNode::new(Style {
        size: SizeStyle {
            width,
            height: LengthOrAuto::Length(Length::Px(h)),
            ..Default::default()
        },
        ..Default::default()
    })
}

fn branch(depth: usize, max_depth: usize) -> LayoutNode {
    if depth == max_depth {
        return panel(0.0, 40.0, 28.0 + depth as f32 * 4.0);
    }
    LayoutNode::with_children(
        Style {
            display: block_flex(),
            flex_direction: if depth % 2 == 0 {
                FlexDirection::Row
            } else {
                FlexDirection::Column
            },
            row_gap: Length::Px(6.0).into(),
            column_gap: Length::Px(6.0).into(),
            item_style: ItemStyle {
                flex_grow: 1.0,
                flex_basis: Length::Px(60.0).into(),
                ..Default::default()
            },
            spacing: Spacing {
                padding_top: Length::Px(6.0),
                padding_bottom: Length::Px(6.0),
                padding_left: Length::Px(6.0),
                padding_right: Length::Px(6.0),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![
            branch(depth + 1, max_depth),
            branch(depth + 1, max_depth),
            panel(0.5, 50.0, 36.0),
        ],
    )
}

fn complex_nested_stress() -> LayoutNode {
    branch(0, 4)
}

#[test]
fn nested_stress_leaf_panels_have_full_size() {
    let mut root = complex_nested_stress();
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    fn check_leaf_heights(node: &LayoutNode, depth: usize, max_depth: usize) {
        if depth == max_depth {
            let h = block_box(node).content_box.height;
            assert!(
                h >= 35.0,
                "Leaf panel at depth {} has content height {}, expected >= 35.0",
                depth,
                h
            );
            return;
        }
        for child in &node.children {
            if let LayoutChild::Node(n) = child {
                check_leaf_heights(n, depth + 1, max_depth);
            }
        }
    }

    check_leaf_heights(&root, 0, 4);

    let root_box = block_box(&root);
    assert!(
        root_box.content_box.height >= 350.0,
        "Root Row content height should be large enough for nested content, got {}",
        root_box.content_box.height
    );

    for i in 0..3 {
        let d1 = node(&root, i);
        if d1.children.len() >= 2 {
            let d2_0 = node(d1, 0);
            let h0 = block_box(d2_0).content_box.height;
            assert!(
                h0 >= 100.0,
                "depth-2 child[{}] should have content height >= 100, got {}",
                i,
                h0
            );
        }
    }
}

#[test]
fn nested_stress_alternating_directions() {
    let mut root = complex_nested_stress();
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert!(block_box(&root).content_box.width > 0.0);
    assert!(block_box(&root).content_box.height > 0.0);

    assert_eq!(block_box(&root).border_box.x, 0.0);
    assert_eq!(block_box(&root).border_box.y, 0.0);
}
