use crate::{LayoutBox, OuterDisplay};
use std::fmt::{self, Debug};

/// The measured size of a [`CustomLayouter`] object.
///
/// Returned by [`CustomLayouter::measure`] to report the object's
/// intrinsic dimensions for use in flex sizing and container
/// auto-sizing.
#[derive(Debug, Clone, Copy, Default)]
pub struct MeasureResult {
    /// The object's intrinsic width.
    pub width: f32,
    /// The object's intrinsic height.
    pub height: f32,
}

/// Read-only layout information handed to [`CustomLayouter::layout`] and
/// [`CustomLayouter::measure`].
///
/// This is a slim, stable view of the current layout situation tailored to
/// custom objects. It deliberately exposes only what a custom object needs to
/// size and position itself; the engine's own bookkeeping context (line
/// cursors, flex state, assigned sizes, …) is kept internal.
///
/// The engine constructs a `LayoutContext` for every custom object at each
/// layout / measure call, so the values describe the *current* pass and must
/// not be cached across calls.
#[derive(Debug, Clone, Copy, Default)]
pub struct LayoutContext {
    /// Containing block width, used for resolving percentage lengths and
    /// intrinsic sizing. `None` when unknown.
    pub containing_block_width: Option<f32>,

    /// Containing block height, used for resolving percentage lengths and
    /// intrinsic sizing. `None` when unknown.
    pub containing_block_height: Option<f32>,

    /// Start position of the current line in the parent's coordinate space.
    ///
    /// Only meaningful for objects participating in an inline flow context;
    /// zero otherwise.
    pub start_pos: (f32, f32),

    /// Remaining inline size on the current line before wrapping.
    ///
    /// Only meaningful for objects participating in an inline flow context;
    /// zero otherwise.
    pub available_inline_size: f32,

    /// Line height of the containing inline formatting context.
    ///
    /// When an inline-level object's [`crate::LineSpan`]s occupy multiple
    /// lines, this value is used as the vertical advance between them.
    ///
    /// Only meaningful for objects participating in an inline flow context;
    /// zero otherwise.
    pub line_height: f32,

    /// Viewport width, used for resolving `Vw` units.
    pub viewport_width: f32,

    /// Viewport height, used for resolving `Vh` units.
    pub viewport_height: f32,
}

/// A unified trait for custom layout objects that can participate in
/// both inline and block formatting contexts.
///
/// `CustomLayouter` replaces the previous `FlowLayouter` / `BlockLayouter`
/// split with a single trait. Custom objects implement a single
/// [`layout`](Self::layout) entry point that returns a [`LayoutBox`]:
///
/// - [`LayoutBox::InlineBox`] — the object placed inline content on the current
///   line (spans plus its box model).
/// - [`LayoutBox::BlockBox`] — the object produced a block-level box.
/// - [`LayoutBox::None`] — the object produced nothing.
///
/// The engine selects how the object participates through
/// [`formatting_context`](Self::formatting_context):
///
/// - [`OuterDisplay::Block`] → block-level layout: forces a new line and stacks
///   vertically.
/// - [`OuterDisplay::Inline`] → inline-level layout: shares the current line.
/// - [`OuterDisplay::None`] → the object is skipped entirely.
///
/// The returned [`LayoutBox`] need not match the declared context. Mismatches
/// are handled gracefully:
///
/// - An inline-level object that returns [`LayoutBox::BlockBox`] is placed
///   atomically on the current line like a fragment: the box is never split,
///   and when it does not fit the whole box wraps to the next line.
/// - A block-level object that returns [`LayoutBox::InlineBox`] is wrapped in
///   an anonymous block box: its box model is placed on its own line and its
///   spans are preserved in the result.
///
/// When the object participates in an inline flow context, the [`LayoutContext`]
/// carries the current line's `start_pos`, `available_inline_size`, and
/// `line_height` so the object can position its spans and decide where to wrap.
///
/// In a flex formatting context every item is blockified, so the engine uses
/// [`measure`](Self::measure) for sizing regardless of the reported context.
/// Objects may therefore implement only the methods their context needs.
pub trait CustomLayouter: Debug {
    /// Reports the outer formatting context in which this object participates.
    ///
    /// The engine uses this to select how the object is laid out:
    /// - [`OuterDisplay::Block`] → the object is laid out as a block-level box.
    /// - [`OuterDisplay::Inline`] → the object is laid out inline.
    /// - [`OuterDisplay::None`] → the object is skipped (display: none).
    ///
    /// This must be implemented so the engine knows how to treat the object.
    fn formatting_context(&self) -> OuterDisplay;

    /// Computes and returns this object's layout result.
    ///
    /// The engine calls this during layout and interprets the returned
    /// [`LayoutBox`] according to [`formatting_context`](Self::formatting_context).
    /// The returned variant need not match the declared context; see the
    /// trait-level docs for how mismatches are handled.
    ///
    /// - A block box is expected to be positioned at the origin; the engine
    ///   translates it to its final position.
    /// - An inline box positions its [`crate::LineSpan`]s relative to the parent
    ///   using the line info carried by the [`LayoutContext`].
    ///
    /// Default implementation returns [`LayoutBox::None`].
    fn layout(&mut self, _ctx: &LayoutContext) -> LayoutBox {
        LayoutBox::None
    }

    /// Returns the intrinsic size of this object.
    ///
    /// Used by the flex layout algorithm for sizing and by flow
    /// containers for auto-height computation.
    ///
    /// Default implementation returns zero size.
    fn measure(&self, _ctx: &LayoutContext) -> MeasureResult {
        MeasureResult {
            width: 0.0,
            height: 0.0,
        }
    }

    /// Writes a human-readable name for debugging and tree rendering.
    ///
    /// The default implementation writes the fully-qualified type name
    /// via [`std::any::type_name`].
    fn write_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", std::any::type_name::<Self>())
    }
}
