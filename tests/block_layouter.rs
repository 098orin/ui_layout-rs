mod common;
use common::*;
use ui_layout::*;

#[derive(Debug)]
struct MockBlock {
    width: f32,
    height: f32,
}

impl BlockLayouter for MockBlock {
    fn layout(&self, ctx: &LayoutContext) -> Rect {
        let w = ctx
            .available_width
            .unwrap_or(ctx.containing_block_width.unwrap_or(self.width))
            .min(self.width);
        let h = ctx
            .containing_block_height
            .map(|ch| ch.min(self.height))
            .unwrap_or(self.height);
        rect(0.0, 0.0, w, h)
    }
}

fn block_child(w: f32, h: f32) -> LayoutChild {
    LayoutChild::Custom {
        layouter: Box::new(MockBlock {
            width: w,
            height: h,
        }),
        node: Box::new(LayoutNode::new(Style::default())),
    }
}

// ========================
// Block in flow (block) layout
// ========================

#[test]
fn block_in_flow_single() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [block_child(100.0, 40.0)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.content_box.height >= 40.0);
}

#[test]
fn block_in_flow_stacks_vertically() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [block_child(100.0, 30.0), block_child(100.0, 50.0)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.content_box.height >= 80.0);
}

#[test]
fn block_width_clamped_to_containing() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(80.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [block_child(200.0, 30.0)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.content_box.width <= 80.0);
}

#[test]
fn block_in_flow_three_children() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [
            block_child(100.0, 20.0),
            block_child(100.0, 30.0),
            block_child(100.0, 40.0),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.content_box.height >= 90.0);
}

// ========================
// Block in flex layout
// ========================

#[test]
fn block_in_flex_row() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        flex_container(200.0, 60.0, FlexDirection::Row),
        [block_child(60.0, 40.0), LayoutChild::from(child)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(&root).content_box.width, 200.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 60.0);
}

#[test]
fn block_in_flex_column() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        flex_container(100.0, 200.0, FlexDirection::Column),
        [block_child(80.0, 40.0), LayoutChild::from(child)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 1)).border_box.y, 40.0);
}

#[test]
fn block_with_gap_in_flex() {
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
            column_gap: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        [block_child(40.0, 20.0), block_child(40.0, 20.0)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(&root).children_box.width, 90.0);
}

#[test]
fn block_align_items_center_in_flex() {
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
        [block_child(50.0, 30.0)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.content_box.height >= 100.0);
}

#[test]
fn block_justify_center_in_flex() {
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
            justify_content: JustifyContent::Center,
            ..Default::default()
        },
        [block_child(60.0, 30.0)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert_eq!(b.content_box.width, 200.0);
}

