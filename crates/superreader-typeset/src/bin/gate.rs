//! Phase 1 check for one chapter.
//!
//! Lays the chapter out on strict Knuth-Plass and compares its all-lines
//! word-gap p95 with the reader's all-lines p95. `--font-expand` and
//! `--letter-space` fail the gate. `--list-faces` prints the faces in a file.

use std::fs;
use std::time::Instant;

use superreader_typeset::{
    list_faces, load_chapter, phase1, BreakTune, ChapterInput, Phase1Failure, Phase1Input,
    Typesetter,
};
use tiny_skia::{Pixmap, PremultipliedColorU8};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|item| item == "--compose-only") {
        compose_only(&args);
        return;
    }
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
    let measure: f32 = required(&args, "--measure")
        .parse()
        .unwrap_or_else(|_| usage("bad --measure"));
    let size: f32 = required(&args, "--size")
        .parse()
        .unwrap_or_else(|_| usage("bad --size"));
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
    let stats = last.line_stats();
    let crate_lines = report.line_count;
    let scored_lines = stats.scored_lines;
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
                    write_side(
                        &reader_bytes,
                        &bytes,
                        side,
                        reader_p95,
                        Some(report.word_gap_p95),
                    );
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
        "{{\n  \"href\": {},\n  \"measure\": {measure:.2},\n  \"font_size\": {size:.2},\n  \"crate_p95\": {:.4},\n  \"crate_max\": {:.4},\n  \"crate_gaps\": {},\n  \"crate_lines\": {crate_lines},\n  \"scored_lines\": {scored_lines},\n  \"reader_p95\": {},\n  \"reader_max\": {},\n  \"kinsoku\": {},\n  \"overflow\": {},\n  \"chapter_ms\": {chapter_ms:.3},\n  \"cold_ms\": {cold_ms:.3},\n  \"warm_min\": {:.3},\n  \"warm_max\": {:.3},\n  \"font_expand\": {},\n  \"letter_space\": {},\n  \"owned\": {},\n  \"native\": {},\n  \"all\": {},\n  \"owned_lines\": {},\n  \"native_lines\": {},\n  \"all_lines\": {},\n  \"column\": {},\n  \"pages\": {},\n  \"png\": {},\n  \"failures\": [{}],\n  \"extra\": [{}],\n  \"ok\": {}\n}}\n",
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

fn compose_only(args: &[String]) {
    let source = required(args, "--reader");
    let side = required(args, "--side");
    let reader_p95 = parse_p95(flag_value(args, "--reader-p95").as_deref());
    let crate_p95 = parse_p95(flag_value(args, "--crate-p95").as_deref());
    let bytes = fs::read(&source).unwrap_or_else(|err| {
        eprintln!("reader {source}: {err}");
        std::process::exit(2);
    });
    let (reader_png, crate_png) = if args.iter().any(|item| item == "--split") {
        split_composite(&bytes).unwrap_or_else(|| usage("no side-by-side gap in --reader"))
    } else {
        let page = required(args, "--png");
        let page_bytes = fs::read(&page).unwrap_or_else(|err| {
            eprintln!("png {page}: {err}");
            std::process::exit(2);
        });
        (bytes, page_bytes)
    };
    write_side(&reader_png, &crate_png, &side, reader_p95, crate_p95);
}

fn parse_p95(text: Option<&str>) -> Option<f64> {
    let text = text?;
    if text == "none" || text == "null" {
        return None;
    }
    text.parse().ok()
}

