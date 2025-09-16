use crate::filter::{highlight_line, line_matches};
use crate::state::{AppState, FilterFocus};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Modifier, Color};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap, List, ListItem, Sparkline};
use ratatui::Terminal;
use std::io;

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
        let enabled = state.enabled_regexes();
        self.terminal.draw(|frame| {
            let area = frame.size();

            // Layout: logs, status, stats, and optional context/filter panels
            let mut constraints = vec![Constraint::Min(1), Constraint::Length(1), Constraint::Length(5)];
            // Context panel sits between stats and filter panels
            if state.context_panel_open { 
                // height: 2*radius + 3 (border + title + padding). Keep minimal 5.
                let h = (state.context_radius * 2 + 3) as u16;
                constraints.push(Constraint::Length(h.max(5)));
            }
            if state.filter_panel_open { constraints.push(Constraint::Length(4)); }
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(constraints)
                .split(area);

            // Determine visible slice based on scroll_offset
            let height = chunks[0].height as usize - 2; // account for borders
            let mut lines: Vec<Line> = Vec::new();
            let total = state.lines.len();
            let start = if total > height {
                total.saturating_sub(height + state.scroll_offset)
            } else { 0 };
            let end = total.saturating_sub(state.scroll_offset);
            for i in start..end {
                let text = &state.lines[i];
                if line_matches(text, &enabled) {
                    let mut line = highlight_line(text, &enabled);
                    if let Some(sel) = state.selected_log { if sel == i {
                        line = apply_line_modifier(line, Modifier::REVERSED);
                    }}
                    lines.push(line);
                }
            }

            let para = Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title("Logs (Enter:Context, j/k:select)"))
                .style(Style::default())
                .wrap(Wrap { trim: false });
            frame.render_widget(para, chunks[0]);

            // Status bar: show active filters count and flags of input
            let active = enabled.len();
            let status = format!(
                "Lines: {}  Scroll: {}  Mode: {}  Filters: {}  [/] Filter Panel  Enter:{}  r:regex={} i:case={} w:word={} x:line={}",
                total,
                state.scroll_offset,
                if state.auto_scroll { "Auto" } else { "Paused" },
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
                if let Some(sel) = state.selected_log { 
                    draw_context_panel(frame, chunks[next_chunk], state, sel);
                } else {
                    // no selection yet; nothing to draw but keep reserved space
                    let empty = Paragraph::new("No selection").block(Block::default().borders(Borders::ALL).title("Context"));
                    frame.render_widget(empty, chunks[next_chunk]);
                }
                next_chunk += 1;
            }

            if state.filter_panel_open {
                draw_filter_panel(frame, chunks[next_chunk], state);
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
        format!("Total lines: {}", state.lines.len()),
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

fn draw_context_panel(frame: &mut ratatui::Frame<'_>, area: Rect, state: &AppState, sel: usize) {
    let total = state.lines.len();
    if total == 0 { return; }
    let radius = state.context_radius;
    let from = sel.saturating_sub(radius);
    let to = (sel + radius + 1).min(total);

    let mut lines: Vec<Line> = Vec::new();
    for i in from..to {
        let content = state.lines[i].clone();
        // Optional: prefix with index for clarity
        // content = format!("{:>6}: {}", i + 1, content);
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
}

pub fn poll_input(state: &AppState) -> anyhow::Result<UiEvent> {
    if event::poll(std::time::Duration::from_millis(10))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
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
                    KeyCode::Enter => { if state.filter_panel_open { UiEvent::AddFilter } else { UiEvent::ToggleContextPanel } },
                    KeyCode::Backspace => UiEvent::Backspace,
                    KeyCode::Tab => UiEvent::FocusNext,
                    KeyCode::Char('r') => UiEvent::ToggleInputRegex,
                    KeyCode::Char('i') => UiEvent::ToggleInputCase,
                    KeyCode::Char('w') => UiEvent::ToggleInputWord,
                    KeyCode::Char('x') => UiEvent::ToggleInputLine,
                    KeyCode::Char('d') => UiEvent::DeleteFilter,
                    KeyCode::Char('k') => UiEvent::SelectUp,
                    KeyCode::Char('j') => UiEvent::SelectDown,
                    KeyCode::Char(c) if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT => UiEvent::InputChar(c),
                    _ => UiEvent::None,
                });
            }
        }
    }
    Ok(UiEvent::None)
}
