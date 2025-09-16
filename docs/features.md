# rtlog Features

Real-time, portable, and safe TUI for monitoring logs on customer machines.

- Input sources
  - Follow a single file (-f path) or read from stdin (pipe)
  - Optional multi-file follow (tail -F–like) without modifying sources
- Filtering and matching
  - Multiple patterns (-e) with regex support
  - Ignore-case (-i), whole-word, and whole-line options (planned)
  - Live filter input panel to refine results without restarting
- Context
  - Display before/after context lines (-B/-A) around matched selections
  - Quick toggle for showing only matches vs. full stream
- Rendering and UX
  - Colorized highlights for matched substrings and critical entries
  - Scrollable main log window with pause/resume auto-scroll
  - Smooth updates under high volume; non-blocking UI
- Status and telemetry
  - Status bar with processed line count, match count, active filters, and source name
  - Optional alerts panel for critical patterns (ERROR, panic, etc.)
- Safety and portability
  - Read-only operation; never alters log files
  - Single-binary deployable; no external runtimes required