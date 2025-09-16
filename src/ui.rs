//! TUI layer: rendering and input handling built on ratatui and crossterm.
//! The UI reads state immutably and emits `UiEvent` to keep concerns separated.

use crate::filter::{highlight_line, line_matches};
use crate::state::{AppState, FilterFocus};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Modifier, Color};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap, List, ListItem, Sparkline, Clear};
use ratatui::Terminal;
use std::io;

/// TUI façade over ratatui/crossterm. Owns the terminal and provides a `draw` method.
pub struct Ui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Ui {
    pub fn new() -> anyhow::Result<Self> {
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    pub fn restore(&mut self) -> anyhow::Result<()> {
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            self.terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    pub fn draw(&mut self, state: &AppState) -> anyhow::Result<()> {
        let filter_regs = state.enabled_regexes();
        let highlights = state.active_highlight_regexes();
        let alert_regs = state.alert_enabled_regexes();
        let now_ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
        let blink_on = (now_ms / 400) % 2 == 0;
        self.terminal.draw(|frame| {
            let area = frame.size();

            // Split horizontally: left sidebar (sources), right main panels
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(22), Constraint::Min(10)])
                .split(area);

            // Sidebar: list all sources, highlight focused
            let side_items: Vec<ListItem> = state.sources.iter().enumerate().map(|(i, s)| {
                let mut line = Line::from(s.name.clone());
                if i == state.focused {
                    line = apply_line_modifier(line, Modifier::REVERSED);
                }
                ListItem::new(line)
            }).collect();
            let side = List::new(side_items)
                .block(Block::default().borders(Borders::ALL).title("Sources (Tab/Shift-Tab, [/]): switch"));
            frame.render_widget(side, cols[0]);

            // Right area: logs, status, stats, and optional context/filter panels
            // Increase stats panel height to show more filter summaries
            let mut constraints = vec![Constraint::Min(1), Constraint::Length(1), Constraint::Length(10)];
            if state.context_panel_open {
                let h = (state.context_radius * 2 + 3) as u16;
                constraints.push(Constraint::Length(h.max(5)));
            }
            if state.filter_panel_open { constraints.push(Constraint::Length(10)); }
            let chunks = Layout::default().direction(Direction::Vertical).constraints(constraints).split(cols[1]);

            // Determine visible slice from the focused source
            let height = chunks[0].height as usize - 2; // borders
            let mut lines: Vec<Line> = Vec::new();
            let (total, scroll_offset, selected_log) = if let Some(src) = state.current_source() {
                (src.lines.len(), src.scroll_offset, src.selected_log)
            } else { (0, 0, None) };
            let start = if total > height { total.saturating_sub(height + scroll_offset) } else { 0 };
            let end = total.saturating_sub(scroll_offset);
            if let Some(src) = state.current_source() {
                for i in start..end {
                    let text = &src.lines[i];
                    if line_matches(text, &filter_regs) {
                        let mut line = highlight_line(text, &highlights);
                        // If this line matches an alert pattern, colorize it strongly
                        if line_matches(text, &alert_regs) {
                            // Make it red and optionally flashing reverse during active blink window
                            line = apply_line_color(line, Color::Red);
                            if now_ms < state.alert_blink_deadline_ms && blink_on {
                                line = apply_line_modifier(line, Modifier::REVERSED);
                            }
                        }
                        if let Some(sel) = selected_log { if sel == i { line = apply_line_modifier(line, Modifier::REVERSED); }}
                        lines.push(line);
                    }
                }
            }

            let title = if let Some(src) = state.current_source() { format!("Logs - {} (Enter:Context, j/k:select)", src.name) } else { "Logs".to_string() };
            let para = Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title(title))
                .style(Style::default())
                .wrap(Wrap { trim: false });
            frame.render_widget(para, chunks[0]);

            // Status bar: show active filters count and flags of input
            let active = filter_regs.len();
            let (auto, so) = if let Some(src) = state.current_source() { (src.auto_scroll, src.scroll_offset) } else { (true, 0) };
            let status = format!(
                "Lines: {}  Scroll: {}  Mode: {}  Filters: {}  [/] Filter Panel  Enter:{}  r:regex={} i:case={} w:word={} x:line={}",
                total,
                so,
                if auto { "Auto" } else { "Paused" },
                active,
                if state.filter_panel_open { "Add Filter" } else { "Toggle Context" },
                state.input_is_regex,
                state.input_case_insensitive,
                state.input_whole_word,
                state.input_whole_line,
            );
            let status_para = Paragraph::new(status)
                .block(Block::default().borders(Borders::TOP))
                .wrap(Wrap { trim: true });
            frame.render_widget(status_para, chunks[1]);

            // Summary / Stats panel
            draw_stats_panel(frame, chunks[2], state);

            let mut next_chunk = 3;
            if state.context_panel_open {
                if let Some(sel) = selected_log {
                    draw_context_panel(frame, chunks[next_chunk], state, sel);
                } else {
                    let empty = Paragraph::new("No selection").block(Block::default().borders(Borders::ALL).title("Context"));
                    frame.render_widget(empty, chunks[next_chunk]);
                }
                next_chunk += 1;
            }

            if state.filter_panel_open {
                draw_filter_panel(frame, chunks[next_chunk], state);
            }

            // Search overlay input (temporary)
            if state.search_open {
                let w = (area.width.saturating_sub(10)).min(60);
                let h = 3;
                let x = area.x + (area.width - w) / 2;
                let y = area.y + (area.height - h) / 2;
                let popup = Rect::new(x, y, w, h);
                frame.render_widget(Clear, popup);
                let title = format!("Search (r:{} i:{}) - Enter:apply Esc:close", state.search_is_regex, state.search_case_insensitive);
                let input = Paragraph::new(state.search_input.clone())
                    .block(Block::default().borders(Borders::ALL).title(title))
                    .wrap(Wrap { trim: false });
                frame.render_widget(input, popup);
            }

            // Alert popup/banner (non-blocking)
            if state.alert_deadline_ms > now_ms {
                let msg = state.alert_message.clone().unwrap_or_else(|| "Alert".into());
                let blink_active = now_ms < state.alert_blink_deadline_ms && blink_on;
                let content = if blink_active { format!("⚠ ALERT: {}", msg) } else { format!("ALERT: {}", msg) };
                let w = (area.width.saturating_sub(10)).min(60);
                let h = 3;
                let x = area.x + (area.width - w) / 2;
                let y = area.y + 1; // near top
                let popup = Rect::new(x, y, w, h);
                frame.render_widget(Clear, popup);
                let style = if blink_active { Style::default().fg(Color::Black).bg(Color::Red).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::Red).add_modifier(Modifier::BOLD) };
                let para = Paragraph::new(content)
                    .block(Block::default().borders(Borders::ALL).title("ALERT"))
                    .style(style)
                    .wrap(Wrap { trim: true });
                frame.render_widget(para, popup);
            }
        })?;
        Ok(())
    }
}

