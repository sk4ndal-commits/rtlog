use anyhow::Result;
use regex::Regex;
use tokio::sync::mpsc;

use crate::filter::build_filter;
use crate::log::stream_file;
use crate::state::{AppState, FilterFocus};
use crate::ui::{poll_input, Ui, UiEvent};

use crate::cli::Config;

/// Application runtime: wires inputs, state, and UI.
pub async fn run(config: Config) -> Result<()> {
    // Build filter from config
    let filter: Option<Regex> = build_filter(config.regex.as_deref())?;

    // Channel for log lines
    let (tx, mut rx) = mpsc::channel::<String>(1024);

    // Spawn log reader
    let path = config.file.clone();
    let follow = config.follow;
    tokio::spawn(async move {
        let _ = stream_file(path, follow, tx).await;
    });

    // Initialize UI and state
    let mut state = AppState::new(filter);
    let mut ui = Ui::new()?;

    // Main loop
    let mut last_draw = std::time::Instant::now();
    let draw_interval = std::time::Duration::from_millis(33); // ~30fps max

    let res = loop {
        // Drain any available lines without blocking
        while let Ok(line) = rx.try_recv() {
            state.push_line(line);
        }

        // Handle user input
        match poll_input(&state)? {
            UiEvent::Quit => break Ok(()),
            UiEvent::None => {}
            UiEvent::ScrollUp(n) => state.scroll_up(n),
            UiEvent::ScrollDown(n) => state.scroll_down(n),
            UiEvent::Top => state.scroll_top(),
            UiEvent::Bottom => state.scroll_bottom(),
            UiEvent::ToggleAuto => state.toggle_auto_scroll(),

            UiEvent::ToggleFilterPanel => { state.filter_panel_open = !state.filter_panel_open; },
            UiEvent::InputChar(c) => {
                if state.filter_panel_open && matches!(state.filter_focus, FilterFocus::Input) { state.filter_input.push(c); }
            }
            UiEvent::Backspace => {
                if state.filter_panel_open && matches!(state.filter_focus, FilterFocus::Input) { state.filter_input.pop(); }
            }
            UiEvent::AddFilter => {
                if state.filter_panel_open { state.add_filter_from_input(); }
            }
            UiEvent::ToggleInputRegex => { if state.filter_panel_open { state.input_is_regex = !state.input_is_regex; } }
            UiEvent::ToggleInputCase => { if state.filter_panel_open { state.input_case_insensitive = !state.input_case_insensitive; } }
            UiEvent::ToggleInputWord => { if state.filter_panel_open { state.input_whole_word = !state.input_whole_word; } }
            UiEvent::ToggleInputLine => { if state.filter_panel_open { state.input_whole_line = !state.input_whole_line; } }
            UiEvent::ToggleFilterEnabled => { if state.filter_panel_open { state.toggle_selected_filter(); } }
            UiEvent::DeleteFilter => { if state.filter_panel_open { state.remove_selected_filter(); } }
            UiEvent::FocusNext => { if state.filter_panel_open { state.filter_focus = match state.filter_focus { FilterFocus::Input => FilterFocus::List, FilterFocus::List => FilterFocus::Input }; } }
            UiEvent::SelectUp => { if state.filter_panel_open { state.move_selection_up(); } }
            UiEvent::SelectDown => { if state.filter_panel_open { state.move_selection_down(); } }
        }

        // Draw at most 30fps
        let should_draw = last_draw.elapsed() >= draw_interval;
        if should_draw {
            ui.draw(&state)?;
            last_draw = std::time::Instant::now();
        } else {
            // small sleep to reduce CPU
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    };

    // Ensure UI is restored even if error
    let _ = ui.restore();
    res
}
