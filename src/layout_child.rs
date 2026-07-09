use crate::{CustomLayout, FlowLayouter, FragmentNode, ItemFragment, LayoutNode};

/// A unified layout item used during layout processing.
///
/// `LayoutChild` represents the kinds of children a [`LayoutNode`] can have:
///
/// - [`Node`](LayoutChild::Node) — a nested layout node (block, inline, flex, etc.)
/// - [`Fragment`](LayoutChild::Fragment) — an inline-level content fragment
/// - [`Object`](LayoutChild::Object) — a custom [`FlowLayouter`] object
/// - [`Custom`](LayoutChild::Custom) — a custom [`CustomLayout`] object
///
/// This abstraction allows the layout engine to treat structural elements,
/// inline fragments, and custom objects uniformly while preserving their order.
#[derive(Debug)]
pub enum LayoutChild {
    /// A nested layout node.
    ///
    /// The child participates in the parent's formatting context according
    /// to its own [`Style::display`] property.
    Node(Box<LayoutNode>),
    /// An inline-level content fragment.
    ///
    /// Fragments are the smallest independently-positionable unit of inline
    /// content (text, images, etc.).  They can be split across lines.
    Fragment(FragmentNode),
    /// A custom object that implements [`FlowLayouter`].
    ///
    /// Objects are self-layouting: they implement [`FlowLayouter::layout`]
    /// for inline flow and [`FlowLayouter::measure`] for flex sizing.
    Object(Box<dyn FlowLayouter>),
    /// A custom object that implements [`CustomLayout`].
    ///
    /// Custom nodes are self-layouting: they implement
    /// [`CustomLayout::layout`] for block-level layout and
    /// [`CustomLayout::measure`] for flex sizing.
    Custom(Box<dyn CustomLayout>),
}

impl LayoutChild {
    /// Returns a reference to the underlying [`LayoutNode`] if this
    /// child is a [`Node`](LayoutChild::Node).
    pub fn node(&self) -> Option<&LayoutNode> {
        match self {
            LayoutChild::Node(n) => Some(n),
            _ => None,
        }
    }

    /// Returns a mutable reference to the underlying [`LayoutNode`]
    /// if this child is a [`Node`](LayoutChild::Node).
    pub fn node_mut(&mut self) -> Option<&mut LayoutNode> {
        match self {
            LayoutChild::Node(n) => Some(n),
            _ => None,
        }
    }

    /// Returns a reference to the underlying [`FragmentNode`] if this
    /// child is a [`Fragment`](LayoutChild::Fragment).
    pub fn fragment(&self) -> Option<&FragmentNode> {
        match self {
            LayoutChild::Fragment(f) => Some(f),
            _ => None,
        }
    }

    /// Returns a mutable reference to the underlying [`FragmentNode`]
    /// if this child is a [`Fragment`](LayoutChild::Fragment).
    pub fn fragment_mut(&mut self) -> Option<&mut FragmentNode> {
        match self {
            LayoutChild::Fragment(f) => Some(f),
            _ => None,
        }
    }

    /// Returns a reference to the underlying [`FlowLayouter`] object
    /// if this child is an [`Object`](LayoutChild::Object).
    pub fn object(&self) -> Option<&dyn FlowLayouter> {
        match self {
            LayoutChild::Object(o) => Some(&**o),
            _ => None,
        }
    }

    /// Returns a reference to the underlying [`CustomLayout`] object
    /// if this child is a [`Custom`](LayoutChild::Custom).
    pub fn custom(&self) -> Option<&dyn CustomLayout> {
        match self {
            LayoutChild::Custom(c) => Some(&**c),
            _ => None,
        }
    }
}

impl From<LayoutNode> for LayoutChild {
    fn from(node: LayoutNode) -> Self {
        LayoutChild::Node(Box::new(node))
    }
}

impl From<FragmentNode> for LayoutChild {
    fn from(fragment: FragmentNode) -> Self {
        LayoutChild::Fragment(fragment)
    }
}

impl From<ItemFragment> for LayoutChild {
    fn from(value: ItemFragment) -> Self {
        LayoutChild::Fragment(FragmentNode::new(value))
    }
}
