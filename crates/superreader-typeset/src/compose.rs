use crate::face::{self, Faces, ShapeOpts};
use crate::hyphen::Hyphenator;
use crate::kp::{self, BreakOpts, Item, Kind, MaxAdjustmentExceeded};
use crate::parse::{self, Align, Block, Chunk, Paint};
use crate::types::*;
use icu_segmenter::options::{LineBreakOptions, LineBreakStrictness, LineBreakWordOption};
use icu_segmenter::LineSegmenter;
use std::collections::HashSet;
use std::time::Instant;
use tiny_skia::Pixmap;

const OPEN: &str = "「『（〈《【〔〖〘〚“‘";
const CLOSE: &str = "」』）〉》】〕〗〙〛”’。、，．！？；：｡､";

pub struct Typesetter {
    faces: Faces,
}

impl Typesetter {
    pub fn open(latin: &str, italic: &str, cjk: &str) -> Result<Self, TypesetError> {
        let faces = Faces::load(latin, italic, cjk).map_err(TypesetError::Font)?;
        Ok(Self { faces })
    }

    pub fn compose(&self, input: ChapterInput<'_>) -> Result<ChapterLayout, TypesetError> {
        let prepared = prepare(input.xhtml, input.stylesheet, input.font_size);
        Ok(self.layout_prepared(&prepared, input.measure))
    }

    pub fn png(&self, layout: &ChapterLayout, page: u32) -> Option<Vec<u8>> {
        let page = layout.pages.get(page as usize)?;
        let mut pixmap = Pixmap::new(PAPERBACK_WIDTH as u32, PAPERBACK_HEIGHT as u32)?;
        pixmap.fill(face::paper());
        for line in &page.lines {
            for glyph in &line.glyphs {
                self.faces.draw(
                    &mut pixmap,
                    glyph.font.0 as u8,
                    glyph.glyph_id,
                    glyph.x,
                    glyph.baseline_y,
                    glyph.size,
                    face::ink(),
                );
            }
        }
        pixmap.encode_png().ok()
    }

    fn layout_prepared(&self, prepared: &Prepared, measure: Option<f32>) -> ChapterLayout {
        let frame = frame_for(prepared.cjk, measure);
        let mut line_breaks = LineBreakOptions::default();
        line_breaks.strictness = Some(LineBreakStrictness::Strict);
        line_breaks.word_option = Some(LineBreakWordOption::Normal);
        let segmenter = LineSegmenter::new_auto(line_breaks);
        let hyphen = Hyphenator::load();
        let mut shape_ms = 0.0;
        let mut break_ms = 0.0;
        let mut pages = vec![Page { lines: Vec::new() }];
        let mut cursor = frame.top;
        let mut gaps = Vec::new();
        let mut hyphenated = 0u32;
        let mut line_count = 0u32;
        let mut kinsoku = 0u32;
        let mut overflow = 0u32;

        for block in &prepared.blocks {
            let shape_at = Instant::now();
            let recs = shape_block(&self.faces, block, prepared.cjk);
            shape_ms += shape_at.elapsed().as_secs_f64() * 1000.0;
            if recs.iter().all(|rec| rec.forced || rec.space) {
                continue;
            }
            cursor += block.paint.margin_top;
            for (seg_i, seg) in split_forced(&recs).into_iter().enumerate() {
                if seg.is_empty() {
                    continue;
                }
                let break_at = Instant::now();
                let (atoms, spans) = break_segment(
                    &self.faces,
                    segmenter,
                    &hyphen,
                    seg,
                    &block.paint,
                    frame.measure,
                    seg_i == 0,
                );
                break_ms += break_at.elapsed().as_secs_f64() * 1000.0;
                for span in spans {
                    let leading = block.paint.leading.max(block.paint.size);
                    if cursor > frame.top + 0.5
                        && cursor + leading > frame.bottom
                        && !pages.last().unwrap().lines.is_empty()
                    {
                        pages.push(Page { lines: Vec::new() });
                        cursor = frame.top;
                    }
                    let baseline = cursor + block.paint.size * 0.8;
                    let placed = place_line(
                        &atoms,
                        &span,
                        frame.left,
                        frame.measure,
                        baseline,
                        &block.paint,
                        span.first,
                    );
                    if placed.hyphenated {
                        hyphenated += 1;
                    }
                    if placed.kinsoku {
                        kinsoku += 1;
                    }
                    if placed.overflow {
                        overflow += 1;
                    }
                    if placed.justified {
                        gaps.extend(placed.gaps.iter().copied());
                    }
                    line_count += 1;
                    pages.last_mut().unwrap().lines.push(Line {
                        glyphs: placed.glyphs,
                        top: cursor,
                        height: leading,
                    });
                    cursor += leading;
                }
            }
            cursor += block.paint.margin_bottom;
        }

        gaps.retain(|gap| gap.is_finite() && *gap > 0.0);
        let word_gap_max = gaps.iter().copied().fold(0.0_f64, f64::max);
        let word_gap_p95 = percentile_95(&mut gaps);
        ChapterLayout {
            content: ContentBox {
                left: frame.left,
                right: frame.left + frame.measure,
                top: frame.top,
                bottom: frame.bottom,
            },
            report: LayoutReport {
                page_count: pages.len() as u32,
                word_gap_p95,
                word_gap_max,
                hyphenated_lines: hyphenated,
                line_count,
                kinsoku_violations: kinsoku,
                overflow_lines: overflow,
                shape_ms,
                break_ms,
            },
            pages,
        }
    }
}

pub struct ChapterLayout {
    pages: Vec<Page>,
    report: LayoutReport,
    content: ContentBox,
}

struct Page {
    lines: Vec<Line>,
}

struct Line {
    glyphs: Vec<PlacedGlyph>,
    top: f32,
    height: f32,
}

impl ChapterLayout {
    pub fn report(&self) -> &LayoutReport {
        &self.report
    }

    pub fn content_box(&self, _page: u32) -> ContentBox {
        self.content
    }

