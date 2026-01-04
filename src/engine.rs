use crate::{
    AlignItems, Display, FlexDirection, JustifyContent, LayoutNode, Rect, SizeStyle, Spacing, Style,
};

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
    pub fn layout(root: &mut LayoutNode, max_width: f32, max_height: f32) {
        Self::layout_node(
            root,
            Some(Rect {
                x: 0.0,
                y: 0.0,
                width: max_width,
                height: max_height,
            }),
        );
    }

    fn layout_node(node: &mut LayoutNode, bounds: Option<Rect>) {
        if let Some(rect) = bounds {
            node.rect = rect;
        } else {
            Self::layout_none_rect(node);
        }

        match node.style.display {
            Display::Flex { flex_direction } => match flex_direction {
                FlexDirection::Column => Self::layout_flex(node, Axis::Vertical),
                FlexDirection::Row => Self::layout_flex(node, Axis::Horizontal),
            },
            Display::Block => Self::layout_block(node),
            Display::None => {}
        }
    }

    fn layout_none_rect(node: &mut LayoutNode) {
        for child in node.children.iter_mut() {
            Self::layout_node(child, None);
        }

        let mut width = 0.0;
        let mut height = 0.0;

        match node.style.display {
            Display::Flex { flex_direction } => {
                let axis = match flex_direction {
                    FlexDirection::Row => Axis::Horizontal,
                    FlexDirection::Column => Axis::Vertical,
                };

                let gap = axis.gap(&node.style).max(0.0);
                let gap_count = node.children.len().saturating_sub(1) as f32;

                let mut main_sum = 0.0;
                let mut cross_max: f32 = 0.0;

                for child in &node.children {
                    let r = child.rect;
                    let main = axis.main(&r) + axis.margin_main(&child.style.spacing);
                    let cross: f32 = axis.cross(&r)
                        + axis.margin_cross_start(&child.style.spacing)
                        + axis.margin_cross_end(&child.style.spacing);
                    main_sum += main;
                    cross_max = cross_max.max(cross);
                }

                main_sum += gap * gap_count;

                match flex_direction {
                    FlexDirection::Row => {
                        width = main_sum
                            + node.style.spacing.padding_left
                            + node.style.spacing.padding_right;
                        height = cross_max
                            + node.style.spacing.padding_top
                            + node.style.spacing.padding_bottom;
                    }
                    FlexDirection::Column => {
                        width = cross_max
                            + node.style.spacing.padding_left
                            + node.style.spacing.padding_right;
                        height = main_sum
                            + node.style.spacing.padding_top
                            + node.style.spacing.padding_bottom;
                    }
                }
            }
            Display::Block => {
                let mut max_width: f32 = 0.0;
                let mut total_height = 0.0;
                for child in &node.children {
                    let r = child.rect;
                    max_width = max_width.max(
                        r.width
                            + child.style.spacing.margin_left
                            + child.style.spacing.margin_right,
                    );
                    total_height += r.height
                        + child.style.spacing.margin_top
                        + child.style.spacing.margin_bottom;
                }
                width =
                    max_width + node.style.spacing.padding_left + node.style.spacing.padding_right;
                height = total_height
                    + node.style.spacing.padding_top
                    + node.style.spacing.padding_bottom;
            }
            Display::None => {}
        }

        node.rect = Rect {
            x: 0.0,
            y: 0.0,
            width,
            height,
        };
    }

    fn layout_flex(node: &mut LayoutNode, axis: Axis) {
        let spacing = &node.style.spacing;

        let gap = axis.gap(&node.style).max(0.0);
        let gap_count = node.children.len().saturating_sub(1) as f32;

        let inner_main = (axis.main(&node.rect)
            - axis.padding_main_start(spacing)
            - axis.padding_main_end(spacing))
        .max(0.0);
        let inner_cross = (axis.cross(&node.rect) - axis.padding_cross(spacing)).max(0.0);

        // --- first pass ---
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

        // --- second pass ---
        let sizes: Vec<f32> = node
            .children
            .iter()
            .map(|child| {
                let base = axis.size(&child.style.size).unwrap_or_else(|| {
                    if total_grow > 0.0 {
                        child.style.item_style.flex_basis.unwrap_or(0.0)
                            + remaining * (child.style.item_style.flex_grow.max(0.0) / total_grow)
                    } else {
                        child.style.item_style.flex_basis.unwrap_or(0.0)
                    }
                });

                clamp(
                    base,
                    axis.min(&child.style.size),
                    axis.max(&child.style.size),
                )
                .max(0.0)
            })
            .collect();

        // --- justify-content ---
        let used: f32 = node
            .children
            .iter()
            .zip(&sizes)
            .map(|(c, s)| s + axis.margin_main(&c.style.spacing))
            .sum::<f32>()
            + gap * gap_count;

        let remaining = (inner_main - used).max(0.0);
        let (start_offset, justify_gap) =
            resolve_justify_content(node.style.justify_content, remaining, node.children.len());

        // --- final layout ---
        let mut cursor = axis.padding_main_start(spacing) + start_offset;

        for (child, main_size) in node.children.iter_mut().zip(sizes) {
            let (item, min, max) = axis.cross_size(&child.style.size);

            let (cross_size, cross_offset) = compute_cross(
                node.style.align_items,
                inner_cross,
                item,
                min,
                max,
                axis.margin_cross_start(&child.style.spacing),
                axis.margin_cross_end(&child.style.spacing),
            );

            let rect = match axis {
                Axis::Horizontal => Rect {
                    x: cursor + child.style.spacing.margin_left,
                    y: spacing.padding_top + cross_offset,
                    width: main_size.max(0.0),
                    height: cross_size,
                },
                Axis::Vertical => Rect {
                    x: spacing.padding_left + cross_offset,
                    y: cursor + child.style.spacing.margin_top,
                    width: cross_size,
                    height: main_size.max(0.0),
                },
            };

            Self::layout_node(child, Some(rect));

            cursor += main_size + axis.margin_main(&child.style.spacing) + gap + justify_gap;
        }
    }

    fn layout_block(node: &mut LayoutNode) {
        let s = &node.style.spacing;
        let inner_width = node.rect.width - s.padding_left - s.padding_right;
        let mut y = s.padding_top;

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
                y: y + child.style.spacing.margin_top,
                width,
                height,
            };

            Self::layout_node(child, Some(rect));
            y += height + child.style.spacing.margin_top + child.style.spacing.margin_bottom;
        }
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
