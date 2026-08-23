use std::fmt;

use crate::*;

// ============================================================
//  Display implementations for primitive style types
// ============================================================

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Length::Px(v) => write!(f, "{}px", v),
            Length::Percent(v) => write!(f, "{}%", v),
            Length::Vw(v) => write!(f, "{}vw", v),
            Length::Vh(v) => write!(f, "{}vh", v),
            Length::Add(a, b) => write!(f, "calc({} + {})", a, b),
            Length::Sub(a, b) => write!(f, "calc({} - {})", a, b),
            Length::Mul(a, n) => write!(f, "calc({} * {})", a, n),
            Length::Div(a, n) => write!(f, "calc({} / {})", a, n),
            Length::Min(a, b) => write!(f, "min({}, {})", a, b),
            Length::Max(a, b) => write!(f, "max({}, {})", a, b),
            Length::Clamp { min, val, max } => write!(f, "clamp({}, {}, {})", min, val, max),
        }
    }
}

impl fmt::Display for LengthOrAuto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LengthOrAuto::Length(l) => write!(f, "{}", l),
            LengthOrAuto::Auto => write!(f, "auto"),
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Position::Static => write!(f, "static"),
            Position::Relative => write!(f, "relative"),
            Position::Absolute => write!(f, "absolute"),
            Position::Fixed => write!(f, "fixed"),
            Position::Sticky => write!(f, "sticky"),
        }
    }
}

impl fmt::Display for OuterDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OuterDisplay::Block => write!(f, "block"),
            OuterDisplay::Inline => write!(f, "inline"),
            OuterDisplay::None => write!(f, "none"),
        }
    }
}

impl fmt::Display for InnerDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InnerDisplay::Flow => write!(f, "flow"),
            InnerDisplay::FlowRoot => write!(f, "flow-root"),
            InnerDisplay::Flex => write!(f, "flex"),
            InnerDisplay::Grid => write!(f, "grid"),
        }
    }
}

impl fmt::Display for Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.outer, self.inner) {
            (OuterDisplay::Block, InnerDisplay::Flow) => write!(f, "block"),
            (OuterDisplay::Inline, InnerDisplay::Flow) => write!(f, "inline"),
            (OuterDisplay::None, InnerDisplay::Flow) => write!(f, "none"),
            (OuterDisplay::Block, InnerDisplay::FlowRoot) => write!(f, "flow-root"),
            (OuterDisplay::Inline, InnerDisplay::FlowRoot) => write!(f, "inline-block"),
            (OuterDisplay::Block, InnerDisplay::Flex) => write!(f, "flex"),
            (OuterDisplay::Inline, InnerDisplay::Flex) => write!(f, "inline-flex"),
            (OuterDisplay::Block, InnerDisplay::Grid) => write!(f, "grid"),
            (OuterDisplay::Inline, InnerDisplay::Grid) => write!(f, "inline-grid"),
            (outer, inner) => write!(f, "{} {}", outer, inner),
        }
    }
}

impl fmt::Display for FlexDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlexDirection::Row => write!(f, "row"),
            FlexDirection::Column => write!(f, "column"),
            FlexDirection::RowReverse => write!(f, "row-reverse"),
            FlexDirection::ColumnReverse => write!(f, "column-reverse"),
        }
    }
}

impl fmt::Display for FlexWrap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlexWrap::NoWrap => write!(f, "nowrap"),
            FlexWrap::Wrap => write!(f, "wrap"),
            FlexWrap::WrapReverse => write!(f, "wrap-reverse"),
        }
    }
}

impl fmt::Display for JustifyItems {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JustifyItems::Start => write!(f, "start"),
            JustifyItems::Center => write!(f, "center"),
            JustifyItems::End => write!(f, "end"),
            JustifyItems::Stretch => write!(f, "stretch"),
        }
    }
}

impl fmt::Display for JustifyContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JustifyContent::Start => write!(f, "start"),
            JustifyContent::Center => write!(f, "center"),
            JustifyContent::End => write!(f, "end"),
            JustifyContent::SpaceBetween => write!(f, "space-between"),
            JustifyContent::SpaceAround => write!(f, "space-around"),
            JustifyContent::SpaceEvenly => write!(f, "space-evenly"),
        }
    }
}

