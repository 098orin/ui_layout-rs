# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog,
and this project loosely follows Semantic Versioning.

---

## [Unreleased]

### Added

* **Positioned layout**: support `relative`, `absolute`, `fixed`, and
  `sticky` positioning with inset edges and sticky positioning constraints.
* **Grid layout**: add grid layout with explicit and implicit tracks,
  repeated tracks, named areas, negative line placement, and grid item
  alignment.
* **Multi-line flex layout**: support `flex-wrap` and multi-line flex item
  placement.
* **Grid alignment**: support `align-*`, `justify-items`, and `justify-self`
  in the grid formatting context.
* **Custom object sizing**: add `AutoSizeBehavior` and
  `SizeStyle::auto_behavior` to control how block replaced-element leaves
  resolve `auto` width/height.
  * `AutoSizeBehavior::Fill` (default) stretches to the containing block.
  * `AutoSizeBehavior::ShrinkToFit` sizes to the custom child's intrinsic
    box, allowing `margin: auto` to center the element.
* **Unified custom layout API**: replace the separate `layout_flow()` and
  `layout_block()` methods with a single `CustomLayouter::layout()` entry
  point returning a [`LayoutBox`].
* **Styled custom children**: `CustomChild` now carries its own [`Style`],
  allowing custom objects to participate in the parent's formatting context
  through `display`.
  * `OuterDisplay::Block` participates as a block-level object.
  * `OuterDisplay::Inline` participates inline.
  * `OuterDisplay::None` is skipped.
* **Custom layout results**: expose custom layout results through
  `LayoutChild::custom_result()` without downcasting.
* **Graceful `LayoutBox` mismatches**: handle custom objects whose returned
  [`LayoutBox`] variant differs from their declared display, including
  atomic inline placement of block boxes and anonymous block wrapping of
  inline boxes.
* **Custom flex results**: retain the final border-box rectangle of custom
  flex items.

### Changed

* **Custom child layout model**: `LayoutChild::Custom` now wraps a
  `CustomChild` containing the layouter and its layout result, while
  `LayoutChild::from(obj)` remains available for ergonomic construction.
* **Custom formatting context**: custom objects now derive their formatting
  context from the [`Style`] carried by `CustomChild` rather than reporting
  it through the layouter trait.
* **Inline-flow context**: move `start_pos`, `available_inline_size`, and
  `line_height` into [`LayoutContext`], removing the separate
  `FlowLayoutContext` type.
* **Block replaced-element sizing**: `auto` width/height now fill the
  containing block by default. Use
  `SizeStyle::auto_behavior = AutoSizeBehavior::ShrinkToFit` for
  shrink-to-fit behavior.
* **Flex sizing**: make `flex-basis` account for the flex item's box sizing
  and content size.
* **Grid sizing and placement**: improve implicit track sizing, prevent
  excessive track expansion, and correctly account for content offsets.
* **LayoutBox inline representation**: enhance the representation of
  `LayoutBox::InlineBox` and preserve inline line information across breaks.

### Fixed

* Correct application of min/max size constraints to block-level nodes.
* Correct block-level sizing to fit the available width.
* Correct inline-level placement inside flex and grid containers.
* Correct inline-flow state and line indices across line breaks.
* Correct custom inline available width after a line break.
* Correct `box-sizing` handling and `flex-basis` resolution.
* Correct sticky edge representation and positioning.
* Correct handling of auto margins with negative free space.
* Prevent overflow and out-of-bounds vector access in edge cases.
* Correct grid placement and track sizing.
* Prevent double application of content offsets during grid layout.

### Removed

* **`CustomLayouter::formatting_context()`**: formatting context is now
  determined from the custom child's [`Style`].

---

## [0.12.1]

### Fixed
- inline-block x-coordinate
  - Add start_x to FlowState for correct inline-block end_pos
  - Use children_width.max(current_x) for content width calculation

