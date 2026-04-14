use gpui::{
    App, Axis, Bounds, ContentMask, CursorStyle, Element, ElementId, GlobalElementId, Hitbox,
    HitboxBehavior, Hsla, InspectorElementId, IntoElement, LayoutId, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, Point, Position, ScrollHandle, Size, Style,
    UniformListScrollHandle, Window, fill, point, px, relative, size,
};
use std::{cell::Cell, panic::Location, rc::Rc};

const TRACK_THICKNESS: Pixels = px(12.0);
const THUMB_INSET: Pixels = px(2.0);
const THUMB_RADIUS: Pixels = px(6.0);
const MIN_THUMB_LENGTH: Pixels = px(28.0);

#[derive(Clone, Copy)]
pub struct ScrollbarTheme {
    pub track: Hsla,
    pub thumb: Hsla,
    pub thumb_hover: Hsla,
    pub thumb_active: Hsla,
}

impl Default for ScrollbarTheme {
    fn default() -> Self {
        Self {
            track: gpui::black().opacity(0.08),
            thumb: gpui::black().opacity(0.24),
            thumb_hover: gpui::black().opacity(0.34),
            thumb_active: gpui::black().opacity(0.5),
        }
    }
}

pub trait ScrollbarHandle: 'static {
    fn offset(&self) -> Point<Pixels>;
    fn set_offset(&self, offset: Point<Pixels>);
    fn content_size(&self) -> Size<Pixels>;
}

impl ScrollbarHandle for ScrollHandle {
    fn offset(&self) -> Point<Pixels> {
        self.offset()
    }

    fn set_offset(&self, offset: Point<Pixels>) {
        self.set_offset(offset);
    }

    fn content_size(&self) -> Size<Pixels> {
        self.bounds().size + self.max_offset()
    }
}

impl ScrollbarHandle for UniformListScrollHandle {
    fn offset(&self) -> Point<Pixels> {
        self.0.borrow().base_handle.offset()
    }

    fn set_offset(&self, offset: Point<Pixels>) {
        self.0.borrow().base_handle.set_offset(offset);
    }

    fn content_size(&self) -> Size<Pixels> {
        let base = &self.0.borrow().base_handle;
        base.bounds().size + base.max_offset()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarAxis {
    Vertical,
    Both,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WheelScrollMode {
    Native,
    Both,
}

#[derive(Clone, Copy, Debug, Default)]
struct DragStateInner {
    axis: Option<Axis>,
    pointer_offset: Pixels,
}

#[derive(Clone, Debug, Default)]
struct DragState(Rc<Cell<DragStateInner>>);

#[derive(Clone)]
struct AxisState {
    axis: Axis,
    bar_hitbox: Option<Hitbox>,
    track_bounds: Bounds<Pixels>,
    thumb_bounds: Bounds<Pixels>,
    scroll_span: Pixels,
    track_span: Pixels,
    thumb_size: Pixels,
}

pub struct Scrollbars {
    id: ElementId,
    axis: ScrollbarAxis,
    wheel_mode: WheelScrollMode,
    scroll_handle: Rc<dyn ScrollbarHandle>,
    theme: ScrollbarTheme,
}

impl Scrollbars {
    #[track_caller]
    pub fn new<H: ScrollbarHandle + Clone>(scroll_handle: &H, theme: ScrollbarTheme) -> Self {
        let caller = Location::caller();
        Self {
            id: ElementId::CodeLocation(*caller),
            axis: ScrollbarAxis::Both,
            wheel_mode: WheelScrollMode::Both,
            scroll_handle: Rc::new(scroll_handle.clone()),
            theme,
        }
    }

    pub fn axis(mut self, axis: ScrollbarAxis) -> Self {
        self.axis = axis;
        self
    }

    pub fn wheel_mode(mut self, wheel_mode: WheelScrollMode) -> Self {
        self.wheel_mode = wheel_mode;
        self
    }
}

impl IntoElement for Scrollbars {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

pub struct PrepaintState {
    bounds: Bounds<Pixels>,
    drag_state: DragState,
    axis_states: Vec<AxisState>,
}

impl Element for Scrollbars {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style {
            position: Position::Absolute,
            flex_grow: 1.0,
            flex_shrink: 1.0,
            ..Default::default()
        };
        style.size.width = relative(1.0).into();
        style.size.height = relative(1.0).into();

        (window.request_layout(style, None, cx), ())
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let drag_state = window
            .use_state(cx, |_, _| DragState::default())
            .read(cx)
            .clone();

        let content_size = self.scroll_handle.content_size();
        let show_vertical = matches!(self.axis, ScrollbarAxis::Vertical | ScrollbarAxis::Both)
            && content_size.height > bounds.size.height;
        let show_horizontal =
            matches!(self.axis, ScrollbarAxis::Both) && content_size.width > bounds.size.width;
        let scroll_offset = self.scroll_handle.offset();
        let mut axis_states = Vec::new();

        if show_vertical
            && let Some(mut axis_state) = build_axis_state(
                bounds,
                Axis::Vertical,
                content_size,
                scroll_offset,
                show_horizontal,
            )
        {
            axis_state.bar_hitbox = Some(
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    window.insert_hitbox(axis_state.track_bounds, HitboxBehavior::Normal)
                }),
            );
            axis_states.push(axis_state);
        }

        if show_horizontal
            && let Some(mut axis_state) = build_axis_state(
                bounds,
                Axis::Horizontal,
                content_size,
                scroll_offset,
                show_vertical,
            )
        {
            axis_state.bar_hitbox = Some(
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    window.insert_hitbox(axis_state.track_bounds, HitboxBehavior::Normal)
                }),
            );
            axis_states.push(axis_state);
        }

