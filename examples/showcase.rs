use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use tuiflow::{EditorEffect, EditorShell, GraphDocument, Selection};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ShowcaseNodeData {
    edit_count: u32,
    note: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ShowcaseEdgeData {
    edit_count: u32,
    label: String,
}

struct ShowcaseApp {
    editor: EditorShell<ShowcaseNodeData, ShowcaseEdgeData>,
    last_external_action: String,
}

impl ShowcaseApp {
    fn new() -> Self {
        Self {
            editor: EditorShell::new(GraphDocument::sample()),
            last_external_action: "Press Enter on a node or edge to edit payload data".to_owned(),
        }
    }

    fn handle_event(&mut self, event: &Event) {
        let effects = self.editor.handle_event(event);
        self.apply_external_effects(effects);
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        let [body, status] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
        let [sidebar, canvas] =
            Layout::horizontal([Constraint::Length(32), Constraint::Fill(1)]).areas(body);

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

    fn apply_external_effects(
        &mut self,
        effects: Vec<EditorEffect<ShowcaseNodeData, ShowcaseEdgeData>>,
    ) {
        for effect in effects {
            match effect {
                EditorEffect::OpenNodeEditor {
                    node_id,
                    title,
                    mut data,
                } => {
                    data.edit_count += 1;
                    if data.note.is_empty() {
                        data.note = format!("{} opened", title);
                    }
                    let _ = self
                        .editor
                        .document_mut()
                        .set_node_data(node_id, data.clone());
                    self.last_external_action =
                        format!("Updated node payload for {} ({})", title, data.edit_count);
                }
                EditorEffect::OpenEdgeEditor { edge_id, mut data } => {
                    data.edit_count += 1;
                    if data.label.is_empty() {
                        data.label = format!("Edge {}", edge_id.0);
                    }
                    let _ = self
                        .editor
                        .document_mut()
                        .set_edge_data(edge_id, data.clone());
                    self.last_external_action = format!(
                        "Updated edge payload for {} ({})",
                        edge_id.0, data.edit_count
                    );
                }
                EditorEffect::RequestPrompt(_) | EditorEffect::Status(_) => {}
            }
        }
    }

    fn sidebar_lines(&self) -> Vec<Line<'static>> {
        let mut lines = vec![
            Line::from("Showcase"),
            Line::from(""),
            Line::from("Keys"),
            Line::from("arrows switch nodes"),
            Line::from("tab toggle connections"),
            Line::from("enter edit payload"),
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
        match selection {
            Selection::None => lines.push(Line::from("None")),
            Selection::Node(node_id) => {
                if let Some(node) = self.editor.document().node(node_id) {
                    lines.push(Line::from(format!("Node: {}", node.title)));
                    lines.push(Line::from(format!("node edits: {}", node.data.edit_count)));
                    lines.push(Line::from(format!("note: {}", node.data.note)));
                }
            }
            Selection::Port(port) => {
                if let Some(port_def) = self.editor.document().find_port(port) {
                    lines.push(Line::from(format!(
                        "{:?}: {}",
                        port.direction, port_def.label
                    )));
                }
            }
            Selection::Edge(edge_id) => {
                if let Some(edge) = self.editor.document().edge(edge_id) {
                    lines.push(Line::from(format!("Edge: {}", edge_id.0)));
                    lines.push(Line::from(format!("edge edits: {}", edge.data.edit_count)));
                    lines.push(Line::from(format!("label: {}", edge.data.label)));
                }
            }
        }

        lines.push(Line::from(format!(
            "Undo depth: {}",
            self.editor.state().undo_depth()
        )));
        lines.push(Line::from(""));
        lines.push(Line::from("Last host action"));
        lines.push(Line::from(self.last_external_action.clone()));
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
            Span::raw(" | "),
            Span::raw(self.last_external_action.clone()),
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
        app.handle_event(&event);
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