---

## [0.12.0]

### Added

- **`display: flow-root` and `display: inline-block`**: New `InnerDisplay::FlowRoot` variant that establishes a Block Formatting Context.
  - `display: flow-root` → `(Block, FlowRoot)` — block-level, isolates margin collapsing.
  - `display: inline-block` → `(Inline, FlowRoot)` — inline-level, atomic inline box with explicit sizing.
  - CSS parsing (`from_css_name`, `from_css`, `parse`) and `fmt::Display` formatting for both keywords.
  - `FlowRoot` reuses the same `layout_flow` path as `Flow`; margin collapsing is gated by a `collapse_margins` flag.
- Spacing display output now emits compact CSS-like notation:
  - All same: margin: 10px
  - TB / LR pair: margin: 10px 20px
  - T / LR / B: margin: 10px 20px 30px
  - All different: margin: 10px 40px 20px 30px (top right bottom left)
  - Single side: margin-top: 10px (individual entry kept when shorter)

### Fixed

- **Inline-block sizing**: Size constraints (width, height, min/max) are now correctly applied to inline-block boxes.
- **Inline-block cursor advancement**: Inline-block boxes now advance the inline cursor by their full box width.
- **Inline-block box model**: Inline-block boxes keep their border/padding origin unshifted, matching atomic inline-level box semantics.

### Internal

- Improved code documentation across the flow layout engine (`FlowState::collapse_margins`, margin collapsing branches, inline-block finalization logic).

---

## [0.11.0] - 2026-07-20

### Breaking Changes

- `Display` enum replaced with `Display` struct `{ outer: OuterDisplay, inner: InnerDisplay }`, following the CSS Display Level 3 specification.
  - New enums: `OuterDisplay` (`Block`, `Inline`, `None`), `InnerDisplay` (`Flow`, `Flex`).
  - `Display::Flex { flex_direction }` variant removed; `flex_direction` is now a separate field on `Style`.
- `LayoutBoxes` renamed to `LayoutBox` with new variants: `None`, `BlockBox(BoxModel)`, `InlineBox(InlineBox)`.
  - Removed variants: `Single`, `Multiple`.
- `LayoutNode.children` changed from `Vec<LayoutNode>` to `Vec<LayoutChild>`.
- `LayoutNode::set_fragments(...)` removed.
- `FragmentPlacement` renamed to `Placement` (adds `#[derive(Default)]`).
- `Length::Auto` variant removed. Auto sizing is now represented by the new `LengthOrAuto` enum.
  - `LengthOrAuto::Length(Length)` | `LengthOrAuto::Auto` (default).
- Size/spacing property types migrated from `Length` to `LengthOrAuto`:
  - `SizeStyle` fields: `width`, `height`, `min_width`, `max_width`, `min_height`, `max_height`.
  - `Spacing` fields: `margin_top`, `margin_bottom`, `margin_left`, `margin_right`.
  - `ItemStyle.flex_basis`, `Style.column_gap`, `Style.row_gap`.
- `FlexDirection` default changed from `Column` to `Row` (matches CSS spec).
- `LayoutNode::with_children(...)` signature changed to accept any `IntoIterator<Item = T> where T: Into<LayoutChild>`.
- `IntoIterator for &LayoutBox` now yields `BoxModel` by value (previously yielded `&BoxModel`).
- `IntoIterator for &mut LayoutBoxes` removed.
- `FlowLayouter::debug_name(...)` replaced by `write_debug(...)`.

### Added

