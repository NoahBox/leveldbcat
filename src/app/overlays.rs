impl LevelDbBrowserApp {
    fn render_options_overlay(
        &self,
        cx: &mut Context<Self>,
        palette: Palette,
        i18n: I18n,
    ) -> gpui::Div {
        div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .bg(palette.overlay)
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(OPTIONS_PANEL_WIDTH)
                    .max_w_full()
                    .bg(palette.surface_bg)
                    .border_1()
                    .border_color(palette.border)
                    .rounded_md()
                    .shadow_lg()
                    .px_4()
                    .py_4()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(div().text_lg().child(i18n.text(TextKey::OptionsTitle)))
                    .child(
                        div()
                            .text_sm()
                            .text_color(palette.muted_text)
                            .child(i18n.text(TextKey::ConfigSavedAutomatically)),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .w(px(180.0))
                                    .flex_none()
                                    .child(i18n.text(TextKey::FontFamily)),
                            )
                            .child(
                                div()
                                    .relative()
                                    .w(px(260.0))
                                    .flex_none()
                                    .child(
                                        div()
                                            .cursor_pointer()
                                            .rounded_sm()
                                            .border_1()
                                            .border_color(palette.border)
                                            .bg(palette.surface_alt_bg)
                                            .px_3()
                                            .py_1()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .id("options-font-toggle")
                                            .child(div().child(
                                                self.config
                                                    .monospace_font_family
                                                    .clone()
                                                    .unwrap_or_else(|| {
                                                        i18n.text(TextKey::SystemDefault)
                                                            .to_owned()
                                                    }),
                                            ))
                                            .child(if self.font_dropdown_open { "▲" } else { "▼" })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.font_dropdown_open = !this.font_dropdown_open;
                                                this.language_dropdown_open = false;
                                                cx.notify();
                                            })),
                                    )
                                    .when(self.font_dropdown_open, |this| {
                                        this.child(
                                            deferred(
                                                div()
                                                    .absolute()
                                                    .top(px(36.0))
                                                    .left_0()
                                                    .right_0()
                                                    .max_h(px(260.0))
                                                    .id("options-font-menu")
                                                    .overflow_scroll()
                                                    .flex()
                                                    .flex_col()
                                                    .rounded_sm()
                                                    .border_1()
                                                    .border_color(palette.border)
                                                    .bg(palette.surface_bg)
                                                    .shadow_lg()
                                                    .on_mouse_down_out(cx.listener(
                                                        |this, _, _, cx| {
                                                            if this.font_dropdown_open {
                                                                this.font_dropdown_open = false;
                                                                cx.notify();
                                                            }
                                                        },
                                                    ))
                                                    .child(
                                                        dropdown_option(
                                                            i18n.text(TextKey::SystemDefault),
                                                            self.config.monospace_font_family.is_none(),
                                                            palette,
                                                        )
                                                        .id("options-font-default")
                                                        .on_mouse_down(MouseButton::Left, cx.listener(
                                                            |this, _, _, cx| {
                                                                this.set_monospace_font_family(None, cx);
                                                                cx.notify();
                                                            },
                                                        )),
                                                    )
                                                    .children(self.installed_monospace_fonts.iter().enumerate().map(
                                                        |(index, font_name)| {
                                                            let selected = self
                                                                .config
                                                                .monospace_font_family
                                                                .as_ref()
                                                                .is_some_and(|current| current == font_name);
                                                            let font_name = font_name.clone();
                                                            dropdown_option(
                                                                font_name.clone(),
                                                                selected,
                                                                palette,
                                                            )
                                                            .id(("options-font", index))
                                                            .on_mouse_down(MouseButton::Left, cx.listener(
                                                                move |this, _, _, cx| {
                                                                    this.set_monospace_font_family(
                                                                        Some(font_name.clone()),
                                                                        cx,
                                                                    );
                                                                    cx.notify();
                                                                },
                                                            ))
                                                            .into_any_element()
                                                        },
                                                    )),
                                            )
                                            .priority(10),
                                        )
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .w(px(180.0))
                                    .flex_none()
                                    .child(i18n.text(TextKey::FontSize)),
                            )
                            .child(
                                compact_button("-", palette)
                                    .id("options-font-decrease")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.adjust_font_size(-1.0);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                div()
                                    .w(px(90.0))
                                    .flex_none()
                                    .text_center()
                                    .child(i18n.font_size_value(self.config.font_size_px)),
                            )
                            .child(
                                compact_button("+", palette)
                                    .id("options-font-increase")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.adjust_font_size(1.0);
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .w(px(180.0))
                                    .flex_none()
                                    .child(i18n.text(TextKey::VisualMode)),
                            )
                            .child(
                                choice_button(
                                    i18n.text(TextKey::Light),
                                    self.config.visual_mode == VisualMode::Light,
                                    palette,
                                )
                                .id("options-theme-light")
                                .on_click(cx.listener(
                                    |this, _, _, cx| {
                                        this.set_visual_mode(VisualMode::Light, cx);
                                        cx.notify();
                                    },
                                )),
                            )
                            .child(
                                choice_button(
                                    i18n.text(TextKey::Dark),
                                    self.config.visual_mode == VisualMode::Dark,
                                    palette,
                                )
                                .id("options-theme-dark")
                                .on_click(cx.listener(
                                    |this, _, _, cx| {
                                        this.set_visual_mode(VisualMode::Dark, cx);
                                        cx.notify();
                                    },
                                )),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .w(px(180.0))
                                    .flex_none()
                                    .child(i18n.text(TextKey::Language)),
                            )
                            .child(
                                div()
                                    .relative()
                                    .w(px(240.0))
                                    .flex_none()
                                    .child(
                                        div()
                                            .cursor_pointer()
                                            .rounded_sm()
                                            .border_1()
                                            .border_color(palette.border)
                                            .bg(palette.surface_alt_bg)
                                            .px_3()
                                            .py_1()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .id("options-language-toggle")
                                            .child(
                                                div().child(
                                                    i18n.language_name(self.config.language),
                                                ),
                                            )
                                            .child(if self.language_dropdown_open {
                                                "▲"
                                            } else {
                                                "▼"
                                            })
                                            .on_click(cx.listener(|this, _, _, cx| {
                                                this.language_dropdown_open =
                                                    !this.language_dropdown_open;
                                                this.font_dropdown_open = false;
                                                cx.notify();
                                            })),
                                    )
                                    .when(self.language_dropdown_open, |this| {
                                        this.child(
                                            deferred(
                                                div()
                                                    .absolute()
                                                    .top(px(36.0))
                                                    .left_0()
                                                    .right_0()
                                                    .flex()
                                                    .flex_col()
                                                    .rounded_sm()
                                                    .border_1()
                                                    .border_color(palette.border)
                                                    .bg(palette.surface_bg)
                                                    .shadow_lg()
                                                    .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                                                        if this.language_dropdown_open {
                                                            this.language_dropdown_open = false;
                                                            cx.notify();
                                                        }
                                                    }))
                                                    .child(
                                                        dropdown_option(
                                                            i18n.language_name(AppLanguage::English),
                                                            self.config.language
                                                                == AppLanguage::English,
                                                            palette,
                                                        )
                                                        .id("options-language-en")
                                                        .on_mouse_down(
                                                            MouseButton::Left,
                                                            cx.listener(|this, _, _, cx| {
                                                                this.set_language(
                                                                    AppLanguage::English,
                                                                    cx,
                                                                );
                                                                cx.notify();
                                                            }),
                                                        ),
                                                    )
                                                    .child(
                                                        dropdown_option(
                                                            i18n.language_name(AppLanguage::Chinese),
                                                            self.config.language
                                                                == AppLanguage::Chinese,
                                                            palette,
                                                        )
                                                        .id("options-language-zh")
                                                        .on_mouse_down(
                                                            MouseButton::Left,
                                                            cx.listener(|this, _, _, cx| {
                                                                this.set_language(
                                                                    AppLanguage::Chinese,
                                                                    cx,
                                                                );
                                                                cx.notify();
                                                            }),
                                                        ),
                                                    )
                                                    .child(
                                                        dropdown_option(
                                                            i18n.language_name(
                                                                AppLanguage::TraditionalChinese,
                                                            ),
                                                            self.config.language
                                                                == AppLanguage::TraditionalChinese,
                                                            palette,
                                                        )
                                                        .id("options-language-zh-hant")
                                                        .on_mouse_down(
                                                            MouseButton::Left,
                                                            cx.listener(|this, _, _, cx| {
                                                                this.set_language(
                                                                    AppLanguage::TraditionalChinese,
                                                                    cx,
                                                                );
                                                                cx.notify();
                                                            }),
                                                        ),
                                                    )
                                                    .child(
                                                        dropdown_option(
                                                            i18n.language_name(AppLanguage::Japanese),
                                                            self.config.language
                                                                == AppLanguage::Japanese,
                                                            palette,
                                                        )
                                                        .id("options-language-ja")
                                                        .on_mouse_down(
                                                            MouseButton::Left,
                                                            cx.listener(|this, _, _, cx| {
                                                                this.set_language(
                                                                    AppLanguage::Japanese,
                                                                    cx,
                                                                );
                                                                cx.notify();
                                                            }),
                                                        ),
                                                    ),
                                            )
                                            .priority(10),
                                        )
                                    }),
                            ),
                    )
                    .child(
                        div().flex().justify_end().child(
                            primary_button(i18n.text(TextKey::Close), palette)
                                .id("options-close")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.options_open = false;
                                    this.font_dropdown_open = false;
                                    this.language_dropdown_open = false;
                                    cx.notify();
                                })),
                        ),
                    ),
            )
    }

    fn render_toast_overlay(&self, palette: Palette) -> Option<gpui::Div> {
        let toast = self.toast.as_ref()?;

        let (background, foreground) = match toast.kind {
            ToastKind::Success => (palette.success_bg, palette.success_fg),
            ToastKind::Error => (palette.error_bg, palette.error_fg),
        };

        Some(
            div()
                .absolute()
                .top(px(16.0))
                .right(px(16.0))
                .w(px(360.0))
                .max_w_full()
                .bg(background)
                .text_color(foreground)
                .border_1()
                .border_color(palette.border)
                .rounded_md()
                .shadow_lg()
                .px_3()
                .py_2()
                .child(toast.message.clone()),
        )
    }

    fn render_context_menu_overlay(
        &self,
        cx: &mut Context<Self>,
        palette: Palette,
        i18n: I18n,
        viewport: gpui::Size<Pixels>,
    ) -> Option<gpui::Div> {
        let menu = self.context_menu.as_ref()?;
        let menu_width = px(180.0);
        let menu_x = menu
            .position
            .x
            .min((viewport.width - menu_width - px(12.0)).max(px(8.0)));
        let menu_y = menu
            .position
            .y
            .min((viewport.height - px(180.0)).max(px(8.0)));

        Some(
            div()
                .absolute()
                .top_0()
                .left_0()
                .right_0()
                .bottom_0()
                .child(
                    div()
                        .absolute()
                        .left(menu_x)
                        .top(menu_y)
                        .w(menu_width)
                        .bg(palette.surface_bg)
                        .border_1()
                        .border_color(palette.border)
                        .rounded_md()
                        .shadow_lg()
                        .px_1()
                        .py_1()
                        .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                            this.close_context_menu();
                            cx.notify();
                        }))
                        .child(
                            context_menu_button(i18n.text(TextKey::Copy), palette)
                                .id("context-copy")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.apply_context_action(ParseMenuAction::Copy, cx);
                                    cx.notify();
                                })),
                        )
                        .child(
                            context_menu_button(i18n.text(TextKey::ParseAsText), palette)
                                .id("context-text")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.apply_context_action(
                                        ParseMenuAction::Mode(ParseMode::Text),
                                        cx,
                                    );
                                    cx.notify();
                                })),
                        )
                        .child(
                            context_menu_button(i18n.text(TextKey::ParseAsJson), palette)
                                .id("context-json")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.apply_context_action(
                                        ParseMenuAction::Mode(ParseMode::Json),
                                        cx,
                                    );
                                    cx.notify();
                                })),
                        )
                        .child(
                            context_menu_button(i18n.text(TextKey::ParseAsBytes), palette)
                                .id("context-bytes")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.apply_context_action(
                                        ParseMenuAction::Mode(ParseMode::Bytes),
                                        cx,
                                    );
                                    cx.notify();
                                })),
                        ),
                ),
        )
    }

    fn render_about_overlay(
        &self,
        cx: &mut Context<Self>,
        palette: Palette,
        i18n: I18n,
    ) -> gpui::Div {
        div()
            .absolute()
            .top_0()
            .left_0()
            .right_0()
            .bottom_0()
            .bg(palette.overlay)
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(520.0))
                    .max_w_full()
                    .bg(palette.surface_bg)
                    .border_1()
                    .border_color(palette.border)
                    .rounded_md()
                    .shadow_lg()
                    .px_4()
                    .py_4()
                    .flex()
                    .flex_col()
                    .gap_4()
                    .child(div().text_lg().child(i18n.text(TextKey::About)))
                    .child(
                        div().flex().flex_col().gap_1().children(
                            i18n.about_text()
                                .lines()
                                .map(|line| div().child(line.to_owned()).into_any_element()),
                        ),
                    )
                    .child(
                        div().flex().justify_end().child(
                            primary_button(i18n.text(TextKey::Close), palette)
                                .id("about-close")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.about_open = false;
                                    cx.notify();
                                })),
                        ),
                    ),
            )
    }
}

