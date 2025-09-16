# Architecture Overview

rtlog follows a modular architecture aligned with the SOLID principles:

- Single Responsibility: Each module focuses on one concern: CLI parsing (cli.rs), runtime orchestration (app.rs), TUI (ui.rs), state (state.rs), filtering/highlighting (filter.rs), and log ingestion (log.rs).
- Open/Closed: The log ingestion layer is extensible via the `LogSource` trait; new backends (e.g., sockets, journald) can be added without changing consumers.
- Liskov Substitution: Any `LogSource` implementor can be substituted in the runtime. The UI consumes read‑only state views and does not depend on concrete implementations.
- Interface Segregation: The UI interacts with `AppState` read-only in draw and with small event enums. The log layer exposes a minimal `LogSource` interface.
- Dependency Inversion: The runtime (`app.rs`) depends on the `LogSource` abstraction rather than a concrete file tailer.

## Modules

- src/main.rs — Thin entry point, starts async runtime with parsed config.
- src/cli.rs — CLI parsing and configuration.
- src/app.rs — Application runtime: wires inputs, spawns tasks, runs the event/render loop.
- src/log.rs — Log ingestion interfaces and file‑tail implementation.
- src/filter.rs — Pattern rules, compilation, filtering, and highlighting.
- src/state.rs — Application state: sources, filters, selection, stats, search, and alerts.
- src/ui.rs — TUI rendering and input handling.

## Data Flow

1. CLI is parsed into an immutable Config.
2. Runtime discovers input files, spawns a task per source using the log ingestion interface.
3. Log lines flow over a bounded mpsc channel to the runtime loop, which pushes them into `AppState`.
4. UI polls input and emits UI events; runtime mutates `AppState` accordingly.
5. UI renders from `AppState` on a fixed cadence (~30 FPS), keeping processing decoupled from rendering.

## Extensibility Points

- Implement `LogSource` to add new ingestion backends.
- Add new panels or widgets by extending `ui.rs` while keeping `AppState` the single source of truth.
- Add new classification or alert rules by building on top of `filter.rs` utilities.

## Concurrency Model

- Tokio multi-threaded runtime.
- One task per input source doing async I/O.
- Main loop handles UI input, drains channel without blocking, and renders.

## Safety

- Read-only file access.
- No external command execution.
- Regexes compiled up-front when possible; fallbacks remain safe.
