use std::time::Instant;

use hayro::hayro_interpret::font::{Glyph as HayroGlyph, GlyphRun};
use hayro::hayro_interpret::hayro_cmap::BfString;
use hayro::hayro_interpret::hayro_syntax::Pdf;
use hayro::hayro_interpret::{
    interpret_page, BlendMode, ClipPath, Context, Device, DrawMode, DrawProps, Image,
    ImageDrawProps, InterpreterCache, InterpreterSettings, SoftMask,
};
use hayro::kurbo::{Affine, BezPath, Point, Rect};
use hayro::vello_cpu::color::palette::css::WHITE;
use hayro::{render, PixmapSettings, RenderCache, RenderSettings};
use pdfium_render::prelude::*;
use spike_report::{rss_kb, PdfEngine, PdfSample, PixelScale, Quad};

const PHRASES: &[&str] = &["Few-Shot Learners", "GPT-3"];

/// Page 0 of the GPT-3 PDF puts footnote asterisks 3.6pt above the name baseline.
/// The next text line is 11pt below. A band of 5pt keeps the mark on its line.
const HAYRO_LINE_SLACK: f32 = 5.0;

/// On that page, a letter pair runs at most 0.5pt past the font advance.
/// A word space runs at least 2.2pt past it. The title word spaces are 4.3pt past it.
const HAYRO_WORD_EXTRA: f32 = 1.0;

#[derive(Clone)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct Glyph {
    pub index: u32,
    pub ch: char,
    pub quad: Quad,
    /// User-space pen advance. pdfium uses the tight box width. hayro uses the font advance.
    pub advance: f32,
}

pub struct PageText {
    pub plain: String,
    pub glyphs: Vec<Glyph>,
}

pub enum Document {
    Pdfium(PdfiumDoc),
    Hayro(HayroDoc),
}

pub struct PdfiumDoc {
    pdfium: &'static Pdfium,
    doc: PdfDocument<'static>,
    count: usize,
}

pub struct HayroDoc {
    pdf: Pdf,
}

impl Document {
    pub fn open(
        engine: PdfEngine,
        bytes: Vec<u8>,
        lib_dir: &str,
    ) -> Result<(Self, f64, f64), String> {
        let started = Instant::now();
        match engine {
            PdfEngine::Pdfium => {
                let pdfium = Box::leak(Box::new(Pdfium::new(
                    Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(lib_dir))
                        .map_err(|e| format!("bind pdfium: {e}"))?,
                )));
                let bind_ms = started.elapsed().as_secs_f64() * 1000.0;
                let opened = Instant::now();
                let doc = pdfium
                    .load_pdf_from_byte_vec(bytes, None)
                    .map_err(|e| format!("pdfium open: {e}"))?;
                let open_ms = opened.elapsed().as_secs_f64() * 1000.0;
                let count = doc.pages().len() as usize;
                Ok((
                    Document::Pdfium(PdfiumDoc { pdfium, doc, count }),
                    bind_ms,
                    open_ms,
                ))
            }
            PdfEngine::Hayro => {
                let bind_ms = 0.0;
                let opened = Instant::now();
                let pdf = Pdf::new(bytes).map_err(|e| format!("hayro open: {e:?}"))?;
                let open_ms = opened.elapsed().as_secs_f64() * 1000.0;
                Ok((Document::Hayro(HayroDoc { pdf }), bind_ms, open_ms))
            }
        }
    }

    pub fn page_count(&self) -> usize {
        match self {
            Document::Pdfium(doc) => doc.count,
            Document::Hayro(doc) => doc.pdf.pages().len(),
        }
    }

    pub fn page_size_pt(&self, index: usize) -> (f32, f32) {
        match self {
            Document::Pdfium(doc) => {
                let page = doc.doc.pages().get(index as i32).expect("page");
                (page.width().value, page.height().value)
            }
            Document::Hayro(doc) => doc.pdf.pages()[index].render_dimensions(),
        }
    }

    pub fn raster(&self, index: usize, scale: PixelScale) -> Result<Rgba, String> {
        match self {
            Document::Pdfium(doc) => raster_pdfium(doc, index, scale),
            Document::Hayro(doc) => raster_hayro(doc, index, scale),
        }
    }

    pub fn text(&self, index: usize) -> PageText {
        match self {
            Document::Pdfium(doc) => text_pdfium(doc, index),
            Document::Hayro(doc) => text_hayro(doc, index),
        }
    }
}

