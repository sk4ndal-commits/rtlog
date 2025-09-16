use regex::{Regex, RegexBuilder};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

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

pub fn highlight_line<'a>(text: &'a str, re: &Option<Regex>) -> Line<'a> {
    match re {
        None => Line::from(text.to_string()),
        Some(regex) => {
            let mut spans: Vec<Span> = Vec::new();
            let mut last = 0;
            for m in regex.find_iter(text) {
                if m.start() > last {
                    spans.push(Span::raw(text[last..m.start()].to_string()));
                }
                spans.push(Span::styled(
                    text[m.start()..m.end()].to_string(),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ));
                last = m.end();
            }
            if last < text.len() {
                spans.push(Span::raw(text[last..].to_string()));
            }
            Line::from(spans)
        }
    }
}
