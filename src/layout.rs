use std::cmp::{max, min};
use std::collections::{HashMap, HashSet, VecDeque};

use unicode_width::UnicodeWidthStr;

use crate::document::{EdgeId, GraphDocument, NodeId, Point, PortDirection, PortId, PortRef, Size};

const NODE_MIN_WIDTH: u16 = 16;
const NODE_MAX_WIDTH: u16 = 28;
const PORT_LABEL_MAX_WIDTH: usize = 12;
const TITLE_MAX_WIDTH: usize = 20;
const H_PADDING: u16 = 2;
const V_PADDING: u16 = 1;
const SAFE_MARGIN: i32 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldRect {
    pub x: i32,
    pub y: i32,
    pub width: u16,
    pub height: u16,
}

impl WorldRect {
    pub fn right(self) -> i32 {
        self.x + i32::from(self.width) - 1
    }

    pub fn bottom(self) -> i32 {
        self.y + i32::from(self.height) - 1
    }

    pub fn center(self) -> Point {
        Point::new(
            self.x + i32::from(self.width / 2),
            self.y + i32::from(self.height / 2),
        )
    }

    pub fn contains(self, point: Point) -> bool {
        point.x >= self.x
            && point.x <= self.right()
            && point.y >= self.y
            && point.y <= self.bottom()
    }
}

#[derive(Clone, Debug)]
pub struct DisplayPort {
    pub port_id: PortId,
    pub label: String,
    pub row: usize,
    pub anchor: Point,
}

#[derive(Clone, Debug)]
pub struct NodeLayout {
    pub node_id: NodeId,
    pub rect: WorldRect,
    pub title: String,
    pub inputs: Vec<DisplayPort>,
    pub outputs: Vec<DisplayPort>,
}

#[derive(Clone, Debug)]
pub struct EdgeLayout {
    pub edge_id: EdgeId,
    pub points: Vec<Point>,
}

#[derive(Clone, Debug, Default)]
pub struct CanvasLayout {
    pub nodes: Vec<NodeLayout>,
    pub edges: Vec<EdgeLayout>,
}

impl CanvasLayout {
    pub fn for_document(document: &GraphDocument) -> Self {
        let nodes: Vec<_> = document
            .nodes
            .iter()
            .map(|node| {
                let title = truncate(&node.title, TITLE_MAX_WIDTH);
                let widest_port = node
                    .inputs
                    .iter()
                    .chain(node.outputs.iter())
                    .map(|port| width_of(&truncate(&port.label, PORT_LABEL_MAX_WIDTH)))
                    .max()
                    .unwrap_or(0);
                let desired_width = max(
                    width_of(&title),
                    widest_port.saturating_mul(2).saturating_add(3),
                );
                let width = desired_width
                    .saturating_add(H_PADDING)
                    .clamp(NODE_MIN_WIDTH, NODE_MAX_WIDTH);
                let rows = max(node.inputs.len(), node.outputs.len()) as u16;
                let height = rows + 2 + V_PADDING;
                let rect = WorldRect {
                    x: node.position.x,
                    y: node.position.y,
                    width,
                    height,
                };

                let inputs = node
                    .inputs
                    .iter()
                    .enumerate()
                    .map(|(row, port)| DisplayPort {
                        port_id: port.id,
                        label: truncate(&port.label, PORT_LABEL_MAX_WIDTH),
                        row,
                        anchor: Point::new(rect.x - 1, rect.y + 1 + row as i32),
                    })
                    .collect();

                let outputs = node
                    .outputs
                    .iter()
                    .enumerate()
                    .map(|(row, port)| DisplayPort {
                        port_id: port.id,
                        label: truncate(&port.label, PORT_LABEL_MAX_WIDTH),
                        row,
                        anchor: Point::new(rect.right() + 1, rect.y + 1 + row as i32),
                    })
                    .collect();

                NodeLayout {
                    node_id: node.id,
                    rect,
                    title,
                    inputs,
                    outputs,
                }
            })
            .collect();

        let by_node: HashMap<_, _> = nodes.iter().map(|node| (node.node_id, node)).collect();
        let edges = document
            .edges
            .iter()
            .filter_map(|edge| {
                let from_node = by_node.get(&edge.from.node_id)?;
                let to_node = by_node.get(&edge.to.node_id)?;
                let from = port_anchor(from_node, edge.from)?;
                let to = port_anchor(to_node, edge.to)?;
                let points = route_edge(from, to, &nodes);
                Some(EdgeLayout {
                    edge_id: edge.id,
                    points,
                })
            })
            .collect();

        Self { nodes, edges }
    }

    pub fn node(&self, node_id: NodeId) -> Option<&NodeLayout> {
        self.nodes.iter().find(|node| node.node_id == node_id)
    }

    pub fn edge(&self, edge_id: EdgeId) -> Option<&EdgeLayout> {
        self.edges.iter().find(|edge| edge.edge_id == edge_id)
    }

    pub fn port_anchor(&self, port_ref: PortRef) -> Option<Point> {
        let node = self.node(port_ref.node_id)?;
        port_anchor(node, port_ref)
    }
}

pub fn truncate(input: &str, max_width: usize) -> String {
    if usize::from(width_of(input)) <= max_width {
        return input.to_owned();
    }

    let mut out = String::new();
    let mut width = 0;
    for ch in input.chars() {
        let ch_width = UnicodeWidthStr::width(ch.encode_utf8(&mut [0; 4]));
        if width + ch_width + 1 > max_width {
            break;
        }
        out.push(ch);
        width += ch_width;
    }
    out.push('…');
    out
}

