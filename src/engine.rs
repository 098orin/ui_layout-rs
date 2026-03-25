// Layout engine
// -----------------------
// This module implements the layout algorithm for `LayoutNode` trees.
// It computes box-models (content / padding / border / children boxes)
// and positions children for block, inline and flex layout modes.

use crate::{
    AlignItems, BoxModel, BoxSizing, Display, FlexDirection, FragmentPlacement, JustifyContent,
    LayoutBoxes, LayoutNode, Length, Rect, Spacing, Style,
};

/// Container dimensions for flex layout.
///
/// Fields:
/// - Main-axis content size (None if auto)
/// - Cross-axis content size (None if auto)
/// - Padding edges (start, before, end, after)
/// - Border edges (start, before, end, after)
type ContainerSizes = (Option<f32>, Option<f32>, Edge, Edge);

#[derive(Clone, Copy, Default)]
struct Edge {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl From<(f32, f32, f32, f32)> for Edge {
    /// Arg valus: (f32, f32, f32, f32) should be:
    /// (left, top, right, bottom)
    fn from(value: (f32, f32, f32, f32)) -> Self {
        let (left, top, right, bottom) = value;
        Self {
            left,
            top,
            right,
            bottom,
        }
    }
}

/// Layout context carrying resolved sizing information down the tree.
///
/// Fields:
/// - `containing_block_*`: Size of the containing block (None if auto)
/// - `available_block_*`: Size of the available space within the containing block (None if auto)
/// - `viewport_*`: Absolute viewport dimensions for percentage resolution
/// - `parent_assigned_border_*`: Border-box sizes assigned by parent (for stretch)
#[allow(dead_code)]
pub(crate) struct LayoutContext {
    pub(crate) containing_block_width: Option<f32>,
    pub(crate) containing_block_height: Option<f32>,
    pub(crate) available_block_width: Option<f32>,
    pub(crate) available_block_height: Option<f32>,
    pub(crate) viewport_width: f32,
    pub(crate) viewport_height: f32,
    pub(crate) parent_assigned_border_width: Option<f32>,
    pub(crate) parent_assigned_border_height: Option<f32>,
}

impl LayoutContext {
    /// Returns the containing-block size along the main axis.
    fn containing_block_main(&self, axis: Axis) -> Option<f32> {
        match axis {
            Axis::Horizontal => self.containing_block_width,
            Axis::Vertical => self.containing_block_height,
        }
    }

    /// Returns the containing-block size along the cross axis.
    fn containing_block_cross(&self, axis: Axis) -> Option<f32> {
        match axis {
            Axis::Horizontal => self.containing_block_height,
            Axis::Vertical => self.containing_block_width,
        }
    }

    /// Returns parent-assigned border-box size along the main axis (for stretch/relative fallback).
    fn parent_assigned_border_main(&self, axis: Axis) -> Option<f32> {
        match axis {
            Axis::Horizontal => self.parent_assigned_border_width,
            Axis::Vertical => self.parent_assigned_border_height,
        }
    }

    /// Returns parent-assigned border-box size along the cross axis.
    fn parent_assigned_border_cross(&self, axis: Axis) -> Option<f32> {
        match axis {
            Axis::Horizontal => self.parent_assigned_border_height,
            Axis::Vertical => self.parent_assigned_border_width,
        }
    }
}

/// Axis orientation for flex and flow layout.
///
/// Provides helper methods to abstract width/height selection, reducing code duplication
/// for row and column layout support.
#[derive(Debug, Clone, Copy)]
enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    fn main(&self, rect: &Rect) -> f32 {
        match self {
            Axis::Horizontal => rect.width,
            Axis::Vertical => rect.height,
        }
    }

    fn cross(&self, rect: &Rect) -> f32 {
        match self {
            Axis::Horizontal => rect.height,
            Axis::Vertical => rect.width,
        }
    }

    fn size_main<'a>(&self, size: &'a crate::SizeStyle) -> &'a Length {
        match self {
            Axis::Horizontal => &size.width,
            Axis::Vertical => &size.height,
        }
    }

    fn size_cross<'a>(&self, size: &'a crate::SizeStyle) -> &'a Length {
        match self {
            Axis::Horizontal => &size.height,
            Axis::Vertical => &size.width,
        }
    }

    fn min_main<'a>(&self, size: &'a crate::SizeStyle) -> &'a Length {
        match self {
            Axis::Horizontal => &size.min_width,
            Axis::Vertical => &size.min_height,
        }
    }

    fn max_main<'a>(&self, size: &'a crate::SizeStyle) -> &'a Length {
        match self {
            Axis::Horizontal => &size.max_width,
            Axis::Vertical => &size.max_height,
        }
    }

    fn min_cross<'a>(&self, size: &'a crate::SizeStyle) -> &'a Length {
        match self {
            Axis::Horizontal => &size.min_height,
            Axis::Vertical => &size.min_width,
        }
    }

    fn max_cross<'a>(&self, size: &'a crate::SizeStyle) -> &'a Length {
        match self {
            Axis::Horizontal => &size.max_height,
            Axis::Vertical => &size.max_width,
        }
    }

    fn padding_main<'a>(&self, spacing: &'a Spacing) -> (&'a Length, &'a Length) {
        match self {
            Axis::Horizontal => (&spacing.padding_left, &spacing.padding_right),
            Axis::Vertical => (&spacing.padding_top, &spacing.padding_bottom),
        }
    }

    fn padding_cross<'a>(&self, spacing: &'a Spacing) -> (&'a Length, &'a Length) {
        match self {
            Axis::Horizontal => (&spacing.padding_top, &spacing.padding_bottom),
            Axis::Vertical => (&spacing.padding_left, &spacing.padding_right),
        }
    }

    fn border_main<'a>(&self, spacing: &'a Spacing) -> (&'a Length, &'a Length) {
        match self {
            Axis::Horizontal => (&spacing.border_left, &spacing.border_right),
            Axis::Vertical => (&spacing.border_top, &spacing.border_bottom),
        }
    }

    fn border_cross<'a>(&self, spacing: &'a Spacing) -> (&'a Length, &'a Length) {
        match self {
            Axis::Horizontal => (&spacing.border_top, &spacing.border_bottom),
            Axis::Vertical => (&spacing.border_left, &spacing.border_right),
        }
    }

    fn margin_main_start<'a>(&self, s: &'a Spacing) -> &'a Length {
        match self {
            Axis::Horizontal => &s.margin_left,
            Axis::Vertical => &s.margin_top,
        }
    }

    fn margin_main_end<'a>(&self, s: &'a Spacing) -> &'a Length {
        match self {
            Axis::Horizontal => &s.margin_right,
            Axis::Vertical => &s.margin_bottom,
        }
    }

    fn margin_cross_start<'a>(&self, s: &'a Spacing) -> &'a Length {
        match self {
            Axis::Horizontal => &s.margin_top,
            Axis::Vertical => &s.margin_left,
        }
    }

    fn margin_cross_end<'a>(&self, s: &'a Spacing) -> &'a Length {
        match self {
            Axis::Horizontal => &s.margin_bottom,
            Axis::Vertical => &s.margin_right,
        }
    }

    fn gap<'a>(&self, style: &'a Style) -> &'a Length {
        match self {
            Axis::Horizontal => &style.column_gap,
            Axis::Vertical => &style.row_gap,
        }
    }
}

