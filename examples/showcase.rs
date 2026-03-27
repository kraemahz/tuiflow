use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use tui_dnd::{EditorShell, GraphDocument, Selection};

struct ShowcaseApp {
    editor: EditorShell,
}

impl ShowcaseApp {
    fn new() -> Self {
        Self {
            editor: EditorShell::new(GraphDocument::sample()),
        }
    }

    fn editor_mut(&mut self) -> &mut EditorShell {
        &mut self.editor
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let [body, status] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
        let [sidebar, canvas] =
            Layout::horizontal([Constraint::Length(30), Constraint::Fill(1)]).areas(body);

        frame.render_widget(
            Paragraph::new(self.sidebar_lines())
                .block(Block::default().title("Inspector").borders(Borders::ALL)),
            sidebar,
        );

        let graph_block = Block::default().title("Graph").borders(Borders::ALL);
        let inner = graph_block.inner(canvas);
        frame.render_widget(graph_block, canvas);
        self.editor.render(frame, inner);

        frame.render_widget(Paragraph::new(self.status_line()), status);
    }

    fn sidebar_lines(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from("Showcase"),
            Line::from(""),
            Line::from("Keys"),
            Line::from("arrows move nodes"),
            Line::from("tab toggle connections"),
            Line::from("n create  r rename"),
            Line::from("m move    c connect"),
            Line::from("d delete  u undo"),
            Line::from("g center"),
            Line::from("shift+arrows pan"),
            Line::from("enter/esc confirm/cancel"),
            Line::from(""),
            Line::from("Selected"),
        ];

        let selection = self.editor.state().selection;
        let detail = match selection {
            Selection::None => "None".to_owned(),
            Selection::Node(node_id) => self
                .editor
                .document()
                .node(node_id)
                .map(|node| format!("Node: {}", node.title))
                .unwrap_or_else(|| "Node".to_owned()),
            Selection::Port(port) => self
                .editor
                .document()
                .find_port(port)
                .map(|port_def| format!("{:?}: {}", port.direction, port_def.label))
                .unwrap_or_else(|| "Port".to_owned()),
            Selection::Edge(edge_id) => format!("Edge: {}", edge_id.0),
        };
        lines.push(Line::from(detail));
        lines.push(Line::from(format!(
            "Undo depth: {}",
            self.editor.state().undo_depth()
        )));
        lines
    }

    fn status_line(&self) -> Line<'static> {
        let state = self.editor.state();
        let mode = format!("{:?}", state.mode);
        let status = &state.status.message;
        Line::from(vec![
            Span::raw("mode "),
            Span::raw(mode),
            Span::raw(" | "),
            Span::raw(status.clone()),
            Span::raw(" | q quit"),
        ])
    }
}

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

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use ratatui::{Terminal, backend::TestBackend};

    use super::*;

    #[test]
    fn showcase_renders_editor_inside_bounded_center_pane() {
        let backend = TestBackend::new(100, 30);
        let mut terminal = Terminal::new(backend).unwrap();
        let app = ShowcaseApp::new();
        terminal
            .draw(|frame| app.render(frame, frame.area()))
            .unwrap();
        let buffer = terminal.backend().buffer();
        let mut lines = Vec::new();
        for y in 0..buffer.area.height {
            let mut line = String::new();
            for x in 0..buffer.area.width {
                line.push_str(buffer[(x, y)].symbol());
            }
            lines.push(line.trim_end().to_owned());
        }
        assert_snapshot!(lines.join("\n"));
    }
}
