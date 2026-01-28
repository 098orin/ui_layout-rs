/// An item fragment for inline layout.
///
/// An item fragment can be either a splittable fragment of inline content
/// or a control character like a line break.
#[derive(Debug, Clone, Copy)]
pub enum ItemFragment {
    /// A splittable fragment of inline content.
    Fragment(Fragment),
    /// A control character representing a line break.
    LineBreak,
}

/// Input fragments owned by this node.
///
/// Fragments are the smallest splittable logical units for inline layout.
/// They carry size and intrinsic shape information only and do not contain
/// positional data.
#[derive(Debug, Clone, Copy)]
pub struct Fragment {
    width: f32,
    height: f32,
}

/// Layout result for inline fragments.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FragmentPlacement {
    offset: (f32, f32),
    line_index: usize,
}
