use crate::{
    AlignItems, Display, FlexDirection, JustifyContent, LayoutNode, Rect, SizeStyle, Spacing, Style,
};

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

    // --- size style ---
    fn size_main(&self, s: &SizeStyle) -> Option<f32> {
        match self {
            Axis::Horizontal => s.width,
            Axis::Vertical => s.height,
        }
    }

    fn size_cross(&self, s: &SizeStyle) -> Option<f32> {
        match self {
            Axis::Horizontal => s.height,
            Axis::Vertical => s.width,
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

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn layout(root: &mut LayoutNode, width: f32, height: f32) {
        let resolved = ResolvedSize {
            width: Some(width),
            height: Some(height),
        };

        Self::layout_size(root, resolved, false);
        Self::layout_position(root, 0.0, 0.0);
    }

    // =========================
    // Size pass
    // =========================

    fn layout_size(node: &mut LayoutNode, resolved: ResolvedSize, self_only: bool) {
        match node.style.display {
            Display::None => {
                node.rect.width = 0.0;
                node.rect.height = 0.0;
            }
            Display::Block => Self::layout_block_size(node, resolved, self_only),
            Display::Flex { flex_direction } => {
                let axis = match flex_direction {
                    FlexDirection::Row => Axis::Horizontal,
                    FlexDirection::Column => Axis::Vertical,
                };
                Self::layout_flex_size(node, axis, resolved, self_only);
            }
        }
    }

    fn layout_block_size(node: &mut LayoutNode, resolved: ResolvedSize, self_only: bool) {
        let s = &node.style.spacing;

        let width = node.style.size.width.or(resolved.width);

        // auto or self_only==false
        let layout_children = width.is_none() || !self_only;

        let inner_width = width.map(|w| (w - s.padding_left - s.padding_right).max(0.0));

        let mut cursor_y = 0.0;
        let mut max_child_width: f32 = 0.0;

        if layout_children {
            for child in &mut node.children {
                let child_resolved = ResolvedSize {
                    width: inner_width,
                    height: None,
                };

                Self::layout_size(child, child_resolved, self_only);

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
            .height
            .or(resolved.height)
            .unwrap_or(cursor_y + s.padding_top + s.padding_bottom);

        node.rect.width = clamp(
            computed_width,
            node.style.size.min_width,
            node.style.size.max_width,
        );
        node.rect.height = clamp(
            computed_height,
            node.style.size.min_height,
            node.style.size.max_height,
        );
    }

    fn layout_flex_size(
        node: &mut LayoutNode,
        axis: Axis,
        resolved: ResolvedSize,
        self_only: bool,
    ) {
        let own_main = axis
            .size_main(&node.style.size)
            .or(axis.resolved_main(&resolved));

        let own_cross = axis
            .size_cross(&node.style.size)
            .or(axis.resolved_cross(&resolved));

        // auto or self_only==false
        let layout_children = (own_main.is_none() || own_cross.is_none()) || !self_only;

        // content box size for compute auto size
        let (content_main, max_cross) = if layout_children {
            Self::layout_flex_children_size(node, axis, resolved, self_only)
        } else {
            (0.0, 0.0)
        };

        let s = &node.style.spacing;

        let final_main = own_main.unwrap_or(content_main + s.padding_left + s.padding_right);
        let final_cross = own_cross.unwrap_or(max_cross + s.padding_top + s.padding_bottom);

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
    ) -> (f32, f32) {
        let s = &node.style.spacing;

        // Parent cross size (content box)
        let parent_cross = axis
            .size_cross(&node.style.size)
            .or(axis.resolved_cross(&resolved))
            .map(|v| v - axis.padding_cross(s))
            .map(|v| v.max(0.0));

        let gap = axis.gap(&node.style).max(0.0);

        // Parent main size (content box)
        let parent_main = axis
            .size_main(&node.style.size)
            .or(axis.resolved_main(&resolved));

        // First pass: measure intrinsic sizes
        let mut total_base_main = 0.0;
        let mut total_flex_grow = 0.0;
        let mut max_cross: f32 = 0.0;

        for child in &mut node.children {
            // Base size: layout with no constraints
            Self::layout_size(child, ResolvedSize::empty(), self_only);

            let base_main = axis.main(&child.rect)
                + child.style.spacing.margin_left
                + child.style.spacing.margin_right;

            total_base_main += base_main;
            total_flex_grow += child.style.item_style.flex_grow;
            max_cross = max_cross.max(axis.cross(&child.rect));
        }

        let gaps = gap * node.children.len().saturating_sub(1) as f32;
        let content_base_main = total_base_main + gaps;

        // Remaining space for flex-grow
        let remaining_main = parent_main
            .map(|m| (m - content_base_main).max(0.0))
            .unwrap_or(0.0);

        // Second pass: apply flex-grow and relayout children
        let mut used_main = 0.0;

        for child in &mut node.children {
            let grow = child.style.item_style.flex_grow;

            let grow_main = if total_flex_grow > 0.0 {
                remaining_main * (grow / total_flex_grow)
            } else {
                0.0
            };

            let align = child
                .style
                .item_style
                .align_self
                .unwrap_or(node.style.align_items);

            let stretch_cross = matches!(align, AlignItems::Stretch)
                && axis.size_cross(&child.style.size).is_none();

            let resolved_child = match axis {
                Axis::Horizontal => ResolvedSize {
                    width: Some(axis.main(&child.rect) + grow_main),
                    height: if stretch_cross { parent_cross } else { None },
                },
                Axis::Vertical => ResolvedSize {
                    width: if stretch_cross { parent_cross } else { None },
                    height: Some(axis.main(&child.rect) + grow_main),
                },
            };

            // Relayout child with grown main size
            Self::layout_size(child, resolved_child, self_only);

            used_main += axis.main(&child.rect)
                + child.style.spacing.margin_left
                + child.style.spacing.margin_right;
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
        let cursor_x = node.style.spacing.padding_left;
        let mut cursor_y = node.style.spacing.padding_top;
        for child in &mut node.children {
            Self::layout_position(child, cursor_x, cursor_y);
            cursor_y += child.rect.height
                + child.style.spacing.margin_top
                + child.style.spacing.margin_bottom;
        }
    }

    fn layout_flex_position(node: &mut LayoutNode, axis: Axis) {
        let s = &node.style.spacing;
        let gap = axis.gap(&node.style).max(0.0);

        let content_main: f32 = node
            .children
            .iter()
            .map(|c| {
                axis.main(&c.rect) + c.style.spacing.margin_left + c.style.spacing.margin_right
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

        let container_cross = axis.cross(&node.rect) - axis.padding_cross(&node.style.spacing);

        let child_len = node.children.len();
        for (i, child) in node.children.iter_mut().enumerate() {
            // main-axis: margin start
            cursor += axis.margin_main_start(&child.style.spacing);

            // cross-axis align
            let cross_offset = resolve_align_position(
                child
                    .style
                    .item_style
                    .align_self
                    .unwrap_or(node.style.align_items),
                axis.cross(&child.rect),
                container_cross,
            );

            match axis {
                Axis::Horizontal => {
                    Self::layout_position(
                        child,
                        s.padding_left + cursor,
                        s.padding_top + cross_offset,
                    );
                }
                Axis::Vertical => {
                    Self::layout_position(
                        child,
                        s.padding_left + cross_offset,
                        s.padding_top + cursor,
                    );
                }
            }

            // main-axis: size + margin end
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
