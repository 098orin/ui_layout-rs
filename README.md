# ui_layout

[![Crates.io](https://img.shields.io/crates/v/ui_layout.svg)](https://crates.io/crates/ui_layout)
[![Docs.rs](https://docs.rs/ui_layout/badge.svg)](https://docs.rs/ui_layout)

A unified layout engine for Rust GUI development that treats Flexbox as a specialized form of Inline Block Flow layout.

This crate provides predictable layout system designed for
custom GUI frameworks, editors, and experimental UI engines.

> [!NOTE]
> This crate is under active development; patch releases may be frequent.

## Features

### Flexbox Support

- Flex layout (Row / Column direction, including `RowReverse` / `ColumnReverse`)
- `flex_grow` with proportional space distribution
- `flex_shrink` with proportional space reduction when overflowing
- `flex_basis` for initial sizing (supports `auto`, pixel values, and percentages)
- `justify_content` (Start, Center, End, SpaceBetween, SpaceAround, SpaceEvenly)
- `align_items` with full `stretch` support (Start, Center, End, Stretch)
- `align_self` for individual item alignment override
- Row and column gaps (`row_gap` / `column_gap`)
- Margin collapsing for block-level elements

### Block, Inline & Flow Root

- Block layout with full CSS Box Model support (margin, padding, border, `box-sizing`)
- Inline layout with multi-line wrapping
- `display: flow-root` and `display: inline-block` — establishes a new Block Formatting Context (margin collapsing isolation)
- `BoxSizing`: ContentBox and BorderBox

### Extensibility

- `FlowLayouter` trait for custom inline flow layout delegation
- `BlockLayouter` trait for custom block layout delegation (behind `feature = "unstable"`)

### Values & Units

- `Length` types: `Px`, `Percent`, `Vw`, `Vh`
- `calc()`-style expressions: `Add`, `Sub`, `Mul`, `Div`, `Min`, `Max`, `Clamp`
- `LengthOrAuto` for properties that support `auto` sizing
- Min/max sizing (`min_width`, `max_width`, `min_height`, `max_height`)
- `line_height` support

## Non-goals

- Full CSS compatibility
- Absolute / fixed positioning
- Web rendering or HTML/CSS parsing

## Design goals

- Simple and explicit layout rules
- Easy to reason about and debug
- Suitable for custom renderers (wgpu, skia, etc.)
- No dependency on web standards or DOM

## Example

```rust
use ui_layout::*;

// Create a flex container
let mut root = LayoutNode::with_children(
    Style {
        display: Display::parse("flex").unwrap(),
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(300.0)),
            height: LengthOrAuto::Length(Length::Px(200.0)),
            ..Default::default()
        },
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        flex_direction: FlexDirection::Row,
        column_gap: LengthOrAuto::Length(Length::Px(20.0)),
        ..Default::default()
    },
    [
        LayoutNode::new(Style {
            item_style: ItemStyle {
                flex_grow: 1.0,
                flex_basis: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        }),
        LayoutNode::new(Style {
            item_style: ItemStyle {
                flex_basis: LengthOrAuto::Length(Length::Px(100.0)),
                flex_shrink: 0.0,
                ..Default::default()
            },
            ..Default::default()
        }),
    ],
);

// Layout with viewport size
LayoutEngine::layout(&mut root, 800.0, 600.0);

// Access results
match &root.layout_box {
    LayoutBox::BlockBox(box_model) => {
        println!("Container: {}x{}", box_model.border_box.width, box_model.border_box.height);
    },
    _ => {}
}
```

For more examples and to understand the behavior of gaps, alignment, and sizing,
see the unit tests in the [`tests/`](tests/) directory. They provide practical usage patterns and expected layouts.

## Specification Compliance

This implementation follows CSS3 specifications with current focus on:

- ✅ **CSS Box Model Module Level 3**: Margin collapsing, padding, border, box-sizing support
- ✅ **CSS Flexible Box Layout Module Level 1**: Core flexbox algorithm including flex-grow, flex-shrink, flex-basis, flex-wrap
- ✅ **CSS Display Module Level 3**: Block, Inline, Flex, FlowRoot, InlineBlock, and None display values
- ✅ **CSS Values and Units Module Level 3**: px, %, vw, vh, auto, and calc() support

## Status

See [CHANGELOG.md](CHANGELOG.md) for a detailed list of changes.

- Version: **[Unreleased]**
- API is evolving but now includes full Flexbox-like alignment, gaps, inline layout, and extensibility via traits

Future versions may add:

- Correct flex-basis priority over width/height in flex item sizing

## License

MIT
