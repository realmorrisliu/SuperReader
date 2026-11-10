use spike_bold_word_starts::{bold_prefix_len, words, LetterRun, BOLD_WORD_STARTS_LABEL};

/// 600 selects the same Iowan Old Style Bold face as 700. The ink pass
/// found no lighter stroke, so this is the lightest weight that matches it.
pub const WORD_START_WEIGHT: u16 = 600;

pub const PROBES: [&str; 15] = [
    "a",
    "be",
    "man",
    "good",
    "truth",
    "single",
    "fortune",
    "entering",
    "character",
    "experience",
    "universally",
    "acknowledged",
    "understanding",
    "characteristic",
    "representations",
];

pub fn menu_label() -> String {
    format!(
        "{} · {}",
        BOLD_WORD_STARTS_LABEL.0, BOLD_WORD_STARTS_LABEL.1
    )
}

pub fn decorate(html: &str, on: bool) -> String {
    if !on {
        return html.to_string();
    }
    let mut painted = insert_style(&paint_html(html));
    if std::env::var("SPIKE_BOLD_PROBES").as_deref() == Ok("1") {
        let block = probe_block();
        if let Some(at) = painted.to_ascii_lowercase().rfind("</body>") {
            painted.insert_str(at, &block);
        } else {
            painted.push_str(&block);
        }
    }
    painted
}

pub fn prefix_of(word: &str) -> String {
    let Some(run) = LetterRun::parse(word) else {
        return String::new();
    };
    let count = bold_prefix_len(run);
    run.as_str().chars().take(count).collect()
}

fn insert_style(html: &str) -> String {
    let style = format!(
        "<style id=\"sr-word-start\">.sr-word-start{{font-weight:{WORD_START_WEIGHT}}}</style>"
    );
    if let Some(at) = html.to_ascii_lowercase().find("</head>") {
        let mut out = String::with_capacity(html.len() + style.len());
        out.push_str(&html[..at]);
        out.push_str(&style);
        out.push_str(&html[at..]);
        out
    } else {
        style + html
    }
}

fn probe_block() -> String {
    let sentence = PROBES.join(" ");
    let painted = paint_text(&sentence);
    format!("<div id=\"sr-bold-probes\" style=\"display:none\">{painted}</div>")
}

fn paint_html(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    let mut blocked = 0u32;
    while !rest.is_empty() {
        if let Some(tag) = split_tag(rest) {
            let name = tag_name(tag);
            if blocks_paint(name) {
                if tag.starts_with("</") {
                    blocked = blocked.saturating_sub(1);
                } else if !tag.ends_with("/>") {
                    blocked += 1;
                }
            }
            out.push_str(tag);
            rest = &rest[tag.len()..];
        } else {
            let end = rest.find('<').unwrap_or(rest.len());
            let text = &rest[..end];
            if blocked == 0 {
                out.push_str(&paint_text(text));
            } else {
                out.push_str(text);
            }
            rest = &rest[end..];
        }
    }
    out
}

fn split_tag(text: &str) -> Option<&str> {
    if !text.starts_with('<') {
        return None;
    }
    let end = text.find('>')?;
    Some(&text[..=end])
}

fn tag_name(tag: &str) -> &str {
    let body = tag.trim_start_matches('<').trim_start_matches('/');
    let end = body
        .find(|ch: char| ch.is_ascii_whitespace() || ch == '>' || ch == '/')
        .unwrap_or(body.len());
    &body[..end]
}

fn blocks_paint(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "b" | "strong" | "script" | "style" | "title"
    )
}

fn paint_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for span in words(text) {
        let run = span.run();
        let count = bold_prefix_len(run);
        let prefix: String = run.as_str().chars().take(count).collect();
        let at = span.at();
        out.push_str(&text[last..at]);
        out.push_str("<span class=\"sr-word-start\">");
        out.push_str(&prefix);
        out.push_str("</span>");
        let run_end = at + run.as_str().len();
        out.push_str(&text[at + prefix.len()..run_end]);
        last = run_end;
    }
    out.push_str(&text[last..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn off_leaves_the_chapter_bytes_unchanged() {
        let html = "<p>It is a truth. 范愛農</p>";
        assert_eq!(decorate(html, false), html);
    }

    #[test]
    fn prefixes_match_the_crate_and_skip_publisher_bold() {
        let html = "<p>good <b>truth</b> <strong>man</strong> 3pm</p><p>范愛農</p>";
        let painted = decorate(html, true);
        assert!(painted.contains("<span class=\"sr-word-start\">go</span>od"));
        assert!(painted.contains("<b>truth</b>"));
        assert!(painted.contains("<strong>man</strong>"));
        assert!(painted.contains("3<span class=\"sr-word-start\">pm</span>"));
        assert!(painted.contains("<p>范愛農</p>"));
        let accented = decorate("<p>café naïve</p>", true);
        assert!(accented.contains("<span class=\"sr-word-start\">ca</span>fé"));
        assert!(accented.contains("<span class=\"sr-word-start\">na</span>ïve"));
        assert!(!painted.contains("b, strong") && !painted.contains("strong{"));
        let probes = PROBES.map(prefix_of);
        assert_eq!(
            probes,
            [
                "a", "be", "ma", "go", "tru", "si", "for", "ente", "char", "exper", "unive",
                "acknow", "unders", "charact", "represe",
            ]
        );
    }
}
