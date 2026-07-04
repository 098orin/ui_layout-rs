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
            InnerDisplay::Flex => write!(f, "flex"),
        }
    }
}

impl fmt::Display for Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.outer, self.inner) {
            (OuterDisplay::Block, InnerDisplay::Flow) => write!(f, "block"),
            (OuterDisplay::Inline, InnerDisplay::Flow) => write!(f, "inline"),
            (OuterDisplay::None, InnerDisplay::Flow) => write!(f, "none"),
            (OuterDisplay::Block, InnerDisplay::Flex) => write!(f, "flex"),
            (OuterDisplay::Inline, InnerDisplay::Flex) => write!(f, "inline-flex"),
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

impl fmt::Display for BoxSizing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoxSizing::ContentBox => write!(f, "content-box"),
            BoxSizing::BorderBox => write!(f, "border-box"),
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
                write!(
                    f,
                    "inline({}x{} @{},{})",
                    w, h, inline.box_model.border_box.x, inline.box_model.border_box.y
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
    entry_if!(entries, style.item_style.flex_grow, "flex-grow");
    entry_if!(entries, style.item_style.flex_shrink, "flex-shrink", 1.0);
    entry_if!(entries, style.item_style.flex_basis, "flex-basis");
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
    entry_if!(entries, style.align_items, "align-items");
    entry_if!(entries, style.flex_direction, "flex-direction");
    entry_if!(entries, style.column_gap, "column-gap");
    entry_if!(entries, style.row_gap, "row-gap");

    entries
}

/// Collects entries for a spacing group (margin, border, padding).
///
/// If all four sides share the same non-default value, emits a single
/// shorthand entry (e.g. `margin: 10px`). Otherwise emits individual
/// side entries for those that differ from the default.
fn collect_spacing_group<T: PartialEq + fmt::Display>(
    entries: &mut Vec<String>,
    group_name: &str,
    sides: &[(&T, &str); 4],
    default: &T,
) {
    let all_same = sides[0].0 == sides[1].0 && sides[0].0 == sides[2].0 && sides[0].0 == sides[3].0;

    if all_same {
        if *sides[0].0 != *default {
            entries.push(format!("{}: {}", group_name, sides[0].0));
        }
    } else {
        for (value, name) in sides {
            if **value != *default {
                entries.push(format!("{}: {}", name, value));
            }
        }
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
        LayoutChild::Object(o) => {
            let branch = if is_last { "└── " } else { "├── " };

            write!(f, "{}{}", prefix, branch)?;

            o.write_debug(f)?;
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
