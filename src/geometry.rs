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
    /// scroll size
    pub children_box: Rect,
}
