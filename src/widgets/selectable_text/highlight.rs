use super::{HighlightMode, JsonHighlightColors, LineState};
use gpui::{Pixels, SharedString, TextRun, Window};

pub(super) fn shape_lines(
    content: SharedString,
    font_size: Pixels,
    color: gpui::Hsla,
    highlight_mode: HighlightMode,
    wrap_width: Option<Pixels>,
    window: &Window,
) -> Vec<LineState> {
    let mut result = Vec::new();
    let content_str = content.as_ref();
    let mut start = 0;
    let font = window.text_style().font();

    for line_text in content_str.split('\n') {
        let end = start + line_text.len();
        let runs = build_runs_for_line(line_text, font.clone(), color, highlight_mode);
        let shaped = shape_single_line(
            SharedString::from(line_text.to_owned()),
            font_size,
            &runs,
            wrap_width,
            window,
        );

        result.push(LineState {
            start,
            end,
            layout: shaped,
            bounds: None,
        });

        start = end + 1;
    }

    if result.is_empty() {
        let shaped = shape_single_line(
            SharedString::new_static(""),
            font_size,
            &[TextRun {
                len: 0,
                font,
                color,
                background_color: None,
                underline: None,
                strikethrough: None,
            }],
            wrap_width,
            window,
        );
        result.push(LineState {
            start: 0,
            end: 0,
            layout: shaped,
            bounds: None,
        });
    }

    result
}

fn shape_single_line(
    line_text: SharedString,
    font_size: Pixels,
    runs: &[TextRun],
    wrap_width: Option<Pixels>,
    window: &Window,
) -> gpui::WrappedLine {
    window
        .text_system()
        .shape_text(line_text, font_size, runs, wrap_width, None)
        .ok()
        .and_then(|mut lines| lines.pop())
        .unwrap()
}

fn build_runs_for_line(
    line_text: &str,
    font: gpui::Font,
    default_color: gpui::Hsla,
    highlight_mode: HighlightMode,
) -> Vec<TextRun> {
    match highlight_mode {
        HighlightMode::Plain => vec![text_run(line_text.len(), font, default_color)],
        HighlightMode::Json(colors) => json_runs_for_line(line_text, font, colors),
    }
}

fn json_runs_for_line(
    line_text: &str,
    font: gpui::Font,
    colors: JsonHighlightColors,
) -> Vec<TextRun> {
    if line_text.is_empty() {
        return vec![text_run(0, font, colors.default)];
    }

    let mut runs = Vec::new();
    let mut index = 0;

    while index < line_text.len() {
        let current = line_text[index..].chars().next().unwrap();

        if current.is_whitespace() {
            let start = index;
            index = advance_while(line_text, index, |ch| ch.is_whitespace());
            push_run(&mut runs, index - start, font.clone(), colors.default);
            continue;
        }

        if current == '"' {
            let start = index;
            index += current.len_utf8();
            while index < line_text.len() {
                let ch = line_text[index..].chars().next().unwrap();
                index += ch.len_utf8();
                if ch == '\\' {
                    if index < line_text.len() {
                        let escaped = line_text[index..].chars().next().unwrap();
                        index += escaped.len_utf8();
                    }
                    continue;
                }
                if ch == '"' {
                    break;
                }
            }

            let next_non_whitespace = advance_while(line_text, index, |ch| ch.is_whitespace());
            let is_key = line_text[next_non_whitespace..].starts_with(':');
            push_run(
                &mut runs,
                index - start,
                font.clone(),
                if is_key { colors.key } else { colors.string },
            );
            continue;
        }

        if matches!(current, '{' | '}' | '[' | ']' | ':' | ',') {
            index += current.len_utf8();
            push_run(
                &mut runs,
                current.len_utf8(),
                font.clone(),
                colors.punctuation,
            );
            continue;
        }

        if current == '-' || current.is_ascii_digit() {
            let start = index;
            index = advance_while(line_text, index, |ch| {
                ch.is_ascii_digit() || matches!(ch, '-' | '+' | '.' | 'e' | 'E')
            });
            push_run(&mut runs, index - start, font.clone(), colors.number);
            continue;
        }

        if line_text[index..].starts_with("true") {
            index += 4;
            push_run(&mut runs, 4, font.clone(), colors.keyword);
            continue;
        }

        if line_text[index..].starts_with("false") {
            index += 5;
            push_run(&mut runs, 5, font.clone(), colors.keyword);
            continue;
        }

        if line_text[index..].starts_with("null") {
            index += 4;
            push_run(&mut runs, 4, font.clone(), colors.keyword);
            continue;
        }

        index += current.len_utf8();
        push_run(&mut runs, current.len_utf8(), font.clone(), colors.default);
    }

    runs
}

fn text_run(len: usize, font: gpui::Font, color: gpui::Hsla) -> TextRun {
    TextRun {
        len,
        font,
        color,
        background_color: None,
        underline: None,
        strikethrough: None,
    }
}

fn push_run(runs: &mut Vec<TextRun>, len: usize, font: gpui::Font, color: gpui::Hsla) {
    if len == 0 {
        return;
    }

    if let Some(last) = runs.last_mut()
        && last.color == color
    {
        last.len += len;
        return;
    }

    runs.push(text_run(len, font, color));
}

fn advance_while(text: &str, mut index: usize, predicate: impl Fn(char) -> bool) -> usize {
    while index < text.len() {
        let ch = text[index..].chars().next().unwrap();
        if !predicate(ch) {
            break;
        }
        index += ch.len_utf8();
    }

    index
}
