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
    pub fn iter(&self) -> impl Iterator<Item = BoxModel> {
        self.into_iter()
    }
}

// =============================================
//   Implementing IntoIterator for LayoutBox
// =============================================

impl IntoIterator for &LayoutBox {
    type Item = BoxModel;
    type IntoIter = std::vec::IntoIter<BoxModel>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            LayoutBox::None => Vec::new().into_iter(),

            LayoutBox::BlockBox(b) => vec![b.clone()].into_iter(),

            LayoutBox::InlineBox(inline) => {
                let len = inline.line_spans.len();

                let left_extra_border =
                    inline.box_model.padding_box.x - inline.box_model.border_box.x;
                let right_extra_border =
                    inline.box_model.border_box.right() - inline.box_model.padding_box.right();
                let left_extra_padding =
                    inline.box_model.content_box.x - inline.box_model.padding_box.x;
                let right_extra_padding =
                    inline.box_model.padding_box.right() - inline.box_model.content_box.right();

                inline
                    .line_spans
                    .iter()
                    .map(|span| {
                        let i = span.line_index;
                        let mut b = inline.box_model.clone();

                        // shift
                        let dx = span.line_pos.0 - b.content_box.x;
                        let dy = span.line_pos.1 - b.content_box.y;
                        b.shift(dx, dy);

                        let new_content_width = span.width();

                        // decide which sides to keep
                        let keep_left = i == 0;
                        let keep_right = i == len - 1;

                        let left_padding = if keep_left { left_extra_padding } else { 0.0 };
                        let right_padding = if keep_right { right_extra_padding } else { 0.0 };
                        let left_border = if keep_left { left_extra_border } else { 0.0 };
                        let right_border = if keep_right { right_extra_border } else { 0.0 };

                        // set content box
                        b.content_box.width = new_content_width;
                        b.content_box.x = left_padding;

                        // rebuild outer boxes
                        b.padding_box.x = left_border;
                        b.padding_box.width = new_content_width + left_padding + right_padding;
                        b.border_box.width = b.padding_box.width + left_border + right_border;
                        b.children_box = b.content_box;

                        b
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
            }
        }
    }
}

impl IntoIterator for LayoutBox {
    type Item = BoxModel;
    type IntoIter = std::vec::IntoIter<BoxModel>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            LayoutBox::None => Vec::new().into_iter(),

            LayoutBox::BlockBox(b) => vec![b].into_iter(),

            LayoutBox::InlineBox(inline) => {
                let len = inline.line_spans.len();

                let base = inline.box_model;
                let spans = inline.line_spans;

                let left_extra_border = base.padding_box.x - base.border_box.x;
                let right_extra_border = base.border_box.right() - base.padding_box.right();
                let left_extra_padding = base.content_box.x - base.padding_box.x;
                let right_extra_padding = base.padding_box.right() - base.content_box.right();

                spans
                    .iter()
                    .map(|span| {
                        let i = span.line_index;
                        let mut b = base.clone();

                        // shift
                        let dx = span.line_pos.0 - b.content_box.x;
                        let dy = span.line_pos.1 - b.content_box.y;
                        b.shift(dx, dy);

                        let new_content_width = span.width();

                        // decide which sides to keep
                        let keep_left = i == 0;
                        let keep_right = i == len - 1;

                        let left_padding = if keep_left { left_extra_padding } else { 0.0 };
                        let right_padding = if keep_right { right_extra_padding } else { 0.0 };
                        let left_border = if keep_left { left_extra_border } else { 0.0 };
                        let right_border = if keep_right { right_extra_border } else { 0.0 };

                        // set content box
                        b.content_box.width = new_content_width;
                        b.content_box.x = left_padding;

                        // rebuild outer boxes
                        b.padding_box.x = left_border;
                        b.padding_box.width = new_content_width + left_padding + right_padding;
                        b.border_box.width = b.padding_box.width + left_border + right_border;
                        b.children_box = b.content_box;

                        b
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
            }
        }
    }
}
