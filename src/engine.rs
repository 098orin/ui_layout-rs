use crate::{
    AlignItems, Display, FlexDirection, JustifyContent, LayoutNode, Length, Rect, SizeStyle,
    Spacing, Style,
};

struct LayoutContext {
    parent_width: f32,
    parent_height: f32,
    viewport_width: f32,
    viewport_height: f32,
}

/// Size resolved by parent layout pass.
/// None means "auto / unconstrained".
#[derive(Debug, Clone, Copy, Default)]
pub struct ResolvedSize {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl ResolvedSize {
    pub fn empty() -> Self {
        Self {
            width: None,
            height: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    // --- size helpers ---
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

    fn resolved_main(&self, r: &ResolvedSize) -> Option<f32> {
        match self {
            Axis::Horizontal => r.width,
            Axis::Vertical => r.height,
        }
    }

    fn resolved_cross(&self, r: &ResolvedSize) -> Option<f32> {
        match self {
            Axis::Horizontal => r.height,
            Axis::Vertical => r.width,
        }
    }

    // --- padding ---
    fn padding_main(&self, s: &Spacing) -> f32 {
        match self {
            Self::Horizontal => s.padding_left + s.padding_right,
            Self::Vertical => s.padding_top + s.padding_bottom,
        }
    }
    fn padding_cross(&self, s: &Spacing) -> f32 {
        match self {
            Self::Horizontal => s.padding_top + s.padding_bottom,
            Self::Vertical => s.padding_left + s.padding_right,
        }
    }

    // --- margin ---
    fn margin_main_start(&self, s: &Spacing) -> f32 {
        match self {
            Axis::Horizontal => s.margin_left,
            Axis::Vertical => s.margin_top,
        }
    }
    fn margin_main_end(&self, s: &Spacing) -> f32 {
        match self {
            Axis::Horizontal => s.margin_right,
            Axis::Vertical => s.margin_bottom,
        }
    }
    fn margin_cross_start(&self, s: &Spacing) -> f32 {
        match self {
            Axis::Horizontal => s.margin_top,
            Axis::Vertical => s.margin_left,
        }
    }
    fn margin_cross_end(&self, s: &Spacing) -> f32 {
        match self {
            Axis::Horizontal => s.margin_bottom,
            Axis::Vertical => s.margin_right,
        }
    }

    // --- gap ---
    fn gap(&self, style: &Style) -> f32 {
        match self {
            Axis::Horizontal => style.column_gap,
            Axis::Vertical => style.row_gap,
        }
    }
}

impl SizeStyle {
    pub(self) fn resolve_width(&self, ctx: &LayoutContext) -> Option<f32> {
        resolve_length(&self.width, ctx, Axis::Horizontal)
    }

    pub(self) fn resolve_height(&self, ctx: &LayoutContext) -> Option<f32> {
        resolve_length(&self.height, ctx, Axis::Vertical)
    }

    pub(self) fn resolve_min_width(&self, ctx: &LayoutContext) -> Option<f32> {
        resolve_length(&self.min_width, ctx, Axis::Horizontal)
    }

    pub(self) fn resolve_max_width(&self, ctx: &LayoutContext) -> Option<f32> {
        resolve_length(&self.max_width, ctx, Axis::Horizontal)
    }

    pub(self) fn resolve_min_height(&self, ctx: &LayoutContext) -> Option<f32> {
        resolve_length(&self.min_height, ctx, Axis::Vertical)
    }

    pub(self) fn resolve_max_height(&self, ctx: &LayoutContext) -> Option<f32> {
        resolve_length(&self.max_height, ctx, Axis::Vertical)
    }
}

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn layout(root: &mut LayoutNode, width: f32, height: f32) {
        let ctx = LayoutContext {
            parent_width: width,
            parent_height: height,
            viewport_width: width,
            viewport_height: height,
        };

        let resolved = ResolvedSize {
            width: Some(width),
            height: Some(height),
        };

        Self::layout_size(root, resolved, false, &ctx);
        Self::layout_position(root, 0.0, 0.0);
    }

    // =========================
    // Size pass
    // =========================

    fn layout_size(
        node: &mut LayoutNode,
        resolved: ResolvedSize,
        self_only: bool,
        ctx: &LayoutContext,
    ) {
        match node.style.display {
            Display::None => {
                node.rect.width = 0.0;
                node.rect.height = 0.0;
            }
            Display::Block => Self::layout_block_size(node, resolved, self_only, ctx),
            Display::Flex { flex_direction } => {
                let axis = match flex_direction {
                    FlexDirection::Row => Axis::Horizontal,
                    FlexDirection::Column => Axis::Vertical,
                };
                Self::layout_flex_size(node, axis, resolved, self_only, ctx);
            }
        }
    }

    fn layout_block_size(
        node: &mut LayoutNode,
        resolved: ResolvedSize,
        self_only: bool,
        ctx: &LayoutContext,
    ) {
        let s = &node.style.spacing;

        let width = node.style.size.resolve_width(ctx).or(resolved.width);

        // auto or self_only==false
        let layout_children = width.is_none() || !self_only;

        let inner_width = width.map(|w| (w - s.padding_left - s.padding_right).max(0.0));

        let mut cursor_y = 0.0;
        let mut max_child_width: f32 = 0.0;

        if layout_children {
            for child in &mut node.children {
                let child_resolved = ResolvedSize {
                    width: inner_width.map(|w| {
                        (w - child.style.spacing.margin_left - child.style.spacing.margin_right)
                            .max(0.0)
                    }),
                    height: None,
                };

                Self::layout_size(child, child_resolved, self_only, ctx);

                cursor_y += child.rect.height
                    + child.style.spacing.margin_top
                    + child.style.spacing.margin_bottom;

                max_child_width = max_child_width.max(
                    child.rect.width
                        + child.style.spacing.margin_left
                        + child.style.spacing.margin_right,
                );
            }
        }

        let computed_width = width.unwrap_or(max_child_width + s.padding_left + s.padding_right);
        let computed_height = node
            .style
            .size
            .resolve_height(ctx)
            .or(resolved.height)
            .unwrap_or(cursor_y + s.padding_top + s.padding_bottom);

        node.rect.width = clamp(
            computed_width,
            node.style.size.resolve_min_width(ctx),
            node.style.size.resolve_max_width(ctx),
        );
        node.rect.height = clamp(
            computed_height,
            node.style.size.resolve_min_height(ctx),
            node.style.size.resolve_max_height(ctx),
        );
    }

    fn layout_flex_size(
        node: &mut LayoutNode,
        axis: Axis,
        resolved: ResolvedSize,
        self_only: bool,
        ctx: &LayoutContext,
    ) {
        let own_main = match axis {
            Axis::Horizontal => node.style.size.resolve_width(ctx),
            Axis::Vertical => node.style.size.resolve_height(ctx),
        }
        .or(axis.resolved_main(&resolved));

        let own_cross = match axis {
            Axis::Horizontal => node.style.size.resolve_height(ctx),
            Axis::Vertical => node.style.size.resolve_width(ctx),
        }
        .or(axis.resolved_cross(&resolved));

        // auto or self_only==false
        let layout_children = (own_main.is_none() || own_cross.is_none()) || !self_only;

        // content box size for compute auto size
        let (content_main, max_child_cross) = if layout_children {
            Self::layout_flex_children_size(node, axis, resolved, self_only, ctx)
        } else {
            (0.0, 0.0)
        };

        let s = &node.style.spacing;

        let (min_main, max_main) = match axis {
            Axis::Horizontal => (
                node.style.size.resolve_min_width(ctx),
                node.style.size.resolve_max_width(ctx),
            ),
            Axis::Vertical => (
                node.style.size.resolve_min_height(ctx),
                node.style.size.resolve_max_height(ctx),
            ),
        };
        let (min_cross, max_cross) = match axis {
            Axis::Horizontal => (
                node.style.size.resolve_min_height(ctx),
                node.style.size.resolve_max_height(ctx),
            ),
            Axis::Vertical => (
                node.style.size.resolve_min_width(ctx),
                node.style.size.resolve_max_width(ctx),
            ),
        };

        let final_main = clamp(
            own_main.unwrap_or(content_main + s.padding_left + s.padding_right),
            min_main,
            max_main,
        );
        let final_cross = clamp(
            own_cross.unwrap_or(max_child_cross + s.padding_top + s.padding_bottom),
            min_cross,
            max_cross,
        );

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
    fn layout_flex_children_size(
        node: &mut LayoutNode,
        axis: Axis,
        resolved: ResolvedSize,
        self_only: bool,
        ctx: &LayoutContext,
    ) -> (f32, f32) {
        let s = &node.style.spacing;
        let count = node.children.len();

        let parent_cross = match axis {
            Axis::Horizontal => node.style.size.resolve_height(ctx),
            Axis::Vertical => node.style.size.resolve_width(ctx),
        }
        .or(axis.resolved_cross(&resolved))
        .map(|v| (v - axis.padding_cross(s)).max(0.0));

        let gap = axis.gap(&node.style).max(0.0);

        let parent_main = match axis {
            Axis::Horizontal => node.style.size.resolve_width(ctx),
            Axis::Vertical => node.style.size.resolve_height(ctx),
        }
        .or(axis.resolved_main(&resolved))
        .map(|m| (m - axis.padding_main(s)).max(0.0));

        /* ---------- intrinsic pass ---------- */

        let mut main_sizes = vec![0.0; count];
        let mut frozen = vec![false; count];
        let mut max_cross: f32 = 0.0;

        for (i, child) in node.children.iter_mut().enumerate() {
            Self::layout_size(child, ResolvedSize::empty(), true, ctx);

            let basis = child.style.item_style.flex_basis;

            let base_content_main = match basis {
                Some(v) => v,
                None => match axis {
                    Axis::Horizontal => node.style.size.resolve_width(ctx),
                    Axis::Vertical => node.style.size.resolve_height(ctx),
                }
                .unwrap_or_else(|| axis.main(&child.rect)),
            };

            let margin = axis.margin_main_start(&child.style.spacing)
                + axis.margin_main_end(&child.style.spacing);

            main_sizes[i] = base_content_main + margin;

            max_cross = max_cross.max(
                axis.cross(&child.rect)
                    + axis.margin_cross_start(&child.style.spacing)
                    + axis.margin_cross_end(&child.style.spacing),
            );
        }

        let total_base_main: f32 = main_sizes.iter().sum();
        let gaps = gap * count.saturating_sub(1) as f32;

        let mut remaining = parent_main
            .map(|m| (m - total_base_main - gaps).max(0.0))
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
            let mut any_frozen = false;

            for (i, child) in node.children.iter().enumerate() {
                if frozen[i] {
                    continue;
                }

                let grow = child.style.item_style.flex_grow;
                let delta = remaining * (grow / total_grow);

                let (min, max) = match axis {
                    Axis::Horizontal => (
                        child.style.size.resolve_min_width(ctx),
                        child.style.size.resolve_max_width(ctx),
                    ),
                    Axis::Vertical => (
                        child.style.size.resolve_min_height(ctx),
                        child.style.size.resolve_max_height(ctx),
                    ),
                };

                let margin = axis.margin_main_start(&child.style.spacing)
                    + axis.margin_main_end(&child.style.spacing);

                let content = main_sizes[i] - margin;

                let proposed_content = content + delta;
                let clamped_content = clamp(proposed_content, min, max);

                let actual = clamped_content - content;

                main_sizes[i] = clamped_content + margin;
                used += actual;

                if (actual - delta).abs() > 0.0001 {
                    frozen[i] = true;
                    any_frozen = true;
                }
            }

            remaining -= used;

            if !any_frozen {
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

            let is_auto_cross = match axis {
                Axis::Horizontal => matches!(child.style.size.height, Length::Auto),
                Axis::Vertical => matches!(child.style.size.width, Length::Auto),
            };

            let stretched_cross = if matches!(align, AlignItems::Stretch) && is_auto_cross {
                parent_cross.map(|v| {
                    (v - axis.margin_cross_start(&child.style.spacing)
                        - axis.margin_cross_end(&child.style.spacing))
                    .max(0.0)
                })
            } else {
                None
            };

            let resolved_child = match axis {
                Axis::Horizontal => ResolvedSize {
                    width: Some(main_sizes[i]),
                    height: stretched_cross,
                },
                Axis::Vertical => ResolvedSize {
                    width: stretched_cross,
                    height: Some(main_sizes[i]),
                },
            };

            Self::layout_size(child, resolved_child, self_only, ctx);

            used_main += main_sizes[i];
        }

        let content_main = used_main + gaps;
        (content_main, max_cross)
    }

    // =========================
    // Position pass
    // =========================

    fn layout_position(node: &mut LayoutNode, x: f32, y: f32) {
        node.rect.x = x;
        node.rect.y = y;

        match node.style.display {
            Display::None => {}
            Display::Block => {
                Self::layout_block_position(node);
            }
            Display::Flex { flex_direction } => {
                let axis = match flex_direction {
                    FlexDirection::Row => Axis::Horizontal,
                    FlexDirection::Column => Axis::Vertical,
                };
                Self::layout_flex_position(node, axis);
            }
        }
    }

    fn layout_block_position(node: &mut LayoutNode) {
        let s = &node.style.spacing;

        let cursor_x = s.padding_left;
        let mut cursor_y = s.padding_top;

        for child in &mut node.children {
            let x = cursor_x + child.style.spacing.margin_left;
            let y = cursor_y + child.style.spacing.margin_top;

            Self::layout_position(child, x, y);

            cursor_y += child.style.spacing.margin_top
                + child.rect.height
                + child.style.spacing.margin_bottom;
        }
    }

    fn layout_flex_position(node: &mut LayoutNode, axis: Axis) {
        let s = &node.style.spacing;
        let gap = axis.gap(&node.style).max(0.0);

        // === main-axis content size ===
        let content_main: f32 = node
            .children
            .iter()
            .map(|c| {
                axis.main(&c.rect)
                    + axis.margin_main_start(&c.style.spacing)
                    + axis.margin_main_end(&c.style.spacing)
            })
            .sum::<f32>()
            + gap * node.children.len().saturating_sub(1) as f32;

        let inner_main = match axis {
            Axis::Horizontal => node.rect.width - s.padding_left - s.padding_right,
            Axis::Vertical => node.rect.height - s.padding_top - s.padding_bottom,
        }
        .max(0.0);

        let remaining = (inner_main - content_main).max(0.0);

        let (start_offset, justify_gap) =
            resolve_justify_content(node.style.justify_content, remaining, node.children.len());

        let mut cursor = start_offset;

        // === cross-axis container size (content box) ===
        let container_cross = axis.cross(&node.rect) - axis.padding_cross(&node.style.spacing);

        let child_len = node.children.len();

        for (i, child) in node.children.iter_mut().enumerate() {
            // --- main-axis margin start ---
            cursor += axis.margin_main_start(&child.style.spacing);

            // --- cross-axis: margin-inclusive size ---
            let child_cross_outer = axis.cross(&child.rect)
                + axis.margin_cross_start(&child.style.spacing)
                + axis.margin_cross_end(&child.style.spacing);

            let align = child
                .style
                .item_style
                .align_self
                .unwrap_or(node.style.align_items);

            let cross_offset = resolve_align_position(align, child_cross_outer, container_cross);

            let cross_pos = cross_offset + axis.margin_cross_start(&child.style.spacing);

            match axis {
                Axis::Horizontal => {
                    Self::layout_position(
                        child,
                        s.padding_left + cursor,
                        s.padding_top + cross_pos,
                    );
                }
                Axis::Vertical => {
                    Self::layout_position(
                        child,
                        s.padding_left + cross_pos,
                        s.padding_top + cursor,
                    );
                }
            }

            // --- main-axis: size + margin end ---
            cursor += axis.main(&child.rect) + axis.margin_main_end(&child.style.spacing);

            if i + 1 < child_len {
                cursor += gap + justify_gap;
            }
        }
    }
}

// =========================
// Helpers
// =========================

fn resolve_length(len: &Length, ctx: &LayoutContext, axis: Axis) -> Option<f32> {
    match len {
        Length::Auto => None,

        Length::Px(v) => Some(*v),

        Length::Percent(p) => match axis {
            Axis::Horizontal => Some(ctx.parent_width * p / 100.0),
            Axis::Vertical => Some(ctx.parent_height * p / 100.0),
        },

        Length::Vw(v) => Some(ctx.viewport_width * v / 100.0),
        Length::Vh(v) => Some(ctx.viewport_height * v / 100.0),

        Length::Add(a, b) => {
            let x = resolve_length(a, ctx, axis)?;
            let y = resolve_length(b, ctx, axis)?;
            Some(x + y)
        }

        Length::Sub(a, b) => {
            let x = resolve_length(a, ctx, axis)?;
            let y = resolve_length(b, ctx, axis)?;
            Some(x - y)
        }
    }
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

// ======================================
//                  test
// ======================================
#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> LayoutContext {
        LayoutContext {
            parent_width: 200.0,
            parent_height: 100.0,
            viewport_width: 1000.0,
            viewport_height: 500.0,
        }
    }

    // ------------------------
    // basic units
    // ------------------------

    #[test]
    fn resolve_px() {
        let c = ctx();
        let l = Length::Px(42.0);
        assert_eq!(resolve_length(&l, &c, Axis::Horizontal), Some(42.0));
    }

    #[test]
    fn resolve_percent_width() {
        let c = ctx();
        let l = Length::Percent(50.0);
        assert_eq!(resolve_length(&l, &c, Axis::Horizontal), Some(100.0));
    }

    #[test]
    fn resolve_percent_height() {
        let c = ctx();
        let l = Length::Percent(25.0);
        assert_eq!(resolve_length(&l, &c, Axis::Vertical), Some(25.0));
    }

    #[test]
    fn resolve_vw_vh() {
        let c = ctx();
        assert_eq!(
            resolve_length(&Length::Vw(10.0), &c, Axis::Horizontal),
            Some(100.0)
        );
        assert_eq!(
            resolve_length(&Length::Vh(10.0), &c, Axis::Vertical),
            Some(50.0)
        );
    }

    #[test]
    fn resolve_auto() {
        let c = ctx();
        assert_eq!(resolve_length(&Length::Auto, &c, Axis::Horizontal), None);
    }

    // ------------------------
    // calc (add / sub)
    // ------------------------

    #[test]
    fn resolve_add() {
        let c = ctx();
        let l = Length::Add(
            Box::new(Length::Px(10.0)),
            Box::new(Length::Percent(50.0)), // 100
        );
        assert_eq!(resolve_length(&l, &c, Axis::Horizontal), Some(110.0));
    }

    #[test]
    fn resolve_sub() {
        let c = ctx();
        let l = Length::Sub(
            Box::new(Length::Vw(10.0)), // 100
            Box::new(Length::Px(40.0)),
        );
        assert_eq!(resolve_length(&l, &c, Axis::Horizontal), Some(60.0));
    }

    #[test]
    fn auto_propagates_in_calc() {
        let c = ctx();
        let l = Length::Add(Box::new(Length::Auto), Box::new(Length::Px(10.0)));
        assert_eq!(resolve_length(&l, &c, Axis::Horizontal), None);
    }

    // ------------------------
    // semantic correctness
    // ------------------------

    #[test]
    fn auto_is_not_zero_percent() {
        let c = ctx();

        assert_eq!(resolve_length(&Length::Auto, &c, Axis::Horizontal), None);
        assert_eq!(
            resolve_length(&Length::Percent(0.0), &c, Axis::Horizontal),
            Some(0.0)
        );
    }

    // ------------------------
    // SizeStyle resolve helpers
    // ------------------------

    #[test]
    fn size_style_resolve_width() {
        let c = ctx();

        let s = SizeStyle {
            width: Length::Percent(50.0),
            ..Default::default()
        };

        assert_eq!(s.resolve_width(&c), Some(100.0));
    }

    #[test]
    fn size_style_auto() {
        let c = ctx();

        let s = SizeStyle {
            width: Length::Auto,
            ..Default::default()
        };

        assert_eq!(s.resolve_width(&c), None);
    }
}
