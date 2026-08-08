/// Represents the outer display type of a box.
///
/// This corresponds to how the element participates in the parent formatting context.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OuterDisplay {
    #[default]
    Block,
    Inline,
    None,
}

/// Represents the inner display type of a box.
///
/// This defines how children are laid out inside the element.
/// In CSS terms, this is the "inner display type".
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum InnerDisplay {
    #[default]
    Flow,
    FlowRoot,
    Flex,
    Grid,
}

// for future implementation:
// https://drafts.csswg.org/css-display/#the-display-properties

/// Full representation of the CSS `display` property,
/// split into outer and inner display types.
///
/// This follows the modern CSS Display specification:
/// <https://www.w3.org/TR/css-display-3/>
///
/// Examples:
/// - `block`        => (Block, Flow)
/// - `inline`       => (Inline, Flow)
/// - `flex`         => (Block, Flex)
/// - `inline-flex`  => (Inline, Flex)
/// - `flow-root`    => (Block, FlowRoot)
/// - `inline-block` => (Inline, FlowRoot)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Display {
    pub outer: OuterDisplay,
    pub inner: InnerDisplay,
}

impl Display {
    /// Parses a single-keyword CSS `display` value.
    ///
    /// This handles legacy single-keyword forms like:
    /// - `block`
    /// - `inline`
    /// - `flex`
    /// - `inline-flex`
    /// - `flow-root`
    /// - `inline-block`
    /// - `none`
    ///
    /// Returns `None` if the keyword is not recognized.
    ///
    /// Note:
    /// This function does NOT support multi-keyword syntax like
    /// `display: inline flex`. Use `from_css` for that.
    pub fn from_css_name(name: &str) -> Option<Self> {
        match name {
            "block" => Some(Self {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flow,
            }),
            "inline" => Some(Self {
                outer: OuterDisplay::Inline,
                inner: InnerDisplay::Flow,
            }),
            "none" => Some(Self {
                outer: OuterDisplay::None,
                inner: InnerDisplay::Flow,
            }),
            "flex" => Some(Self {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Flex,
            }),
            "inline-flex" => Some(Self {
                outer: OuterDisplay::Inline,
                inner: InnerDisplay::Flex,
            }),
            "grid" => Some(Self {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Grid,
            }),
            "inline-grid" => Some(Self {
                outer: OuterDisplay::Inline,
                inner: InnerDisplay::Grid,
            }),
            "flow-root" => Some(Self {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::FlowRoot,
            }),
            "inline-block" => Some(Self {
                outer: OuterDisplay::Inline,
                inner: InnerDisplay::FlowRoot,
            }),
            _ => None,
        }
    }

    /// Parses a CSS `display` value that may contain multiple tokens.
    ///
    /// This supports the modern syntax like:
    /// - `display: block flow`
    /// - `display: inline flex`
    /// - `display: block flow-root`
    /// - `display: inline flow-root`
    ///
    /// Returns a tuple:
    /// `(outer, inner)`
    ///
    /// Each component is optional because:
    /// - CSS allows partial specification
    /// - Missing parts are resolved later via defaults or cascading rules
    ///
    /// Example:
    /// ```
    /// # use ui_layout::{Display, OuterDisplay, InnerDisplay};
    ///
    /// let (outer, inner) = Display::from_css("inline flex");
    /// assert_eq!(outer, Some(OuterDisplay::Inline));
    /// assert_eq!(inner, Some(InnerDisplay::Flex));
    ///
    /// let (outer, inner) = Display::from_css("block flow-root");
    /// assert_eq!(outer, Some(OuterDisplay::Block));
    /// assert_eq!(inner, Some(InnerDisplay::FlowRoot));
    ///
    /// let (outer, inner) = Display::from_css("inline flow-root");
    /// assert_eq!(outer, Some(OuterDisplay::Inline));
    /// assert_eq!(inner, Some(InnerDisplay::FlowRoot));
    /// ```
    ///
    /// Unknown tokens are ignored.
    ///
    /// Note:
    /// This function does not resolve final computed values.
    /// It only performs syntactic parsing.
    pub fn from_css(input: &str) -> (Option<OuterDisplay>, Option<InnerDisplay>) {
        let mut outer = None;
        let mut inner = None;

        for token in input.split_whitespace() {
            match token {
                "block" => outer = Some(OuterDisplay::Block),
                "inline" => outer = Some(OuterDisplay::Inline),
                "none" => outer = Some(OuterDisplay::None),

                "flow" => inner = Some(InnerDisplay::Flow),
                "flow-root" => inner = Some(InnerDisplay::FlowRoot),
                "flex" => inner = Some(InnerDisplay::Flex),
                "grid" => inner = Some(InnerDisplay::Grid),

                _ => {}
            }
        }

        (outer, inner)
    }

