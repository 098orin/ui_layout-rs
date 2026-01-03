use crate::{Display, FlexDirection, LayoutNode, Rect};

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
        let inner_width = node.rect.width - node.style.padding * 2.0;
        let inner_height = node.rect.height - node.style.padding * 2.0;

        let mut fixed_height = 0.0;
        let mut total_grow = 0.0;

        for child in &node.children {
            if let Some(h) = child.style.size.height {
                fixed_height += h;
            } else {
                total_grow += child.style.item_style.flex_grow;
            }
        }

        let remaining = (inner_height - fixed_height).max(0.0);
        let mut cursor_y = node.style.padding;

        for child in &mut node.children {
            let width = clamp(
                child.style.size.width.unwrap_or(inner_width),
                child.style.size.min_width,
                child.style.size.max_width,
            );

            let height = clamp(
                if let Some(h) = child.style.size.height {
                    h
                } else if total_grow > 0.0 {
                    remaining * (child.style.item_style.flex_grow / total_grow)
                } else {
                    inner_height
                },
                child.style.size.min_height,
                child.style.size.max_height,
            );

            let rect = Rect {
                x: node.style.padding,
                y: cursor_y,
                width,
                height,
            };

            Self::layout_node(child, rect);
            cursor_y += height;
        }
    }

    fn layout_row(node: &mut LayoutNode) {
        let inner_width = node.rect.width - node.style.padding * 2.0;
        let inner_height = node.rect.height - node.style.padding * 2.0;

        let mut fixed_width = 0.0;
        let mut total_grow = 0.0;

        for child in &node.children {
            if let Some(w) = child.style.size.width {
                fixed_width += w;
            } else {
                total_grow += child.style.item_style.flex_grow;
            }
        }

        let remaining = (inner_width - fixed_width).max(0.0);
        let mut cursor_x = node.style.padding;

        for child in &mut node.children {
            let width = clamp(
                if let Some(w) = child.style.size.width {
                    w
                } else if total_grow > 0.0 {
                    remaining * (child.style.item_style.flex_grow / total_grow)
                } else {
                    0.0
                },
                child.style.size.min_width,
                child.style.size.max_width,
            );

            let height = clamp(
                child.style.size.height.unwrap_or(inner_height),
                child.style.size.min_height,
                child.style.size.max_height,
            );

            let rect = Rect {
                x: cursor_x,
                y: node.style.padding,
                width,
                height,
            };

            Self::layout_node(child, rect);
            cursor_x += width;
        }
    }

    fn layout_block(node: &mut LayoutNode) {
        let inner_width = node.rect.width - node.style.padding * 2.0;
        let mut cursor_y = node.style.padding;

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
                x: node.style.padding,
                y: cursor_y,
                width,
                height,
            };

            Self::layout_node(child, rect);
            cursor_y += height;
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
