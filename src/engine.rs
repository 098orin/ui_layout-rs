use crate::{
    AlignItems, BoxModel, BoxSizing, FlexDirection, FragmentNode, InlineBox, InnerDisplay,
    ItemFragment, JustifyContent, LayoutBox, LayoutChild, LayoutNode, LengthOrAuto, LineSpan,
    OuterDisplay, Placement, Rect, Spacing, Style,
};

//=====================
// Benchmark
//=====================

#[cfg(feature = "layout-bench")]
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(feature = "layout-bench")]
pub struct LayoutMetrics {
    pub layout_calls: AtomicUsize,
    pub cache_match: AtomicUsize,
    pub cache_miss_match: AtomicUsize,
}

#[cfg(feature = "layout-bench")]
impl LayoutMetrics {
    pub const fn new() -> Self {
        Self {
            layout_calls: AtomicUsize::new(0),
            cache_match: AtomicUsize::new(0),
            cache_miss_match: AtomicUsize::new(0),
        }
    }

    #[inline(always)]
    pub fn reset(&self) {
        self.layout_calls.store(0, Ordering::Relaxed);
        self.cache_match.store(0, Ordering::Relaxed);
        self.cache_miss_match.store(0, Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn count_layout_call(&self) {
        self.layout_calls.fetch_add(1, Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn count_cache_match(&self) {
        self.cache_match.fetch_add(1, Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn count_cache_miss_match(&self) {
        self.cache_miss_match.fetch_add(1, Ordering::Relaxed);
    }

    #[inline(always)]
    pub fn layout_call_count(&self) -> usize {
        self.layout_calls.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn cache_match_count(&self) -> usize {
        self.cache_match.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn cache_miss_match_count(&self) -> usize {
        self.cache_miss_match.load(Ordering::Relaxed)
    }
}

#[cfg(not(feature = "layout-bench"))]
pub struct LayoutMetrics;

#[cfg(not(feature = "layout-bench"))]
impl LayoutMetrics {
    pub const fn new() -> Self {
        LayoutMetrics
    }

    #[inline(always)]
    pub fn reset(&self) {}

    #[inline(always)]
    pub fn count_layout_call(&self) {}

    #[inline(always)]
    pub fn count_cache_match(&self) {}

    #[inline(always)]
    pub fn count_cache_miss_match(&self) {}

    #[inline(always)]
    pub fn layout_call_count(&self) -> usize {
        0
    }

    #[inline(always)]
    pub fn cache_miss_count(&self) -> usize {
        0
    }

    #[inline(always)]
    pub fn cache_miss_match_count(&self) -> usize {
        0
    }
}

impl Default for LayoutMetrics {
    fn default() -> Self {
        Self::new()
    }
}

//=====================
// Main code
//=====================

#[derive(Clone, Copy, Default)]
struct Edge {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

#[derive(Clone, Copy, Default)]
pub struct EdgeOption {
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
}

impl EdgeOption {
    fn unwrap_or_default(self) -> Edge {
        Edge {
            top: self.top.unwrap_or_default(),
            right: self.right.unwrap_or_default(),
            bottom: self.bottom.unwrap_or_default(),
            left: self.left.unwrap_or_default(),
        }
    }
}

pub enum LayoutItem {
    Node(usize),
    Fragments(std::ops::Range<usize>),
}

#[derive(Clone)]
struct FlexItemState {
    frozen_grow: bool,
    frozen_shrink: bool,

    // content-box main size (base or current)
    main_size: f32,

    main_padding: (f32, f32),
    main_border: (f32, f32),
    main_margin: (f32, f32),

    main_min: Option<f32>,
    main_max: Option<f32>,

    grow: f32,
    shrink: f32,
}

impl Default for FlexItemState {
    fn default() -> Self {
        Self {
            // Initially items are not frozen — they can grow/shrink.
            frozen_grow: false,
            frozen_shrink: false,
            main_size: 0.0,
            main_padding: (0.0, 0.0),
            main_border: (0.0, 0.0),
            main_margin: (0.0, 0.0),
            main_min: None,
            main_max: None,
            grow: 0.0,
            shrink: 1.0,
        }
    }
}

/// The difference between containing_block_* and available_* is:
///
/// - containing_block_*:
///   The base size used for resolving percentages and intrinsic sizing.
///   Independent of layout results.
///
/// - available_*:
///   The actual free space available for layout after considering
///   constraints such as sibling layout, margins, and line breaking.
///
/// - `parent_assigned_border_*`:
///   Border-box sizes assigned by parent (for stretch).
pub(crate) struct LayoutContext {
    pub(crate) containing_block_width: Option<f32>,
    pub(crate) containing_block_height: Option<f32>,
    pub(crate) available_width: Option<f32>,
    pub(crate) parent_assigned_border_width: Option<f32>,
    pub(crate) parent_assigned_border_height: Option<f32>,
}

impl LayoutContext {
    /// Returns the containing-block size along the main axis.
    fn containing_block_main(&self, axis: Axis) -> Option<f32> {
        match axis {
            Axis::Horizontal => self.containing_block_width,
            Axis::Vertical => self.containing_block_height,
        }
    }

    /// Returns the containing-block size along the cross axis.
    fn containing_block_cross(&self, axis: Axis) -> Option<f32> {
        match axis {
            Axis::Horizontal => self.containing_block_height,
            Axis::Vertical => self.containing_block_width,
        }
    }
}

/// Axis orientation
///
/// Provides helper methods to abstract width/height selection, reducing code duplication
/// for row and column layout support.
#[derive(Debug, Clone, Copy)]
enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    fn from_flex_direction(value: &FlexDirection) -> Axis {
        match value {
            FlexDirection::Row => Axis::Horizontal,
            FlexDirection::Column => Axis::Vertical,
        }
    }

    fn edge_main(&self, edge: &Edge) -> (f32, f32) {
        match self {
            Self::Horizontal => (edge.left, edge.right),
            Self::Vertical => (edge.top, edge.bottom),
        }
    }

    fn rect_main(&self, rect: &Rect) -> f32 {
        match self {
            Axis::Horizontal => rect.width,
            Axis::Vertical => rect.height,
        }
    }

    fn rect_cross(&self, rect: &Rect) -> f32 {
        match self {
            Axis::Horizontal => rect.height,
            Axis::Vertical => rect.width,
        }
    }

    fn size_main<'a>(&self, size: &'a crate::SizeStyle) -> &'a LengthOrAuto {
        match self {
            Axis::Horizontal => &size.width,
            Axis::Vertical => &size.height,
        }
    }

    fn size_cross<'a>(&self, size: &'a crate::SizeStyle) -> &'a LengthOrAuto {
        match self {
            Axis::Horizontal => &size.height,
            Axis::Vertical => &size.width,
        }
    }

    fn min_main<'a>(&self, size: &'a crate::SizeStyle) -> &'a LengthOrAuto {
        match self {
            Axis::Horizontal => &size.min_width,
            Axis::Vertical => &size.min_height,
        }
    }

    fn max_main<'a>(&self, size: &'a crate::SizeStyle) -> &'a LengthOrAuto {
        match self {
            Axis::Horizontal => &size.max_width,
            Axis::Vertical => &size.max_height,
        }
    }

    fn margin_cross_start<'a>(&self, s: &'a Spacing) -> &'a LengthOrAuto {
        match self {
            Axis::Horizontal => &s.margin_top,
            Axis::Vertical => &s.margin_left,
        }
    }

    fn margin_cross_end<'a>(&self, s: &'a Spacing) -> &'a LengthOrAuto {
        match self {
            Axis::Horizontal => &s.margin_bottom,
            Axis::Vertical => &s.margin_right,
        }
    }

    fn gap<'a>(&self, style: &'a Style) -> &'a LengthOrAuto {
        match self {
            Axis::Horizontal => &style.column_gap,
            Axis::Vertical => &style.row_gap,
        }
    }
}

