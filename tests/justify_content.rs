use ui_layout::*;

#[test]
fn test_justify_content_start() {
    let root = setup_flex_container(&[50.0, 50.0], JustifyContent::Start, 300.0);

    let c1 = &root.children[0].rect;
    let c2 = &root.children[1].rect;

    assert_eq!(c1.x, 0.0, "Start: Child1 left aligned");
    assert_eq!(c2.x, 50.0, "Start: Child2 next to Child1");
}

#[test]
fn test_justify_content_center() {
    let root = setup_flex_container(&[50.0, 50.0], JustifyContent::Center, 300.0);

    let total_width = 50.0 + 50.0;
    let start_x = (300.0 - total_width) / 2.0;

    let c1 = &root.children[0].rect;
    let c2 = &root.children[1].rect;

    assert!((c1.x - start_x).abs() < 0.01, "Center: Child1 centered");
    assert!(
        (c2.x - (start_x + 50.0)).abs() < 0.01,
        "Center: Child2 next to Child1"
    );
}

#[test]
fn test_justify_content_end() {
    let root = setup_flex_container(&[50.0, 50.0], JustifyContent::End, 300.0);

    let start_x = 300.0 - (50.0 + 50.0);

    let c1 = &root.children[0].rect;
    let c2 = &root.children[1].rect;

    assert!((c1.x - start_x).abs() < 0.01, "End: Child1 right aligned");
    assert!(
        (c2.x - (start_x + 50.0)).abs() < 0.01,
        "End: Child2 next to Child1"
    );
}

#[test]
fn test_justify_content_space_between() {
    let root = setup_flex_container(&[50.0, 50.0], JustifyContent::SpaceBetween, 300.0);

    let space = (300.0 - (50.0 + 50.0)) / 1.0; // n-1 gaps
    let c1 = &root.children[0].rect;
    let c2 = &root.children[1].rect;

    assert!(
        (c1.x - 0.0).abs() < 0.01,
        "SpaceBetween: Child1 left aligned"
    );
    assert!(
        (c2.x - (50.0 + space)).abs() < 0.01,
        "SpaceBetween: Child2 spaced to right"
    );
}

#[test]
fn test_justify_content_space_around() {
    let root = setup_flex_container(&[50.0, 50.0], JustifyContent::SpaceAround, 300.0);

    let space = (300.0 - (50.0 + 50.0)) / 2.0; // n gaps = 2 * n
    let c1 = &root.children[0].rect;
    let c2 = &root.children[1].rect;

    assert!(
        (c1.x - space / 2.0).abs() < 0.01,
        "SpaceAround: Child1 start offset"
    );
    assert!(
        (c2.x - (space / 2.0 + 50.0 + space)).abs() < 0.01,
        "SpaceAround: Child2 spaced correctly"
    );
}

fn setup_flex_container(
    child_widths: &[f32],
    justify: JustifyContent,
    container_width: f32,
) -> LayoutNode {
    let mut root = LayoutNode::new(Style {
        display: Display::Flex {
            flex_direction: FlexDirection::Row,
        },
        size: SizeStyle {
            width: Length::Px(container_width),
            height: Length::Px(100.0),
            ..Default::default()
        },
        justify_content: justify,
        ..Default::default()
    });

    for &w in child_widths {
        let child = LayoutNode::new(Style {
            size: SizeStyle {
                width: Length::Px(w),
                height: Length::Px(50.0),
                ..Default::default()
            },
            ..Default::default()
        });
        root.children.push(child);
    }

    LayoutEngine::layout(&mut root, container_width, 100.0);

    root
}
