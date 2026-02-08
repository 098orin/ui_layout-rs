// Layout engine
// -----------------------
// This module implements the layout algorithm for `LayoutNode` trees.
// It computes box-models (content / padding / border / children boxes)
// and positions children for block, inline and flex layout modes.

use crate::{
    AlignItems, BoxModel, BoxSizing, Display, FlexDirection, FragmentPlacement, ItemFragment,
    JustifyContent, LayoutBoxes, LayoutNode, Length, Rect, Spacing, Style,
};

/// Container dimensions for flex layout.
///
/// Fields:
/// - Main-axis content size (None if auto)
/// - Cross-axis content size (None if auto)
/// - Padding edges (start, before, end, after)
/// - Border edges (start, before, end, after)
type ContainerSizes = (
    Option<f32>,
    Option<f32>,
    (f32, f32, f32, f32),
    (f32, f32, f32, f32),
);

/// Parameters for creating and setting a `BoxModel` for a node.
/// Centralizes inputs for box model creation functions.
struct BoxModelParams {
    /// Content sizes along main and cross axes (already resolved)
    content_main: f32,
    content_cross: f32,
    /// Layout axis (horizontal or vertical)
    axis: Axis,
    /// Origin point (border-box top-left) in parent's coordinate space
    origin: (f32, f32),
    /// Padding edges: (start, before, end, after)
    padding: (f32, f32, f32, f32),
    /// Border edges: (start, before, end, after)
    border: (f32, f32, f32, f32),
}

/// Context for laying out inline fragments (text runs, inline content, line breaks).
/// Maintains mutable state for cursor position and line metrics during layout.
struct FragmentLayoutContext {
    cursor_x: f32,
    cursor_y: f32,
    line_height: f32,
    max_width: f32,
    line_index: usize,
    origin: (f32, f32),
}

/// Margin collapsing context for block flow layout.
/// Tracks margin information needed for proper margin collapsing between siblings.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct MarginCollapsingContext {
    /// The bottom margin of this element (for collapsing with next sibling)
    margin_after: f32,
}

/// Layout context carrying resolved sizing information down the tree.
///
/// Fields:
/// - `containing_block_*`: Size of the containing block (None if auto)
/// - `viewport_*`: Absolute viewport dimensions for percentage resolution
/// - `parent_assigned_border_*`: Border-box sizes assigned by parent (for stretch)
pub(crate) struct LayoutContext {
    pub(crate) containing_block_width: Option<f32>,
    pub(crate) containing_block_height: Option<f32>,
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

    /// Returns viewport length along the main axis (used for percentage resolution).
    fn viewport_main(&self, axis: Axis) -> f32 {
        match axis {
            Axis::Horizontal => self.viewport_width,
            Axis::Vertical => self.viewport_height,
        }
    }

