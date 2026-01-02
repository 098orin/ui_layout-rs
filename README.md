# ui_layout

A minimal layout engine for Rust GUI development.

## Features

- Flex Row / Column
- Fixed size + flex_grow
- Padding
- Recursive layout
- Parent-relative positioning

## Non-goals

- Full CSS compatibility
- Inline / text layout
- Absolute positioning
- Web rendering

## Example

```rust
LayoutEngine::layout(&mut root, 800.0, 600.0);
```
