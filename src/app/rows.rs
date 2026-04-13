#[derive(IntoElement)]
struct FolderTreeRow {
    app: Entity<LevelDbBrowserApp>,
    row: TreeRow,
    palette: Palette,
}

impl RenderOnce for FolderTreeRow {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let indent = px(self.row.depth as f32 * 14.0);
        let toggle_path = self.row.path.clone();
        let select_path = self.row.path.clone();
        let label = self.row.label.clone();
        let selected = self.row.selected;
        let palette = self.palette;
        let app_for_toggle = self.app.clone();
        let app_for_select = self.app.clone();

        div()
            .flex()
            .items_center()
            .gap_1()
            .px_2()
            .py_1()
            .bg(if selected {
                palette.selected_bg
            } else {
                palette.surface_bg
            })
            .child(div().flex_none().w(indent))
            .child(
                compact_button(if self.row.expanded { "-" } else { "+" }, palette)
                    .id(("tree-toggle", path_hash(&self.row.path)))
                    .on_click(move |_, _, cx| {
                        let _ = app_for_toggle.update(cx, |this, cx| {
                            this.toggle_directory(toggle_path.clone());
                            cx.notify();
                        });
                    }),
            )
            .child(
                div()
                    .flex_1()
                    .whitespace_nowrap()
                    .cursor_pointer()
                    .rounded_sm()
                    .px_2()
                    .py_1()
                    .bg(if selected {
                        palette.selected_strong_bg
                    } else {
                        palette.surface_alt_bg
                    })
                    .hover(|style| style.bg(palette.hover_bg))
                    .id(("tree-select", path_hash(&self.row.path)))
                    .child(label)
                    .on_click(move |_, _, cx| {
                        let _ = app_for_select.update(cx, |this, cx| {
                            this.select_directory(select_path.clone());
                            cx.notify();
                        });
                    }),
            )
    }
}

#[derive(IntoElement)]
struct FileBrowserCard {
    app: Entity<LevelDbBrowserApp>,
    selected: bool,
    entry: BrowserEntry,
    palette: Palette,
    language: AppLanguage,
}

impl RenderOnce for FileBrowserCard {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let i18n = I18n::new(self.language);
        let navigate_path = self.entry.path.clone();
        let parse_path = self.entry.path.clone();
        let app_for_navigate = self.app.clone();
        let app_for_parse = self.app.clone();
        let is_directory = self.entry.kind.is_directory();
        let palette = self.palette;
        let type_label = self.entry.kind.label(i18n).to_owned();
        let meta_text = match self.entry.kind {
            BrowserEntryKind::Directory => type_label.clone(),
            BrowserEntryKind::File { size_bytes } => size_bytes
                .map(format_file_size)
                .unwrap_or_else(|| i18n.text(TextKey::UnknownSize).to_owned()),
            BrowserEntryKind::Link => i18n.text(TextKey::ReparsePoint).to_owned(),
        };

        let card = div()
            .w(FILE_CARD_WIDTH)
            .h(FILE_CARD_HEIGHT)
            .flex_none()
            .flex()
            .flex_col()
            .justify_between()
            .rounded_md()
            .border_1()
            .border_color(if self.selected {
                palette.selected_strong_bg
            } else {
                palette.border
            })
            .bg(if self.selected {
                palette.selected_bg
            } else {
                palette.surface_alt_bg
            })
            .px_3()
            .py_3()
            .when(is_directory, |this| {
                this.cursor_pointer()
                    .hover(|style| style.bg(palette.hover_alt_bg))
            })
            .id(("browser-card-open", path_hash(&self.entry.path)));

        let card = if is_directory {
            card.on_click(move |_, _, cx| {
                let _ = app_for_navigate.update(cx, |this, cx| {
                    this.select_directory(navigate_path.clone());
                    cx.notify();
                });
            })
        } else {
            card
        };

