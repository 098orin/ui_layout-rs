use crate::{
    AlignItems, BoxModel, BoxSizing, Display, FlexDirection, JustifyContent, LayoutNode, Length,
    Rect, SizeStyle, Spacing, Style, cache,
};

pub(crate) struct LayoutContext {
    pub(crate) containing_block_width: Option<f32>,
    pub(crate) containing_block_height: Option<f32>,
    pub(crate) viewport_width: f32,
    pub(crate) viewport_height: f32,
    pub(crate) forced_border_width: Option<f32>,
    pub(crate) forced_border_height: Option<f32>,
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

    fn forced_border_main(&self, axis: Axis) -> Option<f32> {
        match axis {
            Axis::Horizontal => self.forced_border_width,
            Axis::Vertical => self.forced_border_height,
        }
    }
    fn forced_border_cross(&self, axis: Axis) -> Option<f32> {
        match axis {
            Axis::Horizontal => self.forced_border_height,
            Axis::Vertical => self.forced_border_width,
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

    fn border_main<'a>(&self, s: &'a Spacing) -> (&'a Length, &'a Length) {
        match self {
            Axis::Horizontal => (&s.border_left, &s.border_right),
            Axis::Vertical => (&s.border_top, &s.border_bottom),
        }
    }

    fn border_cross<'a>(&self, s: &'a Spacing) -> (&'a Length, &'a Length) {
        match self {
            Axis::Horizontal => (&s.border_top, &s.border_bottom),
            Axis::Vertical => (&s.border_left, &s.border_right),
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
            containing_block_width: Some(width),
            containing_block_height: Some(height),
            viewport_width: width,
            viewport_height: height,
            forced_border_width: Some(width),
            forced_border_height: Some(height),
        };

        Self::layout_size(root, false, &ctx);
        Self::layout_position(root, (0.0, 0.0), &ctx);
    }

    // =========================
    // Size pass
    // =========================