pub struct LayoutEngine;

impl LayoutEngine {
    /// Main layout entry point.
    /// Initiates layout computation from the root node with specified viewport dimensions.
    ///
    /// # TODO
    /// - Implement CSS-correct flex-basis precedence.
    ///   - flex-basis should override width/height on the main axis,
    ///     but the current algorithm does not guarantee this.
    pub fn layout(root: &mut LayoutNode, width: f32, height: f32) {
        let ctx = LayoutContext {
            containing_block_width: Some(width),
            containing_block_height: Some(height),
            available_block_width: Some(width),
            available_block_height: Some(height),
            viewport_width: width,
            viewport_height: height,
            // Root has no parent; assign viewport dimensions for layout calculations
            parent_assigned_border_width: Some(width),
            parent_assigned_border_height: Some(height),
        };

        let engine = LayoutEngine;
        engine.layout_node(root, false, (0.0, 0.0), Edge::default(), 0.0, &ctx);
    }

    /// Layouts a single node and its descendants.
    /// Returns: ((end_x, end_y), height)
    fn layout_node(
        &self,
        node: &mut LayoutNode,
        intrinsic_pass: bool,
        origin: (f32, f32),
        resolved_margin_edge: Edge,
        incoming_line_height: f32,
        ctx: &LayoutContext,
    ) -> ((f32, f32), f32) {
        if intrinsic_pass {
            let (key, (layout_boxes, out)) = &node.layout_boxes_cache;
            if *key == crate::cache::make_layout_key(ctx) {
                node.layout_boxes = layout_boxes.clone();
                return *out;
            }
        }

        let out = match node.style.display {
            Display::None => {
                node.layout_boxes = LayoutBoxes::None;
                (origin, 0.0)
            }
            Display::Block | Display::Inline | Display::Flex { .. } => self.layout_unified_flow(
                node,
                intrinsic_pass,
                origin,
                resolved_margin_edge,
                incoming_line_height,
                ctx,
            ),
        };

        if intrinsic_pass {
            let key = crate::cache::make_layout_key(ctx);
            node.layout_boxes_cache = (key, (node.layout_boxes.clone(), out));
        }

        out
    }

    /// Unified flow layout handling block, inline, and flex displays.
    /// Dispatches to the appropriate layout algorithm based on display type.
    fn layout_unified_flow(
        &self,
        node: &mut LayoutNode,
        intrinsic_pass: bool,
        origin: (f32, f32),
        resolved_margin_edge: Edge,
        incoming_line_height: f32,
        ctx: &LayoutContext,
    ) -> ((f32, f32), f32) {
        match node.style.display {
            Display::Flex { flex_direction } => {
                let axis = match flex_direction {
                    FlexDirection::Row => Axis::Horizontal,
                    FlexDirection::Column => Axis::Vertical,
                };
                self.layout_flex_as_flow(
                    node,
                    axis,
                    intrinsic_pass,
                    origin,
                    incoming_line_height,
                    ctx,
                )
            }
            Display::Block => {
                self.layout_block_flow(node, intrinsic_pass, origin, incoming_line_height, ctx)
            }
            Display::Inline => self.layout_inline_flow(
                node,
                intrinsic_pass,
                origin,
                resolved_margin_edge,
                incoming_line_height,
                ctx,
            ),
            Display::None => unreachable!(),
        }
    }