pub fn width_of(input: &str) -> u16 {
    UnicodeWidthStr::width(input) as u16
}

pub fn route_edge(from: Point, to: Point, nodes: &[NodeLayout]) -> Vec<Point> {
    if from == to {
        return vec![from];
    }

    let mut blocked = HashSet::new();
    for node in nodes {
        for y in node.rect.y..=node.rect.bottom() {
            for x in node.rect.x..=node.rect.right() {
                blocked.insert((x, y));
            }
        }
    }
    blocked.remove(&(from.x, from.y));
    blocked.remove(&(to.x, to.y));

    let min_x = min(from.x, to.x).min(nodes.iter().map(|node| node.rect.x).min().unwrap_or(from.x))
        - SAFE_MARGIN;
    let max_x = max(from.x, to.x).max(
        nodes
            .iter()
            .map(|node| node.rect.right())
            .max()
            .unwrap_or(to.x),
    ) + SAFE_MARGIN;
    let min_y = min(from.y, to.y).min(nodes.iter().map(|node| node.rect.y).min().unwrap_or(from.y))
        - SAFE_MARGIN;
    let max_y = max(from.y, to.y).max(
        nodes
            .iter()
            .map(|node| node.rect.bottom())
            .max()
            .unwrap_or(to.y),
    ) + SAFE_MARGIN;

    let neighbor_order = ordered_neighbors(from, to);
    let mut queue = VecDeque::from([from]);
    let mut visited = HashSet::from([(from.x, from.y)]);
    let mut came_from: HashMap<(i32, i32), (i32, i32)> = HashMap::new();

    while let Some(current) = queue.pop_front() {
        if current == to {
            break;
        }

        for (dx, dy) in neighbor_order {
            let next = Point::new(current.x + dx, current.y + dy);
            if next.x < min_x || next.x > max_x || next.y < min_y || next.y > max_y {
                continue;
            }
            if blocked.contains(&(next.x, next.y)) || !visited.insert((next.x, next.y)) {
                continue;
            }
            came_from.insert((next.x, next.y), (current.x, current.y));
            queue.push_back(next);
        }
    }

    if !came_from.contains_key(&(to.x, to.y)) {
        return fallback_dogleg(from, to);
    }

    let mut path = vec![to];
    let mut cursor = (to.x, to.y);
    while cursor != (from.x, from.y) {
        cursor = came_from[&cursor];
        path.push(Point::new(cursor.0, cursor.1));
    }
    path.reverse();
    simplify_path(path)
}

fn ordered_neighbors(from: Point, to: Point) -> [(i32, i32); 4] {
    let horizontal = if to.x >= from.x { (1, 0) } else { (-1, 0) };
    let vertical = if to.y >= from.y { (0, 1) } else { (0, -1) };
    [
        horizontal,
        vertical,
        (-vertical.0, -vertical.1),
        (-horizontal.0, -horizontal.1),
    ]
}

fn fallback_dogleg(from: Point, to: Point) -> Vec<Point> {
    let mid_x = from.x + (to.x - from.x) / 2;
    simplify_path(vec![
        from,
        Point::new(mid_x, from.y),
        Point::new(mid_x, to.y),
        to,
    ])
}

fn simplify_path(points: Vec<Point>) -> Vec<Point> {
    if points.len() <= 2 {
        return points;
    }

    let mut simplified = vec![points[0]];
    for window in points.windows(3) {
        let a = window[0];
        let b = window[1];
        let c = window[2];
        let ab = (b.x - a.x, b.y - a.y);
        let bc = (c.x - b.x, c.y - b.y);
        if ab.0.signum() == bc.0.signum() && ab.1.signum() == bc.1.signum() {
            continue;
        }
        simplified.push(b);
    }
    simplified.push(*points.last().unwrap());
    simplified
}

fn port_anchor(node: &NodeLayout, port_ref: PortRef) -> Option<Point> {
    let ports = match port_ref.direction {
        PortDirection::Input => &node.inputs,
        PortDirection::Output => &node.outputs,
    };
    ports
        .iter()
        .find(|port| port.port_id == port_ref.port_id)
        .map(|port| port.anchor)
}

pub fn port_row_count(size: Size) -> u16 {
    size.height.saturating_sub(2 + V_PADDING)
}

#[cfg(test)]
mod tests {
    use crate::document::GraphDocument;

    use super::*;

    #[test]
    fn title_and_port_labels_are_truncated() {
        assert_eq!(truncate("ExtremelyLongNodeTitle", 10), "Extremely…");
        assert_eq!(truncate("tiny", 10), "tiny");
    }

    #[test]
    fn node_sizing_uses_title_and_port_labels() {
        let doc = GraphDocument::sample();
        let layout = CanvasLayout::for_document(&doc);
        let parse = layout
            .nodes
            .iter()
            .find(|node| node.title.starts_with("Parse"))
            .unwrap();
        assert!(parse.rect.width >= NODE_MIN_WIDTH);
        assert!(parse.rect.height >= 4);
    }

    #[test]
    fn router_avoids_node_bounds() {
        let doc = GraphDocument::sample();
        let layout = CanvasLayout::for_document(&doc);
        let edge = &layout.edges[0];
        for point in &edge.points {
            let touches_node = layout.nodes.iter().any(|node| node.rect.contains(*point));
            assert!(!touches_node, "edge point {point:?} intersected a node");
        }
    }

    #[test]
    fn routing_is_deterministic() {
        let from = Point::new(0, 0);
        let to = Point::new(8, 3);
        let path_a = route_edge(from, to, &[]);
        let path_b = route_edge(from, to, &[]);
        assert_eq!(path_a, path_b);
    }
}
