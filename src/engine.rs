use crate::{
    AlignItems, Display, FlexDirection, JustifyContent, LayoutNode, Rect, SizeStyle, Spacing,
};

#[derive(Debug, Clone, Copy)]
enum Axis {
    Horizontal, // row
    Vertical,   // column
}
impl Axis {
    fn main_size(&self, rect: &Rect) -> f32 {
        match self {
            Axis::Horizontal => rect.width,
            Axis::Vertical => rect.height,
        }
    }
    fn cross_size(&self, rect: &Rect) -> f32 {
        match self {
            Axis::Horizontal => rect.height,
            Axis::Vertical => rect.width,
        }
    }

    fn padding_start(&self, s: &Spacing) -> f32 {
        match self {
            Axis::Horizontal => s.padding_left,
            Axis::Vertical => s.padding_top,
        }
    }
    fn padding_end(&self, s: &Spacing) -> f32 {
        match self {
            Axis::Horizontal => s.padding_right,
            Axis::Vertical => s.padding_bottom,
        }
    }

    fn margin_start(&self, s: &Spacing) -> f32 {
        match self {
            Axis::Horizontal => s.margin_left,
            Axis::Vertical => s.margin_top,
        }
    }
    fn margin_end(&self, s: &Spacing) -> f32 {
        match self {
            Axis::Horizontal => s.margin_right,
            Axis::Vertical => s.margin_bottom,
        }
    }

