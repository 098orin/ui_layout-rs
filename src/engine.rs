use crate::{
    AlignItems, BoxModel, BoxSizing, Display, FlexDirection, FragmentPlacement, ItemFragment,
    JustifyContent, LayoutBoxes, LayoutNode, Length, Rect, Spacing, Style,
};

pub(crate) struct LayoutContext {
    pub(crate) containing_block_width: Option<f32>,
    pub(crate) containing_block_height: Option<f32>,
    pub(crate) viewport_width: f32,
    pub(crate) viewport_height: f32,
    pub(crate) parent_assigned_border_width: Option<f32>,
    pub(crate) parent_assigned_border_height: Option<f32>,
}

impl LayoutContext {
    fn containing_block_main(&self, axis: Axis) -> Option<f32> {
        match axis {
            Axis::Horizontal => self.containing_block_width,
            Axis::Vertical => self.containing_block_height,
        }
    }

    fn containing_block_cross(&self, axis: Axis) -> Option<f32> {
        match axis {
            Axis::Horizontal => self.containing_block_height,
            Axis::Vertical => self.containing_block_width,
        }
    }

    fn viewport_main(&self, axis: Axis) -> f32 {
        match axis {
            Axis::Horizontal => self.viewport_width,
            Axis::Vertical => self.viewport_height,
        }
    }

    fn viewport_cross(&self, axis: Axis) -> f32 {
        match axis {
            Axis::Horizontal => self.viewport_height,
            Axis::Vertical => self.viewport_width,
        }
    }

    fn parent_assigned_border_main(&self, axis: Axis) -> Option<f32> {
        match axis {
            Axis::Horizontal => self.parent_assigned_border_width,
            Axis::Vertical => self.parent_assigned_border_height,
        }
    }

