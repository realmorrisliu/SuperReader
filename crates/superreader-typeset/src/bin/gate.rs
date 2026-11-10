//! Phase 1 check for one chapter.
//!
//! Lays the chapter out on strict Knuth-Plass and compares its all-lines
//! word-gap p95 with the reader's all-lines p95. `--font-expand` and
//! `--letter-space` fail the gate. `--list-faces` prints the faces in a file.

use std::fs;
use std::time::Instant;

use superreader_typeset::{
    list_faces, load_chapter, phase1, BreakTune, ChapterInput, Phase1Failure, Phase1Input, Typesetter,
};
use tiny_skia::Pixmap;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if let Some(path) = flag_value(&args, "--list-faces") {
        match list_faces(&path) {
            Ok(faces) => {
                for (index, name) in faces {
                    println!("{index}\t{name}");
                }
            }
            Err(err) => {
                eprintln!("{err}");
                std::process::exit(2);
            }
        }
        return;
    }

    let epub = required(&args, "--epub");
    let href = required(&args, "--href");
    let latin = required(&args, "--latin");
    let italic = required(&args, "--italic");
    let cjk = required(&args, "--cjk");
    let reader_path = required(&args, "--reader");
    let out = required(&args, "--out");
    let measure: f32 = required(&args, "--measure").parse().unwrap_or_else(|_| usage("bad --measure"));
    let size: f32 = required(&args, "--size").parse().unwrap_or_else(|_| usage("bad --size"));
    let latin_index = number(&args, "--latin-index");
    let italic_index = number(&args, "--italic-index");
    let cjk_index = number(&args, "--cjk-index");
    let repeats = number(&args, "--repeats").max(1);
    let mut tune = BreakTune::default();
    if let Some(value) = flag_value(&args, "--font-expand") {
        tune.font_expand = value.parse().unwrap_or(0.0);
    }
    if let Some(value) = flag_value(&args, "--letter-space") {
        tune.letter_space = value.parse().unwrap_or(0.0);
    }
    let png_path = flag_value(&args, "--png");
    let beside = flag_value(&args, "--beside");
    let side_path = flag_value(&args, "--side");

    let reader_text = fs::read_to_string(&reader_path).unwrap_or_else(|err| {
        eprintln!("reader {reader_path}: {err}");
        std::process::exit(2);
    });
    let reader = parse_reader(&reader_text);
    let loaded = load_chapter(&epub, &href).unwrap_or_else(|err| {
        eprintln!("{err}");
        std::process::exit(2);
    });
    let setter = Typesetter::open_at(&latin, latin_index, &italic, italic_index, &cjk, cjk_index)
        .unwrap_or_else(|err| {
            eprintln!("{err}");
            std::process::exit(2);
        });
    let input = ChapterInput {
        xhtml: &loaded.xhtml,
        stylesheet: &loaded.stylesheet,
        base_href: &loaded.href,
        measure: Some(measure),
        font_size: Some(size),
        tune,
    };
    let cold_at = Instant::now();
    let cold = setter.compose(input).unwrap_or_else(|err| {
        eprintln!("{err}");
        std::process::exit(2);
    });
    let cold_ms = cold_at.elapsed().as_secs_f64() * 1000.0;
    let mut warm = Vec::new();
    let mut last = cold;
    for _ in 0..repeats {
        let started = Instant::now();
        last = setter.compose(input).unwrap();
        warm.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    warm.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let chapter_ms = warm[warm.len() / 2];
    let report = last.report();
    let reader_p95 = reader.number(&["gap_p95", "all_gap_p95"]);
    let reader_max = reader.number(&["gap_max", "all_gap_max"]);
    let decision = phase1(Phase1Input {
        crate_p95: report.word_gap_p95,
        reader_p95,
        crate_gaps: report.word_gap_count,
        kinsoku: report.kinsoku_violations,
        overflow: report.overflow_lines,
        chapter_ms,
        font_expand: tune.font_expand,
        letter_space: tune.letter_space,
    });
    let mut extra = Vec::new();
    if let Some(column) = reader.number(&["column"]) {
        if (column as f32 - measure).abs() > 0.5 {
            extra.push(format!("column {column:.2} vs measure {measure:.2}"));
        }
    }
    if let Some(font) = reader.number(&["font_size"]) {
        if (font as f32 - size).abs() > 0.5 {
            extra.push(format!("font {font:.2}px vs {size:.2}px"));
        }
    }
    let mut png_written = String::new();
    if let Some(path) = png_path.as_deref() {
        if let Some(bytes) = setter.png(&last, 0) {
            if let Some(parent) = std::path::Path::new(path).parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::write(path, &bytes).ok();
            png_written = path.to_string();
            if let (Some(reader_png), Some(side)) = (beside.as_deref(), side_path.as_deref()) {
                if let Ok(reader_bytes) = fs::read(reader_png) {
                    write_side(&reader_bytes, &bytes, side);
                }
            }
        }
    }
    let failures = match &decision {
        Ok(()) => Vec::new(),
        Err(items) => items.iter().map(failure_name).collect::<Vec<_>>(),
    };
    let passed = decision.is_ok() && extra.is_empty();
    let body = format!(
        "{{\n  \"href\": {},\n  \"measure\": {measure:.2},\n  \"font_size\": {size:.2},\n  \"crate_p95\": {:.4},\n  \"crate_max\": {:.4},\n  \"crate_gaps\": {},\n  \"reader_p95\": {},\n  \"reader_max\": {},\n  \"kinsoku\": {},\n  \"overflow\": {},\n  \"chapter_ms\": {chapter_ms:.3},\n  \"cold_ms\": {cold_ms:.3},\n  \"warm_min\": {:.3},\n  \"warm_max\": {:.3},\n  \"font_expand\": {},\n  \"letter_space\": {},\n  \"owned\": {},\n  \"native\": {},\n  \"all\": {},\n  \"owned_lines\": {},\n  \"native_lines\": {},\n  \"all_lines\": {},\n  \"column\": {},\n  \"pages\": {},\n  \"png\": {},\n  \"failures\": [{}],\n  \"extra\": [{}],\n  \"ok\": {}\n}}\n",
        json_str(&loaded.href),
        report.word_gap_p95,
        report.word_gap_max,
        report.word_gap_count,
        json_num(reader_p95),
        json_num(reader_max),
        report.kinsoku_violations,
        report.overflow_lines,
        warm[0],
        *warm.last().unwrap(),
        tune.font_expand,
        tune.letter_space,
        json_num(reader.number(&["owned", "paragraphs_owned"])),
        json_num(reader.number(&["native", "paragraphs_native"])),
        json_num(reader.number(&["all", "paragraphs_all"])),
        json_num(reader.number(&["owned_lines"])),
        json_num(reader.number(&["native_lines"])),
        json_num(reader.number(&["all_lines"])),
        json_num(reader.number(&["column"])),
        report.page_count,
        json_str(&png_written),
        failures.iter().map(|item| json_str(item)).collect::<Vec<_>>().join(", "),
        extra.iter().map(|item| json_str(item)).collect::<Vec<_>>().join(", "),
        if passed { "true" } else { "false" }
    );
    if let Some(parent) = std::path::Path::new(&out).parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&out, &body).unwrap();
    println!("{body}");
    if !passed {
        std::process::exit(1);
    }
}

