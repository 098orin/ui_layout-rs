use crate::{
    AlignItems, Display, FlexDirection, JustifyContent, LayoutNode, Length, Rect, SizeStyle,
    Spacing, Style,
};

/// forced_size INCLUDE padding_size
struct LayoutContext {
    containing_block_width: Option<f32>,
    containing_block_height: Option<f32>,
    viewport_width: f32,
    viewport_height: f32,
    forced_width: Option<f32>,
    forced_height: Option<f32>,
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

    fn forced_main(&self, axis: Axis) -> Option<f32> {
        match axis {
            Axis::Horizontal => self.forced_width,
            Axis::Vertical => self.forced_height,
        }
    }
    fn forced_cross(&self, axis: Axis) -> Option<f32> {
        match axis {
            Axis::Horizontal => self.forced_height,
            Axis::Vertical => self.forced_width,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    // =========================
    // Rect access
    // =========================
    fn main(&self, r: &Rect) -> f32 {
        match self {
            Axis::Horizontal => r.width,
            Axis::Vertical => r.height,
        }
    }

    fn cross(&self, r: &Rect) -> f32 {
        match self {
            Axis::Horizontal => r.height,
            Axis::Vertical => r.width,
        }
    }

    // =========================
    // Length accessors
    // =========================
    fn size_main<'a>(&self, s: &'a SizeStyle) -> &'a Length {
        match self {
            Axis::Horizontal => &s.width,
            Axis::Vertical => &s.height,
        }
    }