fn draw_filter_panel(frame: &mut ratatui::Frame<'_>, area: Rect, state: &AppState) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(area);

    // Input line with flags
    let input_title = format!("Filter Input (focus={}): r={} i={} w={} x={}",
        match state.filter_focus { FilterFocus::Input => "input", FilterFocus::List => "list" },
        state.input_is_regex, state.input_case_insensitive, state.input_whole_word, state.input_whole_line);
    let input = Paragraph::new(state.filter_input.clone())
        .block(Block::default().borders(Borders::ALL).title(input_title))
        .wrap(Wrap { trim: false });
    frame.render_widget(input, rows[0]);

    // Filters list
    let items: Vec<ListItem> = state.filters.iter().enumerate().map(|(i, f)| {
        let sel = if i == state.selected_filter { ">" } else { " " };
        let chk = if f.enabled { "[x]" } else { "[ ]" };
        let flags = format!("{}{}{}{}",
            if f.is_regex { 'r' } else { '-' },
            if f.case_insensitive { 'i' } else { '-' },
            if f.whole_word { 'w' } else { '-' },
            if f.whole_line { 'x' } else { '-' },
        );
        ListItem::new(Line::from(vec![
            Span::raw(format!("{} {} {} ", sel, chk, flags)),
            Span::styled(f.pattern.clone(), Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(format!("  ({} matches)", f.match_count)),
        ]))
    }).collect();
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Active Filters (Space:toggle, d:delete, Tab:switch focus)"));
    frame.render_widget(list, rows[1]);
}

fn draw_stats_panel(frame: &mut ratatui::Frame<'_>, area: Rect, state: &AppState) {
    // Split horizontally: left (summary text), right (sparklines stacked)
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Left: totals and per-filter counts
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(vec![Span::styled(
        format!("Total lines: {}", state.current_source().map(|s| s.lines.len()).unwrap_or(0)), 
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    )]));

    // Show counts for enabled filters only
    if state.filters.is_empty() {
        lines.push(Line::from("No filters configured. Press '/' to add."));
    } else {
        for f in state.filters.iter().filter(|f| f.enabled) {
            lines.push(Line::from(vec![
                Span::raw("• "),
                Span::styled(f.pattern.clone(), Style::default().fg(Color::Cyan)),
                Span::raw(format!(": {}", f.match_count)),
            ]));
        }
    }

    let text = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Summary / Stats"))
        .wrap(Wrap { trim: true });
    frame.render_widget(text, cols[0]);

    // Right: error/warn sparklines stacked
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(cols[1]);

    let err_data: Vec<u64> = state.err_buckets.iter().map(|&v| v as u64).collect();
    let warn_data: Vec<u64> = state.warn_buckets.iter().map(|&v| v as u64).collect();

    let err = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title("Errors/sec (last 60s)"))
        .data(&err_data)
        .style(Style::default().fg(Color::Red));
    frame.render_widget(err, rows[0]);

    let warn = Sparkline::default()
        .block(Block::default().borders(Borders::ALL).title("Warnings/sec (last 60s)"))
        .data(&warn_data)
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(warn, rows[1]);
}

