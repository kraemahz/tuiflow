use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Widget},
};

use crate::{
    GraphDocument,
    document::{Point, PortDirection, PortRef},
    editor::{GraphEditorMode, GraphEditorState, Selection},
    layout::{CanvasLayout, WorldRect, width_of},
    theme::GraphTheme,
};

pub struct GraphCanvas<'a> {
    document: &'a GraphDocument,
    state: &'a GraphEditorState,
    theme: &'a GraphTheme,
}

impl<'a> GraphCanvas<'a> {
    pub fn new(
        document: &'a GraphDocument,
        state: &'a GraphEditorState,
        theme: &'a GraphTheme,
    ) -> Self {
        Self {
            document,
            state,
            theme,
        }
    }
}

impl Widget for GraphCanvas<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 3 || area.height < 3 {
            return;
        }

        let mut preview_document = self.document.clone();
        if let GraphEditorMode::MoveNode {
            node_id,
            current_position,
            ..
        } = self.state.mode
        {
            let _ = preview_document.set_node_position(node_id, current_position);
        }
        let layout = CanvasLayout::for_document(&preview_document);
        Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.border)
            .render(area, buf);
        let canvas = area.inner(ratatui::layout::Margin::new(1, 1));

        for edge in &layout.edges {
            let selected = self.state.selection == Selection::Edge(edge.edge_id);
            draw_edge(
                edge.points.as_slice(),
                canvas,
                self.state.viewport,
                buf,
                if selected {
                    self.theme.edge_selected
                } else {
                    self.theme.edge
                },
            );
        }

        if let GraphEditorMode::ConnectEdge {
            source,
            candidate_index,
        } = self.state.mode
        {
            let targets = preview_targets(&preview_document, source);
            if let Some(target) = targets.get(candidate_index).copied() {
                let preview = layout::route_edge(
                    layout.port_anchor(source).unwrap_or(Point::new(0, 0)),
                    layout.port_anchor(target).unwrap_or(Point::new(0, 0)),
                    &layout.nodes,
                );
                draw_edge(
                    preview.as_slice(),
                    canvas,
                    self.state.viewport,
                    buf,
                    self.theme.selected,
                );
            }
        }

        for node in &layout.nodes {
            if !rect_intersects_canvas(node.rect, canvas, self.state.viewport) {
                continue;
            }

            let node_selected = self.state.selection == Selection::Node(node.node_id);
            let node_style = if node_selected {
                self.theme.selected
            } else {
                self.theme.border
            };

            if let Some(rect) = world_rect_to_screen(node.rect, canvas, self.state.viewport) {
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(node_style)
                    .render(rect, buf);
                draw_centered_text(
                    &node.title,
                    rect,
                    buf,
                    if node_selected {
                        self.theme.selected
                    } else {
                        self.theme.accent
                    },
                );
                draw_ports(node, rect, buf, self.state.selection, self.theme);
            }
        }
    }
}

fn preview_targets(document: &GraphDocument, source: PortRef) -> Vec<PortRef> {
    let mut targets = Vec::new();
    for node in &document.nodes {
        if node.id == source.node_id {
            continue;
        }
        for port in &node.inputs {
            let port_ref = PortRef {
                node_id: node.id,
                port_id: port.id,
                direction: PortDirection::Input,
            };
            let exists = document
                .edges
                .iter()
                .any(|edge| edge.from == source && edge.to == port_ref);
            if !exists {
                targets.push(port_ref);
            }
        }
    }
    targets
}

