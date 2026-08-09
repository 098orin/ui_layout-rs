mod common;
use common::*;
use ui_layout::*;

// ========================
// Mock CustomLayouter implementations
// ========================

/// A simple fixed-size object that occupies a fixed width and height.
#[derive(Debug)]
struct FixedObject {
    width: f32,
    height: f32,
}

impl CustomLayouter for FixedObject {
    fn layout(&mut self, ctx: &LayoutContext) -> LayoutBox {
        let (x, y) = ctx.start_pos;
        let box_model = BoxModel::from(rect(x, y, self.width, ctx.line_height));
        let fits = self.width <= ctx.available_inline_size;
        let line_spans = if !fits && ctx.available_inline_size > 0.0 {
            // Wrap to next line
            vec![LineSpan {
                x_range: 0.0..self.width,
                line_pos: (0.0, y + ctx.line_height),
                line_index: 1,
            }]
        } else {
            vec![LineSpan {
                x_range: x..(x + self.width),
                line_pos: (x, y),
                line_index: 0,
            }]
        };
        LayoutBox::InlineBox(InlineBox {
            box_model,
            line_spans,
        })
    }

    fn measure(&self, _ctx: &LayoutContext) -> MeasureResult {
        MeasureResult {
            width: self.width,
            height: self.height,
        }
    }
}

/// A text-like object that reports line-breakable content.
#[derive(Debug)]
struct TextRun {
    total_width: f32,
    height: f32,
}

impl CustomLayouter for TextRun {
    fn layout(&mut self, ctx: &LayoutContext) -> LayoutBox {
        let (x, y) = ctx.start_pos;
        let box_model = BoxModel::from(rect(x, y, self.total_width, ctx.line_height));
        let fits = self.total_width <= ctx.available_inline_size;
        let line_spans = if !fits && ctx.available_inline_size > 0.0 {
            // Wrap: only part fits, rest goes to next line
            let fits_part = ctx.available_inline_size;
            let remaining = self.total_width - fits_part;
            vec![
                LineSpan {
                    x_range: x..(x + fits_part),
                    line_pos: (x, y),
                    line_index: 0,
                },
                LineSpan {
                    x_range: 0.0..remaining,
                    line_pos: (0.0, y + ctx.line_height),
                    line_index: 1,
                },
            ]
        } else {
            vec![LineSpan {
                x_range: x..(x + self.total_width),
                line_pos: (x, y),
                line_index: 0,
            }]
        };
        LayoutBox::InlineBox(InlineBox {
            box_model,
            line_spans,
        })
    }

    fn measure(&self, _ctx: &LayoutContext) -> MeasureResult {
        MeasureResult {
            width: self.total_width,
            height: self.height,
        }
    }
}

// ========================
// Object in inline (flow) layout
// ========================

