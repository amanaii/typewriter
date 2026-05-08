use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{BarChart, Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    app::AppState,
    ui::{screens::centered, theme::Theme},
};

pub fn render(frame: &mut Frame, state: &AppState) {
    let AppState::Results(results) = state else {
        return;
    };
    let theme = Theme::new(&results.config.theme.name, &results.config.theme.accent);
    frame.render_widget(Clear, frame.area());
    let area = centered(frame.area(), 82, 28);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(9),
            Constraint::Length(8),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(area);

    let title = Paragraph::new(vec![
        Line::from(Span::styled("result", theme.accent)),
        Line::from(Span::styled(results.mode.label(), theme.text_dim)),
    ])
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(title, rows[0]);

    let stats = Paragraph::new(vec![
        row(
            theme,
            "wpm",
            results.stats.wpm,
            "raw",
            results.stats.raw_wpm,
        ),
        row(
            theme,
            "accuracy",
            results.stats.accuracy,
            "consistency",
            results.stats.consistency,
        ),
        Line::from(vec![
            Span::styled("correct ", theme.text_dim),
            Span::styled(results.stats.correct_chars.to_string(), theme.accent),
            Span::styled("   incorrect ", theme.text_dim),
            Span::styled(results.stats.incorrect_chars.to_string(), theme.text_error),
            Span::styled("   extra ", theme.text_dim),
            Span::styled(results.stats.extra_chars.to_string(), theme.text_error),
            Span::styled("   missed ", theme.text_dim),
            Span::styled(results.stats.missed_chars.to_string(), theme.text_error),
        ]),
        Line::from(vec![
            Span::styled("time ", theme.text_dim),
            Span::styled(format!("{:.1}s", results.stats.elapsed_secs), theme.accent),
        ]),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(theme.surface)
            .style(theme.bg),
    )
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(stats, rows[1]);

    let labels: Vec<String> = results
        .wpm_samples
        .iter()
        .enumerate()
        .map(|(idx, _)| format!("{}", idx + 1))
        .collect();
    let bars: Vec<(&str, u64)> = results
        .wpm_samples
        .iter()
        .zip(labels.iter())
        .map(|(wpm, label)| (label.as_str(), wpm.round().max(0.0) as u64))
        .collect();
    let chart = BarChart::default()
        .block(
            Block::default()
                .title(" 2s wpm samples ")
                .borders(Borders::ALL)
                .border_style(theme.surface)
                .style(theme.bg),
        )
        .data(&bars)
        .bar_width(3)
        .bar_gap(1)
        .bar_style(Style::default().fg(theme.accent_color).bg(theme.bg_color))
        .value_style(Style::default().fg(theme.bg_color).bg(theme.accent_color));
    frame.render_widget(chart, rows[2]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(" tab ", theme.key()),
        Span::styled(" retry same config   ", theme.text_dim),
        Span::styled(" enter ", theme.key()),
        Span::styled(" new test   ", theme.text_dim),
        Span::styled(" esc ", theme.key()),
        Span::styled(" home", theme.text_dim),
    ]))
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(help, rows[3]);
}

fn row<'a>(theme: Theme, a: &'a str, av: f64, b: &'a str, bv: f64) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!("{a} "), theme.text_dim),
        Span::styled(
            format!("{av:.0}"),
            theme.accent.add_modifier(Modifier::BOLD),
        ),
        Span::styled("      ", theme.text_dim),
        Span::styled(format!("{b} "), theme.text_dim),
        Span::styled(
            format!("{bv:.0}"),
            theme.accent.add_modifier(Modifier::BOLD),
        ),
    ])
}