- **Flex reverse**: `FlexDirection::RowReverse` and `ColumnReverse` variants, with full flex reverse layout support.
- **Margin collapse**: Vertical margins between block-level siblings now collapse per CSS rules.
- **`LayoutChild`** enum with variants: `Node`, `Fragment`, `Object(Box<dyn FlowLayouter>)`, `Custom(Box<dyn BlockLayouter>)` (unstable).
- **`FlowLayouter`** trait for custom inline flow layout delegation (`layout`, `measure`, `write_debug` methods).
- **`BlockLayouter`** trait (`feature = "unstable"`) for custom block layout delegation.
- **`LayoutEngine::layout_root(...)`** method for custom layout delegation.
- **`FragmentNode`** struct wrapping `ItemFragment` with `Placement`.
- **`LayoutBox` iteration** support via named types `LayoutBoxIter` / `LayoutBoxIntoIter` with `ExactSizeIterator`, `DoubleEndedIterator`, `FusedIterator`.
- **`LayoutBox::width_box()`**, **`height_box()`** methods for inline box dimensions.
- **`InlineBox`** struct (`box_model: BoxModel`, `line_spans: Vec<LineSpan>`).
- **`LineSpan`** struct with `x_range`, `line_pos`, `line_index`, and `width()` method.
- **`EdgeOption`** struct (`left`, `top`, `right`, `bottom` as `Option<f32>`).
- **`MeasureResult`** struct (`width`, `height`).
- **`FlowLayoutContext`** struct for flow layout delegation.
- **`LayoutItem`** enum: `Node`, `Fragments`, `Object`, `Custom` (unstable).
- **`Axis`** enum: `Horizontal`, `Vertical`.
- **`Style.line_height`** field added.
- **`fmt::Display`** implementations for: `Length`, `LengthOrAuto`, `OuterDisplay`, `InnerDisplay`, `Display`, `FlexDirection`, `BoxSizing`, `JustifyContent`, `AlignItems`, `ItemStyle`, `SizeStyle`, `Spacing`, `Style`, `Placement`.
- **`Display::from_css_name()`**, **`Display::from_css()`**, **`Display::parse()`** methods and `FromStr` impl.
- **`LayoutMetrics`** struct (behind `feature = "layout-bench"`) for layout call/cache instrumentation.
- **`feature = "unstable"`** flag gating `BlockLayouter` trait and `LayoutChild::Custom`.

### Fixed

- `FlexDirection` default corrected from `Column` to `Row`.
- Flex container auto-sizing now correctly includes child margins.
- `FlowLayouter` objects now handle line breaks during inline flow layout (`LayoutChild::Object`).
- `LayoutBoxIter` returns correct per-line `BoxModel` dimensions (padding/border stripping for non-first/non-last lines).
- `Default` for `Spacing` now correctly initializes margins to `LengthOrAuto::Length(Px(0.0))` instead of `LengthOrAuto::Auto`.

### Internal

- Module reorganization: `src/node.rs` removed; replaced by `src/layout_node.rs` and `src/layout_child.rs`.
- New modules: `src/display.rs`, `src/flow_layouter.rs`, `src/block_layouter.rs`.
- `LayoutEngine` changed from unit struct to struct holding viewport state.
- Extensive internal refactoring of flow, flex, inline, and block layout engines.

---

## [0.10.0] - 2026-3-25

> [!WARNING]
> This version is yanked.

### Fixed

- Inline `LayoutBoxes` will now use `LayoutBoxes::Multiple`, correctly returning results for multi-row LayoutBoxes.
- Margin, Padding, and Border calculations will now be correct due to changes associated with the use of
  `LayoutBoxes::Multiple` (slightly early wrap for Margin will not be implemented).
- Inline Containers will use `LayoutBoxes::Multiple` to correctly wrap inline child elements across multiple rows.

### Breaking Changes

- `FragmentPlacement` has been changed so that it is positioned relative to the LayoutBox of that row.

---

## [0.9.8] - 2026-03-15

### Fixed

- Fixed incorrect width calculation for `Inline` layout box.
  - The width of an `Inline` layout box is now correctly calculated as the maximum width of its child lines, rather than the last line width.

---

## [0.9.7] - 2026-03-14

### Added

- Add new API `width()`, `height()` and `is_line_break()` to `ItemFragment`

