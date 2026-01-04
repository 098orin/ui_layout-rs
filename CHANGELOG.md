# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog,
and this project loosely follows Semantic Versioning.

---

## [0.4.0] - 2026-01-04

### Added

* `align_self` support for flex items
* Automatic size resolution for items with `width` or `height` set to `None`
* Support for negative available space in flex layouts (no panics, layout adjusts automatically)

### Changed

* Layout engine applies `align_self` over parent `align_items`

### Fixed

* Improved calculation of flex layouts when container space is smaller than total children sizes

---

## [0.3.0] - 2026-01-03

### Added
- Row and column gaps for Flex layout (`row_gap` / `column_gap`)
- `justify_content` support: Start, Center, End, SpaceBetween, SpaceAround, SpaceEvenly
- `align_items` support: Start, Center, End, Stretch
- Axis-aware padding and margin calculations now fully applied in Flex layout
- Tests for gap and alignment behavior added

### Changed
- Negative gap values are now clamped to zero to match CSS behavior
- Layout calculations refactored for better clarity and maintainability
- Flex layout now fully respects min/max sizes alongside flex-grow/flex-basis
- Cross-axis margin and padding calculations reorganized for consistency and maintainability

---

## [0.2.0] - 2026-01-03

### Added
- `flex_basis` support for flex items
- Margin and padding via `Spacing` (CSS-like box model)
- Min / max size constraints for width and height
- Block layout implementation
- Basic size clamping logic

### Changed
- Layout calculation refactored to be Flexbox-inspired
- Fixed-size and flexible-size elements can now be mixed more predictably
- Internal style structures reorganized (`SizeStyle`, `Spacing`)

### Fixed
- Block layout previously treated as column layout

---

## [0.1.0] - 2025-01-02

### Added
- Initial layout engine implementation
- Flex layout (Row / Column)
- `flex_grow` support
- Fixed width / height sizing
- Padding support
- Recursive layout tree
