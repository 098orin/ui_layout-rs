# ui_layout

[![Crates.io](https://img.shields.io/crates/v/ui_layout.svg)](https://crates.io/crates/ui_layout)
[![Docs.rs](https://docs.rs/ui_layout/badge.svg)](https://docs.rs/ui_layout)

A minimal Flexbox-inspired layout engine for Rust GUI development.

This crate provides a small, predictable layout system designed for
custom GUI frameworks, editors, and experimental UI engines.

> [!NOTE]
> This crate is under active development; patch releases may be frequent.

## Features

- Flex layout (Row / Column)
- `flex_grow` and `flex_basis`
- Fixed size and flexible size mixing
- Min / max size constraints
- Margin and padding (CSS-like spacing)
- Block layout
- Recursive tree-based layout
- Parent-relative positioning
- Row and column gaps (`row_gap` / `column_gap`)
- Justify content (`justify_content`) and align items (`align_items`)

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
```

For more examples and to understand the behavior of gaps, alignment, and sizing,
see the unit tests in the [`tests/`](tests/) directory. They provide practical usage patterns and expected layouts.


## Status

See [CHANGELOG.md](CHANGELOG.md) for a detailed list of changes.

* Version: **0.4.6**
* API is evolving but now includes full Flexbox-like alignment and gaps

Future versions may add:

* `flex_shrink`
* `grid`
* Additional flex sizing rules (wrap, fr units, etc.)
* Absolute / fixed positioning

## License

MIT
