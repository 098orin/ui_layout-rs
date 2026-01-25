use std::time::Instant;
use ui_layout::*;

fn make_heavy_child() -> LayoutNode {
    LayoutNode::new(Style {
        display: Display::Block,
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
    let mut node = make_heavy_child();

    if depth >= max_depth || *remaining == 0 {
        return node;
    }

    // 深くなるほど子を減らす（爆発防止）
    let max_children = if depth < 4 {
        3
    } else if depth < 8 {
        2
    } else {
        1
    };

    let children_count = max_children.min(*remaining);

    let mut children = Vec::with_capacity(children_count);
    for _ in 0..children_count {
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
    const TOTAL_NODES: usize = 4_000;
    const DEPTH: usize = 1_0;

    println!("layout cache benchmark");
    println!("TOTAL_NODES = {}", TOTAL_NODES);
    println!("DEPTH       = {}", DEPTH);
    println!("-------------------------");

    let mut remaining = TOTAL_NODES - 1; // root 分を引く
    let mut root = make_tree(0, DEPTH, &mut remaining);

    println!("remaining nodes unused = {}", remaining);

    let t1 = {
        let start = Instant::now();
        LayoutEngine::layout(&mut root, 800.0, 600.0);
        start.elapsed()
    };

    let t2 = {
        let start = Instant::now();
        LayoutEngine::layout(&mut root, 800.0, 600.0);
        start.elapsed()
    };

    println!("1st: {:?}", t1);
    println!("2nd: {:?}", t2);
}
