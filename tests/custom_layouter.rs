mod common;
use common::*;
use ui_layout::*;

// ========================
// Mock implementations
// ========================

/// An inline-level custom object that places a single line span.
#[derive(Debug)]
struct InlineWidget {
    width: f32,
    height: f32,
}

impl CustomLayouter for InlineWidget {
    fn layout(&mut self, ctx: &LayoutContext) -> LayoutBox {
        let (x, y) = ctx.start_pos;
        LayoutBox::InlineBox(InlineBox {
            box_model: BoxModel::from(rect(x, y, self.width, ctx.line_height)),
            line_spans: vec![LineSpan {
                x_range: x..(x + self.width),
                line_pos: (x, y),
                line_index: 0,
            }],
        })
    }

    fn measure(&self, _ctx: &LayoutContext) -> MeasureResult {
        MeasureResult {
            width: self.width,
            height: self.height,
        }
    }
}

/// A block-level custom object that occupies a fixed rect.
#[derive(Debug)]
struct BlockBox {
    width: f32,
    height: f32,
}

impl CustomLayouter for BlockBox {
    fn layout(&mut self, _ctx: &LayoutContext) -> LayoutBox {
        LayoutBox::BlockBox(BoxModel::from(rect(0.0, 0.0, self.width, self.height)))
    }

    fn measure(&self, _ctx: &LayoutContext) -> MeasureResult {
        MeasureResult {
            width: self.width,
            height: self.height,
        }
    }
}

/// A custom object that participates in no formatting context.
#[derive(Debug)]
struct HiddenBox;

impl CustomLayouter for HiddenBox {
    fn measure(&self, _ctx: &LayoutContext) -> MeasureResult {
        MeasureResult::default()
    }
}

/// An inline-declared object that (mismatched) returns a block box.
#[derive(Debug)]
struct InlineDeclaredBlock {
    width: f32,
    height: f32,
}

impl CustomLayouter for InlineDeclaredBlock {
    fn layout(&mut self, _ctx: &LayoutContext) -> LayoutBox {
        LayoutBox::BlockBox(BoxModel::from(rect(0.0, 0.0, self.width, self.height)))
    }

    fn measure(&self, _ctx: &LayoutContext) -> MeasureResult {
        MeasureResult {
            width: self.width,
            height: self.height,
        }
    }
}

/// A block-declared object that (mismatched) returns an inline box.
#[derive(Debug)]
struct BlockDeclaredInline {
    width: f32,
    height: f32,
}

impl CustomLayouter for BlockDeclaredInline {
    fn layout(&mut self, _ctx: &LayoutContext) -> LayoutBox {
        LayoutBox::InlineBox(InlineBox {
            box_model: BoxModel::from(rect(0.0, 0.0, self.width, self.height)),
            line_spans: vec![LineSpan {
                x_range: 0.0..self.width,
                line_pos: (0.0, 0.0),
                line_index: 0,
            }],
        })
    }

    fn measure(&self, _ctx: &LayoutContext) -> MeasureResult {
        MeasureResult {
            width: self.width,
            height: self.height,
        }
    }
}

// ========================
// Block custom in flow
// ========================

#[test]
fn block_custom_stacks_vertically_in_flow() {
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
            custom_block(BlockBox {
                width: 100.0,
                height: 30.0,
            }),
            custom_block(BlockBox {
                width: 100.0,
                height: 50.0,
            }),
            custom_block(BlockBox {
                width: 100.0,
                height: 20.0,
            }),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.content_box.height >= 100.0);

    let r0 = root.children[0]
        .custom_result()
        .unwrap()
        .box_model
        .border_box;
    let r1 = root.children[1]
        .custom_result()
        .unwrap()
        .box_model
        .border_box;
    let r2 = root.children[2]
        .custom_result()
        .unwrap()
        .box_model
        .border_box;

    assert_eq!(r0.y, 0.0);
    assert_eq!(r1.y, 30.0);
    assert_eq!(r2.y, 80.0);
    assert_eq!(r0.height, 30.0);
    assert_eq!(r1.height, 50.0);
}

