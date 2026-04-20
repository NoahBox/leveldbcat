struct LevelDbBrowserApp {
    roots: Vec<PathBuf>,
    selected_dir: PathBuf,
    children_cache: HashMap<PathBuf, Vec<BrowserEntry>>,
    expanded: HashSet<PathBuf>,
    entries: Vec<Entry>,
    loaded_db_path: Option<PathBuf>,
    selected_detail: Option<SelectedDetail>,
    sidebar_scroll: ScrollHandle,
    browser_scroll: ScrollHandle,
    entries_scroll: UniformListScrollHandle,
    detail_text_view: Entity<SelectableTextView>,
    search_input: Entity<TextInputView>,
    app_logo: Arc<Image>,
    toast: Option<Toast>,
    config: AppConfig,
    config_path: PathBuf,
    installed_monospace_fonts: Vec<String>,
    options_open: bool,
    font_dropdown_open: bool,
    language_dropdown_open: bool,
    about_open: bool,
    layout: LayoutState,
    drag_state: Option<SplitterDragState>,
    result_columns: ResultColumnLayout,
    column_modes: ColumnModes,
    cell_mode_overrides: HashMap<(usize, ParsedColumn), ParseMode>,
    context_menu: Option<ParseContextMenu>,
    search_state: SearchState,
}

#[derive(Clone)]
struct BrowserEntry {
    path: PathBuf,
    label: String,
    kind: BrowserEntryKind,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BrowserEntryKind {
    Directory,
    File { size_bytes: Option<u64> },
    Link,
}

impl BrowserEntryKind {
    fn is_directory(self) -> bool {
        matches!(self, Self::Directory)
    }

    fn sort_priority(self) -> u8 {
        match self {
            Self::Directory => 0,
            Self::Link => 1,
            Self::File { .. } => 2,
        }
    }

    fn label(self, i18n: I18n) -> &'static str {
        match self {
            Self::Directory => i18n.text(TextKey::Folder),
            Self::File { .. } => i18n.text(TextKey::File),
            Self::Link => i18n.text(TextKey::Link),
        }
    }
}

#[derive(Clone)]
struct TreeRow {
    path: PathBuf,
    label: String,
    depth: usize,
    expanded: bool,
    selected: bool,
}

#[derive(Clone)]
struct SelectedDetail {
    row: usize,
    column: ParsedColumn,
}