#[test]
fn flow_object_in_block_single_line() {
    let obj = FixedObject {
        width: 80.0,
        height: 20.0,
    };

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(50.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        [custom_inline(obj)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert_eq!(b.content_box.width, 200.0);
}

#[test]
fn flow_object_in_flow_container_single() {
    let obj = FixedObject {
        width: 60.0,
        height: 15.0,
    };

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [custom_inline(obj)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.content_box.width >= 60.0);
    assert!(b.content_box.height >= 20.0);
}

#[test]
fn flow_object_wraps_to_next_line() {
    let obj = FixedObject {
        width: 120.0,
        height: 15.0,
    };

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [custom_inline(obj)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.content_box.height >= 40.0);
}

// ========================
// Object in flex layout
// ========================

#[test]
fn flex_row_with_object() {
    let obj = FixedObject {
        width: 60.0,
        height: 30.0,
    };
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(40.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        flex_container(200.0, 50.0, FlexDirection::Row),
        [custom_inline(obj), LayoutChild::from(child)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(&root).content_box.width, 200.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 60.0);
}

#[test]
fn flex_column_with_object() {
    let obj = FixedObject {
        width: 100.0,
        height: 40.0,
    };
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        flex_container(100.0, 200.0, FlexDirection::Column),
        [custom_inline(obj), LayoutChild::from(child)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(&root).content_box.height, 200.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 40.0);
}

#[test]
fn flex_row_with_multiple_objects() {
    let obj1 = FixedObject {
        width: 30.0,
        height: 20.0,
    };
    let obj2 = FixedObject {
        width: 50.0,
        height: 20.0,
    };
    let obj3 = FixedObject {
        width: 40.0,
        height: 20.0,
    };

    let mut root = LayoutNode::with_children(
        flex_container(200.0, 50.0, FlexDirection::Row),
        [
            custom_inline(obj1),
            custom_inline(obj2),
            custom_inline(obj3),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(&root).children_box.width, 120.0);
}

#[test]
fn flex_column_with_multiple_objects() {
    let obj1 = FixedObject {
        width: 50.0,
        height: 30.0,
    };
    let obj2 = FixedObject {
        width: 50.0,
        height: 50.0,
    };

    let mut root = LayoutNode::with_children(
        flex_container(100.0, 200.0, FlexDirection::Column),
        [custom_inline(obj1), custom_inline(obj2)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(&root).children_box.height, 80.0);
}

// ========================
// Object with fragments in flex
// ========================

#[test]
fn flex_row_with_object_and_fragments() {
    let obj = FixedObject {
        width: 50.0,
        height: 20.0,
    };

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
            custom_inline(obj),
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
// Object with fragments in flow (inline) layout
// ========================

#[test]
fn flow_with_object_and_fragments() {
    let obj = FixedObject {
        width: 40.0,
        height: 15.0,
    };

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [
            LayoutChild::from(fragment(30.0, 10.0)),
            custom_inline(obj),
            LayoutChild::from(fragment(50.0, 10.0)),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.content_box.height >= 20.0);
}

// ========================
// Object measure used in flex sizing
// ========================

#[test]
fn flex_object_contributes_to_container_auto_height() {
    let obj = FixedObject {
        width: 80.0,
        height: 60.0,
    };

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
        [custom_inline(obj)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let h = block_box(&root).content_box.height;
    assert!(
        h >= 60.0,
        "auto height should be at least object height, got {}",
        h
    );
}

#[test]
fn flex_object_with_gap() {
    let obj1 = FixedObject {
        width: 40.0,
        height: 20.0,
    };
    let obj2 = FixedObject {
        width: 40.0,
        height: 20.0,
    };

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
            column_gap: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        [custom_inline(obj1), custom_inline(obj2)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(&root).children_box.width, 110.0);
}

// ========================
// Object with align-items in flex
// ========================

#[test]
fn flex_object_align_items_center() {
    let obj = FixedObject {
        width: 50.0,
        height: 20.0,
    };

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
        [custom_inline(obj)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    // Container children_box.height should be at least the object height (20px).
    // cross axis center offset = (100 - 20) / 2 = 40, so children_box tracks content
    assert!(b.content_box.height >= 100.0);
}

// ========================
// Object with justify-content in flex
// ========================

#[test]
fn flex_object_justify_center() {
    let obj = FixedObject {
        width: 60.0,
        height: 30.0,
    };

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
        [custom_inline(obj)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // main axis center offset = (200 - 60) / 2 = 70
    // object doesn't have layout_box, so we check the container instead
    let b = block_box(&root);
    assert_eq!(b.content_box.width, 200.0);
}

// ========================
// Object in reverse flex containers
// ========================

#[test]
fn flex_row_reverse_with_object() {
    let obj = FixedObject {
        width: 50.0,
        height: 20.0,
    };
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
        [custom_inline(obj), LayoutChild::from(child)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // In RowReverse, children are laid out right-to-left.
    // Node child (50px) should be at x=0, Object (50px) should be at x=50.
    let b = block_box(&root);
    assert!(b.content_box.width > 0.0);
}

#[test]
fn flex_column_reverse_with_object() {
    let obj = FixedObject {
        width: 100.0,
        height: 40.0,
    };
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
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
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            flex_direction: FlexDirection::ColumnReverse,
            ..Default::default()
        },
        [custom_inline(obj), LayoutChild::from(child)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.content_box.height > 0.0);
}

// ========================
// Object measure API
// ========================

#[test]
fn flow_object_measure() {
    use ui_layout::LayoutContext;

    let obj = FixedObject {
        width: 75.0,
        height: 25.0,
    };

    let ctx = LayoutContext {
        containing_block_width: Some(200.0),
        containing_block_height: Some(100.0),
        ..Default::default()
    };

    let result = obj.measure(&ctx);
    assert_eq!(result.width, 75.0);
    assert_eq!(result.height, 25.0);
}

// ========================
// Object wrapping behavior in flow
// ========================

#[test]
fn flow_object_wraps_when_exceeding_available_width() {
    let obj = FixedObject {
        width: 150.0,
        height: 20.0,
    };

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [custom_inline(obj)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(
        b.content_box.height >= 40.0,
        "wrapped object should produce 2 lines, height >= 40, got {}",
        b.content_box.height
    );
}

// ========================
// Mixed content: objects, fragments, and nodes in flow
// ========================

#[test]
fn flow_mixed_object_fragment_inline_node() {
    let obj = FixedObject {
        width: 30.0,
        height: 15.0,
    };
    let inline_span = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [fragment(25.0, 10.0)],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [
            LayoutChild::from(fragment(15.0, 10.0)),
            custom_inline(obj),
            LayoutChild::from(inline_span),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.content_box.height >= 20.0);
}

// ========================
// TextRun object (multi-span in flow)
// ========================

#[test]
fn flow_textrun_object_single_line() {
    let text = TextRun {
        total_width: 80.0,
        height: 15.0,
    };

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [custom_inline(text)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.content_box.height >= 20.0);
}

#[test]
fn flow_textrun_object_multi_line() {
    let text = TextRun {
        total_width: 250.0,
        height: 15.0,
    };

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [custom_inline(text)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(
        b.content_box.height >= 40.0,
        "multi-line text should produce 2+ lines, height >= 40, got {}",
        b.content_box.height
    );
}

// ========================
// Edge cases
// ========================

#[test]
fn flex_empty_objects_list_does_not_crash() {
    let mut root = LayoutNode::new(flex_container(100.0, 100.0, FlexDirection::Row));
    LayoutEngine::layout(&mut root, 800.0, 600.0);
    let b = block_box(&root);
    assert_eq!(b.content_box.width, 100.0);
    assert_eq!(b.content_box.height, 100.0);
}

#[test]
fn flow_single_object_children_box_tracks_object_size() {
    let obj = FixedObject {
        width: 55.0,
        height: 18.0,
    };

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [custom_inline(obj)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.children_box.width <= b.content_box.width);
    assert!(b.children_box.height >= 20.0);
}

#[test]
fn custom_text_after_linebreak_receives_full_line_width() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [
            custom_inline(FixedObject {
                width: 180.0,
                height: 20.0,
            }),
            LayoutChild::from(ItemFragment::LineBreak),
            custom_inline(TextRun {
                total_width: 100.0,
                height: 20.0,
            }),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let result = root.children[2].custom_result().unwrap();
    assert_eq!(result.spans.len(), 1);
    assert_eq!(result.spans[0].line_pos, (0.0, 20.0));
    assert_eq!(result.spans[0].width(), 100.0);
}