    /// Returns viewport length along the cross axis.
    fn viewport_cross(&self, axis: Axis) -> f32 {
        match axis {
            Axis::Horizontal => self.viewport_height,
            Axis::Vertical => self.viewport_width,
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

/// A flex item with computed layout parameters.
/// Used internally during flex layout algorithm phases.
#[derive(Debug)]
struct FlexItem {
    index: usize,
    /// flex-basis resolved value
    base_size: f32,
    /// flex-grow property
    flex_grow: f32,
    /// flex-shrink property
    flex_shrink: f32,
    /// min-width/height constraint (or auto)
    min_main: Option<f32>,
    /// max-width/height constraint
    max_main: Option<f32>,
    /// Computed main size after flex algorithm
    final_main_size: f32,
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
    pub fn layout(root: &mut LayoutNode, width: f32, height: f32) {
        let ctx = LayoutContext {
            containing_block_width: Some(width),
            containing_block_height: Some(height),
            viewport_width: width,
            viewport_height: height,
            // Root has no parent; assign viewport dimensions for layout calculations
            parent_assigned_border_width: Some(width),
            parent_assigned_border_height: Some(height),
        };

        let engine = LayoutEngine;
        engine.layout_node(root, false, (0.0, 0.0), 0.0, &ctx);
    }

    /// Layouts a single node and its descendants.
    /// Returns: ((end_x, end_y), height)
    fn layout_node(
        &self,
        node: &mut LayoutNode,
        intrinsic_pass: bool,
        origin: (f32, f32),
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
            Display::Block | Display::Inline | Display::Flex { .. } => {
                self.layout_unified_flow(node, intrinsic_pass, origin, incoming_line_height, ctx)
            }
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
            Display::Inline => {
                self.layout_inline_flow(node, intrinsic_pass, origin, incoming_line_height, ctx)
            }
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
        let (children_main, children_cross) = if !intrinsic_pass
            || content_main.is_none()
            || content_cross.is_none()
        {
            self.layout_flex_children(node, axis, intrinsic_pass, content_main, content_cross, ctx)
        } else {
            (0.0, 0.0)
        };

        // Step 3: Determine final container size
        let mut final_content_main = content_main.unwrap_or(children_main);
        let mut final_content_cross = content_cross.unwrap_or(children_cross);

        // Step 3a: Apply min/max constraints to auto-sized flex container dimensions
        let vm = ctx.viewport_main(axis);
        let vc = ctx.viewport_cross(axis);
        let cbm = ctx.containing_block_main(axis);
        let cbc = ctx.containing_block_cross(axis);

        let main_size_was_auto = content_main.is_none();
        let cross_size_was_auto = content_cross.is_none();

        if main_size_was_auto {
            let min_main = axis.min_main(&node.style.size).resolve_with(cbm, vm);
            let max_main = axis.max_main(&node.style.size).resolve_with(cbm, vm);
            final_content_main = clamp(final_content_main, min_main, max_main);
        }

        if cross_size_was_auto {
            let min_cross = axis.min_cross(&node.style.size).resolve_with(cbc, vc);
            let max_cross = axis.max_cross(&node.style.size).resolve_with(cbc, vc);
            final_content_cross = clamp(final_content_cross, min_cross, max_cross);
        }

        // Step 3b: Re-layout children if main size changed due to min/max constraints
        if !intrinsic_pass && main_size_was_auto && final_content_main != children_main {
            self.layout_flex_children(
                node,
                axis,
                false,
                Some(final_content_main),
                Some(final_content_cross),
                ctx,
            );
        }

        // Step 4: Create box model
        self.create_and_set_box_model(
            node,
            BoxModelParams {
                content_main: final_content_main,
                content_cross: final_content_cross,
                axis,
                origin,
                padding,
                border,
            },
            ctx,
        );

        // Step 4.5: Re-apply cross-axis alignment with final container size
        // Min/max constraints may have changed the container size after child layout
        if !intrinsic_pass {
            self.handle_cross_axis_alignment(node, axis, final_content_cross, ctx);
        }

        // Step 5: Set child positions (only in non-intrinsic pass)
        if !intrinsic_pass {
            self.position_flex_children(node, axis, ctx);
        }

        let end_x = origin.0 + node.layout_boxes.width();
        let end_y = origin.1 + node.layout_boxes.height();
        ((end_x, end_y), node.layout_boxes.height())
    }

    /// Layouts a block container.
    ///
    /// Process:
    /// 1. Resolve element's own size and spacing
    /// 2. Layout children in block flow with margin collapsing
    /// 3. Position children with proper margin collapsing semantics
    /// 4. Create box model
    fn layout_block_flow(
        &self,
        node: &mut LayoutNode,
        intrinsic_pass: bool,
        origin: (f32, f32),
        _incoming_line_height: f32,
        ctx: &LayoutContext,
    ) -> ((f32, f32), f32) {
        let mut cursor_y = origin.1;

        let padding = resolve_padding(&node.style.spacing, &ctx);
        let border = resolve_border(&node.style.spacing, &ctx);

        // Step 1: Resolve the block's own size
        let content_width_opt = node
            .style
            .size
            .width
            .resolve_with(ctx.containing_block_width, ctx.viewport_width)
            .map(|width| {
                let padding_edge = (padding.0, padding.2);
                let border_edge = (border.0, border.2);
                resolve_content_size_with_box_sizing(node, width, padding_edge, border_edge)
            })
            .or(ctx
                .parent_assigned_border_width
                .map(|b| b - border.0 - border.2 - padding.0 - padding.2))
            .or(ctx
                .containing_block_width
                .map(|cbw| cbw - border.0 - border.2 - padding.0 - padding.2));

        let content_height_opt = node
            .style
            .size
            .height
            .resolve_with(ctx.containing_block_height, ctx.viewport_height)
            .map(|height| {
                let padding_edge = (padding.1, padding.3);
                let border_edge = (border.1, border.3);
                resolve_content_size_with_box_sizing(node, height, padding_edge, border_edge)
            })
            .or(ctx
                .parent_assigned_border_height
                .map(|b| b - border.1 - border.3 - padding.1 - padding.3));

        // Step 2: Resolve all children's margins (parent resolves children's margins)
        let containing_width = content_width_opt.unwrap_or(0.0);
        let children_margins = resolve_all_children_margins(
            &node.children,
            containing_width,
            ctx.viewport_width,
            ctx.viewport_height,
        );

        // Step 2b: First pass - layout children to determine sizes
        for child in node.children.iter_mut() {
            let child_ctx = LayoutContext {
                containing_block_width: content_width_opt,
                containing_block_height: content_height_opt,
                parent_assigned_border_width: None,
                parent_assigned_border_height: None,
                ..*ctx
            };

            // Layout child at origin (0,0) for intrinsic sizing
            let ((_, child_end_y), _) =
                self.layout_node(child, intrinsic_pass, (0.0, 0.0), 0.0, &child_ctx);

            // Track vertical layout progression
            cursor_y = child_end_y;
        }

        // Step 3: Apply min/max constraints to content width
        let mut final_content_width = content_width_opt.unwrap_or(0.0);
        final_content_width = apply_size_constraints(
            final_content_width,
            &node.style.size,
            ctx,
            true, // is_width
        );

        // Step 4: Determine final block height
        let mut final_content_height = if let Some(h) = content_height_opt {
            h
        } else {
            // Auto height: use content based on children
            let child_based_height = cursor_y - origin.1;

            // For stretched children, parent-assigned height may be larger
            if let Some(assigned_h) = ctx.parent_assigned_border_height {
                let stretch_height = match node.style.box_sizing {
                    BoxSizing::ContentBox => assigned_h,
                    BoxSizing::BorderBox => {
                        (assigned_h - padding.1 - padding.3 - border.1 - border.3).max(0.0)
                    }
                };
                if stretch_height > child_based_height {
                    stretch_height
                } else {
                    child_based_height
                }
            } else {
                child_based_height
            }
        };
        final_content_height = apply_size_constraints(
            final_content_height,
            &node.style.size,
            ctx,
            false, // is_height
        );

        // Step 5: Resolve block's margins
        let (margins, _) =
            resolve_margins_with_collapsing_enhanced(&node.style.spacing, &ctx, true, 0.0);

        // Step 6: Create box model
        node.layout_boxes = LayoutBoxes::Single(create_box_model(
            final_content_width,
            final_content_height,
            final_content_width,
            final_content_height,
            padding,
            border,
        ));

        if let LayoutBoxes::Single(ref mut box_model) = node.layout_boxes {
            let pos_x = origin.0 + margins.0;
            let pos_y = origin.1 + margins.1;
            set_position(
                box_model,
                (pos_x, pos_y),
                (padding.0, padding.1),
                (border.0, border.1),
            );

            // Step 7: Position children using parent-resolved margins
            if !intrinsic_pass {
                let mut child_y_offset = 0.0;

                for (i, child) in node.children.iter_mut().enumerate() {
                    // Use parent-resolved margins for this child
                    let (margin_left, margin_top, _margin_right, _) = children_margins
                        .get(i)
                        .copied()
                        .unwrap_or((0.0, 0.0, 0.0, 0.0));

                    // Handle auto margins for horizontal center alignment
                    let margin_left_auto = child.style.spacing.margin_left == Length::Auto;
                    let margin_right_auto = child.style.spacing.margin_right == Length::Auto;

                    let child_x = if margin_left_auto && margin_right_auto {
                        // Center child horizontally when both margins are auto
                        let child_width =
                            if let LayoutBoxes::Single(ref box_model) = child.layout_boxes {
                                box_model.border_box.width
                            } else {
                                0.0
                            };
                        (final_content_width - child_width) / 2.0
                    } else {
                        margin_left
                    };

                    // Apply parent-resolved margin (includes collapsing with previous sibling)
                    child_y_offset += margin_top;

                    // Position child relative to parent content box
                    let (child_current_left, child_current_top) =
                        if let LayoutBoxes::Single(ref box_model) = child.layout_boxes {
                            (box_model.border_box.x, box_model.border_box.y)
                        } else {
                            (0.0, 0.0)
                        };

                    let shift_x = child_x - child_current_left;
                    let shift_y = child_y_offset - child_current_top;

                    child.layout_boxes.shift(shift_x, shift_y);

                    // Update offset for next child (only add child height, not margins)
                    // Next child's margin_top already includes proper collapsing via parent resolution
                    if let LayoutBoxes::Single(ref box_model) = child.layout_boxes {
                        child_y_offset += box_model.border_box.height;
                    }
                }
            }
        }

        let total_width = final_content_width
            + padding.0
            + padding.2
            + border.0
            + border.2
            + margins.0
            + margins.2;
        let total_height = final_content_height
            + padding.1
            + padding.3
            + border.1
            + border.3
            + margins.1
            + margins.3;

        (
            (origin.0 + total_width, origin.1 + total_height),
            total_height,
        )
    }

    /// Layouts an inline container as flow layout.
    /// Handles inline content (text runs and inline elements).
    fn layout_inline_flow(
        &self,
        node: &mut LayoutNode,
        intrinsic_pass: bool,
        origin: (f32, f32),
        incoming_line_height: f32,
        ctx: &LayoutContext,
    ) -> ((f32, f32), f32) {
        let (mut cursor_x, mut cursor_y) = origin;
        let mut line_height = incoming_line_height;
        let mut line_index = 0;

        let max_width = ctx.containing_block_width.unwrap_or(ctx.viewport_width);

        // Resolve inline container's spacing
        let ctx_for_inline = LayoutContext {
            containing_block_width: ctx.containing_block_width,
            containing_block_height: ctx.containing_block_height,
            ..*ctx
        };
        let padding = resolve_padding(&node.style.spacing, &ctx_for_inline);
        let border = resolve_border(&node.style.spacing, &ctx_for_inline);
        let (margins, _) = resolve_margins_with_collapsing_enhanced(
            &node.style.spacing,
            &ctx_for_inline,
            false,
            0.0,
        );

        if !intrinsic_pass {
            node.placements.clear();
        }

        // Layout inline fragments (text runs and inline content)
        if !node.self_fragments.is_empty() {
            // Calculate content area boundaries for fragments
            let content_start_x = cursor_x + margins.0 + border.0 + padding.0;
            let content_start_y = cursor_y + margins.1 + border.1 + padding.1;
            let content_cursor_x = content_start_x;
            let content_cursor_y = content_start_y;

            cursor_x = content_cursor_x;
            cursor_y = content_cursor_y;

            for frag in &node.self_fragments {
                if intrinsic_pass {
                    match frag {
                        crate::ItemFragment::LineBreak => {
                            cursor_x = content_start_x;
                            cursor_y += line_height;
                            line_height = 0.0;
                            line_index += 1;
                        }
                        crate::ItemFragment::Fragment(f) => {
                            if cursor_x + f.width > max_width && cursor_x > content_start_x {
                                cursor_x = content_start_x;
                                cursor_y += line_height;
                                line_height = 0.0;
                                line_index += 1;
                            }
                            cursor_x += f.width;
                            line_height = line_height.max(f.height);
                        }
                    }
                } else {
                    let mut ctx = FragmentLayoutContext {
                        cursor_x,
                        cursor_y,
                        line_height,
                        max_width,
                        line_index,
                        origin: (content_start_x, content_start_y),
                    };
                    self.layout_fragment(frag, &mut ctx, &mut node.placements);
                    cursor_x = ctx.cursor_x;
                    cursor_y = ctx.cursor_y;
                    line_height = ctx.line_height;
                    line_index = ctx.line_index;
                }
            }

            let content_width = cursor_x - content_start_x;
            let content_height = cursor_y - content_start_y + line_height;

            // Create box model with spacing
            node.layout_boxes = LayoutBoxes::Single(create_box_model(
                content_width,
                content_height,
                content_width,
                content_height,
                padding,
                border,
            ));

            // Position the box model
            if let LayoutBoxes::Single(ref mut box_model) = node.layout_boxes {
                let pos_x = origin.0 + margins.0;
                let pos_y = origin.1 + margins.1;
                set_position(
                    box_model,
                    (pos_x, pos_y),
                    (padding.0, padding.1),
                    (border.0, border.1),
                );
            }

            let total_width =
                content_width + padding.0 + padding.2 + border.0 + border.2 + margins.0 + margins.2;
            let total_height = content_height
                + padding.1
                + padding.3
                + border.1
                + border.3
                + margins.1
                + margins.3;

            return (
                (origin.0 + total_width, origin.1 + total_height),
                total_height,
            );
        }

        // Empty inline element - still create box model with spacing
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
            let pos_x = origin.0 + margins.0;
            let pos_y = origin.1 + margins.1;
            set_position(
                box_model,
                (pos_x, pos_y),
                (padding.0, padding.1),
                (border.0, border.1),
            );
        }

        let total_width =
            content_width + padding.0 + padding.2 + border.0 + border.2 + margins.0 + margins.2;
        let total_height =
            content_height + padding.1 + padding.3 + border.1 + border.3 + margins.1 + margins.3;

        (
            (origin.0 + total_width, origin.1 + total_height),
            total_height,
        )
    }

    fn layout_fragment(
        &self,
        frag: &ItemFragment,
        ctx: &mut FragmentLayoutContext,
        placements: &mut Vec<FragmentPlacement>,
    ) {
        match frag {
            ItemFragment::LineBreak => {
                ctx.cursor_x = ctx.origin.0;
                ctx.cursor_y += ctx.line_height;
                ctx.line_height = 0.0;
                ctx.line_index += 1;

                placements.push(FragmentPlacement {
                    offset: (ctx.cursor_x - ctx.origin.0, ctx.cursor_y - ctx.origin.1),
                    line_index: ctx.line_index,
                });
            }
            ItemFragment::Fragment(f) => {
                // Check for line wrapping
                if ctx.cursor_x + f.width > ctx.max_width && ctx.cursor_x > ctx.origin.0 {
                    ctx.cursor_x = ctx.origin.0;
                    ctx.cursor_y += ctx.line_height;
                    ctx.line_height = 0.0;
                    ctx.line_index += 1;
                }

                placements.push(FragmentPlacement {
                    offset: (ctx.cursor_x - ctx.origin.0, ctx.cursor_y - ctx.origin.1),
                    line_index: ctx.line_index,
                });

                ctx.cursor_x += f.width;
                ctx.line_height = ctx.line_height.max(f.height);
            }
        }
    }

    /// Resolve Flex container sizes
    /// Resolves container dimensions including padding and border.
    fn resolve_container_sizes(
        &self,
        node: &LayoutNode,
        axis: Axis,
        ctx: &LayoutContext,
    ) -> ContainerSizes {
        let vm = ctx.viewport_main(axis);
        let vc = ctx.viewport_cross(axis);
        let cbm = ctx.containing_block_main(axis);
        let cbc = ctx.containing_block_cross(axis);

        // Resolve padding and border
        let (pms_len, pme_len) = axis.padding_main(&node.style.spacing);
        let (pcs_len, pce_len) = axis.padding_cross(&node.style.spacing);
        let (bms_len, bme_len) = axis.border_main(&node.style.spacing);
        let (bcs_len, bce_len) = axis.border_cross(&node.style.spacing);

        // Percentage values for padding/margin/border are resolved
        // relative to the containing block's width (not height). Use the
        // containing block width (or viewport width fallback) for percentage
        // resolution on vertical edges as well.
        let containing_width = ctx.containing_block_width.unwrap_or(ctx.viewport_width);

        let pms = pms_len
            .resolve_with(Some(containing_width), ctx.viewport_width)
            .unwrap_or(0.0);
        let pme = pme_len
            .resolve_with(Some(containing_width), ctx.viewport_width)
            .unwrap_or(0.0);
        let pcs = pcs_len
            .resolve_with(Some(containing_width), ctx.viewport_width)
            .unwrap_or(0.0);
        let pce = pce_len
            .resolve_with(Some(containing_width), ctx.viewport_width)
            .unwrap_or(0.0);
        let bms = bms_len
            .resolve_with(Some(containing_width), ctx.viewport_width)
            .unwrap_or(0.0);
        let bme = bme_len
            .resolve_with(Some(containing_width), ctx.viewport_width)
            .unwrap_or(0.0);
        let bcs = bcs_len
            .resolve_with(Some(containing_width), ctx.viewport_width)
            .unwrap_or(0.0);
        let bce = bce_len
            .resolve_with(Some(containing_width), ctx.viewport_width)
            .unwrap_or(0.0);

        // Resolve specified sizes
        let specified_main = axis.size_main(&node.style.size).resolve_with(cbm, vm);
        let mut content_main = match (specified_main, node.style.box_sizing) {
            (Some(m), BoxSizing::BorderBox) => Some((m - pms - pme - bms - bme).max(0.0)),
            (Some(m), BoxSizing::ContentBox) => Some(m),
            (None, _) => ctx
                .parent_assigned_border_main(axis)
                .map(|m| (m - pms - pme - bms - bme).max(0.0)),
        };

        let specified_cross = axis.size_cross(&node.style.size).resolve_with(cbc, vc);
        let mut content_cross = match (specified_cross, node.style.box_sizing) {
            (Some(c), BoxSizing::BorderBox) => Some((c - pcs - pce - bcs - bce).max(0.0)),
            (Some(c), BoxSizing::ContentBox) => Some(c),
            (None, _) => ctx
                .parent_assigned_border_cross(axis)
                .map(|c| (c - pcs - pce - bcs - bce).max(0.0)),
        };

        // Apply min/max constraints if sizes are specified
        if let Some(ref mut main) = content_main {
            let min_main = axis.min_main(&node.style.size).resolve_with(cbm, vm);
            let max_main = axis.max_main(&node.style.size).resolve_with(cbm, vm);
            *main = clamp(*main, min_main, max_main);
        }

        if let Some(ref mut cross) = content_cross {
            let min_cross = axis.min_cross(&node.style.size).resolve_with(cbc, vc);
            let max_cross = axis.max_cross(&node.style.size).resolve_with(cbc, vc);
            *cross = clamp(*cross, min_cross, max_cross);
        }

        let padding = match axis {
            Axis::Horizontal => (pms, pcs, pme, pce),
            Axis::Vertical => (pcs, pms, pce, pme),
        };

        let border = match axis {
            Axis::Horizontal => (bms, bcs, bme, bce),
            Axis::Vertical => (bcs, bms, bce, bme),
        };

        (content_main, content_cross, padding, border)
    }

    /// Layout of Flex child elements
    /// Layouts flex children with flex algorithm.
    fn layout_flex_children(
        &self,
        node: &mut LayoutNode,
        axis: Axis,
        _intrinsic_pass: bool,
        container_main: Option<f32>,
        container_cross: Option<f32>,
        ctx: &LayoutContext,
    ) -> (f32, f32) {
        // Create context for child elements
        let (content_width_hint, content_height_hint) = match axis {
            Axis::Horizontal => (container_main, container_cross),
            Axis::Vertical => (container_cross, container_main),
        };

        let mut flex_items = Vec::new();
        let mut max_cross = 0.0f32;

        // Phase 1: Determine flex basis and intrinsic sizes
        for (i, child) in node.children.iter_mut().enumerate() {
            // Initial layout to get intrinsic sizes
            let basic_ctx = LayoutContext {
                containing_block_width: content_width_hint,
                containing_block_height: content_height_hint,
                viewport_width: ctx.viewport_width,
                viewport_height: ctx.viewport_height,
                parent_assigned_border_width: None,
                parent_assigned_border_height: None,
            };

            self.layout_node(child, true, (0.0, 0.0), 0.0, &basic_ctx);

            // Calculate base size (flex-basis or main size)
            let base_size =
                LayoutEngine::calculate_flex_base_size(child, axis, content_width_hint, ctx);

            // Get min/max constraints
            let min_main =
                LayoutEngine::resolve_flex_item_min_main(child, axis, content_width_hint, ctx);
            let max_main = axis
                .max_main(&child.style.size)
                .resolve_with(content_width_hint, ctx.viewport_main(axis));

            let cross_size = if let LayoutBoxes::Single(ref box_model) = child.layout_boxes {
                axis.cross(&box_model.border_box)
            } else {
                0.0
            };

            max_cross = max_cross.max(cross_size);

            flex_items.push(FlexItem {
                index: i,
                base_size: clamp(base_size, min_main, max_main),
                flex_grow: child.style.item_style.flex_grow,
                flex_shrink: child.style.item_style.flex_shrink,
                min_main,
                max_main,
                final_main_size: base_size,
            });
        }

        // Phase 2: Resolve flexible lengths
        if let Some(container_main_size) = container_main {
            LayoutEngine::resolve_flexible_lengths(&mut flex_items, container_main_size);
        }

        // Phase 3: Layout children with resolved sizes
        for flex_item in &flex_items {
            let child = &mut node.children[flex_item.index];

            let (new_width, new_height) = match axis {
                Axis::Horizontal => (Some(flex_item.final_main_size), content_height_hint),
                Axis::Vertical => (content_width_hint, Some(flex_item.final_main_size)),
            };

            let flex_ctx = LayoutContext {
                containing_block_width: new_width,
                containing_block_height: new_height,
                viewport_width: ctx.viewport_width,
                viewport_height: ctx.viewport_height,
                parent_assigned_border_width: new_width,
                parent_assigned_border_height: new_height,
            };

            self.layout_node(child, false, (0.0, 0.0), 0.0, &flex_ctx);

            // Enforce the resolved final main size onto the child's box model
            if let LayoutBoxes::Single(ref mut box_model) = child.layout_boxes {
                match axis {
                    Axis::Horizontal => {
                        let target = flex_item.final_main_size;
                        let delta = target - box_model.border_box.width;
                        box_model.border_box.width = target;
                        box_model.padding_box.width =
                            (box_model.padding_box.width + delta).max(0.0);
                        box_model.content_box.width =
                            (box_model.content_box.width + delta).max(0.0);
                        box_model.children_box.width =
                            (box_model.children_box.width + delta).max(0.0);
                    }
                    Axis::Vertical => {
                        let target = flex_item.final_main_size;
                        let delta = target - box_model.border_box.height;
                        box_model.border_box.height = target;
                        box_model.padding_box.height =
                            (box_model.padding_box.height + delta).max(0.0);
                        box_model.content_box.height =
                            (box_model.content_box.height + delta).max(0.0);
                        box_model.children_box.height =
                            (box_model.children_box.height + delta).max(0.0);
                    }
                }
            }
        }

        // Phase 4: Handle cross-axis alignment (stretch)
        self.handle_cross_axis_alignment(node, axis, container_cross.unwrap_or(max_cross), ctx);

        // Calculate total main axis size including gaps
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

        let gap_total = if node.children.len() > 1 {
            let gap = axis
                .gap(&node.style)
                .resolve_with(container_main, ctx.viewport_main(axis))
                .unwrap_or(0.0);
            gap * (node.children.len() as f32 - 1.0)
        } else {
            0.0
        };

        let total_main_size = children_main_total + gap_total;

        (total_main_size, max_cross)
    }

    /// Calculate flex base size for a flex item
    ///
    /// Priority:
    /// 1. If flex-basis is not 'auto', use it
    /// 2. Else if main size is not 'auto', use it
    /// 3. Else use the content size (or 0 for this simplified implementation)
    fn calculate_flex_base_size(
        child: &LayoutNode,
        axis: Axis,
        containing_block_main: Option<f32>,
        ctx: &LayoutContext,
    ) -> f32 {
        let flex_basis = &child.style.item_style.flex_basis;

        // If flex-basis is not auto, use it
        if let Some(basis_size) =
            flex_basis.resolve_with(containing_block_main, ctx.viewport_main(axis))
        {
            return basis_size;
        }

        // Check if explicit main size is set
        let explicit_main_size = match axis {
            Axis::Horizontal => child
                .style
                .size
                .width
                .resolve_with(containing_block_main, ctx.viewport_main(axis)),
            Axis::Vertical => child
                .style
                .size
                .height
                .resolve_with(containing_block_main, ctx.viewport_main(axis)),
        };

        if let Some(explicit_size) = explicit_main_size {
            return explicit_size;
        }

        // For flex items without explicit size or flex-basis, use 0 as base size
        // TODO: In future, implement intrinsic size calculation (min-content)
        0.0
    }

    /// Resolve min-width/height: auto for flex items
    ///
    /// For flex items:
    /// - min-width: auto -> automatic minimum (based on content)
    /// - min-height: auto -> automatic minimum (based on content)
    ///
    /// This simplified implementation treats auto as 0 for now.
    /// TODO: Implement content-based minimum sizing
    fn resolve_flex_item_min_main(
        child: &LayoutNode,
        axis: Axis,
        containing_block_main: Option<f32>,
        ctx: &LayoutContext,
    ) -> Option<f32> {
        let min_main = axis.min_main(&child.style.size);

        match min_main {
            Length::Auto => {
                // Automatic minimum
                // For now, return None to indicate 'auto' (which will be treated as 0)
                // In future, this should calculate content-based minimum size
                None
            }
            _ => min_main.resolve_with(containing_block_main, ctx.viewport_main(axis)),
        }
    }

    /// Resolve flexible lengths - Phase 2 of flex layout algorithm
    fn resolve_flexible_lengths(flex_items: &mut [FlexItem], container_main_size: f32) {
        // Calculate initial free space
        let total_base_sizes: f32 = flex_items.iter().map(|item| item.base_size).sum();
        let initial_free_space = container_main_size - total_base_sizes;

        if initial_free_space > 0.0 {
            // Positive free space: grow items
            LayoutEngine::grow_flex_items(flex_items, initial_free_space);
        } else if initial_free_space < 0.0 {
            // Negative free space: shrink items
            LayoutEngine::shrink_flex_items(flex_items, -initial_free_space);
        } else {
            // Exactly zero free space: use base sizes
            for item in flex_items.iter_mut() {
                item.final_main_size = item.base_size;
            }
        }
    }

    /// Grow flex items - Handles positive free space distribution
    fn grow_flex_items(flex_items: &mut [FlexItem], free_space: f32) {
        let total_flex_grow: f32 = flex_items.iter().map(|item| item.flex_grow).sum();

        if total_flex_grow == 0.0 {
            // No flexible items, use base sizes
            for item in flex_items.iter_mut() {
                item.final_main_size = item.base_size;
            }
            return;
        }

        let mut remaining_free_space = free_space;
        let mut remaining_flex_grow = total_flex_grow;

        // Distribute free space proportionally
        for item in flex_items.iter_mut() {
            if item.flex_grow == 0.0 {
                item.final_main_size = item.base_size;
                continue;
            }

            let flex_share = remaining_free_space * (item.flex_grow / remaining_flex_grow);
            let target_size = item.base_size + flex_share;

            // Apply min/max constraints
            let constrained_size = clamp(target_size, item.min_main, item.max_main);
            item.final_main_size = constrained_size;

            // Update remaining values for fair distribution
            let actual_growth = constrained_size - item.base_size;
            remaining_free_space -= actual_growth;
            remaining_flex_grow -= item.flex_grow;
        }
    }

    /// Shrink flex items - Handles negative free space distribution
    /// Uses scaled shrink factor: flex-shrink * flex-base-size
    fn shrink_flex_items(flex_items: &mut [FlexItem], mut remaining_deficit: f32) {
        // Calculate total scaled flex shrink factor
        let total_scaled_shrink: f32 = flex_items
            .iter()
            .map(|item| item.base_size * item.flex_shrink)
            .sum();

        if total_scaled_shrink == 0.0 {
            // No flexible items for shrinking, use base sizes
            for item in flex_items.iter_mut() {
                item.final_main_size = item.base_size;
            }
            return;
        }

        // First pass: calculate shrink amounts and track violations
        let mut violations = vec![false; flex_items.len()];
        let mut has_violations = true;

        while has_violations && remaining_deficit > 0.001 {
            has_violations = false;
            let mut total_scaled = 0.0f32;

            // Recalculate total scaled shrink without violated items
            for (i, item) in flex_items.iter().enumerate() {
                if !violations[i] && item.flex_shrink > 0.0 {
                    total_scaled += item.base_size * item.flex_shrink;
                }
            }

            if total_scaled < 0.001 {
                // All items have been reduced to minimum
                break;
            }

            // Apply shrink to non-violated items
            for (i, item) in flex_items.iter_mut().enumerate() {
                if violations[i] {
                    continue;
                }

                if item.flex_shrink == 0.0 {
                    item.final_main_size = item.base_size;
                    continue;
                }

                let scaled_shrink = item.base_size * item.flex_shrink;
                let shrink_ratio = scaled_shrink / total_scaled;
                let shrink_amount = remaining_deficit * shrink_ratio;
                let target_size = item.base_size - shrink_amount;

                // Apply min constraint
                if let Some(min) = item.min_main {
                    if target_size < min {
                        item.final_main_size = min;
                        remaining_deficit -= item.base_size - min;
                        violations[i] = true;
                        has_violations = true;
                    } else {
                        item.final_main_size = target_size;
                    }
                } else {
                    item.final_main_size = target_size.max(0.0);
                }
            }
        }

        // Ensure all items have final sizes set
        for (i, item) in flex_items.iter_mut().enumerate() {
            if !violations[i] || item.final_main_size == 0.0 {
                item.final_main_size = item.final_main_size.max(item.min_main.unwrap_or(0.0));
            }
        }
    }

    /// Handle cross-axis alignment for flex items
    /// Implements align-items and align-self properties
    /// Applies cross-axis alignment to flex children.
    fn handle_cross_axis_alignment(
        &self,
        node: &mut LayoutNode,
        axis: Axis,
        container_cross_size: f32,
        ctx: &LayoutContext,
    ) {
        for child in &mut node.children {
            let align = child
                .style
                .item_style
                .align_self
                .unwrap_or(node.style.align_items);

            if align == crate::AlignItems::Stretch {
                // Check if the child has an explicit cross-axis size
                let should_stretch = match axis {
                    Axis::Horizontal => child.style.size.height == Length::Auto,
                    Axis::Vertical => child.style.size.width == Length::Auto,
                };

                if should_stretch {
                    // Get current main size
                    let current_main =
                        if let LayoutBoxes::Single(ref box_model) = child.layout_boxes {
                            axis.main(&box_model.border_box)
                        } else {
                            0.0
                        };

                    // Stretch cross dimension
                    let (stretch_width, stretch_height) = match axis {
                        Axis::Horizontal => (Some(current_main), Some(container_cross_size)),
                        Axis::Vertical => (Some(container_cross_size), Some(current_main)),
                    };

                    let stretch_ctx = LayoutContext {
                        containing_block_width: stretch_width,
                        containing_block_height: stretch_height,
                        viewport_width: ctx.viewport_width,
                        viewport_height: ctx.viewport_height,
                        parent_assigned_border_width: stretch_width,
                        parent_assigned_border_height: stretch_height,
                    };

                    self.layout_node(child, false, (0.0, 0.0), 0.0, &stretch_ctx);
                }
            }
        }
    }

    /// Create and set box model for a flex container
    /// Creates and sets the box model for a container node.
    fn create_and_set_box_model(
        &self,
        node: &mut LayoutNode,
        params: BoxModelParams,
        ctx: &LayoutContext,
    ) {
        let (content_width, content_height) = match params.axis {
            Axis::Horizontal => (params.content_main, params.content_cross),
            Axis::Vertical => (params.content_cross, params.content_main),
        };

        // Calculate actual children box size including gaps
        let (children_width, children_height) = if node.children.is_empty() {
            (0.0, 0.0)
        } else {
            // Calculate total size of children
            let children_main_total: f32 = node
                .children
                .iter()
                .map(|child| {
                    if let LayoutBoxes::Single(ref box_model) = child.layout_boxes {
                        params.axis.main(&box_model.border_box)
                    } else {
                        0.0
                    }
                })
                .sum();

            // Calculate gaps
            let gap_total = if node.children.len() > 1 {
                let gap = params
                    .axis
                    .gap(&node.style)
                    .resolve_with(
                        ctx.containing_block_main(params.axis),
                        ctx.viewport_main(params.axis),
                    )
                    .unwrap_or(0.0);
                gap * (node.children.len() as f32 - 1.0)
            } else {
                0.0
            };

            let total_children_main = children_main_total + gap_total;

            let children_cross_max: f32 = node
                .children
                .iter()
                .map(|child| {
                    if let LayoutBoxes::Single(ref box_model) = child.layout_boxes {
                        params.axis.cross(&box_model.border_box)
                    } else {
                        0.0
                    }
                })
                .fold(0.0, f32::max);

            match params.axis {
                Axis::Horizontal => (total_children_main, children_cross_max),
                Axis::Vertical => (children_cross_max, total_children_main),
            }
        };

        node.layout_boxes = LayoutBoxes::Single(create_box_model(
            content_width,
            content_height,
            children_width,
            children_height,
            params.padding,
            params.border,
        ));

        if let LayoutBoxes::Single(ref mut box_model) = node.layout_boxes {
            let (pl, pt, _, _) = params.padding;
            let (bl, bt, _, _) = params.border;
            set_position(box_model, params.origin, (pl, pt), (bl, bt));
        }
    }

    /// Positions flex children on main and cross axes.
    /// Applies justify-content and align-items alignment to flex children.
    fn position_flex_children(&self, node: &mut LayoutNode, axis: Axis, ctx: &LayoutContext) {
        if node.children.is_empty() {
            return;
        }

        let content_box = match &node.layout_boxes {
            LayoutBoxes::Single(box_model) => &box_model.content_box,
            _ => return,
        };

        // Resolve gap between flex items
        let gap = axis
            .gap(&node.style)
            .resolve_with(ctx.containing_block_main(axis), ctx.viewport_main(axis))
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
        let has_auto_margins = node.children.iter().any(|child| match axis {
            Axis::Horizontal => {
                child.style.spacing.margin_left == Length::Auto
                    || child.style.spacing.margin_right == Length::Auto
            }
            Axis::Vertical => {
                child.style.spacing.margin_top == Length::Auto
                    || child.style.spacing.margin_bottom == Length::Auto
            }
        });

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
        let mut remaining_auto_space = remaining_space.max(0.0);

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

            if has_auto_margins && remaining_auto_space > 0.0 {
                if margin_start_auto && margin_end_auto {
                    // Both auto: split remaining space equally
                    margin_start = remaining_auto_space / 2.0;
                    margin_end = remaining_auto_space / 2.0;
                    remaining_auto_space = 0.0;
                } else if margin_start_auto {
                    // Only start margin is auto
                    margin_start = remaining_auto_space;
                    remaining_auto_space = 0.0;
                } else if margin_end_auto {
                    // Only end margin is auto
                    margin_end = remaining_auto_space;
                    remaining_auto_space = 0.0;
                }
            }

            cursor_main += margin_start;

            // Position child along main axis
            let child_main_pos = match axis {
                Axis::Horizontal => content_box.x + cursor_main,
                Axis::Vertical => content_box.y + cursor_main,
            };

            // Position child along cross axis (align-items / align-self)
            let child_cross_size = if let LayoutBoxes::Single(ref box_model) = child.layout_boxes {
                axis.cross(&box_model.border_box)
            } else {
                0.0
            };
            let available_cross = axis.cross(content_box);
            let cross_offset = resolve_align_position(
                child
                    .style
                    .item_style
                    .align_self
                    .unwrap_or(node.style.align_items),
                child_cross_size,
                available_cross,
            );

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
            let child_main_size = if let LayoutBoxes::Single(ref box_model) = child.layout_boxes {
                axis.main(&box_model.border_box)
            } else {
                0.0
            };
            cursor_main += child_main_size + margin_end + gap + gap_between;
        }
    }
}

// ===========================
// Helper Functions
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
    padding_edge: (f32, f32, f32, f32),
    border_edge: (f32, f32, f32, f32),
) -> BoxModel {
    let (pl, pt, pr, pb) = padding_edge;
    let (bl, bt, br, bb) = border_edge;

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
    let (min_constraint, max_constraint) = if is_width {
        (
            size_style
                .min_width
                .resolve_with(ctx.containing_block_width, ctx.viewport_width),
            size_style
                .max_width
                .resolve_with(ctx.containing_block_width, ctx.viewport_width),
        )
    } else {
        (
            size_style
                .min_height
                .resolve_with(ctx.containing_block_height, ctx.viewport_height),
            size_style
                .max_height
                .resolve_with(ctx.containing_block_height, ctx.viewport_height),
        )
    };

    clamp(value, min_constraint, max_constraint)
}