    fn layout_size(node: &mut LayoutNode, self_only: bool, ctx: &LayoutContext) {
        if self_only {
            let (key, box_model) = &node.box_model_cache;
            if *key == cache::make_layout_key(&ctx) {
                node.box_model = box_model.clone();
                return;
            }
        }

        match node.style.display {
            Display::None => node.box_model = BoxModel::default(),
            Display::Block => Self::layout_block_size(node, self_only, ctx),
            Display::Flex { flex_direction } => {
                let axis = match flex_direction {
                    FlexDirection::Row => Axis::Horizontal,
                    FlexDirection::Column => Axis::Vertical,
                };
                Self::layout_flex_size(node, axis, self_only, ctx);
            }
        }

        if self_only {
            let key = cache::make_layout_key(&ctx);
            node.box_model_cache = (key, node.box_model.clone());
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
        let bl = s.border_left.resolve_with(cbw, vw).unwrap_or(0.0);
        let br = s.border_right.resolve_with(cbw, vw).unwrap_or(0.0);
        let bt = s.border_top.resolve_with(cbh, vh).unwrap_or(0.0);
        let bb = s.border_bottom.resolve_with(cbh, vh).unwrap_or(0.0);

        let specified_width = node.style.size.width.resolve_with(cbw, vw);

        let content_width = match (specified_width, node.style.box_sizing) {
            (Some(w), BoxSizing::BorderBox) => Some((w - pl - pr - bl - br).max(0.0)),
            (Some(w), BoxSizing::ContentBox) => Some(w),
            (None, _) => ctx
                .forced_border_width
                .map(|w| (w - pl - pr - bl - br).max(0.0)),
        };

        let specified_height = node.style.size.height.resolve_with(cbh, vh);

        let content_height = match (specified_height, node.style.box_sizing) {
            (Some(h), BoxSizing::BorderBox) => Some((h - pt - pb - bt - bb).max(0.0)),
            (Some(h), BoxSizing::ContentBox) => Some(h),
            (None, _) => ctx
                .forced_border_height
                .map(|h| (h - pt - pb - bt - bb).max(0.0)),
        };

        // ========================
        // layout children
        // ========================

        let layout_children = !self_only || content_width.is_none() || content_height.is_none();

        let (children_width, children_height) = if layout_children {
            let mut total_child_height = 0.0;
            let mut max_child_width: f32 = 0.0;
            let mut pending_margin_bottom = 0.0;
            for child in &mut node.children {
                // ---- resolve margins ----
                let spacing = &child.style.spacing;

                let ml = spacing.margin_left.resolve_with(content_width, vw);
                let mr = spacing.margin_right.resolve_with(content_width, vw);
                let mt = spacing.margin_top.resolve_with(content_height, vh);
                let mb = spacing.margin_bottom.resolve_with(content_height, vh);

                // ---- build layout context for child ----
                let forced_border_width = content_width.and_then(|w| match (ml, mr) {
                    (Some(ml), Some(mr)) => Some((w - ml - mr).max(0.0)),
                    _ => None,
                });

                let child_ctx = LayoutContext {
                    containing_block_width: content_width,
                    containing_block_height: content_height,
                    viewport_width: vw,
                    viewport_height: vh,
                    forced_border_width,
                    forced_border_height: None,
                };

                // ---- layout child ----
                Self::layout_size(child, self_only, &child_ctx);

                // ---- accumulate sizes ----
                let child_mar_box_height = child.box_model.border_box.height
                    + mt.unwrap_or(0.0).max(pending_margin_bottom);
                total_child_height += child_mar_box_height;

                let child_mar_box_width =
                    child.box_model.border_box.width + ml.unwrap_or(0.0) + mr.unwrap_or(0.0);
                max_child_width = max_child_width.max(child_mar_box_width);

                pending_margin_bottom = mb.unwrap_or(0.0);
            }
            total_child_height += pending_margin_bottom;

            (max_child_width, total_child_height)
        } else {
            (0.0, 0.0)
        };

        // ========================
        // apply
        // ========================
        let mut content_width = content_width.unwrap_or(children_width);
        let mut content_height = content_height.unwrap_or(children_height);

        let padding_w = pl + pr;
        let padding_h = pt + pb;
        let border_w = bl + br;
        let border_h = bt + bb;

        let padding_border_w = padding_w + border_w;
        let padding_border_h = padding_h + border_h;

        match node.style.box_sizing {
            BoxSizing::ContentBox => {
                content_width = clamp(
                    content_width,
                    node.style.size.min_width.resolve_with(cbw, vw),
                    node.style.size.max_width.resolve_with(cbw, vw),
                );
                content_height = clamp(
                    content_height,
                    node.style.size.min_height.resolve_with(cbh, vh),
                    node.style.size.max_height.resolve_with(cbh, vh),
                );
            }

            BoxSizing::BorderBox => {
                let mut border_width = content_width + padding_border_w;
                let mut border_height = content_height + padding_border_h;

                border_width = clamp(
                    border_width,
                    node.style.size.min_width.resolve_with(cbw, vw),
                    node.style.size.max_width.resolve_with(cbw, vw),
                );
                border_height = clamp(
                    border_height,
                    node.style.size.min_height.resolve_with(cbh, vh),
                    node.style.size.max_height.resolve_with(cbh, vh),
                );

                content_width = (border_width - padding_border_w).max(0.0);
                content_height = (border_height - padding_border_h).max(0.0);
            }
        }

        apply_block_box_model(
            node,
            content_width,
            content_height,
            children_width,
            children_height,
            (pl, pt, pr, pb),
            (bl, bt, br, bb),
        );
    }

    fn layout_flex_size(node: &mut LayoutNode, axis: Axis, self_only: bool, ctx: &LayoutContext) {
        let vm = ctx.viewport_main(axis);
        let vc = ctx.viewport_cross(axis);
        let cbm = ctx.containing_block_main(axis);
        let cbc = ctx.containing_block_cross(axis);

        let (pms, pme) = axis.padding_main(&node.style.spacing);
        let (pcs, pce) = axis.padding_cross(&node.style.spacing);
        let (bms, bme) = axis.border_main(&node.style.spacing);
        let (bcs, bce) = axis.border_cross(&node.style.spacing);
        let pms = pms.resolve_with(cbm, vm).unwrap_or(0.0);
        let pme = pme.resolve_with(cbm, vm).unwrap_or(0.0);
        let pcs = pcs.resolve_with(cbc, vc).unwrap_or(0.0);
        let pce = pce.resolve_with(cbc, vc).unwrap_or(0.0);
        let bms = bms.resolve_with(cbm, vm).unwrap_or(0.0);
        let bme = bme.resolve_with(cbm, vm).unwrap_or(0.0);
        let bcs = bcs.resolve_with(cbc, vc).unwrap_or(0.0);
        let bce = bce.resolve_with(cbc, vc).unwrap_or(0.0);

        let specified_main = axis.size_main(&node.style.size).resolve_with(cbm, vm);

        let content_main = match (specified_main, node.style.box_sizing) {
            (Some(m), BoxSizing::BorderBox) => Some((m - pms - pme - bms - bme).max(0.0)),
            (Some(m), BoxSizing::ContentBox) => Some(m),
            (None, _) => ctx
                .forced_border_main(axis)
                .map(|m| (m - pms - pme - bms - bme).max(0.0)),
        };

        let specified_cross = axis.size_cross(&node.style.size).resolve_with(cbc, vc);

        let content_cross = match (specified_cross, node.style.box_sizing) {
            (Some(c), BoxSizing::BorderBox) => Some((c - pcs - pce - bcs - bce).max(0.0)),
            (Some(c), BoxSizing::ContentBox) => Some(c),
            (None, _) => ctx
                .forced_border_cross(axis)
                .map(|c| (c - pcs - pce - bcs - bce).max(0.0)),
        };

        let layout_children = !self_only || content_main.is_none() || content_cross.is_none();

        let (children_main, children_cross) = if layout_children {
            let (content_width, content_height) = match axis {
                Axis::Horizontal => (content_main, content_cross),
                Axis::Vertical => (content_cross, content_main),
            };
            let children_ctx = LayoutContext {
                containing_block_width: content_width,
                containing_block_height: content_height,
                viewport_width: ctx.viewport_width,
                viewport_height: ctx.viewport_height,
                forced_border_width: None,
                forced_border_height: None,
            };
            Self::layout_flex_children_size(node, axis, self_only, &children_ctx)
        } else {
            (0.0, 0.0)
        };

        let min_main = axis.min_main(&node.style.size).resolve_with(cbm, vm);
        let max_main = axis.max_main(&node.style.size).resolve_with(cbm, vm);
        let min_cross = axis.min_cross(&node.style.size).resolve_with(cbc, vc);
        let max_cross = axis.max_cross(&node.style.size).resolve_with(cbc, vc);

        let mut content_main = content_main.unwrap_or(children_main);
        let mut content_cross = content_cross.unwrap_or(children_cross);
        match node.style.box_sizing {
            BoxSizing::ContentBox => {
                content_main = clamp(content_main, min_main, max_main);
                content_cross = clamp(content_cross, min_cross, max_cross);
            }
            BoxSizing::BorderBox => {
                let mut border_main = content_main + pms + pme + bms + bme;
                let mut border_cross = content_cross + pcs + pce + bcs + bce;

                border_main = clamp(border_main, min_main, max_main);
                border_cross = clamp(border_cross, min_cross, max_cross);

                content_main = (border_main - (pms + pme + bms + bme)).max(0.0);
                content_cross = (border_cross - (pcs + pce + bcs + bce)).max(0.0);
            }
        }

        let (content_width, content_height) = match axis {
            Axis::Horizontal => (content_main, content_cross),
            Axis::Vertical => (content_cross, content_main),
        };
        let (children_width, children_height) = match axis {
            Axis::Horizontal => (children_main, children_cross),
            Axis::Vertical => (children_cross, children_main),
        };
        let (pl, pr, pt, pb) = match axis {
            Axis::Horizontal => (pms, pme, pcs, pce),
            Axis::Vertical => (pcs, pce, pms, pme),
        };
        let (bms, bme, bcs, bce) = match axis {
            Axis::Horizontal => (bms, bme, bcs, bce),
            Axis::Vertical => (bcs, bce, bms, bme),
        };

        apply_block_box_model(
            node,
            content_width,
            content_height,
            children_width,
            children_height,
            (pl, pt, pr, pb),
            (bms, bcs, bme, bce),
        );
    }

    /// Layout sizes of flex children.
    /// This method:
    /// 1. Measures base sizes of all children
    /// 2. Distributes remaining space using flex-grow
    /// 3. Calls layout_size for all children with resolved main size
    ///
    /// All of `ctx.forced_border_*` should be None.
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
            .resolve_with(cbm, vm)
            .unwrap_or(0.0)
            .max(0.0);

        /* ---------- intrinsic pass ---------- */

        let mut frozen = vec![false; count];
        let mut total_grow = 0.0;

        let mut main_sizes: Vec<f32> = vec![0.0; node.children.len()];
        let mut main_padding: Vec<(f32, f32)> = vec![(0.0, 0.0); node.children.len()];
        let mut main_border: Vec<(f32, f32)> = vec![(0.0, 0.0); node.children.len()];
        let mut main_margin: Vec<(f32, f32)> = vec![(0.0, 0.0); node.children.len()];
        let mut main_min_max: Vec<(Option<f32>, Option<f32>)> =
            vec![(None, None); node.children.len()];

        for (i, child) in node.children.iter_mut().enumerate() {
            let (pad_start, pad_end) = axis.padding_main(&child.style.spacing);
            main_padding[i] = (
                pad_start.resolve_with(cbm, vm).unwrap_or(0.0),
                pad_end.resolve_with(cbm, vm).unwrap_or(0.0),
            );

            let (border_start, border_end) = axis.border_main(&child.style.spacing);
            main_border[i] = (
                border_start.resolve_with(cbm, vm).unwrap_or(0.0),
                border_end.resolve_with(cbm, vm).unwrap_or(0.0),
            );

            let mar_start = axis.margin_main_start(&child.style.spacing);
            let mar_end = axis.margin_main_end(&child.style.spacing);
            main_margin[i] = (
                mar_start.resolve_with(cbm, vm).unwrap_or(0.0),
                mar_end.resolve_with(cbm, vm).unwrap_or(0.0),
            );

            let min_main = axis.min_main(&child.style.size).resolve_with(cbm, vm);
            let max_main = axis.max_main(&child.style.size).resolve_with(cbm, vm);
            main_min_max[i] = (min_main, max_main);

            let basis = child.style.item_style.flex_basis.resolve_with(cbm, vm);

            let base_content_main = match basis {
                Some(v) => v,
                None => {
                    let size_opt = axis.size_main(&child.style.size).resolve_with(cbm, vm);
                    match size_opt {
                        None => {
                            Self::layout_size(child, true, ctx);
                            axis.main(&child.box_model.content_box)
                        }
                        Some(v) => {
                            frozen[i] = true;
                            v
                        }
                    }
                }
            };

            if !frozen[i] {
                total_grow += child.style.item_style.flex_grow;
                if child.style.item_style.flex_grow == 0.0 {
                    frozen[i] = true;
                }
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
                (m - (total_base_main
                    + gaps
                    + total_main_padding
                    + total_main_border
                    + total_main_margin))
                    .max(0.0)
            })
            .unwrap_or(0.0);

        /* ---------- redistribute loop ---------- */

        loop {
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

                let min_main = main_min_max[i].0;
                let max_main = main_min_max[i].1;

                let proposed_content = main_sizes[i] + delta;
                let clamped_content = match child.style.box_sizing {
                    BoxSizing::ContentBox => clamp(proposed_content, min_main, max_main),
                    BoxSizing::BorderBox => {
                        let padding_border_main = main_padding[i].0
                            + main_padding[i].1
                            + main_border[i].0
                            + main_border[i].1;
                        let proposed_border = proposed_content + padding_border_main;

                        let clamped_border = clamp(proposed_border, min_main, max_main);

                        (clamped_border - padding_border_main).max(0.0)
                    }
                };

                let actual = clamped_content - main_sizes[i];

                main_sizes[i] = clamped_content;
                used += actual;

                if proposed_content != clamped_content {
                    frozen[i] = true;
                    total_grow -= grow;
                }
            }

            remaining -= used;

            if used.abs() < 0.0001 {
                break;
            }
        }