    fn parent_assigned_border_cross(&self, axis: Axis) -> Option<f32> {
        match axis {
            Axis::Horizontal => self.parent_assigned_border_height,
            Axis::Vertical => self.parent_assigned_border_width,
        }
    }
}

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

    fn margin_main_start<'a>(&self, spacing: &'a Spacing) -> &'a Length {
        match self {
            Axis::Horizontal => &spacing.margin_left,
            Axis::Vertical => &spacing.margin_top,
        }
    }

    fn margin_main_end<'a>(&self, spacing: &'a Spacing) -> &'a Length {
        match self {
            Axis::Horizontal => &spacing.margin_right,
            Axis::Vertical => &spacing.margin_bottom,
        }
    }

    fn margin_cross_start<'a>(&self, spacing: &'a Spacing) -> &'a Length {
        match self {
            Axis::Horizontal => &spacing.margin_top,
            Axis::Vertical => &spacing.margin_left,
        }
    }

    fn margin_cross_end<'a>(&self, spacing: &'a Spacing) -> &'a Length {
        match self {
            Axis::Horizontal => &spacing.margin_bottom,
            Axis::Vertical => &spacing.margin_right,
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
    pub fn layout(root: &mut LayoutNode, width: f32, height: f32) {
        let ctx = LayoutContext {
            containing_block_width: Some(width),
            containing_block_height: Some(height),
            viewport_width: width,
            viewport_height: height,
            parent_assigned_border_width: Some(width),
            parent_assigned_border_height: Some(height),
        };

        Self::layout_node(root, false, (0.0, 0.0), 0.0, &ctx);
    }

    fn layout_node(
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
                Self::layout_unified_flow(node, intrinsic_pass, origin, incoming_line_height, ctx)
            }
        };

        if intrinsic_pass {
            let key = crate::cache::make_layout_key(ctx);
            node.layout_boxes_cache = (key, (node.layout_boxes.clone(), out));
        }

        out
    }

    /// Unified Flow layout: handles Block, Inline, and Flex layouts
    fn layout_unified_flow(
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
                Self::layout_flex_as_flow(
                    node,
                    axis,
                    intrinsic_pass,
                    origin,
                    incoming_line_height,
                    ctx,
                )
            }
            Display::Block => {
                Self::layout_block_flow(node, intrinsic_pass, origin, incoming_line_height, ctx)
            }
            Display::Inline => {
                Self::layout_inline_flow(node, intrinsic_pass, origin, incoming_line_height, ctx)
            }
            Display::None => unreachable!(),
        }
    }

    /// Handle Flex container as Flow layout
    fn layout_flex_as_flow(
        node: &mut LayoutNode,
        axis: Axis,
        intrinsic_pass: bool,
        origin: (f32, f32),
        _incoming_line_height: f32,
        ctx: &LayoutContext,
    ) -> ((f32, f32), f32) {
        // Determine Flex container size
        let (content_main, content_cross, padding, border) =
            Self::resolve_container_sizes(node, axis, ctx);

        // Execute layout for child elements
        let (children_main, children_cross) = if !intrinsic_pass
            || content_main.is_none()
            || content_cross.is_none()
        {
            Self::layout_flex_children(node, axis, intrinsic_pass, content_main, content_cross, ctx)
        } else {
            (0.0, 0.0)
        };

        // Determine final container size
        let final_content_main = content_main.unwrap_or(children_main);
        let final_content_cross = content_cross.unwrap_or(children_cross);

        // Create box model
        Self::create_and_set_box_model(
            node,
            final_content_main,
            final_content_cross,
            children_main,
            children_cross,
            axis,
            origin,
            padding,
            border,
        );

        // Set child positions (only in non-intrinsic pass)
        if !intrinsic_pass {
            Self::position_flex_children(node, axis, ctx);
        }

        let end_x = origin.0 + node.layout_boxes.width();
        let end_y = origin.1 + node.layout_boxes.height();
        ((end_x, end_y), node.layout_boxes.height())
    }

    /// Block layout
    fn layout_block_flow(
        node: &mut LayoutNode,
        intrinsic_pass: bool,
        origin: (f32, f32),
        _incoming_line_height: f32,
        ctx: &LayoutContext,
    ) -> ((f32, f32), f32) {
        let mut cursor_y = origin.1;

        // Resolve node's own size first
        let mut content_width = node
            .style
            .size
            .width
            .resolve_with(ctx.containing_block_width, ctx.viewport_width)
            .unwrap_or_else(|| ctx.containing_block_width.unwrap_or(ctx.viewport_width));

        let mut content_height = node
            .style
            .size
            .height
            .resolve_with(ctx.containing_block_height, ctx.viewport_height);

        // Resolve padding, border, and margins for constraint calculations
        let ctx_with_width = LayoutContext {
            containing_block_width: Some(content_width),
            containing_block_height: content_height,
            ..*ctx
        };
        let padding = resolve_padding(&node.style.spacing, &ctx_with_width);
        let border = resolve_border(&node.style.spacing, &ctx_with_width);
        let margins = resolve_margins(&node.style.spacing, &ctx_with_width);

        // Arrange children vertically
        for child in &mut node.children {
            let margin_left_auto = child.style.spacing.margin_left == Length::Auto;
            let margin_right_auto = child.style.spacing.margin_right == Length::Auto;

            let child_x = if margin_left_auto && margin_right_auto {
                // Auto margin centering
                let child_width = child
                    .style
                    .size
                    .width
                    .resolve_with(Some(content_width), ctx.viewport_width)
                    .unwrap_or(0.0);
                origin.0 + (content_width - child_width) / 2.0
            } else {
                origin.0 // Let child handle its own margins
            };

            let ((_, child_end_y), _) = Self::layout_node(
                child,
                intrinsic_pass,
                (child_x, cursor_y),
                0.0,
                &LayoutContext {
                    containing_block_width: Some(content_width),
                    containing_block_height: None,
                    ..*ctx
                },
            );
            cursor_y = child_end_y;
        }

        // Apply min/max constraints to content size
        content_width = apply_size_constraints(
            content_width,
            &node.style.size,
            ctx,
            true, // is_width
        );

        if content_height.is_none() {
            content_height = Some(cursor_y - origin.1);
        }
        let mut final_content_height = content_height.unwrap_or(0.0);
        final_content_height = apply_size_constraints(
            final_content_height,
            &node.style.size,
            ctx,
            false, // is_height
        );

        // Apply box-sizing logic
        let (final_content_width, final_content_height) = match node.style.box_sizing {
            BoxSizing::ContentBox => (content_width, final_content_height),
            BoxSizing::BorderBox => {
                let content_w =
                    (content_width - padding.0 - padding.2 - border.0 - border.2).max(0.0);
                let content_h =
                    (final_content_height - padding.1 - padding.3 - border.1 - border.3).max(0.0);
                (content_w, content_h)
            }
        };

        // Create box model
        node.layout_boxes = LayoutBoxes::Single(create_box_model(
            final_content_width,
            final_content_height,
            final_content_width,
            final_content_height,
            padding,
            border,
        ));

        if let LayoutBoxes::Single(ref mut box_model) = node.layout_boxes {
            let pos_x = origin.0 + margins.0; // margin_left
            let pos_y = origin.1 + margins.1; // margin_top
            set_position(
                box_model,
                (pos_x, pos_y),
                (padding.0, padding.1),
                (border.0, border.1),
            );
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

    /// Inline layout
    fn layout_inline_flow(
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

        // Resolve inline element's spacing
        let ctx_for_inline = LayoutContext {
            containing_block_width: ctx.containing_block_width,
            containing_block_height: ctx.containing_block_height,
            ..*ctx
        };
        let padding = resolve_padding(&node.style.spacing, &ctx_for_inline);
        let border = resolve_border(&node.style.spacing, &ctx_for_inline);
        let margins = resolve_margins(&node.style.spacing, &ctx_for_inline);

        if !intrinsic_pass {
            node.placements.clear();
        }

        // Fragment layout
        if !node.self_fragments.is_empty() {
            // Calculate content area for fragments
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
                    Self::layout_fragment(
                        frag,
                        &mut cursor_x,
                        &mut cursor_y,
                        &mut line_height,
                        max_width,
                        &mut node.placements,
                        &mut line_index,
                        (content_start_x, content_start_y),
                    );
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
        } else {
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

        // Inline layout for child elements
        for child in &mut node.children {
            match child.style.display {
                Display::Inline => {
                    let ((next_x, next_y), next_lh) = Self::layout_node(
                        child,
                        intrinsic_pass,
                        (cursor_x, cursor_y),
                        line_height,
                        ctx,
                    );
                    cursor_x = next_x;
                    cursor_y = next_y;
                    line_height = next_lh;
                }
                Display::Block | Display::Flex { .. } => {
                    // Block elements force line break
                    if cursor_x > origin.0 {
                        cursor_x = origin.0;
                        cursor_y += line_height;
                        line_height = 0.0;
                        line_index += 1;
                    }

                    let ((_, child_end_y), _) =
                        Self::layout_node(child, intrinsic_pass, (origin.0, cursor_y), 0.0, ctx);

                    cursor_y = child_end_y;
                    cursor_x = origin.0;
                    line_height = 0.0;
                }
                Display::None => {}
            }
        }

        // Container inline layout - handle mixed content
        let container_height = cursor_y - origin.1 + line_height;

        node.layout_boxes = LayoutBoxes::Single(create_box_model(
            cursor_x - origin.0,
            container_height,
            cursor_x - origin.0,
            container_height,
            (0.0, 0.0, 0.0, 0.0),
            (0.0, 0.0, 0.0, 0.0),
        ));

        if let LayoutBoxes::Single(ref mut box_model) = node.layout_boxes {
            set_position(box_model, origin, (0.0, 0.0), (0.0, 0.0));
        }

        ((cursor_x, cursor_y), line_height)
    }

    fn layout_fragment(
        frag: &ItemFragment,
        cursor_x: &mut f32,
        cursor_y: &mut f32,
        line_height: &mut f32,
        max_width: f32,
        placements: &mut Vec<FragmentPlacement>,
        line_index: &mut usize,
        origin: (f32, f32),
    ) {
        match frag {
            ItemFragment::LineBreak => {
                *cursor_x = origin.0;
                *cursor_y += *line_height;
                *line_height = 0.0;
                *line_index += 1;

                placements.push(FragmentPlacement {
                    offset: (*cursor_x - origin.0, *cursor_y - origin.1),
                    line_index: *line_index,
                });
            }
            ItemFragment::Fragment(f) => {
                // Check for line wrapping
                if *cursor_x + f.width > max_width && *cursor_x > origin.0 {
                    *cursor_x = origin.0;
                    *cursor_y += *line_height;
                    *line_height = 0.0;
                    *line_index += 1;
                }

                placements.push(FragmentPlacement {
                    offset: (*cursor_x - origin.0, *cursor_y - origin.1),
                    line_index: *line_index,
                });

                *cursor_x += f.width;
                *line_height = line_height.max(f.height);
            }
        }
    }

    /// Resolve Flex container sizes
    fn resolve_container_sizes(
        node: &LayoutNode,
        axis: Axis,
        ctx: &LayoutContext,
    ) -> (
        Option<f32>,
        Option<f32>,
        (f32, f32, f32, f32),
        (f32, f32, f32, f32),
    ) {
        let vm = ctx.viewport_main(axis);
        let vc = ctx.viewport_cross(axis);
        let cbm = ctx.containing_block_main(axis);
        let cbc = ctx.containing_block_cross(axis);

        // Resolve padding and border
        let (pms_len, pme_len) = axis.padding_main(&node.style.spacing);
        let (pcs_len, pce_len) = axis.padding_cross(&node.style.spacing);
        let (bms_len, bme_len) = axis.border_main(&node.style.spacing);
        let (bcs_len, bce_len) = axis.border_cross(&node.style.spacing);

        let pms = pms_len.resolve_with(cbm, vm).unwrap_or(0.0);
        let pme = pme_len.resolve_with(cbm, vm).unwrap_or(0.0);
        let pcs = pcs_len.resolve_with(cbc, vc).unwrap_or(0.0);
        let pce = pce_len.resolve_with(cbc, vc).unwrap_or(0.0);
        let bms = bms_len.resolve_with(cbm, vm).unwrap_or(0.0);
        let bme = bme_len.resolve_with(cbm, vm).unwrap_or(0.0);
        let bcs = bcs_len.resolve_with(cbc, vc).unwrap_or(0.0);
        let bce = bce_len.resolve_with(cbc, vc).unwrap_or(0.0);

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
    fn layout_flex_children(
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

        let child_ctx = LayoutContext {
            containing_block_width: content_width_hint,
            containing_block_height: content_height_hint,
            viewport_width: ctx.viewport_width,
            viewport_height: ctx.viewport_height,
            parent_assigned_border_width: content_width_hint,
            parent_assigned_border_height: content_height_hint,
        };

        let mut total_main = 0.0;
        let mut max_cross: f32 = 0.0;
        let mut flex_items = Vec::new();
        let mut fixed_items_main = 0.0;

        // First phase: layout children with intrinsic sizes
        for (i, child) in node.children.iter_mut().enumerate() {
            // Use basic layout context first
            let basic_ctx = LayoutContext {
                containing_block_width: content_width_hint,
                containing_block_height: content_height_hint,
                viewport_width: ctx.viewport_width,
                viewport_height: ctx.viewport_height,
                parent_assigned_border_width: None,
                parent_assigned_border_height: None,
            };

            let ((_, _), _) = Self::layout_node(child, true, (0.0, 0.0), 0.0, &basic_ctx);

            let mut child_main = if let LayoutBoxes::Single(ref box_model) = child.layout_boxes {
                axis.main(&box_model.border_box)
            } else {
                0.0
            };
            let child_cross = if let LayoutBoxes::Single(ref box_model) = child.layout_boxes {
                axis.cross(&box_model.border_box)
            } else {
                0.0
            };

            // Apply min/max constraints to child size
            let min_main = axis
                .min_main(&child.style.size)
                .resolve_with(content_width_hint, ctx.viewport_main(axis));
            let max_main = axis
                .max_main(&child.style.size)
                .resolve_with(content_width_hint, ctx.viewport_main(axis));
            child_main = clamp(child_main, min_main, max_main);

            if child.style.item_style.flex_grow > 0.0 {
                flex_items.push((i, child_main));
            } else {
                fixed_items_main += child_main;
            }

            max_cross = max_cross.max(child_cross);
        }

        total_main = fixed_items_main;

        // Flex grow processing
        if let Some(available_main) = container_main {
            let total_flex_grow: f32 = flex_items
                .iter()
                .map(|(i, _)| node.children[*i].style.item_style.flex_grow)
                .sum();

            if total_flex_grow > 0.0 && !flex_items.is_empty() {
                let remaining_space = (available_main - fixed_items_main).max(0.0);

                for (i, _) in &flex_items {
                    let child = &node.children[*i];
                    let flex_grow = child.style.item_style.flex_grow;
                    let mut flex_size = remaining_space * flex_grow / total_flex_grow;

                    // Apply min/max constraints to flex size
                    let min_main = axis
                        .min_main(&child.style.size)
                        .resolve_with(content_width_hint, ctx.viewport_main(axis));
                    let max_main = axis
                        .max_main(&child.style.size)
                        .resolve_with(content_width_hint, ctx.viewport_main(axis));
                    flex_size = clamp(flex_size, min_main, max_main);

                    // Update child with new flex size
                    let (new_width, new_height) = match axis {
                        Axis::Horizontal => (Some(flex_size), content_height_hint),
                        Axis::Vertical => (content_width_hint, Some(flex_size)),
                    };

                    let flex_ctx = LayoutContext {
                        containing_block_width: new_width,
                        containing_block_height: new_height,
                        viewport_width: ctx.viewport_width,
                        viewport_height: ctx.viewport_height,
                        parent_assigned_border_width: new_width,
                        parent_assigned_border_height: new_height,
                    };

                    Self::layout_node(&mut node.children[*i], true, (0.0, 0.0), 0.0, &flex_ctx);
                }
                total_main = available_main;
            } else {
                total_main = fixed_items_main;
            }
        }

        // Apply stretch for cross axis
        {
            let container_cross_size = container_cross.unwrap_or(max_cross);
            for child in &mut node.children {
                let align = child
                    .style
                    .item_style
                    .align_self
                    .unwrap_or(node.style.align_items);
                if align == crate::AlignItems::Stretch {
                    // Check if the child has an explicit cross-axis size (not Auto)
                    let should_stretch = match axis {
                        Axis::Horizontal => child.style.size.height == Length::Auto,
                        Axis::Vertical => child.style.size.width == Length::Auto,
                    };

                    // Only stretch if the child has Auto cross-axis size
                    if should_stretch {
                        // Get current main size of the child after flex grow
                        let current_main =
                            if let LayoutBoxes::Single(ref box_model) = child.layout_boxes {
                                axis.main(&box_model.border_box)
                            } else {
                                0.0
                            };

                        // For stretch, force the cross dimension to match container
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

                        Self::layout_node(child, false, (0.0, 0.0), 0.0, &stretch_ctx);

                        // Update max_cross with the actual stretched size
                        if let LayoutBoxes::Single(ref box_model) = child.layout_boxes {
                            let child_cross = axis.cross(&box_model.border_box);
                            max_cross = max_cross.max(child_cross);
                        }
                    }
                }
            }
        }

        // Calculate gap for children_box width/height
        let gap_total = if node.children.len() > 1 {
            let gap = axis
                .gap(&node.style)
                .resolve_with(container_main, ctx.viewport_main(axis))
                .unwrap_or(0.0);
            gap * (node.children.len() as f32 - 1.0)
        } else {
            0.0
        };

        (gap_total, max_cross)
    }

    /// Create and set box model
    fn create_and_set_box_model(
        node: &mut LayoutNode,
        content_main: f32,
        content_cross: f32,
        children_main: f32,
        children_cross: f32,
        axis: Axis,
        origin: (f32, f32),
        padding: (f32, f32, f32, f32),
        border: (f32, f32, f32, f32),
    ) {
        let (content_width, content_height) = match axis {
            Axis::Horizontal => (content_main, content_cross),
            Axis::Vertical => (content_cross, content_main),
        };

        let (children_width, children_height) = match axis {
            Axis::Horizontal => (children_main, children_cross),
            Axis::Vertical => (children_cross, children_main),
        };

        node.layout_boxes = LayoutBoxes::Single(create_box_model(
            content_width,
            content_height,
            children_width,
            children_height,
            padding,
            border,
        ));

        if let LayoutBoxes::Single(ref mut box_model) = node.layout_boxes {
            let (pl, pt, _, _) = padding;
            let (bl, bt, _, _) = border;
            set_position(box_model, origin, (pl, pt), (bl, bt));
        }
    }

    /// Set positions of Flex child elements
    fn position_flex_children(node: &mut LayoutNode, axis: Axis, ctx: &LayoutContext) {
        if node.children.is_empty() {
            return;
        }

        let content_box = match &node.layout_boxes {
            LayoutBoxes::Single(box_model) => &box_model.content_box,
            _ => return,
        };

        // Calculate gaps
        let gap = axis
            .gap(&node.style)
            .resolve_with(ctx.containing_block_main(axis), ctx.viewport_main(axis))
            .unwrap_or(0.0);

        // Calculate total size of children
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

        let gaps_total = if node.children.len() > 1 {
            gap * (node.children.len() as f32 - 1.0)
        } else {
            0.0
        };

        let remaining_space = axis.main(content_box) - children_main_total - gaps_total;

        // Determine start position and gap based on justify-content
        let (start_offset, gap_between) = resolve_justify_content(
            node.style.justify_content,
            remaining_space.max(0.0),
            node.children.len(),
        );

        // Position child elements
        let mut cursor_main = start_offset;

        for child in &mut node.children {
            // Calculate main axis position
            let child_main_pos = match axis {
                Axis::Horizontal => content_box.x + cursor_main,
                Axis::Vertical => content_box.y + cursor_main,
            };

            // Calculate cross axis position (based on align-items)
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

            // Set child element position
            let child_origin = match axis {
                Axis::Horizontal => (child_main_pos, child_cross_pos),
                Axis::Vertical => (child_cross_pos, child_main_pos),
            };

            // Update child layout boxes
            child.layout_boxes.shift(child_origin.0, child_origin.1);

            // Advance cursor
            let child_main_size = if let LayoutBoxes::Single(ref box_model) = child.layout_boxes {
                axis.main(&box_model.border_box)
            } else {
                0.0
            };
            cursor_main += child_main_size + gap + gap_between;
        }
    }
}

// Helper functions

/// Create a box model
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

fn clamp(value: f32, min: Option<f32>, max: Option<f32>) -> f32 {
    let v = min.map_or(value, |m| value.max(m));
    max.map_or(v, |m| v.min(m))
}

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
    (
        spacing
            .margin_left
            .resolve_with(ctx.containing_block_width, ctx.viewport_width)
            .unwrap_or(0.0),
        spacing
            .margin_top
            .resolve_with(ctx.containing_block_height, ctx.viewport_height)
            .unwrap_or(0.0),
        spacing
            .margin_right
            .resolve_with(ctx.containing_block_width, ctx.viewport_width)
            .unwrap_or(0.0),
        spacing
            .margin_bottom
            .resolve_with(ctx.containing_block_height, ctx.viewport_height)
            .unwrap_or(0.0),
    )
}

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

fn resolve_align_position(align: AlignItems, size: f32, container: f32) -> f32 {
    let free = container - size;

    match align {
        AlignItems::Start | AlignItems::Stretch => 0.0,
        AlignItems::Center => free / 2.0,
        AlignItems::End => free,
    }
}

fn resolve_padding(spacing: &Spacing, ctx: &LayoutContext) -> (f32, f32, f32, f32) {
    // For percentage padding, CSS spec says it's always relative to the containing block's width
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

fn resolve_border(spacing: &Spacing, ctx: &LayoutContext) -> (f32, f32, f32, f32) {
    (
        spacing
            .border_left
            .resolve_with(ctx.containing_block_width, ctx.viewport_width)
            .unwrap_or(0.0),
        spacing
            .border_top
            .resolve_with(ctx.containing_block_height, ctx.viewport_height)
            .unwrap_or(0.0),
        spacing
            .border_right
            .resolve_with(ctx.containing_block_width, ctx.viewport_width)
            .unwrap_or(0.0),
        spacing
            .border_bottom
            .resolve_with(ctx.containing_block_height, ctx.viewport_height)
            .unwrap_or(0.0),
    )
}
