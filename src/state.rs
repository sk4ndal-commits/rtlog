use crate::filter::{compile_enabled_rules, FilterRule};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterFocus { #[default] Input, List }

#[derive(Default)]
pub struct AppState {
    // All received log lines
    pub lines: Vec<String>,
    // Current scroll offset from the bottom; 0 means bottom (latest)
    pub scroll_offset: usize,
    // Whether auto-scroll is enabled; when user scrolls, this becomes false
    pub auto_scroll: bool,

    // Filter system
    pub filters: Vec<FilterRule>,
    pub filter_panel_open: bool,
    pub filter_input: String,
    pub input_is_regex: bool,
    pub input_case_insensitive: bool,
    pub input_whole_word: bool,
    pub input_whole_line: bool,
    pub filter_focus: FilterFocus,
    pub selected_filter: usize,

    // Context/details view
    pub context_panel_open: bool,
    pub context_radius: usize,
    // Selected log line (absolute index in lines vec)
    pub selected_log: Option<usize>,
}

impl AppState {
    pub fn new(initial_cli_regex: Option<regex::Regex>) -> Self {
        let mut s = Self {
            lines: Vec::new(),
            scroll_offset: 0,
            auto_scroll: true,
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
            selected_log: None,
        };
        if let Some(re) = initial_cli_regex {
            // We don't have the original pattern; store the regex string
            let rule = FilterRule { pattern: re.as_str().to_string(), is_regex: true, case_insensitive: true, whole_word: false, whole_line: false, enabled: true };
            s.filters.push(rule);
        }
        s
    }

    pub fn push_line(&mut self, line: String) {
        self.lines.push(line);
        if self.auto_scroll {
            self.scroll_offset = 0;
        }
    }

    pub fn enabled_regexes(&self) -> Vec<regex::Regex> {
        compile_enabled_rules(&self.filters)
    }

    pub fn add_filter_from_input(&mut self) {
        if self.filter_input.is_empty() { return; }
        let rule = FilterRule {
            pattern: self.filter_input.clone(),
            is_regex: self.input_is_regex,
            case_insensitive: self.input_case_insensitive,
            whole_word: self.input_whole_word,
            whole_line: self.input_whole_line,
            enabled: true,
        };
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
        if self.selected_log.is_none() {
            let end = self.lines.len().saturating_sub(self.scroll_offset);
            let sel = end.saturating_sub(1);
            self.selected_log = if self.lines.is_empty() { None } else { Some(sel) };
        }
    }

    pub fn move_log_selection_up(&mut self) {
        self.ensure_log_selection();
        if let Some(idx) = self.selected_log.as_mut() {
            if *idx > 0 { *idx -= 1; }
        }
    }
    pub fn move_log_selection_down(&mut self) {
        self.ensure_log_selection();
        if let Some(idx) = self.selected_log.as_mut() {
            let max = self.lines.len().saturating_sub(1);
            if *idx < max { *idx += 1; }
        }
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.auto_scroll = false;
        let max_offset = self.lines.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + n).min(max_offset);
    }

    pub fn scroll_down(&mut self, n: usize) {
        if self.scroll_offset == 0 { return; }
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
        if self.scroll_offset == 0 {
            self.auto_scroll = true;
        }
    }

    pub fn scroll_top(&mut self) {
        self.auto_scroll = false;
        self.scroll_offset = self.lines.len().saturating_sub(1);
    }

    pub fn scroll_bottom(&mut self) {
        self.scroll_offset = 0;
        self.auto_scroll = true;
    }

    pub fn toggle_auto_scroll(&mut self) {
        if self.auto_scroll {
            self.auto_scroll = false;
        } else {
            self.scroll_bottom();
        }
    }
}
