use ui_layout::*;

/// Tests that flex_basis: auto uses the intrinsic content size of flex items.
/// This is the default behavior when no explicit flex-basis is specified.
#[test]
fn test_flex_basis_auto() {
    let mut container = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Flex,
        },
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(300.0)),
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        flex_direction: FlexDirection::Row,
        ..Default::default()
    });

    // Child 1: Content size 50px
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_basis: LengthOrAuto::Auto,
            ..Default::default()
        },
        ..Default::default()
    });

    // Child 2: Content size 80px
    let child2 = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(80.0)),
            height: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_basis: LengthOrAuto::Auto,
            ..Default::default()
        },
        ..Default::default()
    });

    container.children = vec![
        LayoutChild::Node(Box::new(child1)),
        LayoutChild::Node(Box::new(child2)),
    ];
    LayoutEngine::layout(&mut container, 800.0, 600.0);

    if let LayoutBox::BlockBox(ref container_box) = container.layout_box {
        assert_eq!(container_box.border_box.width, 300.0);
        assert_eq!(container_box.border_box.height, 100.0);
    }

    if let LayoutBox::BlockBox(ref child_box) = container.children[0].layout_box {
        assert_eq!(child_box.border_box.width, 50.0);
        assert_eq!(child_box.border_box.height, 50.0);
    }

    if let LayoutBox::BlockBox(ref child_box) = container.children[1].layout_box {
        assert_eq!(child_box.border_box.width, 80.0);
        assert_eq!(child_box.border_box.height, 50.0);
    }
}

#[test]
#[ignore = "The implementation of ignoring Width and treating it as basis when flexing is not implemented yet."]
fn flex_basis_overrides_width_when_no_grow_or_shrink() {
    let mut container = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Flex,
        },
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(300.0)),
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        flex_direction: FlexDirection::Row,
        ..Default::default()
    });

    let child = LayoutNode::new(Style {
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(50.0)),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_basis: LengthOrAuto::Length(Length::Px(120.0)),
            flex_grow: 0.0,
            flex_shrink: 0.0,
            ..Default::default()
        },
        ..Default::default()
    });

    container.children = vec![LayoutChild::Node(Box::new(child))];
    LayoutEngine::layout(&mut container, 800.0, 600.0);

    if let LayoutBox::BlockBox(ref child_box) = container.children[0].layout_box {
        assert_eq!(child_box.border_box.width, 120.0);
    }
}

#[test]
fn flex_basis_is_starting_point_for_grow() {
    let mut container = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Flex,
        },
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(300.0)),
            ..Default::default()
        },
        flex_direction: FlexDirection::Row,
        ..Default::default()
    });

    let child = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: LengthOrAuto::Length(Length::Px(100.0)),
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    container.children = vec![LayoutChild::Node(Box::new(child))];
    LayoutEngine::layout(&mut container, 800.0, 600.0);

    if let LayoutBox::BlockBox(ref child_box) = container.children[0].layout_box {
        assert_eq!(child_box.border_box.width, 300.0);
    }
}

#[test]
fn flex_basis_is_starting_point_for_shrink() {
    let mut container = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Flex,
        },
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        flex_direction: FlexDirection::Row,
        ..Default::default()
    });

    let child = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: LengthOrAuto::Length(Length::Px(200.0)),
            flex_shrink: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    container.children = vec![LayoutChild::Node(Box::new(child))];
    LayoutEngine::layout(&mut container, 800.0, 600.0);

    if let LayoutBox::BlockBox(ref child_box) = container.children[0].layout_box {
        assert_eq!(child_box.border_box.width, 100.0);
    }
}

/// Tests the interaction between flex_basis and flex_grow.
/// flex_basis sets the initial main size, then flex_grow distributes
/// any remaining space proportionally among growing items.
#[test]
fn test_flex_basis_with_grow() {
    let mut container = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Flex,
        },
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(400.0)),
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        flex_direction: FlexDirection::Row,
        ..Default::default()
    });

    // Child 1: flex-basis 100px, flex-grow 1
    let child1 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: LengthOrAuto::Length(Length::Px(100.0)),
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    // Child 2: flex-basis 100px, flex-grow 2
    let child2 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: LengthOrAuto::Length(Length::Px(100.0)),
            flex_grow: 2.0,
            ..Default::default()
        },
        ..Default::default()
    });

    container.children = vec![
        LayoutChild::Node(Box::new(child1)),
        LayoutChild::Node(Box::new(child2)),
    ];
    LayoutEngine::layout(&mut container, 800.0, 600.0);

    // Total basis=200px, Remaining=200px
    // Child1=100+66.67≈167px, Child2=100+133.33≈233px
    if let LayoutBox::BlockBox(ref child_box) = container.children[0].layout_box {
        assert!((child_box.border_box.width - 166.7).abs() < 0.1);
    }

    if let LayoutBox::BlockBox(ref child_box) = container.children[1].layout_box {
        assert!((child_box.border_box.width - 233.3).abs() < 0.1);
    }
}