    /// Layouts a flex container.
    ///
    /// Process:
    /// 1. Determine flex container size
    /// 2. Layout flex children with flexible lengths
    /// 3. Apply cross-axis alignment
    /// 4. Position children with justify-content
    fn layout_flex_as_flow(
        &self,
        node: &mut LayoutNode,
        axis: Axis,
        intrinsic_pass: bool,
        origin: (f32, f32),
        _incoming_line_height: f32,
        ctx: &LayoutContext,
    ) -> ((f32, f32), f32) {
        // Step 1: Determine Flex container size
        let (content_main, content_cross, padding, border) =
            self.resolve_container_sizes(node, axis, ctx);

        // Step 2: Execute layout for child elements
        let (mut children_main, mut children_cross) =
            if !intrinsic_pass || content_main.is_none() || content_cross.is_none() {
                let (content_width, content_height) = match axis {
                    Axis::Horizontal => (content_main, content_cross),
                    Axis::Vertical => (content_cross, content_main),
                };
                let children_ctx = LayoutContext {
                    containing_block_width: content_width,
                    containing_block_height: content_height,
                    available_block_width: None,
                    available_block_height: None,
                    viewport_width: ctx.viewport_width,
                    viewport_height: ctx.viewport_height,
                    parent_assigned_border_width: None,
                    parent_assigned_border_height: None,
                };
                self.layout_flex_children(node, axis, intrinsic_pass, &children_ctx)
            } else {
                (0.0, 0.0)
            };

        // Step 3: Determine final container size
        let mut final_content_main = content_main.unwrap_or(children_main);
        let mut final_content_cross = content_cross.unwrap_or(children_cross);

        // Step 3a: Apply min/max constraints to auto-sized flex container dimensions
        let cbm = ctx.containing_block_main(axis);
        let cbc = ctx.containing_block_cross(axis);
        let vw = ctx.viewport_width;
        let vh = ctx.viewport_height;

        let main_size_was_auto = content_main.is_none();
        let cross_size_was_auto = content_cross.is_none();

        if main_size_was_auto {
            let min_main = axis.min_main(&node.style.size).resolve_with(cbm, vw, vh);
            let max_main = axis.max_main(&node.style.size).resolve_with(cbm, vw, vh);
            final_content_main = clamp(final_content_main, min_main, max_main);
        }

        if cross_size_was_auto {
            let min_cross = axis.min_cross(&node.style.size).resolve_with(cbc, vw, vh);
            let max_cross = axis.max_cross(&node.style.size).resolve_with(cbc, vw, vh);
            final_content_cross = clamp(final_content_cross, min_cross, max_cross);
        }

        // Step 3b: Re-layout children if main size changed due to min/max constraints
        if !intrinsic_pass && main_size_was_auto && final_content_main != children_main {
            let child_ctx = LayoutContext {
                containing_block_width: Some(final_content_main),
                containing_block_height: Some(final_content_cross),
                available_block_width: None,
                available_block_height: None,
                parent_assigned_border_width: None,
                parent_assigned_border_height: None,
                ..*ctx
            };
            (children_main, children_cross) =
                self.layout_flex_children(node, axis, false, &child_ctx);
        }

        // Step 4: Create box model
        node.layout_boxes = {
            let (content_width, content_height) = match axis {
                Axis::Horizontal => (final_content_main, final_content_cross),
                Axis::Vertical => (final_content_cross, final_content_main),
            };
            let (children_width, children_height) = match axis {
                Axis::Horizontal => (children_main, children_cross),
                Axis::Vertical => (children_cross, children_main),
            };
            LayoutBoxes::Single(create_box_model(
                content_width,
                content_height,
                children_width,
                children_height,
                padding,
                border,
            ))
        };

        if let LayoutBoxes::Single(ref mut box_model) = node.layout_boxes {
            let Edge {
                left: pl, top: pt, ..
            } = padding;
            let Edge {
                left: bl, top: bt, ..
            } = border;
            set_position(box_model, origin, (pl, pt), (bl, bt));
        }

        // Step 5: Set child positions (positioning logic inlined)
        if !intrinsic_pass {
            if node.children.is_empty() {
                // nothing
            } else {
                let content_box = match &node.layout_boxes {
                    LayoutBoxes::Single(box_model) => &box_model.content_box,
                    _ => {
                        // fallback: nothing to position
                        return (
                            (origin.0 + node.layout_boxes.width(), origin.1),
                            node.layout_boxes.height(),
                        );
                    }
                };

                // Resolve gap between flex items
                let vw = ctx.viewport_width;
                let vh = ctx.viewport_height;
                let gap = axis
                    .gap(&node.style)
                    .resolve_with(ctx.containing_block_main(axis), vw, vh)
                    .unwrap_or(0.0);

                // Calculate total size of all flex children
                let children_main_total: f32 = node
                    .children
                    .iter()
                    .map(|child| {
                        if let LayoutBoxes::Single(ref box_model) = child.layout_boxes {
                            axis.main(&box_model.border_box)
                        } else {
                            0.0
                        }
                    })
                    .sum();

                // Calculate total gap between items
                let gaps_total = if node.children.len() > 1 {
                    gap * (node.children.len() as f32 - 1.0)
                } else {
                    0.0
                };

                // Calculate remaining space for justify-content distribution
                let remaining_space = axis.main(content_box) - children_main_total - gaps_total;

                // Check if any child has auto margins on main axis
                let mut auto_margin_count = 0u32;

                for child in &node.children {
                    match axis {
                        Axis::Horizontal => {
                            if child.style.spacing.margin_left == Length::Auto {
                                auto_margin_count += 1;
                            }
                            if child.style.spacing.margin_right == Length::Auto {
                                auto_margin_count += 1;
                            }
                        }
                        Axis::Vertical => {
                            if child.style.spacing.margin_top == Length::Auto {
                                auto_margin_count += 1;
                            }
                            if child.style.spacing.margin_bottom == Length::Auto {
                                auto_margin_count += 1;
                            }
                        }
                    }
                }

                let has_auto_margins = auto_margin_count > 0;

                // auto margin がある場合のみ負値を 0 に丸める
                let remaining_space_for_auto = if has_auto_margins {
                    remaining_space.max(0.0)
                } else {
                    remaining_space
                };

                let auto_unit = if has_auto_margins && auto_margin_count > 0 {
                    remaining_space_for_auto / auto_margin_count as f32
                } else {
                    0.0
                };

                // Auto margins take precedence over justify-content
                let (start_offset, gap_between) = if has_auto_margins {
                    (0.0, 0.0)
                } else {
                    resolve_justify_content(
                        node.style.justify_content,
                        remaining_space.max(0.0),
                        node.children.len(),
                    )
                };

                // Position each flex child
                let mut cursor_main = start_offset;

                for child in &mut node.children {
                    // Detect auto margins on main axis
                    let (margin_start_auto, margin_end_auto) = match axis {
                        Axis::Horizontal => (
                            child.style.spacing.margin_left == Length::Auto,
                            child.style.spacing.margin_right == Length::Auto,
                        ),
                        Axis::Vertical => (
                            child.style.spacing.margin_top == Length::Auto,
                            child.style.spacing.margin_bottom == Length::Auto,
                        ),
                    };

                    // Compute auto margin widths
                    let mut margin_start = 0.0;
                    let mut margin_end = 0.0;

                    if margin_start_auto {
                        margin_start = auto_unit;
                    }
                    if margin_end_auto {
                        margin_end = auto_unit;
                    }

                    cursor_main += margin_start;

                    // Position child along main axis
                    let child_main_pos = match axis {
                        Axis::Horizontal => content_box.x + cursor_main,
                        Axis::Vertical => content_box.y + cursor_main,
                    };

                    // Position child along cross axis (align-items / align-self)
                    let child_cross_size =
                        if let LayoutBoxes::Single(ref box_model) = child.layout_boxes {
                            axis.cross(&box_model.border_box)
                        } else {
                            0.0
                        };
                    let available_cross = axis.cross(content_box);

                    // --- Cross-axis auto margin handling ---

                    let (margin_cross_start_auto, margin_cross_end_auto) = match axis {
                        Axis::Horizontal => (
                            child.style.spacing.margin_top == Length::Auto,
                            child.style.spacing.margin_bottom == Length::Auto,
                        ),
                        Axis::Vertical => (
                            child.style.spacing.margin_left == Length::Auto,
                            child.style.spacing.margin_right == Length::Auto,
                        ),
                    };

                    let cross_offset;

                    if margin_cross_start_auto || margin_cross_end_auto {
                        let free_cross_space = (available_cross - child_cross_size).max(0.0);

                        if margin_cross_start_auto && margin_cross_end_auto {
                            margin_start = free_cross_space / 2.0;
                        } else if margin_cross_start_auto {
                            margin_start = free_cross_space;
                        } else {
                            margin_start = 0.0;
                        }

                        cross_offset = margin_start;
                    } else {
                        // fallback to align-self / align-items
                        cross_offset = resolve_align_position(
                            child
                                .style
                                .item_style
                                .align_self
                                .unwrap_or(node.style.align_items),
                            child_cross_size,
                            available_cross,
                        );
                    }

                    let child_cross_pos = match axis {
                        Axis::Horizontal => content_box.y + cross_offset,
                        Axis::Vertical => content_box.x + cross_offset,
                    };

                    // Compute final child position based on axis orientation
                    let child_origin = match axis {
                        Axis::Horizontal => (child_main_pos, child_cross_pos),
                        Axis::Vertical => (child_cross_pos, child_main_pos),
                    };

                    // Shift child to final position relative to parent content box
                    let relative_x = child_origin.0 - content_box.x;
                    let relative_y = child_origin.1 - content_box.y;
                    child.layout_boxes.shift(relative_x, relative_y);

                    // Move cursor forward for next child
                    let child_main_size =
                        if let LayoutBoxes::Single(ref box_model) = child.layout_boxes {
                            axis.main(&box_model.border_box)
                        } else {
                            0.0
                        };
                    cursor_main += child_main_size + margin_end + gap + gap_between;
                }
            }
        }

        let end_x = origin.0 + node.layout_boxes.width();
        let end_y = origin.1 + node.layout_boxes.height();
        ((end_x, end_y), 0.0)
    }

