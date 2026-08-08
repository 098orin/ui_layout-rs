mod common;
use common::*;
use ui_layout::*;

fn grid_container(width: f32, columns: Vec<GridTrack>) -> Style {
    Style {
        display: Display {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Grid,
        },
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(width)),
            ..Default::default()
        },
        grid_template_columns: columns,
        ..Default::default()
    }
}

#[test]
fn fraction_tracks_share_remaining_width() {
    let mut style = grid_container(310.0, vec![GridTrack::Flex(1.0), GridTrack::Flex(2.0)]);
    style.column_gap = LengthOrAuto::Length(Length::Px(10.0));
    let mut root = LayoutNode::with_children(style, [new_child(40.0), new_child(20.0)]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let first = block_box(node(&root, 0));
    let second = block_box(node(&root, 1));
    assert_eq!(first.border_box, rect(0.0, 0.0, 100.0, 40.0));
    assert_eq!(second.border_box, rect(110.0, 0.0, 200.0, 20.0));
}

#[test]
fn auto_placement_fills_rows_in_source_order() {
    let mut style = grid_container(
        200.0,
        vec![GridTrack::Breadth(LengthOrAuto::Length(Length::Px(100.0,))); 2],
    );
    style.grid_template_rows = vec![
        GridTrack::Breadth(LengthOrAuto::Length(Length::Px(30.0))),
        GridTrack::Breadth(LengthOrAuto::Length(Length::Px(40.0))),
    ];
    let mut root =
        LayoutNode::with_children(style, [new_child(10.0), new_child(10.0), new_child(10.0)]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 100.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 0.0);
    assert_eq!(block_box(node(&root, 2)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 2)).border_box.y, 30.0);
}

#[test]
fn explicit_item_can_span_tracks() {
    let style = grid_container(
        300.0,
        vec![
            GridTrack::Flex(1.0),
            GridTrack::Flex(1.0),
            GridTrack::Flex(1.0),
        ],
    );
    let child = LayoutNode::new(Style {
        grid_column: GridPlacement {
            start: Some(2),
            span: 2,
        },
        grid_row: GridPlacement {
            start: Some(1),
            span: 1,
        },
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(25.0)),
            ..Default::default()
        },
        ..Default::default()
    });
    let mut root = LayoutNode::with_children(style, [child]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(
        block_box(node(&root, 0)).border_box,
        rect(100.0, 0.0, 200.0, 25.0)
    );
}

#[test]
fn auto_track_uses_item_intrinsic_width() {
    let style = grid_container(250.0, vec![GridTrack::default(), GridTrack::Flex(1.0)]);
    let fixed = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });
    let mut root = LayoutNode::with_children(style, [fixed, new_child(20.0)]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.width, 50.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 50.0);
    assert_eq!(block_box(node(&root, 1)).border_box.width, 200.0);
}
