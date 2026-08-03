use crate::{EMPTY_LINE_CONTEXT, LayoutBox, LayoutChild, LineContext, Style};

/// (key, (layout_box, LineContext))
type LayoutCache = (u32, (LayoutBox, LineContext));

/// A node in the layout tree.
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
            children: Vec::new(),
            layout_box: LayoutBox::default(),
            layout_box_cache: (0, (LayoutBox::default(), EMPTY_LINE_CONTEXT)),
        }
    }

    /// Create a LayoutNode with arbitrary children.
    pub fn with_children<I, T>(style: Style, children: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<LayoutChild>,
    {
        let children = children.into_iter().map(Into::into).collect();

        Self {
            style,
            children,
            layout_box: LayoutBox::default(),
            layout_box_cache: (0, (LayoutBox::default(), EMPTY_LINE_CONTEXT)),
        }
    }
}
