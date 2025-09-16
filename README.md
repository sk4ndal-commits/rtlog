# rtlog — Real‑time Log Viewer (TUI)

rtlog is a portable, real‑time, interactive log monitoring tool with a Text User Interface (TUI). It is designed to tail log files, highlight patterns, and keep the UI smooth even under high log volume.

Core qualities:
- Performance: non‑blocking UI with async I/O.
- Portability: single binary, minimal dependencies.
- Usability: live scrolling, status bar, and pattern highlighting.
- Safety: read‑only operation; never modifies log files.


## Features
- Follow a log file (tail -f–like)
- Regex highlighting (case‑insensitive)
- Real-time multi-pattern filtering with a Filter Panel
- Toggle whole-word (-w) and whole-line (-x) matching per filter
- Quickly enable/disable filters and delete them
- Smooth auto‑scroll with pause/resume
- Scrollback navigation (Up/Down, PageUp/PageDown, Home/End)
- Context/Details view: inspect ±N lines around a selected entry without losing scroll position
- Status bar with line count, scroll offset, auto-scroll mode, and active filters
- New: Summary / Stats panel with live counts and sparklines for errors/warnings
- New: Multi-source monitoring with a Sources sidebar and per-source focus
- New: Search / Jump overlay to find and navigate matches quickly
- New: Alerts / Highlighting for critical entries (ERROR, FATAL by default) with non-blocking flashing banner

See docs/ for more details:
- docs/features.md
- docs/architecture.md
- docs/tui_layout.md
- docs/components.md
- docs/dataflow.md


## Installation

### Build from source (Rust toolchain required)
- Prerequisites: Rust 1.78+ (or recent stable), Cargo
- Clone this repo and build:

```
cargo build --release
```

The resulting binary will be at:
- target/release/rtlog

### Optional: static binary builds
- Linux (musl):
  - Install musl target: `rustup target add x86_64-unknown-linux-musl`
  - Build: `cargo build --release --target x86_64-unknown-linux-musl`
  - Binary: `target/x86_64-unknown-linux-musl/release/rtlog`
- Windows: standard MSVC builds typically suffice for portability.

Note: Statically linking all dependencies on every platform can vary by system toolchain. For customer delivery, prefer packaging the release binary directly.


## Usage

Synopsis:
```
rtlog [OPTIONS] PATH...
```

Arguments:
- PATH...  One or more paths to log files or directories.
           If a directory is provided, files within will be added; use -R/--recursive to walk subdirectories.

Options:
- -f, --follow         Follow the files for appended lines (tail -f)
- -r, --regex PAT      Initial regex to highlight (case‑insensitive). This is optional; you can add more patterns from the Filter Panel at runtime.
- -R, --recursive      When a PATH is a directory, include files from subdirectories recursively.
-     --alert PAT      Pattern that triggers a visual alert (repeatable). Defaults: ERROR, FATAL.
- -V, --version        Show version
- -h, --help           Show help

Examples:
- Follow syslog and highlight error lines:
  ```
  rtlog -f -r "error|failed|panic" /var/log/syslog
  ```
- View a static file once (no follow), highlighting IPv4 addresses:
  ```
  rtlog -r "\b(\d{1,3}\.){3}\d{1,3}\b" ./app.log
  ```
- Monitor multiple files at once:
  ```
  rtlog -f ./app.log ./db.log
  ```
- Monitor a directory recursively:
  ```
  rtlog -f -R /var/log
  ```
- Mix files and directories:
  ```
  rtlog -f -R ./services/ /var/log/syslog ./custom.log
  ```

Notes:
- Piped stdin input is not yet supported.
- The Filter Panel is the primary way to add multiple filters interactively; CLI -r is kept for convenience and quick start.


