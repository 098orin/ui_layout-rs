//! Geometry-related types and implementations.

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Represents the layout box model of an element.
///
/// Each field is expressed in absolute coordinates relative to the
/// `border_box` origin.
#[derive(Debug, Clone, Default)]
pub struct BoxModel {
    /// The outermost box including border.
    pub border_box: Rect,
    /// The box inside the border, including padding.
    pub padding_box: Rect,
    /// The innermost box where actual content is placed.
    pub content_box: Rect,
    /// The area covering all child content.
    pub children_box: Rect,
}

/// An inline box that may be split across multiple lines.
///
/// A single logical box is represented together with its per-line [`LineSpan`].
#[derive(Debug, Clone)]
pub struct InlineBox {
    /// The original (unsplit) box model.
    pub box_model: BoxModel,
    /// Span infomation of this box on each line.
    pub line_spans: Vec<LineSpan>,
}

/// A span of an inline box on a single line.
#[derive(Debug, Clone)]
pub struct LineSpan {
    /// X-axis range inside the flat inline box.
    /// This range is unaffected by line positioning.
    pub x_range: std::ops::Range<f32>,
    /// Line position.
    pub line_pos: (f32, f32),
    /// 0-based line index.
    pub line_index: usize,
}

impl LineSpan {
    /*
    /// Shift [`Self::line_pos`].
    fn shift(&mut self, dx: f32, dy: f32) {
        self.line_pos.0 += dx;
        self.line_pos.1 += dy;
    }
    */

    pub fn width(&self) -> f32 {
        self.x_range.end - self.x_range.start
    }
}

/// Types of BoxModel.
///
/// All coordinates are relative to the parent.
#[derive(Debug, Clone, Default)]
pub enum LayoutBox {
    #[default]
    None,
    BlockBox(BoxModel),
    InlineBox(InlineBox),
}

#[derive(Debug, Clone, Copy)]
struct InlineBoxEdges {
    left_border: f32,
    right_border: f32,
    left_padding: f32,
    right_padding: f32,
}

impl InlineBoxEdges {
    fn new(base: &BoxModel) -> Self {
        Self {
            left_border: base.padding_box.x - base.border_box.x,
            right_border: base.border_box.right() - base.padding_box.right(),
            left_padding: base.content_box.x - base.padding_box.x,
            right_padding: base.padding_box.right() - base.content_box.right(),
        }
    }
}

/// Iterator over the [`BoxModel`]s represented by a borrowed [`LayoutBox`].
///
/// Inline boxes are converted one line at a time, avoiding the intermediate
/// allocation that a collected `Vec<BoxModel>` would require.
#[derive(Debug)]
pub struct LayoutBoxIter<'a> {
    inner: LayoutBoxIterInner<'a>,
}

#[derive(Debug)]
enum LayoutBoxIterInner<'a> {
    Empty,
    Block(Option<&'a BoxModel>),
    Inline {
        base: &'a BoxModel,
        spans: std::slice::Iter<'a, LineSpan>,
        len: usize,
        edges: InlineBoxEdges,
    },
}

/// Iterator over the [`BoxModel`]s represented by an owned [`LayoutBox`].
///
/// This consumes inline line spans directly and yields each computed box lazily.
#[derive(Debug)]
pub struct LayoutBoxIntoIter {
    inner: LayoutBoxIntoIterInner,
}

#[derive(Debug)]
enum LayoutBoxIntoIterInner {
    Empty,
    Block(Option<BoxModel>),
    Inline {
        base: BoxModel,
        spans: std::vec::IntoIter<LineSpan>,
        len: usize,
        edges: InlineBoxEdges,
    },
}

impl Rect {
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    pub fn size(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    fn shift(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
    }
}

impl BoxModel {
    /// Translates the entire box by the given offset.
    pub(crate) fn shift(&mut self, dx: f32, dy: f32) {
        self.border_box.shift(dx, dy);
        self.padding_box.shift(dx, dy);
        self.content_box.shift(dx, dy);
        self.children_box.shift(dx, dy);
    }

