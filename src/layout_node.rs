use crate::{
    CustomChild, EMPTY_LINE_CONTEXT, LayoutBox, LayoutChild, LayoutItem, LineContext, Style,
};

/// (key, (layout_box, LineContext))
type LayoutCache = (u32, (LayoutBox, LineContext));

/// A restore entry describing how to put a `display: contents` node's children
/// back after the layout tree has been temporarily flattened.
///
/// Used by the engine to preserve the original tree shape across layout: a
/// `display: contents` node is lifted out of its parent's `children` during
/// layout (its `layout_box` is `None`), and this record allows it to be
/// re-nested afterwards.
#[derive(Debug)]
pub(crate) enum RestoreEntry {
    Direct,
    Contents {
        shell: Box<LayoutNode>,
        child_count: usize,
    },
    CustomContents {
        shell: Box<CustomChild>,
    },
}

/// A node in the layout tree.
#[derive(Debug)]
pub struct LayoutNode {
    pub style: Style,

    pub children: Vec<LayoutChild>,

    pub layout_box: LayoutBox,

    // --- cache ---
    pub(crate) layout_box_cache: LayoutCache,

    /// Reusable scratch buffer for layout item iteration.
    /// Avoids per-layout-pass heap allocations by preserving capacity across calls.
    pub(crate) items_buf: Vec<LayoutItem>,

    /// Scratch storage for restoring the original tree shape after the
    /// engine temporarily flattens `display: contents` nodes during layout.
    pub(crate) flatten_restore: Vec<RestoreEntry>,
}

impl LayoutNode {
    pub fn new(style: Style) -> Self {
        Self {
            style,
            children: Vec::new(),
            layout_box: LayoutBox::default(),
            layout_box_cache: (0, (LayoutBox::default(), EMPTY_LINE_CONTEXT)),
            items_buf: Vec::new(),
            flatten_restore: Vec::new(),
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
            items_buf: Vec::new(),
            flatten_restore: Vec::new(),
        }
    }
}
