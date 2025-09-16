# Data Flow

1. Input
   - Async monitors for files and/or stdin are created (Tokio tasks)
   - File changes are tailed; stdin is read line-by-line
2. Fan-in
   - All sources are merged into a single async stream via mpsc channels
   - Backpressure handled by bounded channel and UI-friendly buffering
3. Processing
   - For each line: apply active filters and regex patterns
   - Derive match metadata (matched ranges) for highlighting
   - Update rolling context buffers to support before/after display
4. State update
   - Shared app state updates counters (processed, matched) and stores lines
   - Emit UI events to notify panels of updates, decoupled from processing
5. UI rendering
   - Main log window appends new lines and highlights matches
   - Filter panel reflects current patterns and input focus
   - Summary panel shows live stats; alerts panel raises critical notices
6. Interaction
   - Keyboard input handled on a separate task to avoid blocking
   - Supports scroll, filter edit, source switch, pause/resume, jump to next/prev match