impl Render for LevelDbBrowserApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.cleanup_expired_toast();

        let i18n = self.i18n();
        let palette = self.palette();
        let viewport = window.viewport_size();
        window.set_rem_size(px(self.config.font_size_px));
        window.set_window_title(i18n.text(TextKey::WindowTitle));
        self.clamp_layout(viewport);

        if self.toast.is_some() {
            window.refresh();
        }

        let app = cx.entity();

        let root = div()
            .relative()
            .size_full()
            .bg(palette.window_bg)
            .text_color(palette.text)
            .flex()
            .flex_col()
            .when_some(
                self.config.monospace_font_family.as_ref(),
                |this, family| this.font_family(family.clone()),
            )
            .child(self.render_toolbar(cx, palette, i18n))
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .flex()
                    .child(self.render_sidebar_panel(app.clone(), cx, palette, i18n))
                    .child(self.render_splitter(cx, palette, SplitterKind::Sidebar))
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .overflow_hidden()
                            .flex()
                            .flex_col()
                            .child(self.render_browser_panel(app.clone(), cx, palette, i18n))
                            .child(self.render_splitter(cx, palette, SplitterKind::Browser))
                            .child(
                                div()
                                    .flex_1()
                                    .overflow_hidden()
                                    .flex()
                                    .flex_col()
                                    .child(self.render_entries_panel(
                                        app.clone(),
                                        cx,
                                        palette,
                                        i18n,
                                        viewport,
                                    ))
                                    .child(self.render_splitter(cx, palette, SplitterKind::Detail))
                                    .child(self.render_detail_panel(cx, palette, i18n)),
                            ),
                    ),
            );

        let root = if let Some(toast) = self.render_toast_overlay(palette) {
            root.child(toast)
        } else {
            root
        };

        let root = if let Some(menu) = self.render_context_menu_overlay(cx, palette, i18n, viewport)
        {
            root.child(menu)
        } else {
            root
        };

        let root = if self.about_open {
            root.child(self.render_about_overlay(cx, palette, i18n))
        } else {
            root
        };

        if self.options_open {
            root.child(self.render_options_overlay(cx, palette, i18n))
        } else {
            root
        }
    }
}
