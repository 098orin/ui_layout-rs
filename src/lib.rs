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
//! 1. Create layout nodes
//! 2. Run the layout engine
//! 3. Access computed layout results

mod cache;
mod engine;
mod fragment;
mod geometry;
mod layout_child;
mod layout_node;
mod style;

pub use engine::*;
pub use fragment::*;
pub use geometry::*;
pub use layout_child::*;
pub use layout_node::*;
pub use style::*;