fn apply_line_modifier(line: Line<'_>, modifier: Modifier) -> Line<'_> {
    // Apply a modifier to all spans in the line while preserving their colors/styles
    let spans = line.spans.into_iter().map(|mut s| {
        let mut style = s.style;
        style = style.add_modifier(modifier);
        s.style = style;
        s
    }).collect::<Vec<_>>();
    Line::from(spans)
}

fn apply_line_color(line: Line<'_>, color: Color) -> Line<'_> {
    // Apply a foreground color to all spans, preserving modifiers
    let spans = line.spans.into_iter().map(|mut s| {
        let mut style = s.style;
        style = style.fg(color);
        s.style = style;
        s
    }).collect::<Vec<_>>();
    Line::from(spans)
}

fn draw_context_panel(frame: &mut ratatui::Frame<'_>, area: Rect, state: &AppState, sel: usize) {
    let Some(src) = state.current_source() else { return; };
    let total = src.lines.len();
    if total == 0 { return; }
    let radius = state.context_radius;
    let from = sel.saturating_sub(radius);
    let to = (sel + radius + 1).min(total);

    let mut lines: Vec<Line> = Vec::new();
    for i in from..to {
        let content = src.lines[i].clone();
        let mut line = Line::from(content);
        if i == sel {
            // Highlight selected line distinctly in context view
            line = apply_line_modifier(line, Modifier::BOLD);
            // Add color for emphasis
            let spans = line.spans.into_iter().map(|mut s| { s.style = s.style.fg(Color::Cyan); s }).collect::<Vec<_>>();
            line = Line::from(spans);
        }
        lines.push(line);
    }

    let title = format!("Context (±{} lines around selected)", radius);
    let para = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(title))
        .wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

pub enum UiEvent {
    Quit,
    None,
    ScrollUp(usize),
    ScrollDown(usize),
    Top,
    Bottom,
    ToggleAuto,

    ToggleFilterPanel,
    ToggleContextPanel,
    InputChar(char),
    Backspace,
    AddFilter,
    ToggleInputRegex,
    ToggleInputCase,
    ToggleInputWord,
    ToggleInputLine,
    ToggleFilterEnabled,
    DeleteFilter,
    FocusNext,
    SelectUp,
    SelectDown,
    NextSource,
    PrevSource,

    // Search
    ToggleSearch,
    CloseSearch,
    SearchChar(char),
    SearchBackspace,
    ApplySearch,
    NextMatch,
    PrevMatch,
    ToggleSearchRegex,
    ToggleSearchCase,
}

pub fn poll_input(state: &AppState) -> anyhow::Result<UiEvent> {
    if event::poll(std::time::Duration::from_millis(10))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if state.search_open {
                    return Ok(match key.code {
                        KeyCode::Esc => UiEvent::CloseSearch,
                        KeyCode::Enter => UiEvent::ApplySearch,
                        KeyCode::Backspace => UiEvent::SearchBackspace,
                        KeyCode::Char('r') => UiEvent::ToggleSearchRegex,
                        KeyCode::Char('i') => UiEvent::ToggleSearchCase,
                        KeyCode::Char(c) if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT => UiEvent::SearchChar(c),
                        _ => UiEvent::None,
                    });
                }
                return Ok(match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => UiEvent::Quit,
                    KeyCode::Up => UiEvent::ScrollUp(1),
                    KeyCode::Down => UiEvent::ScrollDown(1),
                    KeyCode::PageUp => UiEvent::ScrollUp(10),
                    KeyCode::PageDown => UiEvent::ScrollDown(10),
                    KeyCode::Home => UiEvent::Top,
                    KeyCode::End => UiEvent::Bottom,
                    KeyCode::Char(' ') if key.modifiers.is_empty() => { if state.filter_panel_open && matches!(state.filter_focus, FilterFocus::List) { UiEvent::ToggleFilterEnabled } else { UiEvent::ToggleAuto } },

                    KeyCode::Char('/') => UiEvent::ToggleFilterPanel,
                    KeyCode::Char('?') => UiEvent::ToggleSearch,
                    KeyCode::Enter => { if state.filter_panel_open { UiEvent::AddFilter } else { UiEvent::ToggleContextPanel } },
                    KeyCode::Backspace => UiEvent::Backspace,
                    KeyCode::Tab => UiEvent::FocusNext,
                    KeyCode::BackTab => UiEvent::PrevSource,
                    KeyCode::Char(']') => UiEvent::NextSource,
                    KeyCode::Char('[') => UiEvent::PrevSource,
                    KeyCode::Char('r') => UiEvent::ToggleInputRegex,
                    KeyCode::Char('i') => UiEvent::ToggleInputCase,
                    KeyCode::Char('w') => UiEvent::ToggleInputWord,
                    KeyCode::Char('x') => UiEvent::ToggleInputLine,
                    KeyCode::Char('d') => UiEvent::DeleteFilter,
                    KeyCode::Char('k') => UiEvent::SelectUp,
                    KeyCode::Char('j') => UiEvent::SelectDown,
                    KeyCode::Char('n') if key.modifiers.is_empty() && !(state.filter_panel_open && matches!(state.filter_focus, FilterFocus::Input)) => UiEvent::NextMatch,
                    KeyCode::Char('N') if !(state.filter_panel_open && matches!(state.filter_focus, FilterFocus::Input)) => UiEvent::PrevMatch,
                    KeyCode::Char(c) if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT => UiEvent::InputChar(c),
                    _ => UiEvent::None,
                });
            }
        }
    }
    Ok(UiEvent::None)
}