    pub fn glyph_runs(&self, page: u32) -> Vec<GlyphRun> {
        let Some(page) = self.pages.get(page as usize) else {
            return Vec::new();
        };
        let mut runs = Vec::new();
        for line in &page.lines {
            for glyph in &line.glyphs {
                if runs.last().is_some_and(|run: &GlyphRun| run.font == glyph.font) {
                    runs.last_mut().unwrap().glyphs.push(glyph.clone());
                } else {
                    runs.push(GlyphRun {
                        font: glyph.font,
                        glyphs: vec![glyph.clone()],
                    });
                }
            }
        }
        runs
    }

    pub fn hit_test(&self, page: u32, point: Point) -> Option<Caret> {
        let page = self.pages.get(page as usize)?;
        let line = page
            .lines
            .iter()
            .find(|line| point.y >= line.top && point.y < line.top + line.height)
            .or_else(|| page.lines.last())?;
        let glyphs: Vec<&PlacedGlyph> = line.glyphs.iter().filter(|glyph| !glyph.hyphen).collect();
        let first = *glyphs.first()?;
        if point.x <= first.x {
            return Some(Caret { node: first.node, offset: first.offset });
        }
        for glyph in &glyphs {
            if point.x < glyph.x + glyph.advance * 0.5 {
                return Some(Caret { node: glyph.node, offset: glyph.offset });
            }
        }
        let last = *glyphs.last()?;
        Some(Caret {
            node: last.node,
            offset: last.offset.saturating_add(last.span),
        })
    }

    pub fn selection_rects(&self, page: u32, mut range: CaretRange) -> Vec<SelectionRect> {
        if (range.end.node, range.end.offset) < (range.start.node, range.start.offset) {
            std::mem::swap(&mut range.start, &mut range.end);
        }
        let Some(page) = self.pages.get(page as usize) else {
            return Vec::new();
        };
        let mut rects = Vec::new();
        for line in &page.lines {
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut any = false;
            for glyph in &line.glyphs {
                if glyph.hyphen {
                    continue;
                }
                let key = (glyph.node, glyph.offset);
                let start = (range.start.node, range.start.offset);
                let end = (range.end.node, range.end.offset);
                if key >= start && key < end {
                    min_x = min_x.min(glyph.x);
                    max_x = max_x.max(glyph.x + glyph.advance);
                    any = true;
                }
            }
            if any {
                rects.push(SelectionRect {
                    x: min_x,
                    y: line.top,
                    width: (max_x - min_x).max(0.0),
                    height: line.height,
                });
            }
        }
        rects
    }
}

struct Prepared {
    blocks: Vec<Block>,
    cjk: bool,
}

fn prepare(xhtml: &str, stylesheet: &str, font_size: Option<f32>) -> Prepared {
    let cjk = parse::cjk_book(xhtml);
    Prepared {
        blocks: parse::parse_chapter(xhtml, stylesheet, cjk, font_size),
        cjk,
    }
}

struct Frame {
    left: f32,
    top: f32,
    bottom: f32,
    measure: f32,
}

fn frame_for(cjk: bool, measure: Option<f32>) -> Frame {
    let side = PAPERBACK_WIDTH * if cjk { 0.11 } else { 0.14 };
    let measure = measure.unwrap_or(PAPERBACK_WIDTH - side * 2.0).max(1.0);
    if cjk {
        Frame {
            left: side,
            top: 72.0,
            bottom: PAPERBACK_HEIGHT - 64.0,
            measure,
        }
    } else {
        Frame {
            left: side,
            top: 88.0,
            bottom: PAPERBACK_HEIGHT - 72.0,
            measure,
        }
    }
}

struct Rec {
    ch: char,
    node: u32,
    offset: u32,
    size: f32,
    italic: bool,
    weight: u16,
    small_caps: bool,
    cjk: bool,
    space: bool,
    forced: bool,
    font: u8,
    gid: u32,
    advance: f32,
    x_offset: f32,
}

fn shape_block(faces: &Faces, block: &Block, cjk_book: bool) -> Vec<Rec> {
    let mut recs = Vec::new();
    let mut pending_space = false;
    for chunk in &block.chunks {
        match chunk {
            Chunk::Break => {
                pending_space = false;
                recs.push(marker(&block.paint, true));
            }
            Chunk::Text(piece) => {
                let mut offset = 0u32;
                for ch in piece.text.chars() {
                    if ch == '\u{00A0}' {
                        flush_space(&mut recs, &mut pending_space, false);
                        let mut rec = marker(&piece.paint, false);
                        rec.node = piece.node;
                        rec.offset = offset;
                        rec.ch = ch;
                        rec.space = true;
                        recs.push(rec);
                    } else if ch.is_whitespace() {
                        pending_space = !recs.is_empty();
                    } else {
                        let cjk = cjk_char(ch, cjk_book);
                        flush_space(&mut recs, &mut pending_space, cjk);
                        recs.push(Rec {
                            ch,
                            node: piece.node,
                            offset,
                            size: piece.paint.size,
                            italic: piece.paint.italic,
                            weight: piece.paint.weight,
                            small_caps: piece.paint.small_caps,
                            cjk,
                            space: false,
                            forced: false,
                            font: if cjk { 2 } else if piece.paint.italic { 1 } else { 0 },
                            gid: 0,
                            advance: piece.paint.size * 0.5,
                            x_offset: 0.0,
                        });
                    }
                    offset += 1;
                }
            }
        }
    }
    while recs.last().is_some_and(|rec| rec.space) {
        recs.pop();
    }
    tie_ambiguous_quotes(&mut recs);
    assign_glyphs(faces, &mut recs);
    recs
}

fn marker(paint: &Paint, forced: bool) -> Rec {
    Rec {
        ch: if forced { '\n' } else { ' ' },
        node: 0,
        offset: 0,
        size: paint.size,
        italic: false,
        weight: 400,
        small_caps: false,
        cjk: false,
        space: !forced,
        forced,
        font: 0,
        gid: 0,
        advance: paint.size * 0.25,
        x_offset: 0.0,
    }
}

