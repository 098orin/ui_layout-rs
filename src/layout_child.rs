use crate::{FragmentNode, LayoutNode};

/// A unified layout item used during layout processing.
///
/// `LayoutChild` represents multiple units in the layout flow and can be either:
///
/// - Layout nodes ([`LayoutNode`])
/// - Inline-level fragments ([`FragmentNode`])
///
/// This abstraction allows the layout engine to treat structural elements
/// and already-fragmented inline content uniformly while preserving their order.
#[derive(Debug)]
pub enum LayoutChild {
    Node(Box<LayoutNode>),
    Fragment(FragmentNode),
}

impl LayoutChild {
    pub fn node(&self) -> Option<&LayoutNode> {
        match self {
            LayoutChild::Node(n) => Some(n),
            _ => None,
        }
    }

    pub fn node_mut(&mut self) -> Option<&mut LayoutNode> {
        match self {
            LayoutChild::Node(n) => Some(n),
            _ => None,
        }
    }

    pub fn fragment(&self) -> Option<&FragmentNode> {
        match self {
            LayoutChild::Fragment(f) => Some(f),
            _ => None,
        }
    }

    pub fn fragment_mut(&mut self) -> Option<&mut FragmentNode> {
        match self {
            LayoutChild::Fragment(f) => Some(f),
            _ => None,
        }
    }
}
