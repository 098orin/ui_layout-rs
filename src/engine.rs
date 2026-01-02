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
                Self::layout_column(node);
            }
            Display::None => {}
        }
    }

    fn layout_column(node: &mut LayoutNode) {
        let mut y = node.rect.y + node.style.padding;

        for child in &mut node.children {
            let height = child.style.height.unwrap_or(0.0);

            let rect = Rect {
                x: node.rect.x + node.style.padding,
                y,
                width: node.rect.width - node.style.padding * 2.0,
                height,
            };

            Self::layout_node(child, rect);
            y += height;
        }
    }

    fn layout_row(node: &mut LayoutNode) {
        let mut x = node.rect.x + node.style.padding;

        for child in &mut node.children {
            let width = child.style.width.unwrap_or(0.0);

            let rect = Rect {
                x,
                y: node.rect.y + node.style.padding,
                width,
                height: node.rect.height - node.style.padding * 2.0,
            };

            Self::layout_node(child, rect);
            x += width;
        }
    }
}
