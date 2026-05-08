use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    app::{AppState, TestState},
    engine::stats::TypedChar,
    ui::{screens::centered, theme::Theme},
};

pub fn render(frame: &mut Frame, state: &AppState) {
    let AppState::Test(test) = state else {
        return;
    };
    let theme = Theme::new(&test.config.theme.name, &test.config.theme.accent);
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(frame.area());

    let width = root[0].width.saturating_sub(8).max(30);
    let lines = render_words(test, theme, width as usize);
    let height = (lines.len() as u16).clamp(5, 14);
    let text_area = centered(root[0], width, height);
    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::NONE).style(theme.bg))
        .wrap(ratatui::widgets::Wrap { trim: false })
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(paragraph, text_area);

    if !test.started() {
        let prompt = Paragraph::new(Line::from(Span::styled(
            "press any key to start",
            theme.accent,
        )))
        .alignment(ratatui::layout::Alignment::Center);
        let prompt_area = centered(root[0], 30, 1);
        frame.render_widget(prompt, prompt_area);
    }

    let live_wpm = if test.config.behavior.show_live_wpm {
        stat(theme, "wpm", test.stats.wpm)
    } else {
        Span::styled("wpm --", theme.text_dim)
    };
    let stats = Paragraph::new(Line::from(vec![
        live_wpm,
        Span::styled("   ", theme.text_dim),
        stat(theme, "acc", test.stats.accuracy),
        Span::styled("   ", theme.text_dim),
        Span::styled(test.time_label(), theme.accent),
        Span::styled("   ctrl+r restart   esc home", theme.text_dim),
    ]))
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(theme.surface)
            .style(theme.bg),
    )
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(stats, root[1]);
}

fn render_words(test: &TestState, theme: Theme, terminal_width: usize) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut current = Vec::new();
    let mut width = 0usize;
    let max_width = terminal_width.clamp(30, 92);

    for (word_idx, word) in test.words.iter().enumerate() {
        let word_len = word.chars().count() + 1;
        if width + word_len > max_width && !current.is_empty() {
            lines.push(Line::from(current));
            current = Vec::new();
            width = 0;
        }
        push_word(&mut current, test, theme, word_idx, word);
        current.push(Span::styled(" ", theme.text_dim));
        width += word_len;
    }
    if !current.is_empty() {
        lines.push(Line::from(current));
    }
    lines
}

fn push_word(
    spans: &mut Vec<Span<'static>>,
    test: &TestState,
    theme: Theme,
    word_idx: usize,
    word: &str,
) {
    let is_current = word_idx == test.current_word;
    let typed = test.typed.get(word_idx).map(Vec::as_slice).unwrap_or(&[]);
    let word_len = word.chars().count();
    for (char_idx, expected) in word.chars().enumerate() {
        let maybe_typed = typed.get(char_idx).copied();
        let is_cursor = is_current && char_idx == test.cursor_in_word;
        spans.push(char_span(
            theme,
            expected,
            maybe_typed,
            is_current,
            is_cursor,
            test.caret_visible,
        ));
    }
    for (extra_idx, extra) in typed.iter().skip(word_len).enumerate() {
        let char_idx = word_len + extra_idx;
        let is_cursor = is_current && char_idx == test.cursor_in_word;
        spans.push(char_span(
            theme,
            extra.ch,
            Some(*extra),
            is_current,
            is_cursor,
            test.caret_visible,
        ));
    }
    if is_current && test.cursor_in_word == word_len {
        spans.push(cursor_span(theme, " ", test.caret_visible));
    }
}

fn char_span(
    theme: Theme,
    expected: char,
    typed: Option<TypedChar>,
    is_current: bool,
    is_cursor: bool,
    caret_visible: bool,
) -> Span<'static> {
    if is_cursor {
        return cursor_span(theme, &expected.to_string(), caret_visible);
    }
    let style = match typed {
        Some(t) if t.ch == expected => theme.text_correct,
        Some(_) => theme.text_error,
        None => theme.text_dim,
    };
    let style = if is_current {
        style.add_modifier(Modifier::UNDERLINED)
    } else {
        style
    };
    Span::styled(expected.to_string(), style)
}

fn cursor_span(theme: Theme, text: &str, caret_visible: bool) -> Span<'static> {
    if caret_visible {
        Span::styled(text.to_string(), theme.text_cursor)
    } else {
        Span::styled(
            text.to_string(),
            Style::default().fg(theme.accent_color).bg(theme.bg_color),
        )
    }
}

fn stat<'a>(theme: Theme, label: &'a str, value: f64) -> Span<'a> {
    Span::styled(
        format!("{label} {:.0}", value),
        theme.text_correct.add_modifier(Modifier::BOLD),
    )
}