fn flush_space(recs: &mut Vec<Rec>, pending: &mut bool, next_cjk: bool) {
    if !*pending {
        return;
    }
    *pending = false;
    let prev_cjk = recs.iter().rev().find(|rec| !rec.space && !rec.forced).map(|rec| rec.cjk);
    let keep = match prev_cjk {
        Some(false) if !next_cjk => true,
        Some(true) if next_cjk => false,
        Some(_) => true,
        None => false,
    };
    if keep {
        let size = recs.last().map(|rec| rec.size).unwrap_or(22.0);
        recs.push(Rec {
            ch: ' ',
            node: recs.last().map(|rec| rec.node).unwrap_or(0),
            offset: 0,
            size,
            italic: false,
            weight: 400,
            small_caps: false,
            cjk: false,
            space: true,
            forced: false,
            font: 0,
            gid: 0,
            advance: size * 0.25,
            x_offset: 0.0,
        });
    }
}

fn assign_glyphs(faces: &Faces, recs: &mut [Rec]) {
    let mut i = 0;
    while i < recs.len() {
        if recs[i].space || recs[i].forced {
            i += 1;
            continue;
        }
        let font = recs[i].font;
        let size = recs[i].size;
        let italic = recs[i].italic;
        let weight = recs[i].weight;
        let small_caps = recs[i].small_caps;
        let start = i;
        i += 1;
        while i < recs.len()
            && !recs[i].space
            && !recs[i].forced
            && recs[i].font == font
            && (recs[i].size - size).abs() < 0.1
            && recs[i].italic == italic
            && recs[i].weight == weight
            && recs[i].small_caps == small_caps
        {
            i += 1;
        }
        let text: String = recs[start..i].iter().map(|rec| rec.ch).collect();
        let shaped = faces.shape(
            &text,
            &ShapeOpts {
                italic,
                cjk: font == 2,
                size,
                weight,
                small_caps,
                oldstyle: font != 2,
            },
        );
        // A ligature is one glyph whose cluster is the first source character.
        // The other characters in that cluster keep no advance of their own.
        for rec in &mut recs[start..i] {
            rec.gid = 0;
            rec.advance = 0.0;
            rec.x_offset = 0.0;
        }
        for glyph in shaped {
            if glyph.char_index >= i - start {
                continue;
            }
            let rec = &mut recs[start + glyph.char_index];
            if rec.gid == 0 {
                rec.gid = glyph.gid;
                rec.advance = glyph.advance;
                rec.x_offset = glyph.x_offset;
            } else {
                rec.advance += glyph.advance;
            }
        }
    }
}

fn split_forced(recs: &[Rec]) -> Vec<&[Rec]> {
    let mut out = Vec::new();
    let mut start = 0;
    for (i, rec) in recs.iter().enumerate() {
        if rec.forced {
            out.push(&recs[start..i]);
            start = i + 1;
        }
    }
    out.push(&recs[start..]);
    out
}

#[derive(Clone)]
struct AtomGlyph {
    font: u8,
    gid: u32,
    dx: f32,
    advance: f32,
    ch: char,
    node: u32,
    offset: u32,
    span: u32,
    size: f32,
}

#[derive(Clone)]
struct Atom {
    item: Item,
    glyphs: Vec<AtomGlyph>,
}

struct Span {
    start: usize,
    end: usize,
    ratio: f64,
    first: bool,
    ragged: bool,
    hyphenated: bool,
    ideal: f64,
    fill: bool,
}

fn break_segment(
    faces: &Faces,
    segmenter: icu_segmenter::LineSegmenterBorrowed<'static>,
    hyphen: &Hyphenator,
    recs: &[Rec],
    paint: &Paint,
    measure: f32,
    first_segment: bool,
) -> (Vec<Atom>, Vec<Span>) {
    let indent = if first_segment { paint.indent } else { 0.0 };
    let mut lengths = vec![measure as f64; 512];
    lengths[0] = (measure as f64 - indent as f64).max(1.0);
    let allow = icu_breaks(segmenter, recs);
    let cjk_heavy = recs.iter().filter(|rec| rec.cjk).count() * 2 >= recs.len().max(1);
    let plain = itemize(faces, hyphen, recs, paint, &allow, false);
    if cjk_heavy {
        let breaks = greedy_breaks(&plain, &lengths);
        return spans_from(plain, breaks, &lengths, true);
    }
    // TeX §863. Pass 1 is the pretolerance pass and does not hyphenate.
    // Pass 2 is the tolerance pass and does. Both are held to the 0.75 visual
    // cap. TeX's badness 100 and 200 allow ratios of 1 and about 1.26. A trial
    // of those thresholds raised this chapter's p95 from 2.13 to 2.68, so the
    // passes use the cap. Pass 3 adds emergencystretch, smallest of 1em and
    // 2em that fits the cap, and keeps that set only when its true ratios are
    // tighter than the minimum-demerit set.
    if let Some(breaks) = feasible_at(&plain, &lengths, VISUAL_RATIO_CAP) {
        return spans_from(plain, breaks, &lengths, false);
    }
    let em = paint.size as f64;
    if !paint.hyphens {
        return tightest(plain, &lengths, em);
    }
    let with = itemize(faces, hyphen, recs, paint, &allow, true);
    if let Some(breaks) = feasible_at(&with, &lengths, VISUAL_RATIO_CAP) {
        return spans_from(with, breaks, &lengths, false);
    }
    tightest(with, &lengths, em)
}

const VISUAL_RATIO_CAP: f64 = 0.75;

fn break_at(atoms: &[Atom], lengths: &[f64], cap: f64) -> Result<Vec<usize>, MaxAdjustmentExceeded> {
    let mut opts = BreakOpts::product();
    if cap.is_finite() {
        opts.max_adjustment_ratio = Some(cap);
        opts.initial_max_adjustment_ratio = cap.min(0.5);
    } else {
        opts.max_adjustment_ratio = None;
        opts.initial_max_adjustment_ratio = 0.5;
    }
    let items: Vec<Item> = atoms.iter().map(|atom| atom.item).collect();
    kp::break_lines(&items, lengths, opts)
}

fn loosen_glues(atoms: &[Atom], extra: f64) -> Vec<Atom> {
    atoms
        .iter()
        .map(|atom| {
            let mut copy = atom.clone();
            if copy.item.kind == Kind::Glue && copy.item.stretch > 0.0 && copy.item.stretch < 500.0 {
                copy.item.stretch += extra;
            }
            copy
        })
        .collect()
}

