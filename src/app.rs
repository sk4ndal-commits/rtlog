use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use tokio::sync::mpsc;

use crate::filter::build_filter;
use crate::log::stream_file;
use crate::state::{AppState, FilterFocus};
use crate::ui::{poll_input, Ui, UiEvent};

use crate::cli::Config;

fn discover_files(inputs: &[PathBuf], recursive: bool) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack: Vec<PathBuf> = inputs.to_vec();
    while let Some(p) = stack.pop() {
        if let Ok(md) = fs::metadata(&p) {
            if md.is_file() {
                files.push(p);
            } else if md.is_dir() {
                if let Ok(rd) = fs::read_dir(&p) {
                    for entry in rd.flatten() {
                        let path = entry.path();
                        if let Ok(md2) = entry.metadata() {
                            if md2.is_file() { files.push(path); }
                            else if md2.is_dir() && recursive { stack.push(path); }
                        }
                    }
                }
            }
        }
    }
    files.sort();
    files.dedup();
    files
}

/// Application runtime: wires inputs, state, and UI.
pub async fn run(config: Config) -> Result<()> {
    // Build filter from config
    let filter: Option<Regex> = build_filter(config.regex.as_deref())?;

    // Resolve input files
    let files = discover_files(&config.inputs, config.recursive);

    // Channel for log lines tagged with source id
    let (tx, mut rx) = mpsc::channel::<(usize, String)>(1024);

    // Spawn log readers
    for (i, path) in files.iter().cloned().enumerate() {
        let txc = tx.clone();
        let follow = config.follow;
        tokio::spawn(async move {
            let _ = stream_file(path, follow, i, txc).await;
        });
    }

    // Initialize UI and state
    let mut state = AppState::new(filter);
    let sources_meta = files.iter().map(|p| {
        let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("?").to_string();
        (name, p.clone())
    });
    state.set_sources(sources_meta);
    let mut ui = Ui::new()?;

    // Main loop
    let mut last_draw = std::time::Instant::now();
    let draw_interval = std::time::Duration::from_millis(33); // ~30fps max

    let res = loop {
        // Drain any available lines without blocking
        while let Ok((sid, line)) = rx.try_recv() {
            state.push_line_for(sid, line);
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
            UiEvent::ToggleContextPanel => {
                // Initialize selection if needed, then toggle
                state.ensure_log_selection();
                state.context_panel_open = !state.context_panel_open;
            }
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
            UiEvent::SelectUp => { if state.filter_panel_open { state.move_selection_up(); } else { state.move_log_selection_up(); } }
            UiEvent::SelectDown => { if state.filter_panel_open { state.move_selection_down(); } else { state.move_log_selection_down(); } }
            UiEvent::NextSource => { state.focus_next_source(); }
            UiEvent::PrevSource => { state.focus_prev_source(); }

            // Search controls
            UiEvent::ToggleSearch => { state.open_search(); }
            UiEvent::CloseSearch => { state.close_search(); }
            UiEvent::SearchChar(c) => { state.search_push_char(c); }
            UiEvent::SearchBackspace => { state.search_pop_char(); }
            UiEvent::ApplySearch => { state.apply_search(); state.search_open = false; }
            UiEvent::NextMatch => { let _ = state.jump_next_match(); }
            UiEvent::PrevMatch => { let _ = state.jump_prev_match(); }
            UiEvent::ToggleSearchRegex => { state.search_is_regex = !state.search_is_regex; }
            UiEvent::ToggleSearchCase => { state.search_case_insensitive = !state.search_case_insensitive; }
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
