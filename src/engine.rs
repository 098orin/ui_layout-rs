use crate::{
    BoxModel, BoxSizing, FragmentNode, InlineBox, InnerDisplay, ItemFragment, LayoutBox,
    LayoutChild, LayoutNode, LineSpan, OuterDisplay, Placement, Rect, Spacing,
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

/// ((end_x, end_y), (current_x, prev_line_start_x), line_index)
///
/// (current_x, line_start_x) will be zero for non-inline contexts.
pub(crate) type LineContext = ((f32, f32), (f32, f32), usize);

pub(crate) const EMPTY_LINE_CONTEXT: LineContext = ((0.0, 0.0), (0.0, 0.0), 0);

impl LayoutEngine {
    // TODO: implemant parent_margin_end
    /// Main layout entry point.
    /// Initiates layout computation from the root node with specified viewport dimensions.
    pub fn layout(root: &mut LayoutNode, width: f32, height: f32) {
        let ctx = LayoutContext {
            containing_block_width: Some(width),
            containing_block_height: Some(height),
            available_width: Some(width),
            viewport_width: width,
            viewport_height: height,
        };

        let _ = Self::layout_node(root, &ctx, EMPTY_LINE_CONTEXT, false);
    }

    /// Internal method for layout a node.
    /// Layouts a single node and its descendants.
    #[must_use]
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

        let out = Self::layout_by_display(node, ctx, line_ctx, intrinsic_pass);

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
        let ((content_width_opt, content_height_opt), border, padding) =
            resolve_base_content_size_and_spacing(
                &node.style.size,
                &node.style.spacing,
                &node.style.box_sizing,
                ctx,
            );

        let content_width_opt = content_width_opt.or(ctx
            .available_width
            .map(|v| v - border.left - border.right - padding.left - padding.right));

        Self::layout_by_inner_display(
            node,
            ctx,
            line_ctx,
            (content_width_opt, content_height_opt),
            intrinsic_pass,
        )
    }

    fn layout_inline_level(
        node: &mut LayoutNode,
        ctx: &LayoutContext,
        line_ctx: LineContext,
        intrinsic_pass: bool,
    ) -> LineContext {
        let ((content_width_opt, content_height_opt), _, _) = resolve_base_content_size_and_spacing(
            &node.style.size,
            &node.style.spacing,
            &node.style.box_sizing,
            ctx,
        );

        Self::layout_by_inner_display(
            node,
            ctx,
            line_ctx,
            (content_width_opt, content_height_opt),
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
        content_size_opt: (Option<f32>, Option<f32>),
        intrinsic_pass: bool,
    ) -> LineContext {
        let ((end_x, end_y), (parent_current_x, mut line_start_x), mut line_index) = line_ctx;
        let mut current_x = 0.0;
        let (mut cursor_x, mut cursor_y) = (end_x, end_y);

        let (content_width_opt, content_height_opt) = content_size_opt;

        let border = resolve_border(&node.style.spacing, ctx);
        let padding = resolve_padding(&node.style.spacing, ctx);

        let base_ctx_for_child = LayoutContext {
            containing_block_width: content_width_opt,
            containing_block_height: content_height_opt,
            available_width: content_width_opt,
            ..*ctx
        };

        let mut line_span_buf = Vec::new();
        let line_height = node
            .style
            .line_height
            .resolve_with(None, ctx.viewport_width, ctx.viewport_height)
            .unwrap_or_default();
        let mut fragment_node_buffer = Vec::with_capacity(node.children.len());

        for child in &mut node.children {
            match child {
                LayoutChild::Fragment(fragment_node) => {
                    fragment_node_buffer.push(fragment_node);
                }
                LayoutChild::Node(child_node) => {
                    if !fragment_node_buffer.is_empty() {
                        let line_ctx_for_child =
                            ((cursor_x, cursor_y), (current_x, line_start_x), line_index);
                        let (line_spans, updated_line_ctx) = Self::flow_fragments(
                            &mut std::mem::take(&mut fragment_node_buffer),
                            line_ctx_for_child,
                            line_height,
                            content_width_opt.unwrap_or(ctx.viewport_width),
                        );
                        ((cursor_x, cursor_y), (current_x, line_start_x), line_index) =
                            updated_line_ctx;
                        line_span_buf.extend_from_slice(&line_spans);
                    }

                    let child_margin = resolve_margins(&child_node.style.spacing, ctx);

                    let ctx_for_child = LayoutContext {
                        available_width: content_width_opt.map(|v| {
                            v - child_margin.left.unwrap_or(0.0) - child_margin.right.unwrap_or(0.0)
                        }),
                        ..base_ctx_for_child
                    };

                    // Layout Node
                    let line_ctx_for_child =
                        ((cursor_x, cursor_y), (current_x, line_start_x), line_index);
                    ((cursor_x, cursor_y), (current_x, line_start_x), line_index) =
                        Self::layout_node(
                            child_node,
                            &ctx_for_child,
                            line_ctx_for_child,
                            intrinsic_pass,
                        );

                    // Process margin shift.
                    {
                        let EdgeOption {
                            left: ml_opt,
                            top,
                            right: mr_opt,
                            ..
                        } = child_margin;

                        let (ml, _mr) = if child_node.style.display.outer == OuterDisplay::Block {
                            let child_width = child_node.layout_box.width();
                            let (ml, mr) = match (ml_opt, mr_opt, content_width_opt) {
                                (None, None, Some(cw)) => {
                                    let auto_margin = (cw - child_width) / 2.0;
                                    (auto_margin, auto_margin)
                                }
                                (None, Some(mr), Some(cw)) => {
                                    let auto_margin = cw - child_width - mr;
                                    (auto_margin, mr)
                                }
                                (Some(ml), None, Some(cw)) => {
                                    let auto_margin = cw - child_width - ml;
                                    (ml, auto_margin)
                                }
                                _ => (ml_opt.unwrap_or(0.0), mr_opt.unwrap_or(0.0)),
                            };

                            (ml, mr)
                        } else {
                            (ml_opt.unwrap_or(0.0), mr_opt.unwrap_or(0.0))
                        };

                        if child_node.style.display.outer == OuterDisplay::Inline {
                            child_node.layout_box.shift(ml, 0.0);
                        } else {
                            child_node.layout_box.shift(ml, top.unwrap_or(0.0));
                        }
                    }

                    // Collect child's line_spans if the outer display is Inline.
                    if child_node.style.display.outer == OuterDisplay::Inline {}
                }
            }
        }

        if !fragment_node_buffer.is_empty() {
            let line_ctx_for_child = ((cursor_x, cursor_y), (current_x, line_start_x), line_index);
            let (line_spans, updated_line_ctx) = Self::flow_fragments(
                &mut std::mem::take(&mut fragment_node_buffer),
                line_ctx_for_child,
                line_height,
                content_width_opt.unwrap_or(ctx.viewport_width),
            );
            ((cursor_x, cursor_y), (current_x, line_start_x), line_index) = updated_line_ctx;
            line_span_buf.extend_from_slice(&line_spans);
        }

        if node.style.display.outer == OuterDisplay::Inline {
            let box_model = create_box_model(
                current_x,
                line_height,
                current_x,
                line_height,
                padding,
                border,
            );
            let inline_box = InlineBox {
                box_model,
                line_spans: line_span_buf,
            };
            node.layout_box = LayoutBox::InlineBox(inline_box);
        } else {
            current_x = line_ctx.1.0;

            todo!()
        }

        (
            (cursor_x, cursor_y),
            (parent_current_x + current_x, line_start_x),
            line_index,
        )
    }

    fn flow_fragments(
        fragments: &mut Vec<&mut FragmentNode>,
        line_ctx: LineContext,
        line_height: f32,
        outbox_width: f32,
    ) -> (Vec<LineSpan>, LineContext) {
        let ((end_x, end_y), (mut current_x, mut line_start_x), mut line_index) = line_ctx;
        let (mut cursor_x, mut cursor_y) = (end_x, end_y);

        let mut if_first_of_line = current_x == line_start_x;

        let mut line_span_buf = Vec::new();

        for fragment_node in fragments {
            match fragment_node.node {
                ItemFragment::LineBreak => {
                    line_span_buf.push(LineSpan {
                        x_range: (line_start_x)..(current_x),
                        line_pos: (line_start_x, cursor_y),
                        line_index,
                    });

                    fragment_node.placement = Placement {
                        offset: (cursor_x, cursor_y),
                        line_index,
                    };

                    cursor_x = 0.0;
                    cursor_y += line_height;
                    line_index += 1;
                    line_start_x = 0.0;
                    if_first_of_line = true;
                }
                ItemFragment::Fragment(fragment_item) => {
                    if cursor_x + fragment_item.width > outbox_width && !if_first_of_line {
                        line_span_buf.push(LineSpan {
                            x_range: (line_start_x)..(current_x),
                            line_pos: (line_start_x, cursor_y),
                            line_index,
                        });

                        fragment_node.placement = Placement {
                            offset: (cursor_x, cursor_y),
                            line_index,
                        };

                        cursor_x = 0.0;
                        cursor_y += line_height;
                        line_index += 1;
                        line_start_x = 0.0;
                        if_first_of_line = true;
                    } else {
                        if_first_of_line = false;
                    }

                    fragment_node.placement = Placement {
                        offset: (cursor_x, cursor_y),
                        line_index,
                    };

                    cursor_x += fragment_item.width;
                    current_x += fragment_item.width;
                }
            }
        }

        if !if_first_of_line {
            line_span_buf.push(LineSpan {
                x_range: (line_start_x)..(current_x),
                line_pos: (line_start_x, cursor_y),
                line_index,
            });
        }

        (
            line_span_buf,
            ((cursor_x, cursor_y), (current_x, line_start_x), line_index),
        )
    }
}

// ==========================================

/// ((content_width_opt, content_height_opt), border, padding)
fn resolve_base_content_size_and_spacing(
    size_style: &crate::SizeStyle,
    spacing: &crate::Spacing,
    box_sizing: &BoxSizing,
    ctx: &LayoutContext,
) -> ((Option<f32>, Option<f32>), Edge, Edge) {
    let border = resolve_border(spacing, ctx);
    let padding = resolve_padding(spacing, ctx);

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

/// Creates a box model with specified dimensions and spacing.
fn create_box_model(
    content_width: f32,
    content_height: f32,
    children_width: f32,
    children_height: f32,
    padding_edge: Edge,
    border_edge: Edge,
) -> BoxModel {
    let Edge {
        left: pl,
        top: pt,
        right: pr,
        bottom: pb,
    } = padding_edge;
    let Edge {
        left: bl,
        top: bt,
        right: br,
        bottom: bb,
    } = border_edge;

    let border_box = Rect {
        x: 0.0,
        y: 0.0,
        width: content_width + pl + pr + bl + br,
        height: content_height + pt + pb + bt + bb,
    };

    let padding_box = Rect {
        x: bl,
        y: bt,
        width: content_width + pl + pr,
        height: content_height + pt + pb,
    };

    let content_box = Rect {
        x: bl + pl,
        y: bt + pt,
        width: content_width,
        height: content_height,
    };

    let children_box = Rect {
        width: children_width,
        height: children_height,
        ..content_box
    };

    BoxModel {
        content_box,
        padding_box,
        border_box,
        children_box,
    }
}
