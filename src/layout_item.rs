use crate::{ItemFragment, LayoutNode};

/// A unified layout item used during layout processing.
///
/// `LayoutItem` represents a single unit in the layout flow and can be either:
///
/// - A child layout node (`LayoutNode`)
/// - An inline-level fragment (`ItemFragment`)
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