fn feasible_at(atoms: &[Atom], lengths: &[f64], cap: f64) -> Option<Vec<usize>> {
    let breaks = break_at(atoms, lengths, cap).ok()?;
    let ratios = ratios_of(atoms, &breaks, lengths);
    if ratios.iter().all(|ratio| ratio.is_finite() && *ratio <= cap + 1e-6 && *ratio >= -1.0) {
        Some(breaks)
    } else {
        None
    }
}

fn ratios_of(atoms: &[Atom], breaks: &[usize], lengths: &[f64]) -> Vec<f64> {
    let items: Vec<Item> = atoms.iter().map(|atom| atom.item).collect();
    kp::adjustment_ratios(&items, lengths, breaks)
}

fn peak_ratio(atoms: &[Atom], breaks: &[usize], lengths: &[f64]) -> f64 {
    let items: Vec<Item> = atoms.iter().map(|atom| atom.item).collect();
    kp::adjustment_ratios(&items, lengths, breaks)
        .into_iter()
        .filter(|ratio| ratio.is_finite() && *ratio > 0.0)
        .fold(0.0, f64::max)
}

fn tightest(atoms: Vec<Atom>, lengths: &[f64], em: f64) -> (Vec<Atom>, Vec<Span>) {
    let mut best_breaks = None;
    let mut best_peak = f64::INFINITY;
    if let Ok(breaks) = break_at(&atoms, lengths, f64::INFINITY) {
        let ratios = ratios_of(&atoms, &breaks, lengths);
        if ratios.iter().all(|ratio| ratio.is_finite()) {
            best_peak = peak_ratio(&atoms, &breaks, lengths);
            best_breaks = Some(breaks);
        }
    }
    for extra in [em, em * 2.0] {
        let loosened = loosen_glues(&atoms, extra);
        let Ok(breaks) = break_at(&loosened, lengths, VISUAL_RATIO_CAP) else {
            continue;
        };
        if !ratios_of(&atoms, &breaks, lengths).iter().all(|ratio| ratio.is_finite()) {
            continue;
        }
        let peak = peak_ratio(&atoms, &breaks, lengths);
        if peak < best_peak {
            best_breaks = Some(breaks);
        }
        break;
    }
    if let Some(breaks) = best_breaks {
        return spans_from(atoms, breaks, lengths, false);
    }
    let breaks = emergency_breaks(&atoms, lengths);
    spans_from(atoms, breaks, lengths, false)
}

fn spans_from(atoms: Vec<Atom>, breaks: Vec<usize>, lengths: &[f64], fill: bool) -> (Vec<Atom>, Vec<Span>) {
    let items: Vec<Item> = atoms.iter().map(|atom| atom.item).collect();
    let ratios = if !fill && breaks.len() >= 2 {
        kp::adjustment_ratios(&items, lengths, &breaks)
    } else {
        Vec::new()
    };
    let spans = breaks
        .windows(2)
        .enumerate()
        .map(|(n, window)| {
            let end = window[1];
            let start_break = if n == 0 { -1 } else { window[0] as isize };
            let start = kp::line_content_start(&items, start_break, end);
            let ideal = lengths[n.min(lengths.len() - 1)];
            let ragged = end > 0 && items[end].kind == Kind::Penalty && items[end - 1].stretch >= 500.0;
            let ratio = if fill && !ragged {
                fill_ratio(&atoms, start, end, ideal)
            } else if fill {
                0.0
            } else {
                ratios.get(n).copied().unwrap_or(0.0)
            };
            Span {
                start,
                end,
                ratio,
                first: n == 0,
                ragged,
                hyphenated: items.get(end).is_some_and(|item| item.flagged),
                ideal,
                fill: fill && !ragged,
            }
        })
        .collect();
    (atoms, spans)
}

fn fill_ratio(atoms: &[Atom], start: usize, end: usize, ideal: f64) -> f64 {
    if start > end || end >= atoms.len() {
        return 0.0;
    }
    let slack = ideal - natural_width(atoms, start, end);
    if slack <= 0.5 {
        return 0.0;
    }
    let mut last_box = start;
    for index in start..=end {
        if atoms[index].item.kind == Kind::Box && !atoms[index].glyphs.is_empty() {
            last_box = index;
        }
    }
    let mut stretch = 0.0;
    for index in start..last_box {
        let item = atoms[index].item;
        if item.kind == Kind::Glue && item.width == 0.0 && item.stretch > 0.0 && index != start {
            stretch += item.stretch;
        }
    }
    if stretch <= 0.0 { 0.0 } else { slack / stretch }
}

fn greedy_breaks(atoms: &[Atom], lengths: &[f64]) -> Vec<usize> {
    let items: Vec<Item> = atoms.iter().map(|atom| atom.item).collect();
    let mut out = vec![0usize];
    if items.len() < 2 {
        return out;
    }
    let mut from = 0usize;
    let mut line = 0usize;
    loop {
        let ideal = lengths[line.min(lengths.len() - 1)];
        let mut best = None;
        for b in (from + 1)..items.len() {
            if !feasible(&items, b) {
                continue;
            }
            let start_break = if from == 0 && out.len() == 1 { -1 } else { from as isize };
            let start = kp::line_content_start(&items, start_break, b);
            let width = natural_width(atoms, start, b);
            if width <= ideal + 0.51 {
                best = Some(b);
            } else if best.is_some() {
                break;
            }
        }
        let next = best.unwrap_or_else(|| {
            (from + 1..items.len()).find(|b| feasible(&items, *b)).unwrap_or(items.len() - 1)
        });
        if next <= from {
            break;
        }
        out.push(next);
        if next + 1 >= items.len() {
            break;
        }
        from = next;
        line += 1;
        if line > items.len() {
            break;
        }
    }
    out
}

fn icu_breaks(segmenter: icu_segmenter::LineSegmenterBorrowed<'static>, recs: &[Rec]) -> HashSet<usize> {
    let mut text = String::new();
    let mut ends = Vec::with_capacity(recs.len());
    for rec in recs {
        text.push(rec.ch);
        ends.push(text.len());
    }
    let points: HashSet<usize> = segmenter.segment_str(&text).collect();
    let mut after = HashSet::new();
    for (i, end) in ends.iter().enumerate() {
        if i + 1 < recs.len() && points.contains(end) {
            after.insert(i);
        }
    }
    after
}