    /// Layouts a block container.
    ///
    /// Process:
    /// 1. Resolve element's own size and spacing
    /// 2. Layout children
    /// 3. Create box model
    fn layout_block_flow(
        &self,
        node: &mut LayoutNode,
        intrinsic_pass: bool,
        origin: (f32, f32),
        _incoming_line_height: f32,
        ctx: &LayoutContext,
    ) -> ((f32, f32), f32) {
        let padding = resolve_padding(&node.style.spacing, ctx);
        let border = resolve_border(&node.style.spacing, ctx);

        // Step 1: Resolve the block's own size
        let vw = ctx.viewport_width;
        let vh = ctx.viewport_height;

        let content_width_opt = node
            .style
            .size
            .width
            .resolve_with(ctx.containing_block_width, vw, vh)
            .map(|width| {
                let padding_edge = (padding.left, padding.right);
                let border_edge = (border.left, border.right);
                resolve_content_size_with_box_sizing(node, width, padding_edge, border_edge)
            })
            .or(ctx
                .parent_assigned_border_width
                .map(|b| b - border.right - border.left - padding.right - padding.left))
            .or(ctx
                .available_block_width
                .map(|cbw| cbw - border.right - border.left - padding.right - padding.left));

        let content_height_opt = node
            .style
            .size
            .height
            .resolve_with(ctx.containing_block_height, vw, vh)
            .map(|height| {
                let padding_edge = (padding.top, padding.bottom);
                let border_edge = (border.top, border.bottom);
                resolve_content_size_with_box_sizing(node, height, padding_edge, border_edge)
            })
            .or(ctx
                .parent_assigned_border_height
                .map(|b| b - border.top - border.bottom - padding.top - padding.bottom));

        // Step 2: Layout children
        let skip_children_layout =
            intrinsic_pass && content_width_opt.is_some() && content_height_opt.is_some();
        let mut children_width = 0.0;
        let mut children_height = 0.0;

        if !skip_children_layout {
            let child_ctx = LayoutContext {
                containing_block_width: content_width_opt,
                containing_block_height: content_height_opt,
                available_block_width: None,
                available_block_height: None,
                parent_assigned_border_width: None,
                parent_assigned_border_height: None,
                ..*ctx
            };

            let mut cursor_y = 0.0;
            let mut cursor_x = 0.0;

            let mut max_width: f32 = 0.0;
            let mut previous_margin_bottom: f32 = 0.0;
            let mut incoming_line_height = 0.0;

            // Layout each child and position vertically with collapsing margins
            for child in node.children.iter_mut() {
                let is_block_level =
                    matches!(child.style.display, Display::Block | Display::Flex { .. });

                // Resolve margins for child to calculate available space and position
                let Edge {
                    top: mt,
                    bottom: mb,
                    ..
                } = resolve_margins(&child.style.spacing, &child_ctx);

                let (ml_opt, mr_opt) = (
                    child
                        .style
                        .spacing
                        .margin_left
                        .resolve_with(content_width_opt, vw, vh),
                    child
                        .style
                        .spacing
                        .margin_right
                        .resolve_with(content_width_opt, vw, vh),
                );

                // Check if either horizontal margin is auto for block-level children
                let has_side_auto_margin = ml_opt.is_none() || mr_opt.is_none();

                // Culculate available width for child content (auto margins reduce available space)
                let child_availavle_width = if has_side_auto_margin {
                    None
                } else {
                    content_width_opt.map(|w| w - ml_opt.unwrap() - mr_opt.unwrap())
                };

                let child_available_height = content_height_opt.map(|h| h - mt - mb);

                // Create child layout context with available space
                let child_ctx = LayoutContext {
                    available_block_width: child_availavle_width,
                    available_block_height: child_available_height,
                    ..child_ctx
                };

                if is_block_level {
                    cursor_y += incoming_line_height;
                    incoming_line_height = 0.0;

                    cursor_x = 0.0;
                    cursor_y += mt.max(previous_margin_bottom);
                    previous_margin_bottom = mb;
                } else {
                    // For inline children, margins do not collapse and are applied directly
                    cursor_x += ml_opt.unwrap_or(0.0);

                    previous_margin_bottom = 0.0;
                }

                ((cursor_x, cursor_y), incoming_line_height) = {
                    let resolved_margin_edge = Edge {
                        left: ml_opt.unwrap_or(0.0),
                        top: mt,
                        right: mr_opt.unwrap_or(0.0),
                        bottom: mb,
                    };
                    self.layout_node(
                        child,
                        intrinsic_pass,
                        (cursor_x, cursor_y),
                        resolved_margin_edge,
                        incoming_line_height,
                        &child_ctx,
                    )
                };

                if is_block_level {
                    // Resolve auto margins for block-level children (horizontal auto margins are ignored for blocks)
                    // and shift child position accordingly
                    let (ml, mr) = match (ml_opt, mr_opt, content_width_opt) {
                        (None, None, Some(cw)) => {
                            let auto_margin = (cw - child.layout_boxes.width()) / 2.0;
                            (auto_margin, auto_margin)
                        }
                        (None, Some(mr), Some(cw)) => {
                            let auto_margin = cw - child.layout_boxes.width() - mr;
                            (auto_margin, mr)
                        }
                        (Some(ml), None, Some(cw)) => {
                            let auto_margin = cw - child.layout_boxes.width() - ml;
                            (ml, auto_margin)
                        }
                        _ => (ml_opt.unwrap_or(0.0), mr_opt.unwrap_or(0.0)),
                    };

                    child.layout_boxes.shift(ml, 0.0);

                    // Update max width for block-level children
                    let child_total_width = child.layout_boxes.width() + ml + mr;
                    max_width = max_width.max(child_total_width);
                }
            }
            cursor_y += previous_margin_bottom;
            cursor_y += incoming_line_height;

            children_width = max_width;
            children_height = cursor_y;
        }

        let content_width = {
            let content_width = content_width_opt.unwrap_or(children_width);
            apply_size_constraints(content_width, &node.style.size, ctx, true)
        };

        let content_height = {
            let content_height = content_height_opt.unwrap_or(children_height);
            apply_size_constraints(content_height, &node.style.size, ctx, false)
        };

        // Step 3: Create box model
        let mut box_model = create_box_model(
            content_width,
            content_height,
            children_width,
            children_height,
            padding,
            border,
        );

        let padding_edge = (padding.right, padding.top);
        let border_edge = (border.right, border.top);

        set_position(&mut box_model, origin, padding_edge, border_edge);

        node.layout_boxes = LayoutBoxes::Single(box_model);

        (((0.0), (origin.1 + node.layout_boxes.height())), (0.0))
    }

