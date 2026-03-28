use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};

use crate::editor::{EditorAction, FocusDirection, GraphEditorMode, GraphEditorState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyBinding {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBinding {
    pub const fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub const fn plain(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::NONE)
    }

    fn matches(self, key: KeyEvent) -> bool {
        self.code == key.code && self.modifiers == key.modifiers
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavigateBindings {
    pub move_left: Vec<KeyBinding>,
    pub move_right: Vec<KeyBinding>,
    pub move_up: Vec<KeyBinding>,
    pub move_down: Vec<KeyBinding>,
    pub toggle_connections: Vec<KeyBinding>,
    pub activate_selection: Vec<KeyBinding>,
    pub create_node: Vec<KeyBinding>,
    pub rename_node: Vec<KeyBinding>,
    pub begin_move_node: Vec<KeyBinding>,
    pub begin_connect: Vec<KeyBinding>,
    pub undo: Vec<KeyBinding>,
    pub delete_selection: Vec<KeyBinding>,
    pub center_viewport: Vec<KeyBinding>,
}

impl Default for NavigateBindings {
    fn default() -> Self {
        Self {
            move_left: vec![
                KeyBinding::plain(KeyCode::Left),
                KeyBinding::plain(KeyCode::Char('h')),
            ],
            move_right: vec![
                KeyBinding::plain(KeyCode::Right),
                KeyBinding::plain(KeyCode::Char('l')),
            ],
            move_up: vec![
                KeyBinding::plain(KeyCode::Up),
                KeyBinding::plain(KeyCode::Char('k')),
            ],
            move_down: vec![
                KeyBinding::plain(KeyCode::Down),
                KeyBinding::plain(KeyCode::Char('j')),
            ],
            toggle_connections: vec![
                KeyBinding::plain(KeyCode::Tab),
                KeyBinding::new(KeyCode::BackTab, KeyModifiers::SHIFT),
            ],
            activate_selection: vec![KeyBinding::plain(KeyCode::Enter)],
            create_node: vec![KeyBinding::plain(KeyCode::Char('n'))],
            rename_node: vec![KeyBinding::plain(KeyCode::Char('r'))],
            begin_move_node: vec![KeyBinding::plain(KeyCode::Char('m'))],
            begin_connect: vec![KeyBinding::plain(KeyCode::Char('c'))],
            undo: vec![KeyBinding::plain(KeyCode::Char('u'))],
            delete_selection: vec![
                KeyBinding::plain(KeyCode::Char('d')),
                KeyBinding::plain(KeyCode::Delete),
                KeyBinding::plain(KeyCode::Backspace),
            ],
            center_viewport: vec![KeyBinding::plain(KeyCode::Char('g'))],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoveNodeBindings {
    pub move_left: Vec<KeyBinding>,
    pub move_right: Vec<KeyBinding>,
    pub move_up: Vec<KeyBinding>,
    pub move_down: Vec<KeyBinding>,
    pub confirm: Vec<KeyBinding>,
    pub cancel: Vec<KeyBinding>,
}

impl Default for MoveNodeBindings {
    fn default() -> Self {
        Self {
            move_left: vec![
                KeyBinding::plain(KeyCode::Left),
                KeyBinding::plain(KeyCode::Char('h')),
            ],
            move_right: vec![
                KeyBinding::plain(KeyCode::Right),
                KeyBinding::plain(KeyCode::Char('l')),
            ],
            move_up: vec![
                KeyBinding::plain(KeyCode::Up),
                KeyBinding::plain(KeyCode::Char('k')),
            ],
            move_down: vec![
                KeyBinding::plain(KeyCode::Down),
                KeyBinding::plain(KeyCode::Char('j')),
            ],
            confirm: vec![KeyBinding::plain(KeyCode::Enter)],
            cancel: vec![KeyBinding::plain(KeyCode::Esc)],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectBindings {
    pub next_target: Vec<KeyBinding>,
    pub previous_target: Vec<KeyBinding>,
    pub confirm: Vec<KeyBinding>,
    pub cancel: Vec<KeyBinding>,
}

impl Default for ConnectBindings {
    fn default() -> Self {
        Self {
            next_target: vec![
                KeyBinding::plain(KeyCode::Tab),
                KeyBinding::plain(KeyCode::Right),
                KeyBinding::plain(KeyCode::Down),
                KeyBinding::plain(KeyCode::Char('l')),
                KeyBinding::plain(KeyCode::Char('j')),
            ],
            previous_target: vec![
                KeyBinding::new(KeyCode::BackTab, KeyModifiers::SHIFT),
                KeyBinding::plain(KeyCode::Left),
                KeyBinding::plain(KeyCode::Up),
                KeyBinding::plain(KeyCode::Char('h')),
                KeyBinding::plain(KeyCode::Char('k')),
            ],
            confirm: vec![KeyBinding::plain(KeyCode::Enter)],
            cancel: vec![KeyBinding::plain(KeyCode::Esc)],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewportBindings {
    pub pan_left: Vec<KeyBinding>,
    pub pan_right: Vec<KeyBinding>,
    pub pan_up: Vec<KeyBinding>,
    pub pan_down: Vec<KeyBinding>,
}

impl Default for ViewportBindings {
    fn default() -> Self {
        Self {
            pan_left: vec![KeyBinding::new(KeyCode::Left, KeyModifiers::SHIFT)],
            pan_right: vec![KeyBinding::new(KeyCode::Right, KeyModifiers::SHIFT)],
            pan_up: vec![KeyBinding::new(KeyCode::Up, KeyModifiers::SHIFT)],
            pan_down: vec![KeyBinding::new(KeyCode::Down, KeyModifiers::SHIFT)],
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InputMap {
    pub navigate: NavigateBindings,
    pub move_node: MoveNodeBindings,
    pub connect_edge: ConnectBindings,
    pub viewport: ViewportBindings,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MapResult {
    pub consumed: bool,
    pub actions: Vec<EditorAction>,
}

impl MapResult {
    fn action(action: EditorAction) -> Self {
        Self {
            consumed: true,
            actions: vec![action],
        }
    }

    fn none() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug, Default)]
pub struct ActionMapper {
    bindings: InputMap,
}

impl ActionMapper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_bindings(bindings: InputMap) -> Self {
        Self { bindings }
    }

    pub fn bindings(&self) -> &InputMap {
        &self.bindings
    }

    pub fn bindings_mut(&mut self) -> &mut InputMap {
        &mut self.bindings
    }

    pub fn map_event<N, E>(&self, event: &Event, state: &GraphEditorState<N, E>) -> MapResult {
        match event {
            Event::Key(key) => self.map_key(*key, state),
            Event::Mouse(mouse) => self.map_mouse(*mouse),
            _ => MapResult::none(),
        }
    }

    pub fn map_key<N, E>(&self, key: KeyEvent, state: &GraphEditorState<N, E>) -> MapResult {
        if let Some(action) = self.viewport_action(key) {
            return MapResult::action(action);
        }

        match state.mode {
            GraphEditorMode::Navigate => self.map_navigate_key(key),
            GraphEditorMode::MoveNode { .. } => self.map_move_node_key(key),
            GraphEditorMode::ConnectEdge { .. } => self.map_connect_key(key),
        }
    }

    fn map_mouse(&self, mouse: MouseEvent) -> MapResult {
        MapResult::action(EditorAction::MouseEventObserved {
            column: mouse.column,
            row: mouse.row,
        })
    }

    fn viewport_action(&self, key: KeyEvent) -> Option<EditorAction> {
        let bindings = &self.bindings.viewport;
        if matches_any(key, &bindings.pan_left) {
            return Some(EditorAction::PanViewport { dx: -2, dy: 0 });
        }
        if matches_any(key, &bindings.pan_right) {
            return Some(EditorAction::PanViewport { dx: 2, dy: 0 });
        }
        if matches_any(key, &bindings.pan_up) {
            return Some(EditorAction::PanViewport { dx: 0, dy: -1 });
        }
        if matches_any(key, &bindings.pan_down) {
            return Some(EditorAction::PanViewport { dx: 0, dy: 1 });
        }
        None
    }

    fn map_navigate_key(&self, key: KeyEvent) -> MapResult {
        let bindings = &self.bindings.navigate;
        if matches_any(key, &bindings.move_left) {
            return MapResult::action(EditorAction::MoveSelection(FocusDirection::Left));
        }
        if matches_any(key, &bindings.move_right) {
            return MapResult::action(EditorAction::MoveSelection(FocusDirection::Right));
        }
        if matches_any(key, &bindings.move_up) {
            return MapResult::action(EditorAction::MoveSelection(FocusDirection::Up));
        }
        if matches_any(key, &bindings.move_down) {
            return MapResult::action(EditorAction::MoveSelection(FocusDirection::Down));
        }
        if matches_any(key, &bindings.toggle_connections) {
            return MapResult::action(EditorAction::ToggleConnectionSelection);
        }
        if matches_any(key, &bindings.activate_selection) {
            return MapResult::action(EditorAction::ActivateSelection);
        }
        if matches_any(key, &bindings.create_node) {
            return MapResult::action(EditorAction::RequestCreateNode);
        }
        if matches_any(key, &bindings.rename_node) {
            return MapResult::action(EditorAction::RequestRenameNode);
        }
        if matches_any(key, &bindings.begin_move_node) {
            return MapResult::action(EditorAction::BeginMoveNode);
        }
        if matches_any(key, &bindings.begin_connect) {
            return MapResult::action(EditorAction::BeginConnect);
        }
        if matches_any(key, &bindings.undo) {
            return MapResult::action(EditorAction::Undo);
        }
        if matches_any(key, &bindings.delete_selection) {
            return MapResult::action(EditorAction::DeleteSelection);
        }
        if matches_any(key, &bindings.center_viewport) {
            return MapResult::action(EditorAction::CenterViewport);
        }
        MapResult::none()
    }

    fn map_move_node_key(&self, key: KeyEvent) -> MapResult {
        let bindings = &self.bindings.move_node;
        if matches_any(key, &bindings.move_left) {
            return MapResult::action(EditorAction::MoveSelectedNode { dx: -2, dy: 0 });
        }
        if matches_any(key, &bindings.move_right) {
            return MapResult::action(EditorAction::MoveSelectedNode { dx: 2, dy: 0 });
        }
        if matches_any(key, &bindings.move_up) {
            return MapResult::action(EditorAction::MoveSelectedNode { dx: 0, dy: -1 });
        }
        if matches_any(key, &bindings.move_down) {
            return MapResult::action(EditorAction::MoveSelectedNode { dx: 0, dy: 1 });
        }
        if matches_any(key, &bindings.confirm) {
            return MapResult::action(EditorAction::ConfirmMode);
        }
        if matches_any(key, &bindings.cancel) {
            return MapResult::action(EditorAction::CancelMode);
        }
        MapResult::none()
    }

    fn map_connect_key(&self, key: KeyEvent) -> MapResult {
        let bindings = &self.bindings.connect_edge;
        if matches_any(key, &bindings.next_target) {
            return MapResult::action(EditorAction::CycleConnectionTarget(FocusDirection::Next));
        }
        if matches_any(key, &bindings.previous_target) {
            return MapResult::action(EditorAction::CycleConnectionTarget(
                FocusDirection::Previous,
            ));
        }
        if matches_any(key, &bindings.confirm) {
            return MapResult::action(EditorAction::ConfirmMode);
        }
        if matches_any(key, &bindings.cancel) {
            return MapResult::action(EditorAction::CancelMode);
        }
        MapResult::none()
    }
}

fn matches_any(key: KeyEvent, bindings: &[KeyBinding]) -> bool {
    bindings.iter().copied().any(|binding| binding.matches(key))
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyEventKind, MouseButton, MouseEventKind};

    use super::*;
    use crate::editor::{GraphEditorMode, GraphEditorState};

    #[test]
    fn default_mapper_returns_expected_navigate_action() {
        let mapper = ActionMapper::new();
        let state = GraphEditorState::<(), ()>::new();
        let result = mapper.map_event(
            &Event::Key(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE)),
            &state,
        );
        assert!(result.consumed);
        assert_eq!(result.actions, vec![EditorAction::RequestCreateNode]);
    }

    #[test]
    fn unmapped_keys_are_not_consumed() {
        let mapper = ActionMapper::new();
        let state = GraphEditorState::<(), ()>::new();
        let result = mapper.map_key(
            KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE),
            &state,
        );
        assert!(!result.consumed);
        assert!(result.actions.is_empty());
    }

    #[test]
    fn custom_bindings_can_replace_defaults() {
        let mapper = ActionMapper::with_bindings(InputMap {
            navigate: NavigateBindings {
                create_node: vec![KeyBinding::plain(KeyCode::Char('a'))],
                ..NavigateBindings::default()
            },
            ..InputMap::default()
        });
        let state = GraphEditorState::<(), ()>::new();

        let custom = mapper.map_key(
            KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
            &state,
        );
        assert!(custom.consumed);
        assert_eq!(custom.actions, vec![EditorAction::RequestCreateNode]);

        let old = mapper.map_key(
            KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE),
            &state,
        );
        assert!(!old.consumed);
        assert!(old.actions.is_empty());
    }

    #[test]
    fn viewport_bindings_apply_across_modes() {
        let mapper = ActionMapper::new();
        let mut state = GraphEditorState::<(), ()>::new();
        state.mode = GraphEditorMode::MoveNode {
            node_id: crate::NodeId(1),
            original_position: crate::Point::new(0, 0),
            current_position: crate::Point::new(0, 0),
        };

        let result = mapper.map_key(KeyEvent::new(KeyCode::Left, KeyModifiers::SHIFT), &state);
        assert!(result.consumed);
        assert_eq!(
            result.actions,
            vec![EditorAction::PanViewport { dx: -2, dy: 0 }]
        );
    }

    #[test]
    fn mapper_tracks_mouse_without_editing() {
        let mapper = ActionMapper::new();
        let state = GraphEditorState::<(), ()>::new();
        let result = mapper.map_event(
            &Event::Mouse(MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: 5,
                row: 7,
                modifiers: KeyModifiers::NONE,
            }),
            &state,
        );
        assert!(result.consumed);
        assert_eq!(
            result.actions,
            vec![EditorAction::MouseEventObserved { column: 5, row: 7 }]
        );
        let _ = KeyEventKind::Press;
    }

    #[test]
    fn move_mode_uses_mode_specific_bindings() {
        let mapper = ActionMapper::new();
        let mut state = GraphEditorState::<(), ()>::new();
        state.mode = GraphEditorMode::MoveNode {
            node_id: crate::NodeId(1),
            original_position: crate::Point::new(0, 0),
            current_position: crate::Point::new(0, 0),
        };
        let result = mapper.map_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), &state);
        assert!(result.consumed);
        assert_eq!(result.actions, vec![EditorAction::ConfirmMode]);
    }
}
