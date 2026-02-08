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
    None,
    Single(BoxModel),
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
    pub fn width(&self) -> f32 {
        match self {
            LayoutBoxes::None => 0.0,
            LayoutBoxes::Single(b) => b.width(),
            LayoutBoxes::Multiple(list) => list.iter().map(|b| b.width()).fold(0.0, f32::max),
        }
    }

    /// Returns the total height (sum of all boxes).
    pub fn height(&self) -> f32 {
        match self {
            LayoutBoxes::None => 0.0,
            LayoutBoxes::Single(b) => b.height(),
            LayoutBoxes::Multiple(list) => list.iter().map(|b| b.height()).sum(),
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
}
