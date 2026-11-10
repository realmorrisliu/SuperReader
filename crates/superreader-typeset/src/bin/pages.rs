use superreader_typeset::{
    load_chapter, BreakTune, ChapterInput, Typesetter, PAPERBACK_HEIGHT, PAPERBACK_WIDTH,
};
use tiny_skia::Pixmap;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut epub = String::new();
    let mut href = String::new();
    let mut latin = String::new();
    let mut italic = String::new();
    let mut cjk = String::new();
    let mut out = String::new();
    let mut pages = 1u32;
    let mut find = String::new();
    let mut repeats = 5u32;
    let mut measure = None;
    let mut font_size = None;
    let mut tune = BreakTune::default();
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        let value = args.get(i + 1).map(String::as_str).unwrap_or("");
        match arg {
            "--tex-shrink" => {
                tune.tex_shrink = true;
                i += 1;
            }
            "--tolerance" => {
                tune.tolerance = true;
                i += 1;
            }
            "--font-expand" => {
                tune.font_expand = value.parse().unwrap_or(0.0);
                i += 2;
            }
            "--letter-space" => {
                tune.letter_space = value.parse().unwrap_or(0.0);
                i += 2;
            }
            "--epub" => {
                epub = value.to_string();
                i += 2;
            }
            "--href" => {
                href = value.to_string();
                i += 2;
            }
            "--latin" => {
                latin = value.to_string();
                i += 2;
            }
            "--italic" => {
                italic = value.to_string();
                i += 2;
            }
            "--cjk" => {
                cjk = value.to_string();
                i += 2;
            }
            "--out" => {
                out = value.to_string();
                i += 2;
            }
            "--pages" => {
                pages = value.parse().unwrap_or(1);
                i += 2;
            }
            "--find" => {
                find = value.to_string();
                i += 2;
            }
            "--repeats" => {
                repeats = value.parse().unwrap_or(5);
                i += 2;
            }
            "--measure" => {
                measure = value.parse().ok();
                i += 2;
            }
            "--size" => {
                font_size = value.parse().ok();
                i += 2;
            }
            _ => i += 1,
        }
    }
    let loaded = load_chapter(&epub, &href).unwrap_or_else(|err| panic!("{err}"));
    let setter = Typesetter::open(&latin, &italic, &cjk).unwrap();
    let input = ChapterInput {
        xhtml: &loaded.xhtml,
        stylesheet: &loaded.stylesheet,
        base_href: &loaded.href,
        measure,
        font_size,
        tune,
    };
    let cold_at = std::time::Instant::now();
    let cold = setter.compose(input).unwrap();
    let cold_ms = cold_at.elapsed().as_secs_f64() * 1000.0;
    let mut warm = Vec::new();
    let mut last = cold;
    for _ in 0..repeats {
        let started = std::time::Instant::now();
        last = setter.compose(input).unwrap();
        warm.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    warm.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let median = warm[warm.len() / 2];
    let report = last.report();
    std::fs::create_dir_all(&out).ok();
    let mut written = Vec::new();
    if find.is_empty() {
        for page in 0..pages {
            if let Some(bytes) = setter.png(&last, page) {
                let path = format!("{out}/page-{}.png", page + 1);
                std::fs::write(&path, bytes).unwrap();
                written.push(path);
            }
        }
        if pages >= 2 {
            if let Some(path) =
                side_by_side(&setter, &last, 0, 1, &format!("{out}/side-by-side.png"))
            {
                written.push(path);
            }
        }
    } else {
        let page = page_with(&loaded.xhtml, &last, &find).unwrap_or(0);
        if let Some(bytes) = setter.png(&last, page) {
            let path = format!("{out}/page.png");
            std::fs::write(&path, bytes).unwrap();
            written.push(path);
        }
    }
    let width = last.content_box(0).right - last.content_box(0).left;
    let stats = last.line_stats();
    let body = serde_like(
        report,
        &loaded.href,
        cold_ms,
        median,
        warm[0],
        *warm.last().unwrap(),
        &written,
        width,
        font_size,
        stats,
    );
    std::fs::write(format!("{out}/metrics.json"), body).unwrap();
    println!(
        "{}",
        std::fs::read_to_string(format!("{out}/metrics.json")).unwrap()
    );
}

fn page_with(
    xhtml: &str,
    layout: &superreader_typeset::ChapterLayout,
    needle: &str,
) -> Option<u32> {
    let _ = xhtml;
    for page in 0..layout.report().page_count {
        let text: String = layout
            .glyph_runs(page)
            .iter()
            .flat_map(|run| run.glyphs.iter().map(|glyph| glyph.ch))
            .collect();
        if text.contains(needle) {
            return Some(page);
        }
    }
    None
}

fn side_by_side(
    setter: &Typesetter,
    layout: &superreader_typeset::ChapterLayout,
    a: u32,
    b: u32,
    path: &str,
) -> Option<String> {
    let left = setter.png(layout, a)?;
    let right = setter.png(layout, b)?;
    let left = Pixmap::decode_png(&left).ok()?;
    let right = Pixmap::decode_png(&right).ok()?;
    let gap = 24u32;
    let mut canvas = Pixmap::new(PAPERBACK_WIDTH as u32 * 2 + gap, PAPERBACK_HEIGHT as u32)?;
    canvas.fill(tiny_skia::Color::from_rgba8(0x2a, 0x27, 0x23, 0xff));
    canvas.draw_pixmap(
        0,
        0,
        left.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        tiny_skia::Transform::identity(),
        None,
    );
    canvas.draw_pixmap(
        PAPERBACK_WIDTH as i32 + gap as i32,
        0,
        right.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        tiny_skia::Transform::identity(),
        None,
    );
    std::fs::write(path, canvas.encode_png().ok()?).ok()?;
    Some(path.to_string())
}

fn json_str(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

#[allow(clippy::too_many_arguments)]
fn serde_like(
    report: &superreader_typeset::LayoutReport,
    href: &str,
    cold_ms: f64,
    median: f64,
    min: f64,
    max: f64,
    files: &[String],
    measure: f32,
    font_size: Option<f32>,
    stats: superreader_typeset::LineStats,
) -> String {
    let files = files
        .iter()
        .map(|path| json_str(path))
        .collect::<Vec<_>>()
        .join(",");
    let size = font_size
        .map(|size| format!("{size:.2}"))
        .unwrap_or_else(|| "null".to_string());
    format!(
        "{{\n  \"href\": {},\n  \"pages\": {},\n  \"measure\": {measure:.2},\n  \"font_size\": {size},\n  \"cold_ms\": {cold_ms:.3},\n  \"warm_ms\": {median:.3},\n  \"warm_min\": {min:.3},\n  \"warm_max\": {max:.3},\n  \"gap_p95\": {:.4},\n  \"gap_max\": {:.4},\n  \"hyphenated\": {},\n  \"lines\": {},\n  \"lines_over_cap\": {},\n  \"scored_lines\": {},\n  \"hyphen_run\": {},\n  \"kinsoku\": {},\n  \"overflow\": {},\n  \"shape_ms\": {:.3},\n  \"break_ms\": {:.3},\n  \"files\": [{files}]\n}}\n",
        json_str(href),
        report.page_count,
        report.word_gap_p95,
        report.word_gap_max,
        report.hyphenated_lines,
        report.line_count,
        stats.lines_over_cap,
        stats.scored_lines,
        stats.longest_hyphen_run,
        report.kinsoku_violations,
        report.overflow_lines,
        report.shape_ms,
        report.break_ms
    )
}