#[derive(Default)]
struct SearchState {
    open: bool,
    matches: Vec<SelectedDetail>,
    current: Option<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum ParsedColumn {
    Key,
    Value,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum ParseMode {
    Bytes,
    Text,
    Json,
}

#[derive(Clone, Copy)]
struct ColumnModes {
    key: ParseMode,
    value: ParseMode,
}

impl Default for ColumnModes {
    fn default() -> Self {
        Self {
            key: ParseMode::Bytes,
            value: ParseMode::Bytes,
        }
    }
}

#[derive(Clone)]
struct ParseContextMenu {
    target: ParseContextTarget,
    position: Point<Pixels>,
    prefer_selected_detail_text: bool,
}

#[derive(Clone, Copy)]
enum ParseContextTarget {
    Cell { row: usize, column: ParsedColumn },
    Header(ParsedColumn),
}

#[derive(Clone, Copy)]
enum ParseMenuAction {
    Copy,
    Mode(ParseMode),
}

#[derive(Clone)]
struct Toast {
    kind: ToastKind,
    message: String,
    expires_at: Instant,
}

#[derive(Clone, Copy)]
enum ToastKind {
    Success,
    Error,
}

#[derive(Clone, Copy)]
struct LayoutState {
    sidebar_width: Pixels,
    browser_height: Pixels,
    detail_height: Pixels,
}

#[derive(Clone, Copy)]
struct ResultColumnLayout {
    index_width: Pixels,
    key_width: Pixels,
}

impl Default for ResultColumnLayout {
    fn default() -> Self {
        Self {
            index_width: px(52.0),
            key_width: px(320.0),
        }
    }
}

impl ResultColumnLayout {
    fn from_config(config: &AppConfig) -> Self {
        let default = Self::default();

        Self {
            index_width: config
                .result_index_width_px
                .map(px)
                .unwrap_or(default.index_width),
            key_width: config.result_key_width_px.map(px).unwrap_or(default.key_width),
        }
    }
}

impl Default for LayoutState {
    fn default() -> Self {
        Self {
            sidebar_width: INITIAL_SIDEBAR_WIDTH,
            browser_height: INITIAL_BROWSER_HEIGHT,
            detail_height: INITIAL_DETAIL_HEIGHT,
        }
    }
}

impl LayoutState {
    fn from_config(config: &AppConfig) -> Self {
        Self {
            sidebar_width: config
                .sidebar_width_px
                .map(px)
                .unwrap_or(INITIAL_SIDEBAR_WIDTH),
            browser_height: config
                .browser_height_px
                .map(px)
                .unwrap_or(INITIAL_BROWSER_HEIGHT),
            detail_height: config
                .detail_height_px
                .map(px)
                .unwrap_or(INITIAL_DETAIL_HEIGHT),
        }
    }
}

#[derive(Clone, Copy)]
struct LayoutLimits {
    max_sidebar_width: Pixels,
    max_browser_height: Pixels,
    max_detail_height: Pixels,
}

#[derive(Clone, Copy)]
struct ResultColumnLimits {
    max_index_width: Pixels,
    max_key_width: Pixels,
}

#[derive(Clone, Copy)]
struct ResolvedResultWidths {
    index_width: Pixels,
    key_width: Pixels,
    value_width: Pixels,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SplitterKind {
    Sidebar,
    Browser,
    Detail,
    ResultIndex,
    ResultKey,
}

impl SplitterKind {
    fn id(self) -> usize {
        match self {
            Self::Sidebar => 0,
            Self::Browser => 1,
            Self::Detail => 2,
            Self::ResultIndex => 3,
            Self::ResultKey => 4,
        }
    }

    fn is_vertical(self) -> bool {
        matches!(self, Self::Sidebar | Self::ResultIndex | Self::ResultKey)
    }
}

#[derive(Clone, Copy)]
struct SplitterDragState {
    kind: SplitterKind,
    start_position: Point<Pixels>,
    start_value: Pixels,
}

#[derive(Clone, Copy)]
struct SplitterDrag(SplitterKind);

impl Render for SplitterDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

#[derive(Clone, Copy)]
struct Palette {
    window_bg: gpui::Rgba,
    surface_bg: gpui::Rgba,
    surface_alt_bg: gpui::Rgba,
    toolbar_bg: gpui::Rgba,
    toolbar_text: gpui::Rgba,
    text: gpui::Rgba,
    muted_text: gpui::Rgba,
    border: gpui::Rgba,
    subtle_border: gpui::Rgba,
    header_bg: gpui::Rgba,
    selected_bg: gpui::Rgba,
    selected_strong_bg: gpui::Rgba,
    hover_bg: gpui::Rgba,
    hover_alt_bg: gpui::Rgba,
    list_even_bg: gpui::Rgba,
    list_odd_bg: gpui::Rgba,
    splitter: gpui::Rgba,
    splitter_active: gpui::Rgba,
    button_primary_bg: gpui::Rgba,
    button_primary_hover: gpui::Rgba,
    button_compact_bg: gpui::Rgba,
    button_compact_hover: gpui::Rgba,
    error_fg: gpui::Rgba,
    error_bg: gpui::Rgba,
    success_fg: gpui::Rgba,
    success_bg: gpui::Rgba,
    json_key: gpui::Rgba,
    json_string: gpui::Rgba,
    json_number: gpui::Rgba,
    json_keyword: gpui::Rgba,
    json_punctuation: gpui::Rgba,
    overlay: gpui::Hsla,
}

impl Palette {
    fn for_mode(mode: VisualMode) -> Self {
        match mode {
            VisualMode::Light => Self {
                window_bg: rgb(0xf8fafc),
                surface_bg: rgb(0xffffff),
                surface_alt_bg: rgb(0xf8fafc),
                toolbar_bg: rgb(0x0f172a),
                toolbar_text: rgb(0xf8fafc),
                text: rgb(0x111827),
                muted_text: rgb(0x64748b),
                border: rgb(0xd1d5db),
                subtle_border: rgb(0xe5e7eb),
                header_bg: rgb(0xf3f4f6),
                selected_bg: rgb(0xe0f2fe),
                selected_strong_bg: rgb(0xbfdbfe),
                hover_bg: rgb(0xe5e7eb),
                hover_alt_bg: rgb(0xe2e8f0),
                list_even_bg: rgb(0xffffff),
                list_odd_bg: rgb(0xf8fafc),
                splitter: rgb(0xcbd5e1),
                splitter_active: rgb(0x2563eb),
                button_primary_bg: rgb(0x2563eb),
                button_primary_hover: rgb(0x1d4ed8),
                button_compact_bg: rgb(0xe2e8f0),
                button_compact_hover: rgb(0xcbd5e1),
                error_fg: rgb(0x7f1d1d),
                error_bg: rgb(0xfef2f2),
                success_fg: rgb(0x0f5132),
                success_bg: rgb(0xf0fdf4),
                json_key: rgb(0x1d4ed8),
                json_string: rgb(0x047857),
                json_number: rgb(0xb45309),
                json_keyword: rgb(0x7c3aed),
                json_punctuation: rgb(0x475569),
                overlay: gpui::black().opacity(0.45),
            },
            VisualMode::Dark => Self {
                window_bg: rgb(0x0b1220),
                surface_bg: rgb(0x111827),
                surface_alt_bg: rgb(0x0f172a),
                toolbar_bg: rgb(0x020617),
                toolbar_text: rgb(0xf8fafc),
                text: rgb(0xe5e7eb),
                muted_text: rgb(0x94a3b8),
                border: rgb(0x334155),
                subtle_border: rgb(0x1e293b),
                header_bg: rgb(0x1f2937),
                selected_bg: rgb(0x0c4a6e),
                selected_strong_bg: rgb(0x075985),
                hover_bg: rgb(0x1f2937),
                hover_alt_bg: rgb(0x334155),
                list_even_bg: rgb(0x111827),
                list_odd_bg: rgb(0x0f172a),
                splitter: rgb(0x334155),
                splitter_active: rgb(0x38bdf8),
                button_primary_bg: rgb(0x0ea5e9),
                button_primary_hover: rgb(0x0284c7),
                button_compact_bg: rgb(0x1e293b),
                button_compact_hover: rgb(0x334155),
                error_fg: rgb(0xfecaca),
                error_bg: rgb(0x450a0a),
                success_fg: rgb(0xbbf7d0),
                success_bg: rgb(0x052e16),
                json_key: rgb(0x93c5fd),
                json_string: rgb(0x6ee7b7),
                json_number: rgb(0xfcd34d),
                json_keyword: rgb(0xc4b5fd),
                json_punctuation: rgb(0x94a3b8),
                overlay: gpui::black().opacity(0.6),
            },
        }
    }
}
