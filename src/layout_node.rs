use crate::{LayoutBoxes, LayoutItem, Placement, Style};

type LayoutCache = (u32, (LayoutBoxes, Vec<Placement>, ((f32, f32), f32)));

/// A node in the layout tree.
///
/// A `LayoutNode` represents a single layout object and is responsible for:
///
/// - Holding layout-related style information
/// - Owning child layout items (mixed inline fragments and child nodes)
/// - Storing layout results (box model and placements)
///
/// ## Children model
///
/// `children` is an ordered list of layout items (`LayoutItem`), which may contain:
///
/// - Other `LayoutNode`s (block-level or container-level items)
/// - Inline-level fragments (`ItemFragment`)
///
/// This unified representation allows inline and block content to coexist,
/// enabling correct handling of cases such as inline content interrupted by blocks.
///
/// The order of `children` is preserved and defines the layout flow.
///
/// ## Placement model
///
/// `placements` stores the layout result for each child in `children`.
/// Each entry corresponds 1:1 with `children` and provides the computed
/// relative position of that item within this node.
///
/// - `placements.len() == children.len()` after layout
/// - Each placement is expressed in the local coordinate space of this node
/// - Placement data is only valid after layout computation
///
/// This model unifies positioning for both fragments and child nodes.
///
/// ## Box model
///
/// `layout_boxes` stores the computed box model for this node.
/// All box coordinates are defined in the local coordinate space,
/// with the border box origin at (0, 0).
///
/// ## Layout behavior
///
/// This structure does not encode layout behavior at the type level.
/// Instead, behavior (inline, block, flex, etc.) is determined by `Style::display`.
///
/// ## Results storage
///
/// Layout results are stored directly on the node:
///
/// - `layout_boxes`: computed box geometry for this node
/// - `placements`: computed positions for each child in `children`
///
/// This allows efficient post-layout traversal and rendering without recomputation.
#[non_exhaustive]
#[derive(Debug)]
pub struct LayoutNode {
    pub style: Style,

    pub children: Vec<LayoutItem>,

    pub layout_boxes: LayoutBoxes,
    pub placements: Vec<Placement>,

    // --- cache ---
    pub(crate) layout_boxes_cache: LayoutCache, // (key, layout_boxes, placements, ((f32, f32), f32))
}

impl LayoutNode {
    pub fn new(style: Style) -> Self {
        Self {
            style,
            placements: Vec::new(),
            layout_boxes: LayoutBoxes::default(),
            children: Vec::new(),
            layout_boxes_cache: (0, (LayoutBoxes::default(), vec![], ((0.0, 0.0), 0.0))),
        }
    }

    /// A function to create a [`LayoutNode`] whose children are only [`LayoutNode`]
    pub fn with_node_children(style: Style, node_children: Vec<LayoutNode>) -> Self {
        let children = node_children.into_iter().map(LayoutItem::Node).collect();

        Self {
            style,
            placements: Vec::new(),
            layout_boxes: LayoutBoxes::default(),
            children,
            layout_boxes_cache: (0, (LayoutBoxes::default(), vec![], ((0.0, 0.0), 0.0))),
        }
    }

    /// A function to create a [`LayoutNode`] with children.
    ///
    /// See [`LayoutItem`] for detail.
    pub fn with_children(style: Style, children: Vec<LayoutItem>) -> Self {
        Self {
            style,
            placements: Vec::new(),
            layout_boxes: LayoutBoxes::default(),
            children,
            layout_boxes_cache: (0, (LayoutBoxes::default(), vec![], ((0.0, 0.0), 0.0))),
        }
    }
}
