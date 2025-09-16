# Architecture Overview

The application is organized into loosely coupled async components to keep UI rendering responsive while processing high-volume log streams.

- Input/Log sources
  - Single file follow (-f), stdin, or optional multi-file
  - File watcher tails appends; stdin is streamed line-by-line
- Processing pipeline
  - Multi-pattern matching (regex, ignore-case, etc.)
  - Filter evaluation producing match metadata and visibility
  - Context buffers to support before/after display around matches
- State and messaging
  - Shared AppState holds lines, filters, stats; kept minimal and safe
  - Tokio mpsc channels deliver new lines/events to the UI layer
  - UI and processing are decoupled via event messages
- TUI renderer
  - Main Log Window (scrolling, highlight matches)
  - Filter Panel (live input, active patterns)
  - Summary/Stats (processed/matched counts)
  - Context/Details popup on selection
  - Status Bar and Alerts
- Interaction
  - Keyboard input on a separate task; triggers state changes and redraws

This separation ensures performance (non-blocking UI), safety (no shared mutable state without synchronization), and portability (single binary).
