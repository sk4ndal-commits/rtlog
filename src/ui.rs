use crate::filter::highlight_line;
use crate::state::AppState;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
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
        self.terminal.draw(|frame| {
            let area = frame.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(1), // main log window
                    Constraint::Length(1), // status bar
                ])
                .split(area);

            // Determine visible slice based on scroll_offset
            let height = chunks[0].height as usize - 2; // account for borders
            let mut lines: Vec<Line> = Vec::new();
            let total = state.lines.len();
            let start = if total > height {
                total.saturating_sub(height + state.scroll_offset)
            } else {
                0
            };
            let end = total.saturating_sub(state.scroll_offset);
            for i in start..end {
                lines.push(highlight_line(&state.lines[i], &state.filter));
            }

            let para = Paragraph::new(lines)
                .block(Block::default().borders(Borders::ALL).title("Logs"))
                .style(Style::default())
                .wrap(Wrap { trim: false });
            frame.render_widget(para, chunks[0]);

            // Status bar
            let status = format!(
                "Lines: {}  Scroll: {}  Mode: {}",
                total,
                state.scroll_offset,
                if state.auto_scroll { "Auto" } else { "Paused" }
            );
            let status_para = Paragraph::new(status)
                .block(Block::default().borders(Borders::TOP))
                .wrap(Wrap { trim: true });
            frame.render_widget(status_para, chunks[1]);
        })?;
        Ok(())
    }
}

pub enum UiEvent {
    Quit,
    None,
    ScrollUp(usize),
    ScrollDown(usize),
    Top,
    Bottom,
    ToggleAuto,
}

pub fn poll_input() -> anyhow::Result<UiEvent> {
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
                    KeyCode::Char(' ') => UiEvent::ToggleAuto,
                    _ => UiEvent::None,
                });
            }
        }
    }
    Ok(UiEvent::None)
}