---

## [0.9.6] - 2026-03-02

### Fixed

- Fixed incorrect line calculation in `block` layout.
  - Line height is now correctly calculated from `border_box.height` instead of `content_box.height`, ensuring proper spacing and alignment of block-level elements according to CSS specifications.

---

## [0.9.5] - 2026-03-02

### Fixed

- Fixed incorrect line calculation in `flex` layout.

---

## [0.9.4] - 2026-03-01

### Fixed

- Fixed incorrect handling of `auto` margins in block layout.
  - Block-level elements with `margin: auto` now correctly center themselves within their containing block when the available space is larger than the element's size, following CSS specifications.
  - When the available space is smaller than the element's size, `auto` margins are treated as zero, allowing the element to overflow as expected without centering.

### Improved

- Improved `Inline` layout (Not yet fully compliant with CSS inline formatting model, but basic support added):
  - Implemented initial inline layout logic.

---

## [0.9.3] - 2026-02-15

### Fixed

- Fixed block `auto` sizing
- Fixed incorrect distribution of `auto` margins on the main axis in Flexbox layout.
  Previously, remaining free space was assigned to the first encountered `auto` margin.
  It is now evenly distributed across all `auto` margins on the main axis, in accordance with the Flexbox specification.
- Fixed handling of negative free space when main-axis `auto` margins are present.
  Free space is now clamped to zero only when `auto` margins are involved, matching spec behavior.
- Fixed cross-axis `auto` margin behavior in Flexbox layout.
  `auto` margins on the cross axis now correctly override `align-items` / `align-self` and absorb available free space per item:
  - If only one side is `auto`, it absorbs all available free space.
  - If both sides are `auto`, free space is split evenly between them.

---

## [0.9.2] - 2026-02-12

### Fixed

- Fixed example code of README.md.

### Improved

- Improved performance across core layout processes, resulting in significantly faster execution.

---

## [0.9.1] - 2026-02-11

### Added

- Implemented `IntoIterator` for `LayoutBoxes`, `&LayoutBoxes`, and `&mut LayoutBoxes`.
- Added `iter()` and `iter_mut()` convenience methods to `LayoutBoxes` for ergonomic iteration over contained `BoxModel` values.
- Enabled use of standard iterator traits for `LayoutBoxes`, including `ExactSizeIterator`, `DoubleEndedIterator`, and `FusedIterator` through delegation to the standard library iterators.

---

## [0.9.0] - 2026-02-11

### Added

- Inline layout support:
  - Implemented complete inline formatting model
  - Wrapping and line breaking logic
- New `Length` operations:
  - `Mul`, `Div`, `Min`, `Max`, `Clamp`
- Enhanced flex layout algorithm with proper flex item sizing resolution
- Added comprehensive test suite for flex basis scenarios

### Changed

- Refactored flex layout engine for better modularity and maintainability
- Improved flex item size calculation to properly handle flex-basis vs. explicit sizes
- Reduced function argument counts by introducing parameter structs (addresses clippy warnings)

---

## [0.8.3] - 2026-01-26

### Added

- Implemented margin collapsing for Block layouts:
  - Vertical margins between block-level siblings now collapse according to CSS rules.

---

## [0.8.2] - 2026-01-26

### Fixed

- `Display::Flex` layout handling:
  - Fixed incorrect handling of `flex_grow` when all children had zero `flex_grow` values, ensuring they are laid out according to their fixed sizes instead of being collapsed to zero.

### Added

- Add PartialEq to `Rect` for easier testing and comparison.

---

## [0.8.1] - 2026-01-26

### Fixed

- `Display::None` layout handling:
  - Nodes with `Display::None` are now set as zero-size immediately without further layout processing.

---

## v0.8.0

### Breaking Changes

- `LayoutNode` is now marked as `#[non_exhaustive]`.
  - External crates can no longer construct `LayoutNode` using struct literals.
  - This allows new internal fields to be added without introducing further breaking releases.