    /// Layouts an inline container as flow layout.
    /// Handles inline content (text runs and inline elements).
    fn layout_inline_flow(
        &self,
        node: &mut LayoutNode,
        intrinsic_pass: bool,
        origin: (f32, f32),
        resolved_margin_edge: Edge,
        incoming_line_height: f32,
        ctx: &LayoutContext,
    ) -> ((f32, f32), f32) {
        let container_width = ctx.containing_block_width.unwrap_or(ctx.viewport_width);

        // Resolve inline container's spacing
        let ctx_for_inline = LayoutContext {
            containing_block_width: ctx.containing_block_width,
            containing_block_height: ctx.containing_block_height,
            ..*ctx
        };
        let padding = resolve_padding(&node.style.spacing, &ctx_for_inline);
        let border = resolve_border(&node.style.spacing, &ctx_for_inline);

        if !intrinsic_pass {
            node.placements.clear();
        }

        // Layout inline fragments (text runs and inline content)
        if !node.self_fragments.is_empty() {
            // Line Break
            fn do_line_break(
                cursor_x: &mut f32,
                cursor_y: &mut f32,
                line_height: &mut f32,
                line_index: &mut usize,
                first_margin: f32,
                padding_edge: Edge,
                border_edge: Edge,
            ) -> BoxModel {
                let width = if *line_index == 0 {
                    *cursor_x - first_margin
                } else {
                    *cursor_x
                };

                let box_model = create_box_model(
                    width,
                    *line_height,
                    width,
                    *line_height,
                    padding_edge,
                    border_edge,
                );

                *cursor_x = 0.0;
                *cursor_y += *line_height;
                *line_height = 0.0;
                *line_index += 1;

                box_model
            }

            // Place
            #[must_use]
            #[allow(clippy::too_many_arguments)]
            fn place_fragment(
                cursor_x: f32,
                cursor_y: f32,
                line_index: usize,
                first_margin: f32,
            ) -> FragmentPlacement {
                let (mut pos_x, pos_y) = (cursor_x, cursor_y);

                if line_index == 0 {
                    pos_x -= first_margin;
                }

                FragmentPlacement {
                    offset: (pos_x, pos_y),
                    line_index,
                }
            }

            let max_wrap_width =
                container_width - border.left - border.right - padding.left - padding.right;

            let mut cursor_x = 0.0;
            let mut cursor_y = 0.0;
            let mut line_height = incoming_line_height;
            let mut line_index = 0;

            let mut layout_boxes_buf = if intrinsic_pass {
                Vec::new()
            } else {
                Vec::with_capacity(node.self_fragments.len())
            };

            let padding_edge = resolve_padding(&node.style.spacing, ctx);
            let border_edge = resolve_border(&node.style.spacing, ctx);
            let Edge {
                left: first_margin,
                right: last_margin,
                ..
            } = resolved_margin_edge;

            cursor_x += first_margin;

            // Shift layout boxes at the end.
            for frag in &node.self_fragments {
                match frag {
                    crate::ItemFragment::LineBreak => {
                        let box_model = do_line_break(
                            &mut cursor_x,
                            &mut cursor_y,
                            &mut line_height,
                            &mut line_index,
                            first_margin,
                            padding_edge,
                            border_edge,
                        );

                        layout_boxes_buf.push(box_model);
                        if !intrinsic_pass {
                            let fragment_placement =
                                place_fragment(cursor_x, cursor_y, line_index, first_margin);

                            node.placements.push(fragment_placement);
                        }
                    }

                    crate::ItemFragment::Fragment(f) => {
                        if cursor_x + f.width > max_wrap_width && cursor_x != 0.0 {
                            let box_model = do_line_break(
                                &mut cursor_x,
                                &mut cursor_y,
                                &mut line_height,
                                &mut line_index,
                                first_margin,
                                padding_edge,
                                border_edge,
                            );

                            layout_boxes_buf.push(box_model);
                        }

                        if !intrinsic_pass {
                            let fragment_placement =
                                place_fragment(cursor_x, cursor_y, line_index, first_margin);

                            node.placements.push(fragment_placement);
                        }

                        cursor_x += f.width;
                        line_height = line_height.max(f.height);
                    }
                }
            }

            if cursor_x != 0.0 {
                let box_model = do_line_break(
                    &mut cursor_x,
                    &mut cursor_y,
                    &mut line_height,
                    &mut line_index,
                    first_margin,
                    padding_edge,
                    border_edge,
                );

                layout_boxes_buf.push(box_model);
            }

            // Create box model with spacing
            node.layout_boxes = LayoutBoxes::Multiple(layout_boxes_buf);

            // The parent has already calculated the margin,
            // so we need to avoid calculating it again.
            node.layout_boxes.shift(origin.0 - first_margin, origin.1);

            return ((cursor_x, cursor_y), line_height);
        }

        // self_fragment.is_empty && children exist
        // -> Inline container. Layout children in flow layout and size container to fit children.
        if !node.children.is_empty() {
            let mut cursor_x = origin.0 + border.right + padding.right;
            let mut cursor_y = origin.1 + border.top + padding.top;
            let mut incoming_line_height = incoming_line_height.max(0.0);
            for child in node.children.iter_mut() {
                let child_resolved_margin_edge = resolve_margins(&child.style.spacing, ctx);
                ((cursor_x, cursor_y), incoming_line_height) = self.layout_node(
                    child,
                    intrinsic_pass,
                    (cursor_x, cursor_y),
                    child_resolved_margin_edge,
                    incoming_line_height,
                    ctx,
                );
            }
        }
        let content_width = 0.0;
        let content_height = 0.0;

        node.layout_boxes = LayoutBoxes::Single(create_box_model(
            content_width,
            content_height,
            content_width,
            content_height,
            padding,
            border,
        ));

        if let LayoutBoxes::Single(ref mut box_model) = node.layout_boxes {
            let pos_x = origin.0;
            let pos_y = origin.1;
            set_position(
                box_model,
                (pos_x, pos_y),
                (padding.right, padding.top),
                (border.right, border.top),
            );
        }

        let total_width = content_width + padding.right + padding.left + border.right + border.left;
        let total_height =
            content_height + padding.top + padding.bottom + border.top + border.bottom;

        (
            (origin.0 + total_width, origin.1 + total_height),
            total_height,
        )
    }

