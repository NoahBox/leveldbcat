fn export_entries_to_csv(path: &Path, entries: &[Entry]) -> Result<PathBuf, String> {
    let mut final_path = path.to_path_buf();
    if final_path.extension().is_none() {
        final_path.set_extension("csv");
    }

    let mut file = File::create(&final_path)
        .map_err(|error| format!("Failed to create {}: {error}", final_path.display()))?;
    file.write_all(&[0xEF, 0xBB, 0xBF]).map_err(|error| {
        format!(
            "Failed to write UTF-8 BOM to {}: {error}",
            final_path.display()
        )
    })?;

    let mut writer = WriterBuilder::new().from_writer(file);
    writer
        .write_record(["key", "value"])
        .map_err(|error| format!("Failed to write CSV header: {error}"))?;

    for entry in entries {
        writer
            .write_record([
                format_bytes(&entry.key_bytes),
                format_bytes(&entry.value_bytes),
            ])
            .map_err(|error| format!("Failed to write CSV row: {error}"))?;
    }

    writer
        .flush()
        .map_err(|error| format!("Failed to flush CSV file {}: {error}", final_path.display()))?;

    Ok(final_path)
}

fn installed_monospace_fonts() -> Vec<String> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    let mut fonts = db
        .faces()
        .filter(|face| face.monospaced)
        .filter_map(|face| face.families.first().map(|(family, _)| family.clone()))
        .collect::<Vec<_>>();

    fonts.sort_by_cached_key(|name| name.to_ascii_lowercase());
    fonts.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    fonts
}

fn discover_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    for letter in b'A'..=b'Z' {
        let candidate = PathBuf::from(format!("{}:\\", letter as char));
        if candidate.is_dir() {
            roots.push(candidate);
        }
    }

    if roots.is_empty() {
        if let Ok(current_dir) = std::env::current_dir() {
            roots.push(current_dir);
        }
    }

    roots
}

fn ensure_root_for_path(roots: &mut Vec<PathBuf>, path: &Path) {
    let Some(root) = ancestor_chain(path).into_iter().next() else {
        return;
    };

    if !roots.iter().any(|candidate| *candidate == root) {
        roots.push(root);
        roots.sort();
    }
}

fn ancestor_chain(path: &Path) -> Vec<PathBuf> {
    let mut chain = Vec::new();
    let mut current = Some(path);

    while let Some(path) = current {
        chain.push(path.to_path_buf());
        current = path.parent();
    }

    chain.reverse();
    chain
}

fn resolve_existing_directory(path: &Path) -> Option<PathBuf> {
    let mut current = Some(path);

    while let Some(candidate) = current {
        if candidate.is_dir() {
            return Some(candidate.to_path_buf());
        }
        current = candidate.parent();
    }

    None
}

fn breadcrumb_paths(path: &Path) -> (Vec<PathBuf>, bool) {
    let ancestors = ancestor_chain(path);
    if ancestors.len() <= 6 {
        return (ancestors, false);
    }

    let mut visible = Vec::new();
    visible.push(ancestors[0].clone());
    visible.extend(ancestors[ancestors.len() - 5..].iter().cloned());
    (visible, true)
}

fn breadcrumb_label(path: &Path, is_root: bool) -> String {
    if is_root {
        path.display().to_string()
    } else {
        display_name(path)
    }
}

fn display_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| path.display().to_string())
}

fn format_file_size(size_bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;
    const GIB: u64 = MIB * 1024;

    if size_bytes >= GIB {
        format!("{:.2} GiB", size_bytes as f64 / GIB as f64)
    } else if size_bytes >= MIB {
        format!("{:.2} MiB", size_bytes as f64 / MIB as f64)
    } else if size_bytes >= KIB {
        format!("{:.2} KiB", size_bytes as f64 / KIB as f64)
    } else {
        format!("{size_bytes} B")
    }
}

fn estimate_text_width(text: &str, font_size_px: f32) -> f32 {
    text.chars()
        .map(|character| {
            if character.is_ascii() {
                font_size_px * 0.62
            } else {
                font_size_px * 1.05
            }
        })
        .sum()
}

fn path_hash(path: &Path) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish()
}

fn single_line_preview(text: &str) -> String {
    text.chars()
        .map(|character| match character {
            '\r' | '\n' | '\t' => ' ',
            other => other,
        })
        .collect()
}

fn format_text_bytes(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes)
        .chars()
        .filter(|character| !character.is_control() || matches!(character, '\n' | '\r' | '\t'))
        .collect()
}

fn format_bytes_with_mode(bytes: &[u8], mode: ParseMode) -> String {
    match mode {
        ParseMode::Bytes => format_bytes(bytes),
        ParseMode::Text => format_text_bytes(bytes),
        ParseMode::Json => serde_json::from_slice::<serde_json::Value>(bytes)
            .ok()
            .and_then(|value| serde_json::to_string_pretty(&value).ok())
            .unwrap_or_else(|| format_bytes(bytes)),
    }
}