pub struct LayoutEngine {
    pub(crate) viewport_width: f32,
    pub(crate) viewport_height: f32,
    layout_metrics: LayoutMetrics,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct LineContext {
    /// End position of current line: (x, y)
    pub end_pos: (f32, f32),

    /// (parent_current_x, line_start_x)
    ///
    /// Zero for non-inline contexts.
    pub inline_pos: (f32, f32),

    /// Current line index
    pub line_index: usize,
}

pub(crate) const EMPTY_LINE_CONTEXT: LineContext = LineContext {
    end_pos: (0.0, 0.0),
    inline_pos: (0.0, 0.0),
    line_index: 0,
};

impl LayoutEngine {
    // TODO: implement parent_margin_end
    /// Main layout entry point.
    /// Initiates layout computation from the root node with specified viewport dimensions.
    pub fn layout(root: &mut LayoutNode, width: f32, height: f32) {
        let ctx = LayoutContext {
            containing_block_width: Some(width),
            containing_block_height: Some(height),
            available_width: Some(width),
            parent_assigned_border_width: None,
            parent_assigned_border_height: None,
        };

        let engine = LayoutEngine {
            viewport_width: width,
            viewport_height: height,
            layout_metrics: LayoutMetrics::new(),
        };

        let _ = engine.layout_node(root, &ctx, EMPTY_LINE_CONTEXT, false);

        #[cfg(feature = "layout-bench")]
        println!(
            "layout calls    : {}",
            engine.layout_metrics.layout_call_count()
        );
        #[cfg(feature = "layout-bench")]
        println!(
            "cache match     : {}",
            engine.layout_metrics.cache_match_count()
        );
        #[cfg(feature = "layout-bench")]
        println!(
            "cache miss match: {}",
            engine.layout_metrics.cache_miss_match_count()
        );
    }

    /// Internal method for layout a node.
    /// Layouts a single node and its descendants.
    #[must_use]
    fn layout_node(
        &self,
        node: &mut LayoutNode,
        ctx: &LayoutContext,
        line_ctx: LineContext,
        intrinsic_pass: bool,
    ) -> LineContext {
        if intrinsic_pass {
            let (key, (layout_box, line_ctx)) = &node.layout_box_cache;
            if *key == crate::cache::make_layout_key(ctx, self) {
                self.layout_metrics.count_cache_match();

                node.layout_box = layout_box.clone();
                return *line_ctx;
            } else {
                self.layout_metrics.count_cache_miss_match();
            }
        }

        self.layout_metrics.count_layout_call();

        let out = self.layout_by_display(node, ctx, line_ctx, intrinsic_pass);

        if intrinsic_pass {
            let key = crate::cache::make_layout_key(ctx, self);
            node.layout_box_cache = (key, (node.layout_box.clone(), out));
        }

        out
    }

    fn layout_by_display(
        &self,
        node: &mut LayoutNode,
        ctx: &LayoutContext,
        line_ctx: LineContext,
        intrinsic_pass: bool,
    ) -> LineContext {
        match node.style.display.outer {
            OuterDisplay::None => {
                node.layout_box = LayoutBox::None;
                line_ctx
            }
            OuterDisplay::Block => self.layout_block_level(node, ctx, line_ctx, intrinsic_pass),
            OuterDisplay::Inline => self.layout_inline_level(node, ctx, line_ctx, intrinsic_pass),
        }
    }

    fn layout_block_level(
        &self,
        node: &mut LayoutNode,
        ctx: &LayoutContext,
        line_ctx: LineContext,
        intrinsic_pass: bool,
    ) -> LineContext {
        let ((content_width_opt, content_height_opt), border, padding) = self
            .resolve_base_content_size_and_spacing(
                &node.style.size,
                &node.style.spacing,
                &node.style.box_sizing,
                ctx,
            );

        // --- Intrinsic pass ---
        if intrinsic_pass && content_width_opt.is_some() && content_height_opt.is_some() {
            let box_model = create_box_model(
                content_width_opt.unwrap(),
                content_height_opt.unwrap(),
                0.0,
                0.0,
                padding,
                border,
            );

            node.layout_box = LayoutBox::BlockBox(box_model);

            return LineContext {
                end_pos: (0.0, line_ctx.end_pos.1 + content_height_opt.unwrap()),
                line_index: line_ctx.line_index + 1,
                ..line_ctx
            };
        }

        let content_width_opt = content_width_opt.or(ctx
            .available_width
            .map(|v| v - border.left - border.right - padding.left - padding.right));

        self.layout_by_inner_display(
            node,
            ctx,
            line_ctx,
            (content_width_opt, content_height_opt),
            intrinsic_pass,
        )
    }

    fn layout_inline_level(
        &self,
        node: &mut LayoutNode,
        ctx: &LayoutContext,
        line_ctx: LineContext,
        intrinsic_pass: bool,
    ) -> LineContext {
        let ((content_width_opt, content_height_opt), _, _) = self
            .resolve_base_content_size_and_spacing(
                &node.style.size,
                &node.style.spacing,
                &node.style.box_sizing,
                ctx,
            );

        self.layout_by_inner_display(
            node,
            ctx,
            line_ctx,
            (content_width_opt, content_height_opt),
            intrinsic_pass,
        )
    }

    fn layout_by_inner_display(
        &self,
        node: &mut LayoutNode,
        ctx: &LayoutContext,
        line_ctx: LineContext,
        size_opt: (Option<f32>, Option<f32>),
        intrinsic_pass: bool,
    ) -> LineContext {
        match node.style.display.inner {
            InnerDisplay::Flow => self.layout_flow(node, ctx, line_ctx, size_opt, intrinsic_pass),
            InnerDisplay::Flex => self.layout_flex(node, ctx, line_ctx, size_opt, intrinsic_pass),
        }
    }

