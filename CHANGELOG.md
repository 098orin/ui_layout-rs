# Changelog

All notable changes to this project will be documented in this file.

The format is based on Keep a Changelog,
and this project loosely follows Semantic Versioning.

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
