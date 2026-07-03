use crate::{LayoutContext, LineSpan};
use std::fmt::Debug;

pub struct MeasureResult {
    pub width: f32,
    pub height: f32,
}

/// Context passed to a flow layouter.
#[derive(Debug, Clone, Copy)]
pub struct FlowLayoutContext {
    /// Start position in the parent coordinate space.
    pub start_pos: (f32, f32),

    /// Remaining inline size available on the current line.
    pub available_inline_size: f32,

    /// Offset applied when a forced line break occurs.
    pub line_height: f32,
}

/// A self-layouting object that participates in inline flow.
pub trait FlowLayouter: Debug {
    /// Perform layout starting at `start_pos`.
    ///
    /// Returned spans describe the occupied regions in the inline flow.
    fn layout(&self, ctx: &FlowLayoutContext) -> Vec<LineSpan>;

    fn measure(&self, ctx: &LayoutContext) -> MeasureResult;

    fn debug_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}
