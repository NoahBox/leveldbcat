use super::scrollbars::ScrollbarTheme;
use gpui::{
    Bounds, Entity, FocusHandle, PaintQuad, Pixels, ScrollHandle, SharedString, WrappedLine,
};
use std::{cell::RefCell, ops::Range, rc::Rc};

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
    wrap_lines: bool,
    scroll_handle: ScrollHandle,
    last_scroll_offset: Rc<RefCell<gpui::Point<Pixels>>>,
    scrollbar_theme: ScrollbarTheme,
    selected_range: Range<usize>,
    selection_reversed: bool,
    last_lines: Vec<LineState>,
    last_line_height: Pixels,
    is_selecting: bool,
}

#[derive(Clone)]
struct LineState {
    start: usize,
    end: usize,
    layout: WrappedLine,
    bounds: Option<Bounds<Pixels>>,
}

#[derive(Default, Clone)]
struct TextLayoutState(Rc<RefCell<Option<MeasuredTextLayout>>>);

#[derive(Clone)]
struct MeasuredTextLayout {
    lines: Vec<LineState>,
    line_height: Pixels,
}

#[derive(Clone, Copy)]
struct WrappedRow {
    start: usize,
    end: usize,
    start_x: Pixels,
}

struct SelectableTextElement {
    view: Entity<SelectableTextView>,
}

struct PrepaintState {
    lines: Vec<LineState>,
    selections: Vec<PaintQuad>,
    cursor: Option<PaintQuad>,
    line_height: Pixels,
}

fn wrapped_rows(layout: &WrappedLine) -> Vec<WrappedRow> {
    let mut rows = Vec::new();
    let mut start = 0;
    let mut start_x = Pixels::ZERO;

    for boundary in layout.wrap_boundaries() {
        let run = &layout.unwrapped_layout.runs[boundary.run_ix];
        let glyph = &run.glyphs[boundary.glyph_ix];
        rows.push(WrappedRow {
            start,
            end: glyph.index,
            start_x,
        });
        start = glyph.index;
        start_x = glyph.position.x;
    }

    rows.push(WrappedRow {
        start,
        end: layout.len(),
        start_x,
    });

    rows
}

fn local_position_for_index(
    line: &LineState,
    index: usize,
    line_height: Pixels,
    prefer_next_row_on_boundary: bool,
) -> Option<gpui::Point<Pixels>> {
    let local_index = index.checked_sub(line.start)?;
    let rows = wrapped_rows(&line.layout);

    for (row_index, row) in rows.iter().enumerate() {
        let is_last_row = row_index + 1 == rows.len();
        let belongs_to_row = local_index >= row.start
            && (local_index < row.end
                || local_index == row.end
                    && (!prefer_next_row_on_boundary || is_last_row || local_index == row.start));

        if belongs_to_row {
            let x = if local_index == row.start {
                Pixels::ZERO
            } else {
                line.layout.unwrapped_layout.x_for_index(local_index) - row.start_x
            };
            return Some(gpui::point(x, line_height * row_index as f32));
        }
    }

    rows.last().map(|row| {
        gpui::point(
            line.layout
                .unwrapped_layout
                .x_for_index(local_index.min(line.layout.len()))
                - row.start_x,
            line_height * rows.len().saturating_sub(1) as f32,
        )
    })
}
