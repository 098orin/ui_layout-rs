use crate::{Display, FlexDirection, JustifyContent, LayoutNode, Rect};

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
                FlexDirection::Column => Self::layout_column(node),
                FlexDirection::Row => Self::layout_row(node),
            },
            Display::Block => Self::layout_block(node),
            Display::None => {}
        }
    }

    fn layout_column(node: &mut LayoutNode) {
        let s = &node.style.spacing;
        let inner_width = node.rect.width - s.padding_left - s.padding_right;
        let inner_height = node.rect.height - s.padding_top - s.padding_bottom;

        let mut fixed_height = 0.0;
        let mut total_grow = 0.0;

        for child in &node.children {
            let margin = child.style.spacing.margin_top + child.style.spacing.margin_bottom;

            if let Some(h) = child.style.size.height {
                fixed_height += h + margin;
            } else {
                fixed_height += child.style.item_style.flex_basis.unwrap_or(0.0) + margin;
                total_grow += child.style.item_style.flex_grow.max(0.0);
            }
        }

        let remaining = (inner_height - fixed_height).max(0.0);

        // final sizes for children (size resolution phase)
        let mut sizes: Vec<f32> = Vec::with_capacity(node.children.len());

        for child in &node.children {
            let h = clamp(
                if let Some(h) = child.style.size.height {
                    h
                } else if total_grow > 0.0 {
                    child.style.item_style.flex_basis.unwrap_or(0.0)
                        + remaining * (child.style.item_style.flex_grow.max(0.0) / total_grow)
                } else {
                    child.style.item_style.flex_basis.unwrap_or(0.0)
                },
                child.style.size.min_height,
                child.style.size.max_height,
            );

            sizes.push(h);
        }

        // used / remaining (for justify)
        let mut used = 0.0;
        for (child, height) in node.children.iter().zip(&sizes) {
            used += height + child.style.spacing.margin_top + child.style.spacing.margin_bottom;
        }

        let remaining = (inner_height - used).max(0.0);
        let count = node.children.len();

        // justify-content
        let (start_offset, gap) =
            resolve_justify_content(node.style.justify_content, remaining, count);

        let mut cursor_y = s.padding_top + start_offset;

        for (child, height) in node.children.iter_mut().zip(sizes) {
            let width = clamp(
                child.style.size.width.unwrap_or(inner_width),
                child.style.size.min_width,
                child.style.size.max_width,
            );

            let rect = Rect {
                x: s.padding_left + child.style.spacing.margin_left,
                y: cursor_y + child.style.spacing.margin_top,
                width,
                height,
            };

            Self::layout_node(child, rect);

            cursor_y +=
                height + child.style.spacing.margin_top + child.style.spacing.margin_bottom + gap;
        }
    }

    fn layout_row(node: &mut LayoutNode) {
        let s = &node.style.spacing;
        let inner_width = node.rect.width - s.padding_left - s.padding_right;
        let inner_height = node.rect.height - s.padding_top - s.padding_bottom;

        let mut fixed_width = 0.0;
        let mut total_grow = 0.0;

        for child in &node.children {
            let margin = child.style.spacing.margin_left + child.style.spacing.margin_right;

            if let Some(w) = child.style.size.width {
                fixed_width += w + margin;
            } else {
                fixed_width += child.style.item_style.flex_basis.unwrap_or(0.0) + margin;
                total_grow += child.style.item_style.flex_grow.max(0.0);
            }
        }

        let remaining = (inner_width - fixed_width).max(0.0);

        // final sizes for children (size resolution phase)
        let mut sizes: Vec<f32> = Vec::with_capacity(node.children.len());

        for child in &node.children {
            let w = clamp(
                if let Some(w) = child.style.size.width {
                    w
                } else if total_grow > 0.0 {
                    child.style.item_style.flex_basis.unwrap_or(0.0)
                        + remaining * (child.style.item_style.flex_grow.max(0.0) / total_grow)
                } else {
                    child.style.item_style.flex_basis.unwrap_or(0.0)
                },
                child.style.size.min_width,
                child.style.size.max_width,
            );

            sizes.push(w);
        }

        // used / remaining (for justify)
        let mut used = 0.0;
        for (child, width) in node.children.iter().zip(&sizes) {
            used += width + child.style.spacing.margin_left + child.style.spacing.margin_right;
        }

        let remaining = (inner_width - used).max(0.0);
        let count = node.children.len();

        // justify-content
        let (start_offset, gap) =
            resolve_justify_content(node.style.justify_content, remaining, count);

        let mut cursor_x = s.padding_left + start_offset;

        for (child, width) in node.children.iter_mut().zip(sizes) {
            let height = clamp(
                child.style.size.height.unwrap_or(inner_height),
                child.style.size.min_height,
                child.style.size.max_height,
            );

            let rect = Rect {
                x: cursor_x + child.style.spacing.margin_left,
                y: s.padding_top + child.style.spacing.margin_top,
                width,
                height,
            };

            Self::layout_node(child, rect);

            cursor_x +=
                width + child.style.spacing.margin_left + child.style.spacing.margin_right + gap;
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
