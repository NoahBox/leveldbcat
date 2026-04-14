use super::{PrepaintState, SelectableTextElement, TextLayoutState};
use gpui::{
    App, Bounds, Element, ElementId, ElementInputHandler, GlobalElementId, LayoutId, Pixels,
    Window, fill, point, px, rgba, size,
};

impl gpui::IntoElement for SelectableTextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for SelectableTextElement {
    type RequestLayoutState = TextLayoutState;
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
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = window.line_height();
        let (content, highlight_mode, wrap_lines) = {
            let input = self.view.read(cx);
            (
                input.content.clone(),
                input.highlight_mode,
                input.wrap_lines,
            )
        };
        let color = style.color;

        let state = TextLayoutState::default();
        let layout_state = state.clone();
        let layout_id = window.request_measured_layout(
            Default::default(),
            move |known_dimensions, available_space, window, _cx| {
                let wrap_width = if wrap_lines {
                    known_dimensions.width.or(match available_space.width {
                        gpui::AvailableSpace::Definite(width) => Some(width),
                        _ => None,
                    })
                } else {
                    None
                };
                let lines = super::highlight::shape_lines(
                    content.clone(),
                    font_size,
                    color,
                    highlight_mode,
                    wrap_width,
                    window,
                );
                let mut size: gpui::Size<Pixels> = gpui::Size::default();

                for line in &lines {
                    let line_size = line.layout.size(line_height);
                    size.height += line_size.height;
                    size.width = size.width.max(line_size.width).ceil();
                }

                size.width = size.width.max(px(1.0));
                size.height = size.height.max(line_height);

                layout_state
                    .0
                    .borrow_mut()
                    .replace(super::MeasuredTextLayout { lines, line_height });

                size
            },
        );

        (layout_id, state)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let measured = request_layout.0.borrow();
        let Some(measured) = measured.as_ref() else {
            return PrepaintState {
                lines: Vec::new(),
                selections: Vec::new(),
                cursor: None,
                line_height: px(0.0),
            };
        };
        let input = self.view.read(cx);
        let line_height = measured.line_height;
        let cursor_offset = input.cursor_offset();
        let mut selections = Vec::new();
        let mut cursor = None;
        let mut lines = measured.lines.clone();
        let mut line_origin = bounds.origin;

        for line in &mut lines {
            let line_size = line.layout.size(line_height);
            let line_bounds = Bounds::new(line_origin, line_size);
            line.bounds = Some(line_bounds);
            let overlap_start = input.selected_range.start.max(line.start);
            let overlap_end = input.selected_range.end.min(line.end);

            if overlap_start < overlap_end {
                let local_start = overlap_start - line.start;
                let local_end = overlap_end - line.start;

                for (row_index, row) in super::wrapped_rows(&line.layout).into_iter().enumerate() {
                    let segment_start = local_start.max(row.start);
                    let segment_end = local_end.min(row.end);
                    if segment_start >= segment_end {
                        continue;
                    }

                    let start_x =
                        line.layout.unwrapped_layout.x_for_index(segment_start) - row.start_x;
                    let end_x = line.layout.unwrapped_layout.x_for_index(segment_end) - row.start_x;
                    selections.push(fill(
                        Bounds::new(
                            point(
                                line_bounds.left() + start_x,
                                line_bounds.top() + line_height * row_index as f32,
                            ),
                            size((end_x - start_x).max(px(2.0)), line_height),
                        ),
                        rgba(0x3311ff30),
                    ));
                }
            } else if input.selected_range.is_empty()
                && cursor.is_none()
                && cursor_offset >= line.start
                && cursor_offset <= line.end
                && let Some(local_cursor) =
                    super::local_position_for_index(line, cursor_offset, line_height, true)
            {
                cursor = Some(fill(
                    Bounds::new(
                        line_bounds.origin + local_cursor,
                        size(px(2.0), line_height),
                    ),
                    gpui::blue(),
                ));
            }

            line_origin.y += line_size.height;
        }

        PrepaintState {
            lines,
            selections,
            cursor,
            line_height,
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

        let text_align = window.text_style().text_align;
        let line_height = prepaint.line_height.max(px(0.0));

        for line in &prepaint.lines {
            let Some(line_bounds) = line.bounds else {
                continue;
            };
            line.layout
                .paint(
                    line_bounds.origin,
                    line_height,
                    text_align,
                    Some(bounds),
                    window,
                    cx,
                )
                .ok();
        }

        if focus_handle.is_focused(window)
            && let Some(cursor) = prepaint.cursor.take()
        {
            window.paint_quad(cursor);
        }

        let lines = prepaint.lines.clone();
        self.view.update(cx, |input, _cx| {
            input.last_lines = lines;
            input.last_line_height = line_height;
        });
    }
}
