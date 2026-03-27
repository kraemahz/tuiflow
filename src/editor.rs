use crate::document::{EdgeId, GraphDocument, NodeId, Point, PortDirection, PortRef};
use crate::layout::CanvasLayout;

const UNDO_LIMIT: usize = 128;

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
        original_position: Point,
        current_position: Point,
    },
    ConnectEdge {
        source: PortRef,
        candidate_index: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UndoEntry {
    document: GraphDocument,
    selection: GraphSelection,
    mode: GraphEditorMode,
    viewport: Point,
    connection_focus_node: Option<NodeId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphEditorState {
    pub mode: GraphEditorMode,
    pub selection: GraphSelection,
    pub viewport: Point,
    pub status: StatusMessage,
    pub mouse: MouseState,
    pub connection_focus_node: Option<NodeId>,
    change_log: Vec<UndoEntry>,
}

impl Default for GraphEditorState {
    fn default() -> Self {
        Self {
            mode: GraphEditorMode::Navigate,
            selection: GraphSelection::None,
            viewport: Point::new(0, 0),
            status: StatusMessage::default(),
            mouse: MouseState::default(),
            connection_focus_node: None,
            change_log: Vec::new(),
        }
    }
}

impl GraphEditorState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn undo_depth(&self) -> usize {
        self.change_log.len()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorAction {
    MoveSelection(FocusDirection),
    ToggleConnectionSelection,
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
    Undo,
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
        EditorAction::ToggleConnectionSelection => {
            toggle_connection_selection(document, state, &mut effects)
        }
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
            record_undo(document, state);
            submit_create_node(document, state, title, &mut effects);
        }
        EditorAction::RequestRenameNode => request_rename(document, state, &mut effects),
        EditorAction::SubmitRenameNodeTitle(title) => {
            record_undo(document, state);
            submit_rename(document, state, title, &mut effects);
        }
        EditorAction::BeginMoveNode => begin_move_node(document, state, &mut effects),
        EditorAction::MoveSelectedNode { dx, dy } => move_selected_node(state, dx, dy, &mut effects),
        EditorAction::BeginConnect => begin_connect(document, state, &mut effects),
        EditorAction::CycleConnectionTarget(direction) => {
            cycle_connection_target(document, state, direction, &mut effects);
        }
        EditorAction::ConfirmMode => confirm_mode(document, state, &mut effects),
        EditorAction::CancelMode => cancel_mode(state, &mut effects),
        EditorAction::DeleteSelection => {
            record_undo(document, state);
            delete_selection(document, state, &mut effects);
        }
        EditorAction::Undo => undo(document, state, &mut effects),
        EditorAction::MouseEventObserved { column, row } => {
            state.mouse.last_position = Some((column, row));
        }
    }
    effects
}

fn record_undo(document: &GraphDocument, state: &mut GraphEditorState) {
    let (selection, mode, connection_focus_node) = match state.mode {
        GraphEditorMode::MoveNode { node_id, .. } => {
            (GraphSelection::Node(node_id), GraphEditorMode::Navigate, None)
        }
        _ => (state.selection, state.mode, state.connection_focus_node),
    };
    state.change_log.push(UndoEntry {
        document: document.clone(),
        selection,
        mode,
        viewport: state.viewport,
        connection_focus_node,
    });
    if state.change_log.len() > UNDO_LIMIT {
        state.change_log.remove(0);
    }
}

fn undo(
    document: &mut GraphDocument,
    state: &mut GraphEditorState,
    effects: &mut Vec<EditorEffect>,
) {
    let Some(entry) = state.change_log.pop() else {
        set_status(state, StatusKind::Error, "Nothing to undo", effects);
        return;
    };
    *document = entry.document;
    state.selection = entry.selection;
    state.mode = entry.mode;
    state.viewport = entry.viewport;
    state.connection_focus_node = entry.connection_focus_node;
    set_status(state, StatusKind::Info, "Undid last change", effects);
}

fn move_selection(
    document: &GraphDocument,
    state: &mut GraphEditorState,
    direction: FocusDirection,
) {
    if !matches!(state.mode, GraphEditorMode::Navigate) {
        return;
    }

    if let Some(node_id) = state.connection_focus_node {
        cycle_selected_connection(document, state, node_id, direction);
        return;
    }

    let layout = CanvasLayout::for_document(document);
    let nodes: Vec<_> = layout
        .nodes
        .iter()
        .map(|node| SelectableNode {
            node_id: node.node_id,
            point: node.rect.center(),
        })
        .collect();
    if nodes.is_empty() {
        state.selection = GraphSelection::None;
        return;
    }

    let current_node = selected_node_id(state.selection).unwrap_or(nodes[0].node_id);
    let current = nodes
        .iter()
        .find(|item| item.node_id == current_node)
        .copied()
        .unwrap_or(nodes[0]);

    let next = match direction {
        FocusDirection::Next => {
            let idx = nodes
                .iter()
                .position(|item| item.node_id == current.node_id)
                .unwrap_or(0);
            nodes[(idx + 1) % nodes.len()].node_id
        }
        FocusDirection::Previous => {
            let idx = nodes
                .iter()
                .position(|item| item.node_id == current.node_id)
                .unwrap_or(0);
            nodes[idx.checked_sub(1).unwrap_or(nodes.len() - 1)].node_id
        }
        _ => directional_node_selection(current, direction, &nodes).unwrap_or(current.node_id),
    };

    state.selection = GraphSelection::Node(next);
    state.status = StatusMessage {
        kind: StatusKind::Info,
        message: format!("Selected {}", describe_selection(document, state.selection)),
    };
}

fn toggle_connection_selection(
    document: &GraphDocument,
    state: &mut GraphEditorState,
    effects: &mut Vec<EditorEffect>,
) {
    if !matches!(state.mode, GraphEditorMode::Navigate) {
        return;
    }

    if let Some(node_id) = state.connection_focus_node {
        state.connection_focus_node = None;
        state.selection = GraphSelection::Node(node_id);
        set_status(
            state,
            StatusKind::Info,
            "Returned to node selection",
            effects,
        );
        return;
    }

    let Some(node_id) = selected_node_id(state.selection) else {
        set_status(state, StatusKind::Error, "Select a node first", effects);
        return;
    };
    let edges = connected_edges(document, node_id);
    let Some(edge_id) = edges.first().copied() else {
        set_status(
            state,
            StatusKind::Error,
            "Selected node has no connections",
            effects,
        );
        return;
    };
    state.connection_focus_node = Some(node_id);
    state.selection = GraphSelection::Edge(edge_id);
    set_status(
        state,
        StatusKind::Info,
        "Connection selection mode",
        effects,
    );
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
    state.connection_focus_node = None;
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
    let original_position = document.node(node_id).unwrap().position;
    state.mode = GraphEditorMode::MoveNode {
        node_id,
        original_position,
        current_position: original_position,
    };
    state.connection_focus_node = None;
    state.selection = GraphSelection::Node(node_id);
    set_status(state, StatusKind::Info, "Move mode", effects);
}

fn move_selected_node(
    state: &mut GraphEditorState,
    dx: i32,
    dy: i32,
    effects: &mut Vec<EditorEffect>,
) {
    let GraphEditorMode::MoveNode {
        node_id,
        original_position,
        mut current_position,
    } = state.mode
    else {
        return;
    };
    current_position.x += dx;
    current_position.y += dy;
    state.mode = GraphEditorMode::MoveNode {
        node_id,
        original_position,
        current_position,
    };
    state.selection = GraphSelection::Node(node_id);
    set_status(state, StatusKind::Info, "Previewing move", effects);
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
            "Select a node with outputs",
            effects,
            );
        return;
    };
    let candidates = sorted_connection_targets(document, source);
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
    state.connection_focus_node = None;
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
    let candidates = sorted_connection_targets(document, source);
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
        GraphEditorMode::MoveNode {
            node_id,
            current_position,
            ..
        } => {
            record_undo(document, state);
            let _ = document.set_node_position(node_id, current_position);
            state.mode = GraphEditorMode::Navigate;
            state.selection = GraphSelection::Node(node_id);
            state.connection_focus_node = None;
            set_status(state, StatusKind::Info, "Move confirmed", effects);
        }
        GraphEditorMode::ConnectEdge {
            source,
            candidate_index,
        } => {
            let candidates = sorted_connection_targets(document, source);
            let Some(target) = candidates.get(candidate_index).copied() else {
                state.mode = GraphEditorMode::Navigate;
                state.selection = GraphSelection::Node(source.node_id);
                set_status(
                    state,
                    StatusKind::Error,
                    "No valid target to connect",
                    effects,
                );
                return;
            };
            record_undo(document, state);
            if let Some(edge_id) = document.add_edge(source, target) {
                state.mode = GraphEditorMode::Navigate;
                state.selection = GraphSelection::Edge(edge_id);
                state.connection_focus_node = Some(source.node_id);
                set_status(state, StatusKind::Info, "Created edge", effects);
            } else {
                state.mode = GraphEditorMode::Navigate;
                state.selection = GraphSelection::Node(source.node_id);
                set_status(state, StatusKind::Error, "Failed to create edge", effects);
            }
        }
    }
}

