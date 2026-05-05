use crate::{FragmentNode, LayoutBox, LayoutChild, LineContext, Style};

/// (key, (layout_box, LineContext))
type LayoutCache = (u32, (LayoutBox, LineContext));

/// A node in the layout tree.
#[non_exhaustive]
#[derive(Debug)]
pub struct LayoutNode {
    pub style: Style,

    pub children: Vec<LayoutChild>,

    pub layout_box: LayoutBox,

    // --- cache ---
    pub(crate) layout_box_cache: LayoutCache,
}

impl LayoutNode {
    pub fn new(style: Style) -> Self {
        Self {
            style,
            layout_box: LayoutBox::default(),
            children: Vec::new(),
            layout_box_cache: (0, (LayoutBox::default(), ((0.0, 0.0), 0.0))),
        }
    }

    /// A function to create a [`LayoutNode`] whose children are [`LayoutNode`]
    pub fn with_node_children(style: Style, node_children: Vec<LayoutNode>) -> Self {
        let children = node_children.into_iter().map(LayoutChild::Node).collect();

        Self {
            style,
            layout_box: LayoutBox::default(),
            children,
            layout_box_cache: (0, (LayoutBox::default(), ((0.0, 0.0), 0.0))),
        }
    }

    /// A function to create a [`LayoutNode`] whose children are [`crate::ItemFragment`]
    pub fn with_fragment_children(style: Style, fragment_children: Vec<FragmentNode>) -> Self {
        let children = fragment_children
            .into_iter()
            .map(LayoutChild::Fragment)
            .collect();

        Self {
            style,
            layout_box: LayoutBox::default(),
            children,
            layout_box_cache: (0, (LayoutBox::default(), ((0.0, 0.0), 0.0))),
        }
    }
}