/// Puts the reader shot and the crate page on one white sheet.
///
/// Both columns are the same width, both pages sit on #FFFFFF, and each
/// side is labelled with its all-lines p95. A chapter with no scored gaps
/// prints `p95 none`.
fn write_side(
    reader_png: &[u8],
    crate_png: &[u8],
    path: &str,
    reader_p95: Option<f64>,
    crate_p95: Option<f64>,
) {
    let Ok(mut reader) = Pixmap::decode_png(reader_png) else {
        return;
    };
    let Ok(mut page) = Pixmap::decode_png(crate_png) else {
        return;
    };
    whiten_paper(&mut reader);
    whiten_paper(&mut page);
    let reader_col = text_column(&reader);
    let page_col = text_column(&page);
    let scale = if reader_col.width >= 8 && page_col.width >= 8 {
        (page_col.width as f32 / reader_col.width as f32).clamp(0.2, 2.0)
    } else {
        1.0
    };
    let mut fitted = if (scale - 1.0).abs() < 0.01 {
        reader
    } else {
        match downscale(&reader, scale) {
            Some(image) => image,
            None => reader,
        }
    };
    whiten_rim(&mut fitted, 18);
    let fit_scale_x = fitted.width() as f32 / reader_col.source_width as f32;
    let fit_scale_y = fitted.height() as f32 / reader_col.source_height as f32;
    let reader_left = (reader_col.left as f32 * fit_scale_x).round() as i32;
    let reader_top = (reader_col.top as f32 * fit_scale_y).round() as i32;
    let page_left = page_col.left as i32;
    let page_top = page_col.top as i32;
    let mut reader_x = 24 - reader_left;
    let mut page_x = 24 - page_left;
    let mut reader_y = 24 - reader_top;
    let mut page_y = 24 - page_top;
    let min_x = reader_x.min(page_x);
    let min_y = reader_y.min(page_y);
    if min_x < 16 {
        let shift = 16 - min_x;
        reader_x += shift;
        page_x += shift;
    }
    if min_y < 16 {
        let shift = 16 - min_y;
        reader_y += shift;
        page_y += shift;
    }
    let fitted_w = fitted.width() as i32;
    let fitted_h = fitted.height() as i32;
    let page_w = page.width() as i32;
    let page_h = page.height() as i32;
    let panel_w = (reader_x + fitted_w).max(page_x + page_w) + 16;
    let panel_h = (reader_y + fitted_h).max(page_y + page_h) + 16;
    let gap = 48i32;
    let label_h = 72i32;
    let width = (panel_w * 2 + gap) as u32;
    let height = (panel_h + label_h) as u32;
    let Some(mut canvas) = Pixmap::new(width, height) else {
        return;
    };
    canvas.fill(tiny_skia::Color::from_rgba8(0xff, 0xff, 0xff, 0xff));
    blit(&mut canvas, &fitted, reader_x, reader_y);
    blit(&mut canvas, &page, panel_w + gap + page_x, page_y);
    let scale_px = if panel_w < 520 { 3 } else { 4 };
    draw_label(
        &mut canvas,
        "reader (WKWebView)",
        0,
        panel_w,
        panel_h + 8,
        scale_px,
    );
    draw_label(
        &mut canvas,
        &p95_label(reader_p95),
        0,
        panel_w,
        panel_h + 8 + scale_px * 8,
        scale_px,
    );
    draw_label(
        &mut canvas,
        "crate",
        panel_w + gap,
        panel_w,
        panel_h + 8,
        scale_px,
    );
    draw_label(
        &mut canvas,
        &p95_label(crate_p95),
        panel_w + gap,
        panel_w,
        panel_h + 8 + scale_px * 8,
        scale_px,
    );
    if let Some(parent) = std::path::Path::new(path).parent() {
        fs::create_dir_all(parent).ok();
    }
    if let Ok(bytes) = canvas.encode_png() {
        fs::write(path, bytes).ok();
    }
}

fn p95_label(value: Option<f64>) -> String {
    match value {
        Some(value) if value.is_finite() => format!("p95 {value:.4}"),
        _ => "p95 none".to_string(),
    }
}

struct Column {
    left: u32,
    top: u32,
    width: u32,
    source_width: u32,
    source_height: u32,
}