/// Page 1 as a PNG. Hayro draws it, so the call does not need pdfium.
pub fn first_page_png(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let (doc, _, _) = Document::open(PdfEngine::Hayro, bytes.to_vec(), "")?;
    if doc.page_count() == 0 {
        return Err("pdf has no pages".into());
    }
    let image = fit_cover(doc.raster(0, PixelScale::Retina1x)?, 180);
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, image.width, image.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|err| err.to_string())?;
        writer
            .write_image_data(&image.bytes)
            .map_err(|err| err.to_string())?;
    }
    Ok(out)
}

fn fit_cover(image: Rgba, max_edge: u32) -> Rgba {
    let long = image.width.max(image.height);
    if long <= max_edge || image.width == 0 || image.height == 0 {
        return image;
    }
    let width = ((u64::from(image.width) * u64::from(max_edge)) / u64::from(long)).max(1) as u32;
    let height = ((u64::from(image.height) * u64::from(max_edge)) / u64::from(long)).max(1) as u32;
    let mut bytes = vec![0u8; width as usize * height as usize * 4];
    for y in 0..height {
        let src_y = (u64::from(y) * u64::from(image.height) / u64::from(height)) as u32;
        for x in 0..width {
            let src_x = (u64::from(x) * u64::from(image.width) / u64::from(width)) as u32;
            let from = ((src_y * image.width + src_x) * 4) as usize;
            let to = ((y * width + x) * 4) as usize;
            bytes[to..to + 4].copy_from_slice(&image.bytes[from..from + 4]);
        }
    }
    Rgba {
        width,
        height,
        bytes,
    }
}

fn raster_pdfium(doc: &PdfiumDoc, index: usize, scale: PixelScale) -> Result<Rgba, String> {
    let page = doc
        .doc
        .pages()
        .get(index as i32)
        .map_err(|e| format!("page: {e}"))?;
    let px = (page.width().value * scale.px_per_pt()).round() as i32;
    let bitmap = page
        .render_with_config(&PdfRenderConfig::new().set_target_width(px))
        .map_err(|e| format!("render: {e}"))?;
    let bytes = bitmap.as_rgba_bytes();
    let width = bitmap.width() as u32;
    let height = bitmap.height() as u32;
    if bytes.len() != width as usize * height as usize * 4 {
        return Err(format!(
            "rgba length {} != {}x{}x4",
            bytes.len(),
            width,
            height
        ));
    }
    Ok(Rgba {
        width,
        height,
        bytes,
    })
}

fn raster_hayro(doc: &HayroDoc, index: usize, scale: PixelScale) -> Result<Rgba, String> {
    let page = &doc.pdf.pages()[index];
    let (pw, ph) = page.render_dimensions();
    let px_w = (pw * scale.px_per_pt()).round() as u32;
    let px_h = (ph * scale.px_per_pt()).round() as u32;
    if px_w > u16::MAX as u32 || px_h > u16::MAX as u32 {
        return Err(format!("hayro pixmap {px_w}x{px_h} exceeds u16"));
    }
    let cache = RenderCache::new();
    let pixmap = render(
        page,
        &cache,
        &InterpreterSettings::default(),
        &RenderSettings::default(),
        &PixmapSettings {
            x_scale: scale.px_per_pt(),
            y_scale: scale.px_per_pt(),
            bg_color: WHITE,
        },
    );
    let width = pixmap.width() as u32;
    let height = pixmap.height() as u32;
    let bytes = pixmap.data_as_u8_slice().to_vec();
    if bytes.len() != width as usize * height as usize * 4 {
        return Err(format!(
            "hayro rgba length {} != {}x{}x4",
            bytes.len(),
            width,
            height
        ));
    }
    Ok(Rgba {
        width,
        height,
        bytes,
    })
}

