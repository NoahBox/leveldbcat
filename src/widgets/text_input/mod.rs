use gpui::{Bounds, Entity, FocusHandle, PaintQuad, ShapedLine, SharedString, actions};
use std::ops::Range;

actions!(
    search_input,
    [
        Backspace, Delete, Left, Right, SelectAll, Home, End, Paste, Copy
    ]
);

mod behavior;
mod element;

pub struct TextInputView {
    focus_handle: FocusHandle,
    content: SharedString,
    placeholder: SharedString,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<gpui::Pixels>>,
    is_selecting: bool,
}

struct TextElement {
    input: Entity<TextInputView>,
}

struct PrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}