        card.child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .text_color(palette.muted_text)
                        .child(meta_text),
                )
                .child(
                    div()
                        .truncate()
                        .whitespace_nowrap()
                        .child(self.entry.label.clone()),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_end()
                .h(px(28.0))
                .when(is_directory, |this| {
                    this.child(
                        compact_button(i18n.text(TextKey::Parse), palette)
                            .id(("browser-parse", path_hash(&self.entry.path)))
                            .on_click(move |_, _, cx| {
                                let _ = app_for_parse.update(cx, |this, cx| {
                                    this.select_directory(parse_path.clone());
                                    this.try_parse_leveldb(parse_path.clone(), cx);
                                    cx.notify();
                                });
                            }),
                    )
                }),
        )
    }
}

#[derive(IntoElement)]
struct ParsedEntryRow {
    app: Entity<LevelDbBrowserApp>,
    index: usize,
    palette: Palette,
    widths: ResolvedResultWidths,
}

impl RenderOnce for ParsedEntryRow {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let palette = self.palette;
        let row_bg = if self.index.is_multiple_of(2) {
            palette.list_even_bg
        } else {
            palette.list_odd_bg
        };
        let (key_preview, value_preview, key_selected, value_selected) = {
            let app = self.app.read(_cx);
            let key_selected = app.selected_detail.as_ref().is_some_and(|detail| {
                detail.row == self.index && detail.column == ParsedColumn::Key
            });
            let value_selected = app.selected_detail.as_ref().is_some_and(|detail| {
                detail.row == self.index && detail.column == ParsedColumn::Value
            });
            (
                app.preview_cell_text(self.index, ParsedColumn::Key)
                    .unwrap_or_default(),
                app.preview_cell_text(self.index, ParsedColumn::Value)
                    .unwrap_or_default(),
                key_selected,
                value_selected,
            )
        };
        let widths = self.widths;
        let app_for_key = self.app.clone();
        let app_for_value = self.app.clone();
        let app_for_key_menu = self.app.clone();
        let app_for_value_menu = self.app.clone();

        div()
            .flex()
            .items_center()
            .px_3()
            .py_1()
            .border_b_1()
            .border_color(palette.subtle_border)
            .bg(row_bg)
            .child(
                div()
                    .w(widths.index_width)
                    .flex_none()
                    .child((self.index + 1).to_string()),
            )
            .child(div().w(RESULT_COLUMN_SPLITTER_SIZE).flex_none())
            .child(
                preview_cell(key_preview, palette)
                    .w(widths.key_width)
                    .bg(if key_selected {
                        palette.selected_strong_bg
                    } else {
                        row_bg
                    })
                    .id(("entry-key", self.index))
                    .on_click(move |_, _, cx| {
                        let _ = app_for_key.update(cx, |this, cx| {
                            this.show_detail(self.index, ParsedColumn::Key, cx);
                            this.close_context_menu();
                            cx.notify();
                        });
                    })
                    .on_mouse_down(MouseButton::Right, move |event, _, cx| {
                        let _ = app_for_key_menu.update(cx, |this, cx| {
                            this.open_context_menu(
                                ParseContextTarget::Cell {
                                    row: self.index,
                                    column: ParsedColumn::Key,
                                },
                                event.position,
                            );
                            cx.notify();
                        });
                    }),
            )
            .child(div().w(RESULT_COLUMN_SPLITTER_SIZE).flex_none())
            .child(
                preview_cell(value_preview, palette)
                    .w(widths.value_width)
                    .flex_none()
                    .bg(if value_selected {
                        palette.selected_strong_bg
                    } else {
                        row_bg
                    })
                    .id(("entry-value", self.index))
                    .on_click(move |_, _, cx| {
                        let _ = app_for_value.update(cx, |this, cx| {
                            this.show_detail(self.index, ParsedColumn::Value, cx);
                            this.close_context_menu();
                            cx.notify();
                        });
                    })
                    .on_mouse_down(MouseButton::Right, move |event, _, cx| {
                        let _ = app_for_value_menu.update(cx, |this, cx| {
                            this.open_context_menu(
                                ParseContextTarget::Cell {
                                    row: self.index,
                                    column: ParsedColumn::Value,
                                },
                                event.position,
                            );
                            cx.notify();
                        });
                    }),
            )
    }
}
