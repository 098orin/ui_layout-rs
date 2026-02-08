//! Fragment types for inline layout.
//!
//! This module defines the fragment model used for inline content layout,
//! including splittable content fragments and control characters.

/// An item fragment for inline layout.
///
/// Represents the smallest splittable unit of inline content or a control character.
/// Each fragment carries size information and can be positioned independently.
#[derive(Debug, Clone, Copy)]
pub enum ItemFragment {
    /// A splittable fragment of inline content with dimensions.
    Fragment(Fragment),
    /// A control character representing a line break.
    LineBreak,
}

/// A splittable fragment of inline content.
///
/// Fragments are the smallest logical units for inline layout.
/// They carry size information only; positional data is computed during layout.
#[derive(Debug, Clone, Copy)]
pub struct Fragment {
    /// Fragment width
    pub width: f32,
    /// Fragment height
    pub height: f32,
}

/// Layout result for an inline fragment placement.
///
/// Represents where a fragment is positioned after layout computation.
/// Each placement corresponds 1:1 to a fragment in the input.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FragmentPlacement {
    /// Offset position (x, y) relative to the container
    pub offset: (f32, f32),
    /// Line index where this fragment is placed
    pub line_index: usize,
}
