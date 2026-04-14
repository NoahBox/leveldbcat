use super::scrollbars::ScrollbarTheme;
use gpui::{
    Bounds, Entity, FocusHandle, PaintQuad, Pixels, ScrollHandle, ShapedLine, SharedString,
};
use std::ops::Range;

mod behavior;
mod element;
mod highlight;

#[derive(Clone, Copy)]
pub struct JsonHighlightColors {
    pub default: gpui::Hsla,
    pub key: gpui::Hsla,
    pub string: gpui::Hsla,
    pub number: gpui::Hsla,
    pub keyword: gpui::Hsla,
    pub punctuation: gpui::Hsla,
}

#[derive(Clone, Copy)]
pub enum HighlightMode {
    Plain,
    Json(JsonHighlightColors),
}

pub struct SelectableTextView {
    focus_handle: FocusHandle,
    content: SharedString,
    highlight_mode: HighlightMode,
    scroll_handle: ScrollHandle,
    scrollbar_theme: ScrollbarTheme,
    selected_range: Range<usize>,
    selection_reversed: bool,
    last_lines: Vec<LineState>,
    last_bounds: Option<Bounds<Pixels>>,
    is_selecting: bool,
}

#[derive(Clone)]
struct LineState {
    start: usize,
    end: usize,
    layout: ShapedLine,
}

struct SelectableTextElement {
    view: Entity<SelectableTextView>,
}

struct PrepaintState {
    lines: Vec<LineState>,
    selections: Vec<PaintQuad>,
    cursor: Option<PaintQuad>,
}
