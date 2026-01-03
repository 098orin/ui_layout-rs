# ui_layout

[![Crates.io](https://img.shields.io/crates/v/ui_layout.svg)](https://crates.io/crates/ui_layout)
[![Docs.rs](https://docs.rs/ui_layout/badge.svg)](https://docs.rs/ui_layout)

A minimal Flexbox-inspired layout engine for Rust GUI development.

This crate provides a small, predictable layout system designed for
custom GUI frameworks, editors, and experimental UI engines.

## Features

- Flex layout (Row / Column)
- `flex_grow` and `flex_basis`
- Fixed size and flexible size mixing
- Min / max size constraints
- Margin and padding (CSS-like spacing)
- Block layout
- Recursive tree-based layout
- Parent-relative positioning

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
use layout::*;

LayoutEngine::layout(&mut root, 800.0, 600.0);
````

## Status

See [CHANGELOG.md](CHANGELOG.md) for a detailed list of changes.

* Version: **0.2.0**
* API is still evolving
* Focused on Flexbox-like block layouts

Future versions may add:

* `justify-content`
* `align-items`
* `gap`
* Improved flex sizing rules

## License

MIT