## TUI Controls
- q or Esc   Quit
- Space      Toggle auto‑scroll (Auto/Paused) or toggle selected filter when Filter Panel list has focus
- Up/Down    Scroll by 1
- PageUp/Down  Scroll by 10
- Home/End   Jump to top/bottom
- /          Open/close Filter Panel
- ?          Open Search overlay (temporary popup)
- Enter      When Filter Panel open: add filter from input; when Search overlay open: apply search; otherwise: open/close Context View for the selected log line
- Backspace  Delete last character in current input (Filter Panel or Search overlay)
- Tab        Switch focus between input and filter list
- Shift+Tab  Switch to previous source (in Sources sidebar)
- [ / ]      Switch focused source backward/forward (Sources sidebar); main log view updates to that source
- r/i/w/x    Toggle flags on filter input: regex, case-insensitive, whole-word, whole-line
- In Search overlay: r toggles regex mode; i toggles case-insensitive
- n / N      Jump to next / previous match (uses the last applied search)
- d          Delete selected filter (when Filter Panel list has focus)
- j/k        Move selection down/up (in Filter Panel list when open; otherwise selects a log line in the main view)

Status bar shows: total lines, current scroll offset, auto‑scroll mode, active filter count, and current input flags.


## Filter Panel
- Open/close with `/`. The panel shows an input line and the list of active filters.
- Type a pattern in the input. Press Enter to add it as a new filter rule.
- Flags on input:
  - r: treat input as regex (otherwise literal text)
  - i: case-insensitive matching (default on)
  - w: whole-word match (wraps with word boundaries)
  - x: whole-line match (anchors with ^ and $)
- Focus: use Tab to switch between input and filter list.
- In the filter list:
  - Space toggles the selected filter enabled/disabled
  - d deletes the selected filter
  - j/k move selection down/up

Matching behavior:
- If no filters are enabled, all lines are shown.
- If one or more filters are enabled, a line is shown if it matches any enabled filter (logical OR).
- Highlights are applied to all matching ranges from all enabled filters.

## Context / Details View
- Purpose: Inspect lines around a selected log entry to understand its context.
- Open/close: Press Enter when the Filter Panel is closed. This toggles the Context View for the currently selected log line.
- Selecting a line: Use j/k (with the Filter Panel closed) to move the selection up/down in the main log view. The selected line is highlighted.
- Display: Shows ±N neighboring lines around the selection (default N=3). The selected line is emphasized.
- Scroll position: Opening and closing the Context View does not change your current scroll position in the main log view.

## Alerts / Highlighting
- Purpose: Visually surface critical lines immediately.
- Defaults: ERROR and FATAL trigger alerts if you don't pass any --alert options.
- User-defined: Use --alert multiple times to define your own patterns (literal, case-insensitive). Example: --alert timeout --alert "connection lost" --alert "panic".
- Behavior:
  - Lines matching an alert pattern are colored red in the main log view.
  - A small non-blocking flashing banner appears near the top for ~3 seconds showing the alert text.
  - The alert overlay never pauses auto-scroll or blocks input; it is purely visual and transient.

## Summary / Stats Panel
- Always visible beneath the status bar.
- Left side shows:
  - Total lines processed (since program start).
  - Counts of matches for each enabled filter pattern. These counts update in real time as new lines arrive.
- Right side shows:
  - Two rolling sparklines over the last 60 seconds: Errors/sec (red) and Warnings/sec (yellow).
  - Classification is heuristic and file-agnostic: it looks for case-insensitive substrings "error" and "warn" in lines.

Tips:
- Use the Filter Panel ('/') to add patterns you care about; their match counters will start incrementing immediately.
- The stats are kept lightweight and updated incrementally to avoid blocking the UI.


## How it works (high level)
- Async runtime (Tokio) streams file lines without blocking rendering.
- A background task tails the file (when --follow is enabled).
- The UI layer (ratatui + crossterm) renders the main log view, status bar, and the Filter Panel.
- Highlights and filtering are applied to visible lines only for performance.
- UI and processing communicate through lightweight state and events.


## Safety and limitations
- Read‑only: rtlog never modifies your log files.
- File‑agnostic: no assumptions about log format (free‑text lines).
- Single file input in this version; multi‑file and stdin may be added later.
- Regular expressions are case‑insensitive by default in the provided option.


## Troubleshooting
- Permission denied: make sure your user can read the log file.
- No updates when using --follow: the source may not be appending, or you may be looking at a rotated file; try reopening on the new file.
- Terminal glitches: if the UI is corrupted after exit, run `reset` in your terminal.
- Windows terminals: prefer Windows Terminal or recent PowerShell for full ANSI support.


## License
This project is licensed under the MIT license (unless otherwise stated in the repository).
