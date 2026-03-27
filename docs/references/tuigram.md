# `tuigram` Reference Notes

`tuigram` is the strongest visual reference we found for a polished terminal diagram editor.
It is not a fit as a direct foundation because its model is tightly bound to sequence diagrams.

## Why It Is Useful

- The rendered boxes are compact and legible.
- The arrows read clearly at terminal scale.
- The spacing model is simple enough to reason about.
- The theme is restrained instead of over-styled.

## Files Reviewed

- `src/render/sequence.rs`
- `src/theme.rs`
- `src/ui/mod.rs`
- `src/core/models.rs`
- `src/core/sequence.rs`

## Rendering Patterns Worth Borrowing

## 1. Separate structure from drawing

`tuigram` keeps its data model simple and pushes terminal drawing into a dedicated render module.
That is the right pattern for this repository as well.

For us that should become:

- graph model for nodes, ports, edges, and selection
- renderer that only consumes resolved positions and routes
- interaction layer that mutates editor state but does not draw directly

## 2. Stable geometric constants

`tuigram` uses a small set of layout constants for header height and message offset.
That keeps rendering predictable and avoids hidden spacing rules.

For our graph editor, we should define explicit constants for:

- node padding
- port row spacing
- minimum edge clearance
- selection margin
- canvas pan step

## 3. Primitive-first drawing

The visual quality comes from assembling simple primitives:

- bordered blocks for entities
- repeated line glyphs for connectors
- explicit arrowhead glyphs
- straightforward text placement

That matters because we should avoid early abstraction that makes routing harder to debug.

## 4. Theme with semantic roles

`tuigram` defines styles by semantic intent rather than per-widget styling:

- text
- muted
- accent
- border
- selected

We should keep that pattern and avoid style decisions leaking into graph logic.

## 5. Selection is a first-class render input

`tuigram` resolves selected state before rendering and changes style accordingly.
That is the right model for a graph canvas.

We should render selection through:

- selected node border and title style
- selected edge highlight
- selected port marker
- transient connection-preview style

## What Not To Copy

## 1. Sequence-specific coordinate model

`tuigram` spaces participants evenly and treats events as vertical rows.
Our graph editor needs freeform node positions, so equal spacing should not drive the layout.

## 2. Event-list data model

Its model is ordered messages and notes, not a true graph.
We need explicit nodes, ports, and edges with stable identities.

## 3. Diagram semantics mixed into mutation APIs

Methods like swapping participants or pointing events left/right are useful for sequence diagrams, but not reusable for a general graph editor.

## Visual Takeaways To Preserve

- Unicode box drawing over ASCII by default
- short, readable arrowheads
- modest color accents
- enough spacing that lines do not visually merge into labels
- selected elements highlighted by style, not by excessive ornament

## Resulting Direction

Use `tuigram` as a rendering reference for:

- box style
- edge style
- arrowhead style
- label placement
- theme roles

Do not use it as the data model or interaction model for the graph editor.
