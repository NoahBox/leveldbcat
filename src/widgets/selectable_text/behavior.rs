use super::{HighlightMode, SelectableTextElement, SelectableTextView};
use gpui::{
    App, Bounds, Context, CursorStyle, EntityInputHandler, FocusHandle, Focusable, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Point, UTF16Selection, Window, div, point,
    prelude::*, px,
};
use std::{cell::RefCell, ops::Range, rc::Rc};

use crate::widgets::scrollbars::{ScrollbarAxis, Scrollbars, WheelScrollMode};

impl SelectableTextView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: gpui::SharedString::new_static(""),
            highlight_mode: HighlightMode::Plain,
            wrap_lines: false,
            scroll_handle: gpui::ScrollHandle::new(),
            last_scroll_offset: Rc::new(RefCell::new(point(px(0.0), px(0.0)))),
            scrollbar_theme: Default::default(),
            selected_range: 0..0,
            selection_reversed: false,
            last_lines: Vec::new(),
            last_line_height: px(0.0),
            is_selecting: false,
        }
    }

    pub fn set_text(&mut self, text: impl Into<gpui::SharedString>, cx: &mut Context<Self>) {
        self.set_text_with_highlight(text, HighlightMode::Plain, cx);
    }

    pub fn set_text_with_highlight(
        &mut self,
        text: impl Into<gpui::SharedString>,
        highlight_mode: HighlightMode,
        cx: &mut Context<Self>,
    ) {
        self.content = text.into();
        self.highlight_mode = highlight_mode;
        self.scroll_handle.set_offset(point(px(0.0), px(0.0)));
        *self.last_scroll_offset.borrow_mut() = point(px(0.0), px(0.0));
        self.selected_range = 0..0;
        self.selection_reversed = false;
        self.last_lines.clear();
        self.last_line_height = px(0.0);
        self.is_selecting = false;
        cx.notify();
    }

    pub fn set_scrollbar_theme(&mut self, theme: crate::widgets::scrollbars::ScrollbarTheme) {
        self.scrollbar_theme = theme;
    }

    pub fn wrap_lines(&self) -> bool {
        self.wrap_lines
    }

    pub fn set_wrap_lines(&mut self, wrap_lines: bool, cx: &mut Context<Self>) {
        if self.wrap_lines == wrap_lines {
            return;
        }

        self.wrap_lines = wrap_lines;
        self.last_lines.clear();
        self.last_line_height = px(0.0);

        if wrap_lines {
            let mut offset = self.scroll_handle.offset();
            offset.x = px(0.0);
            self.scroll_handle.set_offset(offset);
            *self.last_scroll_offset.borrow_mut() = offset;
        }

        cx.notify();
    }

    pub fn selected_text(&self) -> Option<String> {
        (!self.selected_range.is_empty())
            .then(|| self.content[self.selected_range.clone()].to_string())
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }

        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }

        cx.notify();
    }

    pub(super) fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn index_for_mouse_position(&self, position: Point<gpui::Pixels>) -> usize {
        let Some(first_line) = self.last_lines.first() else {
            return 0;
        };
        let Some(first_bounds) = first_line.bounds else {
            return 0;
        };

        if position.y <= first_bounds.top() || self.last_line_height <= px(0.0) {
            return 0;
        }

        for line in &self.last_lines {
            let Some(bounds) = line.bounds else {
                continue;
            };

            if position.y > bounds.bottom() {
                continue;
            }

            let local_position = position - bounds.origin;
            let local_index = match line
                .layout
                .closest_index_for_position(local_position, self.last_line_height)
            {
                Ok(index) | Err(index) => index,
            };

            return line.start + local_index;
        }

        self.content.len()
    }

    fn global_position_for_index(
        &self,
        index: usize,
        prefer_next_row_on_boundary: bool,
    ) -> Option<Point<gpui::Pixels>> {
        for line in &self.last_lines {
            let Some(bounds) = line.bounds else {
                continue;
            };

            if index < line.start || index > line.end {
                continue;
            }

            let local = super::local_position_for_index(
                line,
                index,
                self.last_line_height,
                prefer_next_row_on_boundary,
            )?;
            return Some(bounds.origin + local);
        }

        None
    }

    fn line_index_for_offset(&self, offset: usize) -> Option<usize> {
        self.last_lines
            .iter()
            .position(|line| offset >= line.start && offset <= line.end)
    }

    fn selection_bounds(&self, range: &Range<usize>) -> Option<Bounds<gpui::Pixels>> {
        if self.last_line_height <= px(0.0) {
            return None;
        }

        let start_line = self.line_index_for_offset(range.start)?;
        let end_line = self.line_index_for_offset(range.end)?;
        let start = self.global_position_for_index(range.start, true)?;
        let end = self.global_position_for_index(range.end, false)?;
        let end_x = if start_line == end_line {
            end.x.max(start.x + px(1.0))
        } else {
            end.x
        };

        Some(Bounds::from_corners(
            start,
            gpui::point(end_x, end.y + self.last_line_height),
        ))
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.is_selecting = true;
        window.focus(&self.focus_handle);

        if event.modifiers.shift {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        } else {
            self.move_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_selecting {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;

        for ch in self.content.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }

        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;

        for ch in self.content.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }

        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }
}

impl EntityInputHandler for SelectableTextView {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.content[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        None
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {}

    fn replace_text_in_range(
        &mut self,
        _range_utf16: Option<Range<usize>>,
        _new_text: &str,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        _range_utf16: Option<Range<usize>>,
        _new_text: &str,
        _new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _bounds: Bounds<gpui::Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<gpui::Pixels>> {
        let range = self.range_from_utf16(&range_utf16);
        self.selection_bounds(&range)
    }

    fn character_index_for_point(
        &mut self,
        point: Point<gpui::Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        Some(self.offset_to_utf16(self.index_for_mouse_position(point)))
    }
}

impl Focusable for SelectableTextView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui::Render for SelectableTextView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let scroll_handle = self.scroll_handle.clone();
        let last_scroll_offset = self.last_scroll_offset.clone();
        let view_id = cx.entity_id();

        *self.last_scroll_offset.borrow_mut() = self.scroll_handle.offset();

        div()
            .size_full()
            .relative()
            .track_focus(&self.focus_handle(cx))
            .child(
                div()
                    .id("selectable-text-scroll")
                    .size_full()
                    .overflow_x_scroll()
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll_handle)
                    .cursor(CursorStyle::IBeam)
                    .on_scroll_wheel(move |_, _, cx| {
                        let mut offset = scroll_handle.offset();
                        let previous_offset = offset;
                        offset.x = last_scroll_offset.borrow().x;

                        if offset != previous_offset {
                            scroll_handle.set_offset(offset);
                            cx.notify(view_id);
                        }

                        cx.stop_propagation();
                    })
                    .child(SelectableTextElement { view: cx.entity() }),
            )
            .child(
                Scrollbars::new(&self.scroll_handle, self.scrollbar_theme)
                    .axis(ScrollbarAxis::Both)
                    .wheel_mode(WheelScrollMode::Native),
            )
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
    }
}