fn text_column(pixmap: &Pixmap) -> Column {
    let width = pixmap.width() as usize;
    let height = pixmap.height() as usize;
    let pixels = pixmap.pixels();
    let margin = 8usize;
    let mut spans = Vec::new();
    if width > margin * 2 {
        for y in 0..height {
            let mut left = None;
            let mut right = 0usize;
            let mut count = 0u32;
            let row = y * width;
            for x in margin..width - margin {
                let pixel = pixels[row + x];
                if pixel.red() < 80 && pixel.green() < 80 && pixel.blue() < 80 {
                    count += 1;
                    if left.is_none() {
                        left = Some(x);
                    }
                    right = x;
                }
            }
            if count >= 12 {
                if let Some(left) = left {
                    spans.push((right - left, left, y));
                }
            }
        }
    }
    if spans.is_empty() {
        return Column {
            left: 0,
            top: 0,
            width: pixmap.width().max(1),
            source_width: pixmap.width().max(1),
            source_height: pixmap.height().max(1),
        };
    }
    spans.sort_by_key(|span| span.0);
    let picked = spans[spans.len() * 9 / 10].0;
    let mut full: Vec<_> = spans
        .into_iter()
        .filter(|span| span.0 + 6 >= picked)
        .collect();
    full.sort_by_key(|span| span.1);
    let left = full[full.len() / 2].1 as u32;
    full.sort_by_key(|span| span.1 + span.0);
    let right = (full[full.len() / 2].1 + full[full.len() / 2].0) as u32;
    full.sort_by_key(|span| span.2);
    let top = full.first().map(|span| span.2).unwrap_or(0) as u32;
    Column {
        left,
        top,
        width: right.saturating_sub(left).max(1),
        source_width: pixmap.width().max(1),
        source_height: pixmap.height().max(1),
    }
}

fn whiten_paper(pixmap: &mut Pixmap) {
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    for (index, pixel) in pixmap.pixels_mut().iter_mut().enumerate() {
        let x = (index as i32) % width;
        let y = (index as i32) / width;
        let red = pixel.red();
        let green = pixel.green();
        let blue = pixel.blue();
        let light = red > 200 && green > 190 && blue > 175;
        let canvas = red.abs_diff(0x2a) < 8 && green.abs_diff(0x27) < 8 && blue.abs_diff(0x23) < 8;
        let edge = x < 24 || y < 24 || x + 24 >= width || y + 24 >= height;
        if light || (canvas && edge) {
            *pixel = PremultipliedColorU8::from_rgba(255, 255, 255, 255).unwrap();
        }
    }
}

fn whiten_rim(pixmap: &mut Pixmap, rim: i32) {
    let width = pixmap.width() as i32;
    let height = pixmap.height() as i32;
    let white = PremultipliedColorU8::from_rgba(255, 255, 255, 255).unwrap();
    for (index, pixel) in pixmap.pixels_mut().iter_mut().enumerate() {
        let x = (index as i32) % width;
        let y = (index as i32) / width;
        if x < rim || y < rim || x + rim >= width || y + rim >= height {
            *pixel = white;
        }
    }
}

fn downscale(src: &Pixmap, scale: f32) -> Option<Pixmap> {
    let width = ((src.width() as f32) * scale).round().max(1.0) as u32;
    let height = ((src.height() as f32) * scale).round().max(1.0) as u32;
    let mut dst = Pixmap::new(width, height)?;
    let src_pixels = src.pixels();
    let src_w = src.width() as usize;
    let src_h = src.height() as usize;
    let dst_pixels = dst.pixels_mut();
    for y in 0..height as usize {
        let y0 = ((y as f32) / scale).floor() as usize;
        let y1 = (((y + 1) as f32) / scale).ceil() as usize;
        let y1 = y1.min(src_h).max(y0 + 1);
        for x in 0..width as usize {
            let x0 = ((x as f32) / scale).floor() as usize;
            let x1 = (((x + 1) as f32) / scale).ceil() as usize;
            let x1 = x1.min(src_w).max(x0 + 1);
            let mut red = 0u32;
            let mut green = 0u32;
            let mut blue = 0u32;
            let mut count = 0u32;
            for yy in y0..y1 {
                for xx in x0..x1 {
                    let pixel = src_pixels[yy * src_w + xx];
                    red += u32::from(pixel.red());
                    green += u32::from(pixel.green());
                    blue += u32::from(pixel.blue());
                    count += 1;
                }
            }
            dst_pixels[y * width as usize + x] = PremultipliedColorU8::from_rgba(
                (red / count) as u8,
                (green / count) as u8,
                (blue / count) as u8,
                255,
            )
            .unwrap();
        }
    }
    Some(dst)
}

