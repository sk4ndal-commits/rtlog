//! Filtering and highlighting utilities.
//! 
//! Provides `FilterRule` for user-defined patterns, helpers to compile patterns into regexes,
//! and functions to filter and highlight lines in the UI. This module is pure and stateless
//! aside from per-rule compiled regex caches, making it easy to test.

use regex::{Regex, RegexBuilder};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// Build a single regex from CLI pattern for backward compatibility
pub fn build_filter(pattern: Option<&str>) -> anyhow::Result<Option<Regex>> {
    if let Some(p) = pattern {
        if p.is_empty() { return Ok(None); }
        let re = RegexBuilder::new(p)
            .case_insensitive(true)
            .build()?;
        Ok(Some(re))
    } else {
        Ok(None)
    }
}

#[derive(Debug, Clone)]
pub struct FilterRule {
    pub pattern: String,
    pub is_regex: bool,
    pub case_insensitive: bool,
    pub whole_word: bool,
    pub whole_line: bool,
    pub enabled: bool,
    // Runtime-only fields for performance and stats
    pub compiled: Option<Regex>,
    pub match_count: usize,
}

impl FilterRule {
    /// Compile this rule into a Regex according to flags
    pub fn compile(&self) -> anyhow::Result<Regex> {
        let mut pat = if self.is_regex {
            self.pattern.clone()
        } else {
            regex::escape(&self.pattern)
        };
        if self.whole_line {
            pat = format!("^{}$", pat);
        } else if self.whole_word {
            // Use word boundary \b
            pat = format!("\\b{}\\b", pat);
        }
        let mut builder = RegexBuilder::new(&pat);
        builder.case_insensitive(self.case_insensitive);
        let re = builder.build()?;
        Ok(re)
    }

    /// Ensure the compiled regex is available in `compiled`
    pub fn ensure_compiled(&mut self) {
        if self.compiled.is_none() {
            if let Ok(re) = self.compile() {
                self.compiled = Some(re);
            }
        }
    }
}

/// Compile all enabled rules into regexes
pub fn compile_enabled_rules(rules: &[FilterRule]) -> Vec<Regex> {
    let mut out = Vec::new();
    for r in rules.iter().filter(|r| r.enabled) {
        if let Ok(re) = r.compile() {
            out.push(re);
        }
    }
    out
}

/// Return true if text matches any of the enabled regexes; if no regexes, allow all
pub fn line_matches(text: &str, enabled: &[Regex]) -> bool {
    if enabled.is_empty() { return true; }
    enabled.iter().any(|re| {
        if re.as_str().starts_with('^') && re.as_str().ends_with('$') {
            re.is_match(text)
        } else {
            re.find(text).is_some()
        }
    })
}

pub fn highlight_line<'a>(text: &'a str, enabled: &[Regex]) -> Line<'a> {
    if enabled.is_empty() {
        return Line::from(text.to_string());
    }
    // Highlight all matches of all enabled regexes by merging spans.
    // Simple approach: build a vector of (start,end) ranges from all regexes and merge overlaps.
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for re in enabled {
        for m in re.find_iter(text) {
            ranges.push((m.start(), m.end()));
        }
    }
    if ranges.is_empty() {
        return Line::from(text.to_string());
    }
    ranges.sort_by_key(|r| r.0);
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (s, e) in ranges {
        if let Some(last) = merged.last_mut() {
            if s <= last.1 { // overlap or adjacent
                if e > last.1 { last.1 = e; }
                continue;
            }
        }
        merged.push((s, e));
    }

    let mut spans: Vec<Span> = Vec::new();
    let mut last = 0;
    for (s, e) in merged {
        if s > last {
            spans.push(Span::raw(text[last..s].to_string()));
        }
        spans.push(Span::styled(
            text[s..e].to_string(),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
        last = e;
    }
    if last < text.len() {
        spans.push(Span::raw(text[last..].to_string()));
    }
    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_matches_any() {
        let r1 = FilterRule { pattern: "ERROR".into(), is_regex: false, case_insensitive: true, whole_word: false, whole_line: false, enabled: true, compiled: None, match_count: 0 };
        let r2 = FilterRule { pattern: "WARN".into(), is_regex: false, case_insensitive: false, whole_word: false, whole_line: false, enabled: true, compiled: None, match_count: 0 };
        let enabled = compile_enabled_rules(&[r1, r2]);
        assert!(line_matches("2025 ERROR something", &enabled));
        assert!(line_matches("2025 WARN something", &enabled));
        assert!(!line_matches("2025 info ok", &enabled));
    }
}
