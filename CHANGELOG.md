# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog,
and this project loosely follows Semantic Versioning.

---

## [Unreleased]
### Added
- Implemented complete `flex_basis` support:
  - Support for `auto`, pixel values (`Px`), and percentage values
  - Proper interaction with `flex_grow` and `flex_shrink`
  - Priority handling over explicit width/height in flex contexts
- Implemented `flex_shrink` with proportional space reduction when container overflows
- Enhanced flex layout algorithm with proper flex item sizing resolution
- Added comprehensive test suite for flex basis scenarios

### Changed
- Refactored flex layout engine for better modularity and maintainability
- Improved flex item size calculation to properly handle flex-basis vs. explicit sizes
- Reduced function argument counts by introducing parameter structs (addresses clippy warnings)

### Fixed
- Fixed flex items not respecting parent-assigned sizes from flex container
- Resolved issues where explicit width/height would override flex-basis calculations
- Fixed clippy warnings including complex type definitions and excessive function parameters

---

## [0.8.3] - 2026-01-26
### Added
- Implemented margin collapsing for Block layouts:
  - Vertical margins between block-level siblings now collapse according to CSS rules.

---

## [0.8.2] - 2026-01-26

### Fixed
* `Display::Flex` layout handling:
  - Fixed incorrect handling of `flex_grow` when all children had zero `flex_grow` values, ensuring they are laid out according to their fixed sizes instead of being collapsed to zero.

### Added
* Add PartialEq to `Rect` for easier testing and comparison.

---

## [0.8.1] - 2026-01-26

### Fixed
* `Display::None` layout handling:
  - Nodes with `Display::None` are now set as zero-size immediately without further layout processing.

---

## v0.8.0

### Breaking Changes

* `LayoutNode` is now marked as `#[non_exhaustive]`.
  - External crates can no longer construct `LayoutNode` using struct literals.
  - This allows new internal fields to be added without introducing further breaking releases.

* Direct field-based construction of `LayoutNode` is no longer supported.
  - Users must construct layout nodes via the provided constructors or helper APIs.

### Improved

* Significantly improved layout performance for deeply nested Flex / Block trees.
  - Reduced redundant layout size calculations in complex flex-chain scenarios.
  - Worst-case layout time for real-world-like structures has been reduced from seconds to milliseconds.

### Internal

* Added internal layout caching to `LayoutNode`.
  - Cached `BoxModel` results are reused when layout inputs are unchanged.
  - Drastically reduces repeated `layout_size` calls in recursive layouts.
* Refactored layout hot paths to better support incremental and cached evaluation.
* No changes to layout results or visual output; behavior remains identical.

### Notes

* This release focuses on performance and long-term API stability.
* Although layout behavior is unchanged, the public construction pattern for `LayoutNode` has been intentionally restricted.

---

## v0.7.0

### Breaking Changes
* Remake `node.rect` to `node.box_model`.

### Added
* Introduced `BoxModel` to represent layout boxes.
  - Supports `border`, `padding`, `content`, and `children` areas.
* Added support for `border` and `box-sizing` properties.

### Fixed
* Various bug fixes and internal improvements.

---

## [0.6.3] - 2026-01-21

### Added
- Eq/PartialEq implementations for style-related types.

---

## [0.6.2] – 2026-01-15

### Fixed
- Fixed incorrect behavior where `margin: auto` on the main axis did not fully consume remaining free space.
- Fixed cases where `justify-content` was still applied even when at least one flex item had an auto margin on the main axis.
- Fixed incorrect handling of cross-axis auto margins where `align-items` / `align-self` were not properly overridden.
- Fixed inconsistent positioning when only auto margins were present without other spacing or alignment rules.

### Improved
- Auto margins now correctly take precedence over alignment and justification, in accordance with the CSS Flexbox specification.
- Unified auto margin resolution logic across main and cross axes to ensure consistent behavior.
- Improved correctness of auto margin centering when both start and end margins are set to `auto`.

### Notes
- These changes affect only `margin: auto` behavior.
- No other spacing, gap, padding, or alignment logic was modified in this release.


---

## [0.6.1] - 2026-01-14

### Fixed
- Fixed incorrect flex container main size calculation caused by double-counted
  child padding.
- Fixed cross-axis size calculation mistakenly using main-axis padding in flex
  layouts.
- Fixed cross size aggregation incorrectly applying main-axis margins instead of
  cross-axis margins.
- Fixed block layout positioning using parent spacing instead of child margins.
- Fixed incorrect handling of `margin: auto` in block layout that relied on
  viewport size instead of the containing block.

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
