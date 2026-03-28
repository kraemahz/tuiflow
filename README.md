# tuiflow

`tuiflow` is a `ratatui`-based library for rendering and editing directed flow diagrams in a terminal UI.

It gives you three layers to build with:

- `GraphDocument<N, E>` for the graph model and typed node/edge payloads
- `GraphEditorState`, `EditorAction`, `apply_action`, and configurable `ActionMapper` bindings for custom host integrations
- `GraphCanvas` for rendering a graph/editor state into any `ratatui` layout
- `EditorShell` for a ready-to-embed keyboard-first editing shell

The best current reference is [`examples/showcase.rs`](./examples/showcase.rs).

![Showcase Gif](https://raw.githubusercontent.com/kraemahz/tuiflow/main/assets/tuiflow.gif "Showcase Gif")

## Features

- Typed graph document with explicit `NodeId`, `PortId`, `EdgeId`, and `PortRef`
- Generic node and edge payloads, so host applications can attach their own metadata
- `serde` support on document types for persistence and round-tripping
- Configurable semantic input bindings through `InputMap`, `ActionMapper::with_bindings`, and `MapResult`
- Keyboard-first editor flow for:
  - directional node selection
  - connection browsing
  - node creation and rename prompts
  - move preview + confirm/cancel
  - edge creation from output ports to valid input ports
  - deletion
  - undo
  - viewport panning and recentering
- Host callbacks via `EditorEffect` so your app can decide how to edit node and edge payloads
- Canvas layout that sizes nodes from titles/ports and routes orthogonal edges around node boxes
- Drop-in sample graph via `GraphDocument::sample()`
- Snapshot-tested rendering using `ratatui`'s test backend

## Quick Start

Run the included showcase:

```bash
cargo run --example showcase
```

The showcase demonstrates:

- embedding the editor inside a larger `ratatui` layout
- rendering your own sidebar and status line around the graph canvas
- handling `EditorEffect::OpenNodeEditor` and `EditorEffect::OpenEdgeEditor`
- mutating typed payload data in response to editor actions

If you need the shell in a larger application, use `EditorShell::with_mapper(...)` or `handle_event_result(...)` so outer modes can see whether the editor consumed the event.

## Example

This is the core integration pattern used by the showcase:

```rust
use crossterm::event::Event;
use ratatui::{Frame, layout::Rect};
use tuiflow::{EditorEffect, EditorShell, GraphDocument};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct NodeData {
    edits: u32,
    note: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct EdgeData {
    edits: u32,
    label: String,
}

struct App {
    editor: EditorShell<NodeData, EdgeData>,
}

impl App {
    fn new() -> Self {
        Self {
            editor: EditorShell::new(GraphDocument::sample()),
        }
    }

    fn handle_event(&mut self, event: &Event) {
        let effects = self.editor.handle_event(event);
        for effect in effects {
            match effect {
                EditorEffect::OpenNodeEditor {
                    node_id,
                    title,
                    mut data,
                } => {
                    data.edits += 1;
                    if data.note.is_empty() {
                        data.note = format!("{title} opened");
                    }
                    let _ = self.editor.document_mut().set_node_data(node_id, data);
                }
                EditorEffect::OpenEdgeEditor { edge_id, mut data } => {
                    data.edits += 1;
                    if data.label.is_empty() {
                        data.label = format!("Edge {}", edge_id.0);
                    }
                    let _ = self.editor.document_mut().set_edge_data(edge_id, data);
                }
                EditorEffect::RequestPrompt(_) | EditorEffect::Status(_) => {}
            }
        }
    }

    fn render(&self, frame: &mut Frame, area: Rect) {
        self.editor.render(frame, area);
    }
}
```

If you want more control, build directly from the lower-level pieces:

```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tuiflow::{
    ActionMapper, GraphCanvas, GraphDocument, GraphEditorState, GraphTheme, InputMap, KeyBinding,
    NavigateBindings,
};

let document = GraphDocument::<(), ()>::sample();
let state = GraphEditorState::new();
let theme = GraphTheme::default();

let canvas = GraphCanvas::new(&document, &state, &theme);
// frame.render_widget(canvas, area);

let mapper = ActionMapper::with_bindings(InputMap {
    navigate: NavigateBindings {
        create_node: vec![KeyBinding::plain(KeyCode::Char('a'))],
        ..NavigateBindings::default()
    },
    ..InputMap::default()
});

let result = mapper.map_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE), &state);
assert!(result.consumed);
```

## Keyboard Controls

The default input mapper supports the following controls:

- Arrow keys or `h/j/k/l`: move selection
- `Tab` / `Shift+Tab`: switch between a node and its connected edges
- `Enter`: activate the selected node or edge
- `n`: create a node
- `r`: rename the selected node
- `m`: move the selected node
- `c`: begin connecting from the selected node's best output port
- `d`, `Delete`, or `Backspace`: delete the selected node or edge
- `u`: undo the last committed graph change
- `g`: center the viewport around the current focus
- `Shift` + arrow keys: pan the viewport
- In move/connect modes, `Enter` confirms and `Esc` cancels

These defaults live in `InputMap::default()`. Host apps can replace only the bindings they care about and leave the editor semantics unchanged.

## Data Model

`GraphDocument<N, E>` stores:

- nodes with a title, position, input ports, output ports, and typed payload `N`
- edges between an output `PortRef` and an input `PortRef`, with typed payload `E`

The document API includes helpers for:

- adding nodes and edges
- updating node positions
- renaming nodes
- reading and mutating node/edge payloads
- removing nodes or edges
- retrieving port references by index

## Current Scope

What is implemented:

- keyboard-first graph editing
- terminal rendering for bounded viewports
- typed host-managed payload editing hooks
- document serialization

What is not a high-level promise yet:

- full mouse editing workflows
- a persistence format beyond the serializable Rust document model
- higher-level application chrome outside the editor shell

## Development

Run tests:

```bash
cargo test
```

The repository includes snapshot tests for both the canvas renderer and the showcase layout.