fn blit(canvas: &mut Pixmap, src: &Pixmap, origin_x: i32, origin_y: i32) {
    let canvas_w = canvas.width() as i32;
    let canvas_h = canvas.height() as i32;
    let src_w = src.width() as i32;
    let src_h = src.height() as i32;
    let src_pixels = src.pixels();
    let dst_w = canvas.width() as usize;
    let dst = canvas.pixels_mut();
    for y in 0..src_h {
        let dy = origin_y + y;
        if dy < 0 || dy >= canvas_h {
            continue;
        }
        for x in 0..src_w {
            let dx = origin_x + x;
            if dx < 0 || dx >= canvas_w {
                continue;
            }
            dst[dy as usize * dst_w + dx as usize] =
                src_pixels[y as usize * src.width() as usize + x as usize];
        }
    }
}

fn draw_label(canvas: &mut Pixmap, text: &str, origin_x: i32, panel_w: i32, y: i32, scale: i32) {
    let width = text.chars().count() as i32 * 6 * scale;
    let x = origin_x + (panel_w - width) / 2;
    draw_text(canvas, text, x, y, scale);
}

fn draw_text(canvas: &mut Pixmap, text: &str, mut x: i32, y: i32, scale: i32) {
    let ink = PremultipliedColorU8::from_rgba(0x1c, 0x19, 0x15, 0xff).unwrap();
    for ch in text.chars() {
        let glyph = glyph(ch);
        for row in 0..7i32 {
            for col in 0..5i32 {
                if glyph[row as usize] & (1 << (4 - col)) == 0 {
                    continue;
                }
                for sy in 0..scale {
                    for sx in 0..scale {
                        put_pixel(canvas, x + col * scale + sx, y + row * scale + sy, ink);
                    }
                }
            }
        }
        x += 6 * scale;
    }
}

fn put_pixel(canvas: &mut Pixmap, x: i32, y: i32, color: PremultipliedColorU8) {
    if x < 0 || y < 0 || x >= canvas.width() as i32 || y >= canvas.height() as i32 {
        return;
    }
    let width = canvas.width() as usize;
    canvas.pixels_mut()[y as usize * width + x as usize] = color;
}

fn glyph(ch: char) -> [u8; 7] {
    match ch {
        ' ' => [0, 0, 0, 0, 0, 0, 0],
        '(' => [0x06, 0x08, 0x10, 0x10, 0x10, 0x08, 0x06],
        ')' => [0x0c, 0x02, 0x01, 0x01, 0x01, 0x02, 0x0c],
        '.' => [0, 0, 0, 0, 0, 0x06, 0x06],
        '0' => [0x0e, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0e],
        '1' => [0x04, 0x0c, 0x04, 0x04, 0x04, 0x04, 0x0e],
        '2' => [0x0e, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1f],
        '3' => [0x1e, 0x01, 0x01, 0x0e, 0x01, 0x01, 0x1e],
        '4' => [0x02, 0x06, 0x0a, 0x12, 0x1f, 0x02, 0x02],
        '5' => [0x1f, 0x10, 0x10, 0x1e, 0x01, 0x01, 0x1e],
        '6' => [0x06, 0x08, 0x10, 0x1e, 0x11, 0x11, 0x0e],
        '7' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0e, 0x11, 0x11, 0x0e, 0x11, 0x11, 0x0e],
        '9' => [0x0e, 0x11, 0x11, 0x0f, 0x01, 0x02, 0x0c],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x0a, 0x0a, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0a],
        'a' => [0, 0, 0x0e, 0x01, 0x0f, 0x11, 0x0f],
        'b' => [0x10, 0x10, 0x1e, 0x11, 0x11, 0x11, 0x1e],
        'c' => [0, 0, 0x0e, 0x10, 0x10, 0x10, 0x0e],
        'd' => [0x01, 0x01, 0x0f, 0x11, 0x11, 0x11, 0x0f],
        'e' => [0, 0, 0x0e, 0x11, 0x1f, 0x10, 0x0e],
        'i' => [0x04, 0, 0x0c, 0x04, 0x04, 0x04, 0x0e],
        'n' => [0, 0, 0x16, 0x19, 0x11, 0x11, 0x11],
        'o' => [0, 0, 0x0e, 0x11, 0x11, 0x11, 0x0e],
        'p' => [0, 0, 0x1e, 0x11, 0x1e, 0x10, 0x10],
        'r' => [0, 0, 0x16, 0x18, 0x10, 0x10, 0x10],
        't' => [0x04, 0x04, 0x1f, 0x04, 0x04, 0x04, 0x03],
        'w' => [0, 0, 0x11, 0x11, 0x15, 0x15, 0x0a],
        _ => [0x1f, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1f],
    }
}