        PrepaintState {
            bounds,
            drag_state,
            axis_states,
        }
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        let mouse_position = window.mouse_position();
        let view_id = window.current_view();

        window.with_content_mask(
            Some(ContentMask {
                bounds: prepaint.bounds,
            }),
            |window| {
                for axis_state in &prepaint.axis_states {
                    let thumb_color = if prepaint.drag_state.0.get().axis == Some(axis_state.axis) {
                        self.theme.thumb_active
                    } else if axis_state.thumb_bounds.contains(&mouse_position) {
                        self.theme.thumb_hover
                    } else {
                        self.theme.thumb
                    };

                    window.paint_quad(
                        fill(axis_state.track_bounds, self.theme.track).corner_radii(THUMB_RADIUS),
                    );
                    window.paint_quad(
                        fill(axis_state.thumb_bounds, thumb_color).corner_radii(THUMB_RADIUS),
                    );
                    if let Some(bar_hitbox) = axis_state.bar_hitbox.as_ref() {
                        window.set_cursor_style(CursorStyle::default(), bar_hitbox);
                    }
                }

                window.on_mouse_event({
                    let axis_states = prepaint.axis_states.clone();
                    let drag_state = prepaint.drag_state.clone();
                    let scroll_handle = self.scroll_handle.clone();

                    move |event: &MouseDownEvent, phase, window, cx| {
                        if event.button != MouseButton::Left || !phase.bubble() {
                            return;
                        }

                        let Some(axis_state) = axis_states.iter().find(|axis_state| {
                            axis_state
                                .bar_hitbox
                                .as_ref()
                                .is_some_and(|hitbox| hitbox.is_hovered(window))
                        }) else {
                            return;
                        };

                        cx.stop_propagation();

                        if axis_state.thumb_bounds.contains(&event.position) {
                            drag_state.0.set(DragStateInner {
                                axis: Some(axis_state.axis),
                                pointer_offset: axis_position(event.position, axis_state.axis)
                                    - axis_position(
                                        axis_state.thumb_bounds.origin,
                                        axis_state.axis,
                                    ),
                            });
                            cx.notify(view_id);
                            return;
                        }

                        let new_axis_offset = axis_offset_for_pointer(
                            axis_position(event.position, axis_state.axis)
                                - axis_state.thumb_size / 2.0,
                            axis_state,
                        );
                        let mut offset = scroll_handle.offset();
                        let previous_offset = offset;
                        set_axis_offset(&mut offset, axis_state.axis, new_axis_offset);
                        scroll_handle.set_offset(offset);
                        if offset != previous_offset {
                            cx.notify(view_id);
                        }
                    }
                });

                window.on_mouse_event({
                    let axis_states = prepaint.axis_states.clone();
                    let drag_state = prepaint.drag_state.clone();
                    let scroll_handle = self.scroll_handle.clone();

                    move |event: &MouseMoveEvent, phase, _, cx| {
                        if !phase.capture() {
                            return;
                        }

                        let drag = drag_state.0.get();
                        let Some(axis) = drag.axis else {
                            return;
                        };
                        if !event.dragging() {
                            return;
                        }

                        let Some(axis_state) = axis_states
                            .iter()
                            .find(|axis_state| axis_state.axis == axis)
                        else {
                            return;
                        };

                        let new_axis_offset = axis_offset_for_pointer(
                            axis_position(event.position, axis) - drag.pointer_offset,
                            axis_state,
                        );
                        let mut offset = scroll_handle.offset();
                        let previous_offset = offset;
                        set_axis_offset(&mut offset, axis, new_axis_offset);
                        scroll_handle.set_offset(offset);
                        if offset != previous_offset {
                            cx.notify(view_id);
                        }
                        cx.stop_propagation();
                    }
                });

                window.on_mouse_event({
                    let drag_state = prepaint.drag_state.clone();

                    move |event: &MouseUpEvent, phase, _, cx| {
                        if event.button == MouseButton::Left
                            && phase.capture()
                            && drag_state.0.get().axis.is_some()
                        {
                            drag_state.0.set(DragStateInner::default());
                            cx.notify(view_id);
                        }
                    }
                });
            },
        );
    }
}