fn resolve_margins(spacing: &Spacing, ctx: &LayoutContext) -> (f32, f32, f32, f32) {
    // Percentage margins resolve against containing block width
    let containing_width = ctx.containing_block_width.unwrap_or(ctx.viewport_width);

    (
        spacing
            .margin_left
            .resolve_with(ctx.containing_block_width, ctx.viewport_width)
            .unwrap_or(0.0),
        spacing
            .margin_top
            .resolve_with(Some(containing_width), ctx.viewport_width)
            .unwrap_or(0.0),
        spacing
            .margin_right
            .resolve_with(ctx.containing_block_width, ctx.viewport_width)
            .unwrap_or(0.0),
        spacing
            .margin_bottom
            .resolve_with(Some(containing_width), ctx.viewport_width)
            .unwrap_or(0.0),
    )
}

/// Resolves margins with support for margin collapsing.
///
/// Handles parent-child and sibling margin collapsing for block-level elements.
///
/// # Arguments
/// - `spacing`: The spacing style of the element
/// - `ctx`: The layout context
/// - `is_block`: Whether this is block-level (affects collapsing)
/// - `previous_margin_bottom`: The previous sibling's bottom margin
///
/// # Returns
/// - `resolved_margins`: (margin_left, margin_top, margin_right, margin_bottom)
/// - `margin_after`: The bottom margin for next sibling collapsing
fn resolve_margins_with_collapsing_enhanced(
    spacing: &Spacing,
    ctx: &LayoutContext,
    is_block: bool,
    previous_margin_bottom: f32,
) -> ((f32, f32, f32, f32), f32) {
    let margins = resolve_margins(spacing, ctx);

    // Apply vertical margin collapsing for block-level elements
    let collapsed_margin_top = if is_block {
        margins.1.max(previous_margin_bottom)
    } else {
        margins.1
    };

    (
        (margins.0, collapsed_margin_top, margins.2, margins.3),
        margins.3,
    )
}