    /// TODO:
    /// Optimize for layout contexts.
    /// - Avoid allocate unnecessary.
    ///
    /// Fixes child height calculation.
    /// - Needs to account for the line layout algorithm.
    fn layout_flow(
        &self,
        node: &mut LayoutNode,
        ctx: &LayoutContext,
        line_ctx: LineContext,
        content_size_opt: (Option<f32>, Option<f32>),
        intrinsic_pass: bool,
    ) -> LineContext {
        let (content_width_opt, content_height_opt) = content_size_opt;

        let border = self.resolve_border(&node.style.spacing, ctx);
        let padding = self.resolve_padding(&node.style.spacing, ctx);

        let LineContext {
            end_pos: (end_x, end_y),
            inline_pos: (parent_current_x, mut line_start_x),
            line_index: parent_line_index,
        } = line_ctx;

        let (mut cursor_x, mut cursor_y) = (end_x, end_y);
        let mut current_x = 0.0;
        let mut line_index = 0;

        let mut previous_child_margin = 0.0_f32;
        let (mut children_width, mut children_height) = (0.0_f32, 0.0_f32);

        let base_ctx_for_child = LayoutContext {
            containing_block_width: content_width_opt,
            containing_block_height: content_height_opt,
            available_width: content_width_opt.or(ctx.available_width),
            parent_assigned_border_width: None,
            parent_assigned_border_height: None,
        };

        let line_height = node
            .style
            .line_height
            .resolve_with(None, self.viewport_width, self.viewport_height)
            .unwrap_or_default();

        let mut line_span_buf = Vec::new();
        let mut max_inline_line_height = line_height;

        // -------------------------------
        // 1. Build LayoutItem stream
        // -------------------------------
        let mut items: Vec<LayoutItem> = Vec::new();
        let mut frag_start: Option<usize> = None;

        for (i, child) in node.children.iter().enumerate() {
            match child {
                LayoutChild::Node(_) => {
                    if let Some(start) = frag_start.take() {
                        items.push(LayoutItem::Fragments(start..i));
                    }
                    items.push(LayoutItem::Node(i));
                }
                LayoutChild::Fragment(_) => {
                    if frag_start.is_none() {
                        frag_start = Some(i);
                    }
                }
            }
        }

        if let Some(start) = frag_start.take() {
            items.push(LayoutItem::Fragments(start..node.children.len()));
        }

        // -------------------------------
        // 2. Process LayoutItems
        // -------------------------------
        for item in items {
            match item {
                LayoutItem::Fragments(range) => {
                    let mut fragment_node_buffer = node.children[range.clone()]
                        .iter_mut()
                        .filter_map(|c| match c {
                            LayoutChild::Fragment(f) => Some(f),
                            _ => None,
                        })
                        .collect();

                    let line_ctx_for_child = LineContext {
                        end_pos: (cursor_x, cursor_y),
                        inline_pos: (current_x, line_start_x),
                        line_index,
                    };

                    let (line_spans, updated_line_ctx) = Self::flow_fragments(
                        &mut fragment_node_buffer,
                        line_ctx_for_child,
                        line_height,
                        content_width_opt
                            .or(ctx.available_width)
                            .unwrap_or(self.viewport_width),
                    );

                    let LineContext {
                        end_pos: (cx, cy),
                        inline_pos: (ix, ils),
                        line_index: li,
                    } = updated_line_ctx;

                    cursor_x = cx;
                    cursor_y = cy;
                    current_x = ix;
                    line_start_x = ils;
                    line_index = li;

                    for span in line_spans {
                        push_or_merge_line_span(&mut line_span_buf, span);
                    }

                    max_inline_line_height = max_inline_line_height.max(line_height);
                }

                LayoutItem::Node(i) => {
                    let child_node = match &mut node.children[i] {
                        LayoutChild::Node(n) => n,
                        _ => unreachable!(),
                    };

                    let child_margin = self.resolve_margin(&child_node.style.spacing, ctx);

                    let ctx_for_child = LayoutContext {
                        available_width: content_width_opt.or(ctx.available_width).map(|v| {
                            v - child_margin.left.unwrap_or(0.0) - child_margin.right.unwrap_or(0.0)
                        }),
                        ..base_ctx_for_child
                    };

                    let line_ctx_for_child = LineContext {
                        end_pos: (cursor_x, cursor_y),
                        inline_pos: (current_x, line_start_x),
                        line_index,
                    };

                    let child_is_block = child_node.style.display.outer == OuterDisplay::Block;
                    let layout_line_ctx = if child_is_block {
                        EMPTY_LINE_CONTEXT
                    } else {
                        line_ctx_for_child
                    };

                    let updated_line_ctx = self.layout_node(
                        child_node,
                        &ctx_for_child,
                        layout_line_ctx,
                        intrinsic_pass,
                    );

                    let (child_position_x, child_position_y) = line_ctx_for_child.end_pos;

                    if child_is_block {
                        cursor_x = 0.0;
                        cursor_y = child_position_y + updated_line_ctx.end_pos.1;
                    } else {
                        LineContext {
                            end_pos: (cursor_x, cursor_y),
                            inline_pos: (current_x, line_start_x),
                            line_index,
                        } = updated_line_ctx;
                    }

                    let EdgeOption {
                        left: ml_opt,
                        top,
                        right: mr_opt,
                        bottom,
                    } = child_margin;

                    let (ml, _mr) = if child_is_block {
                        let child_width = child_node.layout_box.width();
                        match (ml_opt, mr_opt, content_width_opt) {
                            (None, None, Some(cw)) => {
                                let auto = (cw - child_width) / 2.0;
                                (auto, auto)
                            }
                            (None, Some(mr), Some(cw)) => {
                                let auto = cw - child_width - mr;
                                (auto, mr)
                            }
                            (Some(ml), None, Some(cw)) => {
                                let auto = cw - child_width - ml;
                                (ml, auto)
                            }
                            _ => (ml_opt.unwrap_or(0.0), mr_opt.unwrap_or(0.0)),
                        }
                    } else {
                        (ml_opt.unwrap_or(0.0), mr_opt.unwrap_or(0.0))
                    };

                    child_node.layout_box.shift(ml, 0.0);

                    if child_is_block {
                        child_node
                            .layout_box
                            .shift(0.0, previous_child_margin.max(top.unwrap_or_default()));
                        cursor_y += previous_child_margin.max(top.unwrap_or_default());
                        previous_child_margin = bottom.unwrap_or_default();
                    }

                    // Process shift
                    if child_node.style.display.outer == OuterDisplay::Inline {
                        child_node.layout_box.shift(child_position_x, 0.0);
                    } else {
                        child_node
                            .layout_box
                            .shift(child_position_x, child_position_y);
                    }

                    // Collect child's line_spans if the outer display is Inline.
                    if node.style.display.outer == OuterDisplay::Inline
                        && child_node.style.display.outer == OuterDisplay::Inline
                        && let LayoutBox::InlineBox(child_inline) = &child_node.layout_box
                    {
                        let x_offset = line_ctx_for_child.inline_pos.0;

                        for child_span in &child_inline.line_spans {
                            push_or_merge_line_span(
                                &mut line_span_buf,
                                LineSpan {
                                    x_range: (child_span.x_range.start + x_offset)
                                        ..(child_span.x_range.end + x_offset),
                                    line_pos: child_span.line_pos,
                                    line_index: child_span.line_index,
                                },
                            );
                        }
                    }

                    if child_node.style.display.outer == OuterDisplay::Inline {
                        for line_box in child_node.layout_box.iter() {
                            max_inline_line_height =
                                max_inline_line_height.max(line_box.border_box.height);
                        }
                    }

                    // Update children bounds after all shifts have been applied.
                    let (child_right, child_bottom) = child_node
                        .layout_box
                        .iter()
                        .map(|box_model| {
                            (box_model.border_box.right(), box_model.border_box.bottom())
                        })
                        .fold((0.0_f32, 0.0_f32), |acc, extent| {
                            (acc.0.max(extent.0), acc.1.max(extent.1))
                        });

                    children_width = children_width.max(child_right);
                    children_height = children_height.max(
                        child_bottom
                            + if child_is_block {
                                previous_child_margin
                            } else {
                                0.0
                            },
                    );
                }
            }
        }

        // -------------------------------
        // 3. Inline final box creation
        // -------------------------------
        if node.style.display.outer == OuterDisplay::Inline {
            let mut box_model = create_box_model(
                current_x,
                max_inline_line_height,
                current_x,
                max_inline_line_height,
                padding,
                border,
            );

            box_model.shift(-(border.left + padding.left), -(border.top + padding.top));

            node.layout_box = LayoutBox::InlineBox(InlineBox {
                box_model,
                line_spans: line_span_buf,
            });
        } else {
            let content_width = content_width_opt.unwrap_or(children_width);
            let content_height = content_height_opt.unwrap_or(children_height);

            let box_model = create_box_model(
                content_width,
                content_height,
                children_width,
                children_height,
                padding,
                border,
            );

            node.layout_box = LayoutBox::BlockBox(box_model);

            // Update cursor.
            cursor_x = 0.0;
            cursor_y = end_y + content_height;
        }

        LineContext {
            end_pos: (cursor_x, cursor_y),
            inline_pos: (
                parent_current_x + current_x,
                parent_current_x + line_start_x,
            ),
            line_index: parent_line_index + line_index,
        }
    }