fn itemize(
    faces: &Faces,
    hyphen: &Hyphenator,
    recs: &[Rec],
    paint: &Paint,
    allow: &HashSet<usize>,
    hyphenate: bool,
) -> Vec<Atom> {
    let space = space_width(faces, paint.size);
    let shrink = (space - 3.0).max(0.0);
    let hyphen_w = hyphen_atom(faces, paint);
    let mut atoms = Vec::new();
    let mut i = 0;
    while i < recs.len() {
        if recs[i].space {
            let prev_open = previous_glyph(recs, i).is_some_and(|rec| is_open(rec.ch));
            let next_close = recs[i + 1..].iter().find(|rec| !rec.space).is_some_and(|rec| is_close(rec.ch));
            if prev_open || next_close {
                atoms.push(box_atom(space, Vec::new()));
            } else {
                atoms.push(glue_atom(space, space, shrink));
            }
            i += 1;
            continue;
        }
        if let Some(prev) = previous_glyph(recs, i) {
            let boundary_break = prev.cjk != recs[i].cjk
                && !is_open(prev.ch)
                && !is_close(recs[i].ch)
                && !is_ambiguous_quote(prev.ch)
                && !is_ambiguous_quote(recs[i].ch);
            if boundary_break {
                let quarter = recs[i].size as f64 * 0.25;
                // The stretch glue is interior when the penalty is the break, so a short
                // mixed line stays under the 0.75 cap. The quarter-em stays fixed.
                atoms.push(glue_atom(0.0, 1.0, 0.0));
                atoms.push(Atom {
                    item: Item::penalty(0.0, 0.0, false),
                    glyphs: Vec::new(),
                });
                atoms.push(glue_atom(quarter, 0.0, 0.0));
            }
        }
        if recs[i].cjk {
            push_cjk(&mut atoms, &recs[i], i, recs, allow);
            i += 1;
        } else {
            let start = i;
            i += 1;
            while i < recs.len() && !recs[i].cjk && !recs[i].space && !recs[i].forced {
                i += 1;
            }
            push_word(&mut atoms, hyphen, &recs[start..i], hyphenate, &hyphen_w);
        }
    }
    atoms.push(glue_atom(0.0, 1000.0, 0.0));
    atoms.push(Atom {
        item: Item::forced(),
        glyphs: Vec::new(),
    });
    atoms
}

fn previous_glyph(recs: &[Rec], index: usize) -> Option<&Rec> {
    recs[..index].iter().rev().find(|rec| !rec.space && !rec.forced)
}

fn push_cjk(atoms: &mut Vec<Atom>, rec: &Rec, index: usize, recs: &[Rec], allow: &HashSet<usize>) {
    let size = rec.size as f64;
    let aki = size * 0.5;
    let break_before = index > 0 && allow.contains(&(index - 1)) && !recs[index - 1].space;
    let break_after = allow.contains(&index);
    // Curly quotes keep the font's own advance. The half-em aki belongs to
    // the unambiguous brackets, and a quarter-em is not added beside a quote.
    if is_ambiguous_quote(rec.ch) {
        let width = rec.advance as f64;
        atoms.push(box_atom(width, vec![glyph_of(rec, 0.0)]));
        if break_after {
            atoms.push(glue_atom(0.0, 1.0, 0.0));
            atoms.push(Atom {
                item: Item::penalty(0.0, 0.0, false),
                glyphs: Vec::new(),
            });
        }
    } else if is_open(rec.ch) {
        if break_before && !is_close(recs[index - 1].ch) {
            atoms.push(glue_atom(0.0, 1.0, 0.0));
            atoms.push(Atom {
                item: Item::penalty(0.0, 0.0, false),
                glyphs: Vec::new(),
            });
            atoms.push(glue_atom(aki, 0.0, 0.0));
        } else if index > 0 {
            atoms.push(box_atom(aki, Vec::new()));
        }
        atoms.push(box_atom(aki, vec![glyph_of(rec, 0.0)]));
    } else if is_close(rec.ch) {
        atoms.push(box_atom(aki, vec![glyph_of(rec, 0.0)]));
        let next_ok = index + 1 == recs.len() || !is_open(recs[index + 1].ch);
        if break_after && next_ok {
            atoms.push(glue_atom(0.0, 1.0, 0.0));
            atoms.push(glue_atom(aki, 0.0, 0.0));
        } else if index + 1 < recs.len() {
            atoms.push(box_atom(aki, Vec::new()));
        }
    } else {
        let width = rec.advance as f64;
        atoms.push(box_atom(width, vec![glyph_of(rec, 0.0)]));
        let next_is_close = recs[index + 1..]
            .iter()
            .find(|rec| !rec.space && !rec.forced)
            .is_some_and(|rec| is_close(rec.ch));
        if break_after && !next_is_close {
            atoms.push(glue_atom(0.0, 1.0, 0.0));
            atoms.push(Atom {
                item: Item::penalty(0.0, 0.0, false),
                glyphs: Vec::new(),
            });
        }
    }
}

fn push_word(atoms: &mut Vec<Atom>, hyphen: &Hyphenator, recs: &[Rec], hyphenate: bool, hyphen_g: &AtomGlyph) {
    if recs.is_empty() {
        return;
    }
    let chars: Vec<char> = recs.iter().map(|rec| rec.ch).collect();
    let mut points = Vec::new();
    if hyphenate {
        if let Some((start, letters)) = ascii_run(&chars) {
            if letters.len() >= TEX_HYPHEN_LEFT_MIN + TEX_HYPHEN_RIGHT_MIN {
                for point in hyphen.breaks(&letters, TEX_HYPHEN_LEFT_MIN, TEX_HYPHEN_RIGHT_MIN) {
                    points.push(start + point);
                }
            }
        }
    }
    points.push(recs.len());
    let mut prev = 0;
    for point in points {
        if point > prev {
            let slice = &recs[prev..point];
            let mut glyphs: Vec<AtomGlyph> = Vec::new();
            let mut dx = 0.0f32;
            for rec in slice {
                if rec.gid == 0 && rec.advance == 0.0 {
                    if let Some(prev_glyph) = glyphs.last_mut() {
                        if prev_glyph.node == rec.node {
                            prev_glyph.span = prev_glyph.span.saturating_add(1);
                        }
                    }
                    continue;
                }
                glyphs.push(glyph_of(rec, dx));
                dx += rec.advance;
            }
            atoms.push(box_atom(dx as f64, glyphs));
        }
        if point < recs.len() {
            let mut hyphen_glyph = hyphen_g.clone();
            hyphen_glyph.node = recs[point].node;
            hyphen_glyph.offset = recs[point].offset;
            hyphen_glyph.ch = '-';
            hyphen_glyph.span = 0;
            atoms.push(Atom {
                item: Item::penalty(hyphen_glyph.advance as f64, TEX_HYPHEN_PENALTY, true),
                glyphs: vec![hyphen_glyph],
            });
        }
        prev = point;
    }
}

