use std::time::Instant;
use ui_layout::*;

fn make_node(is_flex: bool) -> LayoutNode {
    LayoutNode::new(Style {
        display: if is_flex {
            Display { outer: OuterDisplay::Block, inner: InnerDisplay::Flex } {
                flex_direction: FlexDirection::Row,
            }
        } else {
            Display::Block
        },
        size: SizeStyle {
            width: Length::Auto,
            height: Length::Auto,
            min_width: Length::Px(0.0),
            max_width: Length::Px(10_000.0),
            min_height: Length::Px(0.0),
            max_height: Length::Px(10_000.0),
        },
        spacing: Spacing {
            margin_left: Length::Px(2.0),
            margin_right: Length::Px(2.0),
            margin_top: Length::Px(1.0),
            margin_bottom: Length::Px(1.0),
            padding_left: Length::Px(1.0),
            padding_right: Length::Px(1.0),
            padding_top: Length::Px(1.0),
            padding_bottom: Length::Px(1.0),
            border_left: Length::Px(1.0),
            border_right: Length::Px(1.0),
            border_top: Length::Px(1.0),
            border_bottom: Length::Px(1.0),
            ..Default::default()
        },
        ..Default::default()
    })
}

fn make_tree(depth: usize, max_depth: usize, remaining: &mut usize) -> LayoutNode {
    let is_flex = depth % 2 == 0; // Flex -> Block -> Flex -> ...
    let mut node = make_node(is_flex);

    if depth >= max_depth || *remaining == 0 {
        return node;
    }

    let max_children = if depth < 3 {
        4
    } else if depth < 6 {
        3
    } else if depth < 9 {
        2
    } else {
        1
    };

    let child_count = max_children.min(*remaining);

    let mut children = Vec::with_capacity(child_count);
    for _ in 0..child_count {
        if *remaining == 0 {
            break;
        }
        *remaining -= 1;
        children.push(make_tree(depth + 1, max_depth, remaining));
    }

    node.children = children;
    node
}

fn main() {
    const TOTAL_NODES: usize = 7_000;
    const DEPTH: usize = 16;

    println!("layout performance benchmark");
    println!("TOTAL_NODES = {}", TOTAL_NODES);
    println!("DEPTH       = {}", DEPTH);
    println!("-------------------------");

    let mut remaining = TOTAL_NODES - 1;
    let mut root = make_tree(0, DEPTH, &mut remaining);

    println!("remaining nodes unused = {}", remaining);

    let t1 = {
        let start = Instant::now();
        LayoutEngine::layout(&mut root, 800.0, 600.0);
        start.elapsed()
    };

    println!("1st layout: {:?}", t1);

    let t2 = {
        let start = Instant::now();
        LayoutEngine::layout(&mut root, 800.0, 600.0);
        start.elapsed()
    };

    println!("2nd layout: {:?}", t2);
}
