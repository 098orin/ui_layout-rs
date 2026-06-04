use ui_layout::*;

fn node<'a>(n: &'a LayoutNode, idx: usize) -> &'a LayoutNode {
    n.children[idx].node().expect("expected node child")
}

fn block_box(n: &LayoutNode) -> &BoxModel {
    match &n.layout_box {
        LayoutBox::BlockBox(b) => b,
        _ => panic!("expected block box"),
    }
}

// --- Box model ---

#[test]
fn block_basic_box_model() {
    let mut root = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(200.0)),
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        spacing: Spacing {
            padding_left: Length::Px(10.0),
            padding_right: Length::Px(10.0),
            padding_top: Length::Px(5.0),
            padding_bottom: Length::Px(5.0),
            border_left: Length::Px(2.0),
            border_right: Length::Px(2.0),
            border_top: Length::Px(1.0),
            border_bottom: Length::Px(1.0),
            ..Default::default()
        },
        ..Default::default()
    });

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let b = block_box(&root);
    assert_eq!(b.content_box.width, 200.0);
    assert_eq!(b.content_box.height, 100.0);
    assert_eq!(b.padding_box.width, 220.0);
    assert_eq!(b.padding_box.height, 110.0);
    assert_eq!(b.border_box.width, 224.0);
    assert_eq!(b.border_box.height, 112.0);
}

#[test]
fn block_auto_height_from_children() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(40.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).content_box.height, 40.0);
}

#[test]
fn block_margin_auto_centering() {
    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_left: LengthOrAuto::Auto,
            margin_right: LengthOrAuto::Auto,
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(300.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert_eq!(block_box(node(&root, 0)).border_box.x, 100.0);
}

// --- Padding ---

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

    let b = block_box(&root);
    assert_eq!(b.content_box.width, 200.0);
    assert_eq!(b.content_box.height, 100.0);
    assert_eq!(b.padding_box.width, 225.0);
    assert_eq!(b.padding_box.height, 113.0);
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

    let b = block_box(&root);
    assert_eq!(b.border_box.width, 200.0);
    assert_eq!(b.border_box.height, 100.0);
    assert_eq!(b.padding_box.width, 195.0);
    assert_eq!(b.padding_box.height, 95.0);
    assert_eq!(b.content_box.width, 170.0);
    assert_eq!(b.content_box.height, 82.0);
}

// --- Margins ---

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
        [child],
    );

    let mut root = LayoutNode::with_children(Style::default(), [inner]);
    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let c = node(&root.children[0].node().unwrap(), 0);
    let b = block_box(c);
    assert_eq!(b.border_box.x, 20.0);
    assert_eq!(b.border_box.y, 10.0);
    assert_eq!(b.border_box.width, 100.0);
    assert_eq!(b.border_box.height, 50.0);
}

// --- Margin collapsing ---

#[test]
fn block_vertical_margin_collapsing_between_siblings() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(100.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let c1 = block_box(node(&root, 0));
    let c2 = block_box(node(&root, 1));

    assert_eq!(c1.border_box.y, 0.0);
    assert_eq!(c2.border_box.y, c1.border_box.height + 30.0);
}

#[test]
fn auto_height_covers_deeply_nested_block_children_with_margins() {
    let def_style = Style {
        spacing: Spacing {
            margin_top: Length::Px(10.0).into(),
            margin_bottom: Length::Px(10.0).into(),
            margin_left: Length::Px(10.0).into(),
            margin_right: Length::Px(10.0).into(),
            ..Default::default()
        },
        ..Default::default()
    };

    let mut root = LayoutNode::new(Style {
        spacing: Spacing {
            ..Default::default()
        },
        ..def_style.clone()
    });

    fn push_child(parent: &mut LayoutNode, style: Style, max: usize, current: usize) {
        if current + 1 < max {
            parent.children.push(LayoutNode::new(style.clone()).into());
            push_child(
                parent.children[0].node_mut().unwrap(),
                style,
                max,
                current + 1,
            );
        } else {
            parent.children.push(
                LayoutNode::new(Style {
                    size: SizeStyle {
                        height: Length::Px(50.0).into(),
                        ..Default::default()
                    },
                    ..style
                })
                .into(),
            );
        }
    }

    for i in 0..10 {
        root.children
            .push(LayoutNode::new(def_style.clone()).into());
        push_child(
            root.children[i].node_mut().unwrap(),
            def_style.clone(),
            5,
            0,
        );
    }

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let root_box = block_box(&root);
    let first_child_box = block_box(node(&root, 0));
    let leaf = node(node(node(node(node(node(&root, 0), 0), 0), 0), 0), 0);
    let leaf_box = block_box(leaf);

    assert_eq!(leaf_box.border_box.y + leaf_box.border_box.height, 60.0);
    assert_eq!(first_child_box.border_box.height, 150.0);
    assert_eq!(root_box.border_box.height, 1610.0);
}

