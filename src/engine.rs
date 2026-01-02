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
                FlexDirection::Column => {
                    Self::layout_column(node);
                }
                FlexDirection::Row => {
                    Self::layout_row(node);
                }
            },
            Display::Block => {
                // TODO: block layout (currently treated as column)
                Self::layout_column(node);
            }
            Display::None => {}
        }
    }

    fn layout_column(node: &mut LayoutNode) {
        let inner_width = node.rect.width - node.style.padding * 2.0;
        let inner_height = node.rect.height - node.style.padding * 2.0;

        let mut fixed_height = 0.0;
        let mut total_grow = 0.0;

        for child in &node.children {
            if let Some(h) = child.style.height {
                fixed_height += h;
            } else {
                total_grow += child.style.item_style.flex_grow;
            }
        }

        let remaining = (inner_height - fixed_height).max(0.0);

        let mut cursor_y = node.style.padding;

        for child in &mut node.children {
            let width = if let Some(w) = child.style.width {
                w
            } else {
                inner_width
            };

            let height = if let Some(h) = child.style.height {
                h
            } else if total_grow > 0.0 {
                remaining * (child.style.item_style.flex_grow / total_grow)
            } else {
                inner_height
            };

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
            if let Some(w) = child.style.width {
                fixed_width += w;
            } else {
                total_grow += child.style.item_style.flex_grow;
            }
        }

        let remaining = (inner_width - fixed_width).max(0.0);

        let mut cursor_x = node.style.padding;

        for child in &mut node.children {
            let width = if let Some(w) = child.style.width {
                w
            } else if total_grow > 0.0 {
                remaining * (child.style.item_style.flex_grow / total_grow)
            } else {
                0.0
            };

            let height = if let Some(h) = child.style.height {
                h
            } else {
                inner_height
            };

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
}
