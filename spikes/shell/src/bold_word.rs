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
        if let Some(tag) = take_tag(rest) {
            let name = tag_name(tag);
            if is_raw(name) && !tag.starts_with("</") && !tag.ends_with("/>") {
                out.push_str(tag);
                rest = &rest[tag.len()..];
                if let Some((_, end)) = find_end_tag(rest, name) {
                    out.push_str(&rest[..end]);
                    rest = &rest[end..];
                } else {
                    out.push_str(rest);
                    break;
                }
                continue;
            }
            if blocks_paint(name) {
                if tag.starts_with("</") {
                    blocked = blocked.saturating_sub(1);
                } else if !tag.ends_with("/>") {
                    blocked += 1;
                }
            }
            out.push_str(tag);
            rest = &rest[tag.len()..];
        } else if rest.starts_with('<') {
            let ch = rest.chars().next().unwrap_or('<');
            if blocked == 0 {
                out.push_str(&escape_char(ch));
            } else {
                out.push(ch);
            }
            rest = &rest[ch.len_utf8()..];
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

fn is_raw(name: &str) -> bool {
    matches!(name.to_ascii_lowercase().as_str(), "script" | "style")
}

fn take_tag(text: &str) -> Option<&str> {
    if !text.starts_with('<') {
        return None;
    }
    if text.starts_with("<!--") {
        let end = text.find("-->")?;
        return Some(&text[..end + 3]);
    }
    let bytes = text.as_bytes();
    let mut index = 1;
    if bytes.get(index) == Some(&b'/') {
        index += 1;
    }
    if !bytes
        .get(index)
        .is_some_and(|byte| byte.is_ascii_alphabetic())
    {
        return None;
    }
    let end = text.find('>')?;
    Some(&text[..=end])
}

fn find_end_tag(text: &str, name: &str) -> Option<(usize, usize)> {
    let want = name.to_ascii_lowercase();
    let lower = text.to_ascii_lowercase();
    let mut from = 0;
    while let Some(rel) = lower[from..].find("</") {
        let at = from + rel;
        let mut index = at + 2;
        if !lower[index..].starts_with(&want) {
            from = at + 2;
            continue;
        }
        index += want.len();
        let bytes = lower.as_bytes();
        while bytes
            .get(index)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            index += 1;
        }
        if bytes.get(index) == Some(&b'>') {
            return Some((at, index + 1));
        }
        from = at + 2;
    }
    None
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
    let parts = text_parts(text);
    let decoded: String = parts.iter().map(|part| part.ch).collect();
    let mut out = String::with_capacity(text.len());
    let mut last = 0usize;
    for span in words(&decoded) {
        let run = span.run();
        let count = bold_prefix_len(run);
        let at = span.at();
        emit_parts(&mut out, &parts, last, at);
        out.push_str("<span class=\"sr-word-start\">");
        let prefix_end = at
            + run
                .as_str()
                .chars()
                .take(count)
                .map(|ch| ch.len_utf8())
                .sum::<usize>();
        emit_parts(&mut out, &parts, at, prefix_end);
        out.push_str("</span>");
        let run_end = at + run.as_str().len();
        emit_parts(&mut out, &parts, prefix_end, run_end);
        last = run_end;
    }
    emit_parts(&mut out, &parts, last, decoded.len());
    out
}

struct TextPart {
    raw: String,
    ch: char,
}

fn text_parts(text: &str) -> Vec<TextPart> {
    let mut parts = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        if let Some((raw, ch, len)) = take_entity(rest) {
            parts.push(TextPart { raw, ch });
            rest = &rest[len..];
            continue;
        }
        let ch = rest.chars().next().unwrap();
        parts.push(TextPart {
            raw: escape_char(ch),
            ch,
        });
        rest = &rest[ch.len_utf8()..];
    }
    parts
}

fn emit_parts(out: &mut String, parts: &[TextPart], from: usize, to: usize) {
    let mut at = 0usize;
    for part in parts {
        let next = at + part.ch.len_utf8();
        if next > from && at < to {
            out.push_str(&part.raw);
        }
        at = next;
        if at >= to {
            break;
        }
    }
}

fn escape_char(ch: char) -> String {
    match ch {
        '&' => "&amp;".to_string(),
        '<' => "&lt;".to_string(),
        '>' => "&gt;".to_string(),
        _ => ch.to_string(),
    }
}