        /* ---------- final layout ---------- */
        let mut total_border_size_main: f32 = 0.0;
        let mut children_max_cross: f32 = 0.0;

        for (i, child) in node.children.iter_mut().enumerate() {
            let align = child
                .style
                .item_style
                .align_self
                .unwrap_or(node.style.align_items);

            let is_auto_cross = axis.size_cross(&child.style.size) == &Length::Auto;

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

            let (forced_border_width, forced_border_height) = {
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
                containing_block_width: ctx.containing_block_width,
                containing_block_height: ctx.containing_block_height,
                viewport_width: ctx.viewport_width,
                viewport_height: ctx.viewport_height,
                forced_border_width,
                forced_border_height,
            };

            Self::layout_size(child, self_only, &child_ctx);

            let mcs = axis
                .margin_cross_start(&child.style.spacing)
                .resolve_with(cbc, vc)
                .unwrap_or(0.0);
            let mce = axis
                .margin_cross_end(&child.style.spacing)
                .resolve_with(cbc, vc)
                .unwrap_or(0.0);

            total_border_size_main += axis.main(&child.box_model.border_box);
            children_max_cross =
                children_max_cross.max(axis.cross(&child.box_model.border_box) + mcs + mce);
        }

        let children_main = total_border_size_main + gaps;

