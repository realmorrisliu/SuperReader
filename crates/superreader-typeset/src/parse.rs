//! A small CSS and XHTML reader for the chapter subset this crate lays out.
//! Combinators it does not understand are skipped, so a rule cannot match every paragraph by accident.

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Align {
    Justify,
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug)]
pub struct Paint {
    pub size: f32,
    pub leading: f32,
    pub indent: f32,
    pub align: Align,
    pub italic: bool,
    pub weight: u16,
    pub small_caps: bool,
    pub hyphens: bool,
    pub hanging: bool,
    pub margin_top: f32,
    pub margin_bottom: f32,
    size_set: bool,
    leading_set: bool,
    indent_set: bool,
    align_set: bool,
    hyphens_set: bool,
    hanging_set: bool,
}

#[derive(Clone, Debug)]
pub struct Piece {
    pub node: u32,
    pub text: String,
    pub paint: Paint,
}

#[derive(Clone, Debug)]
pub enum Chunk {
    Text(Piece),
    Break,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub paint: Paint,
    pub chunks: Vec<Chunk>,
}

pub fn parse_chapter(xhtml: &str, stylesheet: &str, cjk_book: bool, font_size: Option<f32>) -> Vec<Block> {
    let rules = parse_css(stylesheet);
    let mut style_buf = String::new();
    harvest_style(xhtml, &mut style_buf);
    let mut rules = rules;
    rules.extend(parse_css(&style_buf));
    let fallback = fallback_paint(cjk_book, font_size);
    let mut parser = Parser {
        rules: &rules,
        fallback,
        node: 1,
        blocks: Vec::new(),
        stack: Vec::new(),
        chunks: Vec::new(),
        skip: 0,
    };
    parser.open_tag("body", "");
    parser.feed(xhtml);
    while parser.stack.len() > 1 {
        parser.close_top();
    }
    parser.finish_chunks();
    parser.blocks
}

pub fn cjk_book(xhtml: &str) -> bool {
    xhtml.chars().any(is_han_script)
}

fn fallback_paint(cjk: bool, font_size: Option<f32>) -> Paint {
    let size = font_size.unwrap_or(if cjk { 20.0 } else { 22.0 });
    let mult = if cjk { 1.70 } else { 1.48 };
    Paint {
        size,
        leading: size * mult,
        indent: size * 2.0,
        align: Align::Justify,
        italic: false,
        weight: 400,
        small_caps: false,
        hyphens: !cjk,
        hanging: true,
        margin_top: 0.0,
        margin_bottom: 0.0,
        size_set: false,
        leading_set: false,
        indent_set: false,
        align_set: false,
        hyphens_set: false,
        hanging_set: false,
    }
}

fn is_han_script(ch: char) -> bool {
    matches!(ch,
        '\u{3040}'..='\u{30FF}'
            | '\u{3400}'..='\u{9FFF}'
            | '\u{AC00}'..='\u{D7AF}'
            | '\u{F900}'..='\u{FAFF}'
    )
}

struct Rule {
    selectors: Vec<Selector>,
    decls: Vec<(String, String)>,
}

struct Selector {
    parts: Vec<Simple>,
}

struct Simple {
    comb: Comb,
    tag: Option<String>,
    classes: Vec<String>,
    id: Option<String>,
}

#[derive(Clone, Copy)]
enum Comb {
    Start,
    Descendant,
    Child,
    Adjacent,
}

fn parse_css(css: &str) -> Vec<Rule> {
    let mut rules = Vec::new();
    let bytes = css.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            continue;
        }
        if i < bytes.len() && bytes[i] == b'@' {
            while i < bytes.len() && bytes[i] != b';' && bytes[i] != b'{' {
                i += 1;
            }
            if i < bytes.len() && bytes[i] == b'{' {
                let mut depth = 1;
                i += 1;
                while i < bytes.len() && depth > 0 {
                    if bytes[i] == b'{' {
                        depth += 1;
                    } else if bytes[i] == b'}' {
                        depth -= 1;
                    }
                    i += 1;
                }
            } else if i < bytes.len() {
                i += 1;
            }
            continue;
        }
        let start = i;
        while i < bytes.len() && bytes[i] != b'{' {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let selector_text = css[start..i].trim();
        i += 1;
        let body_start = i;
        while i < bytes.len() && bytes[i] != b'}' {
            i += 1;
        }
        let body = if i <= css.len() { &css[body_start..i] } else { "" };
        if i < bytes.len() {
            i += 1;
        }
        let decls = parse_decls(body);
        if decls.is_empty() {
            continue;
        }
        let mut selectors = Vec::new();
        for part in split_selectors(selector_text) {
            if let Some(sel) = parse_selector(part.trim()) {
                selectors.push(sel);
            }
        }
        if !selectors.is_empty() {
            rules.push(Rule { selectors, decls });
        }
    }
    rules
}

