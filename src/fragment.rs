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
