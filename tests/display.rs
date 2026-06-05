use ui_layout::*;

#[test]
fn display_default_node() {
    let node = LayoutNode::new(Style::default());
    assert_eq!(format!("{}", node), "LayoutNode\n");

    let mut node2 = LayoutNode::new(Style::default());
    node2.layout_box = LayoutBox::BlockBox(BoxModel {
        border_box: Rect { x: 10.0, y: 20.0, width: 100.0, height: 50.0 },
        padding_box: Rect { x: 12.0, y: 22.0, width: 96.0, height: 46.0 },
        content_box: Rect { x: 14.0, y: 24.0, width: 92.0, height: 42.0 },
        children_box: Rect { x: 14.0, y: 24.0, width: 92.0, height: 42.0 },
    });
    assert_eq!(format!("{}", node2), "LayoutNode\n");
    assert_eq!(format!("{:#}", node2), "LayoutNode block(100x50 @10,20)\n");
}

#[test]
fn display_node_with_style_properties() {
    let mut node = LayoutNode::new(Style::default());
    node.style.display = Display::parse("flex").unwrap();
    assert_eq!(format!("{}", node), "LayoutNode [display: flex]\n");

    let mut node2 = LayoutNode::new(Style::default());
    node2.style.size.width = LengthOrAuto::Length(Length::Px(100.0));
    node2.style.size.height = LengthOrAuto::Length(Length::Px(200.0));
    node2.style.flex_direction = FlexDirection::Row;
    assert_eq!(
        format!("{}", node2),
        "LayoutNode [width: 100px, height: 200px, flex-direction: row]\n"
    );
}

#[test]
fn display_tree_structure() {
    let child_a = LayoutNode::new(Style::default());
    let child_b = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..SizeStyle::default()
        },
        ..Style::default()
    });
    let root = LayoutNode::with_children(Style::default(), vec![child_a, child_b]);

    assert_eq!(
        format!("{}", root),
        "LayoutNode\n├── LayoutNode\n└── LayoutNode [height: 100px]\n"
    );
}

#[test]
fn display_style_formatting() {
    let default_style = Style::default();
    assert_eq!(format!("{}", default_style), "(default)");

    let styled = Style {
        display: Display::parse("flex").unwrap(),
        flex_direction: FlexDirection::Row,
        ..Style::default()
    };
    assert_eq!(format!("{}", styled), "display: flex, flex-direction: row");
}

#[test]
fn display_margin_shorthand() {
    let uniform = Style {
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(10.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(10.0)),
            margin_left: LengthOrAuto::Length(Length::Px(10.0)),
            margin_right: LengthOrAuto::Length(Length::Px(10.0)),
            ..Spacing::default()
        },
        ..Style::default()
    };
    assert_eq!(format!("{}", uniform), "margin: 10px");

    let partial = Style {
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(10.0)),
            ..Spacing::default()
        },
        ..Style::default()
    };
    assert_eq!(format!("{}", partial), "margin-top: 10px");
}

#[test]
fn display_length_and_enum_variants() {
    assert_eq!(format!("{}", Length::Px(42.0)), "42px");
    assert_eq!(format!("{}", Length::Percent(50.0)), "50%");
    assert_eq!(format!("{}", Length::Vw(25.0)), "25vw");
    assert_eq!(format!("{}", Length::Vh(75.0)), "75vh");

    assert_eq!(format!("{}", OuterDisplay::Block), "block");
    assert_eq!(format!("{}", InnerDisplay::Flex), "flex");
    assert_eq!(format!("{}", FlexDirection::Column), "column");
    assert_eq!(format!("{}", JustifyContent::Center), "center");
    assert_eq!(format!("{}", AlignItems::Stretch), "stretch");
    assert_eq!(format!("{}", BoxSizing::BorderBox), "border-box");
    assert_eq!(format!("{}", LengthOrAuto::Auto), "auto");
}

#[test]
fn display_placement() {
    let p = Placement {
        offset: (42.5, 10.0),
        line_index: 2,
    };
    assert_eq!(format!("{}", p), "(42.5, 10) @line 2");

    let default_p = Placement::default();
    assert_eq!(format!("{}", default_p), "(0, 0) @line 0");
}

#[test]
fn display_alternate_with_style_and_box() {
    let mut node = LayoutNode::new(Style::default());
    node.style.display = Display::parse("flex").unwrap();
    node.style.flex_direction = FlexDirection::Row;
    node.layout_box = LayoutBox::BlockBox(BoxModel {
        border_box: Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 },
        padding_box: Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 },
        content_box: Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 },
        children_box: Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 },
    });
    let s = format!("{:#}", node);
    assert_eq!(
        s,
        "LayoutNode [display: flex, flex-direction: row] block(800x600 @0,0)\n"
    );
}

#[test]
fn display_alternate_inline_box() {
    let mut node = LayoutNode::new(Style::default());
    node.layout_box = LayoutBox::InlineBox(InlineBox {
        box_model: BoxModel {
            border_box: Rect { x: 5.0, y: 10.0, width: 200.0, height: 30.0 },
            padding_box: Rect { x: 5.0, y: 10.0, width: 200.0, height: 30.0 },
            content_box: Rect { x: 5.0, y: 10.0, width: 200.0, height: 30.0 },
            children_box: Rect { x: 5.0, y: 10.0, width: 200.0, height: 30.0 },
        },
        line_spans: vec![
            LineSpan {
                x_range: 5.0..105.0,
                line_pos: (5.0, 10.0),
                line_index: 0,
            },
            LineSpan {
                x_range: 105.0..205.0,
                line_pos: (5.0, 40.0),
                line_index: 1,
            },
        ],
    });
    let s = format!("{:#}", node);
    assert_eq!(s, "LayoutNode inline(100x60 @5,10)\n");
}