fn split_selectors(text: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut depth = 0;
    for (i, ch) in text.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                out.push(&text[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&text[start..]);
    out
}

fn parse_selector(text: &str) -> Option<Selector> {
    if text.is_empty() || text.contains('[') || text.contains(':') || text.contains('~') {
        return None;
    }
    let mut parts = Vec::new();
    let mut rest = text.trim();
    let mut comb = Comb::Start;
    while !rest.is_empty() {
        rest = rest.trim_start();
        if rest.is_empty() {
            break;
        }
        if !parts.is_empty() {
            if let Some(stripped) = rest.strip_prefix('>') {
                comb = Comb::Child;
                rest = stripped.trim_start();
            } else if let Some(stripped) = rest.strip_prefix('+') {
                comb = Comb::Adjacent;
                rest = stripped.trim_start();
            } else {
                comb = Comb::Descendant;
            }
        }
        let end = rest.find([' ', '>', '+']).unwrap_or(rest.len());
        let token = &rest[..end];
        if token.is_empty() {
            return None;
        }
        let mut tag = None;
        let mut classes = Vec::new();
        let mut id = None;
        let mut buf = String::new();
        let mut mode = 't';
        for ch in token.chars().chain(std::iter::once('\0')) {
            if ch == '.' || ch == '#' || ch == '\0' {
                if !buf.is_empty() {
                    match mode {
                        't' => tag = Some(buf.to_ascii_lowercase()),
                        'c' => classes.push(buf.clone()),
                        'i' => id = Some(buf.clone()),
                        _ => {}
                    }
                    buf.clear();
                }
                mode = match ch {
                    '.' => 'c',
                    '#' => 'i',
                    _ => mode,
                };
            } else {
                buf.push(ch);
            }
        }
        parts.push(Simple { comb, tag, classes, id });
        rest = &rest[end..];
        comb = Comb::Descendant;
    }
    if parts.is_empty() { None } else { Some(Selector { parts }) }
}

fn parse_decls(body: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for piece in body.split(';') {
        let Some((name, value)) = piece.split_once(':') else { continue };
        let name = name.trim().to_ascii_lowercase();
        let value = value.trim().trim_matches('"').to_string();
        if name.is_empty() || value.is_empty() {
            continue;
        }
        if name.starts_with('-') || name == "adobe-hyphenate" {
            continue;
        }
        out.push((name, value));
    }
    out
}

#[derive(Clone)]
struct Frame {
    tag: String,
    class: String,
    id: String,
    paint: Paint,
    prev_tag: String,
    container: bool,
    inline: bool,
}

struct Parser<'a> {
    rules: &'a [Rule],
    fallback: Paint,
    node: u32,
    blocks: Vec<Block>,
    stack: Vec<Frame>,
    chunks: Vec<Chunk>,
    skip: u32,
}

