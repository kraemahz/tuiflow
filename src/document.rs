use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct NodeId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct PortId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct EdgeId(pub u64);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortRef {
    pub node_id: NodeId,
    pub port_id: PortId,
    pub direction: PortDirection,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphPort {
    pub id: PortId,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: NodeId,
    pub title: String,
    pub position: Point,
    pub inputs: Vec<GraphPort>,
    pub outputs: Vec<GraphPort>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: EdgeId,
    pub from: PortRef,
    pub to: PortRef,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphDocument {
    pub next_node_id: u64,
    pub next_port_id: u64,
    pub next_edge_id: u64,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl Default for GraphDocument {
    fn default() -> Self {
        Self {
            next_node_id: 1,
            next_port_id: 1,
            next_edge_id: 1,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

impl GraphDocument {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_node(
        &mut self,
        title: impl Into<String>,
        position: Point,
        inputs: impl IntoIterator<Item = impl Into<String>>,
        outputs: impl IntoIterator<Item = impl Into<String>>,
    ) -> NodeId {
        let node_id = self.alloc_node_id();
        let inputs = inputs
            .into_iter()
            .map(|label| self.make_port(label))
            .collect();
        let outputs = outputs
            .into_iter()
            .map(|label| self.make_port(label))
            .collect();
        self.nodes.push(GraphNode {
            id: node_id,
            title: title.into(),
            position,
            inputs,
            outputs,
        });
        node_id
    }

    pub fn add_edge(&mut self, from: PortRef, to: PortRef) -> Option<EdgeId> {
        if from.direction != PortDirection::Output || to.direction != PortDirection::Input {
            return None;
        }
        if from.node_id == to.node_id {
            return None;
        }
        if self.find_port(from).is_none() || self.find_port(to).is_none() {
            return None;
        }
        if self
            .edges
            .iter()
            .any(|edge| edge.from == from && edge.to == to)
        {
            return None;
        }

        let edge_id = self.alloc_edge_id();
        self.edges.push(GraphEdge {
            id: edge_id,
            from,
            to,
        });
        Some(edge_id)
    }

    pub fn remove_node(&mut self, node_id: NodeId) -> bool {
        let before = self.nodes.len();
        self.nodes.retain(|node| node.id != node_id);
        if self.nodes.len() == before {
            return false;
        }
        self.edges
            .retain(|edge| edge.from.node_id != node_id && edge.to.node_id != node_id);
        true
    }

    pub fn remove_edge(&mut self, edge_id: EdgeId) -> bool {
        let before = self.edges.len();
        self.edges.retain(|edge| edge.id != edge_id);
        self.edges.len() != before
    }

    pub fn rename_node(&mut self, node_id: NodeId, title: impl Into<String>) -> bool {
        let Some(node) = self.node_mut(node_id) else {
            return false;
        };
        node.title = title.into();
        true
    }

    pub fn move_node_by(&mut self, node_id: NodeId, dx: i32, dy: i32) -> bool {
        let Some(node) = self.node_mut(node_id) else {
            return false;
        };
        node.position.x += dx;
        node.position.y += dy;
        true
    }

    pub fn set_node_position(&mut self, node_id: NodeId, position: Point) -> bool {
        let Some(node) = self.node_mut(node_id) else {
            return false;
        };
        node.position = position;
        true
    }

    pub fn node(&self, node_id: NodeId) -> Option<&GraphNode> {
        self.nodes.iter().find(|node| node.id == node_id)
    }

    pub fn node_mut(&mut self, node_id: NodeId) -> Option<&mut GraphNode> {
        self.nodes.iter_mut().find(|node| node.id == node_id)
    }

    pub fn edge(&self, edge_id: EdgeId) -> Option<&GraphEdge> {
        self.edges.iter().find(|edge| edge.id == edge_id)
    }

    pub fn find_port(&self, port_ref: PortRef) -> Option<&GraphPort> {
        let node = self.node(port_ref.node_id)?;
        match port_ref.direction {
            PortDirection::Input => node.inputs.iter().find(|port| port.id == port_ref.port_id),
            PortDirection::Output => node.outputs.iter().find(|port| port.id == port_ref.port_id),
        }
    }

    pub fn input_port_ref_at(&self, node_id: NodeId, index: usize) -> Option<PortRef> {
        let node = self.node(node_id)?;
        let port = node.inputs.get(index)?;
        Some(PortRef {
            node_id,
            port_id: port.id,
            direction: PortDirection::Input,
        })
    }

    pub fn output_port_ref_at(&self, node_id: NodeId, index: usize) -> Option<PortRef> {
        let node = self.node(node_id)?;
        let port = node.outputs.get(index)?;
        Some(PortRef {
            node_id,
            port_id: port.id,
            direction: PortDirection::Output,
        })
    }

    pub fn sample() -> Self {
        let mut doc = Self::new();
        let input = doc.add_node("Input", Point::new(4, 2), ["File"], ["Raw", "Meta"]);
        let parse = doc.add_node(
            "Parse Records",
            Point::new(30, 2),
            ["Raw"],
            ["Rows", "Rejects"],
        );
        let enrich = doc.add_node("Enrich", Point::new(30, 13), ["Rows", "Meta"], ["Ready"]);
        let review = doc.add_node(
            "Review Rejects",
            Point::new(58, 1),
            ["Rejects"],
            ["Approved"],
        );
        let output = doc.add_node(
            "Export",
            Point::new(60, 12),
            ["Ready", "Approved"],
            ["Done"],
        );

        let _ = doc.add_edge(
            doc.output_port_ref_at(input, 0).unwrap(),
            doc.input_port_ref_at(parse, 0).unwrap(),
        );
        let _ = doc.add_edge(
            doc.output_port_ref_at(input, 1).unwrap(),
            doc.input_port_ref_at(enrich, 1).unwrap(),
        );
        let _ = doc.add_edge(
            doc.output_port_ref_at(parse, 0).unwrap(),
            doc.input_port_ref_at(enrich, 0).unwrap(),
        );
        let _ = doc.add_edge(
            doc.output_port_ref_at(parse, 1).unwrap(),
            doc.input_port_ref_at(review, 0).unwrap(),
        );
        let _ = doc.add_edge(
            doc.output_port_ref_at(review, 0).unwrap(),
            doc.input_port_ref_at(output, 1).unwrap(),
        );
        let _ = doc.add_edge(
            doc.output_port_ref_at(enrich, 0).unwrap(),
            doc.input_port_ref_at(output, 0).unwrap(),
        );

        doc
    }

    fn alloc_node_id(&mut self) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        id
    }

    fn alloc_port_id(&mut self) -> PortId {
        let id = PortId(self.next_port_id);
        self.next_port_id += 1;
        id
    }

    fn alloc_edge_id(&mut self) -> EdgeId {
        let id = EdgeId(self.next_edge_id);
        self.next_edge_id += 1;
        id
    }

    fn make_port(&mut self, label: impl Into<String>) -> GraphPort {
        GraphPort {
            id: self.alloc_port_id(),
            label: label.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_document_round_trips_with_serde() {
        let doc = GraphDocument::sample();
        let json = serde_json::to_string_pretty(&doc).unwrap();
        let round_trip: GraphDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(round_trip, doc);
    }
}
