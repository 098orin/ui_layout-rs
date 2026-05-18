use crate::{EMPTY_LINE_CONTEXT, LayoutBox, LayoutChild, LineContext, Style};

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
            layout_box_cache: (0, (LayoutBox::default(), EMPTY_LINE_CONTEXT)),
        }
    }

    /// Create a LayoutNode with arbitrary children.
    pub fn with_children<T>(style: Style, children: Vec<T>) -> Self
    where
        T: Into<LayoutChild>,
    {
        let children = children.into_iter().map(Into::into).collect();

        Self {
            style,
            layout_box: LayoutBox::default(),
            children,
            layout_box_cache: (0, (LayoutBox::default(), EMPTY_LINE_CONTEXT)),
        }
    }
}