    fn layout_flex(
        &self,
        node: &mut LayoutNode,
        ctx: &LayoutContext,
        line_ctx: LineContext,
        content_size_opt: (Option<f32>, Option<f32>),
        intrinsic_pass: bool,
    ) -> LineContext {
        let axis = Axis::from_flex_direction(&node.style.flex_direction);

        let (content_width_opt, content_height_opt) = content_size_opt;

        let (children_main, children_cross) =
            if !intrinsic_pass || content_width_opt.is_none() || content_height_opt.is_none() {
                let base_ctx_for_children = LayoutContext {
                    containing_block_width: content_width_opt,
                    containing_block_height: content_height_opt,
                    available_width: None,
                    parent_assigned_border_width: None,
                    parent_assigned_border_height: None,
                };
                self.layout_flex_children(node, axis, intrinsic_pass, &base_ctx_for_children)
            } else {
                (0.0, 0.0)
            };

        let (mut children_width, mut children_height) = match axis {
            Axis::Horizontal => (children_main, children_cross),
            Axis::Vertical => (children_cross, children_main),
        };

        // Fallback to children size
        let width_before_constraints = content_width_opt.unwrap_or(children_width);

        let height_before_constraints = content_height_opt.unwrap_or(children_height);

        // Apply min/max
        let final_width =
            self.apply_size_constraints(width_before_constraints, &node.style.size, ctx, true);

        let final_height =
            self.apply_size_constraints(height_before_constraints, &node.style.size, ctx, false);

        // Detect whether constraints changed size
        let relayout_needed = (Some(final_width) != content_width_opt
            || Some(final_height) != content_height_opt)
            && !intrinsic_pass;

        if relayout_needed {
            // Update context
            let base_ctx_for_children = LayoutContext {
                containing_block_width: Some(final_width),
                containing_block_height: Some(final_height),
                available_width: None,
                parent_assigned_border_width: None,
                parent_assigned_border_height: None,
            };

            let (new_main, new_cross) =
                self.layout_flex_children(node, axis, intrinsic_pass, &base_ctx_for_children);

            (children_width, children_height) = match axis {
                Axis::Horizontal => (new_main, new_cross),
                Axis::Vertical => (new_cross, new_main),
            };
        }

        // Create box model
        node.layout_box = {
            let padding = self.resolve_padding(&node.style.spacing, ctx);
            let border = self.resolve_border(&node.style.spacing, ctx);

            LayoutBox::BlockBox(create_box_model(
                final_width,
                final_height,
                children_width,
                children_height,
                padding,
                border,
            ))
        };

        if !intrinsic_pass {
            self.flow_flex_children(node, axis, ctx);
        }

        LineContext {
            end_pos: (
                line_ctx.end_pos.0 + node.layout_box.width(),
                line_ctx.end_pos.1,
            ),
            inline_pos: (
                line_ctx.inline_pos.0 + node.layout_box.width(),
                line_ctx.inline_pos.1,
            ),
            line_index: line_ctx.line_index,
        }
    }

