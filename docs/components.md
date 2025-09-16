| Component                | Rust Crate / Approach                                                                     |
| ------------------------ | ----------------------------------------------------------------------------------------- |
| TUI framework            | [`tui`](https://crates.io/crates/tui) + [`crossterm`](https://crates.io/crates/crossterm) |
| Async runtime            | [`tokio`](https://crates.io/crates/tokio) for non-blocking tasks and channels             |
| Log input (tail)         | [`notify`](https://crates.io/crates/notify) for file changes; async file reads            |
| Regex / Pattern matching | [`regex`](https://crates.io/crates/regex)                                                 |
| Multi-source handling    | Fan-in with `tokio::sync::mpsc` and `tokio::select!`                                      |
| Colorized highlighting   | `tui` styles; optional [`ansi_term`](https://crates.io/crates/ansi_term)                  |
| Configuration (optional) | Load from a simple file (filters/colors)                                                  |
| Alerts / notifications   | TUI pop-up or panel triggered by detected patterns                                        |
