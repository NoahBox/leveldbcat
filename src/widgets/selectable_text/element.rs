use super::{LineState, PrepaintState, SelectableTextElement};
use gpui::{
    App, Bounds, Element, ElementId, ElementInputHandler, GlobalElementId, LayoutId, Pixels, Style,
    Window, fill, point, px, rgba, size,
};

impl gpui::IntoElement for SelectableTextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for SelectableTextElement {
    type RequestLayoutState = Vec<LineState>;
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let input = self.view.read(cx);
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = window.line_height();
        let line_states = super::highlight::shape_lines(
            input.content.clone(),
            font_size,
            style.color,
            input.highlight_mode,
            window,
        );
        let max_width = line_states
            .iter()
            .map(|line| line.layout.width)
            .max()
            .unwrap_or(Pixels::ZERO);

        let mut layout_style = Style::default();
        layout_style.size.width = max_width.max(px(1.0)).into();
        layout_style.size.height = (line_height * line_states.len() as f32).into();

        (window.request_layout(layout_style, [], cx), line_states)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.view.read(cx);
        let line_height = window.line_height();
        let cursor_offset = input.cursor_offset();
        let mut selections = Vec::new();
        let mut cursor = None;

        for (index, line) in request_layout.iter().enumerate() {
            let line_top = bounds.top() + line_height * index as f32;
            let line_bottom = line_top + line_height;
            let overlap_start = input.selected_range.start.max(line.start);
            let overlap_end = input.selected_range.end.min(line.end);

            if overlap_start < overlap_end {
                let start_x = line.layout.x_for_index(overlap_start - line.start);
                let end_x = line.layout.x_for_index(overlap_end - line.start);
                selections.push(fill(
                    Bounds::new(
                        point(bounds.left() + start_x, line_top),
                        size((end_x - start_x).max(px(2.0)), line_bottom - line_top),
                    ),
                    rgba(0x3311ff30),
                ));
            } else if input.selected_range.is_empty()
                && cursor.is_none()
                && cursor_offset >= line.start
                && cursor_offset <= line.end
            {
                let cursor_x = line.layout.x_for_index(cursor_offset - line.start);
                cursor = Some(fill(
                    Bounds::new(
                        point(bounds.left() + cursor_x, line_top),
                        size(px(2.0), line_bottom - line_top),
                    ),
                    gpui::blue(),
                ));
            }
        }

        PrepaintState {
            lines: request_layout.clone(),
            selections,
            cursor,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let focus_handle = self.view.read(cx).focus_handle.clone();
        window.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.view.clone()),
            cx,
        );

        for selection in prepaint.selections.drain(..) {
            window.paint_quad(selection);
        }

        let line_height = window.line_height();
        for (index, line) in prepaint.lines.iter().enumerate() {
            let origin = point(bounds.left(), bounds.top() + line_height * index as f32);
            line.layout.paint(origin, line_height, window, cx).ok();
        }

        if focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }

        let lines = prepaint.lines.clone();
        self.view.update(cx, |input, _cx| {
            input.last_lines = lines;
            input.last_bounds = Some(bounds);
        });
    }
}
