use std::time::Instant;
use ui_layout::*;

/// display だけ指定したノードを作る
fn node(display: Display) -> LayoutNode {
    LayoutNode::new(Style {
        display,
        spacing: Spacing {
            margin_left: Length::Px(2.0),
            margin_right: Length::Px(2.0),
            padding_left: Length::Px(1.0),
            padding_right: Length::Px(1.0),
            border_left: Length::Px(1.0),
            border_right: Length::Px(1.0),
            ..Default::default()
        },
        size: SizeStyle {
            min_width: Length::Px(0.0),
            max_width: Length::Px(10_000.0),
            ..Default::default()
        },
        ..Default::default()
    })
}

/// Flex の深い連鎖を作る
fn make_flex_chain(depth: usize, max_depth: usize) -> LayoutNode {
    let mut root = node(Display { outer: OuterDisplay::Block, inner: InnerDisplay::Flex } {
        flex_direction: FlexDirection::Row,
    });

    if depth >= max_depth {
        root.children.push(node(Display::Block));
        return root;
    }

    // 子1: Block（文脈は切れない）
    let mut block = node(Display::Block);

    // Block の中にさらに Flex
    block.children.push(make_flex_chain(depth + 1, max_depth));

    root.children.push(block);
    root
}

/// 実サイトっぽい「枝」を追加
fn make_branch(depth: usize, max_depth: usize) -> LayoutNode {
    let mut flex = node(Display { outer: OuterDisplay::Block, inner: InnerDisplay::Flex } {
        flex_direction: FlexDirection::Column,
    });

    // 子は少ない（1〜2）
    flex.children.push(make_flex_chain(depth, max_depth));

    if depth % 3 == 0 {
        let mut side = node(Display::Block);
        side.children.push(node(Display::Block));
        flex.children.push(side);
    }

    flex
}

fn make_tree() -> LayoutNode {
    let mut root = node(Display::Block);

    // 上位は Block
    root.children.push(node(Display::Block));

    // 問題の塊
    root.children.push(make_branch(0, 20));

    root
}

fn main() {
    println!("flex-chain layout benchmark");
    println!("----------------------------");

    let mut root = make_tree();

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