fn ascii_run(chars: &[char]) -> Option<(usize, String)> {
    let mut start = None;
    let mut letters = String::new();
    for (index, ch) in chars.iter().enumerate() {
        if ch.is_ascii_alphabetic() {
            if start.is_none() {
                start = Some(index);
            }
            letters.push(*ch);
        } else if start.is_some() {
            break;
        }
    }
    start.map(|start| (start, letters))
}

fn glyph_of(rec: &Rec, dx: f32) -> AtomGlyph {
    AtomGlyph {
        font: rec.font,
        gid: rec.gid,
        dx: dx + rec.x_offset,
        advance: rec.advance,
        ch: rec.ch,
        node: rec.node,
        offset: rec.offset,
        span: 1,
        size: rec.size,
    }
}

fn glue_atom(width: f64, stretch: f64, shrink: f64) -> Atom {
    Atom {
        item: Item::glue(width, stretch, shrink),
        glyphs: Vec::new(),
    }
}

fn box_atom(width: f64, glyphs: Vec<AtomGlyph>) -> Atom {
    Atom {
        item: Item::box_(width),
        glyphs,
    }
}

fn space_width(faces: &Faces, size: f32) -> f64 {
    let width: f32 = faces
        .shape(
            " ",
            &ShapeOpts {
                italic: false,
                cjk: false,
                size,
                weight: 400,
                small_caps: false,
                oldstyle: true,
            },
        )
        .iter()
        .map(|glyph| glyph.advance)
        .sum();
    if width < 0.5 { size as f64 * 0.25 } else { width as f64 }
}

fn hyphen_atom(faces: &Faces, paint: &Paint) -> AtomGlyph {
    let shaped = faces.shape(
        "-",
        &ShapeOpts {
            italic: paint.italic,
            cjk: false,
            size: paint.size,
            weight: paint.weight,
            small_caps: false,
            oldstyle: true,
        },
    );
    let advance = shaped.iter().map(|glyph| glyph.advance).sum::<f32>().max(1.0);
    AtomGlyph {
        font: if paint.italic { 1 } else { 0 },
        gid: shaped.first().map(|glyph| glyph.gid).unwrap_or(0),
        dx: 0.0,
        advance,
        ch: '-',
        node: 0,
        offset: 0,
        span: 0,
        size: paint.size,
    }
}

struct Placed {
    glyphs: Vec<PlacedGlyph>,
    gaps: Vec<f64>,
    hyphenated: bool,
    kinsoku: bool,
    overflow: bool,
    justified: bool,
}

fn place_line(
    atoms: &[Atom],
    span: &Span,
    left: f32,
    measure: f32,
    baseline: f32,
    paint: &Paint,
    paragraph_start: bool,
) -> Placed {
    let justify = paint.align == Align::Justify;
    // positionItems floors the ratio at -1 and draws anything above that,
    // including a ratio past the 0.75 cap. Zeroing those lines leaves them
    // short of the measure and hides them from the gap figure.
    let used = if span.fill && span.ratio.is_finite() {
        span.ratio.max(0.0)
    } else if justify && span.ratio.is_finite() {
        span.ratio.max(-1.0)
    } else {
        0.0
    };
    let natural = natural_width(atoms, span.start, span.end);
    let indent = if span.first {
        (measure as f64 - span.ideal).max(0.0) as f32
    } else {
        0.0
    };
    let mut x = match paint.align {
        Align::Center => left + (measure - natural as f32) / 2.0,
        Align::Right => left + measure - natural as f32,
        Align::Justify | Align::Left => left + indent,
    };
    let right = left + measure;
    let mut glyphs = Vec::new();
    let mut gaps = Vec::new();
    if span.start <= span.end {
        for index in span.start..=span.end {
            let atom = &atoms[index];
            let at_end = (index..=span.end).filter(|n| !atoms[*n].glyphs.is_empty() && atoms[*n].item.kind == Kind::Box).count() <= 1
                && atom.item.kind == Kind::Box
                && !atom.glyphs.is_empty();
            match atom.item.kind {
                Kind::Box => {
                    for glyph in &atom.glyphs {
                        let mut draw_x = x + glyph.dx;
                        if at_end && !paint.hanging && is_close(glyph.ch) {
                            draw_x = x + atom.item.width as f32 - glyph.advance;
                        }
                        glyphs.push(PlacedGlyph {
                            font: FontId(glyph.font as u16),
                            glyph_id: glyph.gid,
                            x: draw_x,
                            baseline_y: baseline,
                            advance: glyph.advance,
                            node: glyph.node,
                            offset: glyph.offset,
                            span: glyph.span,
                            ch: glyph.ch,
                            size: glyph.size,
                            line_end: false,
                            line_start: false,
                            paragraph_start,
                            hyphen: false,
                        });
                    }
                    x += atom.item.width as f32;
                }
                Kind::Glue if index != span.start && index != span.end => {
                    let width = atom.item.width;
                    // CJK fill only grows the width-zero glues that fill_ratio counted.
                    // A space glue also has stretch, and growing it would push the last
                    // box past the measure.
                    let drawn = if span.fill {
                        if width == 0.0 { width + used * atom.item.stretch } else { width }
                    } else if used >= 0.0 {
                        width + used * atom.item.stretch
                    } else {
                        width + used * atom.item.shrink
                    };
                    // Word gap is the drawn space divided by this font's natural space.
                    // The same set the product scores: a justified line, not the last
                    // line of the paragraph, and at least two word spaces. A forced
                    // break is the end of a segment, so it is last. A single-word line
                    // has no word space and drops out with the two-word line.
                    if counts_word_gap(justify, span.ragged, used, width, atom.item.stretch) {
                        gaps.push(drawn / width);
                    }
                    x += drawn as f32;
                }
                Kind::Penalty if index == span.end && atom.item.flagged => {
                    for glyph in &atom.glyphs {
                        glyphs.push(PlacedGlyph {
                            font: FontId(glyph.font as u16),
                            glyph_id: glyph.gid,
                            x: x + glyph.dx,
                            baseline_y: baseline,
                            advance: glyph.advance,
                            node: glyph.node,
                            offset: glyph.offset,
                            span: 0,
                            ch: '-',
                            size: glyph.size,
                            line_end: false,
                            line_start: false,
                            paragraph_start,
                            hyphen: true,
                        });
                    }
                    x += atom.item.width as f32;
                }
                _ => {}
            }
        }
    }
    if let Some(first) = glyphs.iter_mut().find(|glyph| !glyph.hyphen) {
        first.line_start = true;
    }
    if !span.ragged {
        if let Some(last) = glyphs.iter_mut().rev().find(|glyph| !glyph.hyphen) {
            last.line_end = true;
        }
    }
    let mut kinsoku = false;
    if let Some(first) = glyphs.iter().find(|glyph| !glyph.hyphen) {
        if !paragraph_start && is_close(first.ch) {
            kinsoku = true;
        }
    }
    if let Some(last) = glyphs.iter().rev().find(|glyph| !glyph.hyphen) {
        if is_open(last.ch) {
            kinsoku = true;
        }
    }
    let last_ink = glyphs.iter().rev().find(|glyph| !glyph.hyphen).map(|glyph| glyph.x);
    let mut overflow = false;
    for glyph in &glyphs {
        let ink_right = glyph.x + glyph.advance;
        let last_on_line = last_ink == Some(glyph.x);
        let hanging_ok = paint.hanging && is_close(glyph.ch) && (glyph.line_end || (span.ragged && last_on_line));
        if ink_right > right + 0.8 && !hanging_ok {
            overflow = true;
        }
        if glyph.x < left - 0.8 {
            overflow = true;
        }
    }
    let justified = justify && !span.ragged && gaps.len() >= 2;
    if !justified {
        gaps.clear();
    }
    Placed {
        glyphs,
        gaps,
        hyphenated: span.hyphenated,
        kinsoku,
        overflow,
        justified,
    }
}

