use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    app::AppState,
    ui::{screens::centered, theme::Theme},
};

pub fn render(frame: &mut Frame, state: &AppState) {
    let AppState::Home(home) = state else {
        return;
    };
    let theme = Theme::new(&home.config.theme.name, &home.config.theme.accent);
    frame.render_widget(Clear, frame.area());
    let area = centered(frame.area(), 78, 24);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(2),
            Constraint::Length(12),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    let title = Paragraph::new(vec![
        Line::from(Span::styled("typewriter", theme.accent)),
        Line::from(Span::styled("fast terminal typing test", theme.text_dim)),
    ])
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(title, rows[0]);

    let active = Paragraph::new(Line::from(vec![
        Span::styled("selected  ", theme.text_dim),
        Span::styled(home.selected_mode.label(), theme.key()),
        Span::styled(format!("   best {:.0} wpm", home.best_wpm), theme.text_dim),
    ]))
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(active, rows[1]);

    let modes = Paragraph::new(vec![
        line(theme, "1", "25 words", "4", "15 sec"),
        line(theme, "2", "50 words", "5", "30 sec"),
        line(theme, "3", "100 words", "6", "60 sec"),
        Line::from(vec![
            Span::styled(" 7 ", theme.key()),
            Span::raw("  quote"),
            Span::styled("     enter/space start", theme.text_dim),
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
    frame.render_widget(modes, rows[2]);

    let help = Paragraph::new(Line::from(vec![
        Span::styled(" enter ", theme.key()),
        Span::styled(" start   ", theme.text_dim),
        Span::styled(" esc ", theme.key()),
        Span::styled(" quit", theme.text_dim),
    ]))
    .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(help, rows[3]);
}

fn line<'a>(theme: Theme, a: &'a str, left: &'a str, b: &'a str, right: &'a str) -> Line<'a> {
    Line::from(vec![
        Span::styled(format!(" {a} "), theme.key()),
        Span::raw("  "),
        Span::raw(left),
        Span::styled("      ", theme.text_dim),
        Span::styled(format!(" {b} "), theme.key()),
        Span::raw("  "),
        Span::raw(right),
    ])
}
