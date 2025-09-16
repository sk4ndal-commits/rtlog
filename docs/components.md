# Components

This document outlines the main components of rtlog and their responsibilities.

- CLI (src/cli.rs)
  - Parses command line flags and arguments with `clap`.
  - Produces an immutable `Config` used by the runtime.

- Runtime (src/app.rs)
  - Discovers files/directories.
  - Spawns ingestion tasks per source.
  - Runs the main loop: drain channel, handle input, render UI.

- Log Ingestion (src/log.rs)
  - Defines `LogSource` trait.
  - Provides `FileTail` implementation and `stream_file` helper.

- Filtering & Highlighting (src/filter.rs)
  - FilterRule representation and compilation.
  - `line_matches` and `highlight_line` helpers.

- State (src/state.rs)
  - Central state for sources, filters, search, alerts, stats.
  - Small methods to mutate state in response to events.

- UI (src/ui.rs)
  - Rendering with ratatui.
  - Input polling and translation to `UiEvent`.

## Future Components

- Search and Alerts could be extracted into dedicated modules without changing the UI layer.
- Additional `LogSource` implementors (e.g., sockets, journald, stdin) can be added.
