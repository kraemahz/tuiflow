# Roadmap

## Phase 0: Design Baseline

- capture reference notes from `tuigram`
- define graph editor architecture
- agree on keyboard-first interaction model

## Phase 1: Drawable Canvas

- render fixed-position nodes
- render explicit ports
- render straight or orthogonal directed edges
- add selection styling

Exit criteria:

- a hardcoded graph can be rendered legibly in `ratatui`

## Phase 2: Editor Skeleton

- add focus and selection state
- move between nodes and ports
- create nodes
- delete nodes and edges

Exit criteria:

- graph mutations are possible without editing source code

## Phase 3: Node Movement

- move selected node with keyboard
- update edge routing after movement
- add viewport panning when the selection leaves the visible area

Exit criteria:

- graph structure remains readable after repeated moves

## Phase 4: Edge Creation

- start connection from an output port
- choose valid destination input port
- render a live preview route
- commit edge on confirm

Exit criteria:

- a user can build a simple directed graph interactively

## Phase 5: Persistence

- define a stable on-disk document format
- save and load graph documents
- preserve stable ids and explicit coordinates

Exit criteria:

- documents round-trip without losing graph structure