fn draw_ports(
    node: &crate::layout::NodeLayout,
    rect: Rect,
    buf: &mut Buffer,
    selection: Selection,
    theme: &GraphTheme,
) {
    for port in &node.inputs {
        let y = rect.y + 1 + port.row as u16;
        if y >= rect.bottom() {
            continue;
        }
        let selected = selection
            == Selection::Port(PortRef {
                node_id: node.node_id,
                port_id: port.port_id,
                direction: PortDirection::Input,
            });
        set_cell(
            buf,
            rect.x,
            y,
            "◀",
            if selected { theme.selected } else { theme.text },
        );
        draw_string(
            buf,
            rect.x + 1,
            y,
            rect.width.saturating_sub(3),
            &port.label,
            if selected { theme.selected } else { theme.text },
        );
    }

    for port in &node.outputs {
        let y = rect.y + 1 + port.row as u16;
        if y >= rect.bottom() {
            continue;
        }
        let selected = selection
            == Selection::Port(PortRef {
                node_id: node.node_id,
                port_id: port.port_id,
                direction: PortDirection::Output,
            });
        let label_width = width_of(&port.label);
        let start_x = rect
            .right()
            .saturating_sub(label_width.saturating_add(1))
            .max(rect.x + 1);
        draw_string(
            buf,
            start_x,
            y,
            label_width,
            &port.label,
            if selected { theme.selected } else { theme.text },
        );
        set_cell(
            buf,
            rect.right(),
            y,
            "▶",
            if selected { theme.selected } else { theme.text },
        );
    }
}

fn draw_centered_text(text: &str, rect: Rect, buf: &mut Buffer, style: Style) {
    let width = width_of(text);
    let x = rect
        .x
        .saturating_add(rect.width.saturating_sub(width) / 2)
        .max(rect.x + 1);
    draw_string(buf, x, rect.y, rect.width.saturating_sub(2), text, style);
}

fn rect_intersects_canvas(rect: WorldRect, canvas: Rect, viewport: Point) -> bool {
    let left = rect.x - viewport.x + i32::from(canvas.x);
    let right = rect.right() - viewport.x + i32::from(canvas.x);
    let top = rect.y - viewport.y + i32::from(canvas.y);
    let bottom = rect.bottom() - viewport.y + i32::from(canvas.y);
    right >= i32::from(canvas.x)
        && left <= i32::from(canvas.right())
        && bottom >= i32::from(canvas.y)
        && top <= i32::from(canvas.bottom())
}

fn world_rect_to_screen(rect: WorldRect, canvas: Rect, viewport: Point) -> Option<Rect> {
    let x = rect.x - viewport.x + i32::from(canvas.x);
    let y = rect.y - viewport.y + i32::from(canvas.y);
    if x + i32::from(rect.width) <= i32::from(canvas.x)
        || y + i32::from(rect.height) <= i32::from(canvas.y)
        || x >= i32::from(canvas.right())
        || y >= i32::from(canvas.bottom())
    {
        return None;
    }
    Some(Rect::new(
        x.max(i32::from(canvas.x)) as u16,
        y.max(i32::from(canvas.y)) as u16,
        rect.width,
        rect.height,
    ))
}

fn world_to_screen(point: Point, canvas: Rect, viewport: Point) -> Option<(u16, u16)> {
    let x = point.x - viewport.x + i32::from(canvas.x);
    let y = point.y - viewport.y + i32::from(canvas.y);
    if x < i32::from(canvas.x)
        || y < i32::from(canvas.y)
        || x > i32::from(canvas.right())
        || y > i32::from(canvas.bottom())
    {
        return None;
    }
    Some((x as u16, y as u16))
}

fn draw_edge(points: &[Point], canvas: Rect, viewport: Point, buf: &mut Buffer, style: Style) {
    let expanded = expand_polyline(points);
    if expanded.is_empty() {
        return;
    }

    for (idx, point) in expanded.iter().enumerate() {
        let Some((x, y)) = world_to_screen(*point, canvas, viewport) else {
            continue;
        };
        let symbol = if idx == expanded.len() - 1 {
            arrow_symbol(expanded.get(idx.wrapping_sub(1)).copied(), *point)
        } else {
            line_symbol(
                expanded.get(idx.wrapping_sub(1)).copied(),
                *point,
                expanded.get(idx + 1).copied(),
            )
        };
        set_cell(buf, x, y, symbol, style);
    }
}