/// Tests the interaction between flex_basis and flex_shrink.
/// When the total flex_basis exceeds the container size, flex_shrink
/// reduces item sizes proportionally based on their shrink factors.
#[test]
fn test_flex_basis_with_shrink() {
    let mut container = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Flex,
        },
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(200.0)),
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        flex_direction: FlexDirection::Row,
        ..Default::default()
    });

    // Child 1: flex-basis 150px, flex-shrink 1
    let child1 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: LengthOrAuto::Length(Length::Px(150.0)),
            flex_shrink: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    // Child 2: flex-basis 100px, flex-shrink 2
    let child2 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: LengthOrAuto::Length(Length::Px(100.0)),
            flex_shrink: 2.0,
            ..Default::default()
        },
        ..Default::default()
    });

    container.children = vec![
        LayoutChild::Node(Box::new(child1)),
        LayoutChild::Node(Box::new(child2)),
    ];
    LayoutEngine::layout(&mut container, 800.0, 600.0);

    // Total basis=250px, Overflow=50px
    // Child1 shrink factor: 150*1=150, Child2: 100*2=200
    // Child1 shrinks: 50*(150/350)≈21.4px → 128.6px
    // Child2 shrinks: 50*(200/350)≈28.6px → 71.4px
    if let LayoutBox::BlockBox(ref child_box) = container.children[0].layout_box {
        assert!((child_box.border_box.width - 128.6).abs() < 0.1);
    }

    if let LayoutBox::BlockBox(ref child_box) = container.children[1].layout_box {
        assert!((child_box.border_box.width - 71.4).abs() < 0.1);
    }
}

/// Tests flex_basis with percentage values.
/// Percentages are resolved relative to the main size of the flex container.
/// This test also verifies the interaction with flex_grow.
#[test]
fn test_flex_basis_percentage() {
    let mut container = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Flex,
        },
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(400.0)),
            height: LengthOrAuto::Length(Length::Px(100.0)),
            ..Default::default()
        },
        flex_direction: FlexDirection::Row,
        ..Default::default()
    });

    // Child 1: flex-basis 25%
    let child1 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: LengthOrAuto::Length(Length::Percent(25.0)),
            ..Default::default()
        },
        ..Default::default()
    });

    // Child 2: flex-basis 50%
    let child2 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: LengthOrAuto::Length(Length::Percent(50.0)),
            flex_grow: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    container.children = vec![
        LayoutChild::Node(Box::new(child1)),
        LayoutChild::Node(Box::new(child2)),
    ];
    LayoutEngine::layout(&mut container, 800.0, 600.0);

    // Child1=25% of 400px=100px
    // Child2=50% of 400px=200px + remaining space=100px → 300px
    if let LayoutBox::BlockBox(ref child_box) = container.children[0].layout_box {
        assert_eq!(child_box.border_box.width, 100.0);
    }

    if let LayoutBox::BlockBox(ref child_box) = container.children[1].layout_box {
        assert_eq!(child_box.border_box.width, 300.0);
    }
}