fn natural_width(atoms: &[Atom], start: usize, end: usize) -> f64 {
    if start > end || end >= atoms.len() {
        return 0.0;
    }
    let mut width = 0.0;
    for index in start..=end {
        let item = atoms[index].item;
        if item.kind == Kind::Box {
            width += item.width;
        } else if item.kind == Kind::Glue && index != start && index != end {
            width += item.width;
        } else if item.kind == Kind::Penalty && index == end {
            width += item.width;
        }
    }
    width
}

fn emergency_breaks(atoms: &[Atom], lengths: &[f64]) -> Vec<usize> {
    let items: Vec<Item> = atoms.iter().map(|atom| atom.item).collect();
    let mut out = vec![0usize];
    let mut from = 0usize;
    let mut line = 0usize;
    while from + 1 < items.len() {
        let ideal = lengths[line.min(lengths.len() - 1)];
        let mut best = None;
        for b in (from + 1)..items.len() {
            if !feasible(&items, b) {
                continue;
            }
            let width = natural_width(atoms, content_start(&items, from, b), b);
            if width <= ideal + 0.75 {
                best = Some(b);
            } else if best.is_some() {
                break;
            }
        }
        let next = best.unwrap_or_else(|| {
            ((from + 1)..items.len()).find(|b| feasible(&items, *b)).unwrap_or(items.len() - 1)
        });
        if next <= from {
            break;
        }
        out.push(next);
        if next + 1 >= items.len() {
            break;
        }
        from = next;
        line += 1;
    }
    if *out.last().unwrap_or(&0) != items.len().saturating_sub(1) {
        out.push(items.len().saturating_sub(1));
    }
    out
}

fn content_start(items: &[Item], breakpoint: usize, end: usize) -> usize {
    if breakpoint == 0 && end > 0 && items[0].kind == Kind::Box {
        return 0;
    }
    let mut start = breakpoint;
    if start < items.len() && items[start].kind == Kind::Penalty {
        start += 1;
    }
    while start < items.len() && start <= end && items[start].kind != Kind::Box {
        start += 1;
    }
    start.min(end)
}

fn feasible(items: &[Item], b: usize) -> bool {
    let item = items[b];
    (item.kind == Kind::Glue && b > 0 && items[b - 1].kind == Kind::Box)
        || (item.kind == Kind::Penalty && item.cost < 1000.0)
}

/// A word gap counts when the line is justified, it is not the last line of
/// its paragraph or of a forced break, and the glue is a word space.
/// The line itself is dropped unless it has at least two such spaces, which
/// is the product's `gaps.length < 2` cut. A single-word line has none.
fn counts_word_gap(justify: bool, last_line: bool, used: f64, width: f64, stretch: f64) -> bool {
    justify && !last_line && used > 0.0 && width > 1.0 && (stretch - width).abs() < 0.05
}

