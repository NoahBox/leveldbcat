use crate::config::{
    AppConfig, AppLanguage, VisualMode, default_config_path, max_font_size_px, min_font_size_px,
};
use crate::i18n::{I18n, TextKey};
use crate::reader::{Entry, format_bytes, load_entries};
use crate::widgets::scrollbars::{ScrollbarAxis, ScrollbarTheme, Scrollbars, WheelScrollMode};
use crate::widgets::selectable_text::{HighlightMode, JsonHighlightColors, SelectableTextView};
use crate::widgets::text_input::{
    Backspace, Copy, Delete, End, Home, Left, Paste, Right, SelectAll, TextInputView,
};
use csv::WriterBuilder;
use gpui::{
    AnyElement, App, Application, Bounds, ClipboardItem, Context, DragMoveEvent, Empty, Entity,
    KeyBinding, MouseButton, MouseDownEvent, MouseUpEvent, Pixels, Point, Render, ScrollHandle,
    SharedString, TitlebarOptions, UniformListScrollHandle, Window, WindowBounds, WindowOptions,
    actions, deferred, div, prelude::*, px, rgb, size, uniform_list,
};
use rfd::FileDialog;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const CLI_PREVIEW_ROWS: usize = 5;

const INITIAL_SIDEBAR_WIDTH: Pixels = px(320.0);
const INITIAL_BROWSER_HEIGHT: Pixels = px(220.0);
const INITIAL_DETAIL_HEIGHT: Pixels = px(170.0);
const SPLITTER_SIZE: Pixels = px(8.0);
const RESULT_COLUMN_SPLITTER_SIZE: Pixels = px(8.0);

const MIN_SIDEBAR_WIDTH: Pixels = px(220.0);
const MIN_MAIN_WIDTH: Pixels = px(420.0);
const MIN_BROWSER_HEIGHT: Pixels = px(140.0);
const MIN_ENTRIES_HEIGHT: Pixels = px(160.0);
const MIN_DETAIL_HEIGHT: Pixels = px(120.0);
const MIN_RESULT_INDEX_WIDTH: Pixels = px(44.0);
const MIN_RESULT_KEY_WIDTH: Pixels = px(140.0);
const MIN_RESULT_VALUE_WIDTH: Pixels = px(180.0);
const OPTIONS_PANEL_WIDTH: Pixels = px(540.0);

const FILE_CARD_WIDTH: Pixels = px(260.0);
const FILE_CARD_HEIGHT: Pixels = px(88.0);
const TOAST_DURATION: Duration = Duration::from_secs(3);

actions!(leveldbcat, [CopySelectedItem]);

pub(crate) use self::entry::run;

mod entry {
    include!("entry.rs");
}

include!("state.rs");
include!("actions.rs");
include!("tree.rs");
include!("panels.rs");
include!("overlays.rs");
include!("rows.rs");
include!("helpers.rs");