        (children_main, children_max_cross)
    }

    // =========================
    // Position pass
    // =========================

    fn layout_position(node: &mut LayoutNode, border_pos: (f32, f32), ctx: &LayoutContext) {
        let (border_x, border_y) = border_pos;
        node.box_model.border_box.x = border_x;
        node.box_model.border_box.y = border_y;
        node.box_model.padding_box.x = border_x
            + node
                .style
                .spacing
                .border_left
                .resolve_with(ctx.containing_block_width, ctx.viewport_width)
                .unwrap_or(0.0);
        node.box_model.padding_box.y = border_y
            + node
                .style
                .spacing
                .border_top
                .resolve_with(ctx.containing_block_height, ctx.viewport_height)
                .unwrap_or(0.0);
        node.box_model.content_box.x = node.box_model.padding_box.x
            + node
                .style
                .spacing
                .padding_left
                .resolve_with(ctx.containing_block_width, ctx.viewport_width)
                .unwrap_or(0.0);
        node.box_model.content_box.y = node.box_model.padding_box.y
            + node
                .style
                .spacing
                .padding_top
                .resolve_with(ctx.containing_block_height, ctx.viewport_height)
                .unwrap_or(0.0);
        node.box_model.children_box.x = node.box_model.content_box.x;
        node.box_model.children_box.y = node.box_model.content_box.y;

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
        let vw = ctx.viewport_width;
        let vh = ctx.viewport_height;

        let cursor_x = 0.0;
        let mut cursor_y = 0.0;

        let child_cbw = node.box_model.content_box.width;
        let child_cbh = node.box_model.content_box.height;

        let child_ctx = LayoutContext {
            containing_block_width: Some(child_cbw),
            containing_block_height: Some(child_cbh),
            viewport_width: vw,
            viewport_height: vh,
            forced_border_width: None,
            forced_border_height: None,
        };

        let mut pending_margin_bottom = 0.0;

        for child in &mut node.children {
            let child_s = &child.style.spacing;
            let ml_opt = child_s.margin_left.resolve_with(Some(child_cbw), vw);
            let mr_opt = child_s.margin_right.resolve_with(Some(child_cbw), vw);

            let (ml, _mr) = {
                let (ml, mr) = match (ml_opt, mr_opt) {
                    (Some(ml), Some(mr)) => (ml, mr),
                    (Some(ml), None) => (ml, child_cbw - child.box_model.border_box.width - ml),
                    (None, Some(mr)) => (child_cbw - child.box_model.border_box.width - mr, mr),
                    (None, None) => {
                        let m = (child_cbw - child.box_model.border_box.width) / 2.0;
                        (m, m)
                    }
                };
                (ml.max(0.0), mr.max(0.0))
            };

            let x = cursor_x + ml;
            let y = cursor_y
                + child
                    .style
                    .spacing
                    .margin_top
                    .resolve_with(Some(child_cbh), vh)
                    .unwrap_or(0.0)
                    .max(pending_margin_bottom);

            Self::layout_position(child, (x, y), &child_ctx);

            cursor_y += child
                .style
                .spacing
                .margin_top
                .resolve_with(Some(child_cbh), vh)
                .unwrap_or(0.0)
                .max(pending_margin_bottom)
                + child.box_model.border_box.height;

            pending_margin_bottom = child
                .style
                .spacing
                .margin_bottom
                .resolve_with(Some(child_cbh), vh)
                .unwrap_or(0.0);
        }
    }

    fn layout_flex_position(node: &mut LayoutNode, axis: Axis, ctx: &LayoutContext) {
        let vm = ctx.viewport_main(axis);
        let vc = ctx.viewport_cross(axis);
        let cbm = ctx.containing_block_main(axis);
        let cbc = ctx.containing_block_cross(axis);

        let gap = axis
            .gap(&node.style)
            .resolve_with(cbc, vc)
            .unwrap_or(0.0)
            .max(0.0);

        let vw = ctx.viewport_width;
        let vh = ctx.viewport_height;

        let cbm_for_child = axis.main(&node.box_model.children_box);
        let cbc_for_child = axis.cross(&node.box_model.children_box);

        let child_ctx = LayoutContext {
            containing_block_width: Some(node.box_model.content_box.width),
            containing_block_height: Some(node.box_model.content_box.height),
            viewport_width: vw,
            viewport_height: vh,
            forced_border_width: None,
            forced_border_height: None,
        };

        let has_any_auto_margin_main = node.children.iter().any(|child| {
            axis.margin_main_start(&child.style.spacing) == &Length::Auto
                || axis.margin_main_end(&child.style.spacing) == &Length::Auto
        });

        // === total main size ===
        let children_main: f32 = axis.main(&node.box_model.children_box);

        let remaining = cbm.map(|m| (m - children_main).max(0.0)).unwrap_or(0.0);

        // === justify-content ===
        let (start_offset, gap_between) = if has_any_auto_margin_main {
            (0.0, 0.0)
        } else {
            resolve_justify_content(node.style.justify_content, remaining, node.children.len())
        };

        let mut cursor_main = start_offset;

        for child in node.children.iter_mut() {
            let (margin_s, margin_e) = {
                let ms_opt = axis
                    .margin_main_start(&child.style.spacing)
                    .resolve_with(Some(cbm_for_child), vm);
                let me_opt = axis
                    .margin_main_end(&child.style.spacing)
                    .resolve_with(Some(cbm_for_child), vm);

                let (ms, me) = match (ms_opt, me_opt) {
                    (Some(ms), Some(me)) => (ms, me),
                    (Some(ms), None) => (
                        ms,
                        cbm_for_child - axis.main(&child.box_model.border_box) - ms,
                    ),
                    (None, Some(me)) => (
                        cbm_for_child - axis.main(&child.box_model.border_box) - me,
                        me,
                    ),
                    (None, None) => {
                        let m = (cbm_for_child - axis.main(&child.box_model.border_box)) / 2.0;
                        (m, m)
                    }
                };

                (ms.max(0.0), me.max(0.0))
            };

            cursor_main += margin_s;

            // === cross auto margin ===
            let cs_opt = axis
                .margin_cross_start(&child.style.spacing)
                .resolve_with(Some(cbc_for_child), vc);
            let ce_opt = axis
                .margin_cross_end(&child.style.spacing)
                .resolve_with(Some(cbc_for_child), vc);

            let cross_offset = if cs_opt.is_none() || ce_opt.is_none() {
                let (cs, _) = match (cs_opt, ce_opt) {
                    (Some(cs), Some(ce)) => (cs, ce),
                    (Some(cs), None) => (
                        cs,
                        cbc_for_child - axis.cross(&child.box_model.border_box) - cs,
                    ),
                    (None, Some(ce)) => (
                        cbc_for_child - axis.cross(&child.box_model.border_box) - ce,
                        ce,
                    ),
                    (None, None) => {
                        let m = (cbc_for_child - axis.cross(&child.box_model.border_box)) / 2.0;
                        (m, m)
                    }
                };
                cs.max(0.0)
            } else {
                // align-items / align-self
                resolve_align_position(
                    child
                        .style
                        .item_style
                        .align_self
                        .unwrap_or(node.style.align_items),
                    axis.cross(&child.box_model.padding_box),
                    cbc_for_child,
                )
            };

            let (x, y) = match axis {
                Axis::Horizontal => (cursor_main, cross_offset),
                Axis::Vertical => (cross_offset, cursor_main),
            };

            Self::layout_position(child, (x, y), &child_ctx);

            cursor_main += axis.main(&child.box_model.border_box) + margin_e + gap + gap_between;
        }
    }
}

// =========================
// Helpers
// =========================

/// Apply box model to the node
/// # Arguments
/// - node: target layout node
/// - content_width/height: content box size
/// - child_width/height: total size of children
/// - padding_edge: (pl, pt, pr, pb)
/// - border_edge: (bl, bt, br, bb)
fn apply_block_box_model(
    node: &mut LayoutNode,
    content_width: f32,
    content_height: f32,
    children_width: f32,
    children_height: f32,
    padding_edge: (f32, f32, f32, f32),
    border_edge: (f32, f32, f32, f32),
) {
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

    node.box_model = BoxModel {
        content_box,
        padding_box,
        border_box,
        children_box,
    };
}

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