    fn size(&self, size: &SizeStyle) -> Option<f32> {
        match self {
            Axis::Horizontal => size.width,
            Axis::Vertical => size.height,
        }
    }
    fn min_size(&self, size: &SizeStyle) -> Option<f32> {
        match self {
            Axis::Horizontal => size.min_width,
            Axis::Vertical => size.min_height,
        }
    }
    fn max_size(&self, size: &SizeStyle) -> Option<f32> {
        match self {
            Axis::Horizontal => size.max_width,
            Axis::Vertical => size.max_height,
        }
    }
}

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn layout(root: &mut LayoutNode, max_width: f32, max_height: f32) {
        Self::layout_node(
            root,
            Rect {
                x: 0.0,
                y: 0.0,
                width: max_width,
                height: max_height,
            },
        );
    }

    fn layout_node(node: &mut LayoutNode, bounds: Rect) {
        node.rect = bounds;

        match node.style.display {
            Display::Flex { flex_direction } => match flex_direction {
                FlexDirection::Column => Self::layout_flex(node, Axis::Vertical),
                FlexDirection::Row => Self::layout_flex(node, Axis::Horizontal),
            },
            Display::Block => Self::layout_block(node),
            Display::None => {}
        }
    }

    fn layout_flex(node: &mut LayoutNode, axis: Axis) {
        let s = &node.style.spacing;

        let inner_main = axis.main_size(&node.rect) - axis.padding_start(s) - axis.padding_end(s);

        let inner_cross = axis.cross_size(&node.rect)
            - match axis {
                Axis::Horizontal => s.padding_top + s.padding_bottom,
                Axis::Vertical => s.padding_left + s.padding_right,
            };

        // --- first pass ---
        let mut fixed = 0.0;
        let mut total_grow = 0.0;

        for child in &node.children {
            let margin =
                axis.margin_start(&child.style.spacing) + axis.margin_end(&child.style.spacing);

            if let Some(v) = axis.size(&child.style.size) {
                fixed += v + margin;
            } else {
                fixed += child.style.item_style.flex_basis.unwrap_or(0.0) + margin;
                total_grow += child.style.item_style.flex_grow.max(0.0);
            }
        }

        let remaining = (inner_main - fixed).max(0.0);

        // --- second pass ---
        let mut sizes = Vec::with_capacity(node.children.len());

        for child in &node.children {
            let v = clamp(
                if let Some(v) = axis.size(&child.style.size) {
                    v
                } else if total_grow > 0.0 {
                    child.style.item_style.flex_basis.unwrap_or(0.0)
                        + remaining * (child.style.item_style.flex_grow.max(0.0) / total_grow)
                } else {
                    child.style.item_style.flex_basis.unwrap_or(0.0)
                },
                axis.min_size(&child.style.size),
                axis.max_size(&child.style.size),
            );

            sizes.push(v);
        }

        // --- justify-content ---
        let mut used = 0.0;
        for (child, size) in node.children.iter().zip(&sizes) {
            used += size
                + axis.margin_start(&child.style.spacing)
                + axis.margin_end(&child.style.spacing);
        }

        let remaining = (inner_main - used).max(0.0);

        let (start_offset, gap) =
            resolve_justify_content(node.style.justify_content, remaining, node.children.len());

        // --- final layout ---
        let mut cursor = axis.padding_start(s) + start_offset;

        for (child, main_size) in node.children.iter_mut().zip(sizes) {
            let (cross_size, cross_offset) = compute_cross(
                node.style.align_items,
                inner_cross,
                match axis {
                    Axis::Horizontal => child.style.size.height,
                    Axis::Vertical => child.style.size.width,
                },
                match axis {
                    Axis::Horizontal => child.style.size.min_height,
                    Axis::Vertical => child.style.size.min_width,
                },
                match axis {
                    Axis::Horizontal => child.style.size.max_height,
                    Axis::Vertical => child.style.size.max_width,
                },
                match axis {
                    Axis::Horizontal => child.style.spacing.margin_top,
                    Axis::Vertical => child.style.spacing.margin_left,
                },
                match axis {
                    Axis::Horizontal => child.style.spacing.margin_bottom,
                    Axis::Vertical => child.style.spacing.margin_right,
                },
            );

            let rect = match axis {
                Axis::Horizontal => Rect {
                    x: cursor + child.style.spacing.margin_left,
                    y: s.padding_top + cross_offset,
                    width: main_size,
                    height: cross_size,
                },
                Axis::Vertical => Rect {
                    x: s.padding_left + cross_offset,
                    y: cursor + child.style.spacing.margin_top,
                    width: cross_size,
                    height: main_size,
                },
            };

            Self::layout_node(child, rect);

            cursor += main_size
                + axis.margin_start(&child.style.spacing)
                + axis.margin_end(&child.style.spacing)
                + gap;
        }
    }

    fn layout_block(node: &mut LayoutNode) {
        let s = &node.style.spacing;
        let inner_width = node.rect.width - s.padding_left - s.padding_right;
        let mut cursor_y = s.padding_top;

        for child in &mut node.children {
            let width = clamp(
                child.style.size.width.unwrap_or(inner_width),
                child.style.size.min_width,
                child.style.size.max_width,
            );

            let height = clamp(
                child.style.size.height.unwrap_or(0.0),
                child.style.size.min_height,
                child.style.size.max_height,
            );

            let rect = Rect {
                x: s.padding_left + child.style.spacing.margin_left,
                y: cursor_y + child.style.spacing.margin_top,
                width,
                height,
            };

            Self::layout_node(child, rect);
            cursor_y += height + child.style.spacing.margin_top + child.style.spacing.margin_bottom;
        }
    }
}

fn clamp(value: f32, min: Option<f32>, max: Option<f32>) -> f32 {
    let mut v = value;
    if let Some(min) = min {
        v = v.max(min);
    }
    if let Some(max) = max {
        v = v.min(max);
    }
    v
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
    container_size: f32,
    item_size: Option<f32>,
    min: Option<f32>,
    max: Option<f32>,
    margin_start: f32,
    margin_end: f32,
) -> (f32, f32) {
    let mut size = clamp(item_size.unwrap_or(container_size), min, max);

    if matches!(align, AlignItems::Stretch) && item_size.is_none() {
        size = clamp(container_size - margin_start - margin_end, min, max);
    }

    let free = container_size - size - margin_start - margin_end;

    let offset = match align {
        AlignItems::Start | AlignItems::Stretch => margin_start,
        AlignItems::Center => margin_start + free / 2.0,
        AlignItems::End => margin_start + free,
    };

    (size, offset)
}
