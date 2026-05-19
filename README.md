# ui_layout

[![Crates.io](https://img.shields.io/crates/v/ui_layout.svg)](https://crates.io/crates/ui_layout)
[![Docs.rs](https://docs.rs/ui_layout/badge.svg)](https://docs.rs/ui_layout)

A unified layout engine for Rust GUI development that treats Flexbox as a specialized form of Inline Block Flow layout.

This crate provides predictable layout system designed for
custom GUI frameworks, editors, and experimental UI engines.

> [!NOTE]
> This crate is under active development; patch releases may be frequent.
> **CSS3 Specification Compliance**: Phase 1 improvements have been implemented for CSS Box Model and Flexbox.

## Features

### Flexbox Support
- Flex layout (Row / Column direction)
- `flex_grow` with proportional space distribution
- `flex_shrink` with proportional space reduction when overflowing
- `flex_basis` for initial sizing (supports `auto`, pixel values, and percentages)
- `justify_content` (Start, Center, End, SpaceBetween, SpaceAround, SpaceEvenly)
- `align_items` with full `stretch` support (Start, Center, End, Stretch)
- `align_self` for individual item alignment override
- Row and column gaps (`row_gap` / `column_gap`)

## Non-goals

- Full CSS compatibility
- Inline or text layout
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
        display: Display::Flex {
            flex_direction: FlexDirection::Row,
        },
        size: SizeStyle {
            width: Length::Px(300.0),
            height: Length::Px(200.0),
            ..Default::default()
        },
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        column_gap: Length::Px(20.0),
        ..Default::default()
    },
    vec![
        LayoutNode::new(Style {
            item_style: ItemStyle {
                flex_grow: 1.0,
                flex_basis: Length::Auto,
                ..Default::default()
            },
            ..Default::default()
        }),
        LayoutNode::new(Style {
            item_style: ItemStyle {
                flex_basis: Length::Px(100.0),
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

- ✅ **CSS Box Model Module Level 3**: Full margin collapsing, padding, border, box-sizing support
- ✅ **CSS Flexible Box Layout Module Level 1**: Complete flexbox algorithm including flex-grow, flex-shrink, flex-basis
- ✅ **CSS Display Module Level 3**: Block, Inline, Flex, and None display values
- ✅ **CSS Values and Units Module Level 3**: px, %, vw, vh, auto, and calc() support


## Status

See [CHANGELOG.md](CHANGELOG.md) for a detailed list of changes.

* Version: **0.11.0(Unreleased)**
* API is evolving but now includes full Flexbox-like alignment and gaps
* **Phase 1 improvements**: Enhanced margin collapsing, comprehensive specification references

Future versions may add:

* `grid`
* Flex wrap support (`flex-wrap: wrap | nowrap | wrap-reverse`)
* Flex direction reverse (`flex-direction: row-reverse | column-reverse`)
* Additional flex sizing rules (fr units, etc.)
* Absolute / fixed positioning
* Display: inline-block explicit support
* Complete parent-child margin collapsing
* Content-based minimum sizing for flex items
* Correct flex-basis priority over width/height in flex item sizing

## License

MIT