    /// Returns the width based on border-box dimensions.
    pub fn width(&self) -> f32 {
        self.border_box.width
    }

    /// Returns the height based on border-box dimensions.
    pub fn height(&self) -> f32 {
        self.border_box.height
    }
}

impl LayoutBox {
    pub(crate) fn shift(&mut self, dx: f32, dy: f32) {
        match self {
            LayoutBox::None => {}
            LayoutBox::BlockBox(b) => b.shift(dx, dy),
            LayoutBox::InlineBox(inline) => {
                inline.box_model.shift(dx, dy);
            }
        }
    }

    /// Returns the maximum width among all boxes via [`LineSpan::width`] or[`BoxModel::width`].
    /// For [`LayoutBox::InlineBox`], width is calclated via [`LineSpan::width`]
    pub fn width_box(&self) -> f32 {
        match self {
            LayoutBox::None => 0.0,
            LayoutBox::BlockBox(b) => b.width(),
            LayoutBox::InlineBox(l) => l
                .line_spans
                .iter()
                .map(|s| s.width())
                .filter(|v| !v.is_nan())
                .max_by(f32::total_cmp)
                .unwrap_or(0.0),
        }
    }

    /// Returns the width of box.
    /// See [`BoxModel::width`].
    pub fn width(&self) -> f32 {
        match self {
            LayoutBox::None => 0.0,
            LayoutBox::BlockBox(b) => b.width(),
            LayoutBox::InlineBox(l) => l.box_model.width(),
        }
    }

    /// Returns the maximum width among all boxes via [`BoxModel::height`].
    /// For [`LayoutBox::InlineBox`], height is calclated via sum of height for every line.
    pub fn height_box(&self) -> f32 {
        match self {
            LayoutBox::None => 0.0,
            LayoutBox::BlockBox(b) => b.width(),
            LayoutBox::InlineBox(l) => l.box_model.content_box.height * (l.line_spans.len() as f32),
        }
    }

    /// Returns the total height.
    /// See [`BoxModel::height`].
    pub fn height(&self) -> f32 {
        match self {
            LayoutBox::None => 0.0,
            LayoutBox::BlockBox(b) => b.height(),
            LayoutBox::InlineBox(l) => {
                // Last y pos - First y pos + line border height
                if let (Some(first), Some(last)) = (l.line_spans.first(), l.line_spans.last()) {
                    last.line_pos.1 - first.line_pos.1 + l.box_model.height()
                } else {
                    0.0
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            LayoutBox::None => true,
            LayoutBox::BlockBox(_) | LayoutBox::InlineBox(_) => false,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            LayoutBox::None => 0,
            LayoutBox::BlockBox(_) => 1,
            LayoutBox::InlineBox(v) => v.line_spans.len(),
        }
    }

    /// Returns an iterator over references to the contained [`BoxModel`]s.
    ///
    /// The iteration order is:
    /// - empty for [`LayoutBox::None`]
    /// - a single element for [`LayoutBox::BlockBox`]
    /// - the order of elements in the inner vector for [`LayoutBox::InlineBox`]
    ///
    /// This method provides a convenient way to iterate over all boxes
    /// regardless of the internal representation.
    pub fn iter(&self) -> LayoutBoxIter<'_> {
        self.into_iter()
    }
}

// =============================================
//   Implementing IntoIterator for LayoutBox
// =============================================

fn line_box(base: &BoxModel, span: &LineSpan, len: usize, edges: InlineBoxEdges) -> BoxModel {
    let mut b = base.clone();

    let dx = span.line_pos.0 - b.content_box.x;
    let dy = span.line_pos.1 - b.content_box.y;
    b.shift(dx, dy);

    let new_content_width = span.width();
    let keep_left = span.line_index == 0;
    let keep_right = span.line_index == len - 1;

    let left_padding = if keep_left { edges.left_padding } else { 0.0 };
    let right_padding = if keep_right { edges.right_padding } else { 0.0 };
    let left_border = if keep_left { edges.left_border } else { 0.0 };
    let right_border = if keep_right { edges.right_border } else { 0.0 };

    b.content_box.width = new_content_width;
    b.content_box.x = left_padding;

    b.padding_box.x = left_border;
    b.padding_box.width = new_content_width + left_padding + right_padding;
    b.border_box.width = b.padding_box.width + left_border + right_border;
    b.children_box = b.content_box;

    b
}

impl<'a> IntoIterator for &'a LayoutBox {
    type Item = BoxModel;
    type IntoIter = LayoutBoxIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        LayoutBoxIter {
            inner: match self {
                LayoutBox::None => LayoutBoxIterInner::Empty,
                LayoutBox::BlockBox(b) => LayoutBoxIterInner::Block(Some(b)),
                LayoutBox::InlineBox(inline) => LayoutBoxIterInner::Inline {
                    base: &inline.box_model,
                    spans: inline.line_spans.iter(),
                    len: inline.line_spans.len(),
                    edges: InlineBoxEdges::new(&inline.box_model),
                },
            },
        }
    }
}

impl Iterator for LayoutBoxIter<'_> {
    type Item = BoxModel;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            LayoutBoxIterInner::Empty => None,
            LayoutBoxIterInner::Block(b) => b.take().cloned(),
            LayoutBoxIterInner::Inline {
                base,
                spans,
                len,
                edges,
            } => spans.next().map(|span| line_box(base, span, *len, *edges)),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl DoubleEndedIterator for LayoutBoxIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            LayoutBoxIterInner::Empty => None,
            LayoutBoxIterInner::Block(b) => b.take().cloned(),
            LayoutBoxIterInner::Inline {
                base,
                spans,
                len,
                edges,
            } => spans
                .next_back()
                .map(|span| line_box(base, span, *len, *edges)),
        }
    }
}

