# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog,
and this project loosely follows Semantic Versioning.

---

## [0.6.0] - 2026-01-13

### Added
- New `Length` type to represent layout sizes in multiple units (Px, Percent, Vw, Vh, Auto)
- Support for viewport-relative units (`vw`, `vh`)
- Support for percentage-based lengths
- Basic `calc()`-style expressions via `Length::Add` and `Length::Sub`

### Changed
- Layout APIs that previously accepted `f32` (px-only) now use `Length`
- Layout resolution now depends on the available space (for Percent, Vw, Vh, and Auto)
- Spacing (margin, padding, gap) now follows CSS specification semantics
- Internal size and spacing calculations refactored to support unit-aware resolution

### Breaking Changes
- All size-related properties (width, height, min/max sizes, margin, padding, gap, etc.) now use `Length` instead of `f32`

---

## [0.5.0] - 2026-01-09

### Changed

* Reworked auto size resolution logic across the layout engine.
  - Auto sizing is now evaluated using a clearer intrinsic size pass, improving correctness and predictability.
  - Nested flex layouts with auto-sized containers now produce stable and consistent results.

* Internal layout algorithm has been largely rewritten for clarity and correctness.
  - Separation between intrinsic size calculation and flex redistribution has been improved.
  - Layout recalculation order is now more robust when parent sizes change during flex resolution.

### Notes

* This release does **not** introduce API or interface changes.
* Layout results may differ from previous versions due to improved auto size evaluation.

---

## [0.4.6] - 2026-01-05

### Fixed
- Fixed an issue where flex children were not relaid out after their parent
  size changed due to flex-grow redistribution, causing nested layouts to
  use stale sizes.

---

## [0.4.5] - 2026-01-05

### Fixed

* Fixed incorrect `flex-grow` redistribution when min constraints caused the total flex item size to overflow or underflow the container.
  - Remaining space is now handled as a signed value, allowing negative overflow to be redistributed proportionally instead of being clamped to zero.
  - Flex items are now correctly frozen at their max size when distributing positive remaining space, and at their min size when redistributing negative remaining space, ensuring total sizes converge to the container
  size.


---

## [0.4.4] - 2026-01-05

### Fixed

* Block layout now correctly respects height

---

## [0.4.3] - 2026-01-04

### Added

* Improved `flex_grow` distribution in Flex layouts:
  - After initial layout and clamp to min/max sizes, remaining space is redistributed among eligible flex items
  - Supports correct re-layout of children with Auto sizing, ensuring grandchildren sizes are recalculated
  - Handles edge cases where multiple items hit max/min constraints, redistributing leftover space iteratively
  - Prevent flex items from exceeding parent size after redistribution
  - Ensure grandchildren of flex items are recalculated when parent size changes due to flex_grow redistribution

---

## [0.4.2] - 2026-01-04

### Fixed

* Corrected child coordinate calculation bug in block layouts.

---

## [0.4.1] - 2026-01-04

### Fixed

* Corrected child coordinate calculation bug in flex layouts, ensuring positions match expected values.

---

## [0.4.0] - 2026-01-04

> [!WARNING]
> This version is yanked.

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
