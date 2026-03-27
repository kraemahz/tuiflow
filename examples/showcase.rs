use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, Frame};
use tui_dnd::showcase::ShowcaseApp;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = run(&mut terminal);
    ratatui::restore();
    app_result
}

fn run(terminal: &mut DefaultTerminal) -> io::Result<()> {
    let mut app = ShowcaseApp::new();
    loop {
        terminal.draw(|frame| render(frame, &app))?;
        if !event::poll(Duration::from_millis(50))? {
            continue;
        }
        let event = event::read()?;
        if matches!(
            event,
            Event::Key(key)
                if key.kind == KeyEventKind::Press
                    && matches!(key.code, KeyCode::Char('q'))
        ) {
            break;
        }
        app.editor_mut().handle_event(&event);
    }
    Ok(())
}

fn render(frame: &mut Frame, app: &ShowcaseApp) {
    app.render(frame, frame.area());
}
