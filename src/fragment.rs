//! Fragment types for inline layout.
//!
//! This module defines the fragment model used for inline content layout,
//! including splittable content fragments and control characters.

/// A fragment with associated layout placement information.
///
/// Wraps an `ItemFragment` together with its computed `Placement`.
/// Used during layout to track both fragment content and its position.
#[derive(Debug, Clone, Copy)]
pub struct FragmentNode {
    pub node: ItemFragment,
    pub placement: Placement,
}

impl FragmentNode {
    pub fn new(item: ItemFragment) -> Self {
        FragmentNode {
            node: item,
            placement: Placement::default(),
        }
    }
}

impl From<ItemFragment> for FragmentNode {
    fn from(value: ItemFragment) -> Self {
        Self::new(value)
    }
}

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

impl ItemFragment {
    /// Returns the width of the fragment, or 0 for line breaks.
    pub fn width(&self) -> f32 {
        match self {
            ItemFragment::Fragment(frag) => frag.width,
            ItemFragment::LineBreak => 0.0,
        }
    }

    /// Returns the height of the fragment, or 0 for line breaks.
    pub fn height(&self) -> f32 {
        match self {
            ItemFragment::Fragment(frag) => frag.height,
            ItemFragment::LineBreak => 0.0,
        }
    }

    /// Checks if the fragment is a line break.
    pub fn is_line_break(&self) -> bool {
        matches!(self, ItemFragment::LineBreak)
    }
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

/// Layout result for an inline placement.
///
/// Represents where a is positioned after layout computation.
/// Each placement corresponds 1:1 to a in the input.
/// This 1:1 mapping applies uniformly to both `Node` and `Fragment`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Placement {
    /// Offset position (x, y) relative to the container
    pub offset: (f32, f32),
    /// 0-indexed line index where the item is placed
    pub line_index: usize,
}