impl<'a> Parser<'a> {
    fn feed(&mut self, xhtml: &str) {
        let bytes = xhtml.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'<' {
                if xhtml[i..].starts_with("<!--") {
                    if let Some(end) = xhtml[i..].find("-->") {
                        i += end + 3;
                        continue;
                    }
                }
                if xhtml[i..].starts_with("<!") {
                    if let Some(end) = xhtml[i..].find('>') {
                        i += end + 1;
                        continue;
                    }
                }
                let close = xhtml[i + 1..].starts_with('/');
                let tag_start = i + 1 + usize::from(close);
                let mut j = tag_start;
                while j < bytes.len() && bytes[j] != b'>' {
                    j += 1;
                }
                if j >= bytes.len() {
                    break;
                }
                let raw = &xhtml[tag_start..j];
                let self_close = raw.ends_with('/');
                let raw = raw.trim_end_matches('/').trim();
                i = j + 1;
                let mut name_end = 0;
                for (k, ch) in raw.char_indices() {
                    if ch.is_whitespace() {
                        name_end = k;
                        break;
                    }
                    name_end = k + ch.len_utf8();
                }
                let tag = raw[..name_end].to_ascii_lowercase();
                if tag.is_empty() {
                    continue;
                }
                if close {
                    self.close_tag(&tag);
                    continue;
                }
                let attrs = &raw[name_end..];
                if tag == "style" {
                    if let Some(end) = xhtml[i..].to_ascii_lowercase().find("</style>") {
                        i += end + "</style>".len();
                    }
                    continue;
                }
                if tag == "br" || tag == "hr" {
                    if self.skip == 0 {
                        self.chunks.push(Chunk::Break);
                    }
                    continue;
                }
                self.open_tag(&tag, attrs);
                if self_close || is_void(&tag) {
                    self.close_tag(&tag);
                }
            } else {
                let start = i;
                while i < bytes.len() && bytes[i] != b'<' {
                    i += 1;
                }
                if self.skip == 0 {
                    let text = decode_entities(&xhtml[start..i]);
                    if !text.is_empty() && !self.stack.is_empty() {
                        let paint = self.stack.last().unwrap().paint.clone();
                        self.chunks.push(Chunk::Text(Piece {
                            node: self.node,
                            text,
                            paint,
                        }));
                        self.node += 1;
                    }
                }
            }
        }
    }

    fn open_tag(&mut self, tag: &str, attrs: &str) {
        let class = attr(attrs, "class").unwrap_or_default();
        let id = attr(attrs, "id").unwrap_or_default();
        if self.skip > 0
            || class.split_whitespace().any(|c| c.starts_with("pg-boilerplate") || c == "pgheader")
            || id == "pg-header"
            || id == "pg-footer"
            || tag == "script"
            || tag == "head"
        {
            self.skip += 1;
            self.stack.push(Frame {
                tag: tag.to_string(),
                class,
                id,
                paint: self.fallback.clone(),
                prev_tag: String::new(),
                container: true,
                inline: false,
            });
            return;
        }
        let parent = self.stack.last().cloned();
        let mut paint = parent.as_ref().map(|p| p.paint.clone()).unwrap_or_else(|| self.fallback.clone());
        paint.margin_top = 0.0;
        paint.margin_bottom = 0.0;
        let prev = parent.as_ref().map(|p| p.prev_tag.clone()).unwrap_or_default();
        self.apply(&mut paint, tag, &class, &id, &prev, attrs);
        let inline = is_inline(tag);
        let container = inline || !is_block(tag);
        if !container {
            self.finish_chunks();
        }
        self.stack.push(Frame {
            tag: tag.to_string(),
            class,
            id,
            paint,
            prev_tag: String::new(),
            container,
            inline,
        });
    }

    fn close_tag(&mut self, tag: &str) {
        if self.stack.len() <= 1 {
            return;
        }
        let Some(pos) = self.stack.iter().rposition(|frame| frame.tag == tag) else {
            return;
        };
        while self.stack.len() > pos + 1 {
            self.close_top();
        }
        self.close_top();
    }

    fn close_top(&mut self) {
        let Some(frame) = self.stack.pop() else { return };
        if self.skip > 0 {
            self.skip -= 1;
            return;
        }
        if frame.inline {
            return;
        }
        if !frame.container {
            let chunks = std::mem::take(&mut self.chunks);
            if chunks.iter().any(|chunk| matches!(chunk, Chunk::Text(p) if p.text.chars().any(|c| !c.is_whitespace()))) {
                self.blocks.push(Block {
                    paint: frame.paint.clone(),
                    chunks,
                });
            }
        } else {
            self.finish_chunks();
        }
        if let Some(parent) = self.stack.last_mut() {
            parent.prev_tag = frame.tag;
        }
    }

    fn finish_chunks(&mut self) {
        if self.chunks.is_empty() {
            return;
        }
        let paint = self.stack.last().map(|f| f.paint.clone()).unwrap_or_else(|| self.fallback.clone());
        let chunks = std::mem::take(&mut self.chunks);
        if chunks.iter().any(|chunk| matches!(chunk, Chunk::Text(p) if p.text.chars().any(|c| !c.is_whitespace()))) {
            self.blocks.push(Block { paint, chunks });
        }
    }

    fn apply(&self, paint: &mut Paint, tag: &str, class: &str, id: &str, prev: &str, attrs: &str) {
        let chain = ancestor_chain(&self.stack, tag, class, id, prev);
        let mut decls = Vec::new();
        for rule in self.rules {
            for sel in &rule.selectors {
                if selector_matches(sel, &chain) {
                    decls.extend(rule.decls.iter().cloned());
                }
            }
        }
        if let Some(style) = attr(attrs, "style") {
            decls.extend(parse_decls(&style));
        }
        for (name, value) in decls.iter().filter(|(name, _)| name == "font-size") {
            apply_decl(paint, name, value);
        }
        for (name, value) in decls.iter().filter(|(name, _)| name != "font-size") {
            apply_decl(paint, name, value);
        }
        if matches!(tag, "em" | "i") {
            paint.italic = true;
        }
        if matches!(tag, "b" | "strong") {
            paint.weight = 700;
        }
    }
}

