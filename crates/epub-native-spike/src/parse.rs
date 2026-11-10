use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Left,
    Right,
    Center,
    Justify,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Display {
    Block,
    Inline,
    None,
}

#[derive(Clone, Debug)]
pub struct Style {
    pub size: f32,
    pub leading: f32,
    pub indent: f32,
    pub align: Align,
    pub italic: bool,
    pub weight: u16,
    pub small_caps: bool,
    pub oldstyle: bool,
    pub hyphens: bool,
    pub nowrap: bool,
    pub margin_top: f32,
    pub margin_bottom: f32,
    pub display: Display,
}

#[derive(Clone, Debug)]
pub struct Span {
    pub node: u32,
    pub text: String,
    pub style: Style,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub heading: bool,
    pub style: Style,
    pub spans: Vec<Span>,
}

#[derive(Clone, Debug)]
pub struct TextNode {
    pub steps: String,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct Document {
    pub blocks: Vec<Block>,
    pub nodes: Vec<TextNode>,
    pub honored: BTreeSet<String>,
    pub dropped: BTreeSet<String>,
}

pub fn parse_chapter(xhtml: &str, css: &str, cjk: bool) -> Document {
    parse_chapter_opts(xhtml, css, cjk, false)
}

pub fn parse_chapter_opts(xhtml: &str, css: &str, cjk: bool, keep_chrome: bool) -> Document {
    let rules = parse_css(css);
    let mut honored = BTreeSet::new();
    let mut dropped = BTreeSet::new();
    for rule in &rules {
        for (prop, _) in &rule.decls {
            if KNOWN.contains(&prop.as_str()) {
                honored.insert(prop.clone());
            } else {
                dropped.insert(prop.clone());
            }
        }
        if rule.skipped {
            dropped.insert("unsupported-selector".into());
        }
    }
    let tree = parse_html(xhtml);
    let mut nodes = Vec::new();
    let mut blocks = Vec::new();
    let root_style = fallback(cjk);
    walk(
        &tree,
        &root_style,
        &rules,
        &mut Vec::new(),
        &mut nodes,
        &mut blocks,
        true,
        keep_chrome,
    );
    if blocks.is_empty() {
        dropped.insert("no-blocks".into());
    }
    let mut after_heading = false;
    for block in &mut blocks {
        if after_heading && !block.heading {
            block.style.indent = 0.0;
            for span in &mut block.spans {
                span.style.indent = 0.0;
            }
        }
        after_heading = block.heading;
    }
    Document {
        blocks,
        nodes,
        honored,
        dropped,
    }
}

const KNOWN: &[&str] = &[
    "margin",
    "margin-top",
    "margin-bottom",
    "text-indent",
    "text-align",
    "font-size",
    "font-style",
    "font-weight",
    "font-variant",
    "font-variant-numeric",
    "line-height",
    "hyphens",
    "-webkit-hyphens",
    "-moz-hyphens",
    "-epub-hyphens",
    "white-space",
    "display",
];

fn fallback(cjk: bool) -> Style {
    if cjk {
        Style {
            size: 20.0,
            leading: 1.70,
            indent: 40.0,
            align: Align::Justify,
            italic: false,
            weight: 400,
            small_caps: false,
            oldstyle: false,
            hyphens: false,
            nowrap: false,
            margin_top: 0.0,
            margin_bottom: 0.0,
            display: Display::Block,
        }
    } else {
        Style {
            size: 22.0,
            leading: 1.48,
            indent: 44.0,
            align: Align::Justify,
            italic: false,
            weight: 400,
            small_caps: false,
            oldstyle: true,
            hyphens: true,
            nowrap: false,
            margin_top: 0.0,
            margin_bottom: 0.0,
            display: Display::Block,
        }
    }
}

struct El {
    tag: String,
    id: String,
    class: Vec<String>,
    style_attr: String,
    children: Vec<Node>,
}

enum Node {
    El(El),
    Text(String),
}

fn walk(
    node: &Node,
    parent: &Style,
    rules: &[Rule],
    steps: &mut Vec<u32>,
    texts: &mut Vec<TextNode>,
    blocks: &mut Vec<Block>,
    at_flow: bool,
    keep_chrome: bool,
) {
    let Node::El(el) = node else {
        return;
    };
    if el.tag == "script" || el.tag == "style" || el.tag == "head" {
        return;
    }
    let mut style = inherit(parent);
    if matches!(el.tag.as_str(), "em" | "i" | "cite") {
        style.italic = true;
    }
    if matches!(el.tag.as_str(), "b" | "strong") {
        style.weight = 700;
    }
    if matches!(el.tag.as_str(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
        style.align = Align::Center;
        style.small_caps = true;
        style.hyphens = false;
        style.indent = 0.0;
        style.margin_top = style.size * 1.4;
        style.margin_bottom = style.size * 0.8;
        style.display = Display::Block;
    }
    if el.tag == "abbr" {
        style.nowrap = true;
    }
    apply_rules(el, parent, rules, &mut style);
    apply_inline(el, parent, &mut style);
    if style.display == Display::None || (!keep_chrome && skip_chrome(el)) {
        return;
    }
    let blockish = is_block(&el.tag) || (at_flow && style.display == Display::Block && has_direct_text(el));
    if blockish && !matches!(el.tag.as_str(), "div" | "section" | "article" | "body" | "html" | "header" | "footer" | "main")
    {
        let mut spans = Vec::new();
        collect_inline(el, &style, rules, steps, texts, &mut spans);
        if spans.iter().any(|span| !span.text.trim().is_empty()) {
            blocks.push(Block {
                heading: el.tag.starts_with('h'),
                style: style.clone(),
                spans,
            });
        }
        return;
    }
    let mut element_i = 0u32;
    for child in &el.children {
        match child {
            Node::El(_) => {
                let step = (element_i + 1) * 2;
                element_i += 1;
                steps.push(step);
                walk(child, &style, rules, steps, texts, blocks, true, keep_chrome);
                steps.pop();
            }
            Node::Text(text) if at_flow && !text.trim().is_empty() => {
                let id = texts.len() as u32;
                let text_step = text_step_of(el, child);
                texts.push(TextNode {
                    steps: format_steps(steps, text_step),
                    text: text.clone(),
                });
                blocks.push(Block {
                    heading: false,
                    style: style.clone(),
                    spans: vec![Span {
                        node: id,
                        text: text.clone(),
                        style: style.clone(),
                    }],
                });
            }
            Node::Text(_) => {}
        }
    }
}

fn text_step_of(parent: &El, target: &Node) -> u32 {
    let mut i = 0u32;
    for child in &parent.children {
        if std::ptr::eq(child, target) {
            return i * 2 + 1;
        }
        i += 1;
    }
    1
}

fn format_steps(steps: &[u32], text_step: u32) -> String {
    let mut out = String::new();
    for step in steps {
        out.push('/');
        out.push_str(&step.to_string());
    }
    out.push('/');
    out.push_str(&text_step.to_string());
    out
}

fn collect_inline(
    el: &El,
    style: &Style,
    rules: &[Rule],
    steps: &[u32],
    texts: &mut Vec<TextNode>,
    spans: &mut Vec<Span>,
) {
    let mut child_i = 0u32;
    for child in &el.children {
        match child {
            Node::Text(text) => {
                let text_step = child_i * 2 + 1;
                let id = texts.len() as u32;
                texts.push(TextNode {
                    steps: format_steps(steps, text_step),
                    text: text.clone(),
                });
                spans.push(Span {
                    node: id,
                    text: text.clone(),
                    style: style.clone(),
                });
            }
            Node::El(inner) => {
                if inner.tag == "br" {
                    spans.push(Span {
                        node: u32::MAX,
                        text: "\n".into(),
                        style: style.clone(),
                    });
                } else if inner.tag != "script" && inner.tag != "style" {
                    let mut child_style = style.clone();
                    child_style.display = Display::Inline;
                    if matches!(inner.tag.as_str(), "em" | "i" | "cite") {
                        child_style.italic = true;
                    }
                    if matches!(inner.tag.as_str(), "b" | "strong") {
                        child_style.weight = 700;
                    }
                    if inner.tag == "abbr" {
                        child_style.nowrap = true;
                    }
                    apply_rules(inner, style, rules, &mut child_style);
                    apply_inline(inner, style, &mut child_style);
                    if child_style.display != Display::None {
                        collect_inline(inner, &child_style, rules, steps, texts, spans);
                    }
                }
            }
        }
        child_i += 1;
    }
}

fn inherit(parent: &Style) -> Style {
    let mut style = parent.clone();
    style.margin_top = 0.0;
    style.margin_bottom = 0.0;
    style.display = Display::Inline;
    style
}

fn is_block(tag: &str) -> bool {
    matches!(
        tag,
        "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "li" | "blockquote" | "pre" | "figcaption"
    )
}

fn has_direct_text(el: &El) -> bool {
    el.children.iter().any(|child| matches!(child, Node::Text(text) if !text.trim().is_empty()))
}

fn skip_chrome(el: &El) -> bool {
    el.id == "pg-header"
        || el.id == "pg-footer"
        || el.class.iter().any(|name| name == "pg-boilerplate" || name == "pgheader")
}

fn apply_rules(el: &El, parent: &Style, rules: &[Rule], style: &mut Style) {
    let mut best: Vec<(u32, usize, String, String)> = Vec::new();
    for (ord, rule) in rules.iter().enumerate() {
        if rule.skipped {
            continue;
        }
        if selector_matches(&rule.selector, el) {
            for (prop, value) in &rule.decls {
                best.push((rule.specificity, ord, prop.clone(), value.clone()));
            }
        }
    }
    best.sort_by_key(|(spec, ord, prop, _)| (*spec, *ord, prop.clone()));
    let mut seen = BTreeSet::new();
    let mut ordered = Vec::new();
    for item in best.into_iter().rev() {
        if seen.insert(item.2.clone()) {
            ordered.push(item);
        }
    }
    ordered.reverse();
    for (_, _, prop, value) in &ordered {
        if prop == "font-size" {
            style.size = font_size(value, parent.size, style.size);
        }
    }
    for (_, _, prop, value) in &ordered {
        apply_prop(style, prop, value, parent.size);
    }
}

fn apply_inline(el: &El, parent: &Style, style: &mut Style) {
    if el.style_attr.is_empty() {
        return;
    }
    for (prop, value) in parse_decls(&el.style_attr) {
        if prop == "font-size" {
            style.size = font_size(&value, parent.size, style.size);
        }
    }
    for (prop, value) in parse_decls(&el.style_attr) {
        apply_prop(style, &prop, &value, parent.size);
    }
}

fn apply_prop(style: &mut Style, prop: &str, value: &str, parent_size: f32) {
    let value = value.trim();
    match prop {
        "font-size" => {}
        "font-style" => style.italic = value.contains("italic") || value.contains("oblique"),
        "font-weight" => {
            style.weight = if value == "bold" || value == "bolder" {
                700
            } else if value == "normal" {
                400
            } else {
                value.parse().unwrap_or(style.weight)
            };
        }
        "font-variant" => style.small_caps = value.contains("small-caps"),
        "font-variant-numeric" => style.oldstyle = value.contains("oldstyle"),
        "line-height" => {
            if value == "normal" {
                return;
            }
            style.leading = length_mult(value, style.size, parent_size);
        }
        "text-indent" => style.indent = length_px(value, style.size, parent_size),
        "text-align" => {
            style.align = match value {
                "left" | "start" => Align::Left,
                "right" | "end" => Align::Right,
                "center" => Align::Center,
                "justify" => Align::Justify,
                _ => style.align,
            };
        }
        "hyphens" | "-webkit-hyphens" | "-moz-hyphens" | "-epub-hyphens" => {
            style.hyphens = value == "auto";
        }
        "white-space" => style.nowrap = value == "nowrap",
        "display" => {
            style.display = if value == "none" {
                Display::None
            } else if value == "inline" || value == "inline-block" {
                Display::Inline
            } else {
                Display::Block
            };
        }
        "margin-top" => style.margin_top = length_px(value, style.size, parent_size).max(0.0),
        "margin-bottom" => style.margin_bottom = length_px(value, style.size, parent_size).max(0.0),
        "margin" => {
            let parts: Vec<&str> = value.split_whitespace().collect();
            let px = |part: &str| length_px(part, style.size, parent_size).max(0.0);
            match parts.as_slice() {
                [all] => {
                    style.margin_top = px(all);
                    style.margin_bottom = px(all);
                }
                [vert, _] => {
                    style.margin_top = px(vert);
                    style.margin_bottom = px(vert);
                }
                [top, _, bottom] => {
                    style.margin_top = px(top);
                    style.margin_bottom = px(bottom);
                }
                [top, _, bottom, _] => {
                    style.margin_top = px(top);
                    style.margin_bottom = px(bottom);
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn font_size(value: &str, parent: f32, current: f32) -> f32 {
    match value.trim() {
        "smaller" => parent * 0.83,
        "larger" => parent * 1.2,
        other => {
            let px = length_px(other, parent, parent);
            if other.ends_with('%') || other.ends_with("em") || other.ends_with("rem") {
                px
            } else if other.parse::<f32>().is_ok() {
                parent * other.parse::<f32>().unwrap_or(1.0)
            } else {
                if px > 0.0 { px } else { current }
            }
        }
    }
}

fn length_px(value: &str, size: f32, parent: f32) -> f32 {
    let value = value.trim();
    if let Some(num) = value.strip_suffix("px") {
        return num.trim().parse().unwrap_or(0.0);
    }
    if let Some(num) = value.strip_suffix("em") {
        return num.trim().parse::<f32>().unwrap_or(0.0) * size;
    }
    if let Some(num) = value.strip_suffix("rem") {
        return num.trim().parse::<f32>().unwrap_or(0.0) * parent;
    }
    if let Some(num) = value.strip_suffix('%') {
        return num.trim().parse::<f32>().unwrap_or(0.0) / 100.0 * size;
    }
    if let Some(num) = value.strip_suffix("pt") {
        return num.trim().parse::<f32>().unwrap_or(0.0) * 96.0 / 72.0;
    }
    value.parse().unwrap_or(0.0)
}

fn length_mult(value: &str, size: f32, parent: f32) -> f32 {
    let value = value.trim();
    if value.ends_with("px") || value.ends_with("em") || value.ends_with("pt") || value.ends_with('%') {
        let px = length_px(value, size, parent);
        if size > 0.0 { px / size } else { 1.4 }
    } else {
        value.parse().unwrap_or(1.4)
    }
}

struct Rule {
    selector: Selector,
    specificity: u32,
    decls: Vec<(String, String)>,
    skipped: bool,
}

struct Selector {
    compounds: Vec<Compound>,
}

#[derive(Clone)]
struct Compound {
    tag: Option<String>,
    classes: Vec<String>,
    id: Option<String>,
}

fn parse_css(css: &str) -> Vec<Rule> {
    let css = strip_comments(css);
    let mut rules = Vec::new();
    let mut i = 0;
    let chars: Vec<char> = css.chars().collect();
    while i < chars.len() {
        while i < chars.len() && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }
        if chars[i] == '@' {
            i = skip_at(&chars, i);
            continue;
        }
        let start = i;
        while i < chars.len() && chars[i] != '{' {
            i += 1;
        }
        if i >= chars.len() {
            break;
        }
        let prelude: String = chars[start..i].iter().collect();
        i += 1;
        let body_start = i;
        let mut depth = 1;
        while i < chars.len() && depth > 0 {
            if chars[i] == '{' {
                depth += 1;
            } else if chars[i] == '}' {
                depth -= 1;
            }
            i += 1;
        }
        let body: String = chars[body_start..i.saturating_sub(1)].iter().collect();
        let decls = parse_decls(&body);
        for piece in prelude.split(',') {
            let (selector, spec, skipped) = parse_selector(piece.trim());
            if selector.compounds.is_empty() && !skipped {
                continue;
            }
            rules.push(Rule {
                selector,
                specificity: spec,
                decls: decls.clone(),
                skipped,
            });
        }
    }
    rules
}

fn skip_at(chars: &[char], mut i: usize) -> usize {
    while i < chars.len() && chars[i] != '{' && chars[i] != ';' {
        i += 1;
    }
    if i < chars.len() && chars[i] == ';' {
        return i + 1;
    }
    if i >= chars.len() {
        return i;
    }
    i += 1;
    let mut depth = 1;
    while i < chars.len() && depth > 0 {
        if chars[i] == '{' {
            depth += 1;
        } else if chars[i] == '}' {
            depth -= 1;
        }
        i += 1;
    }
    i
}

fn parse_decls(body: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for piece in body.split(';') {
        let Some((prop, value)) = piece.split_once(':') else {
            continue;
        };
        let prop = prop.trim().to_ascii_lowercase();
        if prop.is_empty() {
            continue;
        }
        let value = value.trim().trim_end_matches("!important").trim().to_string();
        out.push((prop, value));
    }
    out
}

fn parse_selector(raw: &str) -> (Selector, u32, bool) {
    if raw.contains('[') || raw.contains(':') || raw.contains('*') || raw.contains('|') {
        return (Selector { compounds: Vec::new() }, 0, true);
    }
    let mut compounds = Vec::new();
    let mut buf = String::new();
    let flush = |buf: &mut String, compounds: &mut Vec<Compound>| {
        let compound = parse_compound(buf);
        buf.clear();
        if compound.tag.is_some() || compound.id.is_some() || !compound.classes.is_empty() {
            compounds.push(compound);
        }
    };
    for c in raw.chars() {
        match c {
            '>' | '+' | '~' => flush(&mut buf, &mut compounds),
            ch if ch.is_whitespace() => {
                if !buf.is_empty() {
                    flush(&mut buf, &mut compounds);
                }
            }
            ch => buf.push(ch),
        }
    }
    flush(&mut buf, &mut compounds);
    let spec = compounds.iter().map(specificity).sum();
    (Selector { compounds }, spec, false)
}

fn parse_compound(raw: &str) -> Compound {
    let mut tag = None;
    let mut classes = Vec::new();
    let mut id = None;
    let mut buf = String::new();
    let mut mode = 't';
    let flush = |mode: char, buf: &mut String, tag: &mut Option<String>, classes: &mut Vec<String>, id: &mut Option<String>| {
        if buf.is_empty() {
            return;
        }
        let value = std::mem::take(buf);
        match mode {
            'c' => classes.push(value),
            'i' => *id = Some(value),
            _ => *tag = Some(value),
        }
    };
    for c in raw.chars() {
        match c {
            '.' => {
                flush(mode, &mut buf, &mut tag, &mut classes, &mut id);
                mode = 'c';
            }
            '#' => {
                flush(mode, &mut buf, &mut tag, &mut classes, &mut id);
                mode = 'i';
            }
            ch => buf.push(ch),
        }
    }
    flush(mode, &mut buf, &mut tag, &mut classes, &mut id);
    Compound { tag, classes, id }
}

fn specificity(compound: &Compound) -> u32 {
    let mut score = 0;
    if compound.tag.is_some() {
        score += 1;
    }
    score += compound.classes.len() as u32 * 10;
    if compound.id.is_some() {
        score += 100;
    }
    score
}

fn selector_matches(selector: &Selector, el: &El) -> bool {
    if selector.compounds.len() != 1 {
        return false;
    }
    compound_matches(&selector.compounds[0], el)
}

fn compound_matches(compound: &Compound, el: &El) -> bool {
    if let Some(tag) = &compound.tag {
        if tag != &el.tag && tag != "*" {
            return false;
        }
    }
    if let Some(id) = &compound.id {
        if id != &el.id {
            return false;
        }
    }
    compound.classes.iter().all(|class| el.class.iter().any(|has| has == class))
}

fn strip_comments(css: &str) -> String {
    let mut out = String::new();
    let mut rest = css;
    while let Some(at) = rest.find("/*") {
        out.push_str(&rest[..at]);
        if let Some(end) = rest[at + 2..].find("*/") {
            rest = &rest[at + 2 + end + 2..];
        } else {
            rest = "";
        }
    }
    out.push_str(rest);
    out
}

fn parse_html(xhtml: &str) -> Node {
    let chars: Vec<char> = xhtml.chars().collect();
    let mut i = 0;
    let mut stack: Vec<El> = Vec::new();
    let mut current = root_placeholder();
    while i < chars.len() {
        if chars[i] == '<' {
            if chars.get(i + 1) == Some(&'!') {
                if chars.get(i..i + 4) == Some(&['<', '!', '-', '-']) {
                    i += 4;
                    while i + 2 < chars.len() && chars[i..i + 3] != ['-', '-', '>'] {
                        i += 1;
                    }
                    i = (i + 3).min(chars.len());
                    continue;
                }
                while i < chars.len() && chars[i] != '>' {
                    i += 1;
                }
                i += 1;
                continue;
            }
            if chars.get(i + 1) == Some(&'/') {
                i += 2;
                let name = read_name(&chars, &mut i);
                while i < chars.len() && chars[i] != '>' {
                    i += 1;
                }
                i += 1;
                close_tag(&mut stack, &mut current, &name);
                continue;
            }
            i += 1;
            let name = read_name(&chars, &mut i).to_ascii_lowercase();
            let (attrs, self_close) = read_attrs(&chars, &mut i);
            if name.is_empty() {
                continue;
            }
            let el = El {
                tag: name.clone(),
                id: attrs.get("id").cloned().unwrap_or_default(),
                class: attrs
                    .get("class")
                    .map(|value| value.split_whitespace().map(|part| part.to_string()).collect())
                    .unwrap_or_default(),
                style_attr: attrs.get("style").cloned().unwrap_or_default(),
                children: Vec::new(),
            };
            if self_close || VOID.contains(&name.as_str()) {
                current.children.push(Node::El(el));
            } else {
                let parent = std::mem::replace(&mut current, el);
                stack.push(parent);
            }
        } else {
            let start = i;
            while i < chars.len() && chars[i] != '<' {
                i += 1;
            }
            let text: String = chars[start..i].iter().collect();
            let text = decode_entities(&text);
            if !text.is_empty() {
                current.children.push(Node::Text(text));
            }
        }
    }
    while let Some(parent) = stack.pop() {
        let child = std::mem::replace(&mut current, parent);
        current.children.push(Node::El(child));
    }
    if current.tag.is_empty() {
        current.tag = "html".into();
    }
    Node::El(current)
}

fn root_placeholder() -> El {
    El {
        tag: String::new(),
        id: String::new(),
        class: Vec::new(),
        style_attr: String::new(),
        children: Vec::new(),
    }
}

fn close_tag(stack: &mut Vec<El>, current: &mut El, name: &str) {
    let name = name.to_ascii_lowercase();
    if current.tag == name {
        if let Some(mut parent) = stack.pop() {
            let child = std::mem::replace(current, root_placeholder());
            parent.children.push(Node::El(child));
            *current = parent;
        }
        return;
    }
    while current.tag != name {
        if let Some(mut parent) = stack.pop() {
            let child = std::mem::replace(current, root_placeholder());
            parent.children.push(Node::El(child));
            *current = parent;
        } else {
            break;
        }
        if current.tag == name {
            if let Some(mut parent) = stack.pop() {
                let child = std::mem::replace(current, root_placeholder());
                parent.children.push(Node::El(child));
                *current = parent;
            }
            break;
        }
    }
}

fn read_name(chars: &[char], i: &mut usize) -> String {
    let start = *i;
    while *i < chars.len() && (chars[*i].is_ascii_alphanumeric() || chars[*i] == ':' || chars[*i] == '-') {
        *i += 1;
    }
    chars[start..*i].iter().collect()
}

fn read_attrs(chars: &[char], i: &mut usize) -> (std::collections::HashMap<String, String>, bool) {
    let mut map = std::collections::HashMap::new();
    let mut self_close = false;
    while *i < chars.len() && chars[*i] != '>' {
        if chars[*i] == '/' {
            self_close = true;
            *i += 1;
            continue;
        }
        if chars[*i].is_whitespace() {
            *i += 1;
            continue;
        }
        let name = read_name(chars, i).to_ascii_lowercase();
        while *i < chars.len() && chars[*i].is_whitespace() {
            *i += 1;
        }
        if *i < chars.len() && chars[*i] == '=' {
            *i += 1;
            while *i < chars.len() && chars[*i].is_whitespace() {
                *i += 1;
            }
            let value = if *i < chars.len() && (chars[*i] == '"' || chars[*i] == '\'') {
                let quote = chars[*i];
                *i += 1;
                let start = *i;
                while *i < chars.len() && chars[*i] != quote {
                    *i += 1;
                }
                let value: String = chars[start..*i].iter().collect();
                if *i < chars.len() {
                    *i += 1;
                }
                value
            } else {
                let start = *i;
                while *i < chars.len() && !chars[*i].is_whitespace() && chars[*i] != '>' && chars[*i] != '/' {
                    *i += 1;
                }
                chars[start..*i].iter().collect()
            };
            if !name.is_empty() {
                map.insert(name, decode_entities(&value));
            }
        } else if !name.is_empty() {
            map.insert(name, String::new());
        } else {
            *i += 1;
        }
    }
    if *i < chars.len() && chars[*i] == '>' {
        *i += 1;
    }
    (map, self_close)
}

const VOID: &[&str] = &["br", "img", "hr", "meta", "link", "col", "wbr"];

fn decode_entities(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        out.push_str(&rest[..at]);
        rest = &rest[at + 1..];
        let Some(end) = rest.find(';') else {
            out.push('&');
            break;
        };
        let name = &rest[..end];
        rest = &rest[end + 1..];
        if let Some(ch) = entity(name) {
            out.push(ch);
        } else {
            out.push('&');
            out.push_str(name);
            out.push(';');
        }
    }
    out.push_str(rest);
    out
}

fn entity(name: &str) -> Option<char> {
    if let Some(hex) = name.strip_prefix("#x").or_else(|| name.strip_prefix("#X")) {
        return char::from_u32(u32::from_str_radix(hex, 16).ok()?);
    }
    if let Some(dec) = name.strip_prefix('#') {
        return char::from_u32(dec.parse().ok()?);
    }
    Some(match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{00a0}',
        "mdash" => '\u{2014}',
        "ndash" => '\u{2013}',
        "rsquo" => '\u{2019}',
        "lsquo" => '\u{2018}',
        "rdquo" => '\u{201d}',
        "ldquo" => '\u{201c}',
        "hellip" => '\u{2026}',
        "thinsp" => '\u{2009}',
        _ => return None,
    })
}
