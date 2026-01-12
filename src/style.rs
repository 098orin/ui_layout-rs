#[derive(Debug, Clone, Copy, Default)]
pub enum Display {
    Flex {
        flex_direction: FlexDirection,
    },
    #[default]
    Block,
    None,
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

impl Length {
    // Resolve Length
    //
    // If the containing block’s is `auto`, then the percentage is treated as `auto` for the purpose of layout.
    pub fn resolve_with(&self, containing_block: Option<f32>, viewport: f32) -> Option<f32> {
        match self {
            Length::Auto => None,
            Length::Px(v) => Some(*v),
            Length::Percent(p) => {
                if let Some(cb) = containing_block {
                    Some(cb * *p / 100.0)
                } else {
                    None
                }
            }
            Length::Vw(v) => Some(viewport * *v / 100.0),
            Length::Vh(v) => Some(viewport * *v / 100.0),
            Length::Add(a, b) => Some(
                a.resolve_with(containing_block, viewport)?
                    + b.resolve_with(containing_block, viewport)?,
            ),
            Length::Sub(a, b) => Some(
                a.resolve_with(containing_block, viewport)?
                    - b.resolve_with(containing_block, viewport)?,
            ),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ItemStyle {
    pub flex_grow: f32,
    pub flex_basis: Length,
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

#[derive(Debug, Clone, Default)]
pub struct Spacing {
    pub margin_top: Length,
    pub margin_bottom: Length,
    pub margin_left: Length,
    pub margin_right: Length,

    pub padding_top: Length,
    pub padding_bottom: Length,
    pub padding_left: Length,
    pub padding_right: Length,
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
    pub column_gap: Length,
    pub row_gap: Length,
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