struct Anc {
    tag: String,
    class: String,
    id: String,
    prev: String,
}

fn ancestor_chain(stack: &[Frame], tag: &str, class: &str, id: &str, prev: &str) -> Vec<Anc> {
    let mut chain: Vec<Anc> = stack
        .iter()
        .map(|frame| Anc {
            tag: frame.tag.clone(),
            class: frame.class.clone(),
            id: frame.id.clone(),
            prev: frame.prev_tag.clone(),
        })
        .collect();
    chain.push(Anc {
        tag: tag.to_string(),
        class: class.to_string(),
        id: id.to_string(),
        prev: prev.to_string(),
    });
    chain
}

fn selector_matches(sel: &Selector, chain: &[Anc]) -> bool {
    if chain.is_empty() || sel.parts.is_empty() {
        return false;
    }
    let mut index = chain.len() - 1;
    let mut part_i = sel.parts.len() - 1;
    if !simple_matches(&sel.parts[part_i], &chain[index]) {
        return false;
    }
    while part_i > 0 {
        let relation = sel.parts[part_i].comb;
        part_i -= 1;
        let part = &sel.parts[part_i];
        match relation {
            Comb::Child => {
                if index == 0 || !simple_matches(part, &chain[index - 1]) {
                    return false;
                }
                index -= 1;
            }
            Comb::Adjacent => {
                if part.tag.as_deref() != Some(chain[index].prev.as_str()) {
                    return false;
                }
            }
            Comb::Descendant => {
                let mut found = false;
                while index > 0 {
                    index -= 1;
                    if simple_matches(part, &chain[index]) {
                        found = true;
                        break;
                    }
                }
                if !found {
                    return false;
                }
            }
            Comb::Start => return false,
        }
    }
    true
}

fn simple_matches(part: &Simple, anc: &Anc) -> bool {
    if let Some(tag) = &part.tag {
        if tag != &anc.tag {
            return false;
        }
    }
    if let Some(id) = &part.id {
        if id != &anc.id {
            return false;
        }
    }
    for class in &part.classes {
        if !anc.class.split_whitespace().any(|item| item == class) {
            return false;
        }
    }
    true
}

fn apply_decl(paint: &mut Paint, name: &str, value: &str) {
    let value = value.split('!').next().unwrap_or(value).trim();
    match name {
        "font-size" => {
            if let Some(size) = length_px(value, paint.size, paint.size) {
                paint.leading = paint.leading / paint.size * size;
                if !paint.indent_set {
                    paint.indent = paint.indent / paint.size * size;
                }
                paint.size = size;
                paint.size_set = true;
            }
        }
        "line-height" => {
            if value == "normal" {
                return;
            }
            if let Ok(num) = value.parse::<f32>() {
                paint.leading = paint.size * num;
                paint.leading_set = true;
            } else if let Some(px) = length_px(value, paint.size, paint.size) {
                paint.leading = px;
                paint.leading_set = true;
            }
        }
        "text-indent" => {
            if let Some(px) = length_px(value, paint.size, paint.size) {
                paint.indent = px.max(0.0);
                paint.indent_set = true;
            }
        }
        "text-align" => {
            paint.align = match value {
                "left" | "start" => Align::Left,
                "center" => Align::Center,
                "right" | "end" => Align::Right,
                "justify" => Align::Justify,
                _ => paint.align,
            };
            paint.align_set = true;
        }
        "font-style" => paint.italic = value == "italic" || value == "oblique",
        "font-weight" => {
            paint.weight = if value == "bold" || value.parse::<u16>().unwrap_or(0) >= 600 { 700 } else { 400 };
        }
        "font-variant" | "font-variant-caps" => {
            paint.small_caps = value.contains("small-caps");
        }
        "hyphens" => {
            paint.hyphens = value != "none" && value != "manual";
            paint.hyphens_set = true;
        }
        "hanging-punctuation" => {
            paint.hanging = value != "none" && !value.is_empty();
            paint.hanging_set = true;
        }
        "margin-top" => {
            if let Some(px) = length_px(value, paint.size, paint.size) {
                paint.margin_top = px;
            }
        }
        "margin-bottom" => {
            if let Some(px) = length_px(value, paint.size, paint.size) {
                paint.margin_bottom = px;
            }
        }
        "margin" => {
            let parts: Vec<&str> = value.split_whitespace().collect();
            if let Some(first) = parts.first() {
                if let Some(px) = length_px(first, paint.size, paint.size) {
                    paint.margin_top = px;
                    paint.margin_bottom = px;
                }
            }
        }
        "display" => {}
        _ => {}
    }
}

