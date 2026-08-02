//! Types for custom/replaced element layout results.

use crate::{BoxModel, LineSpan};

/// Result of laying out a custom/replaced element.
#[derive(Debug, Clone)]
pub struct CustomObjectResult {
    /// Line spans for inline custom elements.
    pub spans: Vec<LineSpan>,
    /// Box model for the custom element.
    pub box_model: BoxModel,
}
