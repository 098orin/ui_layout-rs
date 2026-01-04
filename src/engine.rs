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
        let s = &node.style.spacing;

        // --- resolve own size ---
        let own_main = axis
            .size(&node.style.size)
            .or(axis.main_available(available));
        let own_cross = axis
            .cross_size(&node.style.size)
            .0
            .or(axis.cross_available(available));

        let inner_main =
            (own_main.unwrap_or(0.0) - axis.padding_main_start(s) - axis.padding_main_end(s))
                .max(0.0);

        let inner_cross = own_cross.map(|v| (v - axis.padding_cross(s)).max(0.0));

        // --- gap ---
        let gap = axis.gap(&node.style).max(0.0);
        let gap_count = node.children.len().saturating_sub(1) as f32;

        // --- first pass: fixed & grow ---
        let mut fixed = gap * gap_count;
        let mut total_grow = 0.0;

        for child in &node.children {
            fixed += match axis.size(&child.style.size) {
                Some(v) => v,
                None => child.style.item_style.flex_basis.unwrap_or(0.0),
            } + axis.margin_main(&child.style.spacing);

            if axis.size(&child.style.size).is_none() {
                total_grow += child.style.item_style.flex_grow.max(0.0);
            }
        }

        let remaining = (inner_main - fixed).max(0.0);

        // --- second pass: main size constraints (Option) ---
        let main_constraints: Vec<Option<f32>> = node
            .children
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

                None // auto
            })
            .collect();

        // --- pass 1: layout children WITHOUT justify-content ---
        let mut cursor = axis.padding_main_start(s);
        let mut max_cross: f32 = 0.0;

        for (child, main_opt) in node.children.iter_mut().zip(&main_constraints) {
            let child_cross_fallback = axis
                .cross_size(&child.style.size)
                .0
                .or(inner_cross)
                .unwrap_or(0.0)
                + axis.margin_cross_start(&child.style.spacing)
                + axis.margin_cross_end(&child.style.spacing);

            let align = child
                .style
                .item_style
                .align_self
                .unwrap_or(node.style.align_items);

            let (cross_size, cross_offset) = compute_cross(
                align,
                inner_cross.unwrap_or(child_cross_fallback),
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

            // --- correct x/y computation ---
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

            let child_cross = axis.cross(&child.rect)
                + axis.margin_cross_start(&child.style.spacing)
                + axis.margin_cross_end(&child.style.spacing);

            max_cross = max_cross.max(child_cross);

            cursor += axis.main(&child.rect) + axis.margin_main(&child.style.spacing) + gap;
        }

        // --- pass 2: justify-content (AFTER layout) ---
        let content_main: f32 = node
            .children
            .iter()
            .map(|c| axis.main(&c.rect) + axis.margin_main(&c.style.spacing))
            .sum::<f32>()
            + gap * gap_count;

        let remaining = (inner_main - content_main).max(0.0);

        let (start_offset, justify_gap) =
            resolve_justify_content(node.style.justify_content, remaining, node.children.len());

        // reposition children according to justify-content
        let mut cursor = axis.padding_main_start(s) + start_offset;

        let children_len = node.children.len();

        for (i, child) in node.children.iter_mut().enumerate() {
            match axis {
                Axis::Horizontal => {
                    child.rect.x = origin_x + cursor + child.style.spacing.margin_left;
                }
                Axis::Vertical => {
                    child.rect.y = origin_y + cursor + child.style.spacing.margin_top;
                }
            }

            cursor += axis.main(&child.rect) + axis.margin_main(&child.style.spacing);
            if i + 1 < children_len {
                cursor += gap + justify_gap;
            }
        }

        // --- auto size resolution ---
        let final_main = own_main
            .unwrap_or(content_main + axis.padding_main_start(s) + axis.padding_main_end(s));
        let final_cross = own_cross.unwrap_or(max_cross + axis.padding_cross(s));

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
    }

    fn layout_block(node: &mut LayoutNode, available: ResolvingSize, origin_x: f32, origin_y: f32) {
        let s = &node.style.spacing;

        // --- block width ---
        // width: style > available > auto(None)
        let resolved_width = node.style.size.width.or(available.width);

        let inner_width =
            (resolved_width.unwrap_or(0.0) - s.padding_left - s.padding_right).max(0.0);

        let mut cursor_y = s.padding_top;
        let mut max_child_width: f32 = 0.0;

        // --- children ---
        for child in &mut node.children {
            let child_available = ResolvingSize {
                width: Some(inner_width.max(0.0)),
                height: None, // not fixed
            };

            Self::layout_node(
                child,
                child_available,
                origin_x + s.padding_left + child.style.spacing.margin_left,
                origin_y + cursor_y + child.style.spacing.margin_top,
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

        let computed_height = available.height.unwrap_or(cursor_y + s.padding_bottom);

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
