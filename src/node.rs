use crate::{BoxModel, Style};

#[derive(Debug)]
pub struct LayoutNode {
    pub style: Style,
    pub box_model: BoxModel,
    pub children: Vec<LayoutNode>,
}

impl LayoutNode {
    pub fn new(style: Style) -> Self {
        Self {
            style,
            box_model: BoxModel::default(),
            children: Vec::new(),
        }
    }

    pub fn with_children(style: Style, children: Vec<LayoutNode>) -> Self {
        Self {
            style,
            box_model: BoxModel::default(),
            children,
        }
    }
}