    /// Layout of Flex child elements
    /// Layouts flex children with flex algorithm.
    fn layout_flex_children(
        &self,
        node: &mut LayoutNode,
        axis: Axis,
        intrinsic_pass: bool,
        base_ctx_for_children: &LayoutContext,
    ) -> (f32, f32) {
        let children_count = node.children.len();
        if children_count == 0 {
            return (0.0, 0.0);
        }

        let cbm = base_ctx_for_children.containing_block_main(axis);
        let cbc = base_ctx_for_children.containing_block_cross(axis);
        let vw = self.viewport_width;
        let vh = self.viewport_height;

        let gap = axis
            .gap(&node.style)
            .resolve_with(cbm, vw, vh)
            .unwrap_or(0.0)
            .max(0.0);

        // --------- Convert to FlexItem ----------
        let flex_items = collect_layout_items(&node.children);

        // ---------- Intrinsic pass ----------
        let item_len = flex_items.len();

        let mut states = vec![FlexItemState::default(); item_len];
        let mut total_grow = 0.0;

        for (item, state) in flex_items.iter().zip(states.iter_mut()) {
            match item {
                LayoutItem::Node(index) => {
                    let ctx = base_ctx_for_children;
                    let node = node.children.get_mut(*index).unwrap().node_mut().unwrap();

                    let padding = self.resolve_padding(&node.style.spacing, ctx);
                    state.main_padding = axis.edge_main(&padding);

                    let border = self.resolve_border(&node.style.spacing, ctx);
                    state.main_border = axis.edge_main(&border);

                    let margin = self
                        .resolve_margin(&node.style.spacing, ctx)
                        .unwrap_or_default();
                    state.main_margin = axis.edge_main(&margin);

                    state.main_min = axis.min_main(&node.style.size).resolve_with(cbm, vw, vh);

                    state.main_max = axis.max_main(&node.style.size).resolve_with(cbm, vw, vh);

                    let basis = node.style.item_style.flex_basis.resolve_with(cbm, vw, vh);

                    let base_content_main = match basis {
                        Some(v) => v,
                        None => {
                            let explicit = axis
                                .size_main(&node.style.size)
                                .resolve_with(cbm, vw, vh)
                                .map(|s| {
                                    resolve_content_size_with_box_sizing(
                                        &node.style.box_sizing,
                                        s,
                                        state.main_padding,
                                        state.main_border,
                                    )
                                });

                            match explicit {
                                None => {
                                    let _ = self.layout_node(
                                        node,
                                        base_ctx_for_children,
                                        EMPTY_LINE_CONTEXT,
                                        true,
                                    );

                                    if let LayoutBox::BlockBox(ref box_model) = node.layout_box {
                                        axis.rect_main(&box_model.content_box)
                                    } else {
                                        0.0
                                    }
                                }
                                Some(v) => v,
                            }
                        }
                    };

                    if !state.frozen_grow {
                        total_grow += node.style.item_style.flex_grow;
                        if node.style.item_style.flex_grow == 0.0 {
                            state.frozen_grow = true;
                        }
                    }

                    if node.style.item_style.flex_shrink == 0.0 {
                        state.frozen_shrink = true;
                    }

                    state.main_size = base_content_main;

                    state.grow = node.style.item_style.flex_grow;
                    state.shrink = node.style.item_style.flex_shrink;
                }
                LayoutItem::Fragments(range) => {
                    let line_height = resolved_fragment_line_height(
                        &node.children,
                        range.clone(),
                        node.style.line_height.resolve_with(None, vw, vh),
                    );
                    let (fragment_width, fragment_height, _) = flow_fragment_range(
                        &mut node.children,
                        range.clone(),
                        EMPTY_LINE_CONTEXT,
                        line_height,
                        base_ctx_for_children.containing_block_width.unwrap_or(vw),
                    );

                    state.main_size = match axis {
                        Axis::Horizontal => fragment_width,
                        Axis::Vertical => fragment_height,
                    };
                }
            }
        }

        let total_base_main: f32 = states.iter().map(|i| i.main_size).sum();

        let total_main_padding: f32 = states
            .iter()
            .map(|i| i.main_padding.0 + i.main_padding.1)
            .sum();

        let total_main_border: f32 = states
            .iter()
            .map(|i| i.main_border.0 + i.main_border.1)
            .sum();

        let total_main_margin: f32 = states
            .iter()
            .map(|i| i.main_margin.0 + i.main_margin.1)
            .sum();

        // number of gaps = items - 1 (if at least 2 items)
        let gaps = gap * item_len.saturating_sub(1) as f32;

        let mut remaining = cbm
            .map(|m| {
                m - (total_base_main
                    + gaps
                    + total_main_padding
                    + total_main_border
                    + total_main_margin)
            })
            .unwrap_or(0.0);

        // ---------- redistribute loop ----------

        loop {
            if remaining > 0.0 {
                if total_grow <= 0.0 {
                    break;
                }

                let mut used = 0.0;

                for i in 0..item_len {
                    if states[i].frozen_grow {
                        continue;
                    }

                    let item = &flex_items[i];
                    let grow = states[i].grow;

                    let delta = remaining * (grow / total_grow);

                    let min = states[i].main_min;
                    let max = states[i].main_max;

                    let old_size = states[i].main_size;

                    let clamped_content = clamp_flex_main_size(
                        old_size,
                        delta,
                        min,
                        max,
                        states[i].main_padding.0
                            + states[i].main_padding.1
                            + states[i].main_border.0
                            + states[i].main_border.1,
                        if let LayoutItem::Node(index) = item {
                            Some(node.children[*index].node().unwrap().style.box_sizing)
                        } else {
                            None
                        },
                    );

                    let actual = clamped_content - old_size;

                    states[i].main_size = clamped_content;
                    used += actual;

                    if (old_size + delta - clamped_content).abs() > 0.001 {
                        states[i].frozen_grow = true;
                        total_grow -= grow;
                    }
                }

                remaining -= used;

                if used.abs() < 0.001 {
                    break;
                }
            } else {
                // negative remaining = overflow
                let mut total_shrink_factor = 0.0;

                for state in &mut states {
                    if state.frozen_shrink {
                        continue;
                    }

                    let shrink = state.shrink;
                    total_shrink_factor += shrink * state.main_size;
                }

                if total_shrink_factor <= 0.0 {
                    break;
                }

                let mut used = 0.0;

                for i in 0..item_len {
                    if states[i].frozen_shrink {
                        continue;
                    }

                    let item = &flex_items[i];

                    let shrink = states[i].shrink;
                    let basis = states[i].main_size;

                    let ratio = (shrink * basis) / total_shrink_factor;

                    let delta = remaining * ratio; // remaining is negative
                    let new_size = states[i].main_size + delta;

                    let min = states[i].main_min;
                    let max = states[i].main_max;

                    let old_size = states[i].main_size;

                    let clamped_content = clamp_flex_main_size(
                        old_size,
                        delta,
                        min,
                        max,
                        states[i].main_padding.0
                            + states[i].main_padding.1
                            + states[i].main_border.0
                            + states[i].main_border.1,
                        if let LayoutItem::Node(index) = item {
                            Some(node.children[*index].node().unwrap().style.box_sizing)
                        } else {
                            None
                        },
                    );

                    let actual = clamped_content - old_size;

                    states[i].main_size = clamped_content;
                    used += actual;

                    if (clamped_content - new_size).abs() > 0.001 {
                        states[i].frozen_shrink = true;
                    }
                }

                remaining -= used;

                if used.abs() < 0.001 {
                    break;
                }
            }
        }

        // ---------- final layout ----------

        let mut total_border_main: f32 = 0.0;
        let mut max_cross: f32 = 0.0;

        for (item, state) in flex_items.iter().zip(states) {
            match item {
                LayoutItem::Node(index) => {
                    let child = node.children.get_mut(*index).unwrap().node_mut().unwrap();

                    let is_auto_margin = axis
                        .margin_cross_start(&child.style.spacing)
                        .resolve_with(cbc, vw, vh)
                        .is_none()
                        || axis
                            .margin_cross_end(&child.style.spacing)
                            .resolve_with(cbc, vw, vh)
                            .is_none();

                    let align = child
                        .style
                        .item_style
                        .align_self
                        .unwrap_or(node.style.align_items);

                    let is_auto_cross = axis.size_cross(&child.style.size) == &LengthOrAuto::Auto;

                    let stretched_cross =
                        if !is_auto_margin && matches!(align, AlignItems::Stretch) && is_auto_cross
                        {
                            cbc.map(|v| {
                                v - axis
                                    .margin_cross_start(&child.style.spacing)
                                    .resolve_with(cbc, vw, vh)
                                    .unwrap_or(0.0)
                                    - axis
                                        .margin_cross_end(&child.style.spacing)
                                        .resolve_with(cbc, vw, vh)
                                        .unwrap_or(0.0)
                            })
                        } else {
                            None
                        };

                    let (parent_assigned_border_width, parent_assigned_border_height) = {
                        let main_bargin_box = state.main_size
                            + state.main_padding.0
                            + state.main_padding.1
                            + state.main_border.0
                            + state.main_border.1;
                        match axis {
                            Axis::Horizontal => (Some(main_bargin_box), stretched_cross),
                            Axis::Vertical => (stretched_cross, Some(main_bargin_box)),
                        }
                    };

                    let ctx_for_child = LayoutContext {
                        parent_assigned_border_width,
                        parent_assigned_border_height,
                        ..*base_ctx_for_children
                    };

                    let _ =
                        self.layout_node(child, &ctx_for_child, EMPTY_LINE_CONTEXT, intrinsic_pass);

                    if let LayoutBox::BlockBox(box_model) = &child.layout_box {
                        total_border_main += axis.rect_main(&box_model.border_box);
                        max_cross = max_cross.max(axis.rect_cross(&box_model.border_box));
                    }
                }
                LayoutItem::Fragments(range) => {
                    let line_height = resolved_fragment_line_height(
                        &node.children,
                        range.clone(),
                        node.style.line_height.resolve_with(None, vw, vh),
                    );
                    let (fragment_width, fragment_height, _) = flow_fragment_range(
                        &mut node.children,
                        range.clone(),
                        EMPTY_LINE_CONTEXT,
                        line_height,
                        base_ctx_for_children.containing_block_width.unwrap_or(vw),
                    );

                    total_border_main += match axis {
                        Axis::Horizontal => fragment_width,
                        Axis::Vertical => fragment_height,
                    };

                    let cross = match axis {
                        Axis::Horizontal => fragment_height,
                        Axis::Vertical => fragment_width,
                    };

                    max_cross = max_cross.max(cross);
                }
            }
        }

        let children_main = total_border_main + gaps;

        (children_main, max_cross)
    }

