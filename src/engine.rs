use crate::{
    BoxSizing, FragmentNode, InnerDisplay, LayoutBox, LayoutNode, OuterDisplay, Rect, Spacing,
};

#[derive(Clone, Copy, Default)]
struct Edge {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

#[derive(Clone, Copy, Default)]
pub struct EdgeOption {
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
}

/// The difference between containing_block_* and available_* is:
///
/// - containing_block_*:
///   The base size used for resolving percentages and intrinsic sizing.
///   Independent of layout results.
///
/// - available_*:
///   The actual free space available for layout after considering
///   constraints such as sibling layout, margins, and line breaking.
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

/// (((end_x, end_y), line_height), (margin_end))
///
/// Every field will be zero for non-inline contexts.
pub(crate) type LineContext = (((f32, f32), f32), (f32));

pub(crate) const EMPTY_LINE_CONTEXT: LineContext = (((0.0, 0.0), 0.0), (0.0));

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

        Self::layout_node(root, &ctx, EMPTY_LINE_CONTEXT, false);
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
            let (key, (layout_box, line_ctx)) = &node.layout_box_cache;
            if *key == crate::cache::make_layout_key(ctx) {
                node.layout_box = layout_box.clone();
                return *line_ctx;
            }
        }

        let out = Self::layout_by_display(node, &ctx, line_ctx, intrinsic_pass);

        if intrinsic_pass {
            let key = crate::cache::make_layout_key(ctx);
            node.layout_box_cache = (key, (node.layout_box.clone(), out));
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
                node.layout_box = LayoutBox::None;
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
        let ((width_opt, height_opt), _, _) = resolve_base_content_size_and_spacing(
            &node.style.size,
            &node.style.spacing,
            &node.style.box_sizing,
            ctx,
        );

        let width_opt = width_opt.or(ctx.available_width);

        Self::layout_by_inner_display(
            node,
            &ctx,
            line_ctx,
            (width_opt, height_opt),
            intrinsic_pass,
        )
    }

    fn layout_inline_level(
        node: &mut LayoutNode,
        ctx: &LayoutContext,
        line_ctx: LineContext,
        intrinsic_pass: bool,
    ) -> LineContext {
        let ((width_opt, height_opt), _, _) = resolve_base_content_size_and_spacing(
            &node.style.size,
            &node.style.spacing,
            &node.style.box_sizing,
            ctx,
        );

        Self::layout_by_inner_display(
            node,
            &ctx,
            line_ctx,
            (width_opt, height_opt),
            intrinsic_pass,
        )
    }

    fn layout_by_inner_display(
        node: &mut LayoutNode,
        ctx: &LayoutContext,
        line_ctx: LineContext,
        size_opt: (Option<f32>, Option<f32>),
        intrinsic_pass: bool,
    ) -> LineContext {
        match node.style.display.inner {
            InnerDisplay::Flow => Self::layout_flow(node, ctx, line_ctx, size_opt, intrinsic_pass),
            InnerDisplay::Flex => {
                todo!()
            }
        }
    }

    fn layout_flow(
        node: &mut LayoutNode,
        ctx: &LayoutContext,
        line_ctx: LineContext,
        size_opt: (Option<f32>, Option<f32>),
        intrinsic_pass: bool,
    ) -> LineContext {
        todo!()
    }

    fn flow_fragments(fragments: &mut Vec<FragmentNode>, line_ctx: LineContext, outline: Rect) {}
}

// ==========================================

/// ((width_opt, height_opt), border, padding)
fn resolve_base_content_size_and_spacing(
    size_style: &crate::SizeStyle,
    spacing: &crate::Spacing,
    box_sizing: &BoxSizing,
    ctx: &LayoutContext,
) -> ((Option<f32>, Option<f32>), Edge, Edge) {
    let border = resolve_border(&spacing, ctx);
    let padding = resolve_padding(&spacing, ctx);

    let vw = ctx.viewport_width;
    let vh = ctx.viewport_height;

    // --- width ---
    let content_width = size_style
        .width
        .resolve_with(ctx.containing_block_width, vw, vh)
        .map(|width| {
            let padding_edge = (padding.left, padding.right);
            let border_edge = (border.left, border.right);
            resolve_content_size_with_box_sizing(box_sizing, width, padding_edge, border_edge)
        })
        .map(|width| apply_size_constraints(width, size_style, ctx, true));

    // --- height ---
    let content_height = size_style
        .height
        .resolve_with(ctx.containing_block_height, vw, vh)
        .map(|height| {
            let padding_edge = (padding.top, padding.bottom);
            let border_edge = (border.top, border.bottom);
            resolve_content_size_with_box_sizing(box_sizing, height, padding_edge, border_edge)
        })
        .map(|height| apply_size_constraints(height, size_style, ctx, false));

    ((content_width, content_height), border, padding)
}

/// Resolves content size based on the box-sizing property.
///
/// # Arguments
/// * `box_sizing` - Box-sizing style
/// * `size` - The border or content box size to resolve
/// * `padding_edge` - (padding start, padding end)
/// * `border_edge` - (border start, border end)
fn resolve_content_size_with_box_sizing(
    box_sizing: &BoxSizing,
    size: f32,
    padding_edge: (f32, f32),
    border_edge: (f32, f32),
) -> f32 {
    match box_sizing {
        BoxSizing::ContentBox => size,
        BoxSizing::BorderBox => {
            size - padding_edge.0 - padding_edge.1 - border_edge.0 - border_edge.1
        }
    }
    .max(0.0)
}

/// Applies min/max size constraints to a dimension value.
fn apply_size_constraints(
    value: f32,
    size_style: &crate::SizeStyle,
    ctx: &LayoutContext,
    is_width: bool,
) -> f32 {
    let vw = ctx.viewport_width;
    let vh = ctx.viewport_height;

    let (min_constraint, max_constraint) = if is_width {
        (
            size_style
                .min_width
                .resolve_with(ctx.containing_block_width, vw, vh),
            size_style
                .max_width
                .resolve_with(ctx.containing_block_width, vw, vh),
        )
    } else {
        (
            size_style
                .min_height
                .resolve_with(ctx.containing_block_height, vw, vh),
            size_style
                .max_height
                .resolve_with(ctx.containing_block_height, vw, vh),
        )
    };

    clamp(value, min_constraint, max_constraint)
}

/// Clamps a value between optional minimum and maximum bounds.
fn clamp(value: f32, min: Option<f32>, max: Option<f32>) -> f32 {
    let v = min.map_or(value, |m| value.max(m));
    max.map_or(v, |m| v.min(m))
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

fn resolve_margins(spacing: &Spacing, ctx: &LayoutContext) -> EdgeOption {
    let containing_width = ctx.containing_block_width.unwrap_or(ctx.viewport_width);
    let vw = ctx.viewport_width;
    let vh = ctx.viewport_height;

    EdgeOption {
        left: spacing
            .margin_left
            .resolve_with(ctx.containing_block_width, vw, vh),
        top: spacing
            .margin_top
            .resolve_with(Some(containing_width), vw, vh),
        right: spacing
            .margin_right
            .resolve_with(ctx.containing_block_width, vw, vh),
        bottom: spacing
            .margin_bottom
            .resolve_with(Some(containing_width), vw, vh),
    }
}
