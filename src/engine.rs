use crate::{
    AlignContent, AlignItems, AutoSizeBehavior, BoxModel, BoxSizing, CustomObjectResult, Edge,
    FlexDirection, FlexWrap, FragmentNode, GridPlacement, GridRepeat, GridTrack, InlineBox,
    InnerDisplay, ItemFragment, JustifyContent, LayoutBox, LayoutChild, LayoutNode, LengthOrAuto,
    LineSpan, OuterDisplay, Placement, Position, Rect, Spacing, Style,
};

const EPSILON: f32 = 0.001;
const MAX_GRID_TRACKS: usize = 10_000;

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

//=====================
// Main code
//=====================

#[derive(Clone, Copy, Default)]
struct EdgeOption {
    left: Option<f32>,
    top: Option<f32>,
    right: Option<f32>,
    bottom: Option<f32>,
}

impl EdgeOption {
    fn unwrap_or_default(self) -> Edge {
        Edge {
            left: self.left.unwrap_or_default(),
            top: self.top.unwrap_or_default(),
            right: self.right.unwrap_or_default(),
            bottom: self.bottom.unwrap_or_default(),
        }
    }
}

#[derive(Clone)]
pub enum LayoutItem {
    Node(usize),
    Fragments(std::ops::Range<usize>),
    Custom(usize),
}

