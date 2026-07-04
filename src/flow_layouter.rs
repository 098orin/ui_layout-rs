use crate::{LayoutContext, LineSpan};
use std::fmt::{self, Debug};

/// The measured size of a [`FlowLayouter`] object.
///
/// Returned by [`FlowLayouter::measure`] to report the object's
/// intrinsic dimensions for use in flex sizing and container
/// auto-sizing.
#[derive(Debug, Clone, Copy, Default)]
pub struct MeasureResult {
    /// The object's intrinsic width.
    pub width: f32,
    /// The object's intrinsic height.
    pub height: f32,
}

/// Context passed to [`FlowLayouter::layout`] for inline flow
/// participation.
///
/// Provides the positional and dimensional information an object
/// needs to determine where it should place its content when
/// participating in an inline formatting context.
#[derive(Debug, Clone, Copy)]
pub struct FlowLayoutContext {
    /// Start position in the parent coordinate space.
    ///
    /// This is the (x, y) offset where the object should begin
    /// placing its content on the current line.
    pub start_pos: (f32, f32),

    /// Remaining inline size available on the current line before
    /// wrapping occurs.
    pub available_inline_size: f32,

    /// Line height of the containing formatting context.
    ///
    /// When [`layout`](Self::layout) returns spans that occupy
    /// multiple lines, this value is used as the vertical advance
    /// between consecutive lines.
    pub line_height: f32,
}

/// A self-layouting object that participates in layout flows.
///
/// `FlowLayouter` allows custom types to be embedded directly in
/// a layout tree via [`LayoutChild::Object`](crate::LayoutChild::Object).
/// The trait has two responsibilities:
///
/// - **`measure`** — reports the object's intrinsic size for flex
///   sizing and container auto-sizing.
/// - **`layout`** — performs inline-level layout and returns
///   [`LineSpan`]s describing how the object occupies space in the
///   current formatting context.
///
/// # Flow layout
///
/// In an inline formatting context, the engine calls [`layout`](Self::layout)
/// passing the object's start position, the remaining space on the
/// current line, and the container's line height.  The returned spans
/// describe which regions of the inline coordinate space the object
/// consumes.  When a span does not fit on the current line, the engine
/// advances to the next line and calls `layout` again.
///
/// # Flex layout
///
/// In a flex formatting context, the engine uses [`measure`](Self::measure)
/// to determine the object's main-axis and cross-axis size.  The object
/// is then positioned by the flex algorithm (grow, shrink, alignment,
/// justification, reverse direction, gaps, etc.) like any other flex item.
///
/// # Examples
///
/// ```rust
/// use ui_layout::*;
///
/// #[derive(Debug)]
/// struct MyWidget {
///     width: f32,
///     height: f32,
/// }
///
/// impl FlowLayouter for MyWidget {
///     fn layout(&self, ctx: &FlowLayoutContext) -> Vec<LineSpan> {
///         let (x, y) = ctx.start_pos;
///         vec![LineSpan {
///             x_range: x..(x + self.width),
///             line_pos: (x, y),
///             line_index: 0,
///         }]
///     }
///
///     fn measure(&self, _ctx: &LayoutContext) -> MeasureResult {
///         MeasureResult { width: self.width, height: self.height }
///     }
/// }
///
/// let mut root = LayoutNode::with_children(
///     Style::default(),
///     [LayoutChild::Object(Box::new(MyWidget { width: 50.0, height: 20.0 }))],
/// );
/// LayoutEngine::layout(&mut root, 800.0, 600.0);
/// ```
pub trait FlowLayouter: Debug {
    /// Perform inline layout for this object.
    ///
    /// The returned [`LineSpan`]s describe how this object occupies
    /// space in the inline formatting context.  Each span corresponds
    /// to a contiguous region on a single line.
    ///
    /// Implementations should respect [`ctx.available_inline_size`]
    /// (FlowLayoutContext::available_inline_size) and split content
    /// across multiple spans when necessary.
    fn layout(&self, ctx: &FlowLayoutContext) -> Vec<LineSpan>;

    /// Returns the intrinsic size of this object.
    ///
    /// Used by the flex layout algorithm for sizing and by flow
    /// containers for auto-height computation.  The returned
    /// [`MeasureResult`] should reflect the object's natural
    /// dimensions independent of any particular layout context.
    fn measure(&self, ctx: &LayoutContext) -> MeasureResult;

    /// Writes a human-readable name for debugging and tree
    /// rendering.
    ///
    /// The default implementation writes the fully-qualified
    /// type name via [`std::any::type_name`].
    fn write_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", std::any::type_name::<Self>())
    }
}