#[test]
fn block_custom_result_observable_via_custom_result() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [custom_block(BlockBox {
            width: 80.0,
            height: 40.0,
        })],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let result = root.children[0].custom_result().unwrap();
    assert!(result.spans.is_empty());
    assert_eq!(result.box_model.border_box.size(), (80.0, 40.0));
    assert!(result.box_model.border_box.y >= 0.0);
}

// ========================
// Inline custom in flow
// ========================

#[test]
fn inline_custom_result_stores_spans_and_box() {
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
        [custom_inline(InlineWidget {
            width: 40.0,
            height: 10.0,
        })],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let result = root.children[0].custom_result().unwrap();
    assert_eq!(result.spans.len(), 1);
    assert_eq!(result.spans[0].width(), 40.0);
    assert_eq!(result.box_model.border_box.width, 40.0);
    assert_eq!(result.box_model.border_box.height, 20.0);
}

// ========================
// Mixed inline and block custom
// ========================

#[test]
fn block_custom_after_inline_forces_new_line() {
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
            custom_inline(InlineWidget {
                width: 40.0,
                height: 10.0,
            }),
            custom_block(BlockBox {
                width: 100.0,
                height: 30.0,
            }),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert!(b.content_box.height >= 50.0);

    let block = root.children[1]
        .custom_result()
        .unwrap()
        .box_model
        .border_box;
    assert_eq!(block.x, 0.0);
    assert_eq!(block.y, 20.0);
}

#[test]
fn inline_custom_after_block_flows_below() {
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
            custom_block(BlockBox {
                width: 100.0,
                height: 30.0,
            }),
            custom_inline(InlineWidget {
                width: 40.0,
                height: 10.0,
            }),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let inline = root.children[1]
        .custom_result()
        .unwrap()
        .box_model
        .border_box;
    assert_eq!(inline.y, 30.0);
}

// ========================
// None formatting context
// ========================

#[test]
fn none_custom_is_skipped_in_flow() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [custom_none(HiddenBox)],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert_eq!(b.content_box.height, 0.0);
    assert!(root.children[0].custom_result().is_none());
}

// ========================
// Block custom in flex
// ========================

