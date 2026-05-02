use crate::{LayoutNode, Spacing};

#[derive(Clone, Copy, Default)]
struct Edge {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

pub(crate) struct LayoutContext {
    pub(crate) containing_block_width: Option<f32>,
    pub(crate) containing_block_height: Option<f32>,
    pub(crate) viewport_width: f32,
    pub(crate) viewport_height: f32,
}

/// Axis orientation
///
/// Provides helper methods to abstract width/height selection, reducing code duplication
/// for row and column layout support.
#[derive(Debug, Clone, Copy)]
enum Axis {
    Horizontal,
    Vertical,
}

pub struct LayoutEngine;

impl LayoutEngine {
    /// Main layout entry point.
    /// Initiates layout computation from the root node with specified viewport dimensions.
    pub fn layout(root: &mut LayoutNode, width: f32, height: f32) {}

    /// Internal method for layout a node.
    /// Layouts a single node and its descendants.
    fn layout_node() {}
}

fn resolve_padding(spacing: &Spacing, ctx: &LayoutContext) -> Edge {
    let containing_width = ctx.containing_block_width;
    let vw = ctx.viewport_width;
    let vh = ctx.viewport_height;

    Edge {
        left: spacing
            .border_left
            .resolve_with(containing_width, vw, vh)
            .unwrap_or(0.0),
        top: spacing
            .border_top
            .resolve_with(containing_width, vw, vh)
            .unwrap_or(0.0),
        right: spacing
            .border_right
            .resolve_with(containing_width, vw, vh)
            .unwrap_or(0.0),
        bottom: spacing
            .border_bottom
            .resolve_with(containing_width, vw, vh)
            .unwrap_or(0.0),
    }
}