fn cancel_mode(state: &mut GraphEditorState, effects: &mut Vec<EditorEffect>) {
    match state.mode {
        GraphEditorMode::Navigate => {}
        GraphEditorMode::MoveNode { node_id, .. } => {
            state.mode = GraphEditorMode::Navigate;
            state.selection = GraphSelection::Node(node_id);
            set_status(state, StatusKind::Info, "Move cancelled", effects);
        }
        GraphEditorMode::ConnectEdge { source, .. } => {
            state.mode = GraphEditorMode::Navigate;
            state.selection = GraphSelection::Node(source.node_id);
            set_status(state, StatusKind::Info, "Connect cancelled", effects);
        }
    }
    state.connection_focus_node = None;
}

fn delete_selection(
    document: &mut GraphDocument,
    state: &mut GraphEditorState,
    effects: &mut Vec<EditorEffect>,
) {
    match state.selection {
        GraphSelection::Node(node_id) => {
            if document.remove_node(node_id) {
                state.selection = document
                    .nodes
                    .first()
                    .map(|node| GraphSelection::Node(node.id))
                    .unwrap_or(GraphSelection::None);
                state.mode = GraphEditorMode::Navigate;
                state.connection_focus_node = None;
                set_status(state, StatusKind::Info, "Deleted node", effects);
            }
        }
        GraphSelection::Edge(edge_id) => {
            let focus_node = state.connection_focus_node;
            if document.remove_edge(edge_id) {
                state.mode = GraphEditorMode::Navigate;
                if let Some(node_id) = focus_node {
                    let remaining = connected_edges(document, node_id);
                    if let Some(next_edge) = remaining.first().copied() {
                        state.selection = GraphSelection::Edge(next_edge);
                        state.connection_focus_node = Some(node_id);
                    } else {
                        state.selection = GraphSelection::Node(node_id);
                        state.connection_focus_node = None;
                    }
                } else {
                    state.selection = GraphSelection::None;
                }
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

fn cycle_selected_connection(
    document: &GraphDocument,
    state: &mut GraphEditorState,
    node_id: NodeId,
    direction: FocusDirection,
) {
    let edges = connected_edges(document, node_id);
    if edges.is_empty() {
        state.connection_focus_node = None;
        state.selection = GraphSelection::Node(node_id);
        return;
    }

    let current = match state.selection {
        GraphSelection::Edge(edge_id) => edge_id,
        _ => edges[0],
    };
    let idx = edges
        .iter()
        .position(|edge_id| *edge_id == current)
        .unwrap_or(0);
    let next_idx = match direction {
        FocusDirection::Previous | FocusDirection::Left | FocusDirection::Up => {
            idx.checked_sub(1).unwrap_or(edges.len() - 1)
        }
        _ => (idx + 1) % edges.len(),
    };
    state.selection = GraphSelection::Edge(edges[next_idx]);
    state.status = StatusMessage {
        kind: StatusKind::Info,
        message: format!("Selected {}", describe_selection(document, state.selection)),
    };
}

fn connected_edges(document: &GraphDocument, node_id: NodeId) -> Vec<EdgeId> {
    let mut edges: Vec<_> = document
        .edges
        .iter()
        .filter(|edge| edge.from.node_id == node_id || edge.to.node_id == node_id)
        .map(|edge| edge.id)
        .collect();
    edges.sort();
    edges
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
        GraphSelection::Node(node_id) => best_output_port_for_node(document, node_id),
        GraphSelection::Edge(_) | GraphSelection::Port(_) | GraphSelection::None => None,
    }
}

pub(crate) fn sorted_connection_targets(document: &GraphDocument, source: PortRef) -> Vec<PortRef> {
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
    let layout = CanvasLayout::for_document(document);
    let source_anchor = layout.port_anchor(source);
    targets.sort_by_key(|target| {
        let distance = match (source_anchor, layout.port_anchor(*target)) {
            (Some(source_anchor), Some(target_anchor)) => {
                manhattan_distance(source_anchor, target_anchor)
            }
            _ => i32::MAX,
        };
        (distance, target.node_id.0, target.port_id.0)
    });
    targets
}

fn best_output_port_for_node(document: &GraphDocument, node_id: NodeId) -> Option<PortRef> {
    let node = document.node(node_id)?;
    if node.outputs.is_empty() {
        return None;
    }
    let layout = CanvasLayout::for_document(document);
    let mut best: Option<(i32, PortRef)> = None;
    for port in &node.outputs {
        let source = PortRef {
            node_id,
            port_id: port.id,
            direction: PortDirection::Output,
        };
        let score = sorted_connection_targets(document, source)
            .into_iter()
            .filter_map(|target| {
                Some(manhattan_distance(
                    layout.port_anchor(source)?,
                    layout.port_anchor(target)?,
                ))
            })
            .min()
            .unwrap_or(i32::MAX);
        match best {
            Some((best_score, best_port))
                if score > best_score || (score == best_score && source.port_id.0 >= best_port.port_id.0) => {}
            _ => best = Some((score, source)),
        }
    }
    best.map(|(_, source)| source)
}

fn manhattan_distance(a: Point, b: Point) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

#[derive(Clone, Copy)]
struct SelectableNode {
    node_id: NodeId,
    point: Point,
}

fn directional_node_selection(
    current: SelectableNode,
    direction: FocusDirection,
    nodes: &[SelectableNode],
) -> Option<NodeId> {
    nodes
        .iter()
        .filter(|item| item.node_id != current.node_id)
        .filter_map(|item| {
            let dx = item.point.x - current.point.x;
            let dy = item.point.y - current.point.y;
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
            Some(((primary, secondary), item.node_id))
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
    fn connect_defaults_to_nearest_target_for_selected_source() {
        let mut document = GraphDocument::sample();
        let mut state = GraphEditorState::new();
        state.selection = GraphSelection::Port(
            document
                .output_port_ref_at(document.nodes[0].id, 1)
                .unwrap(),
        );
        let source = selected_output_port(&document, state.selection).unwrap();
        let expected = sorted_connection_targets(&document, source).first().copied();
        let _ = apply_action(&mut document, &mut state, EditorAction::BeginConnect);
        assert_eq!(expected.map(GraphSelection::Port), Some(state.selection));
    }

    #[test]
    fn connect_from_node_picks_spatially_best_output() {
        let document = GraphDocument::sample();
        let node_id = document.nodes[0].id;
        let source = best_output_port_for_node(&document, node_id).unwrap();
        assert_eq!(source, document.output_port_ref_at(node_id, 1).unwrap());
    }

    #[test]
    fn tab_switches_from_node_to_connected_edge_and_back() {
        let mut document = GraphDocument::sample();
        let mut state = GraphEditorState::new();
        let node_id = document.nodes[0].id;
        state.selection = GraphSelection::Node(node_id);
        let _ = apply_action(
            &mut document,
            &mut state,
            EditorAction::ToggleConnectionSelection,
        );
        assert!(matches!(state.selection, GraphSelection::Edge(_)));
        assert_eq!(state.connection_focus_node, Some(node_id));

        let _ = apply_action(
            &mut document,
            &mut state,
            EditorAction::ToggleConnectionSelection,
        );
        assert_eq!(state.selection, GraphSelection::Node(node_id));
        assert_eq!(state.connection_focus_node, None);
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
        assert_eq!(document.node(node_id).unwrap().position, before);
        match state.mode {
            GraphEditorMode::MoveNode { current_position, .. } => {
                assert_eq!(current_position.x, before.x + 3);
                assert_eq!(current_position.y, before.y - 1);
            }
            _ => panic!("expected move preview mode"),
        }
    }

    #[test]
    fn delete_selected_edge_keeps_connection_navigation_alive() {
        let mut document = GraphDocument::sample();
        let mut state = GraphEditorState::new();
        let node_id = document.nodes[1].id;
        state.selection = GraphSelection::Node(node_id);
        let _ = apply_action(
            &mut document,
            &mut state,
            EditorAction::ToggleConnectionSelection,
        );
        let before = document.edges.len();
        let _ = apply_action(&mut document, &mut state, EditorAction::DeleteSelection);
        assert_eq!(document.edges.len(), before - 1);
        assert!(matches!(
            state.selection,
            GraphSelection::Edge(_) | GraphSelection::Node(_)
        ));
    }

    #[test]
    fn undo_restores_last_mutation() {
        let mut document = GraphDocument::sample();
        let mut state = GraphEditorState::new();
        let node_id = document.nodes[0].id;
        state.selection = GraphSelection::Node(node_id);
        let original = document.node(node_id).unwrap().position;
        let _ = apply_action(&mut document, &mut state, EditorAction::BeginMoveNode);
        let _ = apply_action(
            &mut document,
            &mut state,
            EditorAction::MoveSelectedNode { dx: 5, dy: 0 },
        );
        let _ = apply_action(&mut document, &mut state, EditorAction::ConfirmMode);
        let _ = apply_action(&mut document, &mut state, EditorAction::Undo);
        assert_eq!(document.node(node_id).unwrap().position, original);
    }

    #[test]
    fn cancel_move_restores_original_location() {
        let mut document = GraphDocument::sample();
        let mut state = GraphEditorState::new();
        let node_id = document.nodes[0].id;
        let original = document.node(node_id).unwrap().position;
        state.selection = GraphSelection::Node(node_id);
        let _ = apply_action(&mut document, &mut state, EditorAction::BeginMoveNode);
        let _ = apply_action(
            &mut document,
            &mut state,
            EditorAction::MoveSelectedNode { dx: 7, dy: 2 },
        );
        let _ = apply_action(&mut document, &mut state, EditorAction::CancelMode);
        assert_eq!(document.node(node_id).unwrap().position, original);
        assert_eq!(state.selection, GraphSelection::Node(node_id));
        assert!(matches!(state.mode, GraphEditorMode::Navigate));
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
