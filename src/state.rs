use crate::filter::{compile_enabled_rules, FilterRule};
use std::collections::VecDeque;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterFocus { #[default] Input, List }

#[derive(Debug, Default)]
pub struct Source {
    pub name: String,
    pub path: PathBuf,
    pub lines: Vec<String>,
    pub scroll_offset: usize,
    pub auto_scroll: bool,
    pub selected_log: Option<usize>,
}

#[derive(Default)]
pub struct AppState {
    // Multiple sources
    pub sources: Vec<Source>,
    pub focused: usize,

    // Filter system (global)
    pub filters: Vec<FilterRule>,
    pub filter_panel_open: bool,
    pub filter_input: String,
    pub input_is_regex: bool,
    pub input_case_insensitive: bool,
    pub input_whole_word: bool,
    pub input_whole_line: bool,
    pub filter_focus: FilterFocus,
    pub selected_filter: usize,

    // Context/details view (per focused source)
    pub context_panel_open: bool,
    pub context_radius: usize,

    // Stats: rolling counts per second for last N seconds (global)
    pub err_buckets: VecDeque<u16>,
    pub warn_buckets: VecDeque<u16>,
    pub bucket_epoch_sec: u64,
}

const SPARK_WINDOW: usize = 60;

impl AppState {
    pub fn new(initial_cli_regex: Option<regex::Regex>) -> Self {
        let now_sec = current_epoch_sec();
        let mut s = Self {
            sources: Vec::new(),
            focused: 0,
            filters: Vec::new(),
            filter_panel_open: false,
            filter_input: String::new(),
            input_is_regex: false,
            input_case_insensitive: true,
            input_whole_word: false,
            input_whole_line: false,
            filter_focus: FilterFocus::Input,
            selected_filter: 0,
            context_panel_open: false,
            context_radius: 3,
            err_buckets: VecDeque::from(vec![0; SPARK_WINDOW]),
            warn_buckets: VecDeque::from(vec![0; SPARK_WINDOW]),
            bucket_epoch_sec: now_sec.saturating_sub(SPARK_WINDOW as u64 - 1),
        };
        if let Some(re) = initial_cli_regex {
            // We don't have the original pattern; store the regex string
            let rule = FilterRule { pattern: re.as_str().to_string(), is_regex: true, case_insensitive: true, whole_word: false, whole_line: false, enabled: true, compiled: Some(re), match_count: 0 };
            s.filters.push(rule);
        }
        s
    }

    pub fn set_sources<I: IntoIterator<Item = (String, PathBuf)>>(&mut self, inputs: I) {
        self.sources = inputs.into_iter().map(|(name, path)| Source {
            name,
            path,
            lines: Vec::new(),
            scroll_offset: 0,
            auto_scroll: true,
            selected_log: None,
        }).collect();
        self.focused = 0;
    }

    pub fn current_source(&self) -> Option<&Source> { self.sources.get(self.focused) }
    pub fn current_source_mut(&mut self) -> Option<&mut Source> { self.sources.get_mut(self.focused) }

    pub fn push_line_for(&mut self, source_id: usize, line: String) {
        // Update stats globally first to avoid borrow conflicts
        self.update_buckets_for_now();
        self.classify_and_count(&line);
        if let Some(src) = self.sources.get_mut(source_id) {
            src.lines.push(line);
            if src.auto_scroll { src.scroll_offset = 0; }
        }
    }

    fn classify_and_count(&mut self, line: &str) {
        // Per-filter match counts
        for rule in &mut self.filters {
            if !rule.enabled { continue; }
            rule.ensure_compiled();
            if let Some(re) = &rule.compiled {
                let is_match = if re.as_str().starts_with('^') && re.as_str().ends_with('$') { re.is_match(line) } else { re.find(line).is_some() };
                if is_match { rule.match_count = rule.match_count.saturating_add(1); }
            }
        }
        // Error/Warning classification by simple heuristics (case-insensitive substring)
        let lower = line.to_ascii_lowercase();
        if lower.contains("error") { self.bump_bucket(true); }
        if lower.contains("warn") { self.bump_bucket(false); }
    }

    fn bump_bucket(&mut self, is_error: bool) {
        if is_error {
            if let Some(back) = self.err_buckets.back_mut() { *back = back.saturating_add(1); }
        } else {
            if let Some(back) = self.warn_buckets.back_mut() { *back = back.saturating_add(1); }
        }
    }

    fn update_buckets_for_now(&mut self) {
        let now = current_epoch_sec();
        if now <= self.bucket_epoch_sec { return; }
        // Advance buckets to 'now', pushing zeros
        let mut ts = self.bucket_epoch_sec;
        while ts < now {
            // move window forward by 1 second
            if self.err_buckets.len() == SPARK_WINDOW { self.err_buckets.pop_front(); }
            if self.warn_buckets.len() == SPARK_WINDOW { self.warn_buckets.pop_front(); }
            self.err_buckets.push_back(0);
            self.warn_buckets.push_back(0);
            ts += 1;
        }
        self.bucket_epoch_sec = now;
    }

    pub fn enabled_regexes(&self) -> Vec<regex::Regex> {
        compile_enabled_rules(&self.filters)
    }

    pub fn add_filter_from_input(&mut self) {
        if self.filter_input.is_empty() { return; }
        let mut rule = FilterRule {
            pattern: self.filter_input.clone(),
            is_regex: self.input_is_regex,
            case_insensitive: self.input_case_insensitive,
            whole_word: self.input_whole_word,
            whole_line: self.input_whole_line,
            enabled: true,
            compiled: None,
            match_count: 0,
        };
        rule.ensure_compiled();
        self.filters.push(rule);
        self.filter_input.clear();
    }

    pub fn remove_selected_filter(&mut self) {
        if self.filters.is_empty() { return; }
        if self.selected_filter >= self.filters.len() { self.selected_filter = self.filters.len()-1; }
        self.filters.remove(self.selected_filter);
        if self.selected_filter >= self.filters.len() && !self.filters.is_empty() {
            self.selected_filter = self.filters.len()-1;
        }
    }

    pub fn toggle_selected_filter(&mut self) {
        if let Some(rule) = self.filters.get_mut(self.selected_filter) {
            rule.enabled = !rule.enabled;
        }
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_filter > 0 { self.selected_filter -= 1; }
    }
    pub fn move_selection_down(&mut self) {
        if self.selected_filter + 1 < self.filters.len() { self.selected_filter += 1; }
    }

    pub fn ensure_log_selection(&mut self) {
        if let Some(src) = self.current_source_mut() {
            if src.selected_log.is_none() {
                let end = src.lines.len().saturating_sub(src.scroll_offset);
                let sel = end.saturating_sub(1);
                src.selected_log = if src.lines.is_empty() { None } else { Some(sel) };
            }
        }
    }

    pub fn move_log_selection_up(&mut self) {
        self.ensure_log_selection();
        if let Some(src) = self.current_source_mut() {
            if let Some(idx) = src.selected_log.as_mut() {
                if *idx > 0 { *idx -= 1; }
            }
        }
    }
    pub fn move_log_selection_down(&mut self) {
        self.ensure_log_selection();
        if let Some(src) = self.current_source_mut() {
            if let Some(idx) = src.selected_log.as_mut() {
                let max = src.lines.len().saturating_sub(1);
                if *idx < max { *idx += 1; }
            }
        }
    }

    pub fn scroll_up(&mut self, n: usize) {
        if let Some(src) = self.current_source_mut() {
            src.auto_scroll = false;
            let max_offset = src.lines.len().saturating_sub(1);
            src.scroll_offset = (src.scroll_offset + n).min(max_offset);
        }
    }

    pub fn scroll_down(&mut self, n: usize) {
        if let Some(src) = self.current_source_mut() {
            if src.scroll_offset == 0 { return; }
            src.scroll_offset = src.scroll_offset.saturating_sub(n);
            if src.scroll_offset == 0 {
                src.auto_scroll = true;
            }
        }
    }

    pub fn scroll_top(&mut self) {
        if let Some(src) = self.current_source_mut() {
            src.auto_scroll = false;
            src.scroll_offset = src.lines.len().saturating_sub(1);
        }
    }

    pub fn scroll_bottom(&mut self) {
        if let Some(src) = self.current_source_mut() {
            src.scroll_offset = 0;
            src.auto_scroll = true;
        }
    }

    pub fn toggle_auto_scroll(&mut self) {
        if let Some(src) = self.current_source_mut() {
            if src.auto_scroll {
                src.auto_scroll = false;
            } else {
                src.scroll_offset = 0;
                src.auto_scroll = true;
            }
        }
    }

    pub fn focus_next_source(&mut self) {
        if self.sources.is_empty() { return; }
        self.focused = (self.focused + 1) % self.sources.len();
    }
    pub fn focus_prev_source(&mut self) {
        if self.sources.is_empty() { return; }
        if self.focused == 0 { self.focused = self.sources.len() - 1; } else { self.focused -= 1; }
    }
}

fn current_epoch_sec() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}
