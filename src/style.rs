#[derive(Debug, Clone, Copy)]
pub enum Display {
    Flex { flex_direction: FlexDirection },
    Block,
    None,
}

impl Default for Display {
    fn default() -> Self {
        Display::Flex {
            flex_direction: FlexDirection::Column,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum FlexDirection {
    Row,
    #[default]
    Column,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ItemStyle {
    pub flex_grow: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Style {
    pub display: Display,

    pub item_style: ItemStyle,

    pub width: Option<f32>,
    pub height: Option<f32>,

    pub padding: f32,
}
