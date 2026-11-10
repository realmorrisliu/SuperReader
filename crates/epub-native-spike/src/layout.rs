use std::time::Instant;

use crate::breaker::{self, Atom, BrokenLine};
use crate::face::{ink, paper, Faces, ShapeOpts};
use crate::hyphen::Hyphenator;
use crate::parse::{Align, Block, Document, TextNode};
use tiny_skia::{Color, Pixmap};

pub const PAGE_W: u32 = 534;
pub const PAGE_H: u32 = 819;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Punctuation {
    Full,
    Jlreq,
    Hang,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hyphenation {
    Off,
    Moderate,
    Tex,
}

#[derive(Clone, Copy, Debug)]
pub struct Choices {
    pub punctuation: Punctuation,
    pub cjk_latin_em: f32,
    pub hyphenation: Hyphenation,
}

impl Choices {
    pub fn product() -> Self {
        Self {
            punctuation: Punctuation::Jlreq,
            cjk_latin_em: 0.25,
            hyphenation: Hyphenation::Moderate,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Caret {
    pub node: u32,
    pub offset: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Debug)]
struct Glyph {
    face: u8,
    gid: u32,
    dx: f32,
    w: f32,
    size: f32,
    node: u32,
    off: u32,
    class: Class,
    hang: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Class {
    Latin,
    Cjk,
    Open,
    Close,
    Other,
}

#[derive(Clone, Debug)]
struct Line {
    y: f32,
    h: f32,
    baseline: f32,
    glyphs: Vec<Glyph>,
    gap: Option<f32>,
    hyphen: bool,
    overflow: bool,
    kinsoku: bool,
}

#[derive(Clone, Debug)]
struct Page {
    lines: Vec<Line>,
}

pub struct LaidOut {
    pages: Vec<Page>,
    nodes: Vec<TextNode>,
    pub honored: Vec<String>,
    pub dropped: Vec<String>,
    spine_step: u32,
    pub items_ms: f64,
    pub break_ms: f64,
}

pub struct Stats {
    pub pages: usize,
    pub lines: usize,
    pub hyphenated_lines: usize,
    pub gap_p95: f32,
    pub gap_max: f32,
    pub overflow_lines: usize,
    pub kinsoku_violations: usize,
    pub missing_glyphs: usize,
}

impl LaidOut {
    pub fn lay(faces: &Faces, hyphen: &Hyphenator, doc: &Document, choices: Choices, spine_step: u32, cjk: bool) -> Self {
        let frame = frame_for(cjk);
        let mut items_ms = 0.0;
        let mut break_ms = 0.0;
        let mut pages = vec![Page { lines: Vec::new() }];
        let mut y = frame.top;
        let mut prev_bottom = 0.0f32;
        for block in &doc.blocks {
            let (lines, item_ms, broke_ms) = compose(faces, hyphen, block, &frame, choices);
            items_ms += item_ms;
            break_ms += broke_ms;
            if lines.is_empty() {
                continue;
            }
            let gap = (block.style.margin_top - prev_bottom).max(0.0);
            if y > frame.top && y + gap + lines[0].h > frame.bottom {
                pages.push(Page { lines: Vec::new() });
                y = frame.top;
            } else {
                y += gap;
            }
            if block.heading && y + lines.iter().map(|line| line.h).sum::<f32>() + 24.0 > frame.bottom && y > frame.top + 1.0 {
                pages.push(Page { lines: Vec::new() });
                y = frame.top;
            }
            for mut line in lines {
                if y + line.h > frame.bottom + 0.5 && y > frame.top + 1.0 {
                    pages.push(Page { lines: Vec::new() });
                    y = frame.top;
                }
                line.y = y;
                line.baseline = y + line.h * 0.78;
                for glyph in &mut line.glyphs {
                    glyph.dx += frame.side;
                }
                y += line.h;
                pages.last_mut().unwrap().lines.push(line);
            }
            prev_bottom = block.style.margin_bottom;
            y += block.style.margin_bottom;
        }
        Self {
            pages,
            nodes: doc.nodes.clone(),
            honored: doc.honored.iter().cloned().collect(),
            dropped: doc.dropped.iter().cloned().collect(),
            spine_step,
            items_ms,
            break_ms,
        }
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub fn pixmap(&self, faces: &Faces, page: usize) -> Pixmap {
        self.paint(faces, page, &[])
    }

    pub fn paint(&self, faces: &Faces, page: usize, highlights: &[Rect]) -> Pixmap {
        let mut pixmap = Pixmap::new(PAGE_W, PAGE_H).expect("page");
        pixmap.fill(paper());
        if let Some(page) = self.pages.get(page) {
            for line in &page.lines {
                for glyph in &line.glyphs {
                    if glyph.gid == 0 {
                        continue;
                    }
                    faces.draw(
                        &mut pixmap,
                        glyph.face,
                        glyph.gid,
                        glyph.dx,
                        line.baseline,
                        glyph.size,
                        ink(),
                    );
                }
            }
        }
        tint(&mut pixmap, highlights);
        pixmap
    }

    pub fn hit_test(&self, page: usize, x: f32, y: f32) -> Option<Caret> {
        let page = self.pages.get(page)?;
        let mut best: Option<(f32, Caret)> = None;
        for line in &page.lines {
            if y < line.y || y > line.y + line.h {
                continue;
            }
            for glyph in &line.glyphs {
                let dist = if x < glyph.dx {
                    glyph.dx - x
                } else if x > glyph.dx + glyph.w {
                    x - (glyph.dx + glyph.w)
                } else {
                    0.0
                };
                let caret = Caret { node: glyph.node, offset: glyph.off };
                if best.is_none_or(|(best_dist, _)| dist < best_dist) {
                    best = Some((dist, caret));
                }
            }
        }
        best.map(|(_, caret)| caret)
    }

    pub fn cfi(&self, caret: Caret) -> Option<String> {
        let node = self.nodes.get(caret.node as usize)?;
        Some(format!("epubcfi(/6/{}!{}:{})", self.spine_step, node.steps, caret.offset))
    }

    pub fn caret_from_cfi(&self, cfi: &str) -> Option<Caret> {
        let (steps, offset) = split_point(cfi)?;
        let node = self.nodes.iter().position(|node| node.steps == steps)? as u32;
        Some(Caret { node, offset })
    }

    pub fn rects(&self, page: usize, start: Caret, end: Caret) -> Vec<Rect> {
        let Some(page) = self.pages.get(page) else {
            return Vec::new();
        };
        let mut rects = Vec::new();
        for line in &page.lines {
            let mut run: Option<Rect> = None;
            for glyph in &line.glyphs {
                let caret = Caret { node: glyph.node, offset: glyph.off };
                if !in_range(start, caret, end) {
                    if let Some(rect) = run.take() {
                        rects.push(rect);
                    }
                    continue;
                }
                let rect = Rect { x: glyph.dx, y: line.y, w: glyph.w.max(0.5), h: line.h };
                run = Some(match run {
                    Some(prev) if rect.x + 0.5 >= prev.x && rect.x - (prev.x + prev.w) < 24.0 && (prev.y - rect.y).abs() < 0.5 => Rect {
                        x: prev.x,
                        y: prev.y,
                        w: rect.x + rect.w - prev.x,
                        h: prev.h,
                    },
                    Some(prev) => {
                        rects.push(prev);
                        rect
                    }
                    None => rect,
                });
            }
            if let Some(rect) = run {
                rects.push(rect);
            }
        }
        rects
    }

    pub fn find_phrase(&self, phrase: &str) -> Option<(Caret, Caret)> {
        for (index, node) in self.nodes.iter().enumerate() {
            if let Some(byte) = node.text.find(phrase) {
                let start = node.text[..byte].chars().count() as u32;
                let end = start + phrase.chars().count() as u32;
                return Some((
                    Caret { node: index as u32, offset: start },
                    Caret { node: index as u32, offset: end },
                ));
            }
        }
        None
    }

    pub fn page_of(&self, caret: Caret) -> Option<usize> {
        for (index, page) in self.pages.iter().enumerate() {
            for line in &page.lines {
                if line.glyphs.iter().any(|glyph| glyph.node == caret.node && glyph.off >= caret.offset && glyph.off < caret.offset.saturating_add(8))
                    || line.glyphs.iter().any(|glyph| glyph.node == caret.node && glyph.off <= caret.offset)
                        && line.glyphs.iter().any(|glyph| glyph.node == caret.node && glyph.off >= caret.offset.saturating_sub(1))
                {
                    if line.glyphs.iter().any(|glyph| glyph.node == caret.node) {
                        let min = line.glyphs.iter().filter(|glyph| glyph.node == caret.node).map(|glyph| glyph.off).min().unwrap_or(0);
                        let max = line.glyphs.iter().filter(|glyph| glyph.node == caret.node).map(|glyph| glyph.off).max().unwrap_or(0);
                        if caret.offset >= min && caret.offset <= max.saturating_add(1) {
                            return Some(index);
                        }
                    }
                }
            }
        }
        for (index, page) in self.pages.iter().enumerate() {
            if page.lines.iter().any(|line| line.glyphs.iter().any(|glyph| glyph.node == caret.node && glyph.off <= caret.offset)) {
                return Some(index);
            }
        }
        None
    }

    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        let mut missing = std::collections::BTreeSet::new();
        for (index, page) in self.pages.iter().enumerate() {
            for line in &page.lines {
                let loose = line.gap.unwrap_or(1.0) > 1.5;
                if line.overflow || line.kinsoku || loose {
                    if out.len() < 8 {
                        let right = line.glyphs.iter().map(|glyph| glyph.dx + glyph.w).fold(0.0f32, f32::max);
                        out.push(format!(
                            "page {} gap {} right {right:.0} overflow {} kinsoku {} {}",
                            index + 1,
                            line.gap.map(|gap| format!("{gap:.2}")).unwrap_or_else(|| "-".into()),
                            line.overflow,
                            line.kinsoku,
                            line_text(self, line)
                        ));
                    }
                }
                for glyph in &line.glyphs {
                    if glyph.gid == 0 && glyph.size > 0.0 {
                        if let Some(ch) = self.nodes.get(glyph.node as usize).and_then(|node| node.text.chars().nth(glyph.off as usize)) {
                            missing.insert(format!("U+{:04X}", ch as u32));
                        }
                    }
                }
            }
        }
        if !missing.is_empty() {
            out.push(format!("missing {}", missing.into_iter().collect::<Vec<_>>().join(" ")));
        }
        out
    }

    pub fn stats(&self) -> Stats {
        let mut gaps = Vec::new();
        let mut hyphenated = 0;
        let mut overflow = 0;
        let mut kinsoku = 0;
        let mut missing = 0;
        let mut lines = 0;
        for page in &self.pages {
            for line in &page.lines {
                lines += 1;
                if line.hyphen {
                    hyphenated += 1;
                }
                if line.overflow {
                    overflow += 1;
                }
                if line.kinsoku {
                    kinsoku += 1;
                }
                if let Some(gap) = line.gap {
                    gaps.push(gap);
                }
                missing += line.glyphs.iter().filter(|glyph| glyph.gid == 0 && glyph.size > 0.0).count();
            }
        }
        gaps.sort_by(|a, b| a.total_cmp(b));
        let gap_p95 = percentile(&gaps, 0.95);
        let gap_max = gaps.last().copied().unwrap_or(0.0);
        Stats {
            pages: self.pages.len(),
            lines,
            hyphenated_lines: hyphenated,
            gap_p95,
            gap_max,
            overflow_lines: overflow,
            kinsoku_violations: kinsoku,
            missing_glyphs: missing,
        }
    }

    pub fn nodes(&self) -> &[TextNode] {
        &self.nodes
    }
}

struct Frame {
    side: f32,
    top: f32,
    bottom: f32,
    measure: f32,
}

fn frame_for(cjk: bool) -> Frame {
    let w = PAGE_W as f32;
    let h = PAGE_H as f32;
    if cjk {
        let side = w * 0.11;
        Frame { side, top: 72.0, bottom: h - 64.0, measure: w - side * 2.0 }
    } else {
        let side = w * 0.14;
        Frame { side, top: 88.0, bottom: h - 72.0, measure: w - side * 2.0 }
    }
}

fn compose(
    faces: &Faces,
    hyphen: &Hyphenator,
    block: &Block,
    frame: &Frame,
    choices: Choices,
) -> (Vec<Line>, f64, f64) {
    let style = &block.style;
    let started = Instant::now();
    let (atoms, glyphs) = items_for(faces, hyphen, block, choices);
    let item_ms = started.elapsed().as_secs_f64() * 1000.0;
    if atoms.is_empty() {
        return (Vec::new(), item_ms, 0.0);
    }
    let started = Instant::now();
    let indent = if style.align == Align::Center { 0.0 } else { style.indent.max(0.0) };
    let measure = frame.measure;
    let broken = breaker::break_paragraph(&atoms, |line| {
        if line == 0 { (measure - indent).max(40.0) } else { measure }
    }, max_ratio(choices.hyphenation));
    let break_ms = started.elapsed().as_secs_f64() * 1000.0;
    let mut lines = Vec::new();
    let line_h = style.size * style.leading;
    for (index, piece) in broken.iter().enumerate() {
        let placed = place_line(
            &atoms,
            &glyphs,
            piece,
            if index == 0 { indent } else { 0.0 },
            style.align,
            index + 1 == broken.len(),
            measure,
        );
        let kinsoku = kinsoku_bad(&placed);
        let gap = line_gap(&atoms, piece, style.align, index + 1 == broken.len());
        lines.push(Line {
            y: 0.0,
            h: line_h,
            baseline: 0.0,
            glyphs: placed,
            gap,
            hyphen: piece.hyphen,
            overflow: piece.overflow,
            kinsoku,
        });
    }
    (lines, item_ms, break_ms)
}

fn max_ratio(mode: Hyphenation) -> f32 {
    match mode {
        Hyphenation::Tex => 1.26,
        Hyphenation::Moderate | Hyphenation::Off => 0.75,
    }
}

fn items_for(faces: &Faces, hyphen: &Hyphenator, block: &Block, choices: Choices) -> (Vec<Atom>, Vec<Vec<Glyph>>) {
    let mut atoms = Vec::new();
    let mut glyphs = Vec::new();
    let mut prev = Class::Other;
    let latin_book = !block.spans.iter().any(|span| span.text.chars().any(is_cjk));
    for (span_index, span) in block.spans.iter().enumerate() {
        if span.text == "\n" {
            atoms.push(Atom::penalty(breaker::MIN_COST, 0.0, false));
            glyphs.push(Vec::new());
            prev = Class::Other;
            continue;
        }
        let mut chars: Vec<(char, u32)> = Vec::new();
        for (index, ch) in span.text.chars().enumerate() {
            chars.push((ch, index as u32));
        }
        let mut i = 0;
        while i < chars.len() {
            let (ch, off) = chars[i];
            if ch.is_whitespace() && ch != '\u{00a0}' {
                let mut j = i;
                while j < chars.len() && chars[j].0.is_whitespace() && chars[j].0 != '\u{00a0}' {
                    j += 1;
                }
                let next = if j < chars.len() {
                    classify(chars[j].0)
                } else {
                    first_class(&block.spans, span_index + 1)
                };
                let at_start = atoms.iter().all(|atom| atom.kind != breaker::Kind::Box);
                let boundary = script_boundary(prev, next);
                if boundary && !at_start {
                    let gap = span.style.size * choices.cjk_latin_em;
                    if gap > 0.0 {
                        push_glue(&mut atoms, &mut glyphs, Atom::glue(gap, gap * 0.5, gap * 0.25));
                    }
                    prev = Class::Other;
                } else if !at_start && !matches!(prev, Class::Cjk | Class::Open | Class::Close) && !matches!(next, Class::Cjk | Class::Open | Class::Close) {
                    let width = space_width(faces, &span.style);
                    push_glue(&mut atoms, &mut glyphs, Atom::glue(width, width * 0.5, width * 0.33));
                    prev = Class::Other;
                }
                i = j;
                continue;
            }
            if ch.is_whitespace() {
                let width = space_width(faces, &span.style);
                push_box(&mut atoms, &mut glyphs, width, vec![space_glyph(width, span.node, off)]);
                prev = Class::Other;
                i += 1;
                continue;
            }
            if is_latin_punct(ch, latin_book) {
                let opts = shape_opts(&span.style, false);
                let shaped = faces.shape(&ch.to_string(), &opts);
                let adv = shaped.iter().map(|glyph| glyph.advance).sum::<f32>().max(span.style.size * 0.25);
                if script_boundary(prev, Class::Latin) && choices.cjk_latin_em > 0.0 {
                    let gap = span.style.size * choices.cjk_latin_em;
                    push_glue(&mut atoms, &mut glyphs, Atom::glue(gap, gap * 0.5, gap * 0.25));
                }
                let glyph = Glyph {
                    face: face_id(&opts),
                    gid: shaped.first().map(|glyph| glyph.gid).unwrap_or(0),
                    dx: 0.0,
                    w: adv,
                    size: span.style.size,
                    node: span.node,
                    off,
                    class: Class::Latin,
                    hang: false,
                };
                push_box(&mut atoms, &mut glyphs, adv, vec![glyph]);
                prev = Class::Latin;
                i += 1;
                continue;
            }
            let class = classify(ch);
            if class == Class::Latin {
                let start = i;
                if class == Class::Latin {
                    while i < chars.len() && classify(chars[i].0) == Class::Latin {
                        i += 1;
                    }
                } else {
                    i += 1;
                }
                let word: String = chars[start..i].iter().map(|(ch, _)| *ch).collect();
                let off0 = chars[start].1;
                push_latin(
                    faces,
                    hyphen,
                    &mut atoms,
                    &mut glyphs,
                    &word,
                    span,
                    off0,
                    prev,
                    choices,
                    block.style.hyphens && span.style.hyphens && !span.style.nowrap,
                );
                prev = Class::Latin;
                continue;
            }
            let run_start = i;
            if class == Class::Cjk {
                while i < chars.len() && classify(chars[i].0) == Class::Cjk {
                    i += 1;
                }
            } else {
                i += 1;
            }
            let text: String = chars[run_start..i].iter().map(|(ch, _)| *ch).collect();
            let shaped = faces.shape(&text, &shape_opts(&span.style, true));
            let mut grouped: Vec<Vec<crate::face::ShapedGlyph>> = vec![Vec::new(); text.chars().count()];
            for glyph in shaped {
                if glyph.chars < grouped.len() {
                    grouped[glyph.chars].push(glyph);
                }
            }
            for (index, (ch, off)) in chars[run_start..i].iter().copied().enumerate() {
                let class = classify(ch);
                let group = grouped.get(index).cloned().unwrap_or_default();
                let adv = if group.is_empty() {
                    span.style.size
                } else {
                    group.iter().map(|glyph| glyph.advance).sum()
                };
                let gid = group.first().map(|glyph| glyph.gid).unwrap_or(0);
                push_char(
                    &mut atoms,
                    &mut glyphs,
                    &mut prev,
                    class,
                    adv,
                    gid,
                    span.style.size,
                    span.node,
                    off,
                    choices,
                );
            }
        }
    }
    push_glue(&mut atoms, &mut glyphs, Atom::glue(0.0, 10_000.0, 0.0));
    atoms.push(Atom::forced());
    glyphs.push(Vec::new());
    (atoms, glyphs)
}

fn push_latin(
    faces: &Faces,
    hyphen: &Hyphenator,
    atoms: &mut Vec<Atom>,
    payloads: &mut Vec<Vec<Glyph>>,
    word: &str,
    span: &crate::parse::Span,
    off0: u32,
    prev: Class,
    choices: Choices,
    allow_hyphen: bool,
) {
    if matches!(prev, Class::Cjk | Class::Close) && choices.cjk_latin_em > 0.0 {
        let gap = span.style.size * choices.cjk_latin_em;
        push_glue(atoms, payloads, Atom::glue(gap, gap * 0.5, gap * 0.25));
    }
    let opts = shape_opts(&span.style, false);
    let breaks = if allow_hyphen && choices.hyphenation != Hyphenation::Off && word.chars().all(|ch| ch.is_ascii_alphabetic()) {
        let (left, right, min_len) = if choices.hyphenation == Hyphenation::Tex {
            (2, 3, 5)
        } else {
            (3, 3, 6)
        };
        if word.chars().count() >= min_len {
            hyphen.breaks(word, left, right)
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    let penalty = match choices.hyphenation {
        Hyphenation::Tex => 50.0,
        Hyphenation::Moderate | Hyphenation::Off => 200.0,
    };
    let mut points = vec![0];
    points.extend(breaks);
    points.push(word.chars().count());
    let chars: Vec<char> = word.chars().collect();
    for window in points.windows(2) {
        let a = window[0];
        let b = window[1];
        let piece: String = chars[a..b].iter().collect();
        let shaped = faces.shape(&piece, &opts);
        let mut glyph_list = Vec::new();
        let mut width = 0.0;
        for glyph in &shaped {
            glyph_list.push(Glyph {
                face: face_id(&opts),
                gid: glyph.gid,
                dx: glyph.x,
                w: glyph.advance,
                size: span.style.size,
                node: span.node,
                off: off0 + a as u32 + glyph.chars as u32,
                class: Class::Latin,
                hang: false,
            });
            width += glyph.advance;
        }
        if glyph_list.is_empty() {
            width = span.style.size * 0.5 * piece.chars().count() as f32;
            glyph_list.push(Glyph {
                face: face_id(&opts),
                gid: 0,
                dx: 0.0,
                w: width,
                size: span.style.size,
                node: span.node,
                off: off0 + a as u32,
                class: Class::Latin,
                hang: false,
            });
        }
        push_box(atoms, payloads, width, glyph_list);
        if b != chars.len() {
            let hyphen_g = faces.shape("-", &opts);
            let hw = hyphen_g.first().map(|glyph| glyph.advance).unwrap_or(span.style.size * 0.3);
            let mut hy = Vec::new();
            if let Some(glyph) = hyphen_g.first() {
                hy.push(Glyph {
                    face: face_id(&opts),
                    gid: glyph.gid,
                    dx: 0.0,
                    w: hw,
                    size: span.style.size,
                    node: span.node,
                    off: off0 + b as u32,
                    class: Class::Latin,
                    hang: false,
                });
            }
            atoms.push(Atom::penalty(penalty, hw, true));
            payloads.push(hy);
        }
    }
}

fn push_char(
    atoms: &mut Vec<Atom>,
    payloads: &mut Vec<Vec<Glyph>>,
    prev: &mut Class,
    class: Class,
    adv: f32,
    gid: u32,
    size: f32,
    node: u32,
    off: u32,
    choices: Choices,
) {
    let compress = choices.punctuation != Punctuation::Full && matches!(class, Class::Open | Class::Close);
    let break_before = can_break(*prev, class);
    if break_before && *prev != Class::Other {
        if latin_cjk(*prev, class) && choices.cjk_latin_em > 0.0 {
            let gap = size * choices.cjk_latin_em;
            push_glue(atoms, payloads, Atom::glue(gap, gap * 0.5, gap * 0.25));
        } else if matches!(*prev, Class::Cjk | Class::Close) && matches!(class, Class::Cjk | Class::Open) {
            let stretch = size / 18.0;
            push_glue(atoms, payloads, Atom::glue(0.0, stretch, 0.0));
        }
    }
    if choices.punctuation == Punctuation::Hang && class == Class::Close {
        let glyph = Glyph {
            face: 2,
            gid,
            dx: 0.0,
            w: adv,
            size,
            node,
            off,
            class,
            hang: true,
        };
        push_box(atoms, payloads, 0.0, vec![glyph]);
        push_glue(atoms, payloads, Atom::glue(adv, 0.0, 0.0));
        *prev = class;
        return;
    }
    let (box_w, before, after) = if compress && class == Class::Open {
        (adv * 0.5, adv * 0.5, 0.0)
    } else if compress && class == Class::Close {
        (adv * 0.5, 0.0, adv * 0.5)
    } else {
        (adv, 0.0, 0.0)
    };
    if before > 0.0 {
        push_glue(atoms, payloads, Atom::glue(before, 0.0, before));
    }
    let glyph = Glyph {
        face: 2,
        gid,
        dx: 0.0,
        w: box_w,
        size,
        node,
        off,
        class,
        hang: false,
    };
    push_box(atoms, payloads, box_w, vec![glyph]);
    if after > 0.0 {
        push_glue(atoms, payloads, Atom::glue(after, 0.0, after));
    }
    *prev = class;
}

fn can_break(prev: Class, next: Class) -> bool {
    if prev == Class::Open {
        return false;
    }
    if matches!(next, Class::Close) {
        return false;
    }
    true
}

fn script_boundary(prev: Class, next: Class) -> bool {
    latin_cjk(prev, next)
}

fn first_class(spans: &[crate::parse::Span], from: usize) -> Class {
    for span in spans.iter().skip(from) {
        for ch in span.text.chars() {
            if ch.is_whitespace() {
                continue;
            }
            return classify(ch);
        }
    }
    Class::Other
}

fn is_latin_punct(ch: char, latin_book: bool) -> bool {
    ch.is_ascii_punctuation()
        || ch == '…'
        || ch == '–'
        || (latin_book && "“”‘’".contains(ch))
}

fn latin_cjk(prev: Class, next: Class) -> bool {
    let latin = matches!(prev, Class::Latin);
    let cjk = matches!(next, Class::Cjk | Class::Open | Class::Close);
    let latin2 = matches!(next, Class::Latin);
    let cjk2 = matches!(prev, Class::Cjk | Class::Open | Class::Close);
    (latin && cjk) || (cjk2 && latin2)
}

fn push_box(atoms: &mut Vec<Atom>, payloads: &mut Vec<Vec<Glyph>>, width: f32, glyphs: Vec<Glyph>) {
    atoms.push(Atom::box_(width));
    payloads.push(glyphs);
}

fn push_glue(atoms: &mut Vec<Atom>, payloads: &mut Vec<Vec<Glyph>>, atom: Atom) {
    atoms.push(atom);
    payloads.push(Vec::new());
}

fn space_glyph(width: f32, node: u32, off: u32) -> Glyph {
    Glyph {
        face: 0,
        gid: 0,
        dx: 0.0,
        w: width,
        size: 0.0,
        node,
        off,
        class: Class::Other,
        hang: false,
    }
}

fn shape_opts(style: &crate::parse::Style, cjk: bool) -> ShapeOpts {
    ShapeOpts {
        italic: style.italic,
        cjk,
        size: style.size,
        weight: style.weight,
        small_caps: style.small_caps && !cjk,
        oldstyle: style.oldstyle && !cjk,
    }
}

fn face_id(opts: &ShapeOpts) -> u8 {
    if opts.cjk {
        2
    } else if opts.italic {
        1
    } else {
        0
    }
}

fn space_width(faces: &Faces, style: &crate::parse::Style) -> f32 {
    let shaped = faces.shape(" ", &shape_opts(style, false));
    shaped.first().map(|glyph| glyph.advance).filter(|w| *w > 0.1).unwrap_or(style.size * 0.33)
}

fn place_line(
    atoms: &[Atom],
    payloads: &[Vec<Glyph>],
    piece: &BrokenLine,
    indent: f32,
    align: Align,
    last: bool,
    measure: f32,
) -> Vec<Glyph> {
    let mut ratio = if align != Align::Justify || last { 0.0 } else { piece.ratio };
    if !ratio.is_finite() {
        ratio = 0.0;
    }
    let mut x = indent;
    let mut out = Vec::new();
    if piece.start_item > piece.end_item || piece.end_item >= atoms.len() {
        return out;
    }
    for index in piece.start_item..=piece.end_item {
        let atom = &atoms[index];
        match atom.kind {
            breaker::Kind::Box => {
                for glyph in &payloads[index] {
                    let mut placed = glyph.clone();
                    placed.dx += x;
                    out.push(placed);
                }
                x += atom.width;
            }
            breaker::Kind::Glue if index != piece.start_item && index != piece.end_item => {
                x += breaker::glue_width(atom, ratio);
            }
            breaker::Kind::Penalty if index == piece.end_item && atom.width > 0.0 => {
                for glyph in &payloads[index] {
                    let mut placed = glyph.clone();
                    placed.dx += x;
                    out.push(placed);
                }
                x += atom.width;
            }
            _ => {}
        }
    }
    if align == Align::Center && !out.is_empty() {
        let left = out.iter().map(|glyph| glyph.dx).fold(f32::MAX, f32::min);
        let right = out.iter().map(|glyph| glyph.dx + glyph.w).fold(0.0, f32::max);
        let slide = ((measure - (right - left)) * 0.5 - left).max(0.0);
        for glyph in &mut out {
            glyph.dx += slide;
        }
    }
    out
}

fn kinsoku_bad(glyphs: &[Glyph]) -> bool {
    let Some(first) = glyphs.iter().find(|glyph| glyph.w > 0.0) else {
        return false;
    };
    let Some(last) = glyphs.iter().rev().find(|glyph| glyph.w > 0.0 || glyph.hang) else {
        return false;
    };
    first.class == Class::Close || last.class == Class::Open
}

fn line_gap(atoms: &[Atom], piece: &BrokenLine, align: Align, last: bool) -> Option<f32> {
    if align != Align::Justify || last || piece.ratio < 0.0 {
        return None;
    }
    let mut sample = None;
    if piece.start_item > piece.end_item {
        return None;
    }
    for index in piece.start_item..=piece.end_item.min(atoms.len() - 1) {
        let atom = &atoms[index];
        if atom.kind == breaker::Kind::Glue && atom.width > 0.4 && index != piece.end_item {
            let adjusted = breaker::glue_width(atom, piece.ratio.max(0.0));
            sample = Some(adjusted / atom.width);
            break;
        }
    }
    sample
}

fn classify(ch: char) -> Class {
    if "「『（【〈《〔〖〘〚｛".contains(ch) {
        Class::Open
    } else if "」』）】〉》〕〗〙〛｝，。、．！？：；・､｡".contains(ch) {
        Class::Close
    } else if "“‘".contains(ch) {
        Class::Open
    } else if ch == '”' {
        Class::Close
    } else if is_cjk(ch) {
        Class::Cjk
    } else if ch.is_alphanumeric() || ch == '\'' || ch == '’' || ch == '-' || ch == '—' {
        Class::Latin
    } else {
        Class::Other
    }
}

fn is_cjk(ch: char) -> bool {
    let o = ch as u32;
    (0x3400..=0x9FFF).contains(&o)
        || (0xF900..=0xFAFF).contains(&o)
        || (0x3040..=0x30FF).contains(&o)
        || (0xFF01..=0xFF60).contains(&o)
}

fn line_text(laid: &LaidOut, line: &Line) -> String {
    let Some(first) = line.glyphs.first() else {
        return String::new();
    };
    let node = first.node;
    let Some(text) = laid.nodes.get(node as usize).map(|node| node.text.as_str()) else {
        return String::new();
    };
    let start = line.glyphs.iter().filter(|glyph| glyph.node == node).map(|glyph| glyph.off).min().unwrap_or(0);
    let end = line.glyphs.iter().filter(|glyph| glyph.node == node).map(|glyph| glyph.off).max().unwrap_or(start);
    let chars: Vec<char> = text.chars().collect();
    let end = (end as usize + 1).min(chars.len());
    let start = (start as usize).min(end);
    chars[start..end].iter().collect::<String>().chars().take(80).collect()
}

fn percentile(sorted: &[f32], p: f32) -> f32 {
    if sorted.is_empty() {
        return 0.0;
    }
    let index = ((sorted.len() as f32) * p).ceil() as usize;
    sorted[(index.saturating_sub(1)).min(sorted.len() - 1)]
}

fn in_range(start: Caret, caret: Caret, end: Caret) -> bool {
    caret_le(start, caret) && caret_lt(caret, end)
}

fn caret_le(a: Caret, b: Caret) -> bool {
    a.node < b.node || (a.node == b.node && a.offset <= b.offset)
}

fn caret_lt(a: Caret, b: Caret) -> bool {
    a.node < b.node || (a.node == b.node && a.offset < b.offset)
}

fn split_point(cfi: &str) -> Option<(String, u32)> {
    let body = cfi.trim().trim_start_matches("epubcfi(").trim_end_matches(')');
    let content = body.split('!').nth(1).unwrap_or(body);
    let content = content.trim_start_matches(',');
    let (path, offset) = content.rsplit_once(':')?;
    let offset = offset.chars().take_while(|ch| ch.is_ascii_digit()).collect::<String>().parse().ok()?;
    Some((path.to_string(), offset))
}

fn tint(pixmap: &mut Pixmap, rects: &[Rect]) {
    if rects.is_empty() {
        return;
    }
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    let data = pixmap.data_mut();
    for rect in rects {
        let x0 = rect.x.floor().max(0.0) as i32;
        let y0 = rect.y.floor().max(0.0) as i32;
        let x1 = (rect.x + rect.w).ceil().min(width as f32) as i32;
        let y1 = (rect.y + rect.h).ceil().min(height as f32) as i32;
        for y in y0..y1 {
            for x in x0..x1 {
                let i = ((y * width + x) * 4) as usize;
                let sum = data[i] as u16 + data[i + 1] as u16 + data[i + 2] as u16;
                if sum > 520 {
                    data[i] = ((data[i] as u16 * 6 + 0xf6 * 4) / 10) as u8;
                    data[i + 1] = ((data[i + 1] as u16 * 6 + 0xe2 * 4) / 10) as u8;
                    data[i + 2] = ((data[i + 2] as u16 * 6 + 0x7a * 4) / 10) as u8;
                }
            }
        }
    }
}

pub fn encode(pixmap: &Pixmap) -> Vec<u8> {
    pixmap.encode_png().unwrap_or_default()
}

pub fn side_by_side(faces: &Faces, panels: &[(String, &Pixmap)]) -> Pixmap {
    let gap = 28u32;
    let label_h = 36u32;
    let width = panels.iter().map(|(_, pixmap)| pixmap.width()).sum::<u32>() + gap * (panels.len().saturating_sub(1) as u32);
    let height = panels.iter().map(|(_, pixmap)| pixmap.height()).max().unwrap_or(0) + label_h;
    let mut out = Pixmap::new(width.max(1), height.max(1)).expect("sheet");
    out.fill(Color::from_rgba8(0xff, 0xff, 0xff, 0xff));
    let mut x = 0u32;
    for (label, pixmap) in panels {
        draw_label(faces, &mut out, label, x + 8, 24);
        blit(&mut out, pixmap, x, label_h);
        x += pixmap.width() + gap;
    }
    out
}

fn blit(dest: &mut Pixmap, src: &Pixmap, dx: u32, dy: u32) {
    let dw = dest.width();
    let data = dest.data_mut();
    let sw = src.width();
    let sh = src.height();
    let src_data = src.data();
    for y in 0..sh {
        for x in 0..sw {
            let di = (((dy + y) * dw + (dx + x)) * 4) as usize;
            let si = ((y * sw + x) * 4) as usize;
            if di + 3 < data.len() && si + 3 < src_data.len() {
                data[di..di + 4].copy_from_slice(&src_data[si..si + 4]);
            }
        }
    }
}

fn draw_label(faces: &Faces, pixmap: &mut Pixmap, text: &str, x: u32, baseline: u32) {
    let opts = ShapeOpts {
        italic: false,
        cjk: false,
        size: 16.0,
        weight: 600,
        small_caps: false,
        oldstyle: false,
    };
    let shaped = faces.shape(text, &opts);
    for glyph in shaped {
        faces.draw(
            pixmap,
            0,
            glyph.gid,
            x as f32 + glyph.x,
            baseline as f32,
            16.0,
            Color::from_rgba8(0x22, 0x22, 0x22, 0xff),
        );
    }
}
