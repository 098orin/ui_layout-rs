use crate::{
    AlignItems, Display, FlexDirection, JustifyContent, LayoutNode, Rect, SizeStyle, Spacing, Style,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct ResolvingSize {
    pub width: Option<f32>,
    pub height: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
enum Axis {
    Horizontal, // row
    Vertical,   // column
}

impl Axis {
    // --- size helpers ---
    fn main(&self, r: &Rect) -> f32 {
        match self {
            Self::Horizontal => r.width,
            Self::Vertical => r.height,
        }
    }
    fn cross(&self, r: &Rect) -> f32 {
        match self {
            Self::Horizontal => r.height,
            Self::Vertical => r.width,
        }
    }
    fn main_available(&self, available: ResolvingSize) -> Option<f32> {
        match self {
            Self::Horizontal => available.width,
            Self::Vertical => available.height,
        }
    }
    fn cross_available(&self, available: ResolvingSize) -> Option<f32> {
        match self {
            Axis::Horizontal => available.height,
            Axis::Vertical => available.width,
        }
    }

    // --- padding ---
    fn padding_main_start(&self, s: &Spacing) -> f32 {
        match self {
            Self::Horizontal => s.padding_left,
            Self::Vertical => s.padding_top,
        }
    }
    fn padding_main_end(&self, s: &Spacing) -> f32 {
        match self {
            Self::Horizontal => s.padding_right,
            Self::Vertical => s.padding_bottom,
        }
    }
    fn padding_cross(&self, s: &Spacing) -> f32 {
        match self {
            Self::Horizontal => s.padding_top + s.padding_bottom,
            Self::Vertical => s.padding_left + s.padding_right,
        }
    }
    fn padding_cross_start(&self, s: &Spacing) -> f32 {
        match self {
            Self::Horizontal => s.padding_top,
            Self::Vertical => s.padding_left,
        }
    }

    // --- margin ---
    fn margin_main(&self, s: &Spacing) -> f32 {
        match self {
            Self::Horizontal => s.margin_left + s.margin_right,
            Self::Vertical => s.margin_top + s.margin_bottom,
        }
    }
    fn margin_cross_start(&self, s: &Spacing) -> f32 {
        match self {
            Self::Horizontal => s.margin_top,
            Self::Vertical => s.margin_left,
        }
    }
    fn margin_cross_end(&self, s: &Spacing) -> f32 {
        match self {
            Self::Horizontal => s.margin_bottom,
            Self::Vertical => s.margin_right,
        }
    }

    // --- size style ---
    fn size(&self, s: &SizeStyle) -> Option<f32> {
        match self {
            Self::Horizontal => s.width,
            Self::Vertical => s.height,
        }
    }
    fn min(&self, s: &SizeStyle) -> Option<f32> {
        match self {
            Self::Horizontal => s.min_width,
            Self::Vertical => s.min_height,
        }
    }
    fn max(&self, s: &SizeStyle) -> Option<f32> {
        match self {
            Self::Horizontal => s.max_width,
            Self::Vertical => s.max_height,
        }
    }
    fn cross_size(&self, s: &SizeStyle) -> (Option<f32>, Option<f32>, Option<f32>) {
        match self {
            Self::Horizontal => (s.height, s.min_height, s.max_height),
            Self::Vertical => (s.width, s.min_width, s.max_width),
        }
    }

    // --- gap ---
    fn gap(&self, style: &Style) -> f32 {
        match self {
            Self::Horizontal => style.column_gap,
            Self::Vertical => style.row_gap,
        }
    }
}

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn layout(root: &mut LayoutNode, width: f32, height: f32) {
        Self::layout_node(
            root,
            ResolvingSize {
                width: Some(width),
                height: Some(height),
            },
            0.0,
            0.0,
        );
    }

    fn layout_node(node: &mut LayoutNode, available: ResolvingSize, origin_x: f32, origin_y: f32) {
        match node.style.display {
            Display::Flex { flex_direction } => match flex_direction {
                FlexDirection::Row => {
                    Self::layout_flex(node, Axis::Horizontal, available, origin_x, origin_y)
                }
                FlexDirection::Column => {
                    Self::layout_flex(node, Axis::Vertical, available, origin_x, origin_y)
                }
            },
            Display::Block => {
                Self::layout_block(node, available, origin_x, origin_y);
            }
            Display::None => {}
        }
    }

    fn layout_flex(
        node: &mut LayoutNode,
        axis: Axis,
        available: ResolvingSize,
        origin_x: f32,
        origin_y: f32,
    ) {
        let (padding_main_start, padding_main_end, padding_cross) = {
            let s = &node.style.spacing;
            (
                axis.padding_main_start(s),
                axis.padding_main_end(s),
                axis.padding_cross(s),
            )
        };
        let gap = axis.gap(&node.style).max(0.0);

        // --- own size ---
        let own_main = axis
            .size(&node.style.size)
            .or(axis.main_available(available));
        let own_cross = axis
            .cross_size(&node.style.size)
            .0
            .or(axis.cross_available(available));

        let inner_main = (own_main.unwrap_or(0.0) - padding_main_start - padding_main_end).max(0.0);
        let inner_cross = own_cross.map(|v| (v - padding_cross).max(0.0));

        // --- size constraints ---
        let main_constraints = Self::resolve_main_constraints(node, axis, inner_main, gap);

        // --- layout children ---
        let max_child_cross = Self::layout_flex_children(
            node,
            axis,
            &main_constraints,
            inner_cross,
            node.style.spacing.margin_top,
            node.style.spacing.margin_left,
            gap,
        );

        // --- redistribute remaining flex_grow after clamp ---
        Self::redistribute_flex_grow_after_layout(node, axis, inner_main, gap);

        // --- final container size ---
        let content_main = Self::calculate_content_main(node, axis, gap);

        let final_main = own_main.unwrap_or(content_main + padding_main_start + padding_main_end);
        let final_cross = own_cross.unwrap_or(max_child_cross + padding_cross);

        let (width, height) = match axis {
            Axis::Horizontal => (final_main, final_cross),
            Axis::Vertical => (final_cross, final_main),
        };

        node.rect = Rect {
            x: origin_x,
            y: origin_y,
            width: width.max(0.0),
            height: height.max(0.0),
        };

        // --- JustifyContent ---
        Self::apply_justify_content(node, axis, final_main, content_main, gap);
    }

    fn resolve_main_constraints(
        node: &LayoutNode,
        axis: Axis,
        inner_main: f32,
        gap: f32,
    ) -> Vec<Option<f32>> {
        let gap_count = node.children.len().saturating_sub(1) as f32;
        let mut fixed = gap * gap_count;
        let mut total_grow = 0.0;

        for child in &node.children {
            fixed += axis
                .size(&child.style.size)
                .unwrap_or(child.style.item_style.flex_basis.unwrap_or(0.0))
                + axis.margin_main(&child.style.spacing);

            if axis.size(&child.style.size).is_none() {
                total_grow += child.style.item_style.flex_grow.max(0.0);
            }
        }

        let remaining = (inner_main - fixed).max(0.0);

        node.children
            .iter()
            .map(|child| {
                if let Some(v) = axis.size(&child.style.size) {
                    return Some(clamp(
                        v,
                        axis.min(&child.style.size),
                        axis.max(&child.style.size),
                    ));
                }

                let grow = child.style.item_style.flex_grow.max(0.0);
                if total_grow > 0.0 && grow > 0.0 {
                    let base = child.style.item_style.flex_basis.unwrap_or(0.0);
                    let size = base + remaining * (grow / total_grow);
                    return Some(clamp(
                        size,
                        axis.min(&child.style.size),
                        axis.max(&child.style.size),
                    ));
                }
                None
            })
            .collect()
    }

    /// Return: Max cross size
    fn layout_flex_children(
        node: &mut LayoutNode,
        axis: Axis,
        main_constraints: &[Option<f32>],
        inner_cross: Option<f32>,
        origin_x: f32,
        origin_y: f32,
        gap: f32,
    ) -> f32 {
        let s = &node.style.spacing;
        let mut cursor = 0.0; // origin has margin_main
        let mut max_cross: f32 = 0.0;

        for (child, main_opt) in node.children.iter_mut().zip(main_constraints) {
            let (cross_size, cross_offset) = compute_cross(
                child
                    .style
                    .item_style
                    .align_self
                    .unwrap_or(node.style.align_items),
                inner_cross.unwrap_or_else(|| {
                    axis.cross_size(&child.style.size).0.unwrap_or(0.0)
                        + axis.margin_cross_start(&child.style.spacing)
                        + axis.margin_cross_end(&child.style.spacing)
                }),
                axis.cross_size(&child.style.size).0,
                axis.cross_size(&child.style.size).1,
                axis.cross_size(&child.style.size).2,
                axis.margin_cross_start(&child.style.spacing),
                axis.margin_cross_end(&child.style.spacing),
            );

            let child_available = match axis {
                Axis::Horizontal => ResolvingSize {
                    width: *main_opt,
                    height: Some(cross_size),
                },
                Axis::Vertical => ResolvingSize {
                    width: Some(cross_size),
                    height: *main_opt,
                },
            };

            let (cx, cy) = match axis {
                Axis::Horizontal => (
                    origin_x + cursor + child.style.spacing.margin_left,
                    origin_y + axis.padding_cross_start(s) + cross_offset,
                ),
                Axis::Vertical => (
                    origin_x + axis.padding_cross_start(s) + cross_offset,
                    origin_y + cursor + child.style.spacing.margin_top,
                ),
            };

            Self::layout_node(child, child_available, cx, cy);

            let child_outer_cross = axis.cross(&child.rect)
                + axis.margin_cross_start(&child.style.spacing)
                + axis.margin_cross_end(&child.style.spacing);
            max_cross = max_cross.max(child_outer_cross);

            cursor += axis.main(&child.rect) + axis.margin_main(&child.style.spacing) + gap;
        }
        max_cross
    }

    fn redistribute_flex_grow_after_layout(
        node: &mut LayoutNode,
        axis: Axis,
        inner_main: f32,
        gap: f32,
    ) {
        // --- sizes of children ---
        let mut sizes: Vec<f32> = node.children.iter().map(|c| axis.main(&c.rect)).collect();

        let mut iteration = 0;
        loop {
            let content_main: f32 =
                sizes.iter().sum::<f32>() + gap * (node.children.len().saturating_sub(1) as f32);
            let remaining = (inner_main - content_main).max(0.0);
            if remaining <= 0.0 || iteration >= 10 {
                break;
            }

            let total_grow: f32 = node
                .children
                .iter()
                .zip(&sizes)
                .filter(|(c, s)| *s < &axis.max(&c.style.size).unwrap_or(f32::INFINITY))
                .map(|(c, _)| c.style.item_style.flex_grow.max(0.0))
                .sum();

            if total_grow <= 0.0 {
                break;
            }

            let mut leftover = 0.0;
            for (i, child) in node.children.iter().enumerate() {
                let grow = child.style.item_style.flex_grow.max(0.0);
                if grow == 0.0 {
                    continue;
                }

                let extra = remaining * (grow / total_grow);
                let clamped = clamp(
                    sizes[i] + extra,
                    axis.min(&child.style.size),
                    axis.max(&child.style.size),
                );
                leftover += (sizes[i] + extra - clamped).max(0.0);
                sizes[i] = clamped;
            }

            // --- Relayout children ---
            for (child, size) in node.children.iter_mut().zip(sizes.iter()) {
                let available = match axis {
                    Axis::Horizontal => ResolvingSize {
                        width: Some(*size),
                        height: Some(child.rect.height),
                    },
                    Axis::Vertical => ResolvingSize {
                        width: Some(child.rect.width),
                        height: Some(*size),
                    },
                };
                LayoutEngine::layout_node(child, available, child.rect.x, child.rect.y);
            }

            if leftover <= 0.0 {
                break;
            }

            iteration += 1;
        }

        let mut cursor = 0.0;
        for (child, size) in node.children.iter_mut().zip(sizes.iter()) {
            match axis {
                Axis::Horizontal => {
                    child.rect.width = *size;
                    child.rect.x = cursor + child.style.spacing.margin_left;
                    cursor += *size + axis.margin_main(&child.style.spacing) + gap;
                }
                Axis::Vertical => {
                    child.rect.height = *size;
                    child.rect.y = cursor + child.style.spacing.margin_top;
                    cursor += *size + axis.margin_main(&child.style.spacing) + gap;
                }
            }
        }
    }

    fn calculate_content_main(node: &LayoutNode, axis: Axis, gap: f32) -> f32 {
        let gap_count = node.children.len().saturating_sub(1) as f32;
        node.children
            .iter()
            .map(|c| axis.main(&c.rect) + axis.margin_main(&c.style.spacing))
            .sum::<f32>()
            + (gap * gap_count)
    }

    /// Fix with JustifyContent
    fn apply_justify_content(
        node: &mut LayoutNode,
        axis: Axis,
        final_main: f32,
        content_main: f32,
        gap: f32,
    ) {
        let s = &node.style.spacing;
        let inner_main =
            (final_main - axis.padding_main_start(s) - axis.padding_main_end(s)).max(0.0);
        let remaining = (inner_main - content_main).max(0.0);

        let (start_offset, justify_gap) =
            resolve_justify_content(node.style.justify_content, remaining, node.children.len());

        let mut cursor = axis.padding_main_start(s) + start_offset;
        let children_len = node.children.len();

        for (i, child) in node.children.iter_mut().enumerate() {
            match axis {
                Axis::Horizontal => {
                    child.rect.x = cursor + child.style.spacing.margin_left;
                }
                Axis::Vertical => {
                    child.rect.y = cursor + child.style.spacing.margin_top;
                }
            }
            cursor += axis.main(&child.rect) + axis.margin_main(&child.style.spacing);
            if i + 1 < children_len {
                cursor += gap + justify_gap;
            }
        }
    }

    fn layout_block(node: &mut LayoutNode, available: ResolvingSize, origin_x: f32, origin_y: f32) {
        let s = &node.style.spacing;

        // --- block width ---
        // width: style > available > auto(None)
        let resolved_width = node.style.size.width.or(available.width);

        let inner_width = resolved_width.map(|w| (w - s.padding_left - s.padding_right).max(0.0));

        let mut cursor_y = s.padding_top;
        let mut max_child_width: f32 = 0.0;

        // --- children ---
        for child in &mut node.children {
            let child_available = ResolvingSize {
                width: inner_width,
                height: None, // not fixed
            };

            Self::layout_node(
                child,
                child_available,
                s.padding_left + child.style.spacing.margin_left,
                cursor_y + child.style.spacing.margin_top,
            );

            cursor_y += child.rect.height
                + child.style.spacing.margin_top
                + child.style.spacing.margin_bottom;

            max_child_width = max_child_width.max(
                child.rect.width
                    + child.style.spacing.margin_left
                    + child.style.spacing.margin_right,
            );
        }

        // resolve auto size
        let computed_width = available
            .width
            .or(resolved_width)
            .unwrap_or(max_child_width + s.padding_left + s.padding_right);

        let computed_height = node
            .style
            .size
            .height
            .unwrap_or(available.height.unwrap_or(cursor_y + s.padding_bottom));

        // max, min
        let final_width = clamp(
            computed_width,
            node.style.size.min_width,
            node.style.size.max_width,
        );
        let final_height = clamp(
            computed_height,
            node.style.size.min_height,
            node.style.size.max_height,
        );

        node.rect = Rect {
            x: origin_x,
            y: origin_y,
            width: final_width.max(0.0),
            height: final_height.max(0.0),
        };
    }
}

// ========= helpers =========

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

fn compute_cross(
    align: AlignItems,
    container: f32,
    item: Option<f32>,
    min: Option<f32>,
    max: Option<f32>,
    margin_start: f32,
    margin_end: f32,
) -> (f32, f32) {
    let mut size = clamp(item.unwrap_or(container), min, max);

    if matches!(align, AlignItems::Stretch) && item.is_none() {
        size = clamp(container - margin_start - margin_end, min, max);
    }

    size = size.max(0.0);

    let free = container - size - margin_start - margin_end;

    let offset = match align {
        AlignItems::Start | AlignItems::Stretch => margin_start,
        AlignItems::Center => margin_start + free / 2.0,
        AlignItems::End => margin_start + free,
    };

    (size, offset)
}
