use crate::LayoutNode;

#[derive(Clone, Copy, Default)]
struct Edge {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

struct LayoutContext {
    viewport_width: f32,
    viewport_height: f32,
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
}
