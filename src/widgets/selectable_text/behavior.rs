use super::{HighlightMode, SelectableTextElement, SelectableTextView};
use gpui::{
    App, Bounds, Context, CursorStyle, EntityInputHandler, FocusHandle, Focusable, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, Point, UTF16Selection, Window, div, point,
    prelude::*, px,
};
use std::ops::Range;

use crate::widgets::scrollbars::{ScrollbarAxis, Scrollbars, WheelScrollMode};

impl SelectableTextView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            content: gpui::SharedString::new_static(""),
            highlight_mode: HighlightMode::Plain,
            scroll_handle: gpui::ScrollHandle::new(),
            scrollbar_theme: Default::default(),
            selected_range: 0..0,
            selection_reversed: false,
            last_lines: Vec::new(),
            last_bounds: None,
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
        self.selected_range = 0..0;
        self.selection_reversed = false;
        self.last_lines.clear();
        self.last_bounds = None;
        self.is_selecting = false;
        cx.notify();
    }

    pub fn set_scrollbar_theme(&mut self, theme: crate::widgets::scrollbars::ScrollbarTheme) {
        self.scrollbar_theme = theme;
    }

    fn line_count(&self) -> usize {
        self.content.as_ref().split('\n').count().max(1)
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
        let Some(bounds) = self.last_bounds else {
            return 0;
        };

        if position.y <= bounds.top() {
            return 0;
        }

        if position.y >= bounds.bottom() {
            return self.content.len();
        }

        let line_height = if self.last_lines.is_empty() {
            px(0.0)
        } else {
            bounds.size.height / self.last_lines.len() as f32
        };

        if line_height <= px(0.0) {
            return 0;
        }

        let line_index = (((position.y - bounds.top()) / line_height) as usize)
            .min(self.last_lines.len().saturating_sub(1));
        let line = &self.last_lines[line_index];
        let local_x = position.x - bounds.left();
        line.start + line.layout.closest_index_for_x(local_x)
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
        bounds: Bounds<gpui::Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<gpui::Pixels>> {
        let range = self.range_from_utf16(&range_utf16);
        let start = self
            .last_lines
            .iter()
            .enumerate()
            .find_map(|(index, line)| {
                (range.start >= line.start && range.start <= line.end).then_some((index, line))
            })?;
        let end = self
            .last_lines
            .iter()
            .enumerate()
            .find_map(|(index, line)| {
                (range.end >= line.start && range.end <= line.end).then_some((index, line))
            })?;

        let line_height = bounds.size.height / self.line_count() as f32;
        let start_x = start.1.layout.x_for_index(range.start - start.1.start);
        let end_x = end.1.layout.x_for_index(range.end - end.1.start);

        Some(Bounds::from_corners(
            gpui::point(
                bounds.left() + start_x,
                bounds.top() + line_height * start.0 as f32,
            ),
            gpui::point(
                bounds.left() + end_x.max(start_x + px(1.0)),
                bounds.top() + line_height * (end.0 as f32 + 1.0),
            ),
        ))
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
        let view_id = cx.entity_id();

        div()
            .size_full()
            .relative()
            .track_focus(&self.focus_handle(cx))
            .child(
                div()
                    .id("selectable-text-scroll")
                    .size_full()
                    .overflow_scroll()
                    .track_scroll(&self.scroll_handle)
                    .cursor(CursorStyle::IBeam)
                    .on_scroll_wheel(move |event, window, cx| {
                        let mut offset = scroll_handle.offset();
                        let previous_offset = offset;
                        let delta = event.delta.pixel_delta(window.line_height());
                        offset.y =
                            (offset.y + delta.y).clamp(-scroll_handle.max_offset().height, px(0.0));
                        scroll_handle.set_offset(offset);
                        if offset != previous_offset {
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
