mod common;
use common::*;
use ui_layout::*;

fn grow_item() -> LayoutNode {
    LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_grow: 1.0,
            flex_basis: LengthOrAuto::Length(Length::Percent(0.0)),
            ..Default::default()
        },
        ..Default::default()
    })
}

fn display_none_grow_item() -> LayoutNode {
    LayoutNode::new(Style {
        display: Display::None,
        item_style: ItemStyle {
            flex_grow: 1.0,
            flex_basis: LengthOrAuto::Length(Length::Percent(0.0)),
            ..Default::default()
        },
        ..Default::default()
    })
}

fn swap_button() -> LayoutNode {
    new_child(48.0, 48.0)
}

/// A `display: none` flex item must not participate in flex layout.
#[test]
fn display_none_flex_item_is_excluded() {
    let mut root = LayoutNode::with_children(
        flex_container(584.0, 100.0, FlexDirection::Row),
        [grow_item(), display_none_grow_item()],
    );

    LayoutEngine::layout(&mut root, 1200.0, 900.0);

    assert_eq!(block_box(node(&root, 0)).content_box.width, 584.0);
}

/// A `display: none` flex item must not consume flex-grow space.
#[test]
fn display_none_flex_item_does_not_consume_grow_space() {
    let mut root = LayoutNode::with_children(
        flex_container(1176.0, 48.0, FlexDirection::Row),
        [
            grow_item(),
            display_none_grow_item(),
            swap_button(),
            grow_item(),
        ],
    );

    LayoutEngine::layout(&mut root, 1200.0, 900.0);

    assert!(approx_eq(
        block_box(node(&root, 0)).content_box.width,
        564.0
    ));
    assert!(approx_eq(
        block_box(node(&root, 3)).content_box.width,
        564.0
    ));
}

/// A `display: none` flex item does not generate a box, so its margins do not
/// affect the layout of the remaining flex items.
#[test]
fn display_none_flex_item_does_not_apply_margins() {
    let mut root = LayoutNode::with_children(
        flex_container(1176.0, 48.0, FlexDirection::Row),
        [
            display_none_grow_item(),
            grow_item(),
            swap_button(),
            grow_item(),
            display_none_grow_item(),
        ],
    );

    LayoutEngine::layout(&mut root, 1200.0, 900.0);

    assert!(approx_eq(
        block_box(node(&root, 1)).content_box.width,
        564.0
    ));
    assert!(approx_eq(
        block_box(node(&root, 3)).content_box.width,
        564.0
    ));
}