fn text_pdfium(doc: &PdfiumDoc, index: usize) -> PageText {
    let page = doc.doc.pages().get(index as i32).expect("page");
    let height = page.height().value;
    let Ok(text) = page.text() else {
        return PageText {
            plain: String::new(),
            glyphs: Vec::new(),
        };
    };
    let mut glyphs = Vec::new();
    let mut plain = String::new();
    for ch in text.chars().iter() {
        let Some(c) = ch.unicode_char() else {
            continue;
        };
        let Ok(bounds) = ch.tight_bounds() else {
            continue;
        };
        let index = plain.chars().count() as u32;
        plain.push(c);
        let top = height - bounds.top().value;
        glyphs.push(Glyph {
            index,
            ch: c,
            quad: Quad {
                x: bounds.left().value,
                y: top,
                w: bounds.width().value.max(0.0),
                h: (bounds.top().value - bounds.bottom().value).abs().max(0.0),
            },
            advance: bounds.width().value.max(0.0),
        });
    }
    PageText { plain, glyphs }
}

fn text_hayro(doc: &HayroDoc, index: usize) -> PageText {
    let page = &doc.pdf.pages()[index];
    let (w, h) = page.render_dimensions();
    let cache = InterpreterCache::new();
    let mut context = Context::new(
        Affine::IDENTITY,
        Rect::new(0.0, 0.0, w as f64, h as f64),
        &cache,
        doc.pdf.xref(),
        InterpreterSettings::default(),
    );
    let mut extractor = Extractor {
        height: h,
        glyphs: Vec::new(),
    };
    interpret_page(page, &mut context, &mut extractor);
    reading_order(extractor.glyphs)
}

/// hayro draws glyphs and leaves word spaces as positioning gaps.
/// Cluster a line so a superscript stays with its baseline, then insert a space
/// when the next glyph starts more than its advance past the previous one.
fn reading_order(mut glyphs: Vec<Glyph>) -> PageText {
    glyphs.sort_by(|a, b| {
        a.quad
            .y
            .total_cmp(&b.quad.y)
            .then(a.quad.x.total_cmp(&b.quad.x))
    });
    let mut lines: Vec<Vec<Glyph>> = Vec::new();
    for glyph in glyphs {
        let same_line = lines.last().is_some_and(|line| {
            let top = line.iter().map(|g| g.quad.y).fold(f32::MAX, f32::min);
            glyph.quad.y - top <= HAYRO_LINE_SLACK
        });
        if same_line {
            lines.last_mut().unwrap().push(glyph);
        } else {
            lines.push(vec![glyph]);
        }
    }

    let mut plain = String::new();
    let mut ordered = Vec::new();
    for line in &mut lines {
        line.sort_by(|a, b| a.quad.x.total_cmp(&b.quad.x));
        for i in 0..line.len() {
            if i > 0 {
                let extra = line[i].quad.x - line[i - 1].quad.x - line[i - 1].advance;
                if extra > HAYRO_WORD_EXTRA {
                    plain.push(' ');
                }
            }
            line[i].index = plain.chars().count() as u32;
            plain.push(line[i].ch);
            ordered.push(line[i].clone());
        }
        plain.push('\n');
    }
    PageText {
        plain,
        glyphs: ordered,
    }
}

struct Extractor {
    height: f32,
    glyphs: Vec<Glyph>,
}

