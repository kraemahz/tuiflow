use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct NodeId(pub u32);

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct PortId(pub u32);

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct EdgeId(pub u32);

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Size {
    pub width: u16,
    pub height: u16,
}

impl Size {
    pub const fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
pub struct GraphNode<N> {
    pub id: NodeId,
    pub title: String,
    pub position: Point,
    pub inputs: Vec<GraphPort>,
    pub outputs: Vec<GraphPort>,
    pub data: N,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdge<E> {
    pub id: EdgeId,
    pub from: PortRef,
    pub to: PortRef,
    pub data: E,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphDocument<N, E> {
    next_node_id: u32,
    next_port_id: u32,
    next_edge_id: u32,
    pub nodes: Vec<GraphNode<N>>,
    pub edges: Vec<GraphEdge<E>>,
}

impl<N, E> Default for GraphDocument<N, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N, E> GraphDocument<N, E> {
    pub fn new() -> Self {
        Self {
            next_node_id: 1,
            next_port_id: 1,
            next_edge_id: 1,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node<S, I, O>(&mut self, title: S, position: Point, inputs: I, outputs: O) -> NodeId
    where
        N: Default,
        S: Into<String>,
        I: IntoIterator,
        I::Item: AsRef<str>,
        O: IntoIterator,
        O::Item: AsRef<str>,
    {
        self.add_node_with_data(title, position, inputs, outputs, N::default())
    }

    pub fn add_node_with_data<S, I, O>(
        &mut self,
        title: S,
        position: Point,
        inputs: I,
        outputs: O,
        data: N,
    ) -> NodeId
    where
        S: Into<String>,
        I: IntoIterator,
        I::Item: AsRef<str>,
        O: IntoIterator,
        O::Item: AsRef<str>,
    {
        let node_id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        let inputs = self.make_ports(inputs);
        let outputs = self.make_ports(outputs);
        self.nodes.push(GraphNode {
            id: node_id,
            title: title.into(),
            position,
            inputs,
            outputs,
            data,
        });
        node_id
    }

    pub fn add_edge(&mut self, from: PortRef, to: PortRef) -> Option<EdgeId>
    where
        E: Default,
    {
        self.add_edge_with_data(from, to, E::default())
    }

    pub fn add_edge_with_data(&mut self, from: PortRef, to: PortRef, data: E) -> Option<EdgeId> {
        if from.node_id == to.node_id
            || from.direction != PortDirection::Output
            || to.direction != PortDirection::Input
            || !self.has_port(from)
            || !self.has_port(to)
            || self
                .edges
                .iter()
                .any(|edge| edge.from == from && edge.to == to)
        {
            return None;
        }

        let edge_id = EdgeId(self.next_edge_id);
        self.next_edge_id += 1;
        self.edges.push(GraphEdge {
            id: edge_id,
            from,
            to,
            data,
        });
        Some(edge_id)
    }

    pub fn remove_node(&mut self, node_id: NodeId) -> bool {
        let len_before = self.nodes.len();
        self.nodes.retain(|node| node.id != node_id);
        if self.nodes.len() == len_before {
            return false;
        }
        self.edges
            .retain(|edge| edge.from.node_id != node_id && edge.to.node_id != node_id);
        true
    }

    pub fn remove_edge(&mut self, edge_id: EdgeId) -> bool {
        let len_before = self.edges.len();
        self.edges.retain(|edge| edge.id != edge_id);
        self.edges.len() != len_before
    }

    pub fn rename_node(&mut self, node_id: NodeId, title: impl Into<String>) -> bool {
        let Some(node) = self.node_mut(node_id) else {
            return false;
        };
        node.title = title.into();
        true
    }

    pub fn set_node_position(&mut self, node_id: NodeId, position: Point) -> bool {
        let Some(node) = self.node_mut(node_id) else {
            return false;
        };
        node.position = position;
        true
    }

    pub fn set_node_data(&mut self, node_id: NodeId, data: N) -> bool {
        let Some(node) = self.node_mut(node_id) else {
            return false;
        };
        node.data = data;
        true
    }

    pub fn set_edge_data(&mut self, edge_id: EdgeId, data: E) -> bool {
        let Some(edge) = self.edge_mut(edge_id) else {
            return false;
        };
        edge.data = data;
        true
    }

    pub fn node(&self, node_id: NodeId) -> Option<&GraphNode<N>> {
        self.nodes.iter().find(|node| node.id == node_id)
    }

    pub fn node_mut(&mut self, node_id: NodeId) -> Option<&mut GraphNode<N>> {
        self.nodes.iter_mut().find(|node| node.id == node_id)
    }

    pub fn edge(&self, edge_id: EdgeId) -> Option<&GraphEdge<E>> {
        self.edges.iter().find(|edge| edge.id == edge_id)
    }

    pub fn edge_mut(&mut self, edge_id: EdgeId) -> Option<&mut GraphEdge<E>> {
        self.edges.iter_mut().find(|edge| edge.id == edge_id)
    }

    pub fn node_data_mut(&mut self, node_id: NodeId) -> Option<&mut N> {
        Some(&mut self.node_mut(node_id)?.data)
    }

    pub fn edge_data_mut(&mut self, edge_id: EdgeId) -> Option<&mut E> {
        Some(&mut self.edge_mut(edge_id)?.data)
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
        Some(PortRef {
            node_id,
            port_id: node.inputs.get(index)?.id,
            direction: PortDirection::Input,
        })
    }

    pub fn output_port_ref_at(&self, node_id: NodeId, index: usize) -> Option<PortRef> {
        let node = self.node(node_id)?;
        Some(PortRef {
            node_id,
            port_id: node.outputs.get(index)?.id,
            direction: PortDirection::Output,
        })
    }

    fn make_ports<I>(&mut self, labels: I) -> Vec<GraphPort>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        labels
            .into_iter()
            .map(|label| {
                let port = GraphPort {
                    id: PortId(self.next_port_id),
                    label: label.as_ref().to_owned(),
                };
                self.next_port_id += 1;
                port
            })
            .collect()
    }

    fn has_port(&self, port_ref: PortRef) -> bool {
        self.find_port(port_ref).is_some()
    }
}

impl<N, E> GraphDocument<N, E>
where
    N: Default,
    E: Default,
{
    pub fn sample() -> Self {
        let mut document = Self::new();
        let input = document.add_node("Input", Point::new(4, 2), ["File"], ["Raw", "Meta"]);
        let parse = document.add_node(
            "Parse Records",
            Point::new(24, 2),
            ["Raw"],
            ["Rows", "Rejects"],
        );
        let enrich = document.add_node("Enrich", Point::new(30, 15), ["Rows", "Meta"], ["Ready"]);
        let review = document.add_node(
            "Review Rejects",
            Point::new(58, 1),
            ["Rejects"],
            ["Approved"],
        );
        let export = document.add_node(
            "Export",
            Point::new(59, 15),
            ["Ready", "Approved"],
            ["Done"],
        );

        let _ = document.add_edge(
            document.output_port_ref_at(input, 0).unwrap(),
            document.input_port_ref_at(parse, 0).unwrap(),
        );
        let _ = document.add_edge(
            document.output_port_ref_at(input, 1).unwrap(),
            document.input_port_ref_at(enrich, 1).unwrap(),
        );
        let _ = document.add_edge(
            document.output_port_ref_at(parse, 0).unwrap(),
            document.input_port_ref_at(enrich, 0).unwrap(),
        );
        let _ = document.add_edge(
            document.output_port_ref_at(parse, 1).unwrap(),
            document.input_port_ref_at(review, 0).unwrap(),
        );
        let _ = document.add_edge(
            document.output_port_ref_at(review, 0).unwrap(),
            document.input_port_ref_at(export, 1).unwrap(),
        );
        let _ = document.add_edge(
            document.output_port_ref_at(enrich, 0).unwrap(),
            document.input_port_ref_at(export, 0).unwrap(),
        );

        document
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payloads_round_trip_through_serde() {
        let mut document = GraphDocument::<String, u32>::new();
        let node_id = document.add_node_with_data(
            "Node",
            Point::new(1, 2),
            ["In"],
            ["Out"],
            "payload".to_owned(),
        );
        let other = document.add_node("Other", Point::new(10, 2), ["In"], ["Out"]);
        let edge_id = document
            .add_edge_with_data(
                document.output_port_ref_at(node_id, 0).unwrap(),
                document.input_port_ref_at(other, 0).unwrap(),
                7,
            )
            .unwrap();

        let json = serde_json::to_string(&document).unwrap();
        let decoded: GraphDocument<String, u32> = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.node(node_id).unwrap().data, "payload");
        assert_eq!(decoded.edge(edge_id).unwrap().data, 7);
    }

    #[test]
    fn setters_update_payloads() {
        let mut document = GraphDocument::<String, String>::sample();
        let node_id = document.nodes[0].id;
        let edge_id = document.edges[0].id;

        assert!(document.set_node_data(node_id, "node".to_owned()));
        assert!(document.set_edge_data(edge_id, "edge".to_owned()));

        assert_eq!(document.node(node_id).unwrap().data, "node");
        assert_eq!(document.edge(edge_id).unwrap().data, "edge");
    }
}