fn take_entity(text: &str) -> Option<(String, char, usize)> {
    if !text.starts_with('&') {
        return None;
    }
    let end = text.find(';')?;
    if end > 32 {
        return None;
    }
    let body = &text[1..end];
    let named = !body.is_empty() && body.chars().all(|ch| ch.is_ascii_alphanumeric());
    let numeric = body.starts_with('#');
    if !named && !numeric {
        return None;
    }
    let ch = if let Some(hex) = body.strip_prefix("#x").or_else(|| body.strip_prefix("#X")) {
        u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
    } else if let Some(dec) = body.strip_prefix('#') {
        dec.parse::<u32>().ok().and_then(char::from_u32)
    } else {
        named_entity(body)
    };
    Some((text[..=end].to_string(), ch.unwrap_or('\u{FFFC}'), end + 1))
}

fn named_entity(name: &str) -> Option<char> {
    Some(match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{00A0}',
        "shy" => '\u{00AD}',
        "Aacute" => 'Á',
        "aacute" => 'á',
        "Acirc" => 'Â',
        "acirc" => 'â',
        "AElig" => 'Æ',
        "aelig" => 'æ',
        "Agrave" => 'À',
        "agrave" => 'à',
        "Aring" => 'Å',
        "aring" => 'å',
        "Atilde" => 'Ã',
        "atilde" => 'ã',
        "Auml" => 'Ä',
        "auml" => 'ä',
        "Ccedil" => 'Ç',
        "ccedil" => 'ç',
        "Eacute" => 'É',
        "eacute" => 'é',
        "Ecirc" => 'Ê',
        "ecirc" => 'ê',
        "Egrave" => 'È',
        "egrave" => 'è',
        "Euml" => 'Ë',
        "euml" => 'ë',
        "Iacute" => 'Í',
        "iacute" => 'í',
        "Icirc" => 'Î',
        "icirc" => 'î',
        "Igrave" => 'Ì',
        "igrave" => 'ì',
        "Iuml" => 'Ï',
        "iuml" => 'ï',
        "Ntilde" => 'Ñ',
        "ntilde" => 'ñ',
        "Oacute" => 'Ó',
        "oacute" => 'ó',
        "Ocirc" => 'Ô',
        "ocirc" => 'ô',
        "OElig" => 'Œ',
        "oelig" => 'œ',
        "Ograve" => 'Ò',
        "ograve" => 'ò',
        "Oslash" => 'Ø',
        "oslash" => 'ø',
        "Otilde" => 'Õ',
        "otilde" => 'õ',
        "Ouml" => 'Ö',
        "ouml" => 'ö',
        "szlig" => 'ß',
        "Uacute" => 'Ú',
        "uacute" => 'ú',
        "Ucirc" => 'Û',
        "ucirc" => 'û',
        "Ugrave" => 'Ù',
        "ugrave" => 'ù',
        "Uuml" => 'Ü',
        "uuml" => 'ü',
        "Yacute" => 'Ý',
        "yacute" => 'ý',
        "yuml" => 'ÿ',
        _ => return None,
    })
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

    #[test]
    fn entities_stay_words_and_a_bare_less_than_returns() {
        let han = decorate("<p>&#x8303;&#x611B;&#x8FB2;</p>", true);
        assert!(han.contains("<p>&#x8303;&#x611B;&#x8FB2;</p>"));
        assert!(!han.contains("sr-word-start\">x"));
        let space = decorate("<p>范愛農&nbsp;後</p>", true);
        assert!(space.contains("<p>范愛農&nbsp;後</p>"));
        let elite = decorate("<p>&#233;lite</p>", true);
        assert!(elite.contains("<span class=\"sr-word-start\">&#233;</span>lite"));
        let cafe = decorate("<p>caf&eacute;</p>", true);
        assert!(cafe.contains("<span class=\"sr-word-start\">ca</span>f&eacute;"));
        let amp = decorate("<p>Tom &amp; Jerry</p>", true);
        assert!(amp.contains("</span>m &amp; <span class=\"sr-word-start\">Je</span>"));
        assert!(!amp.contains(">a</span>mp"));
        let price = decorate("<p>price < 10 and truth</p>", true);
        assert!(price.contains("<span class=\"sr-word-start\">tru</span>th"));
        assert!(price.contains("&lt;"));
        let script = decorate("<script>if (a < b) return;</script><p>good</p>", true);
        assert!(script.contains("<script>if (a < b) return;</script>"));
        assert!(script.contains("<span class=\"sr-word-start\">go</span>od"));
        let nested = decorate("<b>keep <script>var x = \"</b>\";</script> truth</b>", true);
        assert!(!nested.contains("<span class=\"sr-word-start\">"));
    }
}
