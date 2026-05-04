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

#[derive(Debug, Clone)]
pub struct InlineBox {
    pub box_model: BoxModel,
    pub line_spans: Vec<LineSpan>,
}

#[derive(Debug, Clone)]
pub struct LineSpan {
    pub start_pos: (f32, f32),
    pub end_x_pos: f32,
    /// 0-indexed line index.
    pub line_index: usize,
}

/// Types of BoxModel.
///
/// All coordinates are relative to the parent.
#[derive(Debug, Clone, Default)]
pub enum LayoutBoxes {
    #[default]
    /// No layout boxes.
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

impl LayoutBoxes {
    /// Returns the maximum width among all boxes.
    /// See [`BoxModel::width`].
    pub fn width(&self) -> f32 {
        match self {
            LayoutBoxes::None => 0.0,
            LayoutBoxes::BlockBox(b) => b.width(),
            LayoutBoxes::InlineBox(l) => l.box_model.width(),
        }
    }

    /// Returns the total height.
    /// See [`BoxModel::height`].
    pub fn height(&self) -> f32 {
        match self {
            LayoutBoxes::None => 0.0,
            LayoutBoxes::BlockBox(b) => b.height(),
            LayoutBoxes::InlineBox(l) => {
                // Last y pos - First y pos + line border height
                if let (Some(first), Some(last)) = (l.line_spans.first(), l.line_spans.last()) {
                    last.start_pos.1 - first.start_pos.1 + l.box_model.height()
                } else {
                    0.0
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            LayoutBoxes::None => true,
            LayoutBoxes::BlockBox(_) | LayoutBoxes::InlineBox(_) => false,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            LayoutBoxes::None => 0,
            LayoutBoxes::BlockBox(_) => 1,
            LayoutBoxes::InlineBox(v) => v.line_spans.len(),
        }
    }

    /// Returns an iterator over references to the contained [`BoxModel`]s.
    ///
    /// The iteration order is:
    /// - empty for [`LayoutBoxes::None`]
    /// - a single element for [`LayoutBoxes::Single`]
    /// - the order of elements in the inner vector for [`LayoutBoxes::Multiple`]
    ///
    /// This method provides a convenient way to iterate over all boxes
    /// regardless of the internal representation.
    pub fn iter(&self) -> impl Iterator<Item = &BoxModel> {
        self.into_iter()
    }

    /// Returns an iterator over mutable references to the contained [`BoxModel`]s.
    ///
    /// The iteration order is:
    /// - empty for [`LayoutBoxes::None`]
    /// - a single element for [`LayoutBoxes::Single`]
    /// - the order of elements in the inner vector for [`LayoutBoxes::Multiple`]
    ///
    /// This allows in-place modification of all boxes in a uniform way.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut BoxModel> {
        self.into_iter()
    }
}

// =============================================
//   Implementing IntoIterator for LayoutBoxes
// =============================================

impl<'a> IntoIterator for &'a LayoutBoxes {
    type Item = &'a BoxModel;
    type IntoIter = std::slice::Iter<'a, BoxModel>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            LayoutBoxes::None => [].iter(),
            LayoutBoxes::BlockBox(b) => std::slice::from_ref(b).iter(),
            LayoutBoxes::Multiple(list) => list.iter(),
        }
    }
}

impl<'a> IntoIterator for &'a mut LayoutBoxes {
    type Item = &'a mut BoxModel;
    type IntoIter = std::slice::IterMut<'a, BoxModel>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            LayoutBoxes::None => [].iter_mut(),
            LayoutBoxes::BlockBox(b) => std::slice::from_mut(b).iter_mut(),
            LayoutBoxes::Multiple(list) => list.iter_mut(),
        }
    }
}

impl IntoIterator for LayoutBoxes {
    type Item = BoxModel;
    type IntoIter = std::vec::IntoIter<BoxModel>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            LayoutBoxes::None => Vec::new().into_iter(),
            LayoutBoxes::BlockBox(b) => vec![b].into_iter(),
            LayoutBoxes::Multiple(list) => list.into_iter(),
        }
    }
}
