use crate::{LayoutContext, LayoutEngine, MeasureResult, Rect};
use std::fmt;

/// A self-layouting object that participates in block-level layout.
///
/// `CustomLayout` allows custom types to be embedded directly in
/// a layout tree via [`LayoutChild::Custom`](crate::LayoutChild::Custom).
/// The trait has two responsibilities:
///
/// - **`measure`** — reports the object's intrinsic size for flex
///   sizing and container auto-sizing.
/// - **`layout`** — performs block-level layout and returns
///   the final [`Rect`] of this node in the parent's content-box
///   coordinate space.
///
/// # Block-level layout
///
/// In a block formatting context the engine calls [`measure`](Self::measure)
/// to determine the node's intrinsic size, then [`layout`](Self::layout)
/// to obtain the final positioned rectangle.  The returned rect is used
/// for sibling cursor advancement.
///
/// # Flex layout
///
/// In a flex formatting context, the engine uses [`measure`](Self::measure)
/// to determine the object's main-axis and cross-axis sizes.  The object
/// is then positioned by the flex algorithm.
pub trait CustomLayout: fmt::Debug {
    /// Perform block-level layout for this custom node.
    ///
    /// `ctx` provides the containing block dimensions and the available
    /// inline/extent space.  Returns the final [`Rect`] of this node
    /// in the parent's content-box coordinate space.
    fn layout(&mut self, engine: &LayoutEngine, ctx: &LayoutContext) -> Rect;

    /// Returns the intrinsic size of this object.
    ///
    /// Used by the flex layout algorithm for sizing and by flow
    /// containers for auto-height computation.  The returned
    /// [`MeasureResult`] should reflect the object's natural
    /// dimensions independent of any particular layout context.
    fn measure(&self, ctx: &LayoutContext) -> MeasureResult;

    /// Writes a human-readable name for debugging and tree
    /// rendering.
    fn write_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", std::any::type_name::<Self>())
    }
}
