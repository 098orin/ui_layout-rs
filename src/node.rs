use crate::{FragmentPlacement, ItemFragment, LayoutBoxes, Style};

/// A node in the layout tree.
///
/// A `LayoutNode` represents a single layout object and is responsible for:
///
/// - Holding layout-related style information
/// - Owning child layout nodes (structural hierarchy)
/// - Providing inline fragment inputs
/// - Storing layout results (box model and fragment placements)
///
/// ## Fragment model
///
/// `self_fragments` represent the **input fragments** provided by the user.
/// They are immutable logical units and are never reordered by the layout engine.
///
/// `placements` represent the **layout result** for fragments.
/// Each entry corresponds 1:1 to `self_fragments` and stores relative placement
/// information computed during layout.
///
/// Only inline-level nodes are expected to have non-empty `self_fragments`.
/// For block-level or flex-level nodes, this vector should be empty.
///
/// ## Layout invariants
///
/// - The order of `self_fragments` is preserved during layout
/// - `placements.len() == self_fragments.len()` after layout
/// - `placements` are meaningful only after layout computation
///
/// This structure intentionally does not distinguish between inline, block,
/// or flex nodes at the type level; their behavior is defined by `Style::display`.
///
/// ## Results storage
/// Results of the layout process are stored directly within each `LayoutNode`,
/// allowing for easy access and further processing after layout computation.
/// This includes the computed `BoxModel` and fragment placements.
/// - `layout_boxes`: The computed box model for this node after layout.
/// - `placements`: The computed placements for each fragment in `self_fragments`.
#[non_exhaustive]
#[derive(Debug)]
pub struct LayoutNode {
    pub style: Style,
    pub self_fragments: Vec<ItemFragment>,

    pub children: Vec<LayoutNode>,

    pub layout_boxes: LayoutBoxes,
    pub placements: Vec<FragmentPlacement>,

    // --- cache ---
    pub(crate) layout_boxes_cache: (u32, (LayoutBoxes, ((f32, f32), f32))), // (key, layout_boxes, ((f32, f32), f32))
}

impl LayoutNode {
    pub fn new(style: Style) -> Self {
        Self {
            style,
            self_fragments: Vec::new(),
            placements: Vec::new(),
            layout_boxes: LayoutBoxes::default(),
            children: Vec::new(),
            layout_boxes_cache: (0, (LayoutBoxes::default(), ((0.0, 0.0), 0.0))),
        }
    }

    pub fn with_children(style: Style, children: Vec<LayoutNode>) -> Self {
        Self {
            style,
            self_fragments: Vec::new(),
            placements: Vec::new(),
            layout_boxes: LayoutBoxes::default(),
            children,
            layout_boxes_cache: (0, (LayoutBoxes::default(), ((0.0, 0.0), 0.0))),
        }
    }

    /// Sets input fragments for this node.
    ///
    /// This method defines the inline content owned by the node.
    /// Any previously computed fragment placements become invalid and must
    /// be recomputed during the next layout pass.
    pub fn set_fragments(&mut self, fragments: Vec<ItemFragment>) {
        self.self_fragments = fragments;
        self.placements.reserve(self.self_fragments.len());
    }
}