struct GridItemSlot {
    item: LayoutItem,
    column: usize,
    row: usize,
    column_span: usize,
    row_span: usize,
    intrinsic_width: f32,
    intrinsic_height: f32,
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

struct FlowCursor {
    x: f32,
    y: f32,
    current_x: f32,
    line_index: usize,
}

impl FlowCursor {
    fn pos(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    fn line_ctx(&self) -> LineContext {
        LineContext {
            end_pos: self.pos(),
            current_x: self.current_x,
            margin_start: 0.0,
            margin_end: 0.0,
        }
    }

    fn update_from(&mut self, ctx: &LineContext) {
        self.x = ctx.end_pos.0;
        self.y = ctx.end_pos.1;
        self.current_x = ctx.current_x;
    }

    fn set_pos(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    fn shift_y(&mut self, dy: f32) {
        self.y += dy;
    }
}

struct FlowAccum {
    prev_child_margin: f32,
    first_child_margin: f32,
    pending_advance: f32,
    children_width: f32,
    children_height: f32,
    max_inline_line_height: f32,
    first_block_child_processed: bool,
    line_span_buf: Vec<LineSpan>,
}

struct FlowState {
    cursor: FlowCursor,
    accum: FlowAccum,
    padding: Edge,
    border: Edge,
    start_x: f32,
    end_y: f32,
    parent_current_x: f32,
    content_width_opt: Option<f32>,
    intrinsic_pass: bool,
    /// Whether sibling and parent-child margins should collapse.
    ///
    /// - `true`:  `InnerDisplay::Flow` — vertical margins collapse (CSS normal flow)
    /// - `false`: `InnerDisplay::FlowRoot` — margins are additive (new BFC)
    ///
    /// See: <https://www.w3.org/TR/CSS2/box.html#collapsing-margins>
    collapse_margins: bool,
}

struct FlexPlacementCtx {
    content_box: Rect,
    line_cross_offset: f32,
    cursor_main: f32,
    auto_unit: f32,
    gap_between: f32,
    gap: f32,
    reversed: bool,
}

/// Engine-internal layout context threaded through every layout pass.
///
/// Unlike the public [`crate::LayoutContext`] handed to custom objects, this
/// carries the full bookkeeping the engine needs: the containing block, the
/// actual free space and parent-assigned sizes for flex, and the inline-flow
/// line info. It is private to the crate and never exposed to custom layout
/// objects directly.
#[derive(Debug, Clone, Default)]
pub(crate) struct InternalLayoutContext {
    /// Containing block width for resolving percentages and intrinsic sizing.
    /// Independent of layout results.
    pub(crate) containing_block_width: Option<f32>,

    /// Containing block height for resolving percentages and intrinsic sizing.
    /// Independent of layout results.
    pub(crate) containing_block_height: Option<f32>,

    /// The actual free space available for layout after considering
    /// constraints such as sibling layout, margins, and line breaking.
    pub(crate) available_width: Option<f32>,

    /// Border-box width assigned by the parent (e.g. via flex stretch).
    pub(crate) parent_assigned_border_width: Option<f32>,

    /// Border-box height assigned by the parent (e.g. via flex stretch).
    pub(crate) parent_assigned_border_height: Option<f32>,

    /// Start position of the current line in parent coordinates.
    ///
    /// Only meaningful while laying out an inline-level object in an
    /// inline flow context; zero otherwise.
    pub(crate) start_pos: (f32, f32),

    /// Remaining inline size available on the current line before wrapping.
    ///
    /// Only meaningful while laying out an inline-level object in an
    /// inline flow context; zero otherwise.
    pub(crate) available_inline_size: f32,

    /// Line height of the containing inline formatting context.
    ///
    /// When an inline-level object's [`crate::LineSpan`]s occupy multiple
    /// lines, this value is used as the vertical advance between them.
    ///
    /// Only meaningful while laying out an inline-level object in an
    /// inline flow context; zero otherwise.
    pub(crate) line_height: f32,

    /// Viewport width, used for resolving `Vw` units.
    pub(crate) viewport_width: f32,

    /// Viewport height, used for resolving `Vh` units.
    pub(crate) viewport_height: f32,
}

impl InternalLayoutContext {
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

impl From<&InternalLayoutContext> for crate::LayoutContext {
    fn from(ctx: &InternalLayoutContext) -> Self {
        Self {
            containing_block_width: ctx.containing_block_width,
            containing_block_height: ctx.containing_block_height,
            start_pos: ctx.start_pos,
            available_inline_size: ctx.available_inline_size,
            line_height: ctx.line_height,
            viewport_width: ctx.viewport_width,
            viewport_height: ctx.viewport_height,
        }
    }
}

// Provides helper methods to abstract width/height selection, reducing code duplication
// for row and column layout support.

/// Axis orientation
#[derive(Debug, Clone, Copy)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    fn from_flex_direction(value: &FlexDirection) -> Axis {
        match value {
            FlexDirection::Row | FlexDirection::RowReverse => Axis::Horizontal,
            FlexDirection::Column | FlexDirection::ColumnReverse => Axis::Vertical,
        }
    }

    fn is_reversed(value: &FlexDirection) -> bool {
        matches!(
            value,
            FlexDirection::RowReverse | FlexDirection::ColumnReverse
        )
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

    fn tuple_main(&self, (width, height): (f32, f32)) -> f32 {
        match self {
            Axis::Horizontal => width,
            Axis::Vertical => height,
        }
    }

    fn tuple_cross(&self, (width, height): (f32, f32)) -> f32 {
        match self {
            Axis::Horizontal => height,
            Axis::Vertical => width,
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

    fn cross_gap<'a>(&self, style: &'a Style) -> &'a LengthOrAuto {
        match self {
            Axis::Horizontal => &style.row_gap,
            Axis::Vertical => &style.column_gap,
        }
    }
}

pub struct LayoutEngine {
    pub(crate) viewport_width: f32,
    pub(crate) viewport_height: f32,
    #[cfg(feature = "layout-bench")]
    layout_metrics: LayoutMetrics,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct LineContext {
    /// End position of current line: (x, y)
    pub end_pos: (f32, f32),

    /// Current x offset in the flat inline coordinate space.
    pub current_x: f32,

    /// Collapsed top margin of the first child for parent-child margin collapsing.
    pub margin_start: f32,
    /// Collapsed bottom margin of the last child for parent-child margin collapsing.
    pub margin_end: f32,
}

pub(crate) const EMPTY_LINE_CONTEXT: LineContext = LineContext {
    end_pos: (0.0, 0.0),
    current_x: 0.0,
    margin_start: 0.0,
    margin_end: 0.0,
};

impl LayoutEngine {
    /// Main layout entry point.
    /// Initiates layout computation from the root node with specified viewport dimensions.
    pub fn layout(root: &mut LayoutNode, width: f32, height: f32) {
        let ctx = InternalLayoutContext {
            containing_block_width: Some(width),
            containing_block_height: Some(height),
            available_width: Some(width),
            parent_assigned_border_width: None,
            parent_assigned_border_height: None,
            viewport_width: width,
            viewport_height: height,
            ..Default::default()
        };

        let engine = LayoutEngine {
            viewport_width: width,
            viewport_height: height,
            #[cfg(feature = "layout-bench")]
            layout_metrics: LayoutMetrics::new(),
        };

        let _ = engine.layout_node(root, &ctx, EMPTY_LINE_CONTEXT, false);
        engine.apply_positioning(
            root,
            (0.0, 0.0),
            Rect {
                x: 0.0,
                y: 0.0,
                width,
                height,
            },
        );

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
        ctx: &InternalLayoutContext,
        line_ctx: LineContext,
        intrinsic_pass: bool,
    ) -> LineContext {
        if intrinsic_pass {
            let (key, (layout_box, line_ctx)) = &node.layout_box_cache;
            if *key == crate::cache::make_layout_key(ctx, self) {
                #[cfg(feature = "layout-bench")]
                self.layout_metrics.count_cache_match();

                node.layout_box = layout_box.clone();
                return *line_ctx;
            } else {
                #[cfg(feature = "layout-bench")]
                self.layout_metrics.count_cache_miss_match();
            }
        }

        #[cfg(feature = "layout-bench")]
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
        ctx: &InternalLayoutContext,
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
        ctx: &InternalLayoutContext,
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
        if intrinsic_pass && let (Some(cw), Some(ch)) = (content_width_opt, content_height_opt) {
            let box_model = create_box_model(cw, ch, 0.0, 0.0, padding, border);

            node.layout_box = LayoutBox::BlockBox(box_model);

            return LineContext {
                end_pos: (0.0, line_ctx.end_pos.1 + ch),
                ..line_ctx
            };
        }

        // In-flow block boxes with an `auto` width stretch to the available
        // width. Out-of-flow (absolute/fixed) boxes shrink-to-fit instead,
        // so keep `auto` as `None` to let the inner display shrink-wrap
        // around its content.
        let content_width_opt = if node.style.position.kind.is_out_of_flow() {
            content_width_opt
        } else {
            content_width_opt.or(ctx.available_width.map(|v| {
                let padding_border_edge = border.left + border.right + padding.left + padding.right;
                let value = if node.style.box_sizing == BoxSizing::ContentBox {
                    v - padding_border_edge
                } else {
                    v
                };

                self.apply_size_constraints(
                    value,
                    &node.style.size,
                    ctx,
                    true,
                    &node.style.box_sizing,
                    padding_border_edge,
                )
            }))
        };

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
        ctx: &InternalLayoutContext,
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
        ctx: &InternalLayoutContext,
        line_ctx: LineContext,
        size_opt: (Option<f32>, Option<f32>),
        intrinsic_pass: bool,
    ) -> LineContext {
        match node.style.display.inner {
            InnerDisplay::Flow | InnerDisplay::FlowRoot => {
                self.layout_flow(node, ctx, line_ctx, size_opt, intrinsic_pass)
            }
            InnerDisplay::Flex => self.layout_flex(node, ctx, line_ctx, size_opt, intrinsic_pass),
            InnerDisplay::Grid => self.layout_grid(node, ctx, line_ctx, size_opt, intrinsic_pass),
        }
    }

    fn layout_grid(
        &self,
        node: &mut LayoutNode,
        ctx: &InternalLayoutContext,
        line_ctx: LineContext,
        content_size_opt: (Option<f32>, Option<f32>),
        intrinsic_pass: bool,
    ) -> LineContext {
        let (content_width_opt, content_height_opt) = content_size_opt;
        let available_width = content_width_opt.or(ctx.available_width);
        let column_gap = node
            .style
            .column_gap
            .resolve_with(available_width, self.viewport_width, self.viewport_height)
            .unwrap_or(0.0)
            .max(0.0);
        let row_gap = node
            .style
            .row_gap
            .resolve_with(
                content_height_opt,
                self.viewport_width,
                self.viewport_height,
            )
            .unwrap_or(0.0)
            .max(0.0);

        let base_ctx = InternalLayoutContext {
            containing_block_width: available_width,
            containing_block_height: content_height_opt,
            available_width,
            parent_assigned_border_width: None,
            parent_assigned_border_height: None,
            viewport_width: ctx.viewport_width,
            viewport_height: ctx.viewport_height,
            ..Default::default()
        };
        self.layout_out_of_flow_node_children(node, &base_ctx, intrinsic_pass);

        let items: Vec<_> = LayoutItems::new(&node.children)
            .filter(|item| item_participates_in_normal_flow(&node.children, item))
            .collect();
        let columns = expand_grid_tracks(
            &node.style.grid_template_columns,
            available_width,
            column_gap,
            items.len(),
            self.viewport_width,
            self.viewport_height,
        );
        let rows = expand_grid_tracks(
            &node.style.grid_template_rows,
            content_height_opt,
            row_gap,
            items.len(),
            self.viewport_width,
            self.viewport_height,
        );
        let area_column_count = node
            .style
            .grid_template_areas
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(0);
        let mut slots = build_grid_slots(node, items, columns.len().max(area_column_count));

        for slot in &mut slots {
            let (width, height) =
                self.measure_grid_item(node, &slot.item, &base_ctx, intrinsic_pass);
            slot.intrinsic_width = width;
            slot.intrinsic_height = height;
        }

        let column_count = slots
            .iter()
            .map(|slot| slot.column + slot.column_span)
            .max()
            .unwrap_or(1)
            .max(columns.len())
            .max(area_column_count)
            .max(1);
        let row_count = slots
            .iter()
            .map(|slot| slot.row + slot.row_span)
            .max()
            .unwrap_or(1)
            .max(rows.len())
            .max(node.style.grid_template_areas.len())
            .max(1);

        let mut column_intrinsic = vec![0.0f32; column_count];
        let mut row_intrinsic = vec![0.0f32; row_count];
        for slot in &slots {
            if slot.column_span == 1 {
                column_intrinsic[slot.column] =
                    column_intrinsic[slot.column].max(slot.intrinsic_width);
            }
            if slot.row_span == 1 {
                row_intrinsic[slot.row] = row_intrinsic[slot.row].max(slot.intrinsic_height);
            }
        }

        let columns = resolve_grid_tracks(
            &columns,
            column_count,
            available_width,
            column_gap,
            &column_intrinsic,
            self.viewport_width,
            self.viewport_height,
        );
        let rows = resolve_grid_tracks(
            &rows,
            row_count,
            content_height_opt,
            row_gap,
            &row_intrinsic,
            self.viewport_width,
            self.viewport_height,
        );

        let children_width =
            columns.iter().sum::<f32>() + column_gap * column_count.saturating_sub(1) as f32;
        let children_height =
            rows.iter().sum::<f32>() + row_gap * row_count.saturating_sub(1) as f32;
        let width = content_width_opt
            .or(available_width)
            .unwrap_or(children_width);
        let height = content_height_opt.unwrap_or(children_height);
        let padding = self.resolve_padding(&node.style.spacing, ctx);
        let border = self.resolve_border(&node.style.spacing, ctx);
        node.layout_box = LayoutBox::BlockBox(create_box_model(
            width,
            height,
            children_width,
            children_height,
            padding,
            border,
        ));

        if !intrinsic_pass {
            self.place_grid_items(
                node, &slots, &columns, &rows, column_gap, row_gap, &base_ctx,
            );
        }

        LineContext {
            end_pos: match node.style.display.outer {
                OuterDisplay::Block => (0.0, line_ctx.end_pos.1 + node.layout_box.height_box()),
                _ => (
                    line_ctx.end_pos.0 + node.layout_box.width_box(),
                    line_ctx.end_pos.1,
                ),
            },
            current_x: line_ctx.current_x + node.layout_box.width_box(),
            margin_start: 0.0,
            margin_end: 0.0,
        }
    }

    fn measure_grid_item(
        &self,
        node: &mut LayoutNode,
        item: &LayoutItem,
        ctx: &InternalLayoutContext,
        intrinsic_pass: bool,
    ) -> (f32, f32) {
        match item {
            LayoutItem::Node(index) => {
                let child = node.children[*index].node_mut().unwrap();
                let _ = self.layout_node(child, ctx, EMPTY_LINE_CONTEXT, intrinsic_pass);
                (child.layout_box.width_box(), child.layout_box.height_box())
            }
            LayoutItem::Fragments(range) => {
                let line_height = resolved_fragment_line_height(
                    &node.children,
                    range.clone(),
                    node.style.line_height.resolve_with(
                        None,
                        self.viewport_width,
                        self.viewport_height,
                    ),
                );
                let (width, height, _) = flow_fragment_range(
                    &mut node.children,
                    range.clone(),
                    EMPTY_LINE_CONTEXT,
                    0,
                    line_height,
                    ctx.available_width.unwrap_or(self.viewport_width),
                );
                (width, height)
            }
            LayoutItem::Custom(index) => {
                let object = node.children[*index].custom_child().unwrap();
                let measured = object.layouter().measure(&crate::LayoutContext::from(ctx));
                (measured.width, measured.height)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn place_grid_items(
        &self,
        node: &mut LayoutNode,
        slots: &[GridItemSlot],
        columns: &[f32],
        rows: &[f32],
        column_gap: f32,
        row_gap: f32,
        base_ctx: &InternalLayoutContext,
    ) {
        for slot in slots {
            let x = columns[..slot.column].iter().sum::<f32>() + column_gap * slot.column as f32;
            let y = rows[..slot.row].iter().sum::<f32>() + row_gap * slot.row as f32;
            let width = columns[slot.column..slot.column + slot.column_span]
                .iter()
                .sum::<f32>()
                + column_gap * slot.column_span.saturating_sub(1) as f32;
            let height = rows[slot.row..slot.row + slot.row_span].iter().sum::<f32>()
                + row_gap * slot.row_span.saturating_sub(1) as f32;

            match &slot.item {
                LayoutItem::Node(index) => {
                    let child = node.children[*index].node_mut().unwrap();
                    let assigned_width =
                        matches!(child.style.size.width, LengthOrAuto::Auto).then_some(width);
                    let assigned_height =
                        matches!(child.style.size.height, LengthOrAuto::Auto).then_some(height);
                    let child_ctx = InternalLayoutContext {
                        containing_block_width: Some(width),
                        containing_block_height: Some(height),
                        available_width: Some(width),
                        parent_assigned_border_width: assigned_width,
                        parent_assigned_border_height: assigned_height,
                        ..*base_ctx
                    };
                    let _ = self.layout_node(child, &child_ctx, EMPTY_LINE_CONTEXT, false);
                    child.layout_box.shift(x, y);
                }
                LayoutItem::Fragments(range) => {
                    let line_height = resolved_fragment_line_height(
                        &node.children,
                        range.clone(),
                        node.style.line_height.resolve_with(
                            None,
                            self.viewport_width,
                            self.viewport_height,
                        ),
                    );
                    let _ = flow_fragment_range(
                        &mut node.children,
                        range.clone(),
                        LineContext {
                            end_pos: (x, y),
                            current_x: x,
                            margin_start: 0.0,
                            margin_end: 0.0,
                        },
                        0,
                        line_height,
                        x + width,
                    );
                }
                LayoutItem::Custom(index) => {
                    let object = node.children[*index].custom_child_mut().unwrap();
                    let measured = object
                        .layouter()
                        .measure(&crate::LayoutContext::from(base_ctx));
                    let box_width = if node.style.align_items == AlignItems::Stretch {
                        width
                    } else {
                        measured.width
                    };
                    let box_height = if node.style.align_items == AlignItems::Stretch {
                        height
                    } else {
                        measured.height
                    };
                    let mut box_model = create_box_model(
                        box_width,
                        box_height,
                        box_width,
                        box_height,
                        Edge::default(),
                        Edge::default(),
                    );
                    box_model.shift(x, y);
                    object.set_result(CustomObjectResult {
                        spans: Vec::new(),
                        box_model,
                    });
                }
            }
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
        ctx: &InternalLayoutContext,
        line_ctx: LineContext,
        content_size_opt: (Option<f32>, Option<f32>),
        intrinsic_pass: bool,
    ) -> LineContext {
        let (content_width_opt, content_height_opt) = content_size_opt;

        let border = self.resolve_border(&node.style.spacing, ctx);
        let padding = self.resolve_padding(&node.style.spacing, ctx);

        let LineContext {
            end_pos: (_, end_y),
            current_x: parent_current_x,
            ..
        } = line_ctx;

        let base_ctx_for_child = InternalLayoutContext {
            containing_block_width: content_width_opt,
            containing_block_height: content_height_opt,
            available_width: content_width_opt.or(ctx.available_width),
            parent_assigned_border_width: None,
            parent_assigned_border_height: None,
            viewport_width: ctx.viewport_width,
            viewport_height: ctx.viewport_height,
            ..Default::default()
        };

        let line_height = node
            .style
            .line_height
            .resolve_with(None, self.viewport_width, self.viewport_height)
            .unwrap_or_default();

        let outbox_width = content_width_opt
            .or(ctx.available_width)
            .unwrap_or(self.viewport_width);

        let mut state = FlowState {
            cursor: FlowCursor {
                x: line_ctx.end_pos.0,
                y: end_y,
                current_x: 0.0,
                line_index: 0,
            },
            accum: FlowAccum {
                prev_child_margin: 0.0,
                first_child_margin: 0.0,
                pending_advance: 0.0,
                children_width: 0.0,
                children_height: 0.0,
                max_inline_line_height: line_height,
                first_block_child_processed: false,
                line_span_buf: Vec::new(),
            },
            padding,
            border,
            start_x: line_ctx.end_pos.0,
            end_y,
            parent_current_x,
            content_width_opt,
            intrinsic_pass,
            // Only InnerDisplay::Flow collapses margins.
            // FlowRoot (flow-root / inline-block) establishes a new
            // Block Formatting Context, which isolates margin collapsing.
            collapse_margins: node.style.display.inner == InnerDisplay::Flow,
        };

        let items: Vec<_> = LayoutItems::new(&node.children).collect();

        for item in items {
            match item {
                LayoutItem::Fragments(range) => self.process_flow_fragment_item(
                    node,
                    range,
                    outbox_width,
                    line_height,
                    &mut state,
                ),
                LayoutItem::Node(i) => {
                    self.process_flow_node_item(node, i, ctx, &base_ctx_for_child, &mut state)
                }
                LayoutItem::Custom(i) => {
                    match node.children[i]
                        .custom_child()
                        .unwrap()
                        .style()
                        .display
                        .outer
                    {
                        OuterDisplay::Inline => {
                            let mut ctx_for_child = crate::LayoutContext::from(&base_ctx_for_child);
                            ctx_for_child.start_pos = state.cursor.pos();
                            ctx_for_child.available_inline_size =
                                (outbox_width - state.cursor.x).max(0.0);
                            ctx_for_child.line_height = line_height;
                            self.process_flow_custom_item(
                                node,
                                i,
                                &ctx_for_child,
                                outbox_width,
                                &mut state,
                            );
                        }
                        OuterDisplay::Block => {
                            let ctx_for_child = crate::LayoutContext::from(&base_ctx_for_child);
                            self.process_flow_custom_block_item(
                                node,
                                i,
                                &ctx_for_child,
                                &mut state,
                            );
                        }
                        OuterDisplay::None => {}
                    }
                }
            }
        }

        self.finalize_flow_box(node, ctx, content_width_opt, content_height_opt, state)
    }

    fn process_flow_fragment_item(
        &self,
        node: &mut LayoutNode,
        range: std::ops::Range<usize>,
        outbox_width: f32,
        line_height: f32,
        state: &mut FlowState,
    ) {
        let mut fragment_node_buffer = node.children[range.clone()]
            .iter_mut()
            .filter_map(|c| match c {
                LayoutChild::Fragment(f) => Some(f),
                _ => None,
            })
            .collect();

        let line_ctx_for_child = state.cursor.line_ctx();

        let (line_spans, updated_line_ctx, updated_line_index) = Self::flow_fragments(
            &mut fragment_node_buffer,
            line_ctx_for_child,
            state.cursor.line_index,
            line_height,
            outbox_width,
        );

        let had_line_spans = !line_spans.is_empty();

        if had_line_spans {
            let max_span_width = line_spans
                .iter()
                .map(|s| s.width())
                .filter(|w| !w.is_nan())
                .max_by(f32::total_cmp)
                .unwrap_or(0.0);
            state.accum.children_width = state.accum.children_width.max(max_span_width);
            state.accum.children_height = state
                .accum
                .children_height
                .max(updated_line_ctx.end_pos.1 + state.accum.max_inline_line_height);
        }

        state.cursor.update_from(&updated_line_ctx);
        state.cursor.line_index = updated_line_index;

        for span in line_spans {
            push_or_merge_line_span(&mut state.accum.line_span_buf, span);
        }

        state.accum.max_inline_line_height = state.accum.max_inline_line_height.max(line_height);
        if had_line_spans {
            state.accum.pending_advance = state.accum.max_inline_line_height;
        }
    }

    fn process_flow_node_item(
        &self,
        node: &mut LayoutNode,
        i: usize,
        ctx: &InternalLayoutContext,
        base_ctx_for_child: &InternalLayoutContext,
        state: &mut FlowState,
    ) {
        let child_node = match &mut node.children[i] {
            LayoutChild::Node(n) => n,
            _ => unreachable!(),
        };

        if child_node.style.position.kind.is_out_of_flow() {
            let _ = self.layout_node(
                child_node,
                base_ctx_for_child,
                EMPTY_LINE_CONTEXT,
                state.intrinsic_pass,
            );
            return;
        }

        let child_margin = self.resolve_margin(&child_node.style.spacing, ctx);

        let ctx_for_child = InternalLayoutContext {
            available_width: state
                .content_width_opt
                .or(ctx.available_width)
                .map(|v| v - child_margin.left.unwrap_or(0.0) - child_margin.right.unwrap_or(0.0)),
            ..*base_ctx_for_child
        };

        let line_ctx_for_child = state.cursor.line_ctx();

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
            state.intrinsic_pass,
        );

        let (child_position_x, child_position_y) = if child_is_block {
            (
                0.0,
                line_ctx_for_child.end_pos.1 + state.accum.pending_advance,
            )
        } else {
            line_ctx_for_child.end_pos
        };

        if child_is_block {
            state
                .cursor
                .set_pos(0.0, child_position_y + updated_line_ctx.end_pos.1);
        } else {
            state.cursor.update_from(&updated_line_ctx);
        }

        let EdgeOption {
            left: ml_opt,
            top,
            right: mr_opt,
            bottom,
        } = child_margin;

        let (ml, mr) = resolve_flow_margin_auto(
            ml_opt,
            mr_opt,
            state.content_width_opt,
            child_node,
            child_is_block,
        );

        child_node.layout_box.shift(ml, 0.0);

        if child_is_block {
            let top_collapses = state.collapse_margins
                && !state.accum.first_block_child_processed
                && state.border.top == 0.0
                && state.padding.top == 0.0;

            let effective_top = top.unwrap_or_default().max(updated_line_ctx.margin_start);
            let effective_bottom = bottom.unwrap_or_default().max(updated_line_ctx.margin_end);

            if state.collapse_margins {
                if top_collapses {
                    // First block child with no border/padding above:
                    // store margin for potential collapse with parent.
                    state.accum.first_child_margin = effective_top;
                } else {
                    // Subsequent child: collapse with previous sibling's bottom margin.
                    let margin_top = state.accum.prev_child_margin.max(effective_top);
                    child_node.layout_box.shift(0.0, margin_top);
                    state.cursor.shift_y(margin_top);
                }
            } else {
                // FlowRoot: margins are additive, never collapse.
                let margin_top = state.accum.prev_child_margin + effective_top;
                child_node.layout_box.shift(0.0, margin_top);
                state.cursor.shift_y(margin_top);
            }

            state.accum.prev_child_margin = effective_bottom;
            state.accum.first_block_child_processed = true;
        } else {
            // Inline-level margins consume inline space on both sides.
            state.cursor.x += ml + mr;
            state.cursor.current_x += ml + mr;
        }

        if child_node.style.display.outer == OuterDisplay::Inline {
            child_node.layout_box.shift(child_position_x, 0.0);
        } else {
            child_node
                .layout_box
                .shift(child_position_x, child_position_y);
        }

        if node.style.display.outer == OuterDisplay::Inline
            && child_node.style.display.outer == OuterDisplay::Inline
        {
            collect_inline_spans_from_child(
                child_node,
                line_ctx_for_child,
                &mut state.accum.line_span_buf,
            );
        }

        if child_node.style.display.outer == OuterDisplay::Inline {
            for line_box in child_node.layout_box.iter() {
                state.accum.max_inline_line_height = state
                    .accum
                    .max_inline_line_height
                    .max(line_box.border_box.height);
            }
        }

        let (child_right, child_bottom) = compute_child_layout_extent(child_node);

        state.accum.children_width = state.accum.children_width.max(child_right);
        let inline_extent = compute_inline_extent(
            child_node,
            child_is_block,
            child_bottom,
            state.cursor.y,
            state.accum.prev_child_margin,
            state.accum.max_inline_line_height,
        );
        state.accum.children_height = state.accum.children_height.max(inline_extent);

        if child_node.style.display.outer == OuterDisplay::Inline {
            state.accum.pending_advance = child_node.layout_box.height()
        } else {
            state.accum.pending_advance = 0.0;
        }
    }

    fn process_flow_custom_item(
        &self,
        node: &mut LayoutNode,
        i: usize,
        ctx_for_child: &crate::LayoutContext,
        outbox_width: f32,
        state: &mut FlowState,
    ) {
        let child = match &mut node.children[i] {
            LayoutChild::Custom(c) => c,
            _ => unreachable!(),
        };

        let layout_box = child.layouter_mut().layout(ctx_for_child);

        let line_height = ctx_for_child.line_height;

        let (line_spans, box_model) = match layout_box {
            LayoutBox::InlineBox(inline) => {
                if inline.line_spans.is_empty() {
                    let mut bm = inline.box_model;
                    let width = bm.border_box.width;
                    let height = bm.border_box.height;

                    let (mut x, mut y) = state.cursor.pos();
                    let mut line_index = state.cursor.line_index;

                    if x + width > outbox_width && x > 0.0 {
                        x = 0.0;
                        y += line_height;
                        line_index += 1;
                    }

                    let span = LineSpan {
                        x_range: x..(x + width),
                        line_pos: (x, y),
                        line_index,
                    };

                    bm.shift(x - bm.border_box.x, y - bm.border_box.y);

                    state.accum.max_inline_line_height =
                        state.accum.max_inline_line_height.max(height);
                    state.accum.children_height = state.accum.children_height.max(y + height);

                    (vec![span], bm)
                } else {
                    let mut line_spans = inline.line_spans;
                    for span in &mut line_spans {
                        span.line_index += state.cursor.line_index;
                    }
                    (line_spans, inline.box_model)
                }
            }
            LayoutBox::BlockBox(mut bm) => {
                // The object declared an inline formatting context but
                // produced a block box. Place it atomically like a fragment:
                // the box is never split, so when it does not fit on a
                // non-empty line the whole box wraps to the next line.
                let width = bm.border_box.width;
                let height = bm.border_box.height;

                let (mut x, mut y) = state.cursor.pos();
                let mut line_index = state.cursor.line_index;

                if x + width > outbox_width && x > 0.0 {
                    x = 0.0;
                    y += line_height;
                    line_index += 1;
                }

                let span = LineSpan {
                    x_range: x..(x + width),
                    line_pos: (x, y),
                    line_index,
                };

                bm.shift(x - bm.border_box.x, y - bm.border_box.y);

                state.accum.max_inline_line_height = state.accum.max_inline_line_height.max(height);
                state.accum.children_height = state.accum.children_height.max(y + height);

                (vec![span], bm)
            }
            LayoutBox::None => (Vec::new(), BoxModel::default()),
        };

        let had_line_spans = !line_spans.is_empty();

        if had_line_spans {
            let max_span_width = line_spans
                .iter()
                .map(LineSpan::width)
                .filter(|w| !w.is_nan())
                .max_by(f32::total_cmp)
                .unwrap_or(0.0);

            state.accum.children_width = state.accum.children_width.max(max_span_width);

            if let Some(last_span) = line_spans.last() {
                state.accum.children_height = state
                    .accum
                    .children_height
                    .max(last_span.line_pos.1 + line_height);
            }
        }

        if let Some(last_span) = line_spans.last() {
            state.cursor.set_pos(
                last_span.line_pos.0 + last_span.width(),
                last_span.line_pos.1,
            );
            state.cursor.current_x = state.cursor.x;

            state.cursor.line_index = last_span.line_index;
        }

        for span in &line_spans {
            push_or_merge_line_span(&mut state.accum.line_span_buf, span.clone());
        }

        state.accum.max_inline_line_height = state.accum.max_inline_line_height.max(line_height);

        if had_line_spans {
            state.accum.pending_advance = state.accum.max_inline_line_height;
        }

        child.set_result(CustomObjectResult {
            spans: line_spans,
            box_model,
        });
    }

    /// Lays out a block-level [`LayoutChild::Custom`] child in flow.
    ///
    /// Mirrors the block branch of [`Self::process_flow_node_item`]: the object
    /// forces a new line, is placed below the current block content, and its
    /// border-box [`BoxModel`] is stored in the child's
    /// [`CustomObjectResult::box_model`].
    fn process_flow_custom_block_item(
        &self,
        node: &mut LayoutNode,
        i: usize,
        ctx_for_child: &crate::LayoutContext,
        state: &mut FlowState,
    ) {
        let (mut box_model, mut rect, spans) = match &mut node.children[i] {
            LayoutChild::Custom(c) => match c.layouter_mut().layout(ctx_for_child) {
                LayoutBox::BlockBox(bm) => {
                    let rect = bm.border_box;
                    (bm, rect, Vec::new())
                }
                // The object declared a block formatting context but produced
                // an inline box. Wrap it in an anonymous block box: its box
                // model becomes the block (placed below on its own line) and
                // its spans are preserved for the result.
                LayoutBox::InlineBox(inline) => {
                    let rect = inline.box_model.border_box;
                    (inline.box_model, rect, inline.line_spans)
                }
                LayoutBox::None => (BoxModel::default(), Rect::default(), Vec::new()),
            },
            _ => unreachable!(),
        };

        let child_position_x = 0.0;
        let child_position_y = state.cursor.y + state.accum.pending_advance;

        box_model.shift(child_position_x - rect.x, child_position_y - rect.y);
        rect = box_model.border_box;

        state.cursor.set_pos(0.0, child_position_y + rect.height);
        state.accum.children_width = state.accum.children_width.max(rect.width);
        state.accum.children_height = state
            .accum
            .children_height
            .max(child_position_y + rect.height);
        state.accum.first_block_child_processed = true;
        state.accum.prev_child_margin = 0.0;
        state.accum.pending_advance = 0.0;

        if let LayoutChild::Custom(child) = &mut node.children[i] {
            child.set_result(CustomObjectResult { spans, box_model });
        }
    }

    fn finalize_flow_box(
        &self,
        node: &mut LayoutNode,
        ctx: &InternalLayoutContext,
        content_width_opt: Option<f32>,
        content_height_opt: Option<f32>,
        state: FlowState,
    ) -> LineContext {
        let FlowState {
            cursor:
                FlowCursor {
                    x: cursor_x,
                    y: cursor_y,
                    current_x,
                    ..
                },
            accum:
                FlowAccum {
                    children_width,
                    children_height,
                    max_inline_line_height,
                    prev_child_margin,
                    first_child_margin,
                    mut line_span_buf,
                    ..
                },
            padding,
            border,
            start_x,
            end_y,
            parent_current_x,
            collapse_margins,
            ..
        } = state;

        let pb_w = padding.left + padding.right + border.left + border.right;
        let pb_h = padding.top + padding.bottom + border.top + border.bottom;

        if node.style.display.outer == OuterDisplay::Inline {
            let has_only_blocks = line_span_buf.is_empty();
            let content_w = children_width.max(current_x);
            let content_h = if has_only_blocks {
                children_height.max(max_inline_line_height)
            } else {
                max_inline_line_height
            };

            // An inline-block is an atomic inline-level flow root. Unlike a
            // normal inline box (which shrink-wraps), its explicit dimensions
            // (width/height, min/max constraints) are always applied — even
            // when the box has no children.
            let (content_w, content_h) = if node.style.display.inner == InnerDisplay::FlowRoot {
                (
                    self.apply_size_constraints(
                        content_width_opt.unwrap_or(content_w),
                        &node.style.size,
                        ctx,
                        true,
                        &node.style.box_sizing,
                        pb_w,
                    ),
                    self.apply_size_constraints(
                        content_height_opt.unwrap_or(content_h),
                        &node.style.size,
                        ctx,
                        false,
                        &node.style.box_sizing,
                        pb_h,
                    ),
                )
            } else {
                (content_w, content_h)
            };

            let mut box_model = create_box_model(
                content_w,
                content_h,
                children_width,
                children_height,
                padding,
                border,
            );

            // For inline boxes, shift the box model so that the content
            // origin is at (0, 0). Inline-block (FlowRoot) is an atomic box
            // and keeps its border/padding origin unshifted — its content
            // area remains inset relative to the border box, as browsers do.
            if node.style.display.inner != InnerDisplay::FlowRoot {
                box_model.shift(-(border.left + padding.left), -(border.top + padding.top));
            }

            let line_spans = if node.style.display.inner == InnerDisplay::FlowRoot {
                vec![LineSpan {
                    x_range: 0.0..content_w,
                    line_pos: (start_x, end_y),
                    line_index: 0,
                }]
            } else {
                for (i, span) in line_span_buf.iter_mut().enumerate() {
                    span.line_index = i;
                }
                line_span_buf
            };

            node.layout_box = LayoutBox::InlineBox(InlineBox {
                box_model,
                line_spans,
            });

            // Inline-block advances the inline cursor by its full width (it is
            // an atomic inline-level box).  Normal inline boxes only advance
            // by the placed current_x (the line-end position within the line).
            let (end_pos, current_x) = if node.style.display.inner == InnerDisplay::FlowRoot {
                let width = node.layout_box.width();
                ((start_x + width, end_y), parent_current_x + width)
            } else {
                ((cursor_x, cursor_y), parent_current_x + current_x)
            };

            LineContext {
                end_pos,
                current_x,
                margin_start: 0.0,
                margin_end: 0.0,
            }
        } else {
            // A block-level node wrapping exactly one custom child behaves as
            // a replaced element (e.g. <button>/<img>/<input>). When
            // `SizeStyle::auto_behavior` is `AutoSizeBehavior::ShrinkToFit`,
            // an `auto` width/height shrink-wraps to the custom child's
            // intrinsic-based box instead of stretching to the containing
            // block (which also lets `margin: auto` center such elements).
            // The default is `AutoSizeBehavior::Fill`, i.e. stretch.
            // `content_width_opt` may already be Some(available_width) for an
            // `auto` size, so the explicit-ness is read from the style and the
            // parent assignment (flex) instead.
            let is_custom_leaf = node.style.display.outer == OuterDisplay::Block
                && matches!(&node.children[..], [LayoutChild::Custom(_)]);
            let shrink_to_fit =
                is_custom_leaf && node.style.size.auto_behavior == AutoSizeBehavior::ShrinkToFit;
            let width_auto = matches!(node.style.size.width, LengthOrAuto::Auto);
            let height_auto = matches!(node.style.size.height, LengthOrAuto::Auto);

            let content_width =
                if shrink_to_fit && width_auto && ctx.parent_assigned_border_width.is_none() {
                    (children_width - pb_w).max(0.0)
                } else {
                    content_width_opt.unwrap_or(children_width)
                };

            // Margin collapsing only applies to InnerDisplay::Flow.
            // For FlowRoot the flags stay false so children_height is used as-is.
            let top_collapses = collapse_margins && border.top == 0.0 && padding.top == 0.0;
            let bottom_collapses =
                collapse_margins && border.bottom == 0.0 && padding.bottom == 0.0;

            let content_height =
                if shrink_to_fit && height_auto && ctx.parent_assigned_border_height.is_none() {
                    (children_height - pb_h).max(0.0)
                } else {
                    content_height_opt.unwrap_or(if bottom_collapses {
                        children_height - prev_child_margin
                    } else {
                        children_height
                    })
                };
            let children_h = if bottom_collapses {
                (children_height - prev_child_margin).max(0.0)
            } else {
                children_height
            };

            let content_width = self.apply_size_constraints(
                content_width,
                &node.style.size,
                ctx,
                true,
                &node.style.box_sizing,
                pb_w,
            );
            let content_height = self.apply_size_constraints(
                content_height,
                &node.style.size,
                ctx,
                false,
                &node.style.box_sizing,
                pb_h,
            );

            let box_model = create_box_model(
                content_width,
                content_height,
                children_width,
                children_h,
                padding,
                border,
            );
            let block_height = box_model.border_box.height;

            node.layout_box = LayoutBox::BlockBox(box_model);

            LineContext {
                end_pos: (0.0, end_y + block_height),
                current_x: parent_current_x + current_x,
                // Propagate collapsed margins upward for the parent chain.
                // FlowRoot always yields 0.0 here, stopping the chain.
                margin_start: if top_collapses {
                    first_child_margin
                } else {
                    0.0
                },
                margin_end: if bottom_collapses {
                    prev_child_margin
                } else {
                    0.0
                },
            }
        }
    }

    fn layout_flex(
        &self,
        node: &mut LayoutNode,
        ctx: &InternalLayoutContext,
        line_ctx: LineContext,
        content_size_opt: (Option<f32>, Option<f32>),
        intrinsic_pass: bool,
    ) -> LineContext {
        let axis = Axis::from_flex_direction(&node.style.flex_direction);

        let (content_width_opt, content_height_opt) = content_size_opt;

        let (children_main, children_cross) =
            if !intrinsic_pass || content_width_opt.is_none() || content_height_opt.is_none() {
                let base_ctx_for_children = InternalLayoutContext {
                    containing_block_width: content_width_opt,
                    containing_block_height: content_height_opt,
                    available_width: None,
                    parent_assigned_border_width: None,
                    parent_assigned_border_height: None,
                    viewport_width: ctx.viewport_width,
                    viewport_height: ctx.viewport_height,
                    ..Default::default()
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

        // Apply min/max (resolve padding/border early for box-sizing adjustment)
        let flex_padding = self.resolve_padding(&node.style.spacing, ctx);
        let flex_border = self.resolve_border(&node.style.spacing, ctx);
        let flex_pb_w =
            flex_padding.left + flex_padding.right + flex_border.left + flex_border.right;
        let flex_pb_h =
            flex_padding.top + flex_padding.bottom + flex_border.top + flex_border.bottom;

        let final_width = self.apply_size_constraints(
            width_before_constraints,
            &node.style.size,
            ctx,
            true,
            &node.style.box_sizing,
            flex_pb_w,
        );

        let final_height = self.apply_size_constraints(
            height_before_constraints,
            &node.style.size,
            ctx,
            false,
            &node.style.box_sizing,
            flex_pb_h,
        );

        // Detect whether constraints changed size
        let relayout_needed = (Some(final_width) != content_width_opt
            || Some(final_height) != content_height_opt)
            && !intrinsic_pass;

        // Only relayout if children actually use percentage-based sizes
        // that would resolve differently with the actual containing block.
        let relayout_needed = relayout_needed
            && node
                .children
                .iter()
                .any(|c| c.node().is_some_and(|n| n.style.has_percentage_size()));

        if relayout_needed {
            // Update context
            let base_ctx_for_children = InternalLayoutContext {
                containing_block_width: Some(final_width),
                containing_block_height: Some(final_height),
                available_width: None,
                parent_assigned_border_width: None,
                parent_assigned_border_height: None,
                viewport_width: ctx.viewport_width,
                viewport_height: ctx.viewport_height,
                ..Default::default()
            };

            let (new_main, new_cross) =
                self.layout_flex_children(node, axis, intrinsic_pass, &base_ctx_for_children);

            // Re-key children's cache so the next layout's initial call hits.
            // The relayout stores entries with hash(Some(final_w), ...), but the
            // initial call on the next layout uses hash(content_width_opt, ...).
            let initial_ctx = InternalLayoutContext {
                containing_block_width: content_width_opt,
                containing_block_height: content_height_opt,
                available_width: None,
                parent_assigned_border_width: None,
                parent_assigned_border_height: None,
                viewport_width: ctx.viewport_width,
                viewport_height: ctx.viewport_height,
                ..Default::default()
            };
            let initial_key = crate::cache::make_layout_key(&initial_ctx, self);
            for child in &mut node.children {
                if let LayoutChild::Node(child_node) = child {
                    child_node.layout_box_cache.0 = initial_key;
                }
            }

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
            end_pos: match node.style.display.outer {
                OuterDisplay::Block => (0.0, line_ctx.end_pos.1 + node.layout_box.height_box()),
                _ => (
                    line_ctx.end_pos.0 + node.layout_box.width_box(),
                    line_ctx.end_pos.1,
                ),
            },
            current_x: line_ctx.current_x + node.layout_box.width_box(),
            margin_start: 0.0,
            margin_end: 0.0,
        }
    }

    /// Layout of Flex child elements
    /// Layouts flex children with flex algorithm.
    fn layout_flex_children(
        &self,
        node: &mut LayoutNode,
        axis: Axis,
        intrinsic_pass: bool,
        base_ctx_for_children: &InternalLayoutContext,
    ) -> (f32, f32) {
        self.layout_out_of_flow_node_children(node, base_ctx_for_children, intrinsic_pass);

        let flex_items: Vec<_> = LayoutItems::new(&node.children)
            .filter(|item| item_participates_in_normal_flow(&node.children, item))
            .collect();
        if flex_items.is_empty() {
            return (0.0, 0.0);
        }

        let cbm = base_ctx_for_children.containing_block_main(axis);
        let vw = self.viewport_width;
        let vh = self.viewport_height;

        let gap = axis
            .gap(&node.style)
            .resolve_with(cbm, vw, vh)
            .unwrap_or(0.0)
            .max(0.0);

        let mut states = vec![FlexItemState::default(); flex_items.len()];
        self.init_flex_item_states(node, axis, base_ctx_for_children, &flex_items, &mut states);

        let line_ranges = collect_flex_line_ranges(
            &states
                .iter()
                .map(flex_state_outer_main_size)
                .collect::<Vec<_>>(),
            gap,
            cbm,
            node.style.flex_wrap,
        );
        let cross_gap = axis
            .cross_gap(&node.style)
            .resolve_with(base_ctx_for_children.containing_block_cross(axis), vw, vh)
            .unwrap_or(0.0)
            .max(0.0);

        let mut resolved_lines = Vec::with_capacity(line_ranges.len());
        let mut max_main: f32 = 0.0;
        let mut total_cross: f32 = 0.0;

        for range in line_ranges {
            let items = &flex_items[range.clone()];
            let mut line_states = states[range].to_vec();
            self.resolve_flex_line(node, items, &mut line_states, gap, cbm);
            let line_cross = if node.style.flex_wrap == FlexWrap::NoWrap {
                base_ctx_for_children.containing_block_cross(axis)
            } else {
                None
            };
            let (main, cross) = self.finalize_flex_children_layout(
                node,
                axis,
                base_ctx_for_children,
                intrinsic_pass,
                items,
                line_states.clone(),
                line_cross,
            );
            max_main = max_main.max(main);
            total_cross += cross;
            resolved_lines.push((items.to_vec(), line_states, cross));
        }

        total_cross += cross_gap * resolved_lines.len().saturating_sub(1) as f32;

        if node.style.flex_wrap != FlexWrap::NoWrap
            && node.style.align_content == AlignContent::Stretch
            && let Some(container_cross) = base_ctx_for_children.containing_block_cross(axis)
        {
            let extra =
                (container_cross - total_cross).max(0.0) / resolved_lines.len().max(1) as f32;
            if extra > EPSILON {
                total_cross = cross_gap * resolved_lines.len().saturating_sub(1) as f32;
                for (items, states, cross) in &mut resolved_lines {
                    *cross += extra;
                    let (main, final_cross) = self.finalize_flex_children_layout(
                        node,
                        axis,
                        base_ctx_for_children,
                        intrinsic_pass,
                        items,
                        states.clone(),
                        Some(*cross),
                    );
                    max_main = max_main.max(main);
                    *cross = final_cross.max(*cross);
                    total_cross += *cross;
                }
            }
        }

        (max_main, total_cross)
    }

    fn resolve_flex_line(
        &self,
        node: &LayoutNode,
        flex_items: &[LayoutItem],
        states: &mut [FlexItemState],
        gap: f32,
        containing_main: Option<f32>,
    ) {
        let item_len = flex_items.len();
        let mut total_grow = states
            .iter()
            .filter(|state| !state.frozen_grow)
            .map(|state| state.grow)
            .sum();
        let mut remaining = compute_flex_remaining(containing_main, states, gap, item_len);

        loop {
            if remaining > EPSILON {
                if total_grow <= 0.0 {
                    break;
                }
                self.flex_grow_redistribution(
                    states,
                    flex_items,
                    &mut remaining,
                    item_len,
                    &mut total_grow,
                    node,
                );
                if remaining.abs() < EPSILON {
                    break;
                }
            } else if remaining < -EPSILON {
                let old_remaining = remaining;
                self.flex_shrink_redistribution(states, flex_items, &mut remaining, item_len, node);
                if (old_remaining - remaining).abs() < EPSILON {
                    break;
                }
            } else {
                break;
            }
        }
    }

    fn flex_children_main_total(
        &self,
        node: &mut LayoutNode,
        axis: Axis,
        ctx: &InternalLayoutContext,
        items: &[LayoutItem],
    ) -> f32 {
        let vw = self.viewport_width;
        let vh = self.viewport_height;
        items
            .iter()
            .map(|item| match item {
                LayoutItem::Node(index) => {
                    let child = node.children[*index].node().unwrap();
                    let tuple = (child.layout_box.width_box(), child.layout_box.height_box());
                    let margin = self
                        .resolve_margin(&child.style.spacing, ctx)
                        .unwrap_or_default();
                    let margin_main = axis.edge_main(&margin);
                    axis.tuple_main(tuple) + margin_main.0 + margin_main.1
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
                LayoutItem::Custom(index) => {
                    if let LayoutChild::Custom(child) = &mut node.children[*index] {
                        if child.style().display.outer == OuterDisplay::None {
                            0.0
                        } else {
                            let measured = child
                                .layouter_mut()
                                .measure(&crate::LayoutContext::from(ctx));
                            let tuple = (measured.width, measured.height);
                            axis.tuple_main(tuple)
                        }
                    } else {
                        0.0
                    }
                }
            })
            .sum()
    }

    fn flex_children_cross_max(
        &self,
        node: &mut LayoutNode,
        axis: Axis,
        ctx: &InternalLayoutContext,
        items: &[LayoutItem],
    ) -> f32 {
        let vw = self.viewport_width;
        let vh = self.viewport_height;
        items.iter().fold(0.0_f32, |maximum, item| {
            let size = match item {
                LayoutItem::Node(index) => {
                    let child = node.children[*index].node().unwrap();
                    let tuple = (child.layout_box.width_box(), child.layout_box.height_box());
                    let margin = self
                        .resolve_margin(&child.style.spacing, ctx)
                        .unwrap_or_default();
                    let margin_cross = match axis {
                        Axis::Horizontal => margin.top + margin.bottom,
                        Axis::Vertical => margin.left + margin.right,
                    };
                    axis.tuple_cross(tuple) + margin_cross
                }
                LayoutItem::Fragments(range) => {
                    let line_height = resolved_fragment_line_height(
                        &node.children,
                        range.clone(),
                        node.style.line_height.resolve_with(None, vw, vh),
                    );
                    let line_count = node.children[range.clone()]
                        .iter()
                        .filter(|child| {
                            child
                                .fragment()
                                .is_some_and(|fragment| fragment.node.is_line_break())
                        })
                        .count()
                        + 1;
                    match axis {
                        Axis::Horizontal => line_height * line_count as f32,
                        Axis::Vertical => node.children[range.clone()]
                            .iter()
                            .map(|child| child.fragment().unwrap().node.width())
                            .sum(),
                    }
                }
                LayoutItem::Custom(index) => {
                    let object = node.children[*index].custom_child().unwrap();
                    let measured = object.layouter().measure(&crate::LayoutContext::from(ctx));
                    axis.tuple_cross((measured.width, measured.height))
                }
            };
            maximum.max(size)
        })
    }

    fn count_main_axis_auto_margins(
        &self,
        node: &LayoutNode,
        axis: Axis,
        items: &[LayoutItem],
    ) -> usize {
        let mut count = 0usize;
        for item in items {
            if let LayoutItem::Node(index) = item
                && let crate::LayoutChild::Node(n) = &node.children[*index]
            {
                if n.style.position.kind.is_out_of_flow() {
                    continue;
                }
                let spacing = &n.style.spacing;
                match axis {
                    Axis::Horizontal => {
                        if spacing.margin_left == crate::LengthOrAuto::Auto {
                            count += 1;
                        }
                        if spacing.margin_right == crate::LengthOrAuto::Auto {
                            count += 1;
                        }
                    }
                    Axis::Vertical => {
                        if spacing.margin_top == crate::LengthOrAuto::Auto {
                            count += 1;
                        }
                        if spacing.margin_bottom == crate::LengthOrAuto::Auto {
                            count += 1;
                        }
                    }
                }
            }
        }
        count
    }

    fn position_flex_node_child(
        &self,
        node: &mut LayoutNode,
        axis: Axis,
        ctx: &InternalLayoutContext,
        index: usize,
        placement: &mut FlexPlacementCtx,
    ) {
        let child_node = node.children[index].node_mut().unwrap();

        let child_margin = self
            .resolve_margin(&child_node.style.spacing, ctx)
            .unwrap_or_default();
        let (margin_start_resolved, margin_end_resolved) = axis.edge_main(&child_margin);

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

        let margin_start = if margin_start_auto {
            placement.auto_unit
        } else {
            margin_start_resolved
        };
        let margin_end = if margin_end_auto {
            placement.auto_unit
        } else {
            margin_end_resolved
        };

        let child_main_size = axis.tuple_main((
            child_node.layout_box.width_box(),
            child_node.layout_box.height_box(),
        ));

        if placement.reversed {
            placement.cursor_main -= margin_end;
            placement.cursor_main -= child_main_size;
        } else {
            placement.cursor_main += margin_start;
        }

        let child_main_pos = match axis {
            Axis::Horizontal => placement.content_box.x + placement.cursor_main,
            Axis::Vertical => placement.content_box.y + placement.cursor_main,
        };

        let child_cross_size = axis.tuple_cross((
            child_node.layout_box.width_box(),
            child_node.layout_box.height_box(),
        ));
        let available_cross = axis.rect_cross(&placement.content_box);

        let margin_cross_start_auto = match axis {
            Axis::Horizontal => child_node.style.spacing.margin_top == crate::LengthOrAuto::Auto,
            Axis::Vertical => child_node.style.spacing.margin_left == crate::LengthOrAuto::Auto,
        };
        let margin_cross_end_auto = match axis {
            Axis::Horizontal => child_node.style.spacing.margin_bottom == crate::LengthOrAuto::Auto,
            Axis::Vertical => child_node.style.spacing.margin_right == crate::LengthOrAuto::Auto,
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
            Axis::Horizontal => {
                placement.content_box.y + placement.line_cross_offset + cross_offset
            }
            Axis::Vertical => placement.content_box.x + placement.line_cross_offset + cross_offset,
        };

        let child_origin = match axis {
            Axis::Horizontal => (child_main_pos, child_cross_pos),
            Axis::Vertical => (child_cross_pos, child_main_pos),
        };

        let relative_x = child_origin.0 - placement.content_box.x;
        let relative_y = child_origin.1 - placement.content_box.y;
        child_node.layout_box.shift(relative_x, relative_y);

        if placement.reversed {
            placement.cursor_main -= margin_start + placement.gap + placement.gap_between;
        } else {
            placement.cursor_main +=
                child_main_size + margin_end + placement.gap + placement.gap_between;
        }
    }

    fn position_flex_fragments(
        &self,
        node: &mut LayoutNode,
        axis: Axis,
        range: std::ops::Range<usize>,
        placement: &mut FlexPlacementCtx,
    ) {
        let vw = self.viewport_width;
        let vh = self.viewport_height;
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

        let child_main_pos = if placement.reversed {
            placement.cursor_main -= item_main_size;
            match axis {
                Axis::Horizontal => placement.content_box.x + placement.cursor_main,
                Axis::Vertical => placement.content_box.y + placement.cursor_main,
            }
        } else {
            match axis {
                Axis::Horizontal => placement.content_box.x + placement.cursor_main,
                Axis::Vertical => placement.content_box.y + placement.cursor_main,
            }
        };

        let available_cross = axis.rect_cross(&placement.content_box);
        let cross_offset =
            resolve_align_position(node.style.align_items, item_cross_size, available_cross);

        let child_cross_pos = match axis {
            Axis::Horizontal => {
                placement.content_box.y + placement.line_cross_offset + cross_offset
            }
            Axis::Vertical => placement.content_box.x + placement.line_cross_offset + cross_offset,
        };

        let line_ctx = match axis {
            Axis::Horizontal => LineContext {
                end_pos: (child_main_pos, child_cross_pos),
                current_x: child_main_pos,
                margin_start: 0.0,
                margin_end: 0.0,
            },
            Axis::Vertical => LineContext {
                end_pos: (child_cross_pos, child_main_pos),
                current_x: child_cross_pos,
                margin_start: 0.0,
                margin_end: 0.0,
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
            0,
            line_height,
            outbox_width,
        );

        if placement.reversed {
            placement.cursor_main -= placement.gap + placement.gap_between;
        } else {
            placement.cursor_main += item_main_size + placement.gap + placement.gap_between;
        }
    }

    fn position_flex_custom(
        &self,
        node: &mut LayoutNode,
        axis: Axis,
        ctx: &InternalLayoutContext,
        index: usize,
        placement: &mut FlexPlacementCtx,
    ) {
        let measured = {
            let object = node.children[index].custom_child().unwrap();
            if object.style().display.outer == OuterDisplay::None {
                return;
            }
            object.layouter().measure(&crate::LayoutContext::from(ctx))
        };
        let tuple = (measured.width, measured.height);
        let item_main_size = axis.tuple_main(tuple);
        let item_cross_size = axis.tuple_cross(tuple);

        if placement.reversed {
            placement.cursor_main -= item_main_size;
        }

        let item_main_pos = match axis {
            Axis::Horizontal => placement.content_box.x + placement.cursor_main,
            Axis::Vertical => placement.content_box.y + placement.cursor_main,
        };

        let available_cross = axis.rect_cross(&placement.content_box);
        let cross_offset =
            resolve_align_position(node.style.align_items, item_cross_size, available_cross);

        let item_cross_pos = match axis {
            Axis::Horizontal => {
                placement.content_box.y + placement.line_cross_offset + cross_offset
            }
            Axis::Vertical => placement.content_box.x + placement.line_cross_offset + cross_offset,
        };

        let (x, y) = match axis {
            Axis::Horizontal => (item_main_pos, item_cross_pos),
            Axis::Vertical => (item_cross_pos, item_main_pos),
        };
        let (box_w, box_h) = match axis {
            Axis::Horizontal => (item_main_size, item_cross_size),
            Axis::Vertical => (item_cross_size, item_main_size),
        };

        let mut box_model =
            create_box_model(box_w, box_h, box_w, box_h, Edge::default(), Edge::default());
        box_model.shift(x, y);

        if let LayoutChild::Custom(child) = &mut node.children[index] {
            child.set_result(CustomObjectResult {
                spans: Vec::new(),
                box_model,
            });
        }

        if placement.reversed {
            placement.cursor_main -= placement.gap + placement.gap_between;
        } else {
            placement.cursor_main += item_main_size + placement.gap + placement.gap_between;
        }
    }

    /// Set child positions.
    fn flow_flex_children(&self, node: &mut LayoutNode, axis: Axis, ctx: &InternalLayoutContext) {
        if node.children.is_empty() {
            return;
        }

        let content_box = match &node.layout_box {
            LayoutBox::BlockBox(box_model) => box_model.content_box,
            _ => return,
        };

        let gap = axis
            .gap(&node.style)
            .resolve_with(
                ctx.containing_block_main(axis),
                self.viewport_width,
                self.viewport_height,
            )
            .unwrap_or(0.0)
            .max(0.0);

        let items: Vec<_> = LayoutItems::new(&node.children)
            .filter(|item| item_participates_in_normal_flow(&node.children, item))
            .collect();

        let mut outer_main_sizes = Vec::with_capacity(items.len());
        for item in &items {
            outer_main_sizes.push(self.flex_children_main_total(
                node,
                axis,
                ctx,
                std::slice::from_ref(item),
            ));
        }
        let line_ranges = collect_flex_line_ranges(
            &outer_main_sizes,
            gap,
            Some(axis.rect_main(&content_box)),
            node.style.flex_wrap,
        );
        let cross_gap = axis
            .cross_gap(&node.style)
            .resolve_with(
                Some(axis.rect_cross(&content_box)),
                self.viewport_width,
                self.viewport_height,
            )
            .unwrap_or(0.0)
            .max(0.0);
        let mut line_cross_sizes: Vec<f32> = line_ranges
            .iter()
            .map(|range| self.flex_children_cross_max(node, axis, ctx, &items[range.clone()]))
            .collect();

        if node.style.flex_wrap == FlexWrap::NoWrap {
            line_cross_sizes[0] = axis.rect_cross(&content_box);
        }

        let cross_gaps_total = cross_gap * line_ranges.len().saturating_sub(1) as f32;
        let remaining_cross =
            axis.rect_cross(&content_box) - line_cross_sizes.iter().sum::<f32>() - cross_gaps_total;
        let (cross_start, cross_between, cross_stretch) =
            resolve_align_content(node.style.align_content, remaining_cross, line_ranges.len());
        if cross_stretch > 0.0 {
            for size in &mut line_cross_sizes {
                *size += cross_stretch;
            }
        }

        let reversed_main = Axis::is_reversed(&node.style.flex_direction);
        let mut cross_cursor = if node.style.flex_wrap == FlexWrap::WrapReverse {
            axis.rect_cross(&content_box) - cross_start
        } else {
            cross_start
        };

        for (line_index, range) in line_ranges.into_iter().enumerate() {
            let line_items = &items[range];
            let line_cross = line_cross_sizes[line_index];
            let line_cross_pos = if node.style.flex_wrap == FlexWrap::WrapReverse {
                cross_cursor -= line_cross;
                cross_cursor
            } else {
                cross_cursor
            };
            let line_box = match axis {
                Axis::Horizontal => Rect {
                    x: content_box.x,
                    y: content_box.y,
                    width: content_box.width,
                    height: line_cross,
                },
                Axis::Vertical => Rect {
                    x: content_box.x,
                    y: content_box.y,
                    width: line_cross,
                    height: content_box.height,
                },
            };

            let children_main_total = self.flex_children_main_total(node, axis, ctx, line_items);
            let gaps_total = gap * line_items.len().saturating_sub(1) as f32;
            let remaining_space = axis.rect_main(&line_box) - children_main_total - gaps_total;
            let auto_margin_count = self.count_main_axis_auto_margins(node, axis, line_items);
            let has_auto_margins = auto_margin_count > 0;
            let auto_unit = if has_auto_margins {
                remaining_space.max(0.0) / auto_margin_count as f32
            } else {
                0.0
            };
            let (start_offset, gap_between) = if has_auto_margins {
                (0.0, 0.0)
            } else {
                resolve_justify_content(
                    node.style.justify_content,
                    remaining_space,
                    line_items.len(),
                )
            };
            let use_reversed_alg =
                reversed_main && !matches!(node.style.justify_content, JustifyContent::End);
            let cursor_main = if use_reversed_alg {
                axis.rect_main(&line_box) - start_offset
            } else if reversed_main {
                0.0
            } else {
                start_offset
            };
            let mut placement = FlexPlacementCtx {
                content_box: line_box,
                line_cross_offset: line_cross_pos,
                cursor_main,
                auto_unit,
                gap_between,
                gap,
                reversed: use_reversed_alg,
            };

            for item in line_items.iter().cloned() {
                match item {
                    LayoutItem::Node(index) => {
                        self.position_flex_node_child(node, axis, ctx, index, &mut placement)
                    }
                    LayoutItem::Fragments(range) => {
                        self.position_flex_fragments(node, axis, range, &mut placement)
                    }
                    LayoutItem::Custom(index) => {
                        self.position_flex_custom(node, axis, ctx, index, &mut placement)
                    }
                }
            }

            if node.style.flex_wrap == FlexWrap::WrapReverse {
                cross_cursor -= cross_gap + cross_between;
            } else {
                cross_cursor += line_cross + cross_gap + cross_between;
            }
        }
    }

    fn layout_out_of_flow_node_children(
        &self,
        node: &mut LayoutNode,
        ctx: &InternalLayoutContext,
        intrinsic_pass: bool,
    ) {
        for child in &mut node.children {
            let LayoutChild::Node(child) = child else {
                continue;
            };
            if child.style.position.kind.is_out_of_flow() {
                let _ = self.layout_node(child, ctx, EMPTY_LINE_CONTEXT, intrinsic_pass);
            }
        }
    }

    /// Applies relative and out-of-flow offsets after normal sizing is complete.
    /// Coordinates passed to children are absolute solely for resolving a
    /// positioned descendant against an ancestor containing block; stored box
    /// coordinates remain relative to the direct parent's content box.
    fn apply_positioning(
        &self,
        node: &mut LayoutNode,
        parent_content_origin: (f32, f32),
        containing_block: Rect,
    ) {
        let Some(initial_box) = node.layout_box.iter().next() else {
            return;
        };
        let initial_border = initial_box.border_box;
        let border_size = initial_border.size();

        let offset_basis = if node.style.position.kind == Position::Fixed {
            (self.viewport_width, self.viewport_height)
        } else {
            (containing_block.width, containing_block.height)
        };
        let resolve_x = |value: &LengthOrAuto| {
            value.resolve_with(
                Some(offset_basis.0),
                self.viewport_width,
                self.viewport_height,
            )
        };
        let resolve_y = |value: &LengthOrAuto| {
            value.resolve_with(
                Some(offset_basis.1),
                self.viewport_width,
                self.viewport_height,
            )
        };

        let left = resolve_x(&node.style.position.left);
        let right = resolve_x(&node.style.position.right);
        let top = resolve_y(&node.style.position.top);
        let bottom = resolve_y(&node.style.position.bottom);

        if node.style.position.kind == Position::Sticky {
            let left = left.unwrap_or_default();
            let right = right.unwrap_or_default();
            let top = top.unwrap_or_default();
            let bottom = bottom.unwrap_or_default();

            node.layout_box.set_sticky_edges(Edge {
                left,
                top,
                right,
                bottom,
            });
        } else {
            let (dx, dy) = match node.style.position.kind {
                Position::Static => (0.0, 0.0),
                Position::Relative => (
                    left.unwrap_or_else(|| -right.unwrap_or_default()),
                    top.unwrap_or_else(|| -bottom.unwrap_or_default()),
                ),
                Position::Absolute | Position::Fixed => {
                    let target = if node.style.position.kind == Position::Fixed {
                        Rect {
                            x: 0.0,
                            y: 0.0,
                            width: self.viewport_width,
                            height: self.viewport_height,
                        }
                    } else {
                        containing_block
                    };
                    let target_x = if let Some(left) = left {
                        target.x + left
                    } else if let Some(right) = right {
                        target.right() - right - border_size.0
                    } else {
                        parent_content_origin.0 + initial_border.x
                    };
                    let target_y = if let Some(top) = top {
                        target.y + top
                    } else if let Some(bottom) = bottom {
                        target.bottom() - bottom - border_size.1
                    } else {
                        parent_content_origin.1 + initial_border.y
                    };
                    (
                        target_x - parent_content_origin.0 - initial_border.x,
                        target_y - parent_content_origin.1 - initial_border.y,
                    )
                }
                Position::Sticky => unreachable!(),
            };

            node.layout_box.shift(dx, dy);
        }

        let Some(positioned_box) = node.layout_box.iter().next() else {
            return;
        };
        let border = positioned_box.border_box;
        let border_origin = (
            parent_content_origin.0 + border.x,
            parent_content_origin.1 + border.y,
        );
        let content_origin = (
            border_origin.0 + positioned_box.content_box.x - border.x,
            border_origin.1 + positioned_box.content_box.y - border.y,
        );
        let child_containing_block = if node.style.position.kind == Position::Static {
            containing_block
        } else {
            Rect {
                x: border_origin.0 + positioned_box.padding_box.x - border.x,
                y: border_origin.1 + positioned_box.padding_box.y - border.y,
                width: positioned_box.padding_box.width,
                height: positioned_box.padding_box.height,
            }
        };

        for child in &mut node.children {
            if let LayoutChild::Node(child) = child {
                self.apply_positioning(child, content_origin, child_containing_block);
            }
        }
    }

    fn init_flex_item_states(
        &self,
        node: &mut LayoutNode,
        axis: Axis,
        base_ctx_for_children: &InternalLayoutContext,
        flex_items: &[LayoutItem],
        states: &mut [FlexItemState],
    ) -> f32 {
        let cbm = base_ctx_for_children.containing_block_main(axis);
        let vw = self.viewport_width;
        let vh = self.viewport_height;
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
                        Some(v) => {
                            if cbm.is_none() && node.style.item_style.flex_grow > 0.0 {
                                let _ = self.layout_node(
                                    node,
                                    base_ctx_for_children,
                                    EMPTY_LINE_CONTEXT,
                                    true,
                                );
                                let content_main =
                                    axis.rect_main(&layout_box_content_box(&node.layout_box));
                                v.max(content_main)
                            } else {
                                v
                            }
                        }
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
                                    axis.rect_main(&layout_box_content_box(&node.layout_box))
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
                        0,
                        line_height,
                        base_ctx_for_children.containing_block_width.unwrap_or(vw),
                    );

                    state.main_size = match axis {
                        Axis::Horizontal => fragment_width,
                        Axis::Vertical => fragment_height,
                    };
                }
                LayoutItem::Custom(index) => {
                    if let LayoutChild::Custom(child) = &mut node.children[*index] {
                        if child.style().display.outer == OuterDisplay::None {
                            state.main_size = 0.0;
                        } else {
                            let measured = child
                                .layouter_mut()
                                .measure(&crate::LayoutContext::from(base_ctx_for_children));
                            let tuple = (measured.width, measured.height);
                            state.main_size = axis.tuple_main(tuple);
                        }
                    }
                }
            }
        }

        total_grow
    }

    fn flex_grow_redistribution(
        &self,
        states: &mut [FlexItemState],
        flex_items: &[LayoutItem],
        remaining: &mut f32,
        item_len: usize,
        total_grow: &mut f32,
        node: &LayoutNode,
    ) {
        let mut used = 0.0;

        for i in 0..item_len {
            if states[i].frozen_grow {
                continue;
            }

            let item = &flex_items[i];
            let grow = states[i].grow;
            let delta = *remaining * (grow / *total_grow);
            let min = states[i].main_min;
            let max = states[i].main_max;
            let old_size = states[i].main_size;

            let clamped_content = clamp_flex_main_size(
                old_size,
                delta,
                min,
                max,
                flex_padding_border(&states[i]),
                flex_item_box_sizing(item, node),
            );

            let actual = clamped_content - old_size;
            states[i].main_size = clamped_content;
            used += actual;

            if (old_size + delta - clamped_content).abs() > EPSILON {
                states[i].frozen_grow = true;
                *total_grow -= grow;
            }
        }

        *remaining -= used;
    }

    fn flex_shrink_redistribution(
        &self,
        states: &mut [FlexItemState],
        flex_items: &[LayoutItem],
        remaining: &mut f32,
        item_len: usize,
        node: &LayoutNode,
    ) {
        let mut total_shrink_factor = 0.0;

        for state in &mut *states {
            if state.frozen_shrink {
                continue;
            }
            total_shrink_factor += state.shrink * state.main_size;
        }

        if total_shrink_factor <= 0.0 {
            return;
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
            let delta = *remaining * ratio;
            let new_size = states[i].main_size + delta;
            let min = states[i].main_min;
            let max = states[i].main_max;
            let old_size = states[i].main_size;

            let clamped_content = clamp_flex_main_size(
                old_size,
                delta,
                min,
                max,
                flex_padding_border(&states[i]),
                flex_item_box_sizing(item, node),
            );

            let actual = clamped_content - old_size;
            states[i].main_size = clamped_content;
            used += actual;

            if (clamped_content - new_size).abs() > EPSILON {
                states[i].frozen_shrink = true;
            }
        }

        *remaining -= used;
    }

    fn finalize_flex_children_layout(
        &self,
        node: &mut LayoutNode,
        axis: Axis,
        base_ctx_for_children: &InternalLayoutContext,
        intrinsic_pass: bool,
        flex_items: &[LayoutItem],
        states: Vec<FlexItemState>,
        line_cross_size: Option<f32>,
    ) -> (f32, f32) {
        let vw = self.viewport_width;
        let vh = self.viewport_height;
        let gap = axis
            .gap(&node.style)
            .resolve_with(base_ctx_for_children.containing_block_main(axis), vw, vh)
            .unwrap_or(0.0)
            .max(0.0);
        let gaps = gap * flex_items.len().saturating_sub(1) as f32;
        let mut total_border_main: f32 = 0.0;
        let mut max_cross: f32 = 0.0;

        for (item, state) in flex_items.iter().zip(states) {
            match item {
                LayoutItem::Node(index) => {
                    let child = node.children.get_mut(*index).unwrap().node_mut().unwrap();

                    let stretched_cross = self.compute_stretched_cross(
                        child,
                        axis,
                        line_cross_size,
                        vw,
                        vh,
                        &node.style,
                    );

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

                    let ctx_for_child = InternalLayoutContext {
                        parent_assigned_border_width,
                        parent_assigned_border_height,
                        ..*base_ctx_for_children
                    };

                    let _ =
                        self.layout_node(child, &ctx_for_child, EMPTY_LINE_CONTEXT, intrinsic_pass);

                    let margin = self
                        .resolve_margin(&child.style.spacing, base_ctx_for_children)
                        .unwrap_or_default();
                    let (main_margin_start, main_margin_end) = axis.edge_main(&margin);
                    let cross_margin = match axis {
                        Axis::Horizontal => margin.top + margin.bottom,
                        Axis::Vertical => margin.left + margin.right,
                    };

                    let tuple = (child.layout_box.width_box(), child.layout_box.height_box());
                    total_border_main +=
                        axis.tuple_main(tuple) + main_margin_start + main_margin_end;
                    max_cross = max_cross.max(axis.tuple_cross(tuple) + cross_margin);
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
                        0,
                        line_height,
                        base_ctx_for_children.containing_block_width.unwrap_or(vw),
                    );

                    total_border_main += axis.tuple_main((fragment_width, fragment_height));
                    max_cross = max_cross.max(axis.tuple_cross((fragment_width, fragment_height)));
                }
                LayoutItem::Custom(index) => {
                    let object = node
                        .children
                        .get_mut(*index)
                        .unwrap()
                        .custom_child()
                        .unwrap();
                    if object.style().display.outer == OuterDisplay::None {
                        continue;
                    }
                    let measured = object
                        .layouter()
                        .measure(&crate::LayoutContext::from(base_ctx_for_children));
                    let tuple = (measured.width, measured.height);
                    total_border_main += axis.tuple_main(tuple);
                    max_cross = max_cross.max(axis.tuple_cross(tuple));
                }
            }
        }

        (total_border_main + gaps, max_cross)
    }

    fn compute_stretched_cross(
        &self,
        child: &LayoutNode,
        axis: Axis,
        cbc: Option<f32>,
        vw: f32,
        vh: f32,
        parent_style: &Style,
    ) -> Option<f32> {
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
            .unwrap_or(parent_style.align_items);

        let is_auto_cross = axis.size_cross(&child.style.size) == &LengthOrAuto::Auto;

        if !is_auto_margin && matches!(align, AlignItems::Stretch) && is_auto_cross {
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
        }
    }

    fn flow_fragments(
        fragments: &mut Vec<&mut FragmentNode>,
        line_ctx: LineContext,
        line_index: usize,
        line_height: f32,
        outbox_width: f32,
    ) -> (Vec<LineSpan>, LineContext, usize) {
        let mut cursor_x = line_ctx.end_pos.0;
        let mut cursor_y = line_ctx.end_pos.1;

        let mut current_x = line_ctx.current_x;
        let mut line_start_x = line_ctx.current_x;
        let mut visual_line_start_x = cursor_x;

        let mut line_index = line_index;

        let mut if_first_of_line = true;

        let mut line_span_buf = Vec::new();

        for fragment_node in fragments {
            match fragment_node.node {
                ItemFragment::LineBreak => {
                    let span = LineSpan {
                        x_range: line_start_x..current_x,
                        line_pos: (visual_line_start_x, cursor_y),
                        line_index,
                    };

                    if line_span_buf.is_empty() {
                        line_span_buf.push(span);
                    } else {
                        push_or_merge_line_span(&mut line_span_buf, span);
                    }

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
                    // Wrap if fragment doesn't fit and line isn't physically empty (cursor_x > 0
                    // catches inline child mid-line after previous siblings).
                    if cursor_x + fragment_item.width > outbox_width
                        && (!if_first_of_line || cursor_x > 0.0)
                    {
                        if line_start_x != current_x {
                            push_or_merge_line_span(
                                &mut line_span_buf,
                                LineSpan {
                                    x_range: line_start_x..current_x,
                                    line_pos: (visual_line_start_x, cursor_y),
                                    line_index,
                                },
                            );
                        }

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
                current_x,
                margin_start: 0.0,
                margin_end: 0.0,
            },
            line_index,
        )
    }

    /// ((content_width_opt, content_height_opt), border, padding)
    fn resolve_base_content_size_and_spacing(
        &self,
        size_style: &crate::SizeStyle,
        spacing: &crate::Spacing,
        box_sizing: &BoxSizing,
        ctx: &InternalLayoutContext,
    ) -> ((Option<f32>, Option<f32>), Edge, Edge) {
        let border = self.resolve_border(spacing, ctx);
        let padding = self.resolve_padding(spacing, ctx);

        let vw = self.viewport_width;
        let vh = self.viewport_height;

        // --- width ---
        // Parent-assigned (flex) size takes priority over explicit size
        let content_width = ctx
            .parent_assigned_border_width
            .map(|v| (v - (padding.left + padding.right) - (border.left + border.right)).max(0.0))
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
            .map(|width| {
                self.apply_size_constraints(
                    width,
                    size_style,
                    ctx,
                    true,
                    box_sizing,
                    padding.left + padding.right + border.left + border.right,
                )
            });

        // --- height ---
        // Parent-assigned (flex) size takes priority over explicit size
        let content_height = ctx
            .parent_assigned_border_height
            .map(|v| (v - (padding.top + padding.bottom) - (border.top + border.bottom)).max(0.0))
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
            .map(|height| {
                self.apply_size_constraints(
                    height,
                    size_style,
                    ctx,
                    false,
                    box_sizing,
                    padding.top + padding.bottom + border.top + border.bottom,
                )
            });

        ((content_width, content_height), border, padding)
    }

    /// Applies min/max size constraints to a dimension value.
    fn apply_size_constraints(
        &self,
        value: f32,
        size_style: &crate::SizeStyle,
        ctx: &InternalLayoutContext,
        is_width: bool,
        box_sizing: &BoxSizing,
        padding_border_edge: f32,
    ) -> f32 {
        let containing_size = if is_width {
            ctx.containing_block_width
        } else {
            ctx.containing_block_height
        };

        let (min_constraint, max_constraint) = resolve_min_max(
            size_style,
            containing_size,
            self.viewport_width,
            self.viewport_height,
            is_width,
        );

        let (min_constraint, max_constraint) = match box_sizing {
            BoxSizing::BorderBox => (
                min_constraint.map(|m| (m - padding_border_edge).max(0.0)),
                max_constraint.map(|m| (m - padding_border_edge).max(0.0)),
            ),
            _ => (min_constraint, max_constraint),
        };

        clamp(value, min_constraint, max_constraint)
    }

    fn resolve_padding(&self, spacing: &Spacing, ctx: &InternalLayoutContext) -> Edge {
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

    fn resolve_border(&self, spacing: &Spacing, ctx: &InternalLayoutContext) -> Edge {
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

    fn resolve_margin(&self, spacing: &Spacing, ctx: &InternalLayoutContext) -> EdgeOption {
        let vw = self.viewport_width;
        let vh = self.viewport_height;

        EdgeOption {
            left: spacing
                .margin_left
                .resolve_with(ctx.containing_block_width, vw, vh),
            top: spacing
                .margin_top
                .resolve_with(ctx.containing_block_width, vw, vh),
            right: spacing
                .margin_right
                .resolve_with(ctx.containing_block_width, vw, vh),
            bottom: spacing
                .margin_bottom
                .resolve_with(ctx.containing_block_width, vw, vh),
        }
    }
}

// ==========================================

fn flex_state_outer_main_size(state: &FlexItemState) -> f32 {
    state.main_size
        + state.main_padding.0
        + state.main_padding.1
        + state.main_border.0
        + state.main_border.1
        + state.main_margin.0
        + state.main_margin.1
}

fn collect_flex_line_ranges(
    outer_main_sizes: &[f32],
    gap: f32,
    available_main: Option<f32>,
    flex_wrap: FlexWrap,
) -> Vec<std::ops::Range<usize>> {
    if outer_main_sizes.is_empty() {
        return Vec::new();
    }
    let Some(available_main) = available_main else {
        return vec![0..outer_main_sizes.len()];
    };
    if flex_wrap == FlexWrap::NoWrap {
        return vec![0..outer_main_sizes.len()];
    }

    let mut lines = Vec::new();
    let mut start = 0;
    let mut used = 0.0;
    for (index, size) in outer_main_sizes.iter().copied().enumerate() {
        let candidate = if index == start {
            size
        } else {
            used + gap + size
        };
        if index > start && candidate > available_main + EPSILON {
            lines.push(start..index);
            start = index;
            used = size;
        } else {
            used = candidate;
        }
    }
    lines.push(start..outer_main_sizes.len());
    lines
}

fn compute_flex_remaining(
    cbm: Option<f32>,
    states: &[FlexItemState],
    gap: f32,
    item_len: usize,
) -> f32 {
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
    let gaps = gap * item_len.saturating_sub(1) as f32;
    cbm.map(|m| {
        m - (total_base_main + gaps + total_main_padding + total_main_border + total_main_margin)
    })
    .unwrap_or(0.0)
}

fn build_grid_slots(
    node: &LayoutNode,
    items: Vec<LayoutItem>,
    explicit_column_count: usize,
) -> Vec<GridItemSlot> {
    let mut column_count = explicit_column_count.max(1);
    for item in &items {
        let (column, _) = grid_item_placements(node, item);
        if let Some(start) = column.start {
            column_count = column_count.max(start.saturating_sub(1) + column.span.max(1));
        }
    }

    let mut occupied: Vec<Vec<bool>> = Vec::new();
    let mut cursor = 0usize;
    let mut slots = Vec::with_capacity(items.len());
    for item in items {
        let (column, row) = grid_item_placements(node, &item);
        let column_span = column.span.max(1).min(column_count);
        let row_span = row.span.max(1);
        let explicit_column = column.start.map(|line| line.saturating_sub(1));
        let explicit_row = row.start.map(|line| line.saturating_sub(1));

        let (placed_row, placed_column) = if let (Some(row), Some(column)) =
            (explicit_row, explicit_column)
        {
            (row, column.min(column_count.saturating_sub(column_span)))
        } else {
            let mut candidate = explicit_row
                .map(|row| row * column_count)
                .or_else(|| {
                    explicit_column.map(|column| cursor / column_count * column_count + column)
                })
                .unwrap_or(cursor);
            loop {
                let candidate_row = candidate / column_count;
                let candidate_column = explicit_column.unwrap_or(candidate % column_count);
                let row_matches = explicit_row.is_none_or(|row| row == candidate_row);
                if row_matches
                    && candidate_column + column_span <= column_count
                    && grid_cells_available(
                        &occupied,
                        candidate_row,
                        candidate_column,
                        row_span,
                        column_span,
                    )
                {
                    break (candidate_row, candidate_column);
                }
                candidate = if explicit_column.is_some() {
                    (candidate_row + 1) * column_count + explicit_column.unwrap_or(0)
                } else {
                    candidate + 1
                };
                if explicit_row.is_some() && candidate / column_count > explicit_row.unwrap_or(0) {
                    break (
                        explicit_row.unwrap_or(0),
                        explicit_column
                            .unwrap_or(0)
                            .min(column_count.saturating_sub(column_span)),
                    );
                }
            }
        };

        mark_grid_cells(
            &mut occupied,
            placed_row,
            placed_column,
            row_span,
            column_span,
            column_count,
        );
        cursor = cursor.max(placed_row * column_count + placed_column + column_span);
        slots.push(GridItemSlot {
            item,
            column: placed_column,
            row: placed_row,
            column_span,
            row_span,
            intrinsic_width: 0.0,
            intrinsic_height: 0.0,
        });
    }
    slots
}

fn grid_item_placements(node: &LayoutNode, item: &LayoutItem) -> (GridPlacement, GridPlacement) {
    let style = match item {
        LayoutItem::Node(index) => node.children[*index].node().map(|child| &child.style),
        LayoutItem::Custom(index) => node.children[*index]
            .custom_child()
            .map(|child| child.style()),
        LayoutItem::Fragments(_) => None,
    };
    let Some(style) = style else {
        return Default::default();
    };
    if let Some(area) = style.grid_area.as_deref()
        && let Some(placement) = named_grid_area(&node.style.grid_template_areas, area)
    {
        return placement;
    }
    (style.grid_column, style.grid_row)
}

fn named_grid_area(areas: &[Vec<String>], name: &str) -> Option<(GridPlacement, GridPlacement)> {
    let mut min_column = usize::MAX;
    let mut max_column = 0;
    let mut min_row = usize::MAX;
    let mut max_row = 0;
    for (row, names) in areas.iter().enumerate() {
        for (column, area) in names.iter().enumerate() {
            if area == name {
                min_column = min_column.min(column);
                max_column = max_column.max(column);
                min_row = min_row.min(row);
                max_row = max_row.max(row);
            }
        }
    }
    (min_column != usize::MAX).then_some((
        GridPlacement {
            start: Some(min_column + 1),
            span: max_column - min_column + 1,
        },
        GridPlacement {
            start: Some(min_row + 1),
            span: max_row - min_row + 1,
        },
    ))
}

fn grid_cells_available(
    occupied: &[Vec<bool>],
    row: usize,
    column: usize,
    row_span: usize,
    column_span: usize,
) -> bool {
    (row..row + row_span).all(|r| {
        (column..column + column_span).all(|c| {
            !occupied
                .get(r)
                .and_then(|columns| columns.get(c))
                .copied()
                .unwrap_or(false)
        })
    })
}

fn mark_grid_cells(
    occupied: &mut Vec<Vec<bool>>,
    row: usize,
    column: usize,
    row_span: usize,
    column_span: usize,
    column_count: usize,
) {
    occupied.resize_with(row + row_span, || vec![false; column_count]);
    for columns in occupied.iter_mut().skip(row).take(row_span) {
        for cell in columns.iter_mut().skip(column).take(column_span) {
            *cell = true;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn expand_grid_tracks(
    tracks: &[GridTrack],
    available: Option<f32>,
    gap: f32,
    item_count: usize,
    viewport_width: f32,
    viewport_height: f32,
) -> Vec<GridTrack> {
    let mut expanded = Vec::new();
    for track in tracks {
        let GridTrack::Repeat(repeat, pattern) = track else {
            expanded.push(track.clone());
            continue;
        };
        if pattern.is_empty() {
            continue;
        }
        let count = match repeat {
            GridRepeat::Count(count) => *count,
            GridRepeat::AutoFit | GridRepeat::AutoFill => {
                let pattern_width = pattern
                    .iter()
                    .map(|track| {
                        grid_track_minimum(track, available, viewport_width, viewport_height)
                    })
                    .sum::<f32>()
                    + gap * pattern.len().saturating_sub(1) as f32;
                let fit = if pattern_width <= EPSILON {
                    1
                } else {
                    available
                        .map(|available| {
                            ((available + gap) / (pattern_width + gap)).floor() as usize
                        })
                        .unwrap_or(1)
                        .max(1)
                };
                match repeat {
                    GridRepeat::AutoFit => fit.min(item_count.div_ceil(pattern.len()).max(1)),
                    GridRepeat::AutoFill => fit,
                    GridRepeat::Count(_) => unreachable!(),
                }
            }
        }
        .min(MAX_GRID_TRACKS / pattern.len());
        for _ in 0..count {
            expanded.extend(pattern.iter().cloned());
        }
    }
    expanded
}

fn grid_track_minimum(
    track: &GridTrack,
    available: Option<f32>,
    viewport_width: f32,
    viewport_height: f32,
) -> f32 {
    match track {
        GridTrack::Breadth(breadth) => breadth
            .resolve_with(available, viewport_width, viewport_height)
            .unwrap_or(0.0)
            .max(0.0),
        GridTrack::Flex(_) => 0.0,
        GridTrack::MinMax(minimum, _) => {
            grid_track_minimum(minimum, available, viewport_width, viewport_height)
        }
        GridTrack::Repeat(_, pattern) => pattern
            .iter()
            .map(|track| grid_track_minimum(track, available, viewport_width, viewport_height))
            .sum(),
    }
}

fn grid_track_flex_factor(track: &GridTrack) -> f32 {
    match track {
        GridTrack::Flex(factor) => factor.max(0.0),
        GridTrack::MinMax(_, maximum) => grid_track_flex_factor(maximum),
        _ => 0.0,
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_grid_tracks(
    explicit: &[GridTrack],
    count: usize,
    available: Option<f32>,
    gap: f32,
    intrinsic: &[f32],
    viewport_width: f32,
    viewport_height: f32,
) -> Vec<f32> {
    let mut sizes = vec![0.0; count];
    let mut flex_total = 0.0;
    for index in 0..count {
        match explicit.get(index).cloned().unwrap_or_default() {
            GridTrack::Breadth(breadth) => {
                sizes[index] = breadth
                    .resolve_with(available, viewport_width, viewport_height)
                    .unwrap_or(intrinsic[index])
                    .max(0.0);
            }
            GridTrack::Flex(factor) => {
                flex_total += factor.max(0.0);
                if available.is_none() {
                    sizes[index] = intrinsic[index];
                }
            }
            GridTrack::MinMax(minimum, maximum) => {
                let minimum =
                    grid_track_minimum(&minimum, available, viewport_width, viewport_height);
                let factor = grid_track_flex_factor(&maximum);
                flex_total += factor;
                sizes[index] = minimum.max(if available.is_none() {
                    intrinsic[index]
                } else {
                    0.0
                });
                if factor == 0.0 {
                    sizes[index] = match maximum.as_ref() {
                        GridTrack::Breadth(breadth) => breadth
                            .resolve_with(available, viewport_width, viewport_height)
                            .unwrap_or(intrinsic[index]),
                        _ => intrinsic[index],
                    }
                    .max(minimum);
                }
            }
            GridTrack::Repeat(_, _) => unreachable!("grid repeats must be expanded first"),
        }
    }

    if let Some(available) = available
        && flex_total > 0.0
    {
        let gaps = gap * count.saturating_sub(1) as f32;
        let remaining = (available - gaps - sizes.iter().sum::<f32>()).max(0.0);
        for (index, size) in sizes.iter_mut().enumerate() {
            if let Some(track) = explicit.get(index) {
                let factor = grid_track_flex_factor(track);
                if factor > 0.0 {
                    *size += remaining * factor / flex_total;
                }
            }
        }
    }
    sizes
}

fn flex_item_box_sizing(item: &LayoutItem, node: &LayoutNode) -> Option<BoxSizing> {
    if let LayoutItem::Node(index) = item {
        Some(node.children[*index].node().unwrap().style.box_sizing)
    } else {
        None
    }
}

fn flex_padding_border(state: &FlexItemState) -> f32 {
    state.main_padding.0 + state.main_padding.1 + state.main_border.0 + state.main_border.1
}

fn resolve_min_max(
    size_style: &crate::SizeStyle,
    containing_size: Option<f32>,
    vw: f32,
    vh: f32,
    is_width: bool,
) -> (Option<f32>, Option<f32>) {
    if is_width {
        (
            size_style.min_width.resolve_with(containing_size, vw, vh),
            size_style.max_width.resolve_with(containing_size, vw, vh),
        )
    } else {
        (
            size_style.min_height.resolve_with(containing_size, vw, vh),
            size_style.max_height.resolve_with(containing_size, vw, vh),
        )
    }
}

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

/// Returns the content box of a laid-out box, regardless of whether it was
/// produced as a block or an inline box.
///
/// Flex item measurement needs the intrinsic main size of the child; an
/// inline-level child (e.g. `display: inline-block`) yields an
/// [`LayoutBox::InlineBox`], so reading only the block branch collapses it to
/// zero.
fn layout_box_content_box(layout_box: &LayoutBox) -> Rect {
    match layout_box {
        LayoutBox::BlockBox(b) => b.content_box,
        LayoutBox::InlineBox(l) => l.box_model.content_box,
        LayoutBox::None => Rect::default(),
    }
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
        None => clamp(proposed_content, min, max),
    }
}

fn item_participates_in_normal_flow(children: &[LayoutChild], item: &LayoutItem) -> bool {
    match item {
        LayoutItem::Node(index) => !children[*index]
            .node()
            .unwrap()
            .style
            .position
            .kind
            .is_out_of_flow(),
        LayoutItem::Fragments(_) | LayoutItem::Custom(_) => true,
    }
}

struct LayoutItems<'a> {
    children: &'a [LayoutChild],
    index: usize,
}

impl<'a> LayoutItems<'a> {
    fn new(children: &'a [LayoutChild]) -> Self {
        Self { children, index: 0 }
    }
}

impl<'a> Iterator for LayoutItems<'a> {
    type Item = LayoutItem;

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.index;
        if i >= self.children.len() {
            return None;
        }
        match &self.children[i] {
            LayoutChild::Node(_) => {
                self.index = i + 1;
                Some(LayoutItem::Node(i))
            }
            LayoutChild::Fragment(_) => {
                let start = i;
                let mut end = i + 1;
                while end < self.children.len()
                    && matches!(self.children[end], LayoutChild::Fragment(_))
                {
                    end += 1;
                }
                self.index = end;
                Some(LayoutItem::Fragments(start..end))
            }
            LayoutChild::Custom(_) => {
                self.index = i + 1;
                Some(LayoutItem::Custom(i))
            }
        }
    }
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
    line_index: usize,
    line_height: f32,
    outbox_width: f32,
) -> (f32, f32, LineContext) {
    let mut fragment_node_buffer = children[range]
        .iter_mut()
        .filter_map(|child| child.fragment_mut())
        .collect();

    let (line_spans, line_ctx, _) = LayoutEngine::flow_fragments(
        &mut fragment_node_buffer,
        line_ctx,
        line_index,
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
    if let Some(last) = spans.last_mut()
        && last.line_pos.1 == span.line_pos.1
        && last.x_range.end >= span.x_range.start
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
        sticky_edges: None,
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

fn resolve_align_content(
    align: AlignContent,
    remaining_space: f32,
    lines: usize,
) -> (f32, f32, f32) {
    match align {
        AlignContent::Start => (0.0, 0.0, 0.0),
        AlignContent::Center => (remaining_space / 2.0, 0.0, 0.0),
        AlignContent::End => (remaining_space, 0.0, 0.0),
        AlignContent::SpaceBetween if lines > 1 && remaining_space > 0.0 => {
            (0.0, remaining_space / (lines - 1) as f32, 0.0)
        }
        AlignContent::SpaceAround if lines > 0 && remaining_space > 0.0 => {
            let between = remaining_space / lines as f32;
            (between / 2.0, between, 0.0)
        }
        AlignContent::SpaceEvenly if lines > 0 && remaining_space > 0.0 => {
            let between = remaining_space / (lines + 1) as f32;
            (between, between, 0.0)
        }
        AlignContent::Stretch if lines > 0 && remaining_space > 0.0 => {
            (0.0, 0.0, remaining_space / lines as f32)
        }
        AlignContent::SpaceBetween
        | AlignContent::SpaceAround
        | AlignContent::SpaceEvenly
        | AlignContent::Stretch => (0.0, 0.0, 0.0),
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

fn resolve_flow_margin_auto(
    ml_opt: Option<f32>,
    mr_opt: Option<f32>,
    content_width_opt: Option<f32>,
    child_node: &LayoutNode,
    child_is_block: bool,
) -> (f32, f32) {
    if child_is_block {
        let child_width = child_node.layout_box.width_box();
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
    }
}

fn collect_inline_spans_from_child(
    child_node: &LayoutNode,
    line_ctx_for_child: LineContext,
    line_span_buf: &mut Vec<LineSpan>,
) {
    if let LayoutBox::InlineBox(child_inline) = &child_node.layout_box {
        let x_offset = line_ctx_for_child.current_x;

        for child_span in &child_inline.line_spans {
            push_or_merge_line_span(
                line_span_buf,
                LineSpan {
                    x_range: (child_span.x_range.start + x_offset)
                        ..(child_span.x_range.end + x_offset),
                    line_pos: (child_span.line_pos.0 + x_offset, child_span.line_pos.1),
                    line_index: child_span.line_index,
                },
            );
        }
    }
}

fn compute_child_layout_extent(child_node: &LayoutNode) -> (f32, f32) {
    child_node
        .layout_box
        .iter()
        .map(|box_model| (box_model.border_box.right(), box_model.border_box.bottom()))
        .fold((0.0_f32, 0.0_f32), |acc, extent| {
            (acc.0.max(extent.0), acc.1.max(extent.1))
        })
}

fn compute_inline_extent(
    child_node: &LayoutNode,
    child_is_block: bool,
    child_bottom: f32,
    cursor_y: f32,
    previous_child_margin: f32,
    max_inline_line_height: f32,
) -> f32 {
    if child_is_block {
        child_bottom + previous_child_margin
    } else if cursor_y > child_bottom {
        cursor_y
    } else if cursor_y == child_bottom {
        let has_line_spans = match &child_node.layout_box {
            LayoutBox::InlineBox(b) => !b.line_spans.is_empty(),
            _ => false,
        };
        if has_line_spans {
            cursor_y + max_inline_line_height
        } else {
            child_bottom
        }
    } else {
        child_bottom
    }
}

/// Resolves the content-box size for a custom/replaced element.
///
/// Takes the resolved CSS width/height (None = auto), the element's intrinsic size,
/// and the aspect ratio constraint, and returns the final content-box size.
///
/// Handles:
/// - Box-sizing: for BorderBox, the CSS size already includes padding/border
/// - Min/max constraints (absolute lengths only; percentages are skipped without containing block)
/// - Aspect ratio: when only one axis is specified, derive the other
///
/// Note: This is a helper for engines that manage custom node layout.
/// The engine is responsible for providing the viewport dimensions and intrinsic size.
pub fn resolve_custom_box_size(
    style: &Style,
    intrinsic_width: f32,
    intrinsic_height: f32,
    aspect_ratio: Option<f32>,
    containing_block_width: Option<f32>,
    containing_block_height: Option<f32>,
    viewport_width: f32,
    viewport_height: f32,
) -> (f32, f32) {
    // Resolve CSS width/height
    let resolved_width =
        style
            .size
            .width
            .resolve_with(containing_block_width, viewport_width, viewport_height);
    let resolved_height =
        style
            .size
            .height
            .resolve_with(containing_block_height, viewport_width, viewport_height);
    // Whether each axis was given an explicit CSS size (before any
    // aspect-ratio derivation). An axis left `auto` is flexible and must
    // absorb min/max clamping so the aspect ratio survives.
    let width_specified = resolved_width.is_some();
    let height_specified = resolved_height.is_some();

    // Convert to content-box based on box-sizing
    let pb_h = style
        .spacing
        .padding_left
        .resolve_with(containing_block_width, viewport_width, viewport_height)
        .unwrap_or(0.0)
        + style
            .spacing
            .padding_right
            .resolve_with(containing_block_width, viewport_width, viewport_height)
            .unwrap_or(0.0)
        + style
            .spacing
            .border_left
            .resolve_with(containing_block_width, viewport_width, viewport_height)
            .unwrap_or(0.0)
        + style
            .spacing
            .border_right
            .resolve_with(containing_block_width, viewport_width, viewport_height)
            .unwrap_or(0.0);

    let pb_v = style
        .spacing
        .padding_top
        .resolve_with(containing_block_height, viewport_width, viewport_height)
        .unwrap_or(0.0)
        + style
            .spacing
            .padding_bottom
            .resolve_with(containing_block_height, viewport_width, viewport_height)
            .unwrap_or(0.0)
        + style
            .spacing
            .border_top
            .resolve_with(containing_block_height, viewport_width, viewport_height)
            .unwrap_or(0.0)
        + style
            .spacing
            .border_bottom
            .resolve_with(containing_block_height, viewport_width, viewport_height)
            .unwrap_or(0.0);

    let content_width = resolved_width.map(|w| match style.box_sizing {
        BoxSizing::ContentBox => w,
        BoxSizing::BorderBox => (w - pb_h).max(0.0),
    });
    let content_height = resolved_height.map(|h| match style.box_sizing {
        BoxSizing::ContentBox => h,
        BoxSizing::BorderBox => (h - pb_v).max(0.0),
    });

    // Apply aspect ratio (ratio = width / height). Derives the missing axis
    // when exactly one of width / height is specified.
    let (content_width, content_height) = if let Some(ratio) = aspect_ratio {
        if ratio > 0.0 {
            match (content_width, content_height) {
                (Some(w), None) => (Some(w), Some(w / ratio)),
                (None, Some(h)) => (Some(h * ratio), Some(h)),
                _ => (content_width, content_height),
            }
        } else {
            (content_width, content_height)
        }
    } else {
        (content_width, content_height)
    };

    // Fall back to intrinsic size
    let (mut width, mut height) = (
        content_width.unwrap_or(intrinsic_width),
        content_height.unwrap_or(intrinsic_height),
    );

    // Apply min/max constraints.
    //
    // When the element has an aspect ratio and at least one axis is
    // flexible (`auto`), the constraints are solved while preserving the
    // ratio (CSS 2.1 §10.4): clamping one axis re-derives the other, so
    // `img { max-width: 100% }` keeps its proportions. When both axes are
    // explicitly sized, constraints apply independently and the user's
    // dimensions win (matching browsers).
    let preserves_ratio =
        aspect_ratio.is_some_and(|r| r > 0.0) && (!width_specified || !height_specified);

    if preserves_ratio {
        if let Some(max_w) = style.size.max_width.resolve_with(
            containing_block_width,
            viewport_width,
            viewport_height,
        ) && width > max_w
        {
            width = max_w;
            height = width / aspect_ratio.unwrap();
            if let Some(min_h) = style.size.min_height.resolve_with(
                containing_block_height,
                viewport_width,
                viewport_height,
            ) && height < min_h
            {
                height = min_h;
                width = height * aspect_ratio.unwrap();
            }
        }
        if let Some(max_h) = style.size.max_height.resolve_with(
            containing_block_height,
            viewport_width,
            viewport_height,
        ) && height > max_h
        {
            height = max_h;
            width = height * aspect_ratio.unwrap();
            if let Some(min_w) = style.size.min_width.resolve_with(
                containing_block_width,
                viewport_width,
                viewport_height,
            ) && width < min_w
            {
                width = min_w;
                height = width / aspect_ratio.unwrap();
            }
        }
        if let Some(min_w) = style.size.min_width.resolve_with(
            containing_block_width,
            viewport_width,
            viewport_height,
        ) && width < min_w
        {
            width = min_w;
            height = width / aspect_ratio.unwrap();
            if let Some(max_h) = style.size.max_height.resolve_with(
                containing_block_height,
                viewport_width,
                viewport_height,
            ) && height > max_h
            {
                height = max_h;
                width = height * aspect_ratio.unwrap();
            }
        }
        if let Some(min_h) = style.size.min_height.resolve_with(
            containing_block_height,
            viewport_width,
            viewport_height,
        ) && height < min_h
        {
            height = min_h;
            width = height * aspect_ratio.unwrap();
            if let Some(max_w) = style.size.max_width.resolve_with(
                containing_block_width,
                viewport_width,
                viewport_height,
            ) && width > max_w
            {
                width = max_w;
                height = width / aspect_ratio.unwrap();
            }
        }
    } else {
        if let Some(min_w) = style.size.min_width.resolve_with(
            containing_block_width,
            viewport_width,
            viewport_height,
        ) {
            width = width.max(min_w);
        }
        if let Some(max_w) = style.size.max_width.resolve_with(
            containing_block_width,
            viewport_width,
            viewport_height,
        ) {
            width = width.min(max_w);
        }
        if let Some(min_h) = style.size.min_height.resolve_with(
            containing_block_height,
            viewport_width,
            viewport_height,
        ) {
            height = height.max(min_h);
        }
        if let Some(max_h) = style.size.max_height.resolve_with(
            containing_block_height,
            viewport_width,
            viewport_height,
        ) {
            height = height.min(max_h);
        }
    }

    (width, height)
}
