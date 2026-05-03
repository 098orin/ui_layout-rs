use crate::{LayoutBoxes, LayoutChildren, LineContext, Style};

/// (key, (layout_boxes, LineContext))
type LayoutCache = (u32, (LayoutBoxes, LineContext));

/// A node in the layout tree.
#[non_exhaustive]
#[derive(Debug)]
pub struct LayoutNode {
    pub style: Style,

    pub children: LayoutChildren,

    pub layout_boxes: LayoutBoxes,

    // --- cache ---
    pub(crate) layout_boxes_cache: LayoutCache,
}

impl LayoutNode {
    pub fn new(style: Style) -> Self {
        Self {
            style,
            layout_boxes: LayoutBoxes::default(),
            children: LayoutChildren::new_empty_node(),
            layout_boxes_cache: (0, (LayoutBoxes::default(), ((0.0, 0.0), 0.0))),
        }
    }

    /// A function to create a [`LayoutNode`] whose children are [`LayoutNode`]
    pub fn with_node_children(style: Style, node_children: Vec<LayoutNode>) -> Self {
        let children = LayoutChildren::Node(node_children);

        Self {
            style,
            layout_boxes: LayoutBoxes::default(),
            children,
            layout_boxes_cache: (0, (LayoutBoxes::default(), ((0.0, 0.0), 0.0))),
        }
    }

    /// A function to create a [`LayoutNode`] whose children are [`crate::ItemFragment`]
    pub fn with_fragment_children(style: Style, fragment_children: Vec<LayoutNode>) -> Self {
        let children = LayoutChildren::Node(fragment_children);

        Self {
            style,
            layout_boxes: LayoutBoxes::default(),
            children,
            layout_boxes_cache: (0, (LayoutBoxes::default(), ((0.0, 0.0), 0.0))),
        }
    }
}