    /// Set child positions.
    fn flow_flex_children(&self, node: &mut LayoutNode, axis: Axis, ctx: &LayoutContext) {
        if node.children.is_empty() {
            return;
        }

        // Only position children when we have a block box for the parent
        let content_box = match &node.layout_box {
            LayoutBox::BlockBox(box_model) => &box_model.content_box,
            _ => return,
        };

        // Resolve gap between flex items
        let vw = self.viewport_width;
        let vh = self.viewport_height;
        let gap = axis
            .gap(&node.style)
            .resolve_with(ctx.containing_block_main(axis), vw, vh)
            .unwrap_or(0.0)
            .max(0.0);

        let items = collect_layout_items(&node.children);

        // Calculate total size of all flex items along the main axis
        let children_main_total: f32 = items
            .iter()
            .map(|item| match item {
                LayoutItem::Node(index) => {
                    match &node.children[*index].node().unwrap().layout_box {
                        LayoutBox::BlockBox(box_model) => axis.rect_main(&box_model.border_box),
                        _ => 0.0,
                    }
                }
                LayoutItem::Fragments(range) => match axis {
                    Axis::Horizontal => node.children[range.clone()]
                        .iter()
                        .map(|f| f.fragment().unwrap().node.width())
                        .sum(),
                    Axis::Vertical => {
                        let line_height = resolved_fragment_line_height(
                            &node.children,
                            range.clone(),
                            node.style.line_height.resolve_with(None, vw, vh),
                        );
                        let line_count = node.children[range.clone()]
                            .iter()
                            .filter(|f| {
                                f.fragment()
                                    .map(|fragment| fragment.node.is_line_break())
                                    .unwrap_or(false)
                            })
                            .count()
                            + 1;
                        line_height * line_count as f32
                    }
                },
            })
            .sum();

        // Calculate total gap between items
        let gaps_total = if items.len() > 1 {
            gap * (items.len() as f32 - 1.0)
        } else {
            0.0
        };

        // Remaining space for justify-content distribution
        let remaining_space = axis.rect_main(content_box) - children_main_total - gaps_total;

        // Count auto margins on main axis
        let mut auto_margin_count = 0usize;

        for child in &node.children {
            if let crate::LayoutChild::Node(n) = child {
                let spacing = &n.style.spacing;
                match axis {
                    Axis::Horizontal => {
                        if spacing.margin_left == crate::LengthOrAuto::Auto {
                            auto_margin_count += 1;
                        }
                        if spacing.margin_right == crate::LengthOrAuto::Auto {
                            auto_margin_count += 1;
                        }
                    }
                    Axis::Vertical => {
                        if spacing.margin_top == crate::LengthOrAuto::Auto {
                            auto_margin_count += 1;
                        }
                        if spacing.margin_bottom == crate::LengthOrAuto::Auto {
                            auto_margin_count += 1;
                        }
                    }
                }
            }
        }

        let has_auto_margins = auto_margin_count > 0;
        let remaining_space_for_auto = if has_auto_margins {
            remaining_space.max(0.0)
        } else {
            remaining_space
        };

        let auto_unit = if has_auto_margins && auto_margin_count > 0 {
            remaining_space_for_auto / (auto_margin_count as f32)
        } else {
            0.0
        };

        // Auto margins take precedence over justify-content
        let (start_offset, gap_between) = if has_auto_margins {
            (0.0, 0.0)
        } else {
            resolve_justify_content(
                node.style.justify_content,
                remaining_space.max(0.0),
                items.len(),
            )
        };

        // Position each flex child
        let mut cursor_main = start_offset;

        for item in items {
            match item {
                LayoutItem::Node(index) => {
                    let child_node = node.children[index].node_mut().unwrap();

                    // Resolved numeric margins (fall back to 0.0 for auto; we'll handle autos separately)
                    let child_margin = self
                        .resolve_margin(&child_node.style.spacing, ctx)
                        .unwrap_or_default();
                    let (margin_start_resolved, margin_end_resolved) =
                        axis.edge_main(&child_margin);

                    // Detect auto margins on main axis
                    let (margin_start_auto, margin_end_auto) = match axis {
                        Axis::Horizontal => (
                            child_node.style.spacing.margin_left == crate::LengthOrAuto::Auto,
                            child_node.style.spacing.margin_right == crate::LengthOrAuto::Auto,
                        ),
                        Axis::Vertical => (
                            child_node.style.spacing.margin_top == crate::LengthOrAuto::Auto,
                            child_node.style.spacing.margin_bottom == crate::LengthOrAuto::Auto,
                        ),
                    };

                    // Compute auto margin widths
                    let margin_start = if margin_start_auto {
                        auto_unit
                    } else {
                        margin_start_resolved
                    };
                    let margin_end = if margin_end_auto {
                        auto_unit
                    } else {
                        margin_end_resolved
                    };

                    cursor_main += margin_start;

                    // Position child along main axis
                    let child_main_pos = match axis {
                        Axis::Horizontal => content_box.x + cursor_main,
                        Axis::Vertical => content_box.y + cursor_main,
                    };

                    // Position child along cross axis (align-items / align-self)
                    let child_cross_size = match &child_node.layout_box {
                        LayoutBox::BlockBox(box_model) => axis.rect_cross(&box_model.border_box),
                        _ => 0.0,
                    };
                    let available_cross = axis.rect_cross(content_box);

                    // --- Cross-axis auto margin handling ---
                    let margin_cross_start_auto = match axis {
                        Axis::Horizontal => {
                            child_node.style.spacing.margin_top == crate::LengthOrAuto::Auto
                        }
                        Axis::Vertical => {
                            child_node.style.spacing.margin_left == crate::LengthOrAuto::Auto
                        }
                    };
                    let margin_cross_end_auto = match axis {
                        Axis::Horizontal => {
                            child_node.style.spacing.margin_bottom == crate::LengthOrAuto::Auto
                        }
                        Axis::Vertical => {
                            child_node.style.spacing.margin_right == crate::LengthOrAuto::Auto
                        }
                    };

                    let cross_offset = if margin_cross_start_auto || margin_cross_end_auto {
                        let free_cross_space = (available_cross - child_cross_size).max(0.0);

                        if margin_cross_start_auto && margin_cross_end_auto {
                            free_cross_space / 2.0
                        } else if margin_cross_start_auto {
                            free_cross_space
                        } else {
                            0.0
                        }
                    } else {
                        // fallback to align-self / align-items
                        resolve_align_position(
                            child_node
                                .style
                                .item_style
                                .align_self
                                .unwrap_or(node.style.align_items),
                            child_cross_size,
                            available_cross,
                        )
                    };

                    let child_cross_pos = match axis {
                        Axis::Horizontal => content_box.y + cross_offset,
                        Axis::Vertical => content_box.x + cross_offset,
                    };

                    // Compute final child position based on axis orientation
                    let child_origin = match axis {
                        Axis::Horizontal => (child_main_pos, child_cross_pos),
                        Axis::Vertical => (child_cross_pos, child_main_pos),
                    };

                    // Shift child to final position relative to parent content box
                    let relative_x = child_origin.0 - content_box.x;
                    let relative_y = child_origin.1 - content_box.y;
                    child_node.layout_box.shift(relative_x, relative_y);

                    // Move cursor forward for next child
                    let child_main_size = match &child_node.layout_box {
                        LayoutBox::BlockBox(box_model) => axis.rect_main(&box_model.border_box),
                        _ => 0.0,
                    };

                    cursor_main += child_main_size + margin_end + gap + gap_between;
                }

                LayoutItem::Fragments(range) => {
                    let line_height = resolved_fragment_line_height(
                        &node.children,
                        range.clone(),
                        node.style.line_height.resolve_with(None, vw, vh),
                    );
                    let fragment_width: f32 = node.children[range.clone()]
                        .iter()
                        .map(|f| f.fragment().unwrap().node.width())
                        .sum();
                    let line_count = node.children[range.clone()]
                        .iter()
                        .filter(|f| {
                            f.fragment()
                                .map(|fragment| fragment.node.is_line_break())
                                .unwrap_or(false)
                        })
                        .count()
                        + 1;
                    let fragment_height = line_height * line_count as f32;

                    let item_main_size = match axis {
                        Axis::Horizontal => fragment_width,
                        Axis::Vertical => fragment_height,
                    };

                    let item_cross_size = match axis {
                        Axis::Horizontal => fragment_height,
                        Axis::Vertical => fragment_width,
                    };

                    let child_main_pos = match axis {
                        Axis::Horizontal => content_box.x + cursor_main,
                        Axis::Vertical => content_box.y + cursor_main,
                    };

                    let available_cross = axis.rect_cross(content_box);
                    let cross_offset = resolve_align_position(
                        node.style.align_items,
                        item_cross_size,
                        available_cross,
                    );

                    let child_cross_pos = match axis {
                        Axis::Horizontal => content_box.y + cross_offset,
                        Axis::Vertical => content_box.x + cross_offset,
                    };

                    let line_ctx = match axis {
                        Axis::Horizontal => LineContext {
                            end_pos: (child_main_pos, child_cross_pos),
                            inline_pos: (child_main_pos, child_main_pos),
                            line_index: 0,
                        },
                        Axis::Vertical => LineContext {
                            end_pos: (child_cross_pos, child_main_pos),
                            inline_pos: (child_cross_pos, child_cross_pos),
                            line_index: 0,
                        },
                    };

                    let outbox_width = match axis {
                        Axis::Horizontal => child_main_pos + item_main_size,
                        Axis::Vertical => child_cross_pos + item_cross_size,
                    };

                    let _ = flow_fragment_range(
                        &mut node.children,
                        range,
                        line_ctx,
                        line_height,
                        outbox_width,
                    );

                    cursor_main += item_main_size + gap + gap_between;
                }
            }
        }
    }

