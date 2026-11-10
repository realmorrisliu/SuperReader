use std::time::Instant;

use epub_native_spike::{
    encode, parse_chapter_opts, side_by_side, Choices, Engine, Hyphenation, Punctuation, PAGE_H, PAGE_W,
};
use rbook::Epub;

fn main() {
    let mut args = std::env::args().skip(1);
    let out_dir = args.next().unwrap_or_else(|| "artifacts".into());
    let docs_dir = args.next().unwrap_or_else(|| "docs/spike/epub-native".into());
    std::fs::create_dir_all(&out_dir).expect("out dir");
    std::fs::create_dir_all(&docs_dir).expect("docs dir");

    let latin = "/tmp/superreader-fixtures/EBGaramond-Regular.ttf";
    let italic = "/tmp/superreader-fixtures/EBGaramond-Italic.ttf";
    let cjk = "/tmp/superreader-fixtures/NotoSerifCJKsc-Regular.otf";
    let opened = Instant::now();
    let engine = Engine::open(latin, italic, cjk).expect("fonts");
    let font_ms = opened.elapsed().as_secs_f64() * 1000.0;

    let pride_path = "/tmp/superreader-fixtures/pride-se.epub";
    let zhao_path = "/tmp/superreader-fixtures/zhaohua.epub";
    let pride = load_chapter(pride_path, |href, html| href.contains("chapter-1.xhtml") && html.contains("truth universally acknowledged"));
    let zhao = load_chapter(zhao_path, |_, html| html.contains("小引"));

    let product = Choices::product();
    let pride_doc = parse_chapter_opts(&pride.html, &pride.css, false, false);
    let zhao_doc = parse_chapter_opts(&zhao.html, &zhao.css, true, false);
    let zhao_chrome = parse_chapter_opts(&zhao.html, &zhao.css, true, true);

    let cold_started = Instant::now();
    let pride_cold = engine.layout_doc(&pride_doc, product, pride.spine_step, false);
    let pride_cold_ms = cold_started.elapsed().as_secs_f64() * 1000.0;
    let mut pride_warm = Vec::new();
    let mut pride_laid = pride_cold;
    for _ in 0..5 {
        let started = Instant::now();
        pride_laid = engine.layout_doc(&pride_doc, product, pride.spine_step, false);
        pride_warm.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    let zhao_started = Instant::now();
    let mut zhao_laid = engine.layout_doc(&zhao_doc, product, zhao.spine_step, true);
    let zhao_cold_ms = zhao_started.elapsed().as_secs_f64() * 1000.0;
    let mut zhao_warm = Vec::new();
    for _ in 0..5 {
        let started = Instant::now();
        zhao_laid = engine.layout_doc(&zhao_doc, product, zhao.spine_step, true);
        zhao_warm.push(started.elapsed().as_secs_f64() * 1000.0);
    }

    let pride_stats = pride_laid.stats();
    let zhao_stats = zhao_laid.stats();
    let mut turn = Vec::new();
    let mut paint_only = Vec::new();
    for _ in 0..5 {
        let started = Instant::now();
        let pixmap = pride_laid.pixmap(engine.faces(), 0);
        paint_only.push(started.elapsed().as_secs_f64() * 1000.0);
        let started = Instant::now();
        let _ = encode(&pixmap);
        turn.push(paint_only.last().copied().unwrap_or(0.0) + started.elapsed().as_secs_f64() * 1000.0);
    }

    let phrase = pride_laid.find_phrase("truth universally acknowledged");
    let mut cfi_ok = false;
    if let Some((start, end)) = phrase {
        if let Some(cfi) = pride_laid.cfi(start) {
            cfi_ok = pride_laid.caret_from_cfi(&cfi) == Some(start);
            let highlight_page = pride_laid.page_of(start).unwrap_or(0);
            let rects = pride_laid.rects(highlight_page, start, end);
            let pixmap = pride_laid.paint(engine.faces(), highlight_page, &rects);
            write_both(&out_dir, &docs_dir, "pride-highlight.png", &encode(&pixmap));
            let _ = end;
        }
    }
    let pride_page = pride_laid.pixmap(engine.faces(), 0);
    write_both(&out_dir, &docs_dir, "pride-page1.png", &encode(&pride_page));

    let xiaoyin = zhao_laid.find_phrase("小引").and_then(|(caret, _)| zhao_laid.page_of(caret)).unwrap_or(0);
    let zhao_page = zhao_laid.pixmap(engine.faces(), xiaoyin);
    write_both(&out_dir, &docs_dir, "zhaohua-xiaoyin.png", &encode(&zhao_page));

    let punct = [
        ("full width", Punctuation::Full),
        ("jlreq", Punctuation::Jlreq),
        ("jlreq hang", Punctuation::Hang),
    ];
    let mut punct_panels = Vec::new();
    let mut punct_pix = Vec::new();
    for (label, mode) in punct {
        let choices = Choices { punctuation: mode, ..product };
        let laid = engine.layout_doc(&zhao_doc, choices, zhao.spine_step, true);
        let page = laid.find_phrase("小引").and_then(|(caret, _)| laid.page_of(caret)).unwrap_or(0);
        punct_pix.push(laid.pixmap(engine.faces(), page));
        punct_panels.push(label);
    }
    let punct_refs: Vec<(String, &tiny_skia::Pixmap)> = punct_panels
        .iter()
        .zip(punct_pix.iter())
        .map(|(label, pixmap)| ((*label).to_string(), pixmap))
        .collect();
    let sheet = side_by_side(engine.faces(), &punct_refs);
    write_both(&out_dir, &docs_dir, "choices-punctuation.png", &encode(&sheet));

    let spaces = [("0", 0.0), ("1/8 em", 0.125), ("1/4 em", 0.25)];
    let mut space_pix = Vec::new();
    let mut space_labels = Vec::new();
    for (label, em) in spaces {
        let choices = Choices {
            cjk_latin_em: em,
            punctuation: Punctuation::Jlreq,
            hyphenation: Hyphenation::Off,
        };
        let laid = engine.layout_doc(&zhao_chrome, choices, zhao.spine_step, true);
        space_pix.push(laid.pixmap(engine.faces(), 0));
        space_labels.push(label);
    }
    let space_refs: Vec<(String, _)> = space_labels
        .iter()
        .zip(space_pix.iter())
        .map(|(label, pixmap)| ((*label).to_string(), pixmap))
        .collect();
    let sheet = side_by_side(engine.faces(), &space_refs);
    write_both(&out_dir, &docs_dir, "choices-spacing.png", &encode(&sheet));

    let off = engine.layout_doc(
        &pride_doc,
        Choices { hyphenation: Hyphenation::Off, ..product },
        pride.spine_step,
        false,
    );
    let moderate = engine.layout_doc(&pride_doc, product, pride.spine_step, false);
    let tex = engine.layout_doc(
        &pride_doc,
        Choices { hyphenation: Hyphenation::Tex, ..product },
        pride.spine_step,
        false,
    );
    let mut hy_page = 0usize;
    for index in 0..moderate.page_count().min(tex.page_count()) {
        let a = moderate.pixmap(engine.faces(), index);
        let b = tex.pixmap(engine.faces(), index);
        if ink_delta(&a, &b) > 30 {
            hy_page = index;
            break;
        }
    }
    let page_no = hy_page + 1;
    let hy_labels = [
        format!("off, page {page_no}"),
        format!("moderate, page {page_no}"),
        format!("tex, page {page_no}"),
    ];
    let hy_pix = vec![
        off.pixmap(engine.faces(), hy_page.min(off.page_count().saturating_sub(1))),
        moderate.pixmap(engine.faces(), hy_page),
        tex.pixmap(engine.faces(), hy_page.min(tex.page_count().saturating_sub(1))),
    ];
    let hy_rates = [
        rate("off", &off),
        rate("moderate", &moderate),
        rate("tex", &tex),
    ];
    let hy_refs: Vec<(String, _)> = hy_labels
        .iter()
        .zip(hy_pix.iter())
        .map(|(label, pixmap)| ((*label).to_string(), pixmap))
        .collect();
    let sheet = side_by_side(engine.faces(), &hy_refs);
    write_both(&out_dir, &docs_dir, "choices-hyphenation.png", &encode(&sheet));

    let pride_warm_sorted = sorted(&pride_warm);
    let zhao_warm_sorted = sorted(&zhao_warm);
    let turn_sorted = sorted(&turn);
    let report = format!(
        "{{\n  \"font_load_ms\": {font_ms:.2},\n  \"page\": [{PAGE_W}, {PAGE_H}],\n  \"pride\": {{\n    \"href\": \"{}\",\n    \"spine_step\": {},\n    \"cold_layout_ms\": {pride_cold_ms:.2},\n    \"warm_ms\": {pride_warm:?},\n    \"warm_median_ms\": {:.2},\n    \"warm_min_ms\": {:.2},\n    \"warm_max_ms\": {:.2},\n    \"items_ms\": {:.2},\n    \"break_ms\": {:.2},\n    \"pages\": {},\n    \"lines\": {},\n    \"hyphenated_lines\": {},\n    \"gap_p95\": {:.4},\n    \"gap_max\": {:.4},\n    \"overflow_lines\": {},\n    \"kinsoku\": {},\n    \"missing_glyphs\": {},\n    \"honored\": {},\n    \"dropped\": {},\n    \"cfi_roundtrip\": {cfi_ok}\n  }},\n  \"zhaohua\": {{\n    \"href\": \"{}\",\n    \"cold_layout_ms\": {zhao_cold_ms:.2},\n    \"warm_ms\": {zhao_warm:?},\n    \"warm_median_ms\": {:.2},\n    \"warm_min_ms\": {:.2},\n    \"warm_max_ms\": {:.2},\n    \"items_ms\": {:.2},\n    \"break_ms\": {:.2},\n    \"pages\": {},\n    \"lines\": {},\n    \"overflow_lines\": {},\n    \"kinsoku\": {},\n    \"missing_glyphs\": {},\n    \"xiaoyin_page\": {xiaoyin},\n    \"honored\": {},\n    \"dropped\": {}\n  }},\n  \"page_turn_ms\": {turn:?},\n  \"page_turn_median_ms\": {:.2},\n  \"hyphen_choices\": {hy_rates:?}\n}}\n",
        pride.href,
        pride.spine_step,
        median(&pride_warm_sorted),
        pride_warm_sorted.first().copied().unwrap_or(0.0),
        pride_warm_sorted.last().copied().unwrap_or(0.0),
        pride_laid.items_ms,
        pride_laid.break_ms,
        pride_stats.pages,
        pride_stats.lines,
        pride_stats.hyphenated_lines,
        pride_stats.gap_p95,
        pride_stats.gap_max,
        pride_stats.overflow_lines,
        pride_stats.kinsoku_violations,
        pride_stats.missing_glyphs,
        json_strings(&pride_laid.honored),
        json_strings(&pride_laid.dropped),
        zhao.href,
        median(&zhao_warm_sorted),
        zhao_warm_sorted.first().copied().unwrap_or(0.0),
        zhao_warm_sorted.last().copied().unwrap_or(0.0),
        zhao_laid.items_ms,
        zhao_laid.break_ms,
        zhao_stats.pages,
        zhao_stats.lines,
        zhao_stats.overflow_lines,
        zhao_stats.kinsoku_violations,
        zhao_stats.missing_glyphs,
        json_strings(&zhao_laid.honored),
        json_strings(&zhao_laid.dropped),
        median(&turn_sorted),
    );
    std::fs::write(format!("{out_dir}/metrics.json"), &report).expect("metrics");
    println!("{report}");
    eprintln!("hyphen sheet page {}", hy_page + 1);
    eprintln!("pride problems:");
    for line in pride_laid.problems() {
        eprintln!("  {line}");
    }
    eprintln!("zhaohua problems:");
    for line in zhao_laid.problems() {
        eprintln!("  {line}");
    }
    let paint_sorted = sorted(&paint_only);
    eprintln!(
        "paint median {:.2} range {:.2}..{:.2}",
        median(&paint_sorted),
        paint_sorted.first().copied().unwrap_or(0.0),
        paint_sorted.last().copied().unwrap_or(0.0)
    );
    if !cfi_ok {
        eprintln!("cfi roundtrip failed");
        std::process::exit(1);
    }
}

struct ChapterFile {
    href: String,
    html: String,
    css: String,
    spine_step: u32,
}

fn load_chapter(path: &str, pred: impl Fn(&str, &str) -> bool) -> ChapterFile {
    let epub = Epub::open(path).unwrap_or_else(|err| panic!("{path}: {err}"));
    let mut css = String::new();
    for entry in epub.manifest().styles() {
        if let Ok(text) = entry.read_str() {
            css.push_str(&text);
            css.push('\n');
        }
    }
    let mut reader = epub.reader();
    let mut index = 0u32;
    while let Some(item) = reader.read_next() {
        let data = item.unwrap_or_else(|err| panic!("{err}"));
        let href = data.manifest_entry().href().to_string();
        let html = data.content().to_string();
        if pred(&href, &html) {
            return ChapterFile {
                href,
                html,
                css,
                spine_step: (index + 1) * 2,
            };
        }
        index += 1;
    }
    panic!("chapter not found in {path}");
}

fn write_both(out_dir: &str, docs_dir: &str, name: &str, bytes: &[u8]) {
    std::fs::write(format!("{out_dir}/{name}"), bytes).expect(name);
    std::fs::write(format!("{docs_dir}/{name}"), bytes).expect(name);
}

fn sorted(values: &[f64]) -> Vec<f64> {
    let mut copy = values.to_vec();
    copy.sort_by(|a, b| a.total_cmp(b));
    copy
}

fn median(sorted: &[f64]) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    sorted[sorted.len() / 2]
}

fn json_strings(values: &[String]) -> String {
    let body = values.iter().map(|value| format!("\"{}\"", value.replace('"', "\\\""))).collect::<Vec<_>>().join(", ");
    format!("[{body}]")
}

fn rate(label: &str, laid: &epub_native_spike::LaidOut) -> (String, usize, usize, f32, usize) {
    let stats = laid.stats();
    (
        label.to_string(),
        stats.hyphenated_lines,
        stats.lines,
        stats.gap_p95,
        stats.overflow_lines,
    )
}

fn ink_delta(a: &tiny_skia::Pixmap, b: &tiny_skia::Pixmap) -> u32 {
    if a.width() != b.width() || a.height() != b.height() {
        return u32::MAX;
    }
    a.data()
        .chunks(4)
        .zip(b.data().chunks(4))
        .filter(|(p, q)| {
            let ps = p[0] as u16 + p[1] as u16 + p[2] as u16;
            let qs = q[0] as u16 + q[1] as u16 + q[2] as u16;
            (ps < 500) != (qs < 500)
        })
        .count() as u32
}
