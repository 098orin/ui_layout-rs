#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Display {
    Flex {
        flex_direction: FlexDirection,
    },
    #[default]
    Block,
    Inline,
    None,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    #[default]
    Column,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Length {
    Px(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
    Auto,
    // calc
    Add(Box<Length>, Box<Length>),
    Sub(Box<Length>, Box<Length>),
}

impl Default for Length {
    fn default() -> Self {
        Length::Px(0.0)
    }
}

impl Length {
    /// Resolves a length value to pixels.
    ///
    /// If the containing block is `auto`, percentages are treated as `auto` for layout purposes.
    /// This version uses a single viewport value (for backward compatibility with axis-based layout).
    pub fn resolve_with(
        &self,
        containing_block: Option<f32>,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Option<f32> {
        match self {
            Length::Auto => None,
            Length::Px(v) => Some(*v),
            Length::Percent(p) => containing_block.map(|cb| cb * *p / 100.0),
            Length::Vw(v) => Some(viewport_width * *v / 100.0),
            Length::Vh(v) => Some(viewport_height * *v / 100.0),
            Length::Add(a, b) => Some(
                a.resolve_with(containing_block, viewport_width, viewport_height)?
                    + b.resolve_with(containing_block, viewport_width, viewport_height)?,
            ),
            Length::Sub(a, b) => Some(
                a.resolve_with(containing_block, viewport_width, viewport_height)?
                    - b.resolve_with(containing_block, viewport_width, viewport_height)?,
            ),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStyle {
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: Length,
    pub align_self: Option<AlignItems>,
}

impl Default for ItemStyle {
    fn default() -> Self {
        ItemStyle {
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: Length::Auto,
            align_self: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SizeStyle {
    pub width: Length,
    pub height: Length,
    pub min_width: Length,
    pub max_width: Length,
    pub min_height: Length,
    pub max_height: Length,
}

impl Default for SizeStyle {
    fn default() -> Self {
        SizeStyle {
            width: Length::Auto,
            height: Length::Auto,
            min_width: Length::Auto,
            max_width: Length::Auto,
            min_height: Length::Auto,
            max_height: Length::Auto,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BoxSizing {
    #[default]
    ContentBox,
    BorderBox,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Spacing {
    pub margin_top: Length,
    pub margin_bottom: Length,
    pub margin_left: Length,
    pub margin_right: Length,

    pub border_top: Length,
    pub border_bottom: Length,
    pub border_left: Length,
    pub border_right: Length,

    pub padding_top: Length,
    pub padding_bottom: Length,
    pub padding_left: Length,
    pub padding_right: Length,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum JustifyContent {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AlignItems {
    Start,
    Center,
    End,
    #[default]
    Stretch,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Style {
    pub display: Display,
    pub item_style: ItemStyle,
    pub size: SizeStyle,
    pub box_sizing: BoxSizing,
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
