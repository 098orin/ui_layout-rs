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

#[derive(Debug, Clone, Default)]
pub enum Length {
    Px(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
    #[default]
    Auto,
    // calc
    Add(Box<Length>, Box<Length>),
    Sub(Box<Length>, Box<Length>),
}

#[derive(Debug, Clone, Default)]
pub struct ItemStyle {
    pub flex_grow: f32,
    pub flex_basis: Option<f32>,
    pub align_self: Option<AlignItems>,
}

#[derive(Debug, Clone, Default)]
pub struct SizeStyle {
    pub width: Length,
    pub height: Length,
    pub min_width: Length,
    pub max_width: Length,
    pub min_height: Length,
    pub max_height: Length,
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

#[derive(Debug, Clone, Default)]
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

// =======================

use std::ops::{Add, Sub};

impl Add for Length {
    type Output = Length;

    fn add(self, rhs: Length) -> Length {
        Length::Add(Box::new(self), Box::new(rhs))
    }
}

impl Sub for Length {
    type Output = Length;

    fn sub(self, rhs: Length) -> Length {
        Length::Sub(Box::new(self), Box::new(rhs))
    }
}