    /// Resolve Flex container sizes
    /// Resolves container dimensions including padding and border.
    fn resolve_container_sizes(
        &self,
        node: &LayoutNode,
        axis: Axis,
        ctx: &LayoutContext,
    ) -> ContainerSizes {
        let cbm = ctx.containing_block_main(axis);
        let cbc = ctx.containing_block_cross(axis);
        let vw = ctx.viewport_width;
        let vh = ctx.viewport_height;

        // Resolve padding and border
        let (pms_len, pme_len) = axis.padding_main(&node.style.spacing);
        let (pcs_len, pce_len) = axis.padding_cross(&node.style.spacing);
        let (bms_len, bme_len) = axis.border_main(&node.style.spacing);
        let (bcs_len, bce_len) = axis.border_cross(&node.style.spacing);

        // Percentage values for padding/margin/border are resolved
        // relative to the containing block's width (not height). Use the
        // containing block width (or viewport width fallback) for percentage
        // resolution on vertical edges as well.
        let containing_width = ctx.containing_block_width.unwrap_or(vw);

        let pms = pms_len
            .resolve_with(Some(containing_width), vw, vh)
            .unwrap_or(0.0);
        let pme = pme_len
            .resolve_with(Some(containing_width), vw, vh)
            .unwrap_or(0.0);
        let pcs = pcs_len
            .resolve_with(Some(containing_width), vw, vh)
            .unwrap_or(0.0);
        let pce = pce_len
            .resolve_with(Some(containing_width), vw, vh)
            .unwrap_or(0.0);
        let bms = bms_len
            .resolve_with(Some(containing_width), vw, vh)
            .unwrap_or(0.0);
        let bme = bme_len
            .resolve_with(Some(containing_width), vw, vh)
            .unwrap_or(0.0);
        let bcs = bcs_len
            .resolve_with(Some(containing_width), vw, vh)
            .unwrap_or(0.0);
        let bce = bce_len
            .resolve_with(Some(containing_width), vw, vh)
            .unwrap_or(0.0);

        // Resolve specified sizes
        let specified_main = axis.size_main(&node.style.size).resolve_with(cbm, vw, vh);
        let mut content_main = match (specified_main, node.style.box_sizing) {
            (Some(m), BoxSizing::BorderBox) => Some((m - pms - pme - bms - bme).max(0.0)),
            (Some(m), BoxSizing::ContentBox) => Some(m),
            (None, _) => ctx
                .parent_assigned_border_main(axis)
                .map(|m| (m - pms - pme - bms - bme).max(0.0)),
        };

        let specified_cross = axis.size_cross(&node.style.size).resolve_with(cbc, vw, vh);
        let mut content_cross = match (specified_cross, node.style.box_sizing) {
            (Some(c), BoxSizing::BorderBox) => Some((c - pcs - pce - bcs - bce).max(0.0)),
            (Some(c), BoxSizing::ContentBox) => Some(c),
            (None, _) => ctx
                .parent_assigned_border_cross(axis)
                .map(|c| (c - pcs - pce - bcs - bce).max(0.0)),
        };

        // Apply min/max constraints if sizes are specified
        if let Some(ref mut main) = content_main {
            let min_main = axis.min_main(&node.style.size).resolve_with(cbm, vw, vh);
            let max_main = axis.max_main(&node.style.size).resolve_with(cbm, vw, vh);
            *main = clamp(*main, min_main, max_main);
        }

        if let Some(ref mut cross) = content_cross {
            let min_cross = axis.min_cross(&node.style.size).resolve_with(cbc, vw, vh);
            let max_cross = axis.max_cross(&node.style.size).resolve_with(cbc, vw, vh);
            *cross = clamp(*cross, min_cross, max_cross);
        }

        let padding = match axis {
            Axis::Horizontal => (pms, pcs, pme, pce),
            Axis::Vertical => (pcs, pms, pce, pme),
        }
        .into();

        let border = match axis {
            Axis::Horizontal => (bms, bcs, bme, bce),
            Axis::Vertical => (bcs, bms, bce, bme),
        }
        .into();

        (content_main, content_cross, padding, border)
    }

