use ratatui::style::{Color, Modifier, Style};

#[derive(Clone, Debug)]
pub struct GraphTheme {
    pub text: Style,
    pub muted: Style,
    pub accent: Style,
    pub border: Style,
    pub selected: Style,
    pub error: Style,
    pub edge: Style,
    pub edge_selected: Style,
}

impl Default for GraphTheme {
    fn default() -> Self {
        Self {
            text: Style::default().fg(Color::White),
            muted: Style::default().fg(Color::DarkGray),
            accent: Style::default()
                .fg(Color::Rgb(138, 180, 248))
                .add_modifier(Modifier::BOLD),
            border: Style::default().fg(Color::Rgb(138, 180, 248)),
            selected: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            error: Style::default()
                .fg(Color::LightRed)
                .add_modifier(Modifier::BOLD),
            edge: Style::default().fg(Color::Cyan),
            edge_selected: Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        }
    }
}
