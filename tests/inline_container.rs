use ui_layout::*;

#[test]
fn item_fragment_api_and_fragmentnode() {
    let frag = ItemFragment::Fragment(Fragment {
        width: 50.0,
        height: 20.0,
    });
    assert_eq!(frag.width(), 50.0);
    assert_eq!(frag.height(), 20.0);
    assert!(!frag.is_line_break());

    let lb = ItemFragment::LineBreak;
    assert_eq!(lb.width(), 0.0);
    assert_eq!(lb.height(), 0.0);
    assert!(lb.is_line_break());

    let fragment_node = FragmentNode {
        node: frag,
        placement: Placement {
            offset: (0.0, 0.0),
            line_index: 0,
        },
    };

    assert_eq!(fragment_node.node.width(), 50.0);
}
