use crate::{ItemFragment, LayoutNode};

/// A unified layout item used during layout processing.
///
/// `LayoutItem` represents a single unit in the layout flow and can be either:
///
/// - A child layout node ([`LayoutNode`])
/// - An inline-level fragment ([`ItemFragment`])
///
/// This abstraction allows the layout engine to treat structural elements
/// and already-fragmented inline content uniformly while preserving their order.
///
/// ## Purpose
///
/// By unifying nodes and fragments into a single sequence, the engine can:
///
/// - Handle mixed inline and block content naturally
/// - Process layout in a single pass over ordered items
/// - Flush inline layout when encountering block-level nodes
///
/// This is essential for correctly implementing inline formatting contexts,
/// especially when inline content is interrupted by block-level elements.
#[derive(Debug)]
pub enum LayoutItem {
    Node(LayoutNode),
    Fragment(ItemFragment),
}

impl LayoutItem {
    /// Returns an immutable reference to the inner [LayoutNode] if this item is a node.
    ///
    /// This method provides a convenient way to access the node without pattern matching.
    /// If the item is not a Node, None is returned.
    ///
    /// ## Examples
    ///
    /// ```
    /// if let Some(node) = item.node() { /* Access the node */ }
    /// ```
    pub fn node(&self) -> Option<&LayoutNode> {
        match self {
            LayoutItem::Node(n) => Some(n),
            _ => None,
        }
    }
    pub fn node_mut(&mut self) -> Option<&mut LayoutNode> {
        match self {
            LayoutItem::Node(n) => Some(n),
            _ => None,
        }
    }

    pub fn fragment(&self) -> Option<&ItemFragment> {
        match self {
            LayoutItem::Fragment(f) => Some(f),
            _ => None,
        }
    }
    pub fn fragment_mut(&mut self) -> Option<&mut ItemFragment> {
        match self {
            LayoutItem::Fragment(f) => Some(f),
            _ => None,
        }
    }
}