impl fmt::Display for AlignItems {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlignItems::Start => write!(f, "start"),
            AlignItems::Center => write!(f, "center"),
            AlignItems::End => write!(f, "end"),
            AlignItems::Stretch => write!(f, "stretch"),
        }
    }
}

impl fmt::Display for AlignContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AlignContent::Start => write!(f, "start"),
            AlignContent::Center => write!(f, "center"),
            AlignContent::End => write!(f, "end"),
            AlignContent::SpaceBetween => write!(f, "space-between"),
            AlignContent::SpaceAround => write!(f, "space-around"),
            AlignContent::SpaceEvenly => write!(f, "space-evenly"),
            AlignContent::Stretch => write!(f, "stretch"),
        }
    }
}

impl fmt::Display for BoxSizing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoxSizing::ContentBox => write!(f, "content-box"),
            BoxSizing::BorderBox => write!(f, "border-box"),
        }
    }
}

impl fmt::Display for GridRepeat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GridRepeat::Count(v) => write!(f, "{}", v),
            GridRepeat::AutoFit => write!(f, "auto-fit"),
            GridRepeat::AutoFill => write!(f, "auto-fill"),
        }
    }
}

impl fmt::Display for GridTrack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GridTrack::Breadth(v) => write!(f, "{}", v),
            GridTrack::Flex(v) => write!(f, "{}fr", v),
            GridTrack::MinMax(a, b) => write!(f, "minmax({}, {})", a, b),
            GridTrack::Repeat(a, b) => write!(
                f,
                "repeat({}, {})",
                a,
                b.iter()
                    .map(|v| v.to_string())
                    .collect::<Vec<String>>()
                    .join(" ")
            ),
        }
    }
}

impl fmt::Display for Placement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "({}, {}) @line {}",
            self.offset.0, self.offset.1, self.line_index
        )
    }
}

impl fmt::Display for LayoutBox {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LayoutBox::None => write!(f, "none"),
            LayoutBox::BlockBox(b) => {
                let w = self.width_box();
                let h = self.height_box();
                write!(
                    f,
                    "block({}x{} @{},{})",
                    w, h, b.border_box.x, b.border_box.y
                )
            }
            LayoutBox::InlineBox(inline) => {
                let w = self.width_box();
                let h = self.height_box();
                let line_pos_str = inline
                    .line_spans
                    .iter()
                    .map(|s| format!("({},{})", s.line_pos.0, s.line_pos.1))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(
                    f,
                    "inline({}x{} @{},{} [{}])",
                    w,
                    h,
                    inline.box_model.border_box.x,
                    inline.box_model.border_box.y,
                    line_pos_str
                )
            }
        }
    }
}

// ============================================================
//  Macros for collecting non-default style entries
// ============================================================

/// Pushes `"name: value"` if `field != Type::default()`.
macro_rules! entry_if {
    ($e:expr, $field:expr, $name:expr) => {{
        if $field != Default::default() {
            $e.push(format!("{}: {}", $name, $field));
        }
    }};
    ($e:expr, $field:expr, $name:expr, $default:expr) => {{
        if $field != $default {
            $e.push(format!("{}: {}", $name, $field));
        }
    }};
}

macro_rules! entry_vec {
    ($e:expr, $field:expr, $name:expr) => {{
        if !$field.is_empty() {
            let mut entry = String::new();
            use std::fmt::Write;

            write!(entry, "{}: ", $name).unwrap();

            for (i, item) in $field.iter().enumerate() {
                if i != 0 {
                    entry.push(' ');
                }
                write!(entry, "{item}").unwrap();
            }

            $e.push(entry);
        }
    }};
}

/// Pushes `"name: val"` if the Option field is `Some`.
macro_rules! entry_some {
    ($e:expr, $field:expr, $name:expr) => {{
        if let Some(ref val) = $field {
            $e.push(format!("{}: {}", $name, val));
        }
    }};
}