    /// Unified parser for CSS `display`.
    ///
    /// Internally:
    /// 1. Try single-keyword parsing (`from_css_name`)
    /// 2. Fallback to multi-token parsing (`from_css`)
    /// 3. Resolve defaults
    ///
    /// Returns `None` if nothing could be parsed.
    ///
    ///
    /// # Examples
    ///
    /// Basic single-keyword values:
    ///
    /// ```
    /// # use ui_layout::*;
    /// let d = Display::parse("block").unwrap();
    /// assert_eq!(d.outer, OuterDisplay::Block);
    /// assert_eq!(d.inner, InnerDisplay::Flow);
    ///
    /// let d = Display::parse("inline").unwrap();
    /// assert_eq!(d.outer, OuterDisplay::Inline);
    /// assert_eq!(d.inner, InnerDisplay::Flow);
    /// ```
    ///
    /// Flex values:
    ///
    /// ```
    /// # use ui_layout::*;
    /// let d = Display::parse("flex").unwrap();
    /// assert_eq!(d.outer, OuterDisplay::Block);
    /// assert_eq!(d.inner, InnerDisplay::Flex);
    ///
    /// let d = Display::parse("inline-flex").unwrap();
    /// assert_eq!(d.outer, OuterDisplay::Inline);
    /// assert_eq!(d.inner, InnerDisplay::Flex);
    /// ```
    ///
    /// Flow-root values:
    ///
    /// ```
    /// # use ui_layout::*;
    /// let d = Display::parse("flow-root").unwrap();
    /// assert_eq!(d.outer, OuterDisplay::Block);
    /// assert_eq!(d.inner, InnerDisplay::FlowRoot);
    ///
    /// let d = Display::parse("inline-block").unwrap();
    /// assert_eq!(d.outer, OuterDisplay::Inline);
    /// assert_eq!(d.inner, InnerDisplay::FlowRoot);
    /// ```
    ///
    /// Multi-keyword syntax:
    ///
    /// ```
    /// # use ui_layout::*;
    /// let d = Display::parse("inline flex").unwrap();
    /// assert_eq!(d.outer, OuterDisplay::Inline);
    /// assert_eq!(d.inner, InnerDisplay::Flex);
    ///
    /// let d = Display::parse("block flow").unwrap();
    /// assert_eq!(d.outer, OuterDisplay::Block);
    /// assert_eq!(d.inner, InnerDisplay::Flow);
    ///
    /// let d = Display::parse("block flow-root").unwrap();
    /// assert_eq!(d.outer, OuterDisplay::Block);
    /// assert_eq!(d.inner, InnerDisplay::FlowRoot);
    ///
    /// let d = Display::parse("inline flow-root").unwrap();
    /// assert_eq!(d.outer, OuterDisplay::Inline);
    /// assert_eq!(d.inner, InnerDisplay::FlowRoot);
    /// ```
    ///
    /// Missing parts are filled with defaults:
    ///
    /// ```
    /// # use ui_layout::*;
    /// let d = Display::parse("flex").unwrap();
    /// // outer defaults to Block
    /// assert_eq!(d.outer, OuterDisplay::Block);
    ///
    /// let d = Display::parse("inline").unwrap();
    /// // inner defaults to Flow
    /// assert_eq!(d.inner, InnerDisplay::Flow);
    /// ```
    ///
    /// Special case: `none`
    ///
    /// ```
    /// # use ui_layout::*;
    /// let d = Display::parse("none").unwrap();
    /// assert_eq!(d.outer, OuterDisplay::None);
    /// assert_eq!(d.inner, InnerDisplay::Flow);
    /// ```
    ///
    /// Invalid input:
    ///
    /// ```
    /// # use ui_layout::*;
    /// assert!(Display::parse("unknown").is_none());
    /// assert!(Display::parse("").is_none());
    /// ```
    pub fn parse(input: &str) -> Option<Self> {
        // Fast path: single keyword (also handles inline-flex etc.)
        if let Some(display) = Self::from_css_name(input.trim()) {
            return Some(display);
        }

        // Fallback: multi-token parsing
        let (outer, inner) = Self::from_css(input);

        // Nothing recognized
        if outer.is_none() && inner.is_none() {
            return None;
        }

        // Special case: none
        if matches!(outer, Some(OuterDisplay::None)) {
            return Some(Self {
                outer: OuterDisplay::None,
                inner: InnerDisplay::Flow,
            });
        }

        // Resolve defaults
        let outer = outer.unwrap_or(OuterDisplay::Block);
        let inner = inner.unwrap_or(InnerDisplay::Flow);

        Some(Self { outer, inner })
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Length {
    Px(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
    // calc
    Add(Box<Length>, Box<Length>),
    Sub(Box<Length>, Box<Length>),
    Mul(Box<Length>, f32),
    Div(Box<Length>, f32),

    Min(Box<Length>, Box<Length>),
    Max(Box<Length>, Box<Length>),
    Clamp {
        min: Box<Length>,
        val: Box<Length>,
        max: Box<Length>,
    },
}

impl Default for Length {
    fn default() -> Self {
        Length::Px(0.0)
    }
}

impl Length {
    /// Resolves a length value to pixels.
    ///
    /// Unresolvable values will be [`Option::None`].
    pub fn resolve_with(
        &self,
        percentage_base: Option<f32>,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Option<f32> {
        match self {
            Length::Px(v) => Some(*v),
            Length::Percent(p) => percentage_base.map(|cb| cb * *p / 100.0),
            Length::Vw(v) => Some(viewport_width * *v / 100.0),
            Length::Vh(v) => Some(viewport_height * *v / 100.0),
            Length::Add(a, b) => Some(
                a.resolve_with(percentage_base, viewport_width, viewport_height)?
                    + b.resolve_with(percentage_base, viewport_width, viewport_height)?,
            ),
            Length::Sub(a, b) => Some(
                a.resolve_with(percentage_base, viewport_width, viewport_height)?
                    - b.resolve_with(percentage_base, viewport_width, viewport_height)?,
            ),
            Length::Mul(a, n) => {
                Some(a.resolve_with(percentage_base, viewport_width, viewport_height)? * n)
            }
            Length::Div(a, n) => {
                if *n == 0.0 {
                    None
                } else {
                    Some(a.resolve_with(percentage_base, viewport_width, viewport_height)? / n)
                }
            }
            Length::Min(a, b) => Some(
                a.resolve_with(percentage_base, viewport_width, viewport_height)?
                    .min(b.resolve_with(percentage_base, viewport_width, viewport_height)?),
            ),
            Length::Max(a, b) => Some(
                a.resolve_with(percentage_base, viewport_width, viewport_height)?
                    .max(b.resolve_with(percentage_base, viewport_width, viewport_height)?),
            ),
            Length::Clamp { min, val, max } => {
                let v = val.resolve_with(percentage_base, viewport_width, viewport_height)?;
                let min_v = min.resolve_with(percentage_base, viewport_width, viewport_height)?;
                let max_v = max.resolve_with(percentage_base, viewport_width, viewport_height)?;

                Some(v.clamp(min_v, max_v))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum LengthOrAuto {
    Length(Length),
    #[default]
    Auto,
}

impl LengthOrAuto {
    /// Resolves a length value to pixels.
    ///
    /// Unresolvable values will be [`Option::None`].
    pub fn resolve_with(
        &self,
        percentage_base: Option<f32>,
        viewport_width: f32,
        viewport_height: f32,
    ) -> Option<f32> {
        match self {
            LengthOrAuto::Length(l) => {
                l.resolve_with(percentage_base, viewport_width, viewport_height)
            }
            LengthOrAuto::Auto => None,
        }
    }

    pub fn length(self) -> Option<Length> {
        match self {
            LengthOrAuto::Length(l) => Some(l),
            LengthOrAuto::Auto => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStyle {
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub flex_basis: LengthOrAuto,
    pub align_self: Option<AlignItems>,
}

impl Default for ItemStyle {
    fn default() -> Self {
        ItemStyle {
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: LengthOrAuto::Auto,
            align_self: None,
        }
    }
}

/// How a block-level replaced element (a node wrapping exactly one
/// [`LayoutChild::Custom`], e.g. `<button>/<img>/<input>`) resolves an
/// `auto` width/height.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AutoSizeBehavior {
    /// Stretch to the containing block and fill the available space.
    #[default]
    Fill,
    /// Shrink to the custom child's intrinsic-based box.
    ShrinkToFit,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SizeStyle {
    pub width: LengthOrAuto,
    pub height: LengthOrAuto,
    pub min_width: LengthOrAuto,
    pub max_width: LengthOrAuto,
    pub min_height: LengthOrAuto,
    pub max_height: LengthOrAuto,
    /// Explicit `aspect-ratio` (width / height). `None` when unspecified.
    ///
    /// Used to derive the missing axis when only one of `width` / `height`
    /// is resolved. Takes precedence over the object's intrinsic ratio.
    pub aspect_ratio: Option<f32>,
    /// How `auto` width/height resolve for a block replaced-element leaf.
    pub auto_behavior: AutoSizeBehavior,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BoxSizing {
    #[default]
    ContentBox,
    BorderBox,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spacing {
    pub margin_top: LengthOrAuto,
    pub margin_bottom: LengthOrAuto,
    pub margin_left: LengthOrAuto,
    pub margin_right: LengthOrAuto,

    pub border_top: Length,
    pub border_bottom: Length,
    pub border_left: Length,
    pub border_right: Length,

    pub padding_top: Length,
    pub padding_bottom: Length,
    pub padding_left: Length,
    pub padding_right: Length,
}

impl Default for Spacing {
    fn default() -> Self {
        Spacing {
            margin_top: LengthOrAuto::Length(Length::default()),
            margin_bottom: LengthOrAuto::Length(Length::default()),
            margin_left: LengthOrAuto::Length(Length::default()),
            margin_right: LengthOrAuto::Length(Length::default()),
            border_top: Length::default(),
            border_bottom: Length::default(),
            border_left: Length::default(),
            border_right: Length::default(),
            padding_top: Length::default(),
            padding_bottom: Length::default(),
            padding_left: Length::default(),
            padding_right: Length::default(),
        }
    }
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

/// A single explicit or implicit CSS Grid track.
#[derive(Debug, Clone, PartialEq)]
pub enum GridTrack {
    /// A fixed, percentage, or content-sized track breadth.
    Breadth(LengthOrAuto),
    /// A fraction of the remaining grid container space.
    Flex(f32),
    /// A track clamped between a minimum and maximum breadth.
    MinMax(Box<GridTrack>, Box<GridTrack>),
    /// A repeated track list. Auto repeats are expanded from the available size.
    Repeat(GridRepeat, Vec<GridTrack>),
}

impl Default for GridTrack {
    fn default() -> Self {
        Self::Breadth(LengthOrAuto::Auto)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridRepeat {
    Count(usize),
    AutoFit,
    AutoFill,
}

/// Placement of a grid item on one axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridPlacement {
    /// One-based start line, or `None` for automatic placement.
    pub start: Option<usize>,
    /// Number of tracks occupied by the item.
    pub span: usize,
}

impl Default for GridPlacement {
    fn default() -> Self {
        Self {
            start: None,
            span: 1,
        }
    }
}

/// Controls whether a box participates in normal flow and establishes a
/// containing block for positioned descendants.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Position {
    #[default]
    Static,
    Relative,
    Absolute,
    Fixed,
}

impl Position {
    /// Returns whether the box is removed from normal flow.
    pub fn is_out_of_flow(self) -> bool {
        matches!(self, Self::Absolute | Self::Fixed)
    }
}

/// Positioning mode and offsets used to place a box.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PositionStyle {
    pub kind: Position,
    pub top: LengthOrAuto,
    pub right: LengthOrAuto,
    pub bottom: LengthOrAuto,
    pub left: LengthOrAuto,
}

trait PercentCheck {
    fn is_pct(&self) -> bool;
}

impl PercentCheck for Length {
    fn is_pct(&self) -> bool {
        matches!(self, Length::Percent(_))
    }
}

impl PercentCheck for LengthOrAuto {
    fn is_pct(&self) -> bool {
        matches!(self, LengthOrAuto::Length(Length::Percent(_)))
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Style {
    pub display: Display,

    pub position: PositionStyle,

    pub item_style: ItemStyle,
    pub size: SizeStyle,
    pub box_sizing: BoxSizing,
    pub spacing: Spacing,

    pub line_height: Length,

    pub justify_content: JustifyContent,
    pub align_items: AlignItems,

    pub flex_direction: FlexDirection,
    pub column_gap: LengthOrAuto,
    pub row_gap: LengthOrAuto,
    pub grid_template_columns: Vec<GridTrack>,
    pub grid_template_rows: Vec<GridTrack>,
    pub grid_template_areas: Vec<Vec<String>>,
    pub grid_column: GridPlacement,
    pub grid_row: GridPlacement,
    pub grid_area: Option<String>,
}

impl Style {
    pub(crate) fn has_percentage_size(&self) -> bool {
        self.size.width.is_pct()
            || self.size.height.is_pct()
            || self.size.min_width.is_pct()
            || self.size.max_width.is_pct()
            || self.size.min_height.is_pct()
            || self.size.max_height.is_pct()
            || self.spacing.margin_top.is_pct()
            || self.spacing.margin_bottom.is_pct()
            || self.spacing.margin_left.is_pct()
            || self.spacing.margin_right.is_pct()
            || self.spacing.padding_top.is_pct()
            || self.spacing.padding_bottom.is_pct()
            || self.spacing.padding_left.is_pct()
            || self.spacing.padding_right.is_pct()
            || self.spacing.border_top.is_pct()
            || self.spacing.border_bottom.is_pct()
            || self.spacing.border_left.is_pct()
            || self.spacing.border_right.is_pct()
            || self.item_style.flex_basis.is_pct()
            || self.column_gap.is_pct()
            || self.row_gap.is_pct()
            || self.line_height.is_pct()
            || self.position.top.is_pct()
            || self.position.right.is_pct()
            || self.position.bottom.is_pct()
            || self.position.left.is_pct()
    }
}

// =======================

use std::str::FromStr;

impl FromStr for Display {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse(input).ok_or(())
    }
}

impl From<Length> for LengthOrAuto {
    fn from(value: Length) -> Self {
        Self::Length(value)
    }
}

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