fn split_composite(bytes: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    let pixmap = Pixmap::decode_png(bytes).ok()?;
    let width = pixmap.width();
    let height = pixmap.height();
    let pixels = pixmap.pixels();
    let mut bands = Vec::new();
    let mut start = None;
    for x in 0..width {
        let mut dark = 0u32;
        let mut seen = 0u32;
        for y in (0..height).step_by(4) {
            let pixel = pixels[(y * width + x) as usize];
            seen += 1;
            if pixel.red().abs_diff(0x2a) < 14
                && pixel.green().abs_diff(0x27) < 14
                && pixel.blue().abs_diff(0x23) < 14
            {
                dark += 1;
            }
        }
        let gap = dark * 10 > seen * 8;
        if gap {
            if start.is_none() {
                start = Some(x);
            }
        } else if let Some(from) = start.take() {
            bands.push((from, x));
        }
    }
    if let Some(from) = start {
        bands.push((from, width));
    }
    let (gap_from, gap_to) = bands
        .into_iter()
        .filter(|(from, to)| to - from >= 8)
        .max_by_key(|(from, to)| to - from)?;
    let reader = crop(&pixmap, 0, 0, gap_from, height)?;
    let mut left = width;
    let mut top = height;
    let mut right = gap_to;
    let mut bottom = 0u32;
    for y in 0..height {
        for x in gap_to..width {
            let pixel = pixels[(y * width + x) as usize];
            let canvas = pixel.red().abs_diff(0x2a) < 18
                && pixel.green().abs_diff(0x27) < 18
                && pixel.blue().abs_diff(0x23) < 18;
            if canvas {
                continue;
            }
            left = left.min(x);
            right = right.max(x + 1);
            top = top.min(y);
            bottom = bottom.max(y + 1);
        }
    }
    if left >= right || top >= bottom {
        return None;
    }
    let page = crop(&pixmap, left, top, right - left, bottom - top)?;
    Some((reader.encode_png().ok()?, page.encode_png().ok()?))
}

fn crop(pixmap: &Pixmap, x: u32, y: u32, width: u32, height: u32) -> Option<Pixmap> {
    let mut out = Pixmap::new(width, height)?;
    let src = pixmap.pixels();
    let dst = out.pixels_mut();
    let src_w = pixmap.width();
    for row in 0..height {
        for col in 0..width {
            dst[(row * width + col) as usize] = src[((y + row) * src_w + x + col) as usize];
        }
    }
    Some(out)
}

struct Reader {
    text: String,
}

fn parse_reader(text: &str) -> Reader {
    Reader {
        text: text.to_string(),
    }
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
    let end = rest
        .find(|ch: char| {
            !(ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+' || ch == 'e' || ch == 'E')
        })
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
    args.iter()
        .position(|item| item == flag)
        .and_then(|index| args.get(index + 1).cloned())
}

fn required(args: &[String], flag: &str) -> String {
    flag_value(args, flag).unwrap_or_else(|| usage(&format!("missing {flag}")))
}

fn number(args: &[String], flag: &str) -> u32 {
    flag_value(args, flag)
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

fn usage(message: &str) -> ! {
    eprintln!("{message}");
    std::process::exit(2);
}