fn format_bytes_with_mode_and_json_indent(
    bytes: &[u8],
    mode: ParseMode,
    json_indent_spaces: u8,
) -> String {
    match mode {
        ParseMode::Json => format_json_bytes(bytes, json_indent_spaces)
            .unwrap_or_else(|| format_bytes(bytes)),
        _ => format_bytes_with_mode(bytes, mode),
    }
}

fn format_json_bytes(bytes: &[u8], indent_spaces: u8) -> Option<String> {
    let value = serde_json::from_slice::<serde_json::Value>(bytes).ok()?;
    let indent = vec![b' '; indent_spaces as usize];
    let formatter = serde_json::ser::PrettyFormatter::with_indent(&indent);
    let mut formatted = Vec::new();
    let mut serializer = serde_json::Serializer::with_formatter(&mut formatted, formatter);
    serde::Serialize::serialize(&value, &mut serializer).ok()?;
    String::from_utf8(formatted).ok()
}

fn panel_header(title: impl Into<SharedString>, palette: Palette) -> gpui::Div {
    div()
        .flex_none()
        .px_3()
        .py_2()
        .text_sm()
        .bg(palette.header_bg)
        .text_color(palette.text)
        .border_b_1()
        .border_color(palette.subtle_border)
        .child(title.into())
}

fn empty_state(message: impl Into<SharedString>, palette: Palette) -> gpui::Div {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .text_color(palette.muted_text)
        .child(message.into())
}

fn primary_button(label: impl Into<SharedString>, palette: Palette) -> gpui::Div {
    div()
        .flex_none()
        .cursor_pointer()
        .rounded_sm()
        .px_3()
        .py_1()
        .text_sm()
        .text_color(rgb(0xffffff))
        .bg(palette.button_primary_bg)
        .hover(|style| style.bg(palette.button_primary_hover))
        .child(label.into())
}

fn compact_button(label: impl Into<SharedString>, palette: Palette) -> gpui::Div {
    div()
        .flex_none()
        .cursor_pointer()
        .rounded_sm()
        .px_2()
        .py_0p5()
        .text_xs()
        .text_color(palette.text)
        .bg(palette.button_compact_bg)
        .hover(|style| style.bg(palette.button_compact_hover))
        .child(label.into())
}

fn choice_button(label: impl Into<SharedString>, selected: bool, palette: Palette) -> gpui::Div {
    div()
        .flex_none()
        .cursor_pointer()
        .rounded_sm()
        .px_3()
        .py_1()
        .text_sm()
        .text_color(if selected {
            rgb(0xffffff)
        } else {
            palette.text
        })
        .bg(if selected {
            palette.button_primary_bg
        } else {
            palette.button_compact_bg
        })
        .hover(|style| {
            style.bg(if selected {
                palette.button_primary_hover
            } else {
                palette.button_compact_hover
            })
        })
        .child(label.into())
}

fn context_menu_button(label: impl Into<SharedString>, palette: Palette) -> gpui::Div {
    div()
        .w_full()
        .cursor_pointer()
        .rounded_sm()
        .px_3()
        .py_1()
        .text_sm()
        .hover(|style| style.bg(palette.hover_alt_bg))
        .child(label.into())
}

fn dropdown_option(label: impl Into<SharedString>, selected: bool, palette: Palette) -> gpui::Div {
    div()
        .w_full()
        .cursor_pointer()
        .px_3()
        .py_1()
        .text_sm()
        .text_color(if selected {
            rgb(0xffffff)
        } else {
            palette.text
        })
        .bg(if selected {
            palette.button_primary_bg
        } else {
            palette.surface_bg
        })
        .hover(|style| {
            style.bg(if selected {
                palette.button_primary_hover
            } else {
                palette.hover_alt_bg
            })
        })
        .child(label.into())
}

fn breadcrumb_button(label: String, palette: Palette) -> gpui::Div {
    div()
        .flex_none()
        .whitespace_nowrap()
        .cursor_pointer()
        .rounded_sm()
        .px_2()
        .py_0p5()
        .text_xs()
        .hover(|style| style.bg(palette.hover_alt_bg))
        .child(label)
}

fn preview_cell(text: String, palette: Palette) -> gpui::Div {
    div()
        .overflow_hidden()
        .whitespace_nowrap()
        .cursor_pointer()
        .rounded_sm()
        .px_2()
        .py_1()
        .hover(|style| style.bg(palette.hover_alt_bg))
        .child(text)
}

#[cfg(target_os = "windows")]
fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(target_os = "windows"))]
fn is_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}
