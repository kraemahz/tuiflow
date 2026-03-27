use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::{
    document::GraphDocument,
    editor::{
        EditorAction, EditorEffect, GraphEditorState, PromptRequest, StatusKind, StatusMessage,
        apply_action,
    },
    input::ActionMapper,
    render::GraphCanvas,
    theme::GraphTheme,
};

#[derive(Clone, Debug)]
struct PromptState {
    request: PromptRequest,
    input: String,
}

pub struct EditorShell {
    document: GraphDocument,
    state: GraphEditorState,
    mapper: ActionMapper,
    theme: GraphTheme,
    prompt: Option<PromptState>,
}

impl EditorShell {
    pub fn new(document: GraphDocument) -> Self {
        let mut state = GraphEditorState::new();
        if let Some(node) = document.nodes.first() {
            state.selection = crate::editor::Selection::Node(node.id);
        }
        Self {
            document,
            state,
            mapper: ActionMapper::new(),
            theme: GraphTheme::default(),
            prompt: None,
        }
    }

    pub fn document(&self) -> &GraphDocument {
        &self.document
    }

    pub fn state(&self) -> &GraphEditorState {
        &self.state
    }

    pub fn theme(&self) -> &GraphTheme {
        &self.theme
    }

    pub fn prompt_active(&self) -> bool {
        self.prompt.is_some()
    }

    pub fn handle_event(&mut self, event: &Event) {
        if self.prompt.is_some() {
            self.handle_prompt_event(event);
            return;
        }

        let actions = self.mapper.map_event(event, &self.state);
        for action in actions {
            self.dispatch(action);
        }
    }

    pub fn dispatch(&mut self, action: EditorAction) {
        let effects = apply_action(&mut self.document, &mut self.state, action);
        self.apply_effects(effects);
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            GraphCanvas::new(&self.document, &self.state, &self.theme),
            area,
        );
        if let Some(prompt) = &self.prompt {
            render_prompt(frame, area, prompt, &self.theme);
        }
    }

    fn apply_effects(&mut self, effects: Vec<EditorEffect>) {
        for effect in effects {
            match effect {
                EditorEffect::RequestPrompt(request) => {
                    let input = match &request {
                        PromptRequest::CreateNode { .. } => String::new(),
                        PromptRequest::RenameNode { current_title, .. } => current_title.clone(),
                    };
                    self.prompt = Some(PromptState { request, input });
                }
                EditorEffect::Status(status) => self.state.status = status,
            }
        }
    }

    fn handle_prompt_event(&mut self, event: &Event) {
        let Event::Key(KeyEvent { code, .. }) = event else {
            return;
        };
        let Some(prompt) = &mut self.prompt else {
            return;
        };

        match code {
            KeyCode::Char(ch) => prompt.input.push(*ch),
            KeyCode::Backspace => {
                prompt.input.pop();
            }
            KeyCode::Esc => {
                self.prompt = None;
                self.state.status = StatusMessage {
                    kind: StatusKind::Info,
                    message: "Prompt cancelled".to_owned(),
                };
            }
            KeyCode::Enter => {
                let request = prompt.request.clone();
                let value = prompt.input.clone();
                self.prompt = None;
                match request {
                    PromptRequest::CreateNode { .. } => {
                        self.dispatch(EditorAction::SubmitCreateNodeTitle(value))
                    }
                    PromptRequest::RenameNode { .. } => {
                        self.dispatch(EditorAction::SubmitRenameNodeTitle(value))
                    }
                }
            }
            _ => {}
        }
    }
}

fn render_prompt(frame: &mut Frame, area: Rect, prompt: &PromptState, theme: &GraphTheme) {
    let popup = centered_rect(area, 50, 5);
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Block::default()
            .title(match prompt.request {
                PromptRequest::CreateNode { .. } => "Create Node",
                PromptRequest::RenameNode { .. } => "Rename Node",
            })
            .borders(Borders::ALL)
            .border_style(theme.selected),
        popup,
    );
    let inner = popup.inner(ratatui::layout::Margin::new(1, 1));
    let lines = vec![
        Line::from(Span::styled("Enter title and press Enter", theme.muted)),
        Line::from(prompt.input.as_str()),
    ];
    frame.render_widget(Paragraph::new(lines).alignment(Alignment::Left), inner);
}

fn centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height),
        Constraint::Fill(1),
    ]);
    let horizontal = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(width.min(area.width.saturating_sub(2))),
        Constraint::Fill(1),
    ]);
    let [_, middle, _] = vertical.areas(area);
    let [_, center, _] = horizontal.areas(middle);
    center
}
