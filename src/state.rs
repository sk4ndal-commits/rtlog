use regex::Regex;

#[derive(Default)]
pub struct AppState {
    // All received log lines
    pub lines: Vec<String>,
    // Current scroll offset from the bottom; 0 means bottom (latest)
    pub scroll_offset: usize,
    // Whether auto-scroll is enabled; when user scrolls, this becomes false
    pub auto_scroll: bool,
    // Optional compiled filter regex for highlight
    pub filter: Option<Regex>,
}

impl AppState {
    pub fn new(filter: Option<Regex>) -> Self {
        Self {
            lines: Vec::new(),
            scroll_offset: 0,
            auto_scroll: true,
            filter,
        }
    }

    pub fn push_line(&mut self, line: String) {
        self.lines.push(line);
        if self.auto_scroll {
            self.scroll_offset = 0;
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