#[test]
fn flex_stores_block_custom_final_rect() {
    let mut root = LayoutNode::with_children(
        flex_container(200.0, 50.0, FlexDirection::Row),
        [
            custom_block(BlockBox {
                width: 60.0,
                height: 30.0,
            }),
            custom_block(BlockBox {
                width: 40.0,
                height: 20.0,
            }),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let r0 = root.children[0]
        .custom_result()
        .unwrap()
        .box_model
        .border_box;
    let r1 = root.children[1]
        .custom_result()
        .unwrap()
        .box_model
        .border_box;

    assert_eq!(r0.size(), (60.0, 30.0));
    assert_eq!(r1.size(), (40.0, 20.0));
    assert_eq!(r1.x, 60.0);
}

#[test]
fn flex_skips_none_custom() {
    let mut root = LayoutNode::with_children(
        flex_container(200.0, 50.0, FlexDirection::Row),
        [
            custom_block(BlockBox {
                width: 60.0,
                height: 30.0,
            }),
            custom_none(HiddenBox),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let r0 = root.children[0]
        .custom_result()
        .unwrap()
        .box_model
        .border_box;
    assert_eq!(r0.size(), (60.0, 30.0));
    assert!(root.children[1].custom_result().is_none());
}

// ========================
// Mismatch: inline-declared object returning BlockBox
// ========================

#[test]
fn inline_declared_block_placed_atomically_on_line() {
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
            custom_inline(InlineDeclaredBlock {
                width: 40.0,
                height: 15.0,
            }),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let result = root.children[1].custom_result().unwrap();
    assert_eq!(result.spans.len(), 1);
    assert_eq!(result.spans[0].line_pos, (30.0, 0.0));
    assert_eq!(result.spans[0].line_index, 0);
    assert_eq!(result.spans[0].width(), 40.0);

    let border_box = result.box_model.border_box;
    assert_eq!((border_box.x, border_box.y), (30.0, 0.0));
    assert_eq!(border_box.size(), (40.0, 15.0));

    let b = block_box(&root);
    assert!(b.content_box.height >= 20.0);
}

#[test]
fn inline_declared_block_wraps_whole_to_next_line() {
    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(50.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            line_height: Length::Px(20.0),
            ..Default::default()
        },
        [
            LayoutChild::from(fragment(30.0, 10.0)),
            custom_inline(InlineDeclaredBlock {
                width: 40.0,
                height: 15.0,
            }),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let result = root.children[1].custom_result().unwrap();
    assert_eq!(result.spans.len(), 1);
    assert_eq!(result.spans[0].line_pos, (0.0, 20.0));
    assert_eq!(result.spans[0].line_index, 1);
    assert_eq!(result.spans[0].width(), 40.0);

    let border_box = result.box_model.border_box;
    assert_eq!((border_box.x, border_box.y), (0.0, 20.0));
    assert_eq!(border_box.size(), (40.0, 15.0));

    let b = block_box(&root);
    assert!(
        b.content_box.height >= 40.0,
        "wrapped block should produce 2 lines, height >= 40, got {}",
        b.content_box.height
    );
}

// ========================
// Mismatch: block-declared object returning InlineBox (anonymous block)
// ========================

#[test]
fn block_declared_inline_placed_as_anonymous_block() {
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
        [custom_block(BlockDeclaredInline {
            width: 100.0,
            height: 30.0,
        })],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let result = root.children[0].custom_result().unwrap();
    assert_eq!(result.spans.len(), 1);
    assert_eq!(result.spans[0].width(), 100.0);

    let border_box = result.box_model.border_box;
    assert_eq!((border_box.x, border_box.y), (0.0, 0.0));
    assert_eq!(border_box.size(), (100.0, 30.0));

    let b = block_box(&root);
    assert!(b.content_box.height >= 30.0);
}

#[test]
fn block_declared_inline_forces_new_line_after_inline() {
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
            custom_inline(InlineWidget {
                width: 40.0,
                height: 10.0,
            }),
            custom_block(BlockDeclaredInline {
                width: 100.0,
                height: 30.0,
            }),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let border_box = root.children[1]
        .custom_result()
        .unwrap()
        .box_model
        .border_box;
    assert_eq!(border_box.x, 0.0);
    assert_eq!(border_box.y, 20.0);

    let b = block_box(&root);
    assert!(b.content_box.height >= 50.0);
}

#[derive(Debug)]
struct EmptySpanInlineWidget {
    width: f32,
    height: f32,
}

impl CustomLayouter for EmptySpanInlineWidget {
    fn layout(&mut self, ctx: &LayoutContext) -> LayoutBox {
        let (x, y) = ctx.start_pos;
        LayoutBox::InlineBox(InlineBox {
            box_model: BoxModel::from(rect(x, y, self.width, self.height)),
            line_spans: vec![],
        })
    }

    fn measure(&self, _ctx: &LayoutContext) -> MeasureResult {
        MeasureResult {
            width: self.width,
            height: self.height,
        }
    }
}

#[test]
fn inline_block_auto_width_with_empty_span_custom_child() {
    let custom_child = EmptySpanInlineWidget {
        width: 160.0,
        height: 16.0,
    };

    let inline_block = LayoutNode::with_children(
        Style {
            display: Display::parse("inline-block").unwrap(),
            ..Default::default()
        },
        vec![custom_inline(custom_child)],
    );

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(400.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        vec![inline_block],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let ib_node = node(&root, 0);
    assert_eq!(ib_node.layout_box.width(), 160.0);
    assert_eq!(ib_node.layout_box.width_box(), 160.0);
}

#[test]
fn flex_item_inline_block_with_custom_child_gets_intrinsic_width() {
    let custom_child = InlineWidget {
        width: 160.0,
        height: 16.0,
    };

    let inline_block = LayoutNode::with_children(
        Style {
            display: Display::parse("inline-block").unwrap(),
            ..Default::default()
        },
        vec![custom_inline(custom_child)],
    );

    let mut root = LayoutNode::with_children(
        flex_container(400.0, 200.0, FlexDirection::Row),
        vec![inline_block],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let ib_node = node(&root, 0);
    assert_eq!(ib_node.layout_box.width_box(), 160.0);
    assert_eq!(ib_node.layout_box.width(), 160.0);
}