    fn flow_fragments(
        fragments: &mut Vec<&mut FragmentNode>,
        line_ctx: LineContext,
        line_height: f32,
        outbox_width: f32,
    ) -> (Vec<LineSpan>, LineContext) {
        let mut cursor_x = line_ctx.end_pos.0;
        let mut cursor_y = line_ctx.end_pos.1;

        let mut current_x = line_ctx.inline_pos.0;
        let mut line_start_x = line_ctx.inline_pos.1;
        let mut visual_line_start_x = cursor_x - (current_x - line_start_x);

        let mut line_index = line_ctx.line_index;

        let mut if_first_of_line = current_x == line_start_x;

        let mut line_span_buf = Vec::new();

        for fragment_node in fragments {
            match fragment_node.node {
                ItemFragment::LineBreak => {
                    push_or_merge_line_span(
                        &mut line_span_buf,
                        LineSpan {
                            x_range: line_start_x..current_x,
                            line_pos: (visual_line_start_x, cursor_y),
                            line_index,
                        },
                    );

                    fragment_node.placement = Placement {
                        offset: (cursor_x, cursor_y),
                        line_index,
                    };

                    cursor_x = 0.0;
                    cursor_y += line_height;
                    line_index += 1;
                    line_start_x = current_x;
                    visual_line_start_x = 0.0;
                    if_first_of_line = true;
                }

                ItemFragment::Fragment(fragment_item) => {
                    if cursor_x + fragment_item.width > outbox_width && !if_first_of_line {
                        push_or_merge_line_span(
                            &mut line_span_buf,
                            LineSpan {
                                x_range: line_start_x..current_x,
                                line_pos: (visual_line_start_x, cursor_y),
                                line_index,
                            },
                        );

                        fragment_node.placement = Placement {
                            offset: (cursor_x, cursor_y),
                            line_index,
                        };

                        cursor_x = 0.0;
                        cursor_y += line_height;
                        line_index += 1;
                        line_start_x = current_x;
                        visual_line_start_x = 0.0;
                    }

                    fragment_node.placement = Placement {
                        offset: (cursor_x, cursor_y),
                        line_index,
                    };

                    cursor_x += fragment_item.width;
                    current_x += fragment_item.width;
                    if_first_of_line = false;
                }
            }
        }

        if !if_first_of_line {
            push_or_merge_line_span(
                &mut line_span_buf,
                LineSpan {
                    x_range: line_start_x..current_x,
                    line_pos: (visual_line_start_x, cursor_y),
                    line_index,
                },
            );
        }

        (
            line_span_buf,
            LineContext {
                end_pos: (cursor_x, cursor_y),
                inline_pos: (current_x, line_start_x),
                line_index,
            },
        )
    }

    /// ((content_width_opt, content_height_opt), border, padding)
    ///
    /// TODO:
    /// - Handle min/max with box-sizing correctly.
    fn resolve_base_content_size_and_spacing(
        &self,
        size_style: &crate::SizeStyle,
        spacing: &crate::Spacing,
        box_sizing: &BoxSizing,
        ctx: &LayoutContext,
    ) -> ((Option<f32>, Option<f32>), Edge, Edge) {
        let border = self.resolve_border(spacing, ctx);
        let padding = self.resolve_padding(spacing, ctx);

        let vw = self.viewport_width;
        let vh = self.viewport_height;

        // --- width ---
        // Parent-assigned (flex) size takes priority over explicit size
        let content_width = ctx
            .parent_assigned_border_width
            .map(|v| v - (padding.left + padding.right) - (border.left + border.right))
            .or(size_style
                .width
                .resolve_with(ctx.containing_block_width, vw, vh)
                .map(|width| {
                    let padding_edge = (padding.left, padding.right);
                    let border_edge = (border.left, border.right);
                    resolve_content_size_with_box_sizing(
                        box_sizing,
                        width,
                        padding_edge,
                        border_edge,
                    )
                }))
            .map(|width| self.apply_size_constraints(width, size_style, ctx, true));

        // --- height ---
        // Parent-assigned (flex) size takes priority over explicit size
        let content_height = ctx
            .parent_assigned_border_height
            .map(|v| v - (padding.top + padding.bottom) - (border.top + border.bottom))
            .or(size_style
                .height
                .resolve_with(ctx.containing_block_height, vw, vh)
                .map(|height| {
                    let padding_edge = (padding.top, padding.bottom);
                    let border_edge = (border.top, border.bottom);
                    resolve_content_size_with_box_sizing(
                        box_sizing,
                        height,
                        padding_edge,
                        border_edge,
                    )
                }))
            .map(|height| self.apply_size_constraints(height, size_style, ctx, false));

        ((content_width, content_height), border, padding)
    }

    /// Applies min/max size constraints to a dimension value.
    fn apply_size_constraints(
        &self,
        value: f32,
        size_style: &crate::SizeStyle,
        ctx: &LayoutContext,
        is_width: bool,
    ) -> f32 {
        let vw = self.viewport_width;
        let vh = self.viewport_height;

        let (min_constraint, max_constraint) = if is_width {
            (
                size_style
                    .min_width
                    .resolve_with(ctx.containing_block_width, vw, vh),
                size_style
                    .max_width
                    .resolve_with(ctx.containing_block_width, vw, vh),
            )
        } else {
            (
                size_style
                    .min_height
                    .resolve_with(ctx.containing_block_height, vw, vh),
                size_style
                    .max_height
                    .resolve_with(ctx.containing_block_height, vw, vh),
            )
        };

        clamp(value, min_constraint, max_constraint)
    }

    fn resolve_padding(&self, spacing: &Spacing, ctx: &LayoutContext) -> Edge {
        let containing_width = ctx.containing_block_width;
        let vw = self.viewport_width;
        let vh = self.viewport_height;

        Edge {
            left: spacing
                .padding_left
                .resolve_with(containing_width, vw, vh)
                .unwrap_or(0.0),
            top: spacing
                .padding_top
                .resolve_with(containing_width, vw, vh)
                .unwrap_or(0.0),
            right: spacing
                .padding_right
                .resolve_with(containing_width, vw, vh)
                .unwrap_or(0.0),
            bottom: spacing
                .padding_bottom
                .resolve_with(containing_width, vw, vh)
                .unwrap_or(0.0),
        }
    }

