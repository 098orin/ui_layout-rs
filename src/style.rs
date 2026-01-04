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
    pub flex_basis: Option<f32>,
    pub align_self: Option<AlignItems>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SizeStyle {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Spacing {
    pub margin_top: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub margin_right: f32,
    pub padding_top: f32,
    pub padding_bottom: f32,
    pub padding_left: f32,
    pub padding_right: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum JustifyContent {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum AlignItems {
    Start,
    Center,
    End,
    #[default]
    Stretch,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Style {
    pub display: Display,
    pub item_style: ItemStyle,
    pub size: SizeStyle,
    pub spacing: Spacing,

    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub column_gap: f32,
    pub row_gap: f32,
}
