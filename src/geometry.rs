#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Default)]
pub struct BoxModel {
    pub border_box: Rect,
    pub padding_box: Rect,
    pub content_box: Rect,
    /// Scrollable content size
    pub children_box: Rect,
}

#[derive(Debug, Clone, Default)]
pub enum LayoutBoxes {
    #[default]
    /// No layout boxes.
    None,
    /// A single layout box.
    Single(BoxModel),
    /// Multiple layout boxes split across lines.
    /// Represents line fragments and may be empty.
    Multiple(Vec<BoxModel>),
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
    fn shift(&mut self, dx: f32, dy: f32) {
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
    pub(crate) fn shift(&mut self, dx: f32, dy: f32) {
        match self {
            LayoutBoxes::None => {}
            LayoutBoxes::Single(b) => b.shift(dx, dy),
            LayoutBoxes::Multiple(list) => {
                for b in list {
                    b.shift(dx, dy);
                }
            }
        }
    }

    /// Returns the maximum width among all boxes.
    /// See [`BoxModel::width`].
    pub fn width(&self) -> f32 {
        match self {
            LayoutBoxes::None => 0.0,
            LayoutBoxes::Single(b) => b.width(),
            LayoutBoxes::Multiple(list) => list.iter().map(|b| b.width()).fold(0.0, f32::max),
        }
    }

    /// Returns the total height.
    /// See [`BoxModel::height`].
    pub fn height(&self) -> f32 {
        match self {
            LayoutBoxes::None => 0.0,
            LayoutBoxes::Single(b) => b.height(),
            LayoutBoxes::Multiple(list) => {
                if list.is_empty() {
                    0.0
                } else {
                    let first_box = list.first().unwrap();
                    let last_box = list.last().unwrap();

                    let first_y = first_box.border_box.y;
                    let last_y = last_box.border_box.y + last_box.border_box.height;

                    (last_y - first_y).abs()
                }
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            LayoutBoxes::None => true,
            LayoutBoxes::Single(_) => false,
            LayoutBoxes::Multiple(v) => v.is_empty(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            LayoutBoxes::None => 0,
            LayoutBoxes::Single(_) => 1,
            LayoutBoxes::Multiple(v) => v.len(),
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
            LayoutBoxes::Single(b) => std::slice::from_ref(b).iter(),
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
            LayoutBoxes::Single(b) => std::slice::from_mut(b).iter_mut(),
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
            LayoutBoxes::Single(b) => vec![b].into_iter(),
            LayoutBoxes::Multiple(list) => list.into_iter(),
        }
    }
}