    fn resolve_border(&self, spacing: &Spacing, ctx: &LayoutContext) -> Edge {
        let containing_width = ctx.containing_block_width;
        let vw = self.viewport_width;
        let vh = self.viewport_height;

        Edge {
            left: spacing
                .border_left
                .resolve_with(containing_width, vw, vh)
                .unwrap_or(0.0),
            top: spacing
                .border_top
                .resolve_with(containing_width, vw, vh)
                .unwrap_or(0.0),
            right: spacing
                .border_right
                .resolve_with(containing_width, vw, vh)
                .unwrap_or(0.0),
            bottom: spacing
                .border_bottom
                .resolve_with(containing_width, vw, vh)
                .unwrap_or(0.0),
        }
    }

    fn resolve_margin(&self, spacing: &Spacing, ctx: &LayoutContext) -> EdgeOption {
        let containing_width = ctx.containing_block_width.unwrap_or(self.viewport_width);
        let vw = self.viewport_width;
        let vh = self.viewport_height;

        EdgeOption {
            left: spacing
                .margin_left
                .resolve_with(ctx.containing_block_width, vw, vh),
            top: spacing
                .margin_top
                .resolve_with(Some(containing_width), vw, vh),
            right: spacing
                .margin_right
                .resolve_with(ctx.containing_block_width, vw, vh),
            bottom: spacing
                .margin_bottom
                .resolve_with(Some(containing_width), vw, vh),
        }
    }
}

// ==========================================

/// Resolves content size based on the box-sizing property.
///
/// # Arguments
/// * `box_sizing` - Box-sizing style
/// * `size` - The border or content box size to resolve
/// * `padding_edge` - (padding start, padding end)
/// * `border_edge` - (border start, border end)
fn resolve_content_size_with_box_sizing(
    box_sizing: &BoxSizing,
    size: f32,
    padding_edge: (f32, f32),
    border_edge: (f32, f32),
) -> f32 {
    match box_sizing {
        BoxSizing::ContentBox => size,
        BoxSizing::BorderBox => {
            size - padding_edge.0 - padding_edge.1 - border_edge.0 - border_edge.1
        }
    }
    .max(0.0)
}

/// Clamps a value between optional minimum and maximum bounds.
fn clamp(value: f32, min: Option<f32>, max: Option<f32>) -> f32 {
    let v = min.map_or(value, |m| value.max(m));
    max.map_or(v, |m| v.min(m))
}

/// Clamps a flex item's content size, accounting for box-sizing.
fn clamp_flex_main_size(
    main_size: f32,
    delta: f32,
    min: Option<f32>,
    max: Option<f32>,
    padding_border_main: f32,
    box_sizing: Option<BoxSizing>,
) -> f32 {
    let proposed_content = main_size + delta;
    match box_sizing {
        Some(BoxSizing::ContentBox) => clamp(proposed_content, min, max),
        Some(BoxSizing::BorderBox) => {
            let proposed_border = proposed_content + padding_border_main;
            let clamped_border = clamp(proposed_border, min, max);
            (clamped_border - padding_border_main).max(0.0)
        }
        None => proposed_content,
    }
}

fn collect_layout_items(children: &[LayoutChild]) -> Vec<LayoutItem> {
    let mut items = Vec::new();
    let mut iter = children.iter().enumerate().peekable();

    while let Some((i, child)) = iter.next() {
        match child {
            LayoutChild::Node(_) => items.push(LayoutItem::Node(i)),
            LayoutChild::Fragment(_) => {
                let start = i;
                let mut end = i + 1;

                while let Some((next_i, LayoutChild::Fragment(_))) = iter.peek() {
                    end = *next_i + 1;
                    iter.next();
                }

                items.push(LayoutItem::Fragments(start..end));
            }
        }
    }

    items
}

fn resolved_fragment_line_height(
    children: &[LayoutChild],
    range: std::ops::Range<usize>,
    line_height: Option<f32>,
) -> f32 {
    line_height.unwrap_or_else(|| {
        children[range]
            .iter()
            .filter_map(|child| child.fragment())
            .map(|fragment| fragment.node.height())
            .fold(0.0_f32, f32::max)
    })
}

fn flow_fragment_range(
    children: &mut [LayoutChild],
    range: std::ops::Range<usize>,
    line_ctx: LineContext,
    line_height: f32,
    outbox_width: f32,
) -> (f32, f32, LineContext) {
    let mut fragment_node_buffer = children[range]
        .iter_mut()
        .filter_map(|child| child.fragment_mut())
        .collect();

    let (line_spans, line_ctx) = LayoutEngine::flow_fragments(
        &mut fragment_node_buffer,
        line_ctx,
        line_height,
        outbox_width,
    );

    let width = line_spans
        .iter()
        .map(|span| span.width())
        .filter(|width| !width.is_nan())
        .max_by(f32::total_cmp)
        .unwrap_or(0.0);

    let height = if line_spans.is_empty() {
        0.0
    } else {
        line_height * line_spans.len() as f32
    };

    (width, height, line_ctx)
}

fn push_or_merge_line_span(spans: &mut Vec<LineSpan>, span: LineSpan) {
    if span.x_range.end <= span.x_range.start {
        return;
    }

    if let Some(last) = spans.last_mut()
        && last.line_index == span.line_index
        && last.line_pos.1 == span.line_pos.1
    {
        last.x_range.end = last.x_range.end.max(span.x_range.end);
        last.x_range.start = last.x_range.start.min(span.x_range.start);
        last.line_pos.0 = last.line_pos.0.min(span.line_pos.0);
        return;
    }

    spans.push(span);
}

/// Creates a box model with specified dimensions and spacing.
fn create_box_model(
    content_width: f32,
    content_height: f32,
    children_width: f32,
    children_height: f32,
    padding_edge: Edge,
    border_edge: Edge,
) -> BoxModel {
    let Edge {
        left: pl,
        top: pt,
        right: pr,
        bottom: pb,
    } = padding_edge;
    let Edge {
        left: bl,
        top: bt,
        right: br,
        bottom: bb,
    } = border_edge;

    let border_box = Rect {
        x: 0.0,
        y: 0.0,
        width: content_width + pl + pr + bl + br,
        height: content_height + pt + pb + bt + bb,
    };

    let padding_box = Rect {
        x: bl,
        y: bt,
        width: content_width + pl + pr,
        height: content_height + pt + pb,
    };

    let content_box = Rect {
        x: bl + pl,
        y: bt + pt,
        width: content_width,
        height: content_height,
    };

    let children_box = Rect {
        width: children_width,
        height: children_height,
        ..content_box
    };

    BoxModel {
        content_box,
        padding_box,
        border_box,
        children_box,
    }
}

fn resolve_justify_content(
    justify: JustifyContent,
    remaining_space: f32,
    items: usize,
) -> (f32, f32) {
    match justify {
        JustifyContent::Start => (0.0, 0.0),
        JustifyContent::Center => (remaining_space / 2.0, 0.0),
        JustifyContent::End => (remaining_space, 0.0),
        JustifyContent::SpaceBetween => {
            if items > 1 {
                (0.0, remaining_space / (items - 1) as f32)
            } else {
                (0.0, 0.0)
            }
        }
        JustifyContent::SpaceAround => {
            if items > 0 {
                let gap = remaining_space / items as f32;
                (gap / 2.0, gap)
            } else {
                (0.0, 0.0)
            }
        }
        JustifyContent::SpaceEvenly => {
            if items > 0 {
                let gap = remaining_space / (items + 1) as f32;
                (gap, gap)
            } else {
                (0.0, 0.0)
            }
        }
    }
}

fn resolve_align_position(align: AlignItems, child_size: f32, available: f32) -> f32 {
    match align {
        AlignItems::Start => 0.0,
        AlignItems::Center => ((available - child_size) / 2.0).max(0.0),
        AlignItems::End => (available - child_size).max(0.0),
        AlignItems::Stretch => 0.0,
    }
}
