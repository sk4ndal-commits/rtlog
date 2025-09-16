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
- Status bar with line count, scroll offset, auto-scroll mode, and active filters

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
rtlog [OPTIONS] FILE
```

Arguments:
- FILE  Path to the log file to read.

Options:
- -f, --follow       Follow the file for appended lines (tail -f)
- -e, --regex PAT    Initial regex to highlight (case‑insensitive). This is optional; you can add more patterns from the Filter Panel at runtime.
- -V, --version      Show version
- -h, --help         Show help

Examples:
- Follow syslog and highlight error lines:
  ```
  rtlog -f -e "error|failed|panic" /var/log/syslog
  ```
- View a static file once (no follow), highlighting IPv4 addresses:
  ```
  rtlog -e "\b(\d{1,3}\.){3}\d{1,3}\b" ./app.log
  ```

Notes:
- Current version expects a file path. Piped stdin input is not yet supported.
- The Filter Panel is the primary way to add multiple filters interactively; CLI -e is kept for convenience and backward compatibility.


## TUI Controls
- q or Esc   Quit
- Space      Toggle auto‑scroll (Auto/Paused) or toggle selected filter when Filter Panel list has focus
- Up/Down    Scroll by 1
- PageUp/Down  Scroll by 10
- Home/End   Jump to top/bottom
- /          Open/close Filter Panel
- Enter      Add filter from input
- Backspace  Delete last character in filter input
- Tab        Switch focus between input and filter list
- r/i/w/x    Toggle flags on filter input: regex, case-insensitive, whole-word, whole-line
- d          Delete selected filter (when Filter Panel list has focus)
- j/k        Move selection down/up in filter list

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
