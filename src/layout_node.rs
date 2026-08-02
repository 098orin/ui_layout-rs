use crate::{EMPTY_LINE_CONTEXT, CustomObjectResult, LayoutBox, LayoutChild, LineContext, Style};

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

    // --- custom object results ---
    /// Results for custom/replaced elements (indexed by child position).
    /// This allows engines to store layout results without thread-local caches.
    pub custom_object_results: Vec<Option<CustomObjectResult>>,
}

impl LayoutNode {
    pub fn new(style: Style) -> Self {
        Self {
            style,
            children: Vec::new(),
            layout_box: LayoutBox::default(),
            layout_box_cache: (0, (LayoutBox::default(), EMPTY_LINE_CONTEXT)),
            custom_object_results: Vec::new(),
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
            custom_object_results: Vec::new(),
        }
    }

    /// Sets the layout result for a custom object at the given index.
    pub fn set_custom_object_result(&mut self, index: usize, result: CustomObjectResult) {
        if index >= self.custom_object_results.len() {
            self.custom_object_results.resize(index + 1, None);
        }
        self.custom_object_results[index] = Some(result);
    }

    /// Gets the layout result for a custom object at the given index.
    pub fn get_custom_object_result(&self, index: usize) -> Option<&CustomObjectResult> {
        self.custom_object_results
            .get(index)
            .and_then(|r| r.as_ref())
    }
}
