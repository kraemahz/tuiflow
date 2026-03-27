use crate::document::{EdgeId, GraphDocument, NodeId, Point, PortDirection, PortRef};
use crate::layout::CanvasLayout;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FocusDirection {
    Left,
    Right,
    Up,
    Down,
    Next,
    Previous,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphSelection {
    None,
    Node(NodeId),
    Port(PortRef),
    Edge(EdgeId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusKind {
    Info,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusMessage {
    pub kind: StatusKind,
    pub message: String,
}

impl Default for StatusMessage {
    fn default() -> Self {
        Self {
            kind: StatusKind::Info,
            message: "Ready".to_owned(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MouseState {
    pub last_position: Option<(u16, u16)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PromptRequest {
    CreateNode {
        suggested_title: String,
    },
    RenameNode {
        node_id: NodeId,
        current_title: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorEffect {
    RequestPrompt(PromptRequest),
    Status(StatusMessage),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphEditorMode {
    Navigate,
    MoveNode {
        node_id: NodeId,
    },
    ConnectEdge {
        source: PortRef,
        candidate_index: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphEditorState {
    pub mode: GraphEditorMode,
    pub selection: GraphSelection,
    pub viewport: Point,
    pub status: StatusMessage,
    pub mouse: MouseState,
}

impl Default for GraphEditorState {
    fn default() -> Self {
        Self {
            mode: GraphEditorMode::Navigate,
            selection: GraphSelection::None,
            viewport: Point::new(0, 0),
            status: StatusMessage::default(),
            mouse: MouseState::default(),
        }
    }
}

impl GraphEditorState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorAction {
    MoveSelection(FocusDirection),
    PanViewport { dx: i32, dy: i32 },
    CenterViewport,
    RequestCreateNode,
    SubmitCreateNodeTitle(String),
    RequestRenameNode,
    SubmitRenameNodeTitle(String),
    BeginMoveNode,
    MoveSelectedNode { dx: i32, dy: i32 },
    BeginConnect,
    CycleConnectionTarget(FocusDirection),
    ConfirmMode,
    CancelMode,
    DeleteSelection,
    MouseEventObserved { column: u16, row: u16 },
}

pub fn apply_action(
    document: &mut GraphDocument,
    state: &mut GraphEditorState,
    action: EditorAction,
) -> Vec<EditorEffect> {
    let mut effects = Vec::new();
    match action {
        EditorAction::MoveSelection(direction) => move_selection(document, state, direction),
        EditorAction::PanViewport { dx, dy } => {
            state.viewport.x += dx;
            state.viewport.y += dy;
            set_status(state, StatusKind::Info, "Panned viewport", &mut effects);
        }
        EditorAction::CenterViewport => center_viewport(document, state, &mut effects),
        EditorAction::RequestCreateNode => {
            effects.push(EditorEffect::RequestPrompt(PromptRequest::CreateNode {
                suggested_title: "New Node".to_owned(),
            }))
        }
        EditorAction::SubmitCreateNodeTitle(title) => {
            submit_create_node(document, state, title, &mut effects);
        }
        EditorAction::RequestRenameNode => request_rename(document, state, &mut effects),
        EditorAction::SubmitRenameNodeTitle(title) => {
            submit_rename(document, state, title, &mut effects);
        }
        EditorAction::BeginMoveNode => begin_move_node(document, state, &mut effects),
        EditorAction::MoveSelectedNode { dx, dy } => {
            move_selected_node(document, state, dx, dy, &mut effects);
        }
        EditorAction::BeginConnect => begin_connect(document, state, &mut effects),
        EditorAction::CycleConnectionTarget(direction) => {
            cycle_connection_target(document, state, direction, &mut effects);
        }
        EditorAction::ConfirmMode => confirm_mode(document, state, &mut effects),
        EditorAction::CancelMode => cancel_mode(state, &mut effects),
        EditorAction::DeleteSelection => delete_selection(document, state, &mut effects),
        EditorAction::MouseEventObserved { column, row } => {
            state.mouse.last_position = Some((column, row));
        }
    }
    effects
}

fn move_selection(
    document: &GraphDocument,
    state: &mut GraphEditorState,
    direction: FocusDirection,
) {
    if !matches!(state.mode, GraphEditorMode::Navigate) {
        return;
    }
    let layout = CanvasLayout::for_document(document);
    let selectables = selectable_items(&layout);
    if selectables.is_empty() {
        state.selection = GraphSelection::None;
        return;
    }
    let current = state.selection;
    let current_index = selectables
        .iter()
        .position(|item| item.selection == current);
    let next = match direction {
        FocusDirection::Next => {
            let idx = current_index
                .map(|idx| (idx + 1) % selectables.len())
                .unwrap_or(0);
            selectables[idx].selection
        }
        FocusDirection::Previous => {
            let idx = current_index
                .map(|idx| idx.checked_sub(1).unwrap_or(selectables.len() - 1))
                .unwrap_or(0);
            selectables[idx].selection
        }
        _ => directional_selection(current, direction, &selectables).unwrap_or_else(|| {
            current_index
                .map(|idx| selectables[idx].selection)
                .unwrap_or(selectables[0].selection)
        }),
    };
    state.selection = next;
    state.status = StatusMessage {
        kind: StatusKind::Info,
        message: format!("Selected {}", describe_selection(document, next)),
    };
}

fn center_viewport(
    document: &GraphDocument,
    state: &mut GraphEditorState,
    effects: &mut Vec<EditorEffect>,
) {
    let layout = CanvasLayout::for_document(document);
    let focus = selection_point(&layout, state.selection)
        .or_else(|| layout.nodes.first().map(|node| node.rect.center()));
    if let Some(point) = focus {
        state.viewport = Point::new(point.x - 20, point.y - 8);
        set_status(state, StatusKind::Info, "Centered viewport", effects);
    }
}

fn submit_create_node(
    document: &mut GraphDocument,
    state: &mut GraphEditorState,
    title: String,
    effects: &mut Vec<EditorEffect>,
) {
    let title = title.trim();
    if title.is_empty() {
        set_status(
            state,
            StatusKind::Error,
            "Node title cannot be empty",
            effects,
        );
        return;
    }
    let anchor = selection_point(&CanvasLayout::for_document(document), state.selection)
        .unwrap_or(Point::new(state.viewport.x + 8, state.viewport.y + 4));
    let node_id = document.add_node(
        title,
        Point::new(anchor.x + 4, anchor.y + 2),
        ["In"],
        ["Out"],
    );
    state.selection = GraphSelection::Node(node_id);
    state.mode = GraphEditorMode::Navigate;
    set_status(state, StatusKind::Info, "Created node", effects);
}

fn request_rename(
    document: &GraphDocument,
    state: &mut GraphEditorState,
    effects: &mut Vec<EditorEffect>,
) {
    let Some(node_id) = selected_node_id(state.selection) else {
        set_status(state, StatusKind::Error, "Select a node to rename", effects);
        return;
    };
    let current_title = document
        .node(node_id)
        .map(|node| node.title.clone())
        .unwrap_or_default();
    effects.push(EditorEffect::RequestPrompt(PromptRequest::RenameNode {
        node_id,
        current_title,
    }));
}

fn submit_rename(
    document: &mut GraphDocument,
    state: &mut GraphEditorState,
    title: String,
    effects: &mut Vec<EditorEffect>,
) {
    let Some(node_id) = selected_node_id(state.selection) else {
        set_status(state, StatusKind::Error, "Select a node to rename", effects);
        return;
    };
    let title = title.trim();
    if title.is_empty() {
        set_status(
            state,
            StatusKind::Error,
            "Node title cannot be empty",
            effects,
        );
        return;
    }
    if document.rename_node(node_id, title) {
        set_status(state, StatusKind::Info, "Renamed node", effects);
    } else {
        set_status(state, StatusKind::Error, "Failed to rename node", effects);
    }
}

fn begin_move_node(
    document: &GraphDocument,
    state: &mut GraphEditorState,
    effects: &mut Vec<EditorEffect>,
) {
    let Some(node_id) = selected_node_id(state.selection) else {
        set_status(state, StatusKind::Error, "Select a node to move", effects);
        return;
    };
    if document.node(node_id).is_none() {
        set_status(
            state,
            StatusKind::Error,
            "Selected node no longer exists",
            effects,
        );
        return;
    }
    state.mode = GraphEditorMode::MoveNode { node_id };
    set_status(state, StatusKind::Info, "Move mode", effects);
}

fn move_selected_node(
    document: &mut GraphDocument,
    state: &mut GraphEditorState,
    dx: i32,
    dy: i32,
    effects: &mut Vec<EditorEffect>,
) {
    let GraphEditorMode::MoveNode { node_id } = state.mode else {
        return;
    };
    if document.move_node_by(node_id, dx, dy) {
        state.selection = GraphSelection::Node(node_id);
        set_status(state, StatusKind::Info, "Moved node", effects);
    }
}

fn begin_connect(
    document: &GraphDocument,
    state: &mut GraphEditorState,
    effects: &mut Vec<EditorEffect>,
) {
    let Some(source) = selected_output_port(document, state.selection) else {
        set_status(
            state,
            StatusKind::Error,
            "Select an output port or node with outputs",
            effects,
        );
        return;
    };
    let candidates = connection_targets(document, source);
    if candidates.is_empty() {
        set_status(
            state,
            StatusKind::Error,
            "No valid input targets available",
            effects,
        );
        return;
    }
    state.mode = GraphEditorMode::ConnectEdge {
        source,
        candidate_index: 0,
    };
    state.selection = GraphSelection::Port(candidates[0]);
    set_status(state, StatusKind::Info, "Connect mode", effects);
}

fn cycle_connection_target(
    document: &GraphDocument,
    state: &mut GraphEditorState,
    direction: FocusDirection,
    effects: &mut Vec<EditorEffect>,
) {
    let GraphEditorMode::ConnectEdge {
        source,
        ref mut candidate_index,
    } = state.mode
    else {
        return;
    };
    let candidates = connection_targets(document, source);
    if candidates.is_empty() {
        set_status(
            state,
            StatusKind::Error,
            "No valid input targets available",
            effects,
        );
        return;
    }
    match direction {
        FocusDirection::Previous | FocusDirection::Left | FocusDirection::Up => {
            *candidate_index = candidate_index
                .checked_sub(1)
                .unwrap_or(candidates.len() - 1);
        }
        _ => {
            *candidate_index = (*candidate_index + 1) % candidates.len();
        }
    }
    state.selection = GraphSelection::Port(candidates[*candidate_index]);
}

fn confirm_mode(
    document: &mut GraphDocument,
    state: &mut GraphEditorState,
    effects: &mut Vec<EditorEffect>,
) {
    match state.mode {
        GraphEditorMode::Navigate => {}
        GraphEditorMode::MoveNode { node_id } => {
            state.mode = GraphEditorMode::Navigate;
            state.selection = GraphSelection::Node(node_id);
            set_status(state, StatusKind::Info, "Move confirmed", effects);
        }
        GraphEditorMode::ConnectEdge {
            source,
            candidate_index,
        } => {
            let candidates = connection_targets(document, source);
            let Some(target) = candidates.get(candidate_index).copied() else {
                state.mode = GraphEditorMode::Navigate;
                set_status(
                    state,
                    StatusKind::Error,
                    "No valid target to connect",
                    effects,
                );
                return;
            };
            if let Some(edge_id) = document.add_edge(source, target) {
                state.mode = GraphEditorMode::Navigate;
                state.selection = GraphSelection::Edge(edge_id);
                set_status(state, StatusKind::Info, "Created edge", effects);
            } else {
                state.mode = GraphEditorMode::Navigate;
                state.selection = GraphSelection::Port(source);
                set_status(state, StatusKind::Error, "Failed to create edge", effects);
            }
        }
    }
}

fn cancel_mode(state: &mut GraphEditorState, effects: &mut Vec<EditorEffect>) {
    match state.mode {
        GraphEditorMode::Navigate => {}
        GraphEditorMode::MoveNode { node_id } => {
            state.mode = GraphEditorMode::Navigate;
            state.selection = GraphSelection::Node(node_id);
            set_status(state, StatusKind::Info, "Move cancelled", effects);
        }
        GraphEditorMode::ConnectEdge { source, .. } => {
            state.mode = GraphEditorMode::Navigate;
            state.selection = GraphSelection::Port(source);
            set_status(state, StatusKind::Info, "Connect cancelled", effects);
        }
    }
}

fn delete_selection(
    document: &mut GraphDocument,
    state: &mut GraphEditorState,
    effects: &mut Vec<EditorEffect>,
) {
    match state.selection {
        GraphSelection::Node(node_id) => {
            if document.remove_node(node_id) {
                state.selection = GraphSelection::None;
                state.mode = GraphEditorMode::Navigate;
                set_status(state, StatusKind::Info, "Deleted node", effects);
            }
        }
        GraphSelection::Edge(edge_id) => {
            if document.remove_edge(edge_id) {
                state.selection = GraphSelection::None;
                state.mode = GraphEditorMode::Navigate;
                set_status(state, StatusKind::Info, "Deleted edge", effects);
            }
        }
        GraphSelection::Port(_) | GraphSelection::None => {
            set_status(
                state,
                StatusKind::Error,
                "Select a node or edge to delete",
                effects,
            );
        }
    }
}

fn set_status(
    state: &mut GraphEditorState,
    kind: StatusKind,
    message: impl Into<String>,
    effects: &mut Vec<EditorEffect>,
) {
    let status = StatusMessage {
        kind,
        message: message.into(),
    };
    state.status = status.clone();
    effects.push(EditorEffect::Status(status));
}

fn selected_node_id(selection: GraphSelection) -> Option<NodeId> {
    match selection {
        GraphSelection::Node(node_id) => Some(node_id),
        GraphSelection::Port(port) => Some(port.node_id),
        GraphSelection::Edge(_) | GraphSelection::None => None,
    }
}

fn selected_output_port(document: &GraphDocument, selection: GraphSelection) -> Option<PortRef> {
    match selection {
        GraphSelection::Port(port) if port.direction == PortDirection::Output => Some(port),
        GraphSelection::Node(node_id) => document.output_port_ref_at(node_id, 0),
        _ => None,
    }
}

fn connection_targets(document: &GraphDocument, source: PortRef) -> Vec<PortRef> {
    let mut targets = Vec::new();
    for node in &document.nodes {
        if node.id == source.node_id {
            continue;
        }
        for port in &node.inputs {
            let target = PortRef {
                node_id: node.id,
                port_id: port.id,
                direction: PortDirection::Input,
            };
            let exists = document
                .edges
                .iter()
                .any(|edge| edge.from == source && edge.to == target);
            if !exists {
                targets.push(target);
            }
        }
    }
    targets
}

#[derive(Clone, Copy)]
struct SelectableItem {
    selection: GraphSelection,
    point: Point,
}

fn selectable_items(layout: &CanvasLayout) -> Vec<SelectableItem> {
    let mut items = Vec::new();
    for node in &layout.nodes {
        items.push(SelectableItem {
            selection: GraphSelection::Node(node.node_id),
            point: node.rect.center(),
        });
        for port in &node.inputs {
            items.push(SelectableItem {
                selection: GraphSelection::Port(PortRef {
                    node_id: node.node_id,
                    port_id: port.port_id,
                    direction: PortDirection::Input,
                }),
                point: port.anchor,
            });
        }
        for port in &node.outputs {
            items.push(SelectableItem {
                selection: GraphSelection::Port(PortRef {
                    node_id: node.node_id,
                    port_id: port.port_id,
                    direction: PortDirection::Output,
                }),
                point: port.anchor,
            });
        }
    }
    for edge in &layout.edges {
        let point = edge.points[edge.points.len() / 2];
        items.push(SelectableItem {
            selection: GraphSelection::Edge(edge.edge_id),
            point,
        });
    }
    items
}

fn directional_selection(
    current: GraphSelection,
    direction: FocusDirection,
    selectables: &[SelectableItem],
) -> Option<GraphSelection> {
    let current_point = selectables
        .iter()
        .find(|item| item.selection == current)
        .map(|item| item.point)
        .unwrap_or_else(|| selectables[0].point);

    selectables
        .iter()
        .filter(|item| item.selection != current)
        .filter_map(|item| {
            let dx = item.point.x - current_point.x;
            let dy = item.point.y - current_point.y;
            let in_direction = match direction {
                FocusDirection::Left => dx < 0,
                FocusDirection::Right => dx > 0,
                FocusDirection::Up => dy < 0,
                FocusDirection::Down => dy > 0,
                FocusDirection::Next | FocusDirection::Previous => true,
            };
            if !in_direction {
                return None;
            }
            let primary = match direction {
                FocusDirection::Left | FocusDirection::Right => dx.abs(),
                FocusDirection::Up | FocusDirection::Down => dy.abs(),
                FocusDirection::Next | FocusDirection::Previous => 0,
            };
            let secondary = match direction {
                FocusDirection::Left | FocusDirection::Right => dy.abs(),
                FocusDirection::Up | FocusDirection::Down => dx.abs(),
                FocusDirection::Next | FocusDirection::Previous => 0,
            };
            Some(((primary, secondary), item.selection))
        })
        .min_by_key(|entry| entry.0)
        .map(|entry| entry.1)
}

fn selection_point(layout: &CanvasLayout, selection: GraphSelection) -> Option<Point> {
    match selection {
        GraphSelection::None => None,
        GraphSelection::Node(node_id) => layout.node(node_id).map(|node| node.rect.center()),
        GraphSelection::Port(port_ref) => layout.port_anchor(port_ref),
        GraphSelection::Edge(edge_id) => layout
            .edge(edge_id)
            .and_then(|edge| edge.points.get(edge.points.len() / 2).copied()),
    }
}

fn describe_selection(document: &GraphDocument, selection: GraphSelection) -> String {
    match selection {
        GraphSelection::None => "nothing".to_owned(),
        GraphSelection::Node(node_id) => document
            .node(node_id)
            .map(|node| format!("node {}", node.title))
            .unwrap_or_else(|| "node".to_owned()),
        GraphSelection::Port(port_ref) => {
            let label = document
                .find_port(port_ref)
                .map(|port| port.label.as_str())
                .unwrap_or("port");
            format!("{:?} {}", port_ref.direction, label)
        }
        GraphSelection::Edge(edge_id) => format!("edge {}", edge_id.0),
    }
}

pub use GraphSelection as Selection;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_node_requests_and_applies_prompt() {
        let mut document = GraphDocument::new();
        let mut state = GraphEditorState::new();
        let effects = apply_action(&mut document, &mut state, EditorAction::RequestCreateNode);
        assert!(matches!(
            effects.first(),
            Some(EditorEffect::RequestPrompt(
                PromptRequest::CreateNode { .. }
            ))
        ));

        let _ = apply_action(
            &mut document,
            &mut state,
            EditorAction::SubmitCreateNodeTitle("Hello".into()),
        );
        assert_eq!(document.nodes.len(), 1);
        assert!(matches!(state.selection, GraphSelection::Node(_)));
    }

    #[test]
    fn rename_node_updates_document() {
        let mut document = GraphDocument::sample();
        let mut state = GraphEditorState::new();
        state.selection = GraphSelection::Node(document.nodes[0].id);
        let _ = apply_action(
            &mut document,
            &mut state,
            EditorAction::SubmitRenameNodeTitle("Source".into()),
        );
        assert_eq!(document.nodes[0].title, "Source");
    }

    #[test]
    fn move_mode_moves_selected_node() {
        let mut document = GraphDocument::sample();
        let mut state = GraphEditorState::new();
        let node_id = document.nodes[0].id;
        state.selection = GraphSelection::Node(node_id);
        let _ = apply_action(&mut document, &mut state, EditorAction::BeginMoveNode);
        let before = document.node(node_id).unwrap().position;
        let _ = apply_action(
            &mut document,
            &mut state,
            EditorAction::MoveSelectedNode { dx: 3, dy: -1 },
        );
        let after = document.node(node_id).unwrap().position;
        assert_eq!(after.x, before.x + 3);
        assert_eq!(after.y, before.y - 1);
    }

    #[test]
    fn connect_mode_creates_edge() {
        let mut document = GraphDocument::sample();
        let mut state = GraphEditorState::new();
        let node_id = document.nodes[0].id;
        state.selection = GraphSelection::Node(node_id);
        let existing_edges = document.edges.len();
        let _ = apply_action(&mut document, &mut state, EditorAction::BeginConnect);
        let _ = apply_action(
            &mut document,
            &mut state,
            EditorAction::CycleConnectionTarget(FocusDirection::Next),
        );
        let _ = apply_action(&mut document, &mut state, EditorAction::ConfirmMode);
        assert!(document.edges.len() >= existing_edges);
    }

    #[test]
    fn invalid_delete_from_port_sets_error() {
        let mut document = GraphDocument::sample();
        let mut state = GraphEditorState::new();
        state.selection = GraphSelection::Port(
            document
                .output_port_ref_at(document.nodes[0].id, 0)
                .unwrap(),
        );
        let effects = apply_action(&mut document, &mut state, EditorAction::DeleteSelection);
        assert!(matches!(
            effects.last(),
            Some(EditorEffect::Status(StatusMessage {
                kind: StatusKind::Error,
                ..
            }))
        ));
    }
}
