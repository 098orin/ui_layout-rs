mod common;
use common::*;
use ui_layout::*;

fn child(h: f32) -> LayoutNode {
    new_child(h, 50.0)
}

fn contents(children: Vec<LayoutNode>) -> LayoutNode {
    LayoutNode::with_children(
        Style {
            display: Display::Contents,
            ..Default::default()
        },
        children,
    )
}

// --- Block flow ---

#[test]
fn block_flattens_contents_children() {
    // parent
    // ├── child_a          (30)
    // ├── contents
    // │   ├── child_b      (20)
    // │   └── child_c      (10)
    // └── child_d          (40)
    let child_a = child(30.0);
    let child_b = child(20.0);
    let child_c = child(10.0);
    let child_d = child(40.0);
    let contents_node = contents(vec![child_b, child_c]);

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            ..Default::default()
        },
        [child_a, contents_node, child_d],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // The children of `display: contents` participate in the parent's block
    // formatting context as if the `display: contents` box did not exist.
    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    let contents_box = node(&root, 1);
    assert_eq!(block_box(node(contents_box, 0)).border_box.y, 30.0);
    assert_eq!(block_box(node(contents_box, 1)).border_box.y, 50.0);
    assert_eq!(block_box(node(&root, 2)).border_box.y, 60.0);

    // Parent content height accounts for all four children.
    assert_eq!(block_box(&root).content_box.height, 100.0);
}

#[test]
fn contents_node_layout_box_is_none_but_tree_shape_preserved() {
    let child_b = child(20.0);
    let contents_node = contents(vec![child_b]);

    let mut root = LayoutNode::with_children(
        Style {
            ..Default::default()
        },
        [contents_node],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // The `display: contents` node remains in the tree...
    assert_eq!(root.children.len(), 1);
    let contents_child = &root.children[0];
    let contents_node = contents_child.node().unwrap();

    // ...and has no layout box.
    assert!(matches!(contents_node.layout_box, LayoutBox::None));

    // Its child still participates in layout.
    let b = node(contents_node, 0);
    assert!(matches!(b.layout_box, LayoutBox::BlockBox(_)));
}

#[test]
fn contents_nested_flattening() {
    // parent
    // ├── contents
    // │   └── contents
    // │       ├── child_x   (15)
    // │       └── child_y   (25)
    // └── child_z            (10)
    let child_x = child(15.0);
    let child_y = child(25.0);
    let inner_contents = contents(vec![child_x, child_y]);
    let outer_contents = contents(vec![inner_contents]);
    let child_z = child(10.0);

    let mut root = LayoutNode::with_children(
        Style {
            ..Default::default()
        },
        [outer_contents, child_z],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let outer = node(&root, 0);
    let inner = node(outer, 0);
    assert!(matches!(outer.layout_box, LayoutBox::None));
    assert!(matches!(inner.layout_box, LayoutBox::None));

    assert_eq!(block_box(node(inner, 0)).border_box.y, 0.0);
    assert_eq!(block_box(node(inner, 1)).border_box.y, 15.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 40.0);
}

#[test]
fn empty_contents_node_contributes_nothing() {
    // parent
    // ├── child_a   (20)
    // ├── contents  (no children)
    // └── child_b   (30)
    let child_a = child(20.0);
    let empty_contents = contents(vec![]);
    let child_b = child(30.0);

    let mut root = LayoutNode::with_children(Style::default(), [child_a, empty_contents, child_b]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // The empty `display: contents` node adds no box and no children, so the
    // surrounding siblings stack as adjacent blocks.
    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    assert_eq!(block_box(node(&root, 2)).border_box.y, 20.0);
    assert_eq!(block_box(&root).content_box.height, 50.0);

    // The empty contents node itself produces no box.
    assert!(matches!(node(&root, 1).layout_box, LayoutBox::None));
}

#[test]
fn contents_in_flex_are_flex_items() {
    let mut root = LayoutNode::with_children(
        flex_container(100.0, 100.0, FlexDirection::Column),
        [
            child(20.0),
            contents(vec![child(30.0), child(10.0)]),
            child(40.0),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // The grandchildren under `display: contents` become flex items stacked
    // along the column after `child_a`: a(0), b(20), c(50), d(60).
    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    let contents_child = node(&root, 1);
    assert_eq!(block_box(node(contents_child, 0)).border_box.y, 20.0);
    assert_eq!(block_box(node(contents_child, 1)).border_box.y, 50.0);
    assert_eq!(block_box(node(&root, 2)).border_box.y, 60.0);

    // The `display: contents` node does not generate a flex item.
    assert!(matches!(node(&root, 1).layout_box, LayoutBox::None));
}

#[test]
fn contents_in_grid_are_grid_items() {
    let mut root = LayoutNode::with_children(
        Style {
            display: Display::OutsideInner {
                outer: OuterDisplay::Block,
                inner: InnerDisplay::Grid,
            },
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Length(Length::Px(200.0)),
                ..Default::default()
            },
            grid_template_columns: vec![
                GridTrack::Breadth(LengthOrAuto::Length(Length::Px(100.0))),
                GridTrack::Breadth(LengthOrAuto::Length(Length::Px(100.0))),
            ],
            grid_template_rows: vec![
                GridTrack::Breadth(LengthOrAuto::Length(Length::Px(100.0))),
                GridTrack::Breadth(LengthOrAuto::Length(Length::Px(100.0))),
            ],
            ..Default::default()
        },
        [
            child(10.0),
            contents(vec![child(10.0), child(10.0)]),
            child(10.0),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // The `display: contents` node does not generate a grid item. Its children
    // participate in the grid container's grid formatting context directly.
    // Four grid items are therefore auto-placed row-major into the 2x2 grid.
    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    let contents_child = node(&root, 1);
    assert_eq!(block_box(node(contents_child, 0)).border_box.y, 0.0);
    assert_eq!(block_box(node(contents_child, 1)).border_box.y, 100.0);
    assert_eq!(block_box(node(&root, 2)).border_box.y, 100.0);
}