// --- Auto height ---

#[test]
fn multiple_block_children_auto_height() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert!((block_box(&root).content_box.height - 100.0).abs() < 0.1);
    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 30.0);
    assert_eq!(block_box(node(&root, 2)).border_box.y, 80.0);
}

#[test]
fn block_children_with_padding_in_parent_auto_height() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(5.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(5.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(10.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            spacing: Spacing {
                padding_top: Length::Px(8.0),
                padding_bottom: Length::Px(8.0),
                ..Default::default()
            },
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let root_box = block_box(&root);
    assert!((root_box.content_box.height - 75.0).abs() < 0.1);
    assert!((root_box.border_box.height - 91.0).abs() < 0.1);
    assert_eq!(root_box.content_box.y, 8.0);
}

#[test]
fn three_block_siblings_margin_collapse_auto_height() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(15.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(5.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    assert!((block_box(&root).content_box.height - 90.0).abs() < 0.1);
    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 40.0);
    assert_eq!(block_box(node(&root, 2)).border_box.y, 70.0);
}

#[test]
fn block_children_y_positions_are_consecutive() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let c1 = block_box(node(&root, 0));
    let c2 = block_box(node(&root, 1));
    let c3 = block_box(node(&root, 2));

    // Each child's y should be the sum of all previous children's heights.
    assert_eq!(c1.border_box.y, 0.0);
    assert_eq!(c2.border_box.y, c1.border_box.height);
    assert_eq!(c3.border_box.y, c1.border_box.height + c2.border_box.height);
    assert!((block_box(&root).content_box.height - 100.0).abs() < 0.1);
}

#[test]
fn three_block_children_with_margins_y_positions() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(20.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_bottom: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(5.0)),
            margin_bottom: LengthOrAuto::Length(Length::Px(15.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let child3 = LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(10.0)),
            ..Default::default()
        },
        spacing: Spacing {
            margin_top: LengthOrAuto::Length(Length::Px(8.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child1, child2, child3],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    // Collapsed margins:
    // between 1-2: max(10, 5) = 10
    // between 2-3: max(15, 8) = 15
    // child1: y=0, h=20, mb=10
    // child2: y=20+10=30, h=30, mb=15
    // child3: y=30+30+15=75, h=10
    // total: 20+10+30+15+10 = 85
    assert_eq!(block_box(node(&root, 0)).border_box.y, 0.0);
    assert_eq!(block_box(node(&root, 1)).border_box.y, 30.0);
    assert_eq!(block_box(node(&root, 2)).border_box.y, 75.0);
    assert!((block_box(&root).content_box.height - 85.0).abs() < 0.1);
}

#[test]
fn block_child_with_padding_followed_by_block_child() {
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Auto,
            height: LengthOrAuto::Length(Length::Px(30.0)),
            ..Default::default()
        },
        spacing: Spacing {
            padding_top: Length::Px(10.0),
            padding_bottom: Length::Px(10.0),
            ..Default::default()
        },
        ..Default::default()
    });

    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Auto,
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    let mut root = LayoutNode::with_children(
        Style {
            size: SizeStyle {
                width: LengthOrAuto::Length(Length::Px(200.0)),
                height: LengthOrAuto::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        [child1, child2],
    );

    LayoutEngine::layout(&mut root, 800.0, 600.0);

    let root_box = block_box(&root);
    let c1 = block_box(node(&root, 0));
    let c2 = block_box(node(&root, 1));

    // child1: padding_top=10, height=30, padding_bottom=10 → total border height = 50
    // child2: height=50, no padding
    // Total should be: child1 height(50) + child2 height(50) = 100
    assert_eq!(c1.border_box.y, 0.0, "child1 y should be 0");
    assert_eq!(
        c2.border_box.y, 50.0,
        "child2 y should be 50 (child1's full border height)"
    );
    assert!(
        (root_box.content_box.height - 100.0).abs() < 0.1,
        "root height should be 100"
    );
}
