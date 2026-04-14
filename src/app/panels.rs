impl LevelDbBrowserApp {
    fn render_toolbar(&self, cx: &mut Context<Self>, palette: Palette, i18n: I18n) -> gpui::Div {
        div()
            .flex_none()
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .py_2()
            .bg(palette.toolbar_bg)
            .text_color(palette.toolbar_text)
            .child(div().text_lg().child(i18n.text(TextKey::WindowTitle)))
            .child(div().flex_1())
            .child(
                primary_button(i18n.text(TextKey::Options), palette)
                    .id("toolbar-options")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.options_open = true;
                        cx.notify();
                    })),
            )
            .child(
                primary_button(i18n.text(TextKey::About), palette)
                    .id("toolbar-about")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.about_open = true;
                        cx.notify();
                    })),
            )
    }

    fn render_sidebar_panel(
        &self,
        app: Entity<Self>,
        _cx: &mut Context<Self>,
        palette: Palette,
        i18n: I18n,
    ) -> gpui::Div {
        let tree_rows = self.visible_tree_rows();
        let content_width = self.tree_content_width();
        let scrollbar_theme = self.scrollbar_theme();
        let body = div()
            .id("folder-tree-scroll")
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .overflow_scroll()
            .track_scroll(&self.sidebar_scroll);
        let body = if tree_rows.is_empty() {
            body.child(empty_state(i18n.text(TextKey::NoFoldersAvailable), palette))
        } else {
            let app = app.clone();
            let palette = palette;
            body.child(div().w(content_width).flex().flex_col().children(
                tree_rows.into_iter().map(|row| FolderTreeRow {
                    app: app.clone(),
                    row,
                    palette,
                }),
            ))
        };

        div()
            .w(self.layout.sidebar_width)
            .h_full()
            .flex_none()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(palette.surface_bg)
            .border_r_1()
            .border_color(palette.border)
            .child(panel_header(i18n.text(TextKey::FolderTree), palette))
            .child(
                div()
                    .flex_1()
                    .relative()
                    .overflow_hidden()
                    .child(body)
                    .child(
                        Scrollbars::new(&self.sidebar_scroll, scrollbar_theme)
                            .axis(ScrollbarAxis::Both)
                            .wheel_mode(WheelScrollMode::Native),
                    ),
            )
    }

    fn render_browser_panel(
        &self,
        app: Entity<Self>,
        cx: &mut Context<Self>,
        palette: Palette,
        i18n: I18n,
    ) -> gpui::Div {
        let browser_entries = self.current_browser_entries();
        let scrollbar_theme = self.scrollbar_theme();
        let grid = div()
            .id("browser-grid-scroll")
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .overflow_y_scroll()
            .overflow_x_hidden()
            .track_scroll(&self.browser_scroll)
            .px_3()
            .py_3();
        let grid = if browser_entries.is_empty() {
            grid.child(empty_state(i18n.text(TextKey::FolderEmpty), palette))
        } else {
            let palette = palette;
            let language = i18n.language();
            grid.child(div().flex().flex_wrap().items_start().gap_3().children(
                browser_entries.into_iter().map(|entry| FileBrowserCard {
                    app: app.clone(),
                    selected: self.selected_dir == entry.path,
                    entry,
                    palette,
                    language,
                }),
            ))
        };

        div()
            .h(self.layout.browser_height)
            .flex_none()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(palette.surface_bg)
            .child(self.render_browser_title_bar(cx, palette, i18n))
            .child(
                div()
                    .flex_1()
                    .relative()
                    .overflow_hidden()
                    .child(grid)
                    .child(
                        Scrollbars::new(&self.browser_scroll, scrollbar_theme)
                            .axis(ScrollbarAxis::Vertical)
                            .wheel_mode(WheelScrollMode::Native),
                    ),
            )
    }

    fn render_browser_title_bar(
        &self,
        cx: &mut Context<Self>,
        palette: Palette,
        i18n: I18n,
    ) -> gpui::Div {
        div()
            .flex_none()
            .flex()
            .items_center()
            .gap_2()
            .px_3()
            .py_2()
            .bg(palette.header_bg)
            .border_b_1()
            .border_color(palette.subtle_border)
            .child(
                div()
                    .flex_none()
                    .text_sm()
                    .child(i18n.text(TextKey::FileBrowser)),
            )
            .child(self.render_breadcrumbs(cx, palette))
            .child(
                compact_button(i18n.text(TextKey::Up), palette)
                    .id("browser-up")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.open_parent_directory();
                        cx.notify();
                    })),
            )
            .child(
                compact_button(i18n.text(TextKey::Refresh), palette)
                    .id("browser-refresh")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.refresh_selected_directory();
                        cx.notify();
                    })),
            )
            .child(
                compact_button(i18n.text(TextKey::Parse), palette)
                    .id("browser-parse-current")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.parse_selected_directory(cx);
                        cx.notify();
                    })),
            )
    }

    fn render_breadcrumbs(&self, cx: &mut Context<Self>, palette: Palette) -> gpui::Div {
        let (visible_paths, omitted_prefix) = breadcrumb_paths(&self.selected_dir);
        let mut children: Vec<AnyElement> = Vec::new();

        for (index, path) in visible_paths.iter().enumerate() {
            if index > 0 {
                children.push(
                    div()
                        .flex_none()
                        .text_color(palette.muted_text)
                        .child("\\")
                        .into_any_element(),
                );
            }

            if index == 1 && omitted_prefix {
                children.push(
                    div()
                        .flex_none()
                        .text_color(palette.muted_text)
                        .child("...")
                        .into_any_element(),
                );
                children.push(
                    div()
                        .flex_none()
                        .text_color(palette.muted_text)
                        .child("\\")
                        .into_any_element(),
                );
            }

            let navigate_path = path.clone();
            children.push(
                breadcrumb_button(breadcrumb_label(path, index == 0), palette)
                    .id(("breadcrumb", path_hash(path)))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.select_directory(navigate_path.clone());
                        cx.notify();
                    }))
                    .into_any_element(),
            );
        }

        div()
            .flex_1()
            .overflow_hidden()
            .flex()
            .items_center()
            .gap_1()
            .children(children)
    }

    fn render_entries_panel(
        &self,
        app: Entity<Self>,
        cx: &mut Context<Self>,
        palette: Palette,
        i18n: I18n,
        viewport: gpui::Size<Pixels>,
    ) -> gpui::Div {
        let widths = self.resolved_result_widths(viewport);
        let search_count = self.search_state.matches.len();
        let search_position = self
            .search_state
            .current
            .map(|index| index + 1)
            .unwrap_or(0);
        let scrollbar_theme = self.scrollbar_theme();
        let list_body = div().flex_1().overflow_hidden();
        let list_body = if self.entries.is_empty() {
            list_body.child(empty_state(i18n.text(TextKey::NoEntries), palette))
        } else {
            let app = app.clone();
            let palette = palette;
            let widths = widths;
            list_body.child(
                div()
                    .size_full()
                    .relative()
                    .child(
                        uniform_list(
                            "leveldb-entries",
                            self.entries.len(),
                            cx.processor(move |this, range: std::ops::Range<usize>, _, _| {
                                (range.start..range.end.min(this.entries.len()))
                                    .map(|index| ParsedEntryRow {
                                        app: app.clone(),
                                        index,
                                        palette,
                                        widths,
                                    })
                                    .collect::<Vec<_>>()
                            }),
                        )
                        .track_scroll(self.entries_scroll.clone())
                        .size_full(),
                    )
                    .child(
                        Scrollbars::new(&self.entries_scroll, scrollbar_theme)
                            .axis(ScrollbarAxis::Vertical)
                            .wheel_mode(WheelScrollMode::Native),
                    ),
            )
        };

        div()
            .flex_1()
            .overflow_hidden()
            .flex()
            .flex_col()
            .bg(palette.surface_bg)
            .child(
                div()
                    .flex_none()
                    .flex()
                    .gap_2()
                    .items_center()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(palette.subtle_border)
                    .text_sm()
                    .child(div().flex_none().child(i18n.text(TextKey::ParsedEntries)))
                    .child(div().flex_none().text_color(palette.muted_text).child("|"))
                    .child(i18n.parsed_entries_count(self.entries.len()))
                    .child(
                        div()
                            .flex_1()
                            .truncate()
                            .whitespace_nowrap()
                            .child(self.loaded_database_label(i18n)),
                    )
                    .when(self.search_state.open, |this| {
                        this.child(
                            div()
                                .flex_none()
                                .flex()
                                .items_center()
                                .gap_2()
                                .border_1()
                                .border_color(palette.border)
                                .rounded_sm()
                                .px_2()
                                .py_1()
                                .bg(palette.surface_alt_bg)
                                .child(
                                    div()
                                        .w(px(180.0))
                                        .h(px(22.0))
                                        .child(self.search_input.clone()),
                                )
                                .child(
                                    div()
                                        .flex_none()
                                        .text_xs()
                                        .text_color(palette.muted_text)
                                        .child(format!("{search_position}/{search_count}")),
                                )
                                .child(compact_button("↑", palette).id("search-prev").on_click(
                                    cx.listener(|this, _, _, cx| {
                                        this.navigate_search(-1, cx);
                                        cx.notify();
                                    }),
                                ))
                                .child(compact_button("↓", palette).id("search-next").on_click(
                                    cx.listener(|this, _, _, cx| {
                                        this.navigate_search(1, cx);
                                        cx.notify();
                                    }),
                                ))
                                .child(compact_button("✕", palette).id("search-close").on_click(
                                    cx.listener(|this, _, _, cx| {
                                        this.close_search(cx);
                                        cx.notify();
                                    }),
                                )),
                        )
                    })
                    .child(
                        compact_button(i18n.text(TextKey::Search), palette)
                            .id("entries-search")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_search(window, cx);
                                cx.notify();
                            })),
                    )
                    .child(
                        compact_button(i18n.text(TextKey::ExportCsv), palette)
                            .id("entries-export-csv")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.export_entries_with_dialog();
                                cx.notify();
                            })),
                    ),
            )
            .child(
                div()
                    .flex_none()
                    .flex()
                    .items_center()
                    .px_3()
                    .py_1()
                    .text_xs()
                    .text_color(palette.muted_text)
                    .bg(palette.header_bg)
                    .border_b_1()
                    .border_color(palette.subtle_border)
                    .child(
                        div()
                            .w(widths.index_width)
                            .flex_none()
                            .child(i18n.text(TextKey::EntryIndex)),
                    )
                    .child(self.render_splitter(cx, palette, SplitterKind::ResultIndex))
                    .child(
                        div()
                            .w(widths.key_width)
                            .flex_none()
                            .cursor_context_menu()
                            .id("entry-header-key")
                            .child(i18n.text(TextKey::EntryKey))
                            .on_mouse_down(
                                MouseButton::Right,
                                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                                    this.open_context_menu(
                                        ParseContextTarget::Header(ParsedColumn::Key),
                                        event.position,
                                    );
                                    cx.notify();
                                }),
                            ),
                    )
                    .child(self.render_splitter(cx, palette, SplitterKind::ResultKey))
                    .child(
                        div()
                            .w(widths.value_width)
                            .flex_none()
                            .cursor_context_menu()
                            .id("entry-header-value")
                            .child(i18n.text(TextKey::EntryValue))
                            .on_mouse_down(
                                MouseButton::Right,
                                cx.listener(|this, event: &MouseDownEvent, _, cx| {
                                    this.open_context_menu(
                                        ParseContextTarget::Header(ParsedColumn::Value),
                                        event.position,
                                    );
                                    cx.notify();
                                }),
                            ),
                    ),
            )
            .child(list_body)
    }

    fn render_detail_panel(
        &self,
        cx: &mut Context<Self>,
        palette: Palette,
        i18n: I18n,
    ) -> gpui::Div {
        let selected_mode = self.selected_detail_mode();
        let body: AnyElement = if let Some(detail) = &self.selected_detail {
            let target = ParseContextTarget::Cell {
                row: detail.row,
                column: detail.column,
            };
            div()
                .flex_1()
                .overflow_hidden()
                .px_3()
                .py_2()
                .flex()
                .flex_col()
                .cursor_context_menu()
                .id("detail-panel-body")
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                        this.open_context_menu(target, event.position);
                        cx.notify();
                    }),
                )
                .child(
                    div()
                        .flex_1()
                        .overflow_hidden()
                        .child(self.detail_text_view.clone()),
                )
                .into_any_element()
        } else {
            div()
                .flex_1()
                .overflow_hidden()
                .child(empty_state(i18n.text(TextKey::NothingSelected), palette))
                .into_any_element()
        };

        div()
            .h(self.layout.detail_height)
            .flex_none()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(palette.surface_bg)
            .child(
                div()
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .bg(palette.header_bg)
                    .text_color(palette.text)
                    .border_b_1()
                    .border_color(palette.subtle_border)
                    .child(div().flex_1().child(i18n.text(TextKey::SelectedValue)))
                    .child(
                        choice_button(
                            i18n.text(TextKey::ModeBytes),
                            selected_mode == Some(ParseMode::Bytes),
                            palette,
                        )
                        .id("detail-mode-bytes")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.set_selected_detail_mode(ParseMode::Bytes, cx);
                            cx.notify();
                        })),
                    )
                    .child(
                        choice_button(
                            i18n.text(TextKey::ModeJson),
                            selected_mode == Some(ParseMode::Json),
                            palette,
                        )
                        .id("detail-mode-json")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.set_selected_detail_mode(ParseMode::Json, cx);
                            cx.notify();
                        })),
                    )
                    .child(
                        choice_button(
                            i18n.text(TextKey::ModeText),
                            selected_mode == Some(ParseMode::Text),
                            palette,
                        )
                        .id("detail-mode-text")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.set_selected_detail_mode(ParseMode::Text, cx);
                            cx.notify();
                        })),
                    ),
            )
            .child(body)
    }

    fn render_splitter(
        &self,
        cx: &mut Context<Self>,
        palette: Palette,
        kind: SplitterKind,
    ) -> impl IntoElement {
        let is_active = self
            .drag_state
            .is_some_and(|drag_state| drag_state.kind == kind);
        let splitter_size = match kind {
            SplitterKind::ResultIndex | SplitterKind::ResultKey => RESULT_COLUMN_SPLITTER_SIZE,
            _ => SPLITTER_SIZE,
        };
        let line_color = if is_active {
            palette.splitter_active
        } else {
            palette.splitter
        };

        div()
            .id(("splitter", kind.id()))
            .flex_none()
            .relative()
            .bg(palette.window_bg)
            .when(kind.is_vertical(), |this| {
                this.cursor_col_resize()
                    .w(splitter_size)
                    .when(
                        matches!(kind, SplitterKind::ResultIndex | SplitterKind::ResultKey),
                        |this| this.h(px(24.0)),
                    )
                    .when(
                        !matches!(kind, SplitterKind::ResultIndex | SplitterKind::ResultKey),
                        |this| this.h_full(),
                    )
                    .child(
                        div()
                            .absolute()
                            .top_0()
                            .bottom_0()
                            .left((splitter_size - px(2.0)) / 2.0)
                            .w(px(2.0))
                            .bg(line_color),
                    )
            })
            .when(!kind.is_vertical(), |this| {
                this.cursor_row_resize().w_full().h(splitter_size).child(
                    div()
                        .absolute()
                        .left_0()
                        .right_0()
                        .top((splitter_size - px(2.0)) / 2.0)
                        .h(px(2.0))
                        .bg(line_color),
                )
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                    this.begin_splitter_drag(kind, event.position);
                    cx.notify();
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, cx| {
                    this.clear_splitter_drag();
                    cx.notify();
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _: &MouseUpEvent, _, cx| {
                    this.clear_splitter_drag();
                    cx.notify();
                }),
            )
            .on_drag(SplitterDrag(kind), |drag, _, _, cx| {
                cx.stop_propagation();
                let drag = *drag;
                cx.new(|_| drag)
            })
            .on_drag_move(cx.listener(
                move |this, event: &DragMoveEvent<SplitterDrag>, window, cx| {
                    if event.drag(cx).0 != kind {
                        return;
                    }

                    this.apply_splitter_drag(event.event.position, window.viewport_size());
                    cx.notify();
                },
            ))
    }
}
