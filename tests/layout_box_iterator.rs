use ui_layout::*;

fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}

fn inline_layout_box() -> LayoutBox {
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

#[test]
fn borrowed_layout_box_iter_tracks_remaining_len() {
    let layout_box = inline_layout_box();
    let mut iter = layout_box.iter();

    assert_eq!(iter.len(), 3);
    assert_eq!(iter.size_hint(), (3, Some(3)));

    let first = iter.next().unwrap();
    assert_eq!(first.content_box.width, 40.0);
    assert_eq!(first.border_box.x, 6.0);
    assert_eq!(first.content_box.x, 6.0);
    assert_eq!(first.border_box.width, 46.0);
    assert_eq!(iter.len(), 2);
    assert_eq!(iter.size_hint(), (2, Some(2)));

    let last = iter.next_back().unwrap();
    assert_eq!(last.content_box.width, 30.0);
    assert_eq!(last.content_box.x, 0.0);
    assert_eq!(last.border_box.width, 38.0);
    assert_eq!(iter.len(), 1);

    let middle = iter.next().unwrap();
    assert_eq!(middle.content_box.width, 30.0);
    assert_eq!(middle.content_box.x, 0.0);
    assert_eq!(middle.border_box.width, 30.0);
    assert_eq!(iter.len(), 0);
    assert!(iter.next().is_none());
    assert!(iter.next_back().is_none());
}

#[test]
fn owned_layout_box_into_iter_yields_line_boxes_lazily() {
    let mut iter = inline_layout_box().into_iter();

    assert_eq!(iter.len(), 3);

    let first = iter.next().unwrap();
    assert_eq!(first.border_box.y, 0.0);
    assert_eq!(first.content_box.y, 0.0);
    assert_eq!(first.content_box.width, 40.0);

    let second = iter.next().unwrap();
    assert_eq!(second.border_box.y, 20.0);
    assert_eq!(second.content_box.y, 0.0);
    assert_eq!(second.content_box.width, 30.0);
    assert_eq!(second.border_box.width, 30.0);

    let third = iter.next().unwrap();
    assert_eq!(third.border_box.y, 40.0);
    assert_eq!(third.content_box.y, 0.0);
    assert_eq!(third.content_box.width, 30.0);
    assert_eq!(third.border_box.width, 38.0);

    assert_eq!(iter.len(), 0);
    assert!(iter.next().is_none());
}

#[test]
fn none_and_block_iterators_report_len_and_end_correctly() {
    let none = LayoutBox::None;
    let mut none_iter = none.iter();
    assert_eq!(none_iter.len(), 0);
    assert!(none_iter.next().is_none());
    assert!(none_iter.next().is_none());

    let block = LayoutBox::BlockBox(BoxModel {
        border_box: rect(1.0, 2.0, 30.0, 40.0),
        padding_box: rect(1.0, 2.0, 30.0, 40.0),
        content_box: rect(1.0, 2.0, 30.0, 40.0),
        children_box: rect(1.0, 2.0, 30.0, 40.0),
    });
    let mut block_iter = block.iter();
    assert_eq!(block_iter.len(), 1);

    let b = block_iter.next_back().unwrap();
    assert_eq!(b.content_box, rect(1.0, 2.0, 30.0, 40.0));
    assert_eq!(block_iter.len(), 0);
    assert!(block_iter.next().is_none());
}