/// Emits shorthand (`margin: ...`) when all four sides are equal,
/// or individual side entries otherwise.
macro_rules! spacing_group {
    ($e:expr, $s:expr, margin) => {{
        collect_spacing_group(
            &mut $e,
            "margin",
            &[
                (&$s.spacing.margin_top, "margin-top"),
                (&$s.spacing.margin_bottom, "margin-bottom"),
                (&$s.spacing.margin_left, "margin-left"),
                (&$s.spacing.margin_right, "margin-right"),
            ],
            &LengthOrAuto::Length(Length::Px(0.0)),
        );
    }};
    ($e:expr, $s:expr, border) => {{
        collect_spacing_group(
            &mut $e,
            "border",
            &[
                (&$s.spacing.border_top, "border-top"),
                (&$s.spacing.border_bottom, "border-bottom"),
                (&$s.spacing.border_left, "border-left"),
                (&$s.spacing.border_right, "border-right"),
            ],
            &Length::Px(0.0),
        );
    }};
    ($e:expr, $s:expr, padding) => {{
        collect_spacing_group(
            &mut $e,
            "padding",
            &[
                (&$s.spacing.padding_top, "padding-top"),
                (&$s.spacing.padding_bottom, "padding-bottom"),
                (&$s.spacing.padding_left, "padding-left"),
                (&$s.spacing.padding_right, "padding-right"),
            ],
            &Length::Px(0.0),
        );
    }};
}

// ============================================================
//  Collect non-default style entries
// ============================================================

fn collect_style_entries(style: &Style) -> Vec<String> {
    let mut entries: Vec<String> = Vec::new();

    entry_if!(entries, style.display, "display");
    entry_if!(entries, style.position.kind, "position");
    entry_if!(entries, style.position.top, "top");
    entry_if!(entries, style.position.right, "right");
    entry_if!(entries, style.position.bottom, "bottom");
    entry_if!(entries, style.position.left, "left");
    entry_if!(entries, style.item_style.flex_grow, "flex-grow");
    entry_if!(entries, style.item_style.flex_shrink, "flex-shrink", 1.0);
    entry_if!(entries, style.item_style.flex_basis, "flex-basis");
    entry_some!(entries, style.item_style.justify_self, "justify-self");
    entry_some!(entries, style.item_style.align_self, "align-self");

    entry_if!(entries, style.size.width, "width");
    entry_if!(entries, style.size.height, "height");
    entry_if!(entries, style.size.min_width, "min-width");
    entry_if!(entries, style.size.max_width, "max-width");
    entry_if!(entries, style.size.min_height, "min-height");
    entry_if!(entries, style.size.max_height, "max-height");

    entry_if!(entries, style.box_sizing, "box-sizing");

    spacing_group!(entries, style, margin);
    spacing_group!(entries, style, border);
    spacing_group!(entries, style, padding);

    entry_if!(entries, style.line_height, "line-height");
    entry_if!(entries, style.justify_content, "justify-content");
    entry_if!(entries, style.align_content, "align-content");
    entry_if!(entries, style.flex_direction, "flex-direction");
    entry_if!(entries, style.flex_wrap, "flex-wrap");
    entry_if!(entries, style.column_gap, "column-gap");
    entry_if!(entries, style.row_gap, "row-gap");

    entry_vec!(
        entries,
        style.grid_template_columns,
        "grid-template-columns"
    );
    entry_vec!(entries, style.grid_template_rows, "grid-template-rows");

    entries
}

/// Collects entries for a spacing group (margin, border, padding).
///
/// Uses CSS shorthand notation when possible:
/// - All same:       `margin: 10px`
/// - TB / LR pair:   `margin: 10px 20px`
/// - T / LR / B:     `margin: 10px 20px 30px`
/// - All different:   `margin: 10px 20px 30px 40px`  (top right bottom left)
///
/// When only a single side is set, emits the individual side entry
/// (e.g. `margin-top: 10px`) since that is shorter than the shorthand.
fn collect_spacing_group<T: PartialEq + fmt::Display>(
    entries: &mut Vec<String>,
    group_name: &str,
    sides: &[(&T, &str); 4],
    default: &T,
) {
    let top = sides[0].0;
    let bottom = sides[1].0;
    let left = sides[2].0;
    let right = sides[3].0;

    let non_default_count = [
        *top != *default,
        *bottom != *default,
        *left != *default,
        *right != *default,
    ]
    .iter()
    .filter(|&&b| b)
    .count();

    if non_default_count == 0 {
        return;
    }

    if non_default_count == 1 {
        for (value, name) in sides {
            if **value != *default {
                entries.push(format!("{}: {}", name, value));
            }
        }
        return;
    }

    if *top == *bottom && *left == *right {
        if *top == *left {
            // All same: margin: 10px
            entries.push(format!("{}: {}", group_name, top));
        } else {
            // TB / LR: margin: 10px 20px
            entries.push(format!("{}: {} {}", group_name, top, left));
        }
    } else if *left == *right {
        // T / LR / B: margin: 10px 20px 30px
        entries.push(format!("{}: {} {} {}", group_name, top, left, bottom));
    } else {
        // All different: top right bottom left
        entries.push(format!(
            "{}: {} {} {} {}",
            group_name, top, right, bottom, left
        ));
    }
}

