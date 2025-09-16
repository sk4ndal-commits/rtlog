# TUI Layout

┌───────────────────────────────┐
│ Status Bar (filters, counts) │
├───────────────────────────────┤
│ Main Log Window               │
│ ───────────────────────────── │
│ Live log lines, colorized     │
├─────────────┬─────────────────┤
│ Filter Panel│ Summary / Stats │
│ Active filters, regex input   │
│                               │
├─────────────┴─────────────────┤
│ Context / Details (popup)     │
└───────────────────────────────┘

Notes
- Main log window occupies most of the screen and auto-scrolls while not paused.
- Filter panel + summary panel can be placed at the bottom or side depending on space.
- Context/details appear as a temporary overlay when a line is selected.

Keyboard (proposed)
- j/k or Down/Up: scroll lines
- g/G: jump to start/end
- Space: pause/resume auto-scroll
- /: focus filter input; Enter to apply
- n/N: next/previous match
- s: switch source (if multiple)
- a: toggle alerts panel
- c: toggle context display
- q: quit