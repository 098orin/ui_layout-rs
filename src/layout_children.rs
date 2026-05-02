use crate::{FragmentNode, LayoutNode};

/// A unified layout item used during layout processing.
///
/// `LayoutChildren` represents multiple units in the layout flow and can be either:
///
/// - Layout nodes ([`LayoutNode`])
/// - Inline-level fragments ([`FragmentNode`])
///
/// This abstraction allows the layout engine to treat structural elements
/// and already-fragmented inline content uniformly while preserving their order.
#[derive(Debug)]
pub enum LayoutChildren {
    Node(Vec<LayoutNode>),
    Fragment(Vec<FragmentNode>),
}

impl LayoutChildren {
    pub(crate) fn new_empty_node() -> Self {
        LayoutChildren::Node(Vec::new())
    }

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
    pub fn node(&self) -> Option<&Vec<LayoutNode>> {
        match self {
            LayoutChildren::Node(n) => Some(n),
            _ => None,
        }
    }
    pub fn node_mut(&mut self) -> Option<&mut Vec<LayoutNode>> {
        match self {
            LayoutChildren::Node(n) => Some(n),
            _ => None,
        }
    }

    pub fn fragment(&self) -> Option<&Vec<FragmentNode>> {
        match self {
            LayoutChildren::Fragment(f) => Some(f),
            _ => None,
        }
    }
    pub fn fragment_mut(&mut self) -> Option<&mut Vec<FragmentNode>> {
        match self {
            LayoutChildren::Fragment(f) => Some(f),
            _ => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            LayoutChildren::Node(v) => v.is_empty(),
            LayoutChildren::Fragment(v) => v.is_empty(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            LayoutChildren::Node(v) => v.len(),
            LayoutChildren::Fragment(v) => v.len(),
        }
    }
}