#[test]
fn block_in_flex_auto_height() {
    let mut root = LayoutNode::with_children(
        Style {
            display: Display {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            flex_direction: FlexDirection::Column,
            ..Default::default()
        },
        [block_child(100.0, 60.0)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let h = block_box(&root).content_box.height;
    assert!(
        h >= 60.0,
        "auto height should be at least block height, got {}",
        h
    );
}

// ========================
// Block in reverse flex
// ========================

#[test]
fn block_row_reverse() {
    let child = LayoutNode::new(Style {
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
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(50.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::RowReverse,
            ..Default::default()
        },
        [block_child(50.0, 30.0), LayoutChild::from(child)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.content_box.width > 0.0);
}

#[test]
fn block_column_reverse() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
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
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::ColumnReverse,
            ..Default::default()
        },
        [block_child(80.0, 40.0), LayoutChild::from(child)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.content_box.height > 0.0);
}

// ========================
// Block mixed with fragments
// ========================

#[test]
fn block_mixed_with_fragments_in_flex() {
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
            LayoutChild::from(fragment(30.0, 10.0)),
            block_child(50.0, 20.0),
            LayoutChild::from(fragment(20.0, 10.0)),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(
        root.children[0].fragment().unwrap().placement.offset,
        (0.0, 0.0)
    );
    assert_eq!(
        root.children[2].fragment().unwrap().placement.offset,
        (80.0, 0.0)
    );
}

// ========================
// Block with margin
// ========================

#[test]
fn block_with_margin() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [LayoutChild::Custom {
            layouter: Box::new(MockBlock {
                width: 100.0,
                height: 40.0,
            }),
            node: Box::new(LayoutNode::new(Style {
                spacing: Spacing {
                    margin_top: LengthOrAuto::Length(Length::Px(5.0)),
                    margin_right: LengthOrAuto::Length(Length::Px(10.0)),
                    margin_bottom: LengthOrAuto::Length(Length::Px(5.0)),
                    margin_left: LengthOrAuto::Length(Length::Px(10.0)),
                    ..Default::default()
                },
                ..Default::default()
            })),
        }],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Auto-height parent: content_height = child height (40) minus collapsed bottom margin (5) = 35
    let b = block_box(&root);
    assert!(b.content_box.height >= 35.0);
    // top margin collapses to first_child_margin (returned as margin_start on LineContext)
    // child is positioned at y=0 since top margin collapses
    let (_, child_node) = root.children[0].custom().unwrap();
    let cb = block_box(child_node);
    assert!(
        cb.border_box.height >= 40.0,
        "child should have valid height"
    );
}

// ========================
// Block with padding
// ========================

#[test]
fn block_with_padding() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [LayoutChild::Custom {
            layouter: Box::new(MockBlock {
                width: 100.0,
                height: 40.0,
            }),
            node: Box::new(LayoutNode::new(Style {
                spacing: Spacing {
                    padding_top: Length::Px(5.0),
                    padding_right: Length::Px(10.0),
                    padding_bottom: Length::Px(5.0),
                    padding_left: Length::Px(10.0),
                    ..Default::default()
                },
                ..Default::default()
            })),
        }],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.content_box.height >= 50.0);
}

// ========================
// Block with border
// ========================

#[test]
fn block_with_border() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [LayoutChild::Custom {
            layouter: Box::new(MockBlock {
                width: 100.0,
                height: 40.0,
            }),
            node: Box::new(LayoutNode::new(Style {
                spacing: Spacing {
                    border_top: Length::Px(2.0),
                    border_right: Length::Px(2.0),
                    border_bottom: Length::Px(2.0),
                    border_left: Length::Px(2.0),
                    ..Default::default()
                },
                ..Default::default()
            })),
        }],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.content_box.height >= 44.0);
}

// ========================
// Block margin collapse in flow
// ========================

#[test]
fn block_in_flow_with_margin_collapse() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [
            LayoutChild::Custom {
                layouter: Box::new(MockBlock {
                    width: 100.0,
                    height: 30.0,
                }),
                node: Box::new(LayoutNode::new(Style {
                    spacing: Spacing {
                        margin_bottom: LengthOrAuto::Length(Length::Px(10.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                })),
            },
            LayoutChild::Custom {
                layouter: Box::new(MockBlock {
                    width: 100.0,
                    height: 30.0,
                }),
                node: Box::new(LayoutNode::new(Style {
                    spacing: Spacing {
                        margin_top: LengthOrAuto::Length(Length::Px(5.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                })),
            },
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    // 30 + 30 = 60 content, margins are positioning only (not additive to box height in flow)
    assert!(b.content_box.height >= 60.0);
}

// ========================
// Block mixed with node children
// ========================

#[test]
fn block_mixed_with_node_children() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(25.0)),
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
        [block_child(100.0, 40.0), LayoutChild::from(child)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.content_box.height >= 65.0);
}

// ========================
// Block multiple blocks in flex column
// ========================

#[test]
fn flex_column_with_multiple_blocks() {
    let mut root = LayoutNode::with_children(
        flex_container(100.0, 200.0, FlexDirection::Column),
        [block_child(50.0, 30.0), block_child(50.0, 50.0)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(&root).children_box.height, 80.0);
}

// ========================
// Block in flex with row gap
// ========================

#[test]
fn block_with_row_gap_in_flex_column() {
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
            row_gap: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        [block_child(50.0, 30.0), block_child(50.0, 40.0)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(&root).children_box.height, 85.0);
}

// ========================
// Block children_box tracks content
// ========================

#[test]
fn block_single_child_children_box_tracks_size() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [block_child(120.0, 55.0)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.children_box.width <= b.content_box.width);
    assert!(b.children_box.height >= 55.0);
}
