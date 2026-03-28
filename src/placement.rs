/// Layout result for an inline placement.
///
/// Represents where a is positioned after layout computation.
/// Each placement corresponds 1:1 to a in the input.
/// This 1:1 mapping applies uniformly to both `Node` and `Fragment`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    /// Offset position (x, y) relative to the container
    pub offset: (f32, f32),
    /// 0-indexed line index where the item is placed
    pub line_index: usize,
}
