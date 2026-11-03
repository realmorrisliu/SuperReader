use std::time::Instant;

use hayro::hayro_interpret::font::GlyphRun;
use hayro::hayro_interpret::hayro_cmap::BfString;
use hayro::hayro_interpret::hayro_syntax::Pdf;
use hayro::hayro_interpret::{
    BlendMode, ClipPath, Context, Device, DrawMode, DrawProps, Image, ImageDrawProps,
    InterpreterCache, InterpreterSettings, SoftMask, interpret_page,
};
use hayro::kurbo::{Affine, BezPath, Point, Rect};
use hayro::vello_cpu::color::palette::css::WHITE;
use hayro::{PixmapSettings, RenderCache, RenderSettings, render};
use pdfium_render::prelude::*;
use spike_report::{PdfEngine, PdfSample, PixelScale, Quad, rss_kb};

const PHRASES: &[&str] = &["Few-Shot Learners", "GPT-3"];

#[derive(Clone)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
}

pub struct Glyph {
    pub index: u32,
    pub ch: char,
    pub quad: Quad,
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
    pub fn open(engine: PdfEngine, bytes: Vec<u8>, lib_dir: &str) -> Result<(Self, f64, f64), String> {
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
        let Ok(bounds) = ch.loose_bounds() else {
            continue;
        };
        let index = plain.chars().count() as u32;
        plain.push(c);
        glyphs.push(Glyph {
            index,
            ch: c,
            quad: Quad {
                x: bounds.left().value,
                y: height - bounds.top().value,
                w: bounds.width().value.max(0.0),
                h: bounds.height().value.max(0.0),
            },
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
        plain: String::new(),
    };
    interpret_page(page, &mut context, &mut extractor);
    let mut glyphs = extractor.glyphs;
    glyphs.sort_by(|a, b| {
        a.quad
            .y
            .total_cmp(&b.quad.y)
            .then(a.quad.x.total_cmp(&b.quad.x))
    });
    let mut plain = String::new();
    for (i, glyph) in glyphs.iter_mut().enumerate() {
        glyph.index = i as u32;
        plain.push(glyph.ch);
    }
    PageText { plain, glyphs }
}

struct Extractor {
    height: f32,
    glyphs: Vec<Glyph>,
    plain: String,
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
            let flip = Affine::translate((0.0, self.height as f64)) * Affine::scale_non_uniform(1.0, -1.0);
            let transform = flip * props.transform * glyph.transform();
            let position = transform * Point::ZERO;
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
                });
            }
        }
    }
}

pub fn select_drag(text: &PageText, x0: f32, y0: f32, x1: f32, y1: f32) -> Option<spike_report::Anchor> {
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

pub fn sample(engine: PdfEngine, path: &str, scale: PixelScale, process_started: Instant) -> Result<PdfSample, String> {
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

pub fn paint_highlight(image: &mut Rgba, quads: &[Quad], scale: PixelScale) {
    for quad in quads {
        let px = quad.pixels(scale);
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
                image.bytes[i] = 255;
                image.bytes[i + 1] = 230;
                image.bytes[i + 2] = 80;
                image.bytes[i + 3] = 180;
            }
        }
    }
}