    /// Layout of Flex child elements
    /// Layouts flex children with flex algorithm.
    ///
    /// Note: Several helper functions that were previously extracted have been inlined here
    /// to keep the flex algorithm flow in one place (per the requested refactor).
    fn layout_flex_children(
        &self,
        node: &mut LayoutNode,
        axis: Axis,
        intrinsic_pass: bool,
        child_ctx: &LayoutContext,
    ) -> (f32, f32) {
        let count = node.children.len();
        if count == 0 {
            return (0.0, 0.0);
        }

        let cbm = child_ctx.containing_block_main(axis);
        let cbc = child_ctx.containing_block_cross(axis);
        let vw = child_ctx.viewport_width;
        let vh = child_ctx.viewport_height;

        let gap = axis
            .gap(&node.style)
            .resolve_with(cbm, vw, vh)
            .unwrap_or(0.0)
            .max(0.0);

        // ---------- Intrinsic pass ----------

        let mut frozen_grow = vec![false; count];
        let mut frozen_shrink = vec![false; count];
        let mut total_grow = 0.0;

        let mut main_sizes = vec![0.0; count]; // content-box main size (base or current)
        let mut main_padding = vec![(0.0, 0.0); count];
        let mut main_border = vec![(0.0, 0.0); count];
        let mut main_margin = vec![(0.0, 0.0); count];
        let mut main_min_max = vec![(None, None); count];

        for (i, child) in node.children.iter_mut().enumerate() {
            let (pad_start, pad_end) = axis.padding_main(&child.style.spacing);
            main_padding[i] = (
                pad_start.resolve_with(cbm, vw, vh).unwrap_or(0.0),
                pad_end.resolve_with(cbm, vw, vh).unwrap_or(0.0),
            );

            let (border_start, border_end) = axis.border_main(&child.style.spacing);
            main_border[i] = (
                border_start.resolve_with(cbm, vw, vh).unwrap_or(0.0),
                border_end.resolve_with(cbm, vw, vh).unwrap_or(0.0),
            );

            let mar_start = axis.margin_main_start(&child.style.spacing);
            let mar_end = axis.margin_main_end(&child.style.spacing);
            main_margin[i] = (
                mar_start.resolve_with(cbm, vw, vh).unwrap_or(0.0),
                mar_end.resolve_with(cbm, vw, vh).unwrap_or(0.0),
            );

            let min_main = axis.min_main(&child.style.size).resolve_with(cbm, vw, vh);
            let max_main = axis.max_main(&child.style.size).resolve_with(cbm, vw, vh);
            main_min_max[i] = (min_main, max_main);

            let basis = child.style.item_style.flex_basis.resolve_with(cbm, vw, vh);

            let base_content_main = match basis {
                Some(v) => v,
                None => {
                    let explicit = axis
                        .size_main(&child.style.size)
                        .resolve_with(cbm, vw, vh)
                        .map(|s| {
                            resolve_content_size_with_box_sizing(
                                child,
                                s,
                                main_padding[i],
                                main_border[i],
                            )
                        });

                    match explicit {
                        None => {
                            // intrinsic measurement
                            // Set containing block size to none
                            // to prevent the child from expanding out of the parent's size
                            let intrinsic_ctx = LayoutContext {
                                containing_block_width: child_ctx.containing_block_width,
                                containing_block_height: child_ctx.containing_block_height,
                                available_block_width: None,
                                available_block_height: None,
                                viewport_width: vw,
                                viewport_height: vh,
                                parent_assigned_border_width: None,
                                parent_assigned_border_height: None,
                            };

                            self.layout_node(
                                child,
                                true,
                                (0.0, 0.0),
                                Edge::default(),
                                0.0,
                                &intrinsic_ctx,
                            );

                            if let LayoutBoxes::Single(ref box_model) = child.layout_boxes {
                                axis.main(&box_model.content_box)
                            } else {
                                0.0
                            }
                        }
                        Some(v) => {
                            frozen_grow[i] = true;
                            frozen_shrink[i] = true;
                            v
                        }
                    }
                }
            };

            if !frozen_grow[i] {
                total_grow += child.style.item_style.flex_grow;
                if child.style.item_style.flex_grow == 0.0 {
                    frozen_grow[i] = true;
                }
            }

            if child.style.item_style.flex_shrink == 0.0 {
                frozen_shrink[i] = true;
            }

            main_sizes[i] = base_content_main;
        }

        let total_base_main: f32 = main_sizes.iter().sum();
        let total_main_padding: f32 = main_padding.iter().map(|(start, end)| start + end).sum();
        let total_main_border: f32 = main_border.iter().map(|(start, end)| start + end).sum();
        let total_main_margin: f32 = main_margin.iter().map(|(start, end)| start + end).sum();
        let gaps = gap * count.saturating_sub(1) as f32;

        let mut remaining = cbm
            .map(|m| {
                m - (total_base_main
                    + gaps
                    + total_main_padding
                    + total_main_border
                    + total_main_margin)
            })
            .unwrap_or(0.0);

        // ---------- redistribute loop ----------

        loop {
            if remaining > 0.0 {
                if total_grow <= 0.0 {
                    break;
                }

                let mut used = 0.0;

                for i in 0..count {
                    if frozen_grow[i] {
                        continue;
                    }

                    let child = &node.children[i];
                    let grow = child.style.item_style.flex_grow;

                    let delta = remaining * (grow / total_grow);

                    let min = main_min_max[i].0;
                    let max = main_min_max[i].1;

                    let proposed_content = main_sizes[i] + delta;
                    let clamped_content = match child.style.box_sizing {
                        BoxSizing::ContentBox => clamp(proposed_content, min, max),
                        BoxSizing::BorderBox => {
                            let padding_border_main = main_padding[i].0
                                + main_padding[i].1
                                + main_border[i].0
                                + main_border[i].1;
                            let proposed_border = proposed_content + padding_border_main;

                            let clamped_border = clamp(proposed_border, min, max);

                            (clamped_border - padding_border_main).max(0.0)
                        }
                    };

                    let actual = clamped_content - main_sizes[i];

                    main_sizes[i] = clamped_content;
                    used += actual;

                    if proposed_content != clamped_content {
                        frozen_grow[i] = true;
                        total_grow -= grow;
                    }
                }

                remaining -= used;

                if used.abs() < 0.001 {
                    break;
                }
            } else {
                // negative remaining = overflow
                let mut total_shrink_factor = 0.0;

                for (i, child) in node.children.iter().enumerate() {
                    if frozen_shrink[i] {
                        continue;
                    }

                    let shrink = child.style.item_style.flex_shrink;
                    total_shrink_factor += shrink * main_sizes[i];
                }

                if total_shrink_factor <= 0.0 {
                    break;
                }

                let mut used = 0.0;

                for i in 0..count {
                    if frozen_shrink[i] {
                        continue;
                    }

                    let child = &node.children[i];

                    let shrink = child.style.item_style.flex_shrink;
                    let basis = main_sizes[i];

                    let ratio = (shrink * basis) / total_shrink_factor;

                    let delta = remaining * ratio; // remaining is negative
                    let new_size = main_sizes[i] + delta;

                    let min = main_min_max[i].0;
                    let max = main_min_max[i].1;

                    let proposed_content = main_sizes[i] + delta;
                    let clamped_content = match child.style.box_sizing {
                        BoxSizing::ContentBox => clamp(proposed_content, min, max),
                        BoxSizing::BorderBox => {
                            let padding_border_main = main_padding[i].0
                                + main_padding[i].1
                                + main_border[i].0
                                + main_border[i].1;
                            let proposed_border = proposed_content + padding_border_main;

                            let clamped_border = clamp(proposed_border, min, max);

                            (clamped_border - padding_border_main).max(0.0)
                        }
                    };

                    let actual = clamped_content - main_sizes[i];

                    main_sizes[i] = clamped_content;
                    used += actual;

                    if (clamped_content - new_size).abs() > 0.001 {
                        frozen_shrink[i] = true;
                    }
                }

                remaining -= used;

                if used.abs() < 0.001 {
                    break;
                }
            }
        }

        // ---------- final layout ----------

        let mut total_border_main: f32 = 0.0;
        let mut max_cross: f32 = 0.0;

        for (i, child) in node.children.iter_mut().enumerate() {
            let is_auto_margin = axis
                .margin_cross_start(&child.style.spacing)
                .resolve_with(cbc, vw, vh)
                .is_none()
                || axis
                    .margin_cross_end(&child.style.spacing)
                    .resolve_with(cbc, vw, vh)
                    .is_none();

            let align = child
                .style
                .item_style
                .align_self
                .unwrap_or(node.style.align_items);

            let is_auto_cross = axis.size_cross(&child.style.size) == &Length::Auto;

            let stretched_cross =
                if !is_auto_margin && matches!(align, AlignItems::Stretch) && is_auto_cross {
                    cbc.map(|v| {
                        v - axis
                            .margin_cross_start(&child.style.spacing)
                            .resolve_with(cbc, vw, vh)
                            .unwrap_or(0.0)
                            - axis
                                .margin_cross_end(&child.style.spacing)
                                .resolve_with(cbc, vw, vh)
                                .unwrap_or(0.0)
                    })
                } else {
                    None
                };

            let (parent_assigned_border_width, parent_assigned_border_height) = {
                let main_bargin_box = main_sizes[i]
                    + main_padding[i].0
                    + main_padding[i].1
                    + main_border[i].0
                    + main_border[i].1;
                match axis {
                    Axis::Horizontal => (Some(main_bargin_box), stretched_cross),
                    Axis::Vertical => (stretched_cross, Some(main_bargin_box)),
                }
            };

            let child_ctx = LayoutContext {
                parent_assigned_border_width,
                parent_assigned_border_height,
                ..*child_ctx
            };

            self.layout_node(
                child,
                intrinsic_pass,
                (0.0, 0.0),
                Edge::default(),
                0.0,
                &child_ctx,
            );

            if let LayoutBoxes::Single(box_model) = &child.layout_boxes {
                total_border_main += axis.main(&box_model.border_box);
                max_cross = max_cross.max(axis.cross(&box_model.border_box));
            }
        }

        let children_main = total_border_main + gaps;

        (children_main, max_cross)
    }
}