fn percentile_95(values: &mut [f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let index = ((values.len() as f64) * 0.95).ceil() as usize - 1;
    values[index.min(values.len() - 1)]
}

fn tie_ambiguous_quotes(recs: &mut [Rec]) {
    for i in 0..recs.len() {
        if recs[i].space || recs[i].forced || !is_ambiguous_quote(recs[i].ch) {
            continue;
        }
        if neighbor_is_cjk(recs, i, false) || neighbor_is_cjk(recs, i, true) {
            recs[i].cjk = true;
            recs[i].font = 2;
        }
    }
}

fn neighbor_is_cjk(recs: &[Rec], index: usize, forward: bool) -> bool {
    let mut cursor = index;
    loop {
        if forward {
            cursor += 1;
            if cursor >= recs.len() {
                return false;
            }
        } else if cursor == 0 {
            return false;
        } else {
            cursor -= 1;
        }
        if recs[cursor].space || recs[cursor].forced || is_ambiguous_quote(recs[cursor].ch) {
            continue;
        }
        return recs[cursor].cjk;
    }
}

fn is_ambiguous_quote(ch: char) -> bool {
    matches!(ch, '\u{2018}' | '\u{2019}' | '\u{201C}' | '\u{201D}')
}

fn cjk_char(ch: char, cjk_book: bool) -> bool {
    if is_ambiguous_quote(ch) {
        return false;
    }
    if is_open(ch) || is_close(ch) {
        return cjk_book;
    }
    matches!(ch,
        '\u{3040}'..='\u{30FF}' | '\u{3400}'..='\u{9FFF}' | '\u{AC00}'..='\u{D7AF}' | '\u{F900}'..='\u{FAFF}' | '\u{FF00}'..='\u{FFEF}'
    )
}

fn is_open(ch: char) -> bool {
    OPEN.contains(ch)
}

fn is_close(ch: char) -> bool {
    CLOSE.contains(ch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icu_strict_breaks_inside_han_and_keeps_closing_marks() {
        let mut line_breaks = LineBreakOptions::default();
        line_breaks.strictness = Some(LineBreakStrictness::Strict);
        line_breaks.word_option = Some(LineBreakWordOption::Normal);
        let segmenter = LineSegmenter::new_auto(line_breaks);
        let han: Vec<usize> = segmenter.segment_str("漢字").collect();
        assert_eq!(han, vec![0, 3, 6], "han breaks {han:?}");
        let hello: Vec<usize> = segmenter.segment_str("Hello World").collect();
        assert_eq!(hello, vec![0, 6, 11]);
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;
    use crate::book::{self, load_chapter};

    fn fonts() -> Option<Typesetter> {
        if !book::fonts_present() {
            return None;
        }
        let dir = book::fixture_dir();
        Some(
            Typesetter::open(
                &format!("{dir}/EBGaramond-Regular.ttf"),
                &format!("{dir}/EBGaramond-Italic.ttf"),
                &format!("{dir}/NotoSerifCJKsc-Regular.otf"),
            )
            .unwrap(),
        )
    }

    #[test]
    fn pride_chapter_one_breakpoints_match_the_javascript_breaker() {
        let Some(setter) = fonts() else {
            eprintln!("fixtures missing");
            return;
        };
        let chapter = load_chapter(
            &format!("{}/pride-se.epub", book::fixture_dir()),
            "chapter-1.xhtml",
        )
        .unwrap();
        let prepared = prepare(&chapter.xhtml, &chapter.stylesheet, None);
        let mut line_breaks = LineBreakOptions::default();
        line_breaks.strictness = Some(LineBreakStrictness::Strict);
        line_breaks.word_option = Some(LineBreakWordOption::Normal);
        let segmenter = LineSegmenter::new_auto(line_breaks);
        let hyphen = Hyphenator::load();
        let mut compared = 0;
        for block in &prepared.blocks {
            let recs = shape_block(&setter.faces, block, prepared.cjk);
            for (seg_i, seg) in split_forced(&recs).into_iter().enumerate() {
                if seg.is_empty() {
                    continue;
                }
                let indent = if seg_i == 0 { block.paint.indent } else { 0.0 };
                let measure = frame_for(false, None).measure;
                let mut lengths = vec![measure as f64; 512];
                lengths[0] = (measure as f64 - indent as f64).max(1.0);
                let allow = icu_breaks(segmenter, seg);
                let plain = itemize(&setter.faces, &hyphen, seg, &block.paint, &allow, false);
                let plain_items: Vec<Item> = plain.iter().map(|atom| atom.item).collect();
                let rust = kp::break_lines(&plain_items, &lengths, BreakOpts::product());
                let js = kp::javascript_breaks(&plain_items, &lengths, BreakOpts::product());
                assert_eq!(rust.is_ok(), js.is_ok(), "throw mismatch on paragraph {compared}");
                if let (Ok(rust), Ok(js)) = (&rust, &js) {
                    assert_eq!(rust, js, "paragraph {compared}");
                }
                if rust.is_err() && block.paint.hyphens {
                    let with = itemize(&setter.faces, &hyphen, seg, &block.paint, &allow, true);
                    let items: Vec<Item> = with.iter().map(|atom| atom.item).collect();
                    let rust = kp::break_lines(&items, &lengths, BreakOpts::product());
                    let js = kp::javascript_breaks(&items, &lengths, BreakOpts::product());
                    assert_eq!(rust.is_ok(), js.is_ok());
                    if let (Ok(rust), Ok(js)) = (rust, js) {
                        assert_eq!(rust, js, "hyphen paragraph {compared}");
                    }
                }
                compared += 1;
            }
        }
        assert!(compared > 20, "compared {compared}");
    }

    #[test]
    fn fi_ligature_width_equals_the_harfrust_advances() {
        let Some(setter) = fonts() else {
            return;
        };
        let shaped = setter.faces.shape(
            "first",
            &crate::face::ShapeOpts {
                italic: false,
                cjk: false,
                size: 22.0,
                weight: 400,
                small_caps: false,
                oldstyle: true,
            },
        );
        let shaped_sum: f32 = shaped.iter().map(|glyph| glyph.advance).sum();
        let layout = setter
            .compose(ChapterInput {
                xhtml: "<html><body><p style=\"text-align:left;text-indent:0\">first</p></body></html>",
                stylesheet: "",
                base_href: "",
                measure: None,
                font_size: None,
            })
            .unwrap();
        let placed_sum: f32 = layout
            .glyph_runs(0)
            .into_iter()
            .flat_map(|run| run.glyphs)
            .filter(|glyph| glyph.glyph_id != 0)
            .map(|glyph| glyph.advance)
            .sum();
        assert!(
            (placed_sum - shaped_sum).abs() < 0.5,
            "placed {placed_sum} shaped {shaped_sum} glyphs {}",
            shaped.len()
        );
    }
}
