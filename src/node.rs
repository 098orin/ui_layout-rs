use crate::{BoxModel, Style};

#[non_exhaustive]
#[derive(Debug)]
pub struct LayoutNode {
    pub style: Style,
    pub box_model: BoxModel,
    pub children: Vec<LayoutNode>,

    // --- cache ---
    pub(crate) box_model_cache: (u32, BoxModel), // (key, box_model)
}

impl LayoutNode {
    pub fn new(style: Style) -> Self {
        Self {
            style,
            box_model: BoxModel::default(),
            children: Vec::new(),
            box_model_cache: (0, BoxModel::default()),
        }
    }

    pub fn with_children(style: Style, children: Vec<LayoutNode>) -> Self {
        Self {
            style,
            box_model: BoxModel::default(),
            children,
            box_model_cache: (0, BoxModel::default()),
        }
    }
}