// ===========================
// Helper Functions (non-flex specific)
// ===========================

/// Resolves content size based on the box-sizing property.
///
/// # Arguments
/// * `node` - The layout node with box-sizing style
/// * `size` - The border or content box size to resolve
/// * `padding_edge` - (padding start, padding end)
/// * `border_edge` - (border start, border end)
fn resolve_content_size_with_box_sizing(
    node: &LayoutNode,
    size: f32,
    padding_edge: (f32, f32),
    border_edge: (f32, f32),
) -> f32 {
    match node.style.box_sizing {
        BoxSizing::ContentBox => size,
        BoxSizing::BorderBox => {
            size - padding_edge.0 - padding_edge.1 - border_edge.0 - border_edge.1
        }
    }
    .max(0.0)
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
        x: 0.0,
        y: 0.0,
        width: content_width + pl + pr,
        height: content_height + pt + pb,
    };

    let content_box = Rect {
        x: 0.0,
        y: 0.0,
        width: content_width,
        height: content_height,
    };

    let children_box = Rect {
        x: 0.0,
        y: 0.0,
        width: children_width,
        height: children_height,
    };

    BoxModel {
        content_box,
        padding_box,
        border_box,
        children_box,
    }
}

/// Sets the position of a box model at given border-box coordinates.
///
/// # Arguments
/// * `box_model` - The box model to position
/// * `border_pos` - (x, y) position for the border box
/// * `padding_edge` - (padding left, padding top)
/// * `border_edge` - (border left, border top)
fn set_position(
    box_model: &mut BoxModel,
    border_pos: (f32, f32),
    padding_edge: (f32, f32),
    border_edge: (f32, f32),
) {
    let (bx, by) = border_pos;

    box_model.border_box.x = bx;
    box_model.border_box.y = by;

    let (pl, pt) = padding_edge;
    let (bl, bt) = border_edge;

    box_model.padding_box.x = bx + bl;
    box_model.padding_box.y = by + bt;

    box_model.content_box.x = bx + bl + pl;
    box_model.content_box.y = by + bt + pt;

    box_model.children_box.x = box_model.content_box.x;
    box_model.children_box.y = box_model.content_box.y;
}

/// Clamps a value between optional minimum and maximum bounds.
fn clamp(value: f32, min: Option<f32>, max: Option<f32>) -> f32 {
    let v = min.map_or(value, |m| value.max(m));
    max.map_or(v, |m| v.min(m))
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

fn resolve_margins(spacing: &Spacing, ctx: &LayoutContext) -> Edge {
    // Percentage margins resolve against containing block width
    let containing_width = ctx.containing_block_width.unwrap_or(ctx.viewport_width);
    let vw = ctx.viewport_width;
    let vh = ctx.viewport_height;

    (
        spacing
            .margin_left
            .resolve_with(ctx.containing_block_width, vw, vh)
            .unwrap_or(0.0),
        spacing
            .margin_top
            .resolve_with(Some(containing_width), vw, vh)
            .unwrap_or(0.0),
        spacing
            .margin_right
            .resolve_with(ctx.containing_block_width, vw, vh)
            .unwrap_or(0.0),
        spacing
            .margin_bottom
            .resolve_with(Some(containing_width), vw, vh)
            .unwrap_or(0.0),
    )
        .into()
}

/// Computes justify-content offset and gap spacing.
/// Returns: (start_offset, gap_between_items)
fn resolve_justify_content(justify: JustifyContent, remaining: f32, count: usize) -> (f32, f32) {
    match justify {
        JustifyContent::Start => (0.0, 0.0),
        JustifyContent::Center => (remaining / 2.0, 0.0),
        JustifyContent::End => (remaining, 0.0),
        JustifyContent::SpaceBetween => {
            if count > 1 {
                (0.0, remaining / (count as f32 - 1.0))
            } else {
                (0.0, 0.0)
            }
        }
        JustifyContent::SpaceAround => {
            if count > 0 {
                let gap = remaining / count as f32;
                (gap / 2.0, gap)
            } else {
                (0.0, 0.0)
            }
        }
        JustifyContent::SpaceEvenly => {
            if count > 0 {
                let gap = remaining / (count as f32 + 1.0);
                (gap, gap)
            } else {
                (0.0, 0.0)
            }
        }
    }
}

/// Computes align-items position offset for an item.
fn resolve_align_position(align: AlignItems, size: f32, container: f32) -> f32 {
    let free = container - size;

    match align {
        AlignItems::Start | AlignItems::Stretch => 0.0,
        AlignItems::Center => free / 2.0,
        AlignItems::End => free,
    }
}

/// Resolves padding values.
/// Percentage values resolve relative to the containing block's width.
///
/// # Returns
/// - (padding_left, padding_top, padding_right, padding_bottom)
fn resolve_padding(spacing: &Spacing, ctx: &LayoutContext) -> Edge {
    let containing_width = ctx.containing_block_width.unwrap_or(ctx.viewport_width);
    let vw = ctx.viewport_width;
    let vh = ctx.viewport_height;

    (
        spacing
            .padding_left
            .resolve_with(Some(containing_width), vw, vh)
            .unwrap_or(0.0),
        spacing
            .padding_top
            .resolve_with(Some(containing_width), vw, vh)
            .unwrap_or(0.0),
        spacing
            .padding_right
            .resolve_with(Some(containing_width), vw, vh)
            .unwrap_or(0.0),
        spacing
            .padding_bottom
            .resolve_with(Some(containing_width), vw, vh)
            .unwrap_or(0.0),
    )
        .into()
}

/// Resolves border values.
/// Percentage values resolve relative to the containing block's width.
///
/// # Returns
/// - (border_left, border_top, border_right, border_bottom)
fn resolve_border(spacing: &Spacing, ctx: &LayoutContext) -> Edge {
    let containing_width = ctx.containing_block_width.unwrap_or(ctx.viewport_width);
    let vw = ctx.viewport_width;
    let vh = ctx.viewport_height;

    (
        spacing
            .border_left
            .resolve_with(Some(containing_width), vw, vh)
            .unwrap_or(0.0),
        spacing
            .border_top
            .resolve_with(Some(containing_width), vw, vh)
            .unwrap_or(0.0),
        spacing
            .border_right
            .resolve_with(Some(containing_width), vw, vh)
            .unwrap_or(0.0),
        spacing
            .border_bottom
            .resolve_with(Some(containing_width), vw, vh)
            .unwrap_or(0.0),
    )
        .into()
}
