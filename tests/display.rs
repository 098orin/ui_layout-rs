use ui_layout::*;

#[test]
fn display_default_node() {
    let node = LayoutNode::new(Style::default());
    assert_eq!(format!("{}", node), "LayoutNode\n");

    let mut node2 = LayoutNode::new(Style::default());
    node2.layout_box = LayoutBox::BlockBox(BoxModel {
        sticky_edges: None,
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
    assert_eq!(
        format!("{}", node2),
        "LayoutNode [width: 100px, height: 200px]\n"
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
        flex_direction: FlexDirection::Column,
        ..Style::default()
    };
    assert_eq!(
        format!("{}", styled),
        "display: flex, flex-direction: column"
    );
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
    assert_eq!(format!("{}", InnerDisplay::Grid), "grid");
    assert_eq!(format!("{}", FlexDirection::Column), "column");
    assert_eq!(format!("{}", JustifyContent::Center), "center");
    assert_eq!(format!("{}", AlignItems::Stretch), "stretch");
    assert_eq!(format!("{}", BoxSizing::BorderBox), "border-box");
    assert_eq!(format!("{}", LengthOrAuto::Auto), "auto");
}

#[test]
fn display_node_with_position_properties() {
    let mut node = LayoutNode::new(Style::default());
    node.style.position.kind = Position::Fixed;
    node.style.position.top = LengthOrAuto::Length(Length::Px(10.0));
    node.style.position.right = LengthOrAuto::Length(Length::Px(20.0));
    assert_eq!(
        format!("{}", node),
        "LayoutNode [position: fixed, top: 10px, right: 20px]\n"
    );
}

#[test]
fn display_style_position() {
    let style = Style {
        position: PositionStyle {
            kind: Position::Absolute,
            bottom: LengthOrAuto::Length(Length::Percent(5.0)),
            ..Default::default()
        },
        ..Style::default()
    };
    assert_eq!(format!("{}", style), "position: absolute, bottom: 5%");

    assert_eq!(format!("{}", Position::Static), "static");
    assert_eq!(format!("{}", Position::Relative), "relative");
    assert_eq!(format!("{}", Position::Absolute), "absolute");
    assert_eq!(format!("{}", Position::Fixed), "fixed");
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
        sticky_edges: None,
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
    assert_eq!(s, "LayoutNode [display: flex] block(800x600 @0,0)\n");
}

#[test]
fn display_alternate_inline_box() {
    let mut node = LayoutNode::new(Style::default());
    node.layout_box = LayoutBox::InlineBox(InlineBox {
        box_model: BoxModel {
            sticky_edges: None,
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
    assert_eq!(s, "LayoutNode inline(100x60 @5,10 [(5,10), (5,40)])\n");
}

// --- LengthOrAuto ---

#[test]
fn length_or_auto_length_extracts_length() {
    let l = LengthOrAuto::Length(Length::Px(42.0));
    assert_eq!(l.clone().length(), Some(Length::Px(42.0)));

    let auto = LengthOrAuto::Auto;
    assert_eq!(auto.length(), None);
}

#[test]
fn length_or_auto_resolve_with() {
    let l = LengthOrAuto::Length(Length::Px(100.0));
    assert_eq!(l.resolve_with(None, 800.0, 600.0), Some(100.0));

    let auto = LengthOrAuto::Auto;
    assert_eq!(auto.resolve_with(None, 800.0, 600.0), None);
}

// --- Display parsing utilities ---

#[test]
fn display_from_css_name_all_variants() {
    assert_eq!(
        Display::from_css_name("block").unwrap(),
        Display::OutsideInner {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Flow,
        }
    );
    assert_eq!(
        Display::from_css_name("inline").unwrap(),
        Display::OutsideInner {
            outer: OuterDisplay::Inline,
            inner: InnerDisplay::Flow,
        }
    );
    assert_eq!(
        Display::from_css_name("flex").unwrap(),
        Display::OutsideInner {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Flex,
        }
    );
    assert_eq!(
        Display::from_css_name("inline-flex").unwrap(),
        Display::OutsideInner {
            outer: OuterDisplay::Inline,
            inner: InnerDisplay::Flex,
        }
    );
    assert_eq!(
        Display::from_css_name("grid").unwrap(),
        Display::OutsideInner {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Grid,
        }
    );
    assert_eq!(
        Display::from_css_name("inline-grid").unwrap(),
        Display::OutsideInner {
            outer: OuterDisplay::Inline,
            inner: InnerDisplay::Grid,
        }
    );
    assert_eq!(Display::from_css_name("none").unwrap(), Display::None);
    assert!(Display::from_css_name("unknown").is_none());
}

#[test]
fn display_from_css_multi_token() {
    let (outer, inner) = Display::from_css("block flow");
    assert_eq!(outer, Some(OuterDisplay::Block));
    assert_eq!(inner, Some(InnerDisplay::Flow));

    let (outer, inner) = Display::from_css("inline flex");
    assert_eq!(outer, Some(OuterDisplay::Inline));
    assert_eq!(inner, Some(InnerDisplay::Flex));

    let (outer, inner) = Display::from_css("flex");
    assert_eq!(outer, None);
    assert_eq!(inner, Some(InnerDisplay::Flex));

    let (outer, inner) = Display::from_css("");
    assert_eq!(outer, None);
    assert_eq!(inner, None);

    let (outer, inner) = Display::from_css("garbage");
    assert_eq!(outer, None);
    assert_eq!(inner, None);
}

#[test]
fn display_from_str() {
    use std::str::FromStr;
    let d = Display::from_str("flex").unwrap();
    assert_eq!(d.outer(), Some(OuterDisplay::Block));
    assert_eq!(d.inner(), Some(InnerDisplay::Flex));

    assert!(Display::from_str("invalid").is_err());
}

// --- Length arithmetic ---

#[test]
fn length_add_sub() {
    let a = Length::Px(10.0);
    let b = Length::Px(20.0);
    assert_eq!((a + b).resolve_with(None, 800.0, 600.0), Some(30.0));

    let c = Length::Px(50.0);
    let d = Length::Px(15.0);
    assert_eq!((c - d).resolve_with(None, 800.0, 600.0), Some(35.0));
}

#[test]
fn length_resolve_with_all_variants() {
    let vw = Length::Vw(50.0);
    assert_eq!(vw.resolve_with(None, 800.0, 600.0), Some(400.0));

    let vh = Length::Vh(25.0);
    assert_eq!(vh.resolve_with(None, 800.0, 600.0), Some(150.0));

    let pct = Length::Percent(30.0);
    assert_eq!(pct.resolve_with(Some(200.0), 800.0, 600.0), Some(60.0));
    assert_eq!(pct.resolve_with(None, 800.0, 600.0), None);

    let min = Length::Min(Box::new(Length::Px(10.0)), Box::new(Length::Px(30.0)));
    assert_eq!(min.resolve_with(None, 800.0, 600.0), Some(10.0));

    let max = Length::Max(Box::new(Length::Px(10.0)), Box::new(Length::Px(30.0)));
    assert_eq!(max.resolve_with(None, 800.0, 600.0), Some(30.0));

    let clamp = Length::Clamp {
        min: Box::new(Length::Px(5.0)),
        val: Box::new(Length::Px(50.0)),
        max: Box::new(Length::Px(20.0)),
    };
    assert_eq!(clamp.resolve_with(None, 800.0, 600.0), Some(20.0));

    let div = Length::Div(Box::new(Length::Px(100.0)), 0.0);
    assert_eq!(div.resolve_with(None, 800.0, 600.0), None);

    let mul = Length::Mul(Box::new(Length::Px(10.0)), 3.0);
    assert_eq!(mul.resolve_with(None, 800.0, 600.0), Some(30.0));
}

// --- LayoutChild accessors ---

#[test]
fn layout_child_node_mut_modifies_child() {
    let mut child = LayoutChild::from(LayoutNode::new(Style::default()));
    assert!(child.node().is_some());
    assert!(child.fragment().is_none());

    if let Some(n) = child.node_mut() {
        n.style.size.width = LengthOrAuto::Length(Length::Px(100.0));
    }
    assert_eq!(
        child.node().unwrap().style.size.width,
        LengthOrAuto::Length(Length::Px(100.0))
    );
}

#[test]
fn layout_child_fragment_mut_modifies_fragment() {
    let mut child = LayoutChild::from(ItemFragment::Fragment(Fragment {
        width: 10.0,
        height: 5.0,
    }));
    assert!(child.fragment().is_some());
    assert!(child.node().is_none());

    if let Some(f) = child.fragment_mut() {
        f.placement.offset = (42.0, 10.0);
    }
    assert_eq!(child.fragment().unwrap().placement.offset, (42.0, 10.0));
}

// --- FlexDirection defaults ---

#[test]
fn flex_direction_default_is_row() {
    assert_eq!(FlexDirection::default(), FlexDirection::Row);
}

// --- ContentBox variant display ---

#[test]
fn display_content_box_variant() {
    assert_eq!(format!("{}", BoxSizing::ContentBox), "content-box");
    assert_eq!(format!("{}", BoxSizing::BorderBox), "border-box");
}

#[test]
fn display_layout_box_variants() {
    let none = LayoutBox::None;
    assert_eq!(format!("{}", none), "none");
    assert_eq!(format!("{:#}", none), "none");

    let b = LayoutBox::BlockBox(BoxModel {
        sticky_edges: None,
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
    assert_eq!(format!("{}", b), "block(100x50 @10,20)");
    assert_eq!(format!("{:#}", b), "block(100x50 @10,20)");
}

#[test]
fn display_inline_box_full_variant() {
    let inline = LayoutBox::InlineBox(InlineBox {
        box_model: BoxModel {
            sticky_edges: None,
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
        line_spans: vec![LineSpan {
            x_range: 5.0..105.0,
            line_pos: (5.0, 10.0),
            line_index: 0,
        }],
    });
    assert_eq!(format!("{}", inline), "inline(100x30 @5,10 [(5,10)])");
}

// --- Owned LayoutBox into_iter ---

#[test]
fn owned_layout_box_into_iter_block() {
    let block = LayoutBox::BlockBox(BoxModel {
        sticky_edges: None,
        border_box: rect(0.0, 0.0, 100.0, 50.0),
        padding_box: rect(0.0, 0.0, 100.0, 50.0),
        content_box: rect(0.0, 0.0, 100.0, 50.0),
        children_box: rect(0.0, 0.0, 100.0, 50.0),
    });
    let boxes: Vec<BoxModel> = block.into_iter().collect();
    assert_eq!(boxes.len(), 1);
    assert_eq!(boxes[0].content_box.width, 100.0);
}

#[test]
fn owned_layout_box_into_iter_none() {
    let none = LayoutBox::None;
    let boxes: Vec<BoxModel> = none.into_iter().collect();
    assert!(boxes.is_empty());
}

#[test]
fn owned_layout_box_into_iter_empty_inline() {
    let inline = LayoutBox::InlineBox(InlineBox {
        box_model: BoxModel {
            sticky_edges: None,
            border_box: rect(0.0, 0.0, 50.0, 20.0),
            padding_box: rect(0.0, 0.0, 50.0, 20.0),
            content_box: rect(0.0, 0.0, 50.0, 20.0),
            children_box: rect(0.0, 0.0, 50.0, 20.0),
        },
        line_spans: vec![],
    });
    let boxes: Vec<BoxModel> = inline.into_iter().collect();
    assert_eq!(boxes.len(), 1);
}

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}
