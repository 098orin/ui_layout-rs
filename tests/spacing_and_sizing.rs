use ui_layout::*;

fn node<'a>(n: &'a LayoutNode, idx: usize) -> &'a LayoutNode {
    n.children[idx].node().expect("expected node child")
}

#[test]
fn padding_content_box() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(200.0)),
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(15.0),
            padding_top: Length::Px(5.0),
            padding_bottom: Length::Px(8.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_box {
        LayoutBox::BlockBox(box_model) => {
            assert_eq!(box_model.content_box.width, 200.0);
            assert_eq!(box_model.content_box.height, 100.0);
            assert_eq!(box_model.padding_box.width, 225.0); // 200 + 10 + 15
            assert_eq!(box_model.padding_box.height, 113.0); // 100 + 5 + 8
        }
        _ => panic!("Expected block box"),
    }
}

#[test]
fn border_box_sizing() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(200.0)),
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        box_sizing: BoxSizing::BorderBox,
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(15.0),
            padding_top: Length::Px(5.0),
            padding_bottom: Length::Px(8.0),
            border_left: Length::Px(2.0),
            border_right: Length::Px(3.0),
            border_top: Length::Px(1.0),
            border_bottom: Length::Px(4.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    match &root.layout_box {
        LayoutBox::BlockBox(box_model) => {
            assert_eq!(box_model.border_box.width, 200.0);
            assert_eq!(box_model.border_box.height, 100.0);
            assert_eq!(box_model.padding_box.width, 195.0); // 200 - 2 - 3
            assert_eq!(box_model.padding_box.height, 95.0); // 100 - 1 - 4
            assert_eq!(box_model.content_box.width, 170.0); // 200 - 2 -3 -10 -15
            assert_eq!(box_model.content_box.height, 82.0); // 100 -1 -4 -5 -8
        }
        _ => panic!("Expected block box"),
    }
}

#[test]
fn margins_affect_positioning_simple() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Length(Length::Px(20.0)),
            margin_top: LengthOrAuto::Length(Length::Px(10.0)),
            margin_right: LengthOrAuto::Length(Length::Px(30.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let inner = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        vec![child],
    );

    let mut root = LayoutNode::with_children(Style::default(), vec![inner]);

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let c = node(&root.children[0].node().unwrap(), 0);
    match &c.layout_box {
        LayoutBox::BlockBox(box_model) => {
            assert_eq!(box_model.border_box.x, 20.0);
            assert_eq!(box_model.border_box.y, 10.0);
            assert_eq!(box_model.border_box.width, 100.0);
            assert_eq!(box_model.border_box.height, 50.0);
        }
        _ => panic!("Expected block box"),
    }
}
