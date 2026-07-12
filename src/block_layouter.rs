use crate::{LayoutContext, Rect};
use std::fmt::{self, Debug};

/// A self-layouting block-level component.
///
/// `BlockLayouter` allows custom types to be embedded in a layout
/// tree as block-level children via
/// [`LayoutChild::Custom`](crate::LayoutChild::Custom).
///
/// # Block layout
///
/// In a block formatting context, the engine positions the component
/// as a block child (stacked vertically).  The component returns its
/// border-box [`Rect`] via [`layout`](Self::layout).
///
/// # Flex layout
///
/// In a flex formatting context, the engine uses the [`Rect`] returned
/// by [`layout`](Self::layout) to determine the component's main-axis
/// and cross-axis size.
///
/// # Examples
///
/// ```rust
/// use ui_layout::*;
///
/// #[derive(Debug)]
/// struct MyPanel {
///     width: f32,
///     height: f32,
/// }
///
/// impl BlockLayouter for MyPanel {
///     fn layout(&mut self, _ctx: &LayoutContext) -> Rect {
///         Rect { x: 0.0, y: 0.0, width: self.width, height: self.height }
///     }
/// }
///
/// let panel: Box<dyn BlockLayouter> = Box::new(MyPanel { width: 200.0, height: 100.0 });
/// let root = LayoutNode::with_children(
///     Style::default(),
///     [LayoutChild::Custom(panel)],
/// );
/// ```
pub trait BlockLayouter: Debug {
    /// Compute and return the component's border-box rect.
    ///
    /// The returned [`Rect`] describes the component's position and
    /// size including border, padding, and content area.  The engine
    /// writes this result into the parent's layout output.
    fn layout(&mut self, ctx: &LayoutContext) -> Rect;

    /// Writes a human-readable name for debugging and tree rendering.
    ///
    /// The default implementation writes the fully-qualified type name
    /// via [`std::any::type_name`].
    fn write_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", std::any::type_name::<Self>())
    }
}