- Direct field-based construction of `LayoutNode` is no longer supported.
  - Users must construct layout nodes via the provided constructors or helper APIs.

### Improved

- Significantly improved layout performance for deeply nested Flex / Block trees.
  - Reduced redundant layout size calculations in complex flex-chain scenarios.
  - Worst-case layout time for real-world-like structures has been reduced from seconds to milliseconds.

### Internal

- Added internal layout caching to `LayoutNode`.
  - Cached `BoxModel` results are reused when layout inputs are unchanged.
  - Drastically reduces repeated `layout_size` calls in recursive layouts.
- Refactored layout hot paths to better support incremental and cached evaluation.
- No changes to layout results or visual output; behavior remains identical.

### Notes

- This release focuses on performance and long-term API stability.
- Although layout behavior is unchanged, the public construction pattern for `LayoutNode` has been intentionally restricted.

---

## v0.7.0

### Breaking Changes

- Remake `node.rect` to `node.box_model`.

### Added

- Introduced `BoxModel` to represent layout boxes.
  - Supports `border`, `padding`, `content`, and `children` areas.
- Added support for `border` and `box-sizing` properties.

### Fixed

- Various bug fixes and internal improvements.

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

- Reworked auto size resolution logic across the layout engine.
  - Auto sizing is now evaluated using a clearer intrinsic size pass, improving correctness and predictability.
  - Nested flex layouts with auto-sized containers now produce stable and consistent results.

- Internal layout algorithm has been largely rewritten for clarity and correctness.
  - Separation between intrinsic size calculation and flex redistribution has been improved.
  - Layout recalculation order is now more robust when parent sizes change during flex resolution.

### Notes

- This release does **not** introduce API or interface changes.
- Layout results may differ from previous versions due to improved auto size evaluation.

---

## [0.4.6] - 2026-01-05

### Fixed

- Fixed an issue where flex children were not relaid out after their parent
  size changed due to flex-grow redistribution, causing nested layouts to
  use stale sizes.

---

## [0.4.5] - 2026-01-05

### Fixed

- Fixed incorrect `flex-grow` redistribution when min constraints caused the total flex item size to overflow or underflow the container.
  - Remaining space is now handled as a signed value, allowing negative overflow to be redistributed proportionally instead of being clamped to zero.
  - Flex items are now correctly frozen at their max size when distributing positive remaining space, and at their min size when redistributing negative remaining space, ensuring total sizes converge to the container
    size.

---

## [0.4.4] - 2026-01-05

### Fixed

- Block layout now correctly respects height

---

## [0.4.3] - 2026-01-04

### Added

- Improved `flex_grow` distribution in Flex layouts:
  - After initial layout and clamp to min/max sizes, remaining space is redistributed among eligible flex items
  - Supports correct re-layout of children with Auto sizing, ensuring grandchildren sizes are recalculated
  - Handles edge cases where multiple items hit max/min constraints, redistributing leftover space iteratively
  - Prevent flex items from exceeding parent size after redistribution
  - Ensure grandchildren of flex items are recalculated when parent size changes due to flex_grow redistribution

---

## [0.4.2] - 2026-01-04

### Fixed

- Corrected child coordinate calculation bug in block layouts.

---

## [0.4.1] - 2026-01-04

### Fixed

- Corrected child coordinate calculation bug in flex layouts, ensuring positions match expected values.

---

## [0.4.0] - 2026-01-04

> [!WARNING]
> This version is yanked.

### Added

- `align_self` support for flex items
- Automatic size resolution for items with `width` or `height` set to `None`
- Support for negative available space in flex layouts (no panics, layout adjusts automatically)

### Changed

- Layout engine applies `align_self` over parent `align_items`

### Fixed

- Improved calculation of flex layouts when container space is smaller than total children sizes

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
