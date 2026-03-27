use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};

use crate::editor::{EditorAction, FocusDirection, GraphEditorMode, GraphEditorState};

#[derive(Default)]
pub struct ActionMapper;

impl ActionMapper {
    pub fn new() -> Self {
        Self
    }

    pub fn map_event<N, E>(
        &self,
        event: &Event,
        state: &GraphEditorState<N, E>,
    ) -> Vec<EditorAction> {
        match event {
            Event::Key(key) => self.map_key(*key, state),
            Event::Mouse(mouse) => self.map_mouse(*mouse),
            _ => Vec::new(),
        }
    }

    fn map_key<N, E>(&self, key: KeyEvent, state: &GraphEditorState<N, E>) -> Vec<EditorAction> {
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            return match key.code {
                KeyCode::Left => vec![EditorAction::PanViewport { dx: -2, dy: 0 }],
                KeyCode::Right => vec![EditorAction::PanViewport { dx: 2, dy: 0 }],
                KeyCode::Up => vec![EditorAction::PanViewport { dx: 0, dy: -1 }],
                KeyCode::Down => vec![EditorAction::PanViewport { dx: 0, dy: 1 }],
                _ => Vec::new(),
            };
        }

        match state.mode {
            GraphEditorMode::MoveNode { .. } => map_move_mode(key),
            GraphEditorMode::ConnectEdge { .. } => map_connect_mode(key),
            GraphEditorMode::Navigate => map_navigate_mode(key),
        }
    }

    fn map_mouse(&self, mouse: MouseEvent) -> Vec<EditorAction> {
        vec![EditorAction::MouseEventObserved {
            column: mouse.column,
            row: mouse.row,
        }]
    }
}

fn map_navigate_mode(key: KeyEvent) -> Vec<EditorAction> {
    match key.code {
        KeyCode::Left | KeyCode::Char('h') => {
            vec![EditorAction::MoveSelection(FocusDirection::Left)]
        }
        KeyCode::Right | KeyCode::Char('l') => {
            vec![EditorAction::MoveSelection(FocusDirection::Right)]
        }
        KeyCode::Up | KeyCode::Char('k') => vec![EditorAction::MoveSelection(FocusDirection::Up)],
        KeyCode::Down | KeyCode::Char('j') => {
            vec![EditorAction::MoveSelection(FocusDirection::Down)]
        }
        KeyCode::Tab | KeyCode::BackTab => vec![EditorAction::ToggleConnectionSelection],
        KeyCode::Enter => vec![EditorAction::ActivateSelection],
        KeyCode::Char('n') => vec![EditorAction::RequestCreateNode],
        KeyCode::Char('r') => vec![EditorAction::RequestRenameNode],
        KeyCode::Char('m') => vec![EditorAction::BeginMoveNode],
        KeyCode::Char('c') => vec![EditorAction::BeginConnect],
        KeyCode::Char('u') => vec![EditorAction::Undo],
        KeyCode::Char('d') | KeyCode::Delete | KeyCode::Backspace => {
            vec![EditorAction::DeleteSelection]
        }
        KeyCode::Char('g') => vec![EditorAction::CenterViewport],
        _ => Vec::new(),
    }
}

fn map_move_mode(key: KeyEvent) -> Vec<EditorAction> {
    match key.code {
        KeyCode::Left | KeyCode::Char('h') => {
            vec![EditorAction::MoveSelectedNode { dx: -2, dy: 0 }]
        }
        KeyCode::Right | KeyCode::Char('l') => {
            vec![EditorAction::MoveSelectedNode { dx: 2, dy: 0 }]
        }
        KeyCode::Up | KeyCode::Char('k') => vec![EditorAction::MoveSelectedNode { dx: 0, dy: -1 }],
        KeyCode::Down | KeyCode::Char('j') => vec![EditorAction::MoveSelectedNode { dx: 0, dy: 1 }],
        KeyCode::Enter => vec![EditorAction::ConfirmMode],
        KeyCode::Esc => vec![EditorAction::CancelMode],
        _ => Vec::new(),
    }
}

fn map_connect_mode(key: KeyEvent) -> Vec<EditorAction> {
    match key.code {
        KeyCode::Tab | KeyCode::Right | KeyCode::Down | KeyCode::Char('l') | KeyCode::Char('j') => {
            vec![EditorAction::CycleConnectionTarget(FocusDirection::Next)]
        }
        KeyCode::BackTab
        | KeyCode::Left
        | KeyCode::Up
        | KeyCode::Char('h')
        | KeyCode::Char('k') => {
            vec![EditorAction::CycleConnectionTarget(
                FocusDirection::Previous,
            )]
        }
        KeyCode::Enter => vec![EditorAction::ConfirmMode],
        KeyCode::Esc => vec![EditorAction::CancelMode],
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyEventKind, MouseButton, MouseEventKind};

    use super::*;
    use crate::editor::GraphEditorState;

    #[test]
    fn mapper_returns_expected_navigate_action() {
        let mapper = ActionMapper::new();
        let state = GraphEditorState::<(), ()>::new();
        let actions = mapper.map_event(
            &Event::Key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE)),
            &state,
        );
        assert_eq!(actions, vec![EditorAction::RequestCreateNode]);
    }

    #[test]
    fn mapper_tracks_mouse_without_editing() {
        let mapper = ActionMapper::new();
        let state = GraphEditorState::<(), ()>::new();
        let actions = mapper.map_event(
            &Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 5,
                row: 7,
                modifiers: KeyModifiers::NONE,
            }),
            &state,
        );
        assert_eq!(
            actions,
            vec![EditorAction::MouseEventObserved { column: 5, row: 7 }]
        );
        let _ = KeyEventKind::Press;
    }

    #[test]
    fn mapper_uses_enter_to_activate_selection() {
        let mapper = ActionMapper::new();
        let state = GraphEditorState::<(), ()>::new();
        let actions = mapper.map_event(
            &Event::Key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            &state,
        );
        assert_eq!(actions, vec![EditorAction::ActivateSelection]);
    }
}