fn expand_polyline(points: &[Point]) -> Vec<Point> {
    if points.is_empty() {
        return Vec::new();
    }
    let mut out = vec![points[0]];
    for segment in points.windows(2) {
        let start = segment[0];
        let end = segment[1];
        let step_x = (end.x - start.x).signum();
        let step_y = (end.y - start.y).signum();
        let mut current = start;
        while current != end {
            current = Point::new(current.x + step_x, current.y + step_y);
            out.push(current);
        }
    }
    out
}

fn arrow_symbol(previous: Option<Point>, current: Point) -> &'static str {
    let Some(previous) = previous else {
        return "•";
    };
    match (current.x - previous.x, current.y - previous.y) {
        (1, 0) => "▶",
        (-1, 0) => "◀",
        (0, 1) => "▼",
        (0, -1) => "▲",
        _ => "•",
    }
}

fn line_symbol(previous: Option<Point>, current: Point, next: Option<Point>) -> &'static str {
    let prev = previous.unwrap_or(current);
    let next = next.unwrap_or(current);
    let a = (current.x - prev.x, current.y - prev.y);
    let b = (next.x - current.x, next.y - current.y);
    match (a, b) {
        ((-1, 0), (1, 0)) | ((1, 0), (-1, 0)) | ((1, 0), (1, 0)) | ((-1, 0), (-1, 0)) => "─",
        ((0, -1), (0, 1)) | ((0, 1), (0, -1)) | ((0, 1), (0, 1)) | ((0, -1), (0, -1)) => "│",
        ((1, 0), (0, 1)) | ((0, -1), (-1, 0)) => "┐",
        ((-1, 0), (0, 1)) | ((0, -1), (1, 0)) => "┌",
        ((1, 0), (0, -1)) | ((0, 1), (-1, 0)) => "┘",
        ((-1, 0), (0, -1)) | ((0, 1), (1, 0)) => "└",
        _ => "•",
    }
}

fn draw_string(buf: &mut Buffer, x: u16, y: u16, max_width: u16, text: &str, style: Style) {
    let mut cursor_x = x;
    for ch in text.chars() {
        if cursor_x >= x + max_width {
            break;
        }
        set_cell(buf, cursor_x, y, &ch.to_string(), style);
        cursor_x += 1;
    }
}

fn set_cell(buf: &mut Buffer, x: u16, y: u16, symbol: &str, style: Style) {
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_symbol(symbol);
        cell.set_style(style);
    }
}

mod layout {
    pub use crate::layout::route_edge;
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use ratatui::{Terminal, backend::TestBackend};

    use crate::{document::GraphDocument, editor::GraphEditorState, theme::GraphTheme};

    use super::*;

    fn render_string(document: &GraphDocument, state: &GraphEditorState, area: Rect) -> String {
        let backend = TestBackend::new(area.width, area.height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                frame.render_widget(
                    GraphCanvas::new(document, state, &GraphTheme::default()),
                    area,
                );
            })
            .unwrap();
        buffer_to_string(terminal.backend().buffer())
    }

    fn buffer_to_string(buf: &Buffer) -> String {
        let mut lines = Vec::new();
        for y in 0..buf.area.height {
            let mut line = String::new();
            for x in 0..buf.area.width {
                line.push_str(buf[(x, y)].symbol());
            }
            lines.push(line.trim_end().to_owned());
        }
        lines.join("\n")
    }

    #[test]
    fn renders_selected_node_snapshot() {
        let document = GraphDocument::sample();
        let mut state = GraphEditorState::new();
        state.selection = Selection::Node(document.nodes[0].id);
        assert_snapshot!(render_string(&document, &state, Rect::new(0, 0, 80, 24)));
    }

    #[test]
    fn renders_clipped_subrect_snapshot() {
        let document = GraphDocument::sample();
        let state = GraphEditorState::new();
        assert_snapshot!(render_string(&document, &state, Rect::new(0, 0, 36, 14)));
    }
}
