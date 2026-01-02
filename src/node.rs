use crate::{Rect, Style};

#[derive(Debug)]
pub struct LayoutNode {
    pub style: Style,
    pub rect: Rect,
    pub children: Vec<LayoutNode>,
}

impl LayoutNode {
    pub fn new(style: Style) -> Self {
        Self {
            style,
            rect: Rect::default(),
            children: Vec::new(),
        }
    }

    pub fn with_children(style: Style, children: Vec<LayoutNode>) -> Self {
        Self {
            style,
            rect: Rect::default(),
            children,
        }
    }
}
