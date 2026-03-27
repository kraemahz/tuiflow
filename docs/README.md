# tui_dnd Docs

This directory is the design workspace for the repository.

The current goal is to design a `ratatui`-based directed-graph editor with:

- movable node boxes
- directed edges between ports
- keyboard-first editing
- room for optional mouse support later

We are using `tuigram` as a visual and interaction reference, not as a base dependency.

## Structure

- `references/tuigram.md`
  Notes from the `tuigram` source code that are worth borrowing.
- `design/graph_canvas.md`
  Initial architecture for a graph canvas, renderer, and editor model.
- `design/roadmap.md`
  Near-term milestones for getting from blank repo to usable editor.

## Working Rules

- Prefer documenting design intent before implementation.
- Treat external crates as references unless they are mature enough to become dependencies.
- Keep rendering concerns separate from graph mutation and interaction logic.