impl Device<'_> for Extractor {
    fn draw_path(&mut self, _: &BezPath, _: DrawProps<'_>, _: &DrawMode) {}
    fn push_clip_path(&mut self, _: &ClipPath) {}
    fn push_transparency_group(&mut self, _: f32, _: Option<SoftMask<'_>>, _: BlendMode) {}
    fn pop_clip(&mut self) {}
    fn pop_transparency_group(&mut self) {}
    fn draw_image(&mut self, _: Image<'_, '_>, _: ImageDrawProps<'_>) {}

    fn draw_glyph_run(&mut self, glyph_run: &GlyphRun<'_, '_>, props: DrawProps<'_>, _: &DrawMode) {
        for glyph in glyph_run.glyphs() {
            let Some(unicode) = glyph.as_unicode() else {
                continue;
            };
            let text = match unicode {
                BfString::Char(c) => c.to_string(),
                BfString::String(s) => s,
            };
            let flip =
                Affine::translate((0.0, self.height as f64)) * Affine::scale_non_uniform(1.0, -1.0);
            let transform = flip * props.transform * glyph.transform();
            let position = transform * Point::ZERO;
            let units = match &**glyph {
                HayroGlyph::Outline(outline) => outline.advance_width().unwrap_or(0.0),
                HayroGlyph::Type3(type3) => type3.advance_width().unwrap_or(0.0),
            };
            let advanced = transform * Point::new(units as f64, 0.0);
            let advance = (advanced.x - position.x) as f32;
            for ch in text.chars() {
                self.glyphs.push(Glyph {
                    index: 0,
                    ch,
                    quad: Quad {
                        x: position.x as f32,
                        y: position.y as f32,
                        w: 6.0,
                        h: 10.0,
                    },
                    advance,
                });
            }
        }
    }
}

pub fn select_drag(
    text: &PageText,
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
) -> Option<spike_report::Anchor> {
    let left = x0.min(x1);
    let right = x0.max(x1);
    let top = y0.min(y1);
    let bottom = y0.max(y1);
    let mut hit: Vec<&Glyph> = text
        .glyphs
        .iter()
        .filter(|g| {
            let cx = g.quad.x + g.quad.w * 0.5;
            let cy = g.quad.y + g.quad.h * 0.5;
            cx >= left && cx <= right && cy >= top && cy <= bottom
        })
        .collect();
    if hit.is_empty() {
        return None;
    }
    hit.sort_by_key(|g| g.index);
    let start = hit[0].index;
    let end = hit.last().unwrap().index + 1;
    let selected: String = text
        .plain
        .chars()
        .skip(start as usize)
        .take((end - start) as usize)
        .collect();
    if selected.trim().is_empty() {
        return None;
    }
    let quads = hit.iter().map(|g| g.quad).collect();
    Some(spike_report::Anchor::Pdf {
        page: 0,
        start,
        end,
        text: selected,
        quads,
    })
}

pub fn sample(
    engine: PdfEngine,
    path: &str,
    scale: PixelScale,
    process_started: Instant,
) -> Result<PdfSample, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
    let (doc, bind_ms, open_ms) = Document::open(engine, bytes, &spike_report::pdfium_lib())?;
    let pages = doc.page_count();
    if pages < 50 {
        return Err(format!("{path} has {pages} pages, want at least 50"));
    }
    let mut errors = 0u32;
    let raster_started = Instant::now();
    let first = match doc.raster(0, scale) {
        Ok(image) => image,
        Err(e) => return Err(e),
    };
    let first_raster_ms = raster_started.elapsed().as_secs_f64() * 1000.0;
    let open_to_first_ms = process_started.elapsed().as_secs_f64() * 1000.0;
    let rgba_ok = first.bytes.len() == first.width as usize * first.height as usize * 4;
    if !rgba_ok {
        errors += 1;
    }
    let rss_after_first_kb = rss_kb();
    let mut later_page_ms = Vec::new();
    for index in 1..pages {
        let started = Instant::now();
        match doc.raster(index, scale) {
            Ok(image) => {
                let expect = image.width as usize * image.height as usize * 4;
                if image.bytes.len() != expect {
                    if errors == 0 {
                        eprintln!(
                            "page {index} rgba {} != {expect} ({}x{})",
                            image.bytes.len(),
                            image.width,
                            image.height
                        );
                    }
                    errors += 1;
                }
            }
            Err(err) => {
                if errors == 0 {
                    eprintln!("page {index} {err}");
                }
                errors += 1;
            }
        }
        later_page_ms.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    let rss_after_last_kb = rss_kb();
    let text = doc.text(0);
    let text_hits = PHRASES
        .iter()
        .filter(|phrase| text.plain.contains(**phrase))
        .count() as u32;
    Ok(PdfSample {
        engine: engine.name().into(),
        scale: scale.name().into(),
        release: !cfg!(debug_assertions),
        pages,
        bind_ms,
        open_ms,
        first_raster_ms,
        open_to_first_ms,
        later_page_ms,
        errors,
        rgba_ok,
        text_hits,
        text_chars: text.plain.chars().count(),
        glyph_boxes: text.glyphs.len(),
        rss_after_first_kb,
        rss_after_last_kb,
        px_w: first.width,
        px_h: first.height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn four_words(line: &str) -> Option<String> {
        let words: Vec<&str> = line.split_whitespace().collect();
        let phrase = words.windows(4).find(|window| {
            window.iter().all(|word| {
                word.len() >= 3
                    && word
                        .chars()
                        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '\'')
                    && !word.contains("fi")
                    && !word.contains("fl")
                    && !word.contains("ff")
            })
        })?;
        Some(phrase.join(" "))
    }

    #[test]
    fn hayro_word_gap_recovers_the_title_phrase() {
        let path = spike_report::gpt3_path();
        if !std::path::Path::new(&path).exists() {
            return;
        }
        let bytes = std::fs::read(&path).unwrap();
        let (pdfium, _, _) = Document::open(
            PdfEngine::Pdfium,
            bytes.clone(),
            &spike_report::pdfium_lib(),
        )
        .unwrap();
        let (hayro, _, _) =
            Document::open(PdfEngine::Hayro, bytes, &spike_report::pdfium_lib()).unwrap();
        let pdfium_page = pdfium.text(0).plain;
        let hayro_page = hayro.text(0).plain;
        assert!(pdfium_page.contains("Few-Shot Learners"));
        assert!(
            hayro_page.contains("Few-Shot Learners"),
            "hayro title after the gap pass: {}",
            hayro_page.lines().next().unwrap_or("")
        );
        assert!(hayro_page.contains("GPT-3"));
        assert!(
            hayro_page.contains("Brown∗"),
            "footnote mark should stay on the name line"
        );
        assert!(
            hayro_page.contains('ﬁ'),
            "hayro still emits the fi ligature"
        );
        assert!(pdfium_page.contains("fi") && !pdfium_page.contains('ﬁ'));
        assert!(hayro_page.contains('-') && pdfium_page.contains('-'));
        assert_eq!(pdfium_page.chars().filter(|ch| *ch == '\u{2}').count(), 1);
        assert!(!hayro_page.contains('\u{2}'));

        for index in [1usize, 2, 10] {
            let page = pdfium.text(index).plain;
            let from_pdfium = page
                .lines()
                .find_map(four_words)
                .unwrap_or_else(|| panic!("page {index} has no four-word line"));
            let from_hayro = hayro.text(index).plain;
            assert!(
                from_hayro.contains(&from_pdfium),
                "page {index} missing {from_pdfium:?}"
            );
        }
    }

    #[test]
    fn the_shelf_field_guide_is_not_a_blank_page() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../library/fixtures/apple/Field Guide.pdf"
        );
        let bytes = std::fs::read(path).unwrap();
        let (doc, _, _) =
            Document::open(PdfEngine::Hayro, bytes, &spike_report::pdfium_lib()).unwrap();
        let image = doc.raster(0, PixelScale::Retina1x).unwrap();
        let dark = image
            .bytes
            .chunks(4)
            .filter(|px| px.len() == 4 && px[0] < 40 && px[1] < 40 && px[2] < 40)
            .count();
        assert!(dark > 100, "dark pixels {dark}");
    }

    #[test]
    fn highlight_keeps_the_ink_and_sits_on_abstract() {
        let path = spike_report::gpt3_path();
        if !std::path::Path::new(&path).exists() {
            return;
        }
        let bytes = std::fs::read(&path).unwrap();
        let (doc, _, _) =
            Document::open(PdfEngine::Pdfium, bytes, &spike_report::pdfium_lib()).unwrap();
        let scale = PixelScale::Retina1x;
        let mut image = doc.raster(0, scale).unwrap();
        let text = doc.text(0);
        let start = text.plain.find("Abstract").expect("Abstract");
        let from = text.plain[..start].chars().count() as u32;
        let quads: Vec<Quad> = text
            .glyphs
            .iter()
            .filter(|glyph| glyph.index >= from && glyph.index < from + 8)
            .map(|glyph| glyph.quad)
            .collect();
        assert_eq!(quads.len(), 8, "one box per letter of Abstract");
        let before = ink_in(&image, &quads, scale);
        paint_highlight(&mut image, &quads, scale);
        let after = ink_in(&image, &quads, scale);
        assert!(
            before > 20,
            "the word's ink is inside the quads, before={before}"
        );
        assert!(
            after + 5 >= before,
            "ink survived the tint, before={before} after={after}"
        );
    }

    fn ink_in(image: &Rgba, quads: &[Quad], scale: PixelScale) -> usize {
        let mut n = 0;
        for quad in quads {
            let px = quad.pixels(scale);
            let x0 = px.x.round().max(0.0) as i32;
            let y0 = px.y.round().max(0.0) as i32;
            let x1 = (px.x + px.w).round() as i32;
            let y1 = (px.y + px.h).round() as i32;
            for y in y0..y1 {
                for x in x0..x1 {
                    if y < 0 || x < 0 || y >= image.height as i32 || x >= image.width as i32 {
                        continue;
                    }
                    let i = ((y as u32 * image.width + x as u32) * 4) as usize;
                    if image.bytes[i] < 96 && image.bytes[i + 1] < 96 && image.bytes[i + 2] < 96 {
                        n += 1;
                    }
                }
            }
        }
        n
    }
}

pub fn paint_highlight(image: &mut Rgba, quads: &[Quad], scale: PixelScale) {
    for quad in quads {
        let mut band = *quad;
        band.y = (band.y - 1.5).max(0.0);
        band.h += 3.0;
        let px = band.pixels(scale);
        let x0 = px.x.round().max(0.0) as i32;
        let y0 = px.y.round().max(0.0) as i32;
        let x1 = (px.x + px.w).round() as i32;
        let y1 = (px.y + px.h).round() as i32;
        for y in y0..y1 {
            if y < 0 || y >= image.height as i32 {
                continue;
            }
            for x in x0..x1 {
                if x < 0 || x >= image.width as i32 {
                    continue;
                }
                let i = ((y as u32 * image.width + x as u32) * 4) as usize;
                tint_under_ink(&mut image.bytes[i..i + 4]);
            }
        }
    }
}

fn tint_under_ink(px: &mut [u8]) {
    let lum = px[0] as u16 + px[1] as u16 + px[2] as u16;
    if lum < 540 {
        px[3] = 255;
        return;
    }
    px[0] = ((px[0] as u16 * 2 + 255 * 8) / 10) as u8;
    px[1] = ((px[1] as u16 * 2 + 210 * 8) / 10) as u8;
    px[2] = ((px[2] as u16 * 2 + 40 * 8) / 10) as u8;
    px[3] = 255;
}

pub fn select_points(
    text: &PageText,
    a: (f32, f32),
    b: (f32, f32),
) -> Option<spike_report::Anchor> {
    let start = nearest(text, a.0, a.1)?;
    let end = nearest(text, b.0, b.1)?;
    let (from, to) = if start.index <= end.index {
        (start.index, end.index)
    } else {
        (end.index, start.index)
    };
    let glyphs: Vec<&Glyph> = text
        .glyphs
        .iter()
        .filter(|glyph| glyph.index >= from && glyph.index <= to)
        .collect();
    if glyphs.is_empty() {
        return None;
    }
    let selected: String = text
        .plain
        .chars()
        .skip(from as usize)
        .take((to - from + 1) as usize)
        .collect();
    if selected.trim().is_empty() {
        return None;
    }
    Some(spike_report::Anchor::Pdf {
        page: 0,
        start: from,
        end: to + 1,
        text: selected,
        quads: glyphs.iter().map(|glyph| glyph.quad).collect(),
    })
}

fn nearest(text: &PageText, x: f32, y: f32) -> Option<&Glyph> {
    text.glyphs.iter().min_by(|p, q| {
        let dp = (p.quad.x + p.quad.w * 0.5 - x).hypot(p.quad.y + p.quad.h * 0.5 - y);
        let dq = (q.quad.x + q.quad.w * 0.5 - x).hypot(q.quad.y + q.quad.h * 0.5 - y);
        dp.total_cmp(&dq)
    })
}
