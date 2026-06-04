use ui_layout::*;

#[test]
fn display_default_node() {
    let node = LayoutNode::new(Style::default());
    let s = format!("{}", node);
    assert_eq!(s, "LayoutNode\n");
}

#[test]
fn display_node_with_non_default_style() {
    let mut node = LayoutNode::new(Style::default());
    node.style.display = Display::parse("flex").unwrap();
    let s = format!("{}", node);
    assert_eq!(s, "LayoutNode [display: flex]\n");
}

#[test]
fn display_node_with_multiple_properties() {
    let mut node = LayoutNode::new(Style::default());
    node.style.size.width = LengthOrAuto::Length(Length::Px(100.0));
    node.style.size.height = LengthOrAuto::Length(Length::Px(200.0));
    node.style.flex_direction = FlexDirection::Row;
    let s = format!("{}", node);
    assert_eq!(
        s,
        "LayoutNode [width: 100px, height: 200px, flex-direction: row]\n"
    );
}

#[test]
fn display_tree_with_children() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            ..SizeStyle::default()
        },
        ..Style::default()
    });

    let root = LayoutNode::with_children(Style::default(), vec![child]);

    let s = format!("{}", root);
    assert_eq!(s, "LayoutNode\n└── LayoutNode [width: 50px]\n");
}

#[test]
fn display_tree_with_multiple_children() {
    let child_a = LayoutNode::new(Style::default());
    let child_b = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..SizeStyle::default()
        },
        ..Style::default()
    });

    let root = LayoutNode::with_children(Style::default(), vec![child_a, child_b]);

    let s = format!("{}", root);
    assert_eq!(
        s,
        "LayoutNode\n├── LayoutNode\n└── LayoutNode [height: 100px]\n"
    );
}

#[test]
fn display_style_default_prints_nothing() {
    let style = Style::default();
    let s = format!("{}", style);
    assert_eq!(s, "(default)");
}

#[test]
fn display_style_with_properties() {
    let style = Style {
        display: Display::parse("flex").unwrap(),
        flex_direction: FlexDirection::Row,
        ..Style::default()
    };
    let s = format!("{}", style);
    assert_eq!(s, "display: flex, flex-direction: row");
}

#[test]
fn display_nested_tree() {
    let grandchild = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(25.0)),
            height: LengthOrAuto::Length(Length::Px(25.0)),
            ..SizeStyle::default()
        },
        ..Style::default()
    });

    let child = LayoutNode::with_children(
        Style {
            item_style: ItemStyle {
                flex_grow: 1.0,
                ..ItemStyle::default()
            },
            ..Style::default()
        },
        vec![grandchild],
    );

    let root = LayoutNode::with_children(
        Style {
            display: Display::parse("flex").unwrap(),
            ..Style::default()
        },
        vec![child],
    );

    let s = format!("{}", root);
    let expected = "\
LayoutNode [display: flex]
└── LayoutNode [flex-grow: 1]
    └── LayoutNode [width: 25px, height: 25px]
";
    assert_eq!(s, expected);
}

#[test]
fn display_margin_shorthand() {
    let style = Style {
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(10.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(10.0)),
            margin_left: LengthOrAuto::Length(Length::Px(10.0)),
            margin_right: LengthOrAuto::Length(Length::Px(10.0)),
            ..Spacing::default()
        },
        ..Style::default()
    };
    let s = format!("{}", style);
    assert_eq!(s, "margin: 10px");
}

#[test]
fn display_partial_margins() {
    let style = Style {
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(10.0)),
            ..Spacing::default()
        },
        ..Style::default()
    };
    let s = format!("{}", style);
    assert_eq!(s, "margin-top: 10px");
}

#[test]
fn display_length_variants() {
    assert_eq!(format!("{}", Length::Px(42.0)), "42px");
    assert_eq!(format!("{}", Length::Percent(50.0)), "50%");
    assert_eq!(format!("{}", Length::Vw(25.0)), "25vw");
    assert_eq!(format!("{}", Length::Vh(75.0)), "75vh");
}

#[test]
fn display_enum_values() {
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
}

#[test]
fn display_placement_default() {
    let p = Placement::default();
    assert_eq!(format!("{}", p), "(0, 0) @line 0");
}

#[test]
fn display_layout_box_block() {
    let mut node = LayoutNode::new(Style::default());
    node.layout_box = LayoutBox::BlockBox(BoxModel {
        border_box: Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
        },
        padding_box: Rect {
            x: 12.0,
            y: 22.0,
            width: 96.0,
            height: 46.0,
        },
        content_box: Rect {
            x: 14.0,
            y: 24.0,
            width: 92.0,
            height: 42.0,
        },
        children_box: Rect {
            x: 14.0,
            y: 24.0,
            width: 92.0,
            height: 42.0,
        },
    });
    let s = format!("{:#}", node);
    assert_eq!(s, "LayoutNode block(100x50 @10,20)\n");
}

#[test]
fn display_layout_box_none() {
    let node = LayoutNode::new(Style::default());
    let s = format!("{:#}", node);
    assert_eq!(s, "LayoutNode none\n");
}

#[test]
fn display_alternate_shows_string() {
    let mut node = LayoutNode::new(Style::default());
    node.style.display = Display::parse("flex").unwrap();
    node.style.flex_direction = FlexDirection::Row;
    node.layout_box = LayoutBox::BlockBox(BoxModel {
        border_box: Rect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        },
        padding_box: Rect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        },
        content_box: Rect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        },
        children_box: Rect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        },
    });
    let s = format!("{:#}", node);
    assert_eq!(
        s,
        "LayoutNode [display: flex, flex-direction: row] block(800x600 @0,0)\n"
    );
}

#[test]
fn display_default_without_alternate_unchanged() {
    let mut node = LayoutNode::new(Style::default());
    node.layout_box = LayoutBox::BlockBox(BoxModel {
        border_box: Rect {
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
        },
        padding_box: Rect {
            x: 12.0,
            y: 22.0,
            width: 96.0,
            height: 46.0,
        },
        content_box: Rect {
            x: 14.0,
            y: 24.0,
            width: 92.0,
            height: 42.0,
        },
        children_box: Rect {
            x: 14.0,
            y: 24.0,
            width: 92.0,
            height: 42.0,
        },
    });
    // Without `#`, layout info is hidden
    let s = format!("{}", node);
    assert_eq!(s, "LayoutNode\n");
}

#[test]
fn display_alternate_inline_box() {
    let mut node = LayoutNode::new(Style::default());
    node.layout_box = LayoutBox::InlineBox(InlineBox {
        box_model: BoxModel {
            border_box: Rect {
                x: 5.0,
                y: 10.0,
                width: 200.0,
                height: 30.0,
            },
            padding_box: Rect {
                x: 5.0,
                y: 10.0,
                width: 200.0,
                height: 30.0,
            },
            content_box: Rect {
                x: 5.0,
                y: 10.0,
                width: 200.0,
                height: 30.0,
            },
            children_box: Rect {
                x: 5.0,
                y: 10.0,
                width: 200.0,
                height: 30.0,
            },
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