impl ExactSizeIterator for LayoutBoxIter<'_> {
    fn len(&self) -> usize {
        match &self.inner {
            LayoutBoxIterInner::Empty => 0,
            LayoutBoxIterInner::Block(b) => usize::from(b.is_some()),
            LayoutBoxIterInner::Inline { spans, .. } => spans.len(),
        }
    }
}

impl std::iter::FusedIterator for LayoutBoxIter<'_> {}

impl IntoIterator for LayoutBox {
    type Item = BoxModel;
    type IntoIter = LayoutBoxIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        LayoutBoxIntoIter {
            inner: match self {
                LayoutBox::None => LayoutBoxIntoIterInner::Empty,
                LayoutBox::BlockBox(b) => LayoutBoxIntoIterInner::Block(Some(b)),
                LayoutBox::InlineBox(inline) => {
                    let edges = InlineBoxEdges::new(&inline.box_model);
                    LayoutBoxIntoIterInner::Inline {
                        base: inline.box_model,
                        len: inline.line_spans.len(),
                        spans: inline.line_spans.into_iter(),
                        edges,
                    }
                }
            },
        }
    }
}

impl Iterator for LayoutBoxIntoIter {
    type Item = BoxModel;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            LayoutBoxIntoIterInner::Empty => None,
            LayoutBoxIntoIterInner::Block(b) => b.take(),
            LayoutBoxIntoIterInner::Inline {
                base,
                spans,
                len,
                edges,
            } => spans.next().map(|span| line_box(base, &span, *len, *edges)),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl DoubleEndedIterator for LayoutBoxIntoIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        match &mut self.inner {
            LayoutBoxIntoIterInner::Empty => None,
            LayoutBoxIntoIterInner::Block(b) => b.take(),
            LayoutBoxIntoIterInner::Inline {
                base,
                spans,
                len,
                edges,
            } => spans
                .next_back()
                .map(|span| line_box(base, &span, *len, *edges)),
        }
    }
}

impl ExactSizeIterator for LayoutBoxIntoIter {
    fn len(&self) -> usize {
        match &self.inner {
            LayoutBoxIntoIterInner::Empty => 0,
            LayoutBoxIntoIterInner::Block(b) => usize::from(b.is_some()),
            LayoutBoxIntoIterInner::Inline { spans, .. } => spans.len(),
        }
    }
}

impl std::iter::FusedIterator for LayoutBoxIntoIter {}
