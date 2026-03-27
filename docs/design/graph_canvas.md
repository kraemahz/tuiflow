# Graph Canvas Design

## Purpose

Design a `ratatui` graph editor for directed graphs with:

- explicit node placement
- explicit edge creation
- movable nodes
- keyboard-first editing
- later optional mouse support

## Core Design Decision

Build the editor core from scratch and borrow only the visual language from `tuigram`.

Reason:

- available crates are either renderers or highly specialized editors
- general graph editing needs a stronger internal model than the current ecosystem provides
- the visual side is easier to borrow than the editing semantics

## Editor Layers

## 1. Graph Model

Owns persistent diagram state.

Suggested types:

- `GraphDocument`
- `NodeId`
- `PortId`
- `EdgeId`
- `GraphNode`
- `GraphPort`
- `GraphEdge`

Responsibilities:

- stable identities
- node titles and metadata
- node coordinates
- port definitions and directions
- edge endpoints

## 2. View State

Owns ephemeral UI state.

Suggested contents:

- current tool or mode
- selected node, port, or edge
- focused node
- viewport origin
- pending connection source
- drag or move preview

This state must stay separate from the saved graph document.

## 3. Layout and Routing

Consumes graph data and produces drawable geometry.

Suggested outputs:

- node rectangles
- port anchor points
- routed edge segments
- label anchor positions

Initial rule:

- node positions are explicit
- edges are orthogonally routed around node bounds
- no automatic node layout in v0

## 4. Renderer

Consumes resolved geometry and writes to a `ratatui` frame.

Responsibilities:

- draw node boxes
- draw titles
- draw ports
- draw routed edges
- draw arrowheads
- draw selection and preview overlays

## 5. Interaction Controller

Maps input events into state transitions and graph mutations.

Responsibilities:

- move selection
- enter connect mode
- confirm edge creation
- create node
- move node
- delete selection
- pan viewport

## Data Model

Suggested baseline:

```rust
pub struct GraphDocument {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

pub struct GraphNode {
    pub id: NodeId,
    pub title: String,
    pub position: Point,
    pub size: Size,
    pub inputs: Vec<GraphPort>,
    pub outputs: Vec<GraphPort>,
}

pub struct GraphPort {
    pub id: PortId,
    pub label: String,
}

pub struct GraphEdge {
    pub id: EdgeId,
    pub from: PortRef,
    pub to: PortRef,
}
```

Important constraint:

- store graph semantics in typed structures
- do not treat rendered glyphs as state

## Coordinate Model

Use a world-space canvas model.

- node positions are world coordinates
- viewport origin maps world coordinates into terminal coordinates
- renderer clips to visible terminal bounds

This avoids repainting the model around terminal-relative positions.

## Interaction Model

Default to keyboard-first operation.

## Modes

- `Navigate`
- `MoveNode`
- `ConnectEdge`
- `CreateNode`
- `RenameNode`

## Proposed v0 bindings

- arrows or `h/j/k/l`: move focus
- `n`: create node
- `m`: move selected node
- `c`: start edge connection from selected output
- `Tab`: cycle ports or nearby targets
- `Enter`: confirm action
- `Esc`: cancel mode
- `d`: delete selected element
- `g`: center viewport on selection

Mouse support can be added later without changing the core architecture if the controller already talks in terms of semantic actions.

## Rendering Rules

## Nodes

- use bordered boxes with a centered title
- reserve left side for inputs and right side for outputs
- make port rows visually consistent

## Edges

- orthogonal routing first
- use Unicode line characters by default
- terminate with a clear arrowhead
- keep labels optional in v0

## Selection

- selected node gets highlighted border and title
- selected port gets a stronger marker
- selected edge is redrawn in highlight style

## Theme

Start with the same semantic style buckets that worked well in `tuigram`:

- text
- muted
- accent
- border
- selected
- error

## Non-Goals For v0

- automatic graph layout
- zoom
- rich inline text editing
- subgraphs or grouping
- edge labels with collision avoidance
- mouse-first drag handling

## Risks

## 1. Routing complexity grows fast

If we mix routing and rendering too early, debugging edge behavior will become expensive.

Mitigation:

- keep routed geometry as a separate intermediate representation

## 2. Selection can become ambiguous

Nodes, ports, and edges all compete for focus.

Mitigation:

- use explicit selection enums instead of ad hoc booleans

## 3. Terminal constraints will pressure layout

Small windows can make dense graphs unreadable.

Mitigation:

- design around clipping and panning first
- defer zoom and auto-layout

## Open Questions

- Should v0 permit edges only from output ports to input ports, or allow generic directed endpoints?
- Should node size be fixed in v0, or derived from title and port labels?
- Should routing avoid only node bounds, or also preserve existing edge lanes where possible?
