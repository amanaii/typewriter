use ratatui::style::{Color, Modifier, Style};

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub bg: Style,
    pub surface: Style,
    pub text_dim: Style,
    pub text_correct: Style,
    pub text_error: Style,
    pub text_cursor: Style,
    pub accent: Style,
    pub bg_color: Color,
    pub accent_color: Color,
}

impl Theme {
    pub fn new(name: &str, accent: &str) -> Self {
        let accent_color = parse_hex(accent).unwrap_or(Color::Rgb(226, 183, 20));
        let (bg, surface, dim, text, error, cursor) = match name {
            "light" => (
                Color::Rgb(246, 247, 242),
                Color::Rgb(226, 228, 220),
                Color::Rgb(116, 120, 112),
                Color::Rgb(24, 26, 27),
                Color::Rgb(190, 45, 54),
                Color::Rgb(246, 247, 242),
            ),
            "catppuccin" | "catppuccin-mocha" => (
                Color::Rgb(30, 30, 46),
                Color::Rgb(49, 50, 68),
                Color::Rgb(127, 132, 156),
                Color::Rgb(205, 214, 244),
                Color::Rgb(243, 139, 168),
                Color::Rgb(30, 30, 46),
            ),
            "nord" => (
                Color::Rgb(46, 52, 64),
                Color::Rgb(59, 66, 82),
                Color::Rgb(129, 161, 193),
                Color::Rgb(236, 239, 244),
                Color::Rgb(191, 97, 106),
                Color::Rgb(46, 52, 64),
            ),
            "dracula" => (
                Color::Rgb(40, 42, 54),
                Color::Rgb(68, 71, 90),
                Color::Rgb(98, 114, 164),
                Color::Rgb(248, 248, 242),
                Color::Rgb(255, 85, 85),
                Color::Rgb(40, 42, 54),
            ),
            _ => (
                Color::Rgb(9, 11, 15),
                Color::Rgb(18, 22, 30),
                Color::Rgb(92, 101, 116),
                Color::Rgb(236, 240, 246),
                Color::Rgb(255, 91, 110),
                Color::Rgb(9, 11, 15),
            ),
        };

        Self {
            bg: Style::default().fg(text).bg(bg),
            surface: Style::default().fg(text).bg(surface),
            text_dim: Style::default().fg(dim).bg(bg),
            text_correct: Style::default().fg(text).bg(bg),
            text_error: Style::default().fg(error).bg(Color::Rgb(82, 24, 34)),
            text_cursor: Style::default()
                .fg(cursor)
                .bg(accent_color)
                .add_modifier(Modifier::BOLD),
            accent: Style::default()
                .fg(accent_color)
                .bg(bg)
                .add_modifier(Modifier::BOLD),
            bg_color: bg,
            accent_color,
        }
    }

    pub fn key(self) -> Style {
        Style::default()
            .fg(self.bg_color)
            .bg(self.accent_color)
            .add_modifier(Modifier::BOLD)
    }
}

fn parse_hex(input: &str) -> Option<Color> {
    let hex = input.strip_prefix('#').unwrap_or(input);
    if hex.len() != 6 {
        return None;
    }
    let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(Color::Rgb(red, green, blue))
}
