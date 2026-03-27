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

pub struct EditorShell<N, E> {
    document: GraphDocument<N, E>,
    state: GraphEditorState<N, E>,
    mapper: ActionMapper,
    theme: GraphTheme,
    prompt: Option<PromptState>,
}

impl<N, E> EditorShell<N, E>
where
    N: Clone + Default,
    E: Clone + Default,
{
    pub fn new(document: GraphDocument<N, E>) -> Self {
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

    pub fn document(&self) -> &GraphDocument<N, E> {
        &self.document
    }

    pub fn document_mut(&mut self) -> &mut GraphDocument<N, E> {
        &mut self.document
    }

    pub fn state(&self) -> &GraphEditorState<N, E> {
        &self.state
    }

    pub fn theme(&self) -> &GraphTheme {
        &self.theme
    }

    pub fn prompt_active(&self) -> bool {
        self.prompt.is_some()
    }

    pub fn handle_event(&mut self, event: &Event) -> Vec<EditorEffect<N, E>> {
        if self.prompt.is_some() {
            return self.handle_prompt_event(event);
        }

        let actions = self.mapper.map_event(event, &self.state);
        let mut external_effects = Vec::new();
        for action in actions {
            external_effects.extend(self.dispatch(action));
        }
        external_effects
    }

    pub fn dispatch(&mut self, action: EditorAction) -> Vec<EditorEffect<N, E>> {
        let effects = apply_action(&mut self.document, &mut self.state, action);
        self.apply_effects(effects)
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

    fn apply_effects(&mut self, effects: Vec<EditorEffect<N, E>>) -> Vec<EditorEffect<N, E>> {
        let mut external = Vec::new();
        for effect in effects {
            match effect {
                EditorEffect::RequestPrompt(request) => {
                    let input = match &request {
                        PromptRequest::CreateNode { suggested_title } => suggested_title.clone(),
                        PromptRequest::RenameNode { current_title, .. } => current_title.clone(),
                    };
                    self.prompt = Some(PromptState { request, input });
                }
                EditorEffect::Status(status) => self.state.status = status,
                other => external.push(other),
            }
        }
        external
    }

    fn handle_prompt_event(&mut self, event: &Event) -> Vec<EditorEffect<N, E>> {
        let Event::Key(KeyEvent { code, .. }) = event else {
            return Vec::new();
        };
        let Some(prompt) = &mut self.prompt else {
            return Vec::new();
        };

        match code {
            KeyCode::Char(ch) => {
                prompt.input.push(*ch);
                Vec::new()
            }
            KeyCode::Backspace => {
                prompt.input.pop();
                Vec::new()
            }
            KeyCode::Esc => {
                self.prompt = None;
                self.state.status = StatusMessage {
                    kind: StatusKind::Info,
                    message: "Prompt cancelled".to_owned(),
                };
                Vec::new()
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
            _ => Vec::new(),
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

#[cfg(test)]
mod tests {
    use crate::{EditorAction, editor::EditorEffect};

    use super::*;

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    struct NodeData {
        edits: u32,
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    struct EdgeData;

    #[test]
    fn create_node_prompt_starts_blank() {
        let mut shell = EditorShell::<NodeData, EdgeData>::new(GraphDocument::new());
        let effects = shell.dispatch(EditorAction::RequestCreateNode);
        assert!(effects.is_empty());
        assert!(shell.prompt_active());
        let prompt = shell.prompt.as_ref().unwrap();
        assert!(matches!(prompt.request, PromptRequest::CreateNode { .. }));
        assert!(prompt.input.is_empty());
    }

    #[test]
    fn shell_returns_external_edit_effects() {
        let mut shell = EditorShell::<NodeData, EdgeData>::new(GraphDocument::sample());
        let effects = shell.dispatch(EditorAction::ActivateSelection);
        assert!(
            effects
                .iter()
                .any(|effect| matches!(effect, EditorEffect::OpenNodeEditor { .. }))
        );
    }
}
