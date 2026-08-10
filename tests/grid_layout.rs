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

#[test]
fn fixed_repeat_expands_track_pattern() {
    let style = grid_container(
        300.0,
        vec![GridTrack::Repeat(
            GridRepeat::Count(3),
            vec![GridTrack::Flex(1.0)],
        )],
    );
    let mut root =
        LayoutNode::with_children(style, [new_child(10.0), new_child(10.0), new_child(10.0)]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 100.0);
    assert_eq!(block_box(node(&root, 2)).border_box.x, 200.0);
}

#[test]
fn auto_fit_minmax_collapses_empty_tracks() {
    let mut style = grid_container(
        550.0,
        vec![GridTrack::Repeat(
            GridRepeat::AutoFit,
            vec![GridTrack::MinMax(
                Box::new(GridTrack::Breadth(LengthOrAuto::Length(Length::Px(100.0)))),
                Box::new(GridTrack::Flex(1.0)),
            )],
        )],
    );
    style.column_gap = LengthOrAuto::Length(Length::Px(10.0));
    let mut root =
        LayoutNode::with_children(style, [new_child(10.0), new_child(10.0), new_child(10.0)]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert!((block_box(node(&root, 0)).border_box.width - 176.66667).abs() < 0.01);
    assert!((block_box(node(&root, 1)).border_box.x - 186.66667).abs() < 0.01);
    assert!((block_box(node(&root, 2)).border_box.x - 373.33334).abs() < 0.01);
}

#[test]
fn named_areas_place_and_span_items() {
    let mut style = grid_container(300.0, vec![GridTrack::Flex(1.0), GridTrack::Flex(2.0)]);
    style.grid_template_areas = vec![
        vec!["header".into(), "header".into()],
        vec!["sidebar".into(), "main".into()],
        vec!["footer".into(), "footer".into()],
    ];
    let item = |area: &str| {
        LayoutNode::new(Style {
            grid_area: Some(area.into()),
            size: SizeStyle {
                height: LengthOrAuto::Length(Length::Px(20.0)),
                ..Default::default()
            },
            ..Default::default()
        })
    };
    let mut root = LayoutNode::with_children(
        style,
        [
            item("header"),
            item("sidebar"),
            item("main"),
            item("footer"),
        ],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(
        block_box(node(&root, 0)).border_box,
        rect(0.0, 0.0, 300.0, 20.0)
    );
    assert_eq!(
        block_box(node(&root, 1)).border_box,
        rect(0.0, 20.0, 100.0, 20.0)
    );
    assert_eq!(
        block_box(node(&root, 2)).border_box,
        rect(100.0, 20.0, 200.0, 20.0)
    );
    assert_eq!(
        block_box(node(&root, 3)).border_box,
        rect(0.0, 40.0, 300.0, 20.0)
    );
}

#[test]
fn grid_children_are_relative_to_padded_content_box() {
    let mut style = grid_container(300.0, vec![GridTrack::Flex(1.0), GridTrack::Flex(1.0)]);
    style.column_gap = LengthOrAuto::Length(Length::Px(10.0));
    style.spacing = Spacing {
        padding_left: Length::Px(10.0),
        padding_right: Length::Px(10.0),
        padding_top: Length::Px(10.0),
        padding_bottom: Length::Px(10.0),
        border_left: Length::Px(2.0),
        border_right: Length::Px(2.0),
        border_top: Length::Px(2.0),
        border_bottom: Length::Px(2.0),
        ..Default::default()
    };
    let mut root = LayoutNode::with_children(style, [new_child(20.0), new_child(20.0)]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let root_box = block_box(&root);
    assert_eq!(root_box.content_box.x, 12.0);
    assert_eq!(root_box.content_box.y, 12.0);
    assert_eq!(root_box.content_box.width, 300.0);
    assert_eq!(root_box.border_box.width, 324.0);
    assert_eq!(block_box(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    assert_eq!(block_box(node(&root, 0)).border_box.width, 145.0);
    assert_eq!(block_box(node(&root, 1)).border_box.x, 155.0);
}

#[test]
fn mulpiple_inline_in_a_grid_container() {
    let first = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            ..Default::default()
        },
        [fragment(30.0, 10.0)],
    );
    let second = LayoutNode::with_children(
        Style {
            display: Display::parse("inline").unwrap(),
            ..Default::default()
        },
        [fragment(20.0, 10.0)],
    );
    let mut root = LayoutNode::with_children(
        grid_container(300.0, vec![GridTrack::Flex(1.0), GridTrack::Flex(1.0)]),
        [first, second],
    );

    LayoutEngine::layout(&mut root, 200.0, 100.0);

    assert_eq!(inline_box_model(node(&root, 0)).border_box.x, 0.0);
    assert_eq!(inline_box_model(node(&root, 1)).border_box.x, 150.0);

    let boxes: Vec<BoxModel> = node(&root, 1).layout_box.iter().collect();
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0].border_box.x, 150.0);
}
