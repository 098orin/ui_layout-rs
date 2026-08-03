use crate::{CustomLayouter, CustomObjectResult, FragmentNode, ItemFragment, LayoutNode};

/// A custom layout object together with its last layout result.
///
/// Wraps a [`Box<dyn CustomLayouter>`] and stores the [`CustomObjectResult`]
/// produced by the engine during layout, so callers can observe how the
/// object was positioned without downcasting the trait object.
#[derive(Debug)]
pub struct CustomChild {
    layouter: Box<dyn CustomLayouter>,
    result: Option<CustomObjectResult>,
}

impl CustomChild {
    /// Wraps a [`CustomLayouter`] object without a layout result yet.
    pub fn new(layouter: impl CustomLayouter + 'static) -> Self {
        Self {
            layouter: Box::new(layouter),
            result: None,
        }
    }

    /// Wraps a boxed [`CustomLayouter`] object without a layout result yet.
    pub fn from_box(layouter: Box<dyn CustomLayouter>) -> Self {
        Self {
            layouter,
            result: None,
        }
    }

    /// Returns a reference to the underlying [`CustomLayouter`].
    pub fn layouter(&self) -> &dyn CustomLayouter {
        &*self.layouter
    }

    /// Returns a mutable reference to the underlying [`CustomLayouter`].
    pub fn layouter_mut(&mut self) -> &mut dyn CustomLayouter {
        &mut *self.layouter
    }

    /// Returns the last layout result of this object, if any.
    pub fn result(&self) -> Option<&CustomObjectResult> {
        self.result.as_ref()
    }

    /// Returns a mutable reference to the last layout result, if any.
    pub fn result_mut(&mut self) -> Option<&mut CustomObjectResult> {
        self.result.as_mut()
    }

    /// Stores the layout result produced by the engine.
    pub(crate) fn set_result(&mut self, result: CustomObjectResult) {
        self.result = Some(result);
    }
}

/// A unified layout item used during layout processing.
///
/// `LayoutChild` represents the kinds of children a [`LayoutNode`] can have:
///
/// - [`Node`](LayoutChild::Node) — a nested layout node (block, inline, flex, etc.)
/// - [`Fragment`](LayoutChild::Fragment) — an inline-level content fragment
/// - [`Custom`](LayoutChild::Custom) — a custom layout object (inline or block)
///
/// This abstraction allows the layout engine to treat structural elements,
/// inline fragments, and custom objects uniformly while preserving their order.
#[derive(Debug)]
pub enum LayoutChild {
    /// A nested layout node.
    ///
    /// The child participates in the parent's formatting context according
    /// to its own [`crate::Style::display`] property.
    Node(Box<LayoutNode>),
    /// An inline-level content fragment.
    ///
    /// Fragments are the smallest independently-positionable unit of inline
    /// content (text, images, etc.).  They can be split across lines.
    Fragment(FragmentNode),
    /// A custom layout object that implements [`CustomLayouter`].
    ///
    /// Custom objects can participate in both inline and block formatting
    /// contexts. The object determines its own layout behavior based on
    /// the context in which it's used.
    Custom(CustomChild),
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

    /// Returns a reference to the underlying [`CustomLayouter`] if this
    /// child is a [`Custom`](LayoutChild::Custom).
    pub fn custom(&self) -> Option<&dyn CustomLayouter> {
        match self {
            LayoutChild::Custom(c) => Some(c.layouter()),
            _ => None,
        }
    }

    /// Returns a mutable reference to the underlying [`CustomLayouter`]
    /// if this child is a [`Custom`](LayoutChild::Custom).
    pub fn custom_mut(&mut self) -> Option<&mut dyn CustomLayouter> {
        match self {
            LayoutChild::Custom(c) => Some(c.layouter_mut()),
            _ => None,
        }
    }

    /// Returns a reference to the underlying [`CustomChild`] if this
    /// child is a [`Custom`](LayoutChild::Custom).
    pub fn custom_child(&self) -> Option<&CustomChild> {
        match self {
            LayoutChild::Custom(c) => Some(c),
            _ => None,
        }
    }

    /// Returns a mutable reference to the underlying [`CustomChild`] if this
    /// child is a [`Custom`](LayoutChild::Custom).
    pub fn custom_child_mut(&mut self) -> Option<&mut CustomChild> {
        match self {
            LayoutChild::Custom(c) => Some(c),
            _ => None,
        }
    }

    /// Returns the layout result of this child if it is a
    /// [`Custom`](LayoutChild::Custom) that has been laid out.
    pub fn custom_result(&self) -> Option<&CustomObjectResult> {
        self.custom_child().and_then(|c| c.result())
    }

    /// Returns true if this child is a [`Custom`](LayoutChild::Custom).
    pub fn is_custom(&self) -> bool {
        matches!(self, LayoutChild::Custom(_))
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

impl<L> From<L> for LayoutChild
where
    L: CustomLayouter + 'static,
{
    fn from(layouter: L) -> Self {
        LayoutChild::Custom(CustomChild::new(layouter))
    }
}
