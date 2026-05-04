use crate::{InnerDisplay, LayoutBoxes, LayoutNode, OuterDisplay, Spacing};

#[derive(Clone, Copy, Default)]
struct Edge {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

pub(crate) struct LayoutContext {
    pub(crate) containing_block_width: Option<f32>,
    pub(crate) containing_block_height: Option<f32>,
    pub(crate) available_width: Option<f32>,
    pub(crate) available_height: Option<f32>,
    pub(crate) viewport_width: f32,
    pub(crate) viewport_height: f32,
}

/// Axis orientation
///
/// Provides helper methods to abstract width/height selection, reducing code duplication
/// for row and column layout support.
#[derive(Debug, Clone, Copy)]
enum Axis {
    Horizontal,
    Vertical,
}

pub struct LayoutEngine;

/// ((end_x, end_y), line_height)
pub(crate) type LineContext = ((f32, f32), f32);

impl LayoutEngine {
    /// Main layout entry point.
    /// Initiates layout computation from the root node with specified viewport dimensions.
    pub fn layout(root: &mut LayoutNode, width: f32, height: f32) {
        let ctx = LayoutContext {
            containing_block_width: Some(width),
            containing_block_height: Some(height),
            available_width: Some(width),
            available_height: Some(height),
            viewport_width: width,
            viewport_height: height,
        };

        Self::layout_node(root, &ctx, ((0.0, 0.0), 0.0), false);
    }

    /// Internal method for layout a node.
    /// Layouts a single node and its descendants.
    fn layout_node(
        node: &mut LayoutNode,
        ctx: &LayoutContext,
        line_ctx: LineContext,
        intrinsic_pass: bool,
    ) -> LineContext {
        if intrinsic_pass {
            let (key, (layout_boxes, line_ctx)) = &node.layout_boxes_cache;
            if *key == crate::cache::make_layout_key(ctx) {
                node.layout_boxes = layout_boxes.clone();
                return *line_ctx;
            }
        }

        let out = Self::layout_by_display(node, &ctx, line_ctx, intrinsic_pass);

        if intrinsic_pass {
            let key = crate::cache::make_layout_key(ctx);
            node.layout_boxes_cache = (key, (node.layout_boxes.clone(), out));
        }

        out
    }

    fn layout_by_display(
        node: &mut LayoutNode,
        ctx: &LayoutContext,
        line_ctx: LineContext,
        intrinsic_pass: bool,
    ) -> LineContext {
        match node.style.display.outer {
            OuterDisplay::None => {
                node.layout_boxes = LayoutBoxes::None;
                line_ctx
            }
            OuterDisplay::Block => Self::layout_block_level(node, ctx, line_ctx, intrinsic_pass),
            OuterDisplay::Inline => Self::layout_inline_level(node, ctx, line_ctx, intrinsic_pass),
        }
    }

    fn layout_block_level(
        node: &mut LayoutNode,
        ctx: &LayoutContext,
        line_ctx: LineContext,
        intrinsic_pass: bool,
    ) -> LineContext {
    }

    fn layout_inline_level(
        node: &mut LayoutNode,
        ctx: &LayoutContext,
        line_ctx: LineContext,
        intrinsic_pass: bool,
    ) -> LineContext {
    }

    fn layout_by_inner_display(
        node: &mut LayoutNode,
        ctx: &LayoutContext,
        line_ctx: LineContext,
        intrinsic_pass: bool,
    ) -> LineContext {
        match node.style.display.inner {
            InnerDisplay::Flow => {}
            InnerDisplay::Flex => {}
        }
    }
}

fn resolve_padding(spacing: &Spacing, ctx: &LayoutContext) -> Edge {
    let containing_width = ctx.containing_block_width;
    let vw = ctx.viewport_width;
    let vh = ctx.viewport_height;

    Edge {
        left: spacing
            .padding_left
            .resolve_with(containing_width, vw, vh)
            .unwrap_or(0.0),
        top: spacing
            .padding_top
            .resolve_with(containing_width, vw, vh)
            .unwrap_or(0.0),
        right: spacing
            .padding_right
            .resolve_with(containing_width, vw, vh)
            .unwrap_or(0.0),
        bottom: spacing
            .padding_bottom
            .resolve_with(containing_width, vw, vh)
            .unwrap_or(0.0),
    }
}

fn resolve_border(spacing: &Spacing, ctx: &LayoutContext) -> Edge {
    let containing_width = ctx.containing_block_width;
    let vw = ctx.viewport_width;
    let vh = ctx.viewport_height;

    Edge {
        left: spacing
            .border_left
            .resolve_with(containing_width, vw, vh)
            .unwrap_or(0.0),
        top: spacing
            .border_top
            .resolve_with(containing_width, vw, vh)
            .unwrap_or(0.0),
        right: spacing
            .border_right
            .resolve_with(containing_width, vw, vh)
            .unwrap_or(0.0),
        bottom: spacing
            .border_bottom
            .resolve_with(containing_width, vw, vh)
            .unwrap_or(0.0),
    }
}
