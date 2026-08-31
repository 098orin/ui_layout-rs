//! # ui_layout
//!
//! Minimal CSS-like layout engine for UI frameworks.
//! Designed for lightweight, fast, and portable applications (e.g. editors and IDEs).
//!
//! Provides predictable layout behavior for custom GUI frameworks,
//! editors, and experimental rendering engines.
//!
//! ## Getting Started
//!
//! Basic usage follows a simple flow:
//!
//! 1. Create a layout tree
//! 2. Run the layout engine
//! 3. Access computed layout results
//!
//! ```rust
//! use ui_layout::*;
//!
//! let mut root = LayoutNode::new(Style::default());
//!
//! // Compute layout using a viewport size.
//! LayoutEngine::layout(&mut root, 800.0, 600.0);
//! ```
//!
//! ## Core Concepts
//!
//! Layout behavior is configured through [`Style`], which groups
//! commonly used layout properties into focused categories:
//!
//! - [`Display`] — controls layout mode
//! - [`SizeStyle`] — width and height constraints
//! - [`Spacing`] — margin, padding, and borders
//! - [`ItemStyle`] — item-specific flex behavior
//! - `justify_content` / `align_items` — child alignment
//! - `flex_direction`, `row_gap`, `column_gap` — flex container behavior
//!
//! This organization keeps layout rules explicit and easy to reason about.
//!
//! ## Observing Layout Results
//!
//! After layout computation, results are written into each
//! [`LayoutNode`] through its `layout_box` field.
//!
//! The resulting [`LayoutBox`] describes how the node was laid out:
//!
//! - `LayoutBox::None` — no layout result
//! - `LayoutBox::BlockBox(BoxModel)` — standard box layout result
//! - `LayoutBox::InlineBox(InlineBox)` — inline content that may span multiple lines
//!
//! [`BoxModel`] contains absolute rectangles for different regions:
//!
//! - `border_box` — outer box including borders
//! - `padding_box` — inner box including padding
//! - `content_box` — content area
//! - `children_box` — area occupied by child content
//!
//! Inline layouts additionally provide [`LineSpan`] information for
//! observing how a box is split across lines.
//!
//! ## Children
//!
//! Layout trees contain children represented by [`LayoutChild`]:
//!
//! - `LayoutChild::Node` — a normal layout node
//! - `LayoutChild::Fragment` — an inline-level fragment
//! - `LayoutChild::Custom` — a custom [`CustomLayouter`] object
//!
//! [`ItemFragment`] represents the smallest independently positioned
//! piece of inline content.
//!
//! Common fragment types:
//!
//! - `Fragment` — inline content with dimensions
//! - `LineBreak` — forces a line break
//!
//! During layout, fragments are wrapped in [`FragmentNode`] and assigned
//! placement information, allowing inline content to be split and
//! positioned across multiple lines.
//!
//! ## Custom Objects
//!
//! The [`CustomLayouter`] trait allows custom types to participate directly
//! in layout. Custom objects implement a single
//! [`layout`](CustomLayouter::layout) method returning a [`LayoutBox`] —
//! [`LayoutBox::InlineBox`] for inline content, [`LayoutBox::BlockBox`] for
//! block-level boxes. How the object participates is declared by the
//! [`Style`] it carries in its [`LayoutChild::Custom`]:
//!
//! - `Display::OutsideInner { outer: OuterDisplay::Block, .. }` → the engine lays the object out as a block
//! - `Display::OutsideInner { outer: OuterDisplay::Inline, .. }` → the engine lays the object out inline
//! - `Display::None` → the object is skipped
//! - `Display::Contents` → the object is skipped (children participate in parent context)
//!
//! The returned [`LayoutBox`] need not match the declared context. An inline
//! object returning `BlockBox` is placed atomically on the current line like a
//! fragment; a block object returning `InlineBox` is wrapped in an anonymous
//! block box.
//!
//! Layout results are stored on the child in a [`CustomObjectResult`],
//! accessible through [`LayoutChild::custom_result`] after layout.

mod cache;
mod custom_layouter;
mod custom_object;
mod display;
mod engine;
mod fragment;
mod geometry;
mod layout_child;
mod layout_node;
mod style;

pub use custom_layouter::*;
pub use custom_object::*;
pub use engine::*;
pub use fragment::*;
pub use geometry::*;
pub use layout_child::*;
pub use layout_node::*;
pub use style::*;