/// Tests a complex realistic flex layout scenario mixing:
/// - auto basis (content-sized)
/// - fixed pixel basis with no flex
/// - percentage basis with flex grow
///
/// Layout rules under CSS Flexbox:
/// Container width: 500px
///
/// Child configuration:
/// 1) width:60px, flex-basis:auto, flex-grow:1
/// 2) flex-basis:120px, flex-grow:0
/// 3) flex-basis:20%, flex-grow:2
///
/// Expected behavior:
/// - auto basis resolves to content width (60px)
/// - percent basis resolves against container (20% of 500 = 100px)
/// - remaining space distributed by grow factors
#[test]
#[ignore = "The implementation of ignoring Width and treating it as basis when flexing is not implemented yet."]
fn test_mixed_flex_properties_complete() {
    let mut container = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Flex,
        },
        size: SizeStyle {
            width: Length::Px(500.0),
            height: Length::Px(100.0),
            ..Default::default()
        },
        flex_direction: FlexDirection::Row,
        ..Default::default()
    });

    // Child 1: Auto basis (content width = 60px), grow 1
    let child1 = LayoutNode::new(Style {
        size: SizeStyle {
            width: Length::Px(60.0),
            ..Default::default()
        },
        item_style: ItemStyle {
            flex_basis: Length::Auto,
            flex_grow: 1.0,
            flex_shrink: 1.0,
            align_self: None,
        },
        ..Default::default()
    });

    // Child 2: Fixed basis, completely inflexible
    let child2 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: Length::Px(120.0),
            flex_grow: 0.0,
            flex_shrink: 0.0,
            align_self: None,
        },
        ..Default::default()
    });

    // Child 3: Percentage basis (20% of container), grow 2
    let child3 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: Length::Percent(20.0), // → 100px
            flex_grow: 2.0,
            flex_shrink: 1.0,
            align_self: None,
        },
        ..Default::default()
    });

    container.children = vec![child1, child2, child3];

    // Perform layout
    LayoutEngine::layout(&mut container, 500.0, 100.0);

    // ---- Expected CSS flex calculation ----
    //
    // Resolved flex bases:
    // - Child1: 60px (auto → content size)
    // - Child2: 120px (fixed)
    // - Child3: 100px (20% of 500px)
    //
    // Total basis = 280px
    // Remaining space = 500 - 280 = 220px
    //
    // Flex grow sum = 1 + 0 + 2 = 3
    //
    // Distribution:
    // - Child1: 60 + 220 * (1/3) ≈ 133.33px
    // - Child2: 120px (grow 0)
    // - Child3: 100 + 220 * (2/3) ≈ 246.67px
    //
    // ---------------------------------------

    // Validate Child1
    if let LayoutBoxes::Single(ref child_box) = container.children[0].layout_boxes {
        let expected = 60.0 + 220.0 * (1.0 / 3.0);
        assert!(
            (child_box.border_box.width - expected).abs() < 0.1,
            "Child1 width incorrect: expected {}, got {}",
            expected,
            child_box.border_box.width
        );
    } else {
        panic!("Child1 layout box missing");
    }

    // Validate Child2 (fixed, no flex)
    if let LayoutBoxes::Single(ref child_box) = container.children[1].layout_boxes {
        assert_eq!(
            child_box.border_box.width, 120.0,
            "Child2 should remain fixed at 120px"
        );
    } else {
        panic!("Child2 layout box missing");
    }

    // Validate Child3
    if let LayoutBoxes::Single(ref child_box) = container.children[2].layout_boxes {
        let expected = 100.0 + 220.0 * (2.0 / 3.0);
        assert!(
            (child_box.border_box.width - expected).abs() < 0.1,
            "Child3 width incorrect: expected {}, got {}",
            expected,
            child_box.border_box.width
        );
    } else {
        panic!("Child3 layout box missing");
    }

    // Additional structural sanity checks

    // Total width should fill container exactly
    let total: f32 = container
        .children
        .iter()
        .map(|c| {
            if let LayoutBoxes::Single(ref b) = c.layout_boxes {
                b.border_box.width
            } else {
                0.0
            }
        })
        .sum();

    assert!(
        (total - 500.0).abs() < 0.1,
        "Total width {} does not match container width 500px",
        total
    );
}

/// Tests flex_basis in column direction (affecting height instead of width).
/// In column flex containers, flex_basis should control the height of items
/// while they fill the container's width.
#[test]
fn test_flex_basis_column_direction() {
    let mut container = LayoutNode::new(Style {
        display: Display {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Flex,
        },
        size: SizeStyle {
            width: Length::Px(200.0),
            height: Length::Px(300.0),
            ..Default::default()
        },
        flex_direction: FlexDirection::Column,
        ..Default::default()
    });

    // Child 1: flex-basis 100px in column (height)
    let child1 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: Length::Px(100.0),
            ..Default::default()
        },
        ..Default::default()
    });

    // Child 2: flex-basis 150px in column (height)
    let child2 = LayoutNode::new(Style {
        item_style: ItemStyle {
            flex_basis: Length::Px(150.0),
            ..Default::default()
        },
        ..Default::default()
    });

    container.children = vec![child1, child2];
    LayoutEngine::layout(&mut container, 800.0, 600.0);

    // In column direction, flex-basis affects height
    if let LayoutBoxes::Single(ref child_box) = container.children[0].layout_boxes {
        assert_eq!(child_box.border_box.height, 100.0);
        assert_eq!(child_box.border_box.width, 200.0); // Should fill container width
    }

    if let LayoutBoxes::Single(ref child_box) = container.children[1].layout_boxes {
        assert_eq!(child_box.border_box.height, 150.0);
        assert_eq!(child_box.border_box.width, 200.0); // Should fill container width
    }
}