// ============================================================
//  Display for Style
// ============================================================

impl fmt::Display for Style {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries = collect_style_entries(self);
        if entries.is_empty() {
            write!(f, "(default)")
        } else {
            write!(f, "{}", entries.join(", "))
        }
    }
}

// ============================================================
//  Display for LayoutNode (tree rendering)
// ============================================================

impl fmt::Display for LayoutNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Root node ── no prefix, no connector
        write!(f, "LayoutNode")?;
        let entries = collect_style_entries(&self.style);
        if !entries.is_empty() {
            write!(f, " [{}]", entries.join(", "))?;
        }
        if f.alternate() {
            write!(f, " {}", self.layout_box)?;
        }
        writeln!(f)?;

        let prefix = "";
        for (i, child) in self.children.iter().enumerate() {
            let last = i == self.children.len() - 1;
            write_child(f, child, prefix, last)?;
        }
        Ok(())
    }
}

/// Recursively writes a non-root `LayoutNode` with tree-drawing characters.
///
/// `prefix` is the indentation string accumulated from ancestors.
/// `is_last` indicates whether this node is the last child of its parent.
fn write_node(
    f: &mut fmt::Formatter<'_>,
    node: &LayoutNode,
    prefix: &str,
    is_last: bool,
) -> fmt::Result {
    let connector = if is_last { "└── " } else { "├── " };
    write!(f, "{}{}LayoutNode", prefix, connector)?;

    let entries = collect_style_entries(&node.style);
    if !entries.is_empty() {
        write!(f, " [{}]", entries.join(", "))?;
    }
    if f.alternate() {
        write!(f, " {}", node.layout_box)?;
    }
    writeln!(f)?;

    let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

    for (i, child) in node.children.iter().enumerate() {
        let last = i == node.children.len() - 1;
        write_child(f, child, &child_prefix, last)?;
    }

    Ok(())
}

fn write_child(
    f: &mut fmt::Formatter<'_>,
    child: &LayoutChild,
    prefix: &str,
    is_last: bool,
) -> fmt::Result {
    match child {
        LayoutChild::Node(n) => write_node(f, n, prefix, is_last),
        LayoutChild::Fragment(frag) => write_fragment(f, frag, prefix, is_last),
        LayoutChild::Custom(child) => {
            let branch = if is_last { "└── " } else { "├── " };

            write!(f, "{}{}", prefix, branch)?;

            child.layouter().write_debug(f)?;

            if f.alternate()
                && let Some(result) = child.result()
            {
                let b = &result.box_model.border_box;
                write!(f, " [{}x{} @({}, {})]", b.width, b.height, b.x, b.y)?;
            }
            writeln!(f)
        }
    }
}

fn write_fragment(
    f: &mut fmt::Formatter<'_>,
    frag: &FragmentNode,
    prefix: &str,
    is_last: bool,
) -> fmt::Result {
    let connector = if is_last { "└── " } else { "├── " };
    write!(f, "{}{}", prefix, connector)?;

    match frag.node {
        ItemFragment::Fragment(c) => write!(f, "Fragment [{}x{}]", c.width, c.height)?,
        ItemFragment::LineBreak => write!(f, "LineBreak")?,
    }
    if f.alternate() {
        write!(f, " {}", frag.placement)?;
    }
    writeln!(f)
}