/// Computes resolved margins for all children with proper collapsing.
///
/// Resolves margins for all children in a block container, applying:
/// - Sibling collapsing: each child's top margin collapses with previous bottom
/// - Parent-child collapsing: first child's top margin with parent border
///
/// # Returns
/// Vector of resolved margins per child: (margin_left, margin_top, margin_right, margin_bottom)
fn resolve_all_children_margins(
    children: &[LayoutNode],
    containing_block_width: f32,
    viewport_width: f32,
    _viewport_height: f32,
) -> Vec<(f32, f32, f32, f32)> {
    let mut resolved_margins = Vec::with_capacity(children.len());
    let mut previous_margin_bottom = 0.0;

    for (i, child) in children.iter().enumerate() {
        let is_block_level = matches!(child.style.display, Display::Block | Display::Flex { .. });

        // Resolve all four margins
        let raw_margins = (
            child
                .style
                .spacing
                .margin_left
                .resolve_with(Some(containing_block_width), viewport_width)
                .unwrap_or(0.0),
            child
                .style
                .spacing
                .margin_top
                .resolve_with(Some(containing_block_width), viewport_width)
                .unwrap_or(0.0),
            child
                .style
                .spacing
                .margin_right
                .resolve_with(Some(containing_block_width), viewport_width)
                .unwrap_or(0.0),
            child
                .style
                .spacing
                .margin_bottom
                .resolve_with(Some(containing_block_width), viewport_width)
                .unwrap_or(0.0),
        );

        // Apply sibling margin collapsing for block-level children
        let collapsed_margin_top = if is_block_level && i > 0 {
            // Take max of current top and previous sibling's bottom
            raw_margins.1.max(previous_margin_bottom)
        } else {
            raw_margins.1
        };

        let collapsed_margins = (
            raw_margins.0,
            collapsed_margin_top,
            raw_margins.2,
            raw_margins.3,
        );
        resolved_margins.push(collapsed_margins);

        // Update previous margin for next iteration
        if is_block_level {
            previous_margin_bottom = raw_margins.3;
        }
    }

    resolved_margins
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
fn resolve_padding(spacing: &Spacing, ctx: &LayoutContext) -> (f32, f32, f32, f32) {
    let containing_width = ctx.containing_block_width.unwrap_or(ctx.viewport_width);

    (
        spacing
            .padding_left
            .resolve_with(Some(containing_width), ctx.viewport_width)
            .unwrap_or(0.0),
        spacing
            .padding_top
            .resolve_with(Some(containing_width), ctx.viewport_width)
            .unwrap_or(0.0),
        spacing
            .padding_right
            .resolve_with(Some(containing_width), ctx.viewport_width)
            .unwrap_or(0.0),
        spacing
            .padding_bottom
            .resolve_with(Some(containing_width), ctx.viewport_width)
            .unwrap_or(0.0),
    )
}

/// Resolves border values.
/// Percentage values resolve relative to the containing block's width.
fn resolve_border(spacing: &Spacing, ctx: &LayoutContext) -> (f32, f32, f32, f32) {
    // Per CSS spec, percentage borders resolve to containing block width
    let containing_width = ctx.containing_block_width.unwrap_or(ctx.viewport_width);

    (
        spacing
            .border_left
            .resolve_with(ctx.containing_block_width, ctx.viewport_width)
            .unwrap_or(0.0),
        spacing
            .border_top
            .resolve_with(Some(containing_width), ctx.viewport_width)
            .unwrap_or(0.0),
        spacing
            .border_right
            .resolve_with(ctx.containing_block_width, ctx.viewport_width)
            .unwrap_or(0.0),
        spacing
            .border_bottom
            .resolve_with(Some(containing_width), ctx.viewport_width)
            .unwrap_or(0.0),
    )
}