fn build_axis_state(
    bounds: Bounds<Pixels>,
    axis: Axis,
    content_size: Size<Pixels>,
    scroll_offset: Point<Pixels>,
    reserve_end_corner: bool,
) -> Option<AxisState> {
    let track_length = if matches!(axis, Axis::Vertical) {
        bounds.size.height
            - if reserve_end_corner {
                TRACK_THICKNESS
            } else {
                px(0.0)
            }
    } else {
        bounds.size.width
            - if reserve_end_corner {
                TRACK_THICKNESS
            } else {
                px(0.0)
            }
    };
    if track_length <= px(0.0) {
        return None;
    }

    let scroll_size = if matches!(axis, Axis::Vertical) {
        content_size.height
    } else {
        content_size.width
    };
    if scroll_size <= track_length {
        return None;
    }

    let thumb_size = (track_length / scroll_size * track_length)
        .max(MIN_THUMB_LENGTH)
        .min(track_length);
    let track_span = (track_length - thumb_size).max(px(0.0));
    let scroll_span = (scroll_size - track_length).max(px(0.0));
    let current_offset = axis_offset(scroll_offset, axis).clamp(-scroll_span, px(0.0));
    let thumb_start = if scroll_span <= px(0.0) || track_span <= px(0.0) {
        px(0.0)
    } else {
        -(current_offset / scroll_span * track_span)
    };

    let track_bounds = if matches!(axis, Axis::Vertical) {
        Bounds::new(
            point(bounds.right() - TRACK_THICKNESS, bounds.top()),
            size(TRACK_THICKNESS, track_length),
        )
    } else {
        Bounds::new(
            point(bounds.left(), bounds.bottom() - TRACK_THICKNESS),
            size(track_length, TRACK_THICKNESS),
        )
    };

    let thumb_bounds = if matches!(axis, Axis::Vertical) {
        Bounds::new(
            point(
                track_bounds.left() + THUMB_INSET,
                track_bounds.top() + thumb_start,
            ),
            size(track_bounds.size.width - THUMB_INSET * 2.0, thumb_size),
        )
    } else {
        Bounds::new(
            point(
                track_bounds.left() + thumb_start,
                track_bounds.top() + THUMB_INSET,
            ),
            size(thumb_size, track_bounds.size.height - THUMB_INSET * 2.0),
        )
    };

    Some(AxisState {
        axis,
        bar_hitbox: None,
        track_bounds,
        thumb_bounds,
        scroll_span,
        track_span,
        thumb_size,
    })
}

fn axis_position(position: Point<Pixels>, axis: Axis) -> Pixels {
    match axis {
        Axis::Horizontal => position.x,
        Axis::Vertical => position.y,
    }
}

fn axis_offset(offset: Point<Pixels>, axis: Axis) -> Pixels {
    match axis {
        Axis::Horizontal => offset.x,
        Axis::Vertical => offset.y,
    }
}

fn set_axis_offset(offset: &mut Point<Pixels>, axis: Axis, value: Pixels) {
    match axis {
        Axis::Horizontal => offset.x = value,
        Axis::Vertical => offset.y = value,
    }
}

fn axis_offset_for_pointer(pointer_position: Pixels, axis_state: &AxisState) -> Pixels {
    if axis_state.track_span <= px(0.0) {
        return px(0.0);
    }

    let track_origin = axis_position(axis_state.track_bounds.origin, axis_state.axis);
    let thumb_start = (pointer_position - track_origin).clamp(px(0.0), axis_state.track_span);
    let percent = (thumb_start / axis_state.track_span).clamp(0.0, 1.0);
    -(axis_state.scroll_span * percent)
}