fn failure_name(failure: &Phase1Failure) -> &'static str {
    match failure {
        Phase1Failure::FontExpand => "font-expand",
        Phase1Failure::LetterSpace => "letter-space",
        Phase1Failure::WorseThanReader => "crate p95 > reader p95",
        Phase1Failure::Kinsoku => "kinsoku",
        Phase1Failure::Overflow => "overflow",
        Phase1Failure::ChapterMs => "chapter_ms",
    }
}

fn write_side(reader_png: &[u8], crate_png: &[u8], path: &str) {
    let Ok(reader) = Pixmap::decode_png(reader_png) else { return };
    let Ok(page) = Pixmap::decode_png(crate_png) else { return };
    let gap = 24u32;
    let width = reader.width() + gap + page.width();
    let height = reader.height().max(page.height());
    let Some(mut canvas) = Pixmap::new(width, height) else { return };
    canvas.fill(tiny_skia::Color::from_rgba8(0x2a, 0x27, 0x23, 0xff));
    canvas.draw_pixmap(0, 0, reader.as_ref(), &tiny_skia::PixmapPaint::default(), tiny_skia::Transform::identity(), None);
    canvas.draw_pixmap(
        reader.width() as i32 + gap as i32,
        0,
        page.as_ref(),
        &tiny_skia::PixmapPaint::default(),
        tiny_skia::Transform::identity(),
        None,
    );
    if let Some(parent) = std::path::Path::new(path).parent() {
        fs::create_dir_all(parent).ok();
    }
    if let Some(bytes) = canvas.encode_png().ok() {
        fs::write(path, bytes).ok();
    }
}

struct Reader {
    text: String,
}

fn parse_reader(text: &str) -> Reader {
    Reader { text: text.to_string() }
}

impl Reader {
    fn number(&self, keys: &[&str]) -> Option<f64> {
        for key in keys {
            if let Some(value) = json_field(&self.text, key) {
                return value;
            }
        }
        None
    }
}

/// Pulls a JSON number for a top-level key. Enough for the flat reader file
/// the gate script writes. Nested objects are not read.
fn json_field(text: &str, key: &str) -> Option<Option<f64>> {
    let pattern = format!("\"{key}\"");
    let at = text.find(&pattern)?;
    let rest = text[at + pattern.len()..].trim_start();
    let rest = rest.strip_prefix(':')?.trim_start();
    if rest.starts_with("null") {
        return Some(None);
    }
    let end = rest.find(|ch: char| !(ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+' || ch == 'e' || ch == 'E'))
        .unwrap_or(rest.len());
    let number = rest[..end].parse().ok()?;
    Some(Some(number))
}

fn json_num(value: Option<f64>) -> String {
    match value {
        Some(value) if value.is_finite() => format!("{value:.4}"),
        _ => "null".to_string(),
    }
}

fn json_str(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

fn flag_value(args: &[String], flag: &str) -> Option<String> {
    args.iter().position(|item| item == flag).and_then(|index| args.get(index + 1).cloned())
}

fn required(args: &[String], flag: &str) -> String {
    flag_value(args, flag).unwrap_or_else(|| usage(&format!("missing {flag}")))
}

fn number(args: &[String], flag: &str) -> u32 {
    flag_value(args, flag).and_then(|value| value.parse().ok()).unwrap_or(0)
}

fn usage(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(2);
}