fn length_px(value: &str, em: f32, parent: f32) -> Option<f32> {
    let value = value.trim();
    if value == "0" {
        return Some(0.0);
    }
    if let Some(num) = value.strip_suffix("px") {
        return num.trim().parse().ok();
    }
    if let Some(num) = value.strip_suffix("em") {
        return num.trim().parse::<f32>().ok().map(|n| n * em);
    }
    if let Some(num) = value.strip_suffix('%') {
        return num.trim().parse::<f32>().ok().map(|n| n / 100.0 * parent);
    }
    None
}

fn attr(attrs: &str, name: &str) -> Option<String> {
    let lower = attrs.to_ascii_lowercase();
    let needle = format!("{name}=");
    let Some(pos) = lower.find(&needle) else { return None };
    let rest = attrs[pos + needle.len()..].trim_start();
    let quote = rest.chars().next()?;
    if quote == '"' || quote == '\'' {
        let body = &rest[1..];
        let end = body.find(quote)?;
        Some(body[..end].to_string())
    } else {
        let end = rest.find(char::is_whitespace).unwrap_or(rest.len());
        Some(rest[..end].to_string())
    }
}

fn harvest_style(xhtml: &str, out: &mut String) {
    let lower = xhtml.to_ascii_lowercase();
    let mut from = 0;
    while let Some(rel) = lower[from..].find("<style") {
        let start = from + rel;
        let Some(open) = xhtml[start..].find('>') else { break };
        let body = start + open + 1;
        let Some(close) = lower[body..].find("</style>") else { break };
        out.push_str(&xhtml[body..body + close]);
        out.push('\n');
        from = body + close + 8;
    }
}

fn decode_entities(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(pos) = rest.find('&') {
        out.push_str(&rest[..pos]);
        rest = &rest[pos + 1..];
        let Some(end) = rest.find(';') else {
            out.push('&');
            break;
        };
        let name = &rest[..end];
        rest = &rest[end + 1..];
        if let Some(ch) = match name {
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            "nbsp" => Some('\u{00A0}'),
            _ if name.starts_with("#x") => char::from_u32(u32::from_str_radix(&name[2..], 16).unwrap_or(0)),
            _ if name.starts_with('#') => name[1..].parse::<u32>().ok().and_then(char::from_u32),
            _ => None,
        } {
            out.push(ch);
        }
    }
    out.push_str(rest);
    out
}

fn is_void(tag: &str) -> bool {
    matches!(tag, "br" | "hr" | "img" | "meta" | "link" | "wbr")
}

fn is_block(tag: &str) -> bool {
    matches!(
        tag,
        "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "blockquote" | "li" | "figcaption" | "pre"
    )
}

fn is_inline(tag: &str) -> bool {
    matches!(
        tag,
        "a" | "abbr" | "b" | "bdi" | "bdo" | "cite" | "code" | "data" | "dfn" | "em" | "i" | "kbd"
            | "mark" | "q" | "rp" | "rt" | "ruby" | "s" | "samp" | "small" | "span" | "strong"
            | "sub" | "sup" | "time" | "u" | "var"
    )
}
