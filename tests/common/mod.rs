#![allow(dead_code)]
use ui_layout::*;

pub fn node<'a>(n: &'a LayoutNode, idx: usize) -> &'a LayoutNode {
    n.children[idx].node().expect("expected node child")
}

pub fn block_box(n: &LayoutNode) -> &BoxModel {
    match &n.layout_box {
        LayoutBox::BlockBox(b) => b,
        _ => panic!("expected block box"),
    }
}

pub fn inline_box_model(n: &LayoutNode) -> &BoxModel {
    match &n.layout_box {
        LayoutBox::InlineBox(inline) => &inline.box_model,
        _ => panic!("expected inline box"),
    }
}

pub fn fragment(width: f32, height: f32) -> ItemFragment {
    ItemFragment::Fragment(Fragment { width, height })
}

pub fn new_child(height: f32) -> LayoutNode {
    LayoutNode::new(Style {
        size: SizeStyle {
            height: LengthOrAuto::Length(Length::Px(height)),
            ..Default::default()
        },
        ..Default::default()
    })
}

pub fn flex_container(width: f32, height: f32, direction: FlexDirection) -> Style {
    Style {
        display: Display {
            outer: OuterDisplay::Block,
            inner: InnerDisplay::Flex,
        },
        size: SizeStyle {
            width: LengthOrAuto::Length(Length::Px(width)),
            height: LengthOrAuto::Length(Length::Px(height)),
            ..Default::default()
        },
        flex_direction: direction,
        ..Default::default()
    }
}

pub fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}

pub fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.01
}

pub fn mock_inline_box() -> LayoutBox {
    LayoutBox::InlineBox(InlineBox {
        box_model: BoxModel {
            border_box: rect(0.0, 0.0, 114.0, 20.0),
            padding_box: rect(2.0, 0.0, 110.0, 20.0),
            content_box: rect(6.0, 0.0, 100.0, 20.0),
            children_box: rect(6.0, 0.0, 100.0, 20.0),
        },
        line_spans: vec![
            LineSpan {
                x_range: 0.0..40.0,
                line_pos: (6.0, 0.0),
                line_index: 0,
            },
            LineSpan {
                x_range: 40.0..70.0,
                line_pos: (0.0, 20.0),
                line_index: 1,
            },
            LineSpan {
                x_range: 70.0..100.0,
                line_pos: (0.0, 40.0),
                line_index: 2,
            },
        ],
    })
}
