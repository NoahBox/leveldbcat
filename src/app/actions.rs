impl LevelDbBrowserApp {
    fn new(
        config: AppConfig,
        config_path: PathBuf,
        initial_dir: PathBuf,
        cx: &mut Context<Self>,
    ) -> Self {
        let config = config.sanitized();
        let mut roots = discover_roots();
        ensure_root_for_path(&mut roots, &initial_dir);
        let installed_monospace_fonts = installed_monospace_fonts();
        let detail_text_view = cx.new(SelectableTextView::new);
        let search_input = cx.new(|cx| TextInputView::new("", cx));
        let layout = LayoutState::from_config(&config);
        let result_columns = ResultColumnLayout::from_config(&config);

        let mut app = Self {
            roots,
            selected_dir: initial_dir.clone(),
            children_cache: HashMap::new(),
            expanded: HashSet::new(),
            entries: Vec::new(),
            loaded_db_path: None,
            selected_detail: None,
            sidebar_scroll: ScrollHandle::new(),
            browser_scroll: ScrollHandle::new(),
            entries_scroll: UniformListScrollHandle::new(),
            detail_text_view,
            search_input: search_input.clone(),
            app_logo: app_logo_image(),
            toast: None,
            config,
            config_path,
            installed_monospace_fonts,
            options_open: false,
            font_dropdown_open: false,
            language_dropdown_open: false,
            about_open: false,
            layout,
            drag_state: None,
            result_columns,
            column_modes: ColumnModes::default(),
            cell_mode_overrides: HashMap::new(),
            context_menu: None,
            search_state: SearchState::default(),
        };

        if let Some(font) = &app.config.monospace_font_family
            && !app.installed_monospace_fonts.iter().any(|candidate| candidate == font)
        {
            app.config.monospace_font_family = None;
        }

        app.expand_to_directory(&initial_dir);
        app.config.last_workdir = Some(initial_dir.clone());
        app.persist_config();
        app.sync_search_placeholder(cx);
        cx.observe(&search_input, |this, _, cx| {
            this.refresh_search_matches(cx);
            cx.notify();
        })
        .detach();
        app.sync_detail_view(cx);
        app
    }

    fn i18n(&self) -> I18n {
        I18n::new(self.config.language)
    }

    fn palette(&self) -> Palette {
        Palette::for_mode(self.config.visual_mode)
    }

    fn scrollbar_theme(&self) -> ScrollbarTheme {
        let palette = self.palette();
        ScrollbarTheme {
            track: palette.window_bg.into(),
            thumb: palette.border.into(),
            thumb_hover: palette.button_compact_hover.into(),
            thumb_active: palette.splitter_active.into(),
        }
    }

    fn show_toast(&mut self, kind: ToastKind, message: String) {
        self.toast = Some(Toast {
            kind,
            message,
            expires_at: Instant::now() + TOAST_DURATION,
        });
    }

    fn show_success(&mut self, message: String) {
        self.show_toast(ToastKind::Success, message);
    }

    fn show_error(&mut self, message: String) {
        self.show_toast(ToastKind::Error, message);
    }

    fn persist_config(&mut self) {
        if let Err(error) = self.config.save(&self.config_path) {
            let message = self.i18n().config_save_failed(&error);
            self.show_error(message);
        }
    }

    fn cleanup_expired_toast(&mut self) {
        if self
            .toast
            .as_ref()
            .is_some_and(|toast| Instant::now() >= toast.expires_at)
        {
            self.toast = None;
        }
    }

    fn chrome_height_estimate(&self) -> Pixels {
        px(self.config.font_size_px + 30.0)
    }

    fn layout_limits(&self, viewport: gpui::Size<Pixels>) -> LayoutLimits {
        let max_sidebar_width =
            (viewport.width - MIN_MAIN_WIDTH - SPLITTER_SIZE).max(MIN_SIDEBAR_WIDTH);
        let main_height = (viewport.height - self.chrome_height_estimate())
            .max(MIN_BROWSER_HEIGHT + MIN_ENTRIES_HEIGHT + MIN_DETAIL_HEIGHT + SPLITTER_SIZE * 2.0);
        let browser_height = self
            .layout
            .browser_height
            .clamp(MIN_BROWSER_HEIGHT, main_height);
        let max_browser_height =
            (main_height - MIN_ENTRIES_HEIGHT - MIN_DETAIL_HEIGHT - SPLITTER_SIZE * 2.0)
                .max(MIN_BROWSER_HEIGHT);
        let lower_height = (main_height - browser_height - SPLITTER_SIZE)
            .max(MIN_ENTRIES_HEIGHT + MIN_DETAIL_HEIGHT + SPLITTER_SIZE);
        let max_detail_height =
            (lower_height - MIN_ENTRIES_HEIGHT - SPLITTER_SIZE).max(MIN_DETAIL_HEIGHT);

        LayoutLimits {
            max_sidebar_width,
            max_browser_height,
            max_detail_height,
        }
    }

    fn main_content_width(&self, viewport: gpui::Size<Pixels>) -> Pixels {
        (viewport.width - self.layout.sidebar_width - SPLITTER_SIZE).max(MIN_MAIN_WIDTH)
    }

    fn result_column_limits(&self, viewport: gpui::Size<Pixels>) -> ResultColumnLimits {
        let available_width = self.main_content_width(viewport);
        let max_index_width = (available_width
            - MIN_RESULT_KEY_WIDTH
            - MIN_RESULT_VALUE_WIDTH
            - RESULT_COLUMN_SPLITTER_SIZE * 2.0)
            .max(MIN_RESULT_INDEX_WIDTH);
        let max_key_width = (available_width
            - self.result_columns.index_width
            - MIN_RESULT_VALUE_WIDTH
            - RESULT_COLUMN_SPLITTER_SIZE * 2.0)
            .max(MIN_RESULT_KEY_WIDTH);

        ResultColumnLimits {
            max_index_width,
            max_key_width,
        }
    }

    fn resolved_result_widths(&self, viewport: gpui::Size<Pixels>) -> ResolvedResultWidths {
        let available =
            (self.main_content_width(viewport) - RESULT_COLUMN_SPLITTER_SIZE * 2.0 - px(24.0))
                .max(MIN_RESULT_INDEX_WIDTH + MIN_RESULT_KEY_WIDTH + MIN_RESULT_VALUE_WIDTH);

        let index_width = self.result_columns.index_width.clamp(
            MIN_RESULT_INDEX_WIDTH,
            available - MIN_RESULT_KEY_WIDTH - MIN_RESULT_VALUE_WIDTH,
        );
        let remaining =
            (available - index_width).max(MIN_RESULT_KEY_WIDTH + MIN_RESULT_VALUE_WIDTH);
        let key_width = self
            .result_columns
            .key_width
            .clamp(MIN_RESULT_KEY_WIDTH, remaining - MIN_RESULT_VALUE_WIDTH);
        let value_width = (available - index_width - key_width).max(MIN_RESULT_VALUE_WIDTH);

        ResolvedResultWidths {
            index_width,
            key_width,
            value_width,
        }
    }

    fn clamp_layout(&mut self, viewport: gpui::Size<Pixels>) {
        let limits = self.layout_limits(viewport);
        self.layout.sidebar_width = self
            .layout
            .sidebar_width
            .clamp(MIN_SIDEBAR_WIDTH, limits.max_sidebar_width);
        self.layout.browser_height = self
            .layout
            .browser_height
            .clamp(MIN_BROWSER_HEIGHT, limits.max_browser_height);
        self.layout.detail_height = self
            .layout
            .detail_height
            .clamp(MIN_DETAIL_HEIGHT, limits.max_detail_height);
        self.clamp_result_columns(viewport);
    }

    fn clamp_result_columns(&mut self, viewport: gpui::Size<Pixels>) {
        let limits = self.result_column_limits(viewport);
        self.result_columns.index_width = self
            .result_columns
            .index_width
            .clamp(MIN_RESULT_INDEX_WIDTH, limits.max_index_width);
        self.result_columns.key_width = self
            .result_columns
            .key_width
            .clamp(MIN_RESULT_KEY_WIDTH, limits.max_key_width);
    }

    fn begin_splitter_drag(&mut self, kind: SplitterKind, start_position: Point<Pixels>) {
        let start_value = match kind {
            SplitterKind::Sidebar => self.layout.sidebar_width,
            SplitterKind::Browser => self.layout.browser_height,
            SplitterKind::Detail => self.layout.detail_height,
            SplitterKind::ResultIndex => self.result_columns.index_width,
            SplitterKind::ResultKey => self.result_columns.key_width,
        };

        self.drag_state = Some(SplitterDragState {
            kind,
            start_position,
            start_value,
        });
    }

    fn clear_splitter_drag(&mut self) {
        if self.drag_state.take().is_some() {
            self.persist_resized_layout();
        }
    }

    fn persist_resized_layout(&mut self) {
        self.config.sidebar_width_px = Some(f32::from(self.layout.sidebar_width));
        self.config.browser_height_px = Some(f32::from(self.layout.browser_height));
        self.config.detail_height_px = Some(f32::from(self.layout.detail_height));
        self.config.result_index_width_px = Some(f32::from(self.result_columns.index_width));
        self.config.result_key_width_px = Some(f32::from(self.result_columns.key_width));
        self.persist_config();
    }

    fn apply_splitter_drag(&mut self, position: Point<Pixels>, viewport: gpui::Size<Pixels>) {
        let Some(drag_state) = self.drag_state else {
            return;
        };

        let limits = self.layout_limits(viewport);
        let result_limits = self.result_column_limits(viewport);

        match drag_state.kind {
            SplitterKind::Sidebar => {
                let delta = position.x - drag_state.start_position.x;
                self.layout.sidebar_width = (drag_state.start_value + delta)
                    .clamp(MIN_SIDEBAR_WIDTH, limits.max_sidebar_width);
            }
            SplitterKind::Browser => {
                let delta = position.y - drag_state.start_position.y;
                self.layout.browser_height = (drag_state.start_value + delta)
                    .clamp(MIN_BROWSER_HEIGHT, limits.max_browser_height);
            }
            SplitterKind::Detail => {
                let delta = drag_state.start_position.y - position.y;
                self.layout.detail_height = (drag_state.start_value + delta)
                    .clamp(MIN_DETAIL_HEIGHT, limits.max_detail_height);
            }
            SplitterKind::ResultIndex => {
                let delta = position.x - drag_state.start_position.x;
                self.result_columns.index_width = (drag_state.start_value + delta)
                    .clamp(MIN_RESULT_INDEX_WIDTH, result_limits.max_index_width);
            }
            SplitterKind::ResultKey => {
                let delta = position.x - drag_state.start_position.x;
                self.result_columns.key_width = (drag_state.start_value + delta)
                    .clamp(MIN_RESULT_KEY_WIDTH, result_limits.max_key_width);
            }
        }

        self.clamp_layout(viewport);
    }

    fn adjust_font_size(&mut self, delta_px: f32) {
        self.config.font_size_px =
            (self.config.font_size_px + delta_px).clamp(min_font_size_px(), max_font_size_px());
        self.persist_config();
    }

    fn adjust_json_indent_spaces(&mut self, delta: i8, cx: &mut Context<Self>) {
        let current = i16::from(self.config.json_indent_spaces);
        let next = (current + i16::from(delta)).clamp(
            i16::from(min_json_indent_spaces()),
            i16::from(max_json_indent_spaces()),
        ) as u8;

        if next == self.config.json_indent_spaces {
            return;
        }

        self.config.json_indent_spaces = next;
        self.persist_config();
        self.sync_detail_view(cx);
    }

    fn set_monospace_font_family(
        &mut self,
        font_family: Option<String>,
        cx: &mut Context<Self>,
    ) {
        self.config.monospace_font_family = font_family;
        self.font_dropdown_open = false;
        self.persist_config();
        self.sync_detail_view(cx);
    }

    fn set_visual_mode(&mut self, visual_mode: VisualMode, cx: &mut Context<Self>) {
        if self.config.visual_mode != visual_mode {
            self.config.visual_mode = visual_mode;
            self.persist_config();
            self.sync_detail_view(cx);
        }
    }

    fn set_language(&mut self, language: AppLanguage, cx: &mut Context<Self>) {
        if self.config.language != language {
            self.config.language = language;
            self.persist_config();
            self.sync_search_placeholder(cx);
        }
        self.language_dropdown_open = false;
    }

    fn close_context_menu(&mut self) {
        self.context_menu = None;
    }

    fn open_context_menu(
        &mut self,
        target: ParseContextTarget,
        position: Point<Pixels>,
        prefer_selected_detail_text: bool,
    ) {
        self.context_menu = Some(ParseContextMenu {
            target,
            position,
            prefer_selected_detail_text,
        });
    }

    fn parse_selected_directory(&mut self, cx: &mut Context<Self>) {
        self.try_parse_leveldb(self.selected_dir.clone(), cx);
    }

    fn export_entries_with_dialog(&mut self) {
        if self.entries.is_empty() || self.loaded_db_path.is_none() {
            self.show_error(self.i18n().no_entries_to_export());
            return;
        }

        let loaded_db_path = self
            .loaded_db_path
            .clone()
            .unwrap_or_else(|| self.selected_dir.clone());
        let i18n = self.i18n();
        let file_name = format!("{}.csv", display_name(&loaded_db_path));

        let mut dialog = FileDialog::new()
            .set_title(i18n.text(TextKey::ExportDialogTitle))
            .add_filter("CSV", &["csv"])
            .set_file_name(&file_name);

        if let Some(parent) = loaded_db_path.parent() {
            dialog = dialog.set_directory(parent);
        }

        let Some(path) = dialog.save_file() else {
            return;
        };

        match export_entries_to_csv(&path, &self.entries) {
            Ok(final_path) => self.show_success(i18n.export_success(&final_path)),
            Err(error) => self.show_error(i18n.export_failed(&error)),
        }
    }

    fn refresh_selected_directory(&mut self) {
        let path = self.selected_dir.clone();
        self.children_cache.remove(&path);

        match self.ensure_children_loaded(&path) {
            Ok(()) => self.show_success(self.i18n().refreshed(&path)),
            Err(error) => self.show_error(error),
        }
    }

    fn open_parent_directory(&mut self) {
        let Some(parent) = self.selected_dir.parent().map(Path::to_path_buf) else {
            return;
        };

        self.select_directory(parent);
    }

    fn toggle_directory(&mut self, path: PathBuf) {
        self.close_context_menu();
        if self.expanded.remove(&path) {
            return;
        }

        match self.ensure_children_loaded(&path) {
            Ok(()) => {
                self.expanded.insert(path);
            }
            Err(error) => self.show_error(error),
        }
    }

    fn select_directory(&mut self, path: PathBuf) {
        let Some(directory) = resolve_existing_directory(&path) else {
            self.show_error(self.i18n().directory_missing(&path));
            return;
        };

        ensure_root_for_path(&mut self.roots, &directory);
        self.selected_dir = directory.clone();
        self.expand_to_directory(&directory);
        self.config.last_workdir = Some(directory);
        self.persist_config();
        self.close_context_menu();
    }

    fn try_parse_leveldb(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let load_result = match persisted_lock_file_name(&path) {
            Ok(Some(lock_file_name)) => {
                if self.confirm_locked_database_open(&path, lock_file_name) {
                    load_entries_ignoring_lock_file(&path)
                } else {
                    self.close_context_menu();
                    return;
                }
            }
            Ok(None) => load_entries(&path),
            Err(error) => Err(error),
        };

        match load_result {
            Ok(entries) => {
                let count = entries.len();
                self.entries = entries;
                self.loaded_db_path = Some(path.clone());
                self.selected_detail = None;
                self.detail_text_view
                    .update(cx, |view, cx| view.set_text(String::new(), cx));
                self.config.last_workdir = Some(path.clone());
                self.persist_config();
                self.show_success(self.i18n().loaded_entries(count, &path));
            }
            Err(error) => {
                self.entries.clear();
                self.loaded_db_path = None;
                self.selected_detail = None;
                self.detail_text_view
                    .update(cx, |view, cx| view.set_text(String::new(), cx));
                self.show_error(error);
            }
        }
        self.cell_mode_overrides.clear();
        self.column_modes = ColumnModes::default();
        self.close_context_menu();
        self.refresh_search_matches(cx);
    }

    fn confirm_locked_database_open(&self, path: &Path, lock_file_name: &str) -> bool {
        let i18n = self.i18n();

        matches!(
            MessageDialog::new()
                .set_title(i18n.locked_database_title())
                .set_description(i18n.locked_database_warning(path, lock_file_name))
                .set_buttons(MessageButtons::YesNo)
                .set_level(MessageLevel::Warning)
                .show(),
            MessageDialogResult::Yes
        )
    }

    fn show_detail(&mut self, row: usize, column: ParsedColumn, cx: &mut Context<Self>) {
        self.selected_detail = Some(SelectedDetail { row, column });
        self.sync_detail_view(cx);
    }

    fn sync_detail_view(&mut self, cx: &mut Context<Self>) {
        let (text, highlight_mode) = self
            .selected_detail
            .as_ref()
            .and_then(|detail| {
                self.formatted_detail_text(detail.row, detail.column)
                    .map(|text| {
                        let mode = self.effective_mode(detail.row, detail.column);
                        let is_valid_json = self
                            .entry_bytes(detail.row, detail.column)
                            .and_then(|bytes| {
                                serde_json::from_slice::<serde_json::Value>(bytes).ok()
                            })
                            .is_some();
                        let highlight = if mode == ParseMode::Json && is_valid_json {
                            HighlightMode::Json(self.json_highlight_colors())
                        } else {
                            HighlightMode::Plain
                        };
                        (text, highlight)
                    })
            })
            .unwrap_or_else(|| (String::new(), HighlightMode::Plain));
        let scrollbar_theme = self.scrollbar_theme();

        self.detail_text_view.update(cx, |view, cx| {
            view.set_scrollbar_theme(scrollbar_theme);
            view.set_text_with_highlight(text, highlight_mode, cx)
        });
    }

    fn json_highlight_colors(&self) -> JsonHighlightColors {
        let palette = self.palette();
        JsonHighlightColors {
            default: palette.text.into(),
            key: palette.json_key.into(),
            string: palette.json_string.into(),
            number: palette.json_number.into(),
            keyword: palette.json_keyword.into(),
            punctuation: palette.json_punctuation.into(),
        }
    }

    fn selected_detail_mode(&self) -> Option<ParseMode> {
        let detail = self.selected_detail.as_ref()?;
        Some(self.effective_mode(detail.row, detail.column))
    }

    fn set_selected_detail_mode(&mut self, mode: ParseMode, cx: &mut Context<Self>) {
        let Some(detail) = self.selected_detail.clone() else {
            return;
        };

        self.apply_mode_to_cell(detail.row, detail.column, mode, cx);
    }

    fn sync_search_placeholder(&mut self, cx: &mut Context<Self>) {
        let placeholder = self.i18n().text(TextKey::SearchPlaceholder).to_owned();
        self.search_input.update(cx, |input, cx| {
            input.set_placeholder(placeholder, cx);
        });
    }

    fn search_query(&self, cx: &App) -> String {
        self.search_input.read(cx).text().to_owned()
    }

    fn open_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.search_state.open = true;
        let focus = self.search_input.read(cx).focus_handle();
        window.focus(&focus);
        self.refresh_search_matches(cx);
    }

    fn close_search(&mut self, cx: &mut Context<Self>) {
        self.search_state = SearchState::default();
        self.search_input.update(cx, |input, cx| input.clear(cx));
    }

    fn refresh_search_matches(&mut self, cx: &mut Context<Self>) {
        let query = self.search_query(cx);
        if query.is_empty() {
            self.search_state.matches.clear();
            self.search_state.current = None;
            return;
        }

        let query_lower = query.to_lowercase();
        let matches = self
            .entries
            .iter()
            .enumerate()
            .flat_map(|(row, _)| {
                [ParsedColumn::Key, ParsedColumn::Value]
                    .into_iter()
                    .filter_map(|column| {
                        self.formatted_cell_text(row, column).and_then(|text| {
                            text.to_lowercase()
                                .contains(&query_lower)
                                .then_some(SelectedDetail { row, column })
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        self.search_state.matches = matches;

        if self.search_state.matches.is_empty() {
            self.search_state.current = None;
            return;
        }

        if let Some(selected) = &self.selected_detail
            && let Some(index) =
                self.search_state.matches.iter().position(|detail| {
                    detail.row == selected.row && detail.column == selected.column
                })
        {
            self.search_state.current = Some(index);
            return;
        }

        self.search_state.current = Some(0);
        if let Some(detail) = self.search_state.matches.first().cloned() {
            self.show_detail(detail.row, detail.column, cx);
        }
    }

    fn navigate_search(&mut self, step: isize, cx: &mut Context<Self>) {
        if self.search_state.matches.is_empty() {
            return;
        }

        let len = self.search_state.matches.len() as isize;
        let current = self.search_state.current.unwrap_or(0) as isize;
        let next = (current + step).rem_euclid(len) as usize;
        self.search_state.current = Some(next);

        if let Some(detail) = self.search_state.matches.get(next).cloned() {
            self.show_detail(detail.row, detail.column, cx);
        }
    }

    fn column_mode(&self, column: ParsedColumn) -> ParseMode {
        match column {
            ParsedColumn::Key => self.column_modes.key,
            ParsedColumn::Value => self.column_modes.value,
        }
    }

    fn effective_mode(&self, row: usize, column: ParsedColumn) -> ParseMode {
        self.cell_mode_overrides
            .get(&(row, column))
            .copied()
            .unwrap_or_else(|| self.column_mode(column))
    }

    fn entry_bytes(&self, row: usize, column: ParsedColumn) -> Option<&[u8]> {
        let entry = self.entries.get(row)?;
        Some(match column {
            ParsedColumn::Key => entry.key_bytes.as_slice(),
            ParsedColumn::Value => entry.value_bytes.as_slice(),
        })
    }

    fn formatted_cell_text(&self, row: usize, column: ParsedColumn) -> Option<String> {
        let bytes = self.entry_bytes(row, column)?;
        Some(format_bytes_with_mode(
            bytes,
            self.effective_mode(row, column),
        ))
    }

    fn formatted_detail_text(&self, row: usize, column: ParsedColumn) -> Option<String> {
        let bytes = self.entry_bytes(row, column)?;
        Some(format_bytes_with_mode_and_json_indent(
            bytes,
            self.effective_mode(row, column),
            self.config.json_indent_spaces,
        ))
    }

    fn preview_cell_text(&self, row: usize, column: ParsedColumn) -> Option<String> {
        let text = self.formatted_cell_text(row, column)?;
        Some(single_line_preview(&text))
    }

    fn apply_mode_to_cell(
        &mut self,
        row: usize,
        column: ParsedColumn,
        mode: ParseMode,
        cx: &mut Context<Self>,
    ) {
        self.cell_mode_overrides.insert((row, column), mode);
        if self
            .selected_detail
            .as_ref()
            .is_some_and(|detail| detail.row == row && detail.column == column)
        {
            self.sync_detail_view(cx);
        }
        self.refresh_search_matches(cx);
    }

    fn apply_mode_to_column(
        &mut self,
        column: ParsedColumn,
        mode: ParseMode,
        cx: &mut Context<Self>,
    ) {
        match column {
            ParsedColumn::Key => self.column_modes.key = mode,
            ParsedColumn::Value => self.column_modes.value = mode,
        }
        self.cell_mode_overrides
            .retain(|(_, current_column), _| *current_column != column);
        if self
            .selected_detail
            .as_ref()
            .is_some_and(|detail| detail.column == column)
        {
            self.sync_detail_view(cx);
        }
        self.refresh_search_matches(cx);
    }

    fn apply_context_action(&mut self, action: ParseMenuAction, cx: &mut Context<Self>) {
        let Some(menu) = self.context_menu.clone() else {
            return;
        };

        match action {
            ParseMenuAction::Copy => {
                let text = if menu.prefer_selected_detail_text {
                    self.selected_detail_text(cx)
                        .or_else(|| self.detail_target_text(menu.target))
                } else {
                    self.copy_target_text(menu.target)
                };

                if let Some(text) = text {
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                }
            }
            ParseMenuAction::Mode(mode) => match menu.target {
                ParseContextTarget::Cell { row, column } => {
                    self.apply_mode_to_cell(row, column, mode, cx);
                }
                ParseContextTarget::Header(column) => {
                    self.apply_mode_to_column(column, mode, cx);
                }
            },
        }

        self.close_context_menu();
    }

    fn copy_target_text(&self, target: ParseContextTarget) -> Option<String> {
        match target {
            ParseContextTarget::Cell { row, column } => self.formatted_cell_text(row, column),
            ParseContextTarget::Header(column) => Some(
                self.entries
                    .iter()
                    .enumerate()
                    .filter_map(|(row, _)| self.formatted_cell_text(row, column))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
        }
    }

    fn detail_target_text(&self, target: ParseContextTarget) -> Option<String> {
        match target {
            ParseContextTarget::Cell { row, column } => self.formatted_detail_text(row, column),
            ParseContextTarget::Header(_) => self.copy_target_text(target),
        }
    }

    fn selected_detail_text(&self, cx: &App) -> Option<String> {
        self.detail_text_view.read(cx).selected_text()
    }

    fn copy_selected_item_to_clipboard(&mut self, cx: &mut Context<Self>) {
        let Some(detail) = self.selected_detail.clone() else {
            return;
        };

        let text = self.selected_detail_text(cx).or_else(|| {
            self.detail_target_text(ParseContextTarget::Cell {
                row: detail.row,
                column: detail.column,
            })
        });

        if let Some(text) = text {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }
}
