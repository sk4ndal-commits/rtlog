# Data Flow

This document explains the runtime data flow from inputs to UI.

1. CLI → Config
   - `cli::parse()` returns an immutable `Config` containing inputs, flags, and alert patterns.

2. Discovery → Sources
   - `app::run` discovers files (optionally recursively) and assigns each a source ID.
   - UI sidebar lists the discovered sources.

3. Ingestion → Channel
   - For each path, the runtime spawns a task that uses `log::FileTail` (via `stream_file`) to read lines.
   - Lines are sent as `(source_id, String)` over a bounded `tokio::mpsc` channel.

4. Channel → State
   - The main loop non‑blocking drains the channel and calls `AppState::push_line_for`.
   - `AppState` classifies lines for stats and checks alert rules.

5. Input → Events
   - `ui::poll_input` translates terminal events to a small `UiEvent` enum.
   - The main loop mutates `AppState` fields or calls methods based on the event.

6. State → UI
   - `Ui::draw` reads from `AppState` to render panels.
   - Filtering and highlighting are applied only to visible lines using helpers in `filter.rs`.

This decoupling allows ingestion, processing, and rendering to evolve independently and keeps the UI responsive under load.