    fn size_cross<'a>(&self, s: &'a SizeStyle) -> &'a Length {
        match self {
            Axis::Horizontal => &s.height,
            Axis::Vertical => &s.width,
        }
    }

    fn min_main<'a>(&self, s: &'a SizeStyle) -> &'a Length {
        match self {
            Axis::Horizontal => &s.min_width,
            Axis::Vertical => &s.min_height,
        }
    }

    fn max_main<'a>(&self, s: &'a SizeStyle) -> &'a Length {
        match self {
            Axis::Horizontal => &s.max_width,
            Axis::Vertical => &s.max_height,
        }
    }

    fn min_cross<'a>(&self, s: &'a SizeStyle) -> &'a Length {
        match self {
            Axis::Horizontal => &s.min_height,
            Axis::Vertical => &s.min_width,
        }
    }

    fn max_cross<'a>(&self, s: &'a SizeStyle) -> &'a Length {
        match self {
            Axis::Horizontal => &s.max_height,
            Axis::Vertical => &s.max_width,
        }
    }

    // =========================
    // Spacing Length access
    // =========================
    fn padding_main<'a>(&self, s: &'a Spacing) -> (&'a Length, &'a Length) {
        match self {
            Axis::Horizontal => (&s.padding_left, &s.padding_right),
            Axis::Vertical => (&s.padding_top, &s.padding_bottom),
        }
    }

    fn padding_cross<'a>(&self, s: &'a Spacing) -> (&'a Length, &'a Length) {
        match self {
            Axis::Horizontal => (&s.padding_top, &s.padding_bottom),
            Axis::Vertical => (&s.padding_left, &s.padding_right),
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

    // =========================
    // Gap
    // =========================
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
            containing_block_height: Some(height),
            containing_block_width: Some(width),
            viewport_width: width,
            viewport_height: height,
            forced_width: Some(width),
            forced_height: Some(height),
        };

        Self::layout_size(root, false, &ctx);
        Self::layout_position(root, 0.0, 0.0, &ctx);
    }

    // =========================
    // Size pass
    // =========================

    fn layout_size(node: &mut LayoutNode, self_only: bool, ctx: &LayoutContext) {
        match node.style.display {
            Display::None => {
                node.rect.width = 0.0;
                node.rect.height = 0.0;
            }
            Display::Block => Self::layout_block_size(node, self_only, ctx),
            Display::Flex { flex_direction } => {
                let axis = match flex_direction {
                    FlexDirection::Row => Axis::Horizontal,
                    FlexDirection::Column => Axis::Vertical,
                };
                Self::layout_flex_size(node, axis, self_only, ctx);
            }
        }
    }

    fn layout_block_size(node: &mut LayoutNode, self_only: bool, ctx: &LayoutContext) {
        let s = &node.style.spacing;
        let cbw = ctx.containing_block_width;
        let cbh = ctx.containing_block_height;
        let vw = ctx.viewport_width;
        let vh = ctx.viewport_height;

        let pl = s.padding_left.resolve_with(cbw, vw).unwrap_or(0.0);
        let pr = s.padding_right.resolve_with(cbw, vw).unwrap_or(0.0);
        let pt = s.padding_top.resolve_with(cbh, vh).unwrap_or(0.0);
        let pb = s.padding_bottom.resolve_with(cbh, vh).unwrap_or(0.0);
        let ml_opt = s.margin_left.resolve_with(cbw, vw);
        let mr_opt = s.margin_right.resolve_with(cbw, vw);

        let specified_width = node
            .style
            .size
            .width
            .resolve_with(cbw, vw)
            .or(ctx.forced_width.map(|v| v - pl - pr));
        let content_width = match specified_width {
            Some(w) => Some(w),
            None => {
                if let Some(c) = cbw {
                    Some((c - ml_opt.unwrap_or(0.0) - mr_opt.unwrap_or(0.0) - pl - pr).max(0.0))
                } else {
                    None
                }
            }
        };
        let content_height = node
            .style
            .size
            .height
            .resolve_with(cbh, vh)
            .or(ctx.forced_height.map(|v| v - pt - pb));

        // ========================
        // layout children
        // ========================
        let mut total_child_height = 0.0;
        let mut max_child_width: f32 = 0.0;

        let should_layout_children =
            content_width.is_none() || content_height.is_none() || !self_only;

        if should_layout_children {
            for child in &mut node.children {
                // ---- resolve margins ----
                let spacing = &child.style.spacing;

                let ml = spacing.margin_left.resolve_with(content_width, vw);
                let mr = spacing.margin_right.resolve_with(content_width, vw);
                let mt = spacing.margin_top.resolve_with(content_height, vh);
                let mb = spacing.margin_bottom.resolve_with(content_height, vh);

                // ---- build layout context for child ----
                let forced_width = content_width.and_then(|w| match (ml, mr) {
                    (Some(ml), Some(mr)) => Some((w - ml - mr).max(0.0)),
                    _ => None,
                });

                let child_ctx = LayoutContext {
                    containing_block_width: content_width,
                    containing_block_height: content_height,
                    viewport_width: vw,
                    viewport_height: vh,
                    forced_width,
                    forced_height: None,
                };

                // ---- layout child ----
                Self::layout_size(child, self_only, &child_ctx);

                // ---- accumulate sizes ----
                let child_mar_box_height =
                    child.rect.height + mt.unwrap_or(0.0) + mb.unwrap_or(0.0);
                total_child_height += child_mar_box_height;

                let child_mar_box_width = child.rect.width + ml.unwrap_or(0.0) + mr.unwrap_or(0.0);
                max_child_width = max_child_width.max(child_mar_box_width);
            }
        }

        // ========================
        // apply
        // ========================
        let computed_width = content_width.unwrap_or(max_child_width);
        let computed_height = content_height.unwrap_or(total_child_height);

        let final_width = clamp(
            computed_width,
            node.style.size.min_width.resolve_with(cbw, vw),
            node.style.size.max_width.resolve_with(cbw, vw),
        );
        let final_height = clamp(
            computed_height,
            node.style.size.min_height.resolve_with(cbh, vh),
            node.style.size.max_height.resolve_with(cbh, vh),
        );

        node.rect.width = final_width + pl + pr;
        node.rect.height = final_height + pt + pb;
    }

    fn layout_flex_size(node: &mut LayoutNode, axis: Axis, self_only: bool, ctx: &LayoutContext) {
        let vm = ctx.viewport_main(axis);
        let vc = ctx.viewport_cross(axis);
        let cbm = ctx.containing_block_main(axis);
        let cbc = ctx.containing_block_cross(axis);

        let (pms, pme) = axis.padding_main(&node.style.spacing);
        let (pcs, pce) = axis.padding_cross(&node.style.spacing);
        let pms = pms.resolve_with(cbm, vm).unwrap_or(0.0);
        let pme = pme.resolve_with(cbm, vm).unwrap_or(0.0);
        let pcs = pcs.resolve_with(cbc, vc).unwrap_or(0.0);
        let pce = pce.resolve_with(cbc, vc).unwrap_or(0.0);

        let own_main = axis
            .size_main(&node.style.size)
            .resolve_with(cbm, vm)
            .or(ctx.forced_main(axis).map(|v| v - pms - pme));

        let own_cross = axis
            .size_cross(&node.style.size)
            .resolve_with(cbc, vc)
            .or(ctx.forced_cross(axis).map(|v| v - pcs - pce));

        // auto || self_only
        let layout_children = (own_main.is_none() || own_cross.is_none()) || !self_only;

        // content box size for compute auto size
        let (content_main, max_child_cross) = if layout_children {
            let (own_width, own_height) = match axis {
                Axis::Horizontal => (own_main, own_cross),
                Axis::Vertical => (own_cross, own_main),
            };
            let children_ctx = LayoutContext {
                containing_block_width: own_width,
                containing_block_height: own_height,
                viewport_width: ctx.viewport_width,
                viewport_height: ctx.viewport_height,
                forced_width: None,
                forced_height: None,
            };
            Self::layout_flex_children_size(node, axis, self_only, &children_ctx)
        } else {
            (0.0, 0.0)
        };

        let min_main = axis.min_main(&node.style.size).resolve_with(cbm, vm);
        let max_main = axis.max_main(&node.style.size).resolve_with(cbm, vm);
        let min_cross = axis.min_cross(&node.style.size).resolve_with(cbc, vc);
        let max_cross = axis.max_cross(&node.style.size).resolve_with(cbc, vc);

        let final_main = clamp(own_main.unwrap_or(content_main), min_main, max_main) + pms + pme;
        let final_cross =
            clamp(own_cross.unwrap_or(max_child_cross), min_cross, max_cross) + pcs + pce;

        match axis {
            Axis::Horizontal => {
                node.rect.width = final_main;
                node.rect.height = final_cross;
            }
            Axis::Vertical => {
                node.rect.width = final_cross;
                node.rect.height = final_main;
            }
        }
    }

    /// Layout sizes of flex children.
    /// This method:
    /// 1. Measures base sizes of all children
    /// 2. Distributes remaining space using flex-grow
    /// 3. Calls layout_size for all children with resolved main size
    ///
    /// All of `ctx.forced` should be None.
    fn layout_flex_children_size(
        node: &mut LayoutNode,
        axis: Axis,
        self_only: bool,
        ctx: &LayoutContext,
    ) -> (f32, f32) {
        let count = node.children.len();

        let vm = ctx.viewport_main(axis);
        let vc = ctx.viewport_cross(axis);
        let cbm = ctx.containing_block_main(axis);
        let cbc = ctx.containing_block_cross(axis);

        let gap = axis
            .gap(&node.style)
            .resolve_with(cbc, vc)
            .unwrap_or(0.0)
            .max(0.0);

        /* ---------- intrinsic pass ---------- */

        let mut frozen = vec![false; count];

        let mut main_sizes: Vec<f32> = vec![0.0; node.children.len()];
        let mut main_padding: Vec<(f32, f32)> = vec![(0.0, 0.0); node.children.len()];
        let mut main_margin: Vec<(f32, f32)> = vec![(0.0, 0.0); node.children.len()];
        let mut max_cross: f32 = 0.0;

        for (i, child) in node.children.iter_mut().enumerate() {
            Self::layout_size(child, true, ctx);

            let (pad_start, pad_end) = axis.padding_main(&child.style.spacing);
            main_padding[i] = (
                pad_start.resolve_with(cbm, vm).unwrap_or(0.0),
                pad_end.resolve_with(cbm, vm).unwrap_or(0.0),
            );

            let mar_start = axis.margin_main_start(&child.style.spacing);
            let mar_end = axis.margin_main_end(&child.style.spacing);
            main_margin[i] = (
                mar_start.resolve_with(cbm, vm).unwrap_or(0.0),
                mar_end.resolve_with(cbm, vm).unwrap_or(0.0),
            );

            let basis = child.style.item_style.flex_basis.resolve_with(cbm, vm);

            let base_content_main = match basis {
                Some(v) => v,
                None => {
                    let size_opt = axis.size_main(&child.style.size).resolve_with(cbm, vm);
                    match size_opt {
                        None => {
                            if matches!(child.style.display, Display::Block)
                                && matches!(axis, Axis::Horizontal)
                            {
                                0.0
                            } else {
                                axis.main(&child.rect) - main_padding[i].0 - main_padding[i].1
                            }
                        }
                        Some(v) => {
                            frozen[i] = true;
                            v
                        }
                    }
                }
            };

            main_sizes[i] = base_content_main;

            let (pcs, pce) = axis.padding_cross(&child.style.spacing);
            let cross_padding =
                pcs.resolve_with(cbc, vc).unwrap_or(0.0) + pce.resolve_with(cbc, vc).unwrap_or(0.0);

            let cross_size = axis
                .size_cross(&child.style.size)
                .resolve_with(cbc, vc)
                .map(|v| v + cross_padding)
                .unwrap_or(axis.cross(&child.rect));

            let cross_margin = axis
                .margin_cross_start(&child.style.spacing)
                .resolve_with(cbc, vc)
                .unwrap_or(0.0)
                + axis
                    .margin_cross_end(&child.style.spacing)
                    .resolve_with(cbc, vc)
                    .unwrap_or(0.0);

            max_cross = max_cross.max(cross_size + cross_margin);
        }

        let total_base_main: f32 = main_sizes.iter().sum();
        let total_main_padding: f32 = main_padding.iter().map(|(start, end)| start + end).sum();
        let total_main_margin: f32 = main_margin.iter().map(|(start, end)| start + end).sum();
        let gaps = gap * count.saturating_sub(1) as f32;

        let mut remaining = cbm
            .map(|m| {
                (m - (total_base_main + gaps + total_main_padding + total_main_margin)).max(0.0)
            })
            .unwrap_or(0.0);

        /* ---------- redistribute loop ---------- */

        loop {
            let mut total_grow = 0.0;
            for (i, child) in node.children.iter().enumerate() {
                if !frozen[i] {
                    total_grow += child.style.item_style.flex_grow;
                }
            }

            if total_grow == 0.0 {
                break;
            }

            let mut used = 0.0;

            for (i, child) in node.children.iter().enumerate() {
                if frozen[i] {
                    continue;
                }

                let grow = child.style.item_style.flex_grow;
                let delta = remaining * (grow / total_grow);

                let min_main = axis.min_main(&child.style.size).resolve_with(cbm, vm);
                let max_main = axis.max_main(&child.style.size).resolve_with(cbm, vm);

                let proposed_content = main_sizes[i] + delta;
                let clamped_content = clamp(proposed_content, min_main, max_main);

                let actual = clamped_content - main_sizes[i];

                main_sizes[i] = clamped_content;
                used += actual;

                if proposed_content != clamped_content {
                    frozen[i] = true;
                }
            }

            remaining -= used;

            if remaining.abs() < 0.0001 || used.abs() < 0.0001 {
                break;
            }
        }

        /* ---------- final layout ---------- */

        let mut used_main = 0.0;

        for (i, child) in node.children.iter_mut().enumerate() {
            let align = child
                .style
                .item_style
                .align_self
                .unwrap_or(node.style.align_items);

            let is_auto_cross = matches!(axis.size_cross(&child.style.size), Length::Auto);

            let stretched_cross = if matches!(align, AlignItems::Stretch) && is_auto_cross {
                cbc.map(|v| {
                    v - axis
                        .margin_cross_start(&child.style.spacing)
                        .resolve_with(cbc, vc)
                        .unwrap_or(0.0)
                        - axis
                            .margin_cross_end(&child.style.spacing)
                            .resolve_with(cbc, vc)
                            .unwrap_or(0.0)
                })
            } else {
                None
            };

            let (forced_width, forced_height) = match axis {
                Axis::Horizontal => (
                    Some(main_sizes[i] + main_padding[i].0 + main_padding[i].1),
                    stretched_cross,
                ),
                Axis::Vertical => (
                    stretched_cross,
                    Some(main_sizes[i] + main_padding[i].0 + main_padding[i].1),
                ),
            };

            let child_ctx = LayoutContext {
                containing_block_width: ctx.containing_block_width,
                containing_block_height: ctx.containing_block_height,
                viewport_width: ctx.viewport_width,
                viewport_height: ctx.viewport_height,
                forced_width,
                forced_height,
            };

            Self::layout_size(child, self_only, &child_ctx);

            used_main += main_sizes[i];
        }

        let content_main = used_main + total_main_margin + total_main_padding + gaps;

        (content_main, max_cross)
    }

    // =========================
    // Position pass
    // =========================

    fn layout_position(node: &mut LayoutNode, x: f32, y: f32, ctx: &LayoutContext) {
        node.rect.x = x;
        node.rect.y = y;

        match node.style.display {
            Display::None => {}
            Display::Block => {
                Self::layout_block_position(node, ctx);
            }
            Display::Flex { flex_direction } => {
                let axis = match flex_direction {
                    FlexDirection::Row => Axis::Horizontal,
                    FlexDirection::Column => Axis::Vertical,
                };
                Self::layout_flex_position(node, axis, ctx);
            }
        }
    }

    fn layout_block_position(node: &mut LayoutNode, ctx: &LayoutContext) {
        let s = &node.style.spacing;
        let cbw = ctx.containing_block_width.unwrap();
        let cbh = ctx.containing_block_height.unwrap();
        let vw = ctx.viewport_width;
        let vh = ctx.viewport_height;

        let pl = s.padding_left.resolve_with(Some(cbw), vw).unwrap_or(0.0);
        let pr = s.padding_right.resolve_with(Some(cbw), vw).unwrap_or(0.0);
        let pt = s.padding_top.resolve_with(Some(cbh), vh).unwrap_or(0.0);
        let pb = s.padding_bottom.resolve_with(Some(cbh), vh).unwrap_or(0.0);

        let cursor_x = s.padding_left.resolve_with(Some(cbw), vw).unwrap_or(0.0);
        let mut cursor_y = s.padding_top.resolve_with(Some(cbh), vh).unwrap_or(0.0);

        let child_cbw = node.rect.width - pl - pr;
        let child_cbh = node.rect.height - pt - pb;

        let child_ctx = LayoutContext {
            containing_block_width: Some(child_cbw),
            containing_block_height: Some(child_cbh),
            viewport_width: vw,
            viewport_height: vh,
            forced_width: None,
            forced_height: None,
        };

        for child in &mut node.children {
            let child_s = &child.style.spacing;
            let ml_opt = child_s.margin_left.resolve_with(Some(child_cbw), vw);
            let mr_opt = child_s.margin_right.resolve_with(Some(child_cbw), vw);

            let (ml, _mr) = match (ml_opt, mr_opt) {
                (Some(ml), Some(mr)) => (ml, mr),
                (Some(ml), None) => (ml, node.rect.width - ml),
                (None, Some(mr)) => (ctx.containing_block_width.unwrap() - mr, mr),
                (None, None) => {
                    let m = node.rect.width - child.rect.width;
                    (m, m)
                }
            };

            let x = cursor_x + ml;
            let y = cursor_y
                + child
                    .style
                    .spacing
                    .margin_top
                    .resolve_with(Some(child_cbh), vh)
                    .unwrap_or(0.0);

            Self::layout_position(child, x, y, &child_ctx);

            cursor_y += child
                .style
                .spacing
                .margin_top
                .resolve_with(Some(child_cbh), vh)
                .unwrap_or(0.0)
                + child.rect.height
                + child
                    .style
                    .spacing
                    .margin_bottom
                    .resolve_with(Some(child_cbh), vh)
                    .unwrap_or(0.0);
        }
    }

    fn layout_flex_position(node: &mut LayoutNode, axis: Axis, ctx: &LayoutContext) {
        let s = &node.style.spacing;
        let vm = ctx.viewport_main(axis);
        let vc = ctx.viewport_cross(axis);
        let cbm = ctx.containing_block_main(axis);
        let cbc = ctx.containing_block_cross(axis);
        let gap = axis
            .gap(&node.style)
            .resolve_with(cbc, vc)
            .unwrap_or(0.0)
            .max(0.0);

        let cbw = ctx.containing_block_width.unwrap();
        let cbh = ctx.containing_block_height.unwrap();
        let vw = ctx.viewport_width;
        let vh = ctx.viewport_height;

        let pl = s.padding_left.resolve_with(Some(cbw), vw).unwrap_or(0.0);
        let pr = s.padding_right.resolve_with(Some(cbw), vw).unwrap_or(0.0);
        let pt = s.padding_top.resolve_with(Some(cbh), vh).unwrap_or(0.0);
        let pb = s.padding_bottom.resolve_with(Some(cbh), vh).unwrap_or(0.0);

        let child_ctx = LayoutContext {
            containing_block_width: Some(node.rect.width - pl - pr),
            containing_block_height: Some(node.rect.height - pt - pb),
            viewport_width: ctx.viewport_width,
            viewport_height: ctx.viewport_height,
            forced_width: None,
            forced_height: None,
        };

        let total_main: f32 = node
            .children
            .iter()
            .map(|child| {
                axis.main(&child.rect)
                    + axis
                        .margin_main_start(&child.style.spacing)
                        .resolve_with(cbm, vm)
                        .unwrap_or(0.0)
                    + axis
                        .margin_main_end(&child.style.spacing)
                        .resolve_with(cbm, vm)
                        .unwrap_or(0.0)
            })
            .sum::<f32>()
            + gap * (node.children.len().saturating_sub(1) as f32);

        let remaining = cbm.map(|m| (m - total_main).max(0.0)).unwrap_or(0.0);
        let (start_offset, gap_between) =
            resolve_justify_content(node.style.justify_content, remaining, node.children.len());

        let mut cursor_main =
            start_offset + axis.padding_main(&s).0.resolve_with(cbm, vm).unwrap_or(0.0);
        let cursor_cross_padding = axis
            .padding_cross(&s)
            .0
            .resolve_with(cbc, vc)
            .unwrap_or(0.0);

        let (pm, pc) = match axis {
            Axis::Horizontal => (pl + pr, pt + pb),
            Axis::Vertical => (pt + pb, pl + pr),
        };
        let child_cbm = axis.main(&node.rect) - pm;
        let child_cbc = axis.cross(&node.rect) - pc;

        for child in node.children.iter_mut() {
            let margin_s = axis
                .margin_main_start(&child.style.spacing)
                .resolve_with(Some(child_cbm), vm)
                .unwrap_or(0.0);
            cursor_main += margin_s;

            let child_pc = {
                let (pcs, pce) = axis.padding_cross(&child.style.spacing);
                let resolve = |v: &Length| v.resolve_with(Some(child_cbc), vc).unwrap_or(0.0);
                resolve(pcs) + resolve(pce)
            };

            let cross_offset = cursor_cross_padding
                + resolve_align_position(
                    child
                        .style
                        .item_style
                        .align_self
                        .unwrap_or(node.style.align_items),
                    axis.cross(&child.rect) - child_pc,
                    child_cbc,
                );

            let (x, y) = match axis {
                Axis::Horizontal => (cursor_main, cross_offset),
                Axis::Vertical => (cross_offset, cursor_main),
            };
            Self::layout_position(child, x, y, &child_ctx);

            let margin_e = axis
                .margin_main_end(&child.style.spacing)
                .resolve_with(Some(child_cbm), vm)
                .unwrap_or(0.0);

            cursor_main += axis.main(&child.rect) + margin_e + gap + gap_between;
        }
    }
}

// =========================
// Helpers
// =========================

fn clamp(value: f32, min: Option<f32>, max: Option<f32>) -> f32 {
    let v = min.map_or(value, |m| value.max(m));
    max.map_or(v, |m| v.min(m))
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
