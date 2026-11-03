use rbook::Epub;
#[cfg(feature = "blitz")]
use std::time::Instant;
use spike_report::Anchor;

pub const PAGE_W: u32 = 720;
pub const PAGE_H: u32 = 960;

pub struct Chapter {
    pub title: String,
    pub href: String,
    pub html: String,
    pub flow_html: String,
    pub plain: String,
}

pub struct Book {
    pub title: String,
    pub chapters: Vec<Chapter>,
}

pub fn open_book(path: &str) -> Result<Book, String> {
    let epub = Epub::open(path).map_err(|e| format!("epub open: {e}"))?;
    let title = epub
        .metadata()
        .title()
        .map(|t| t.value().to_string())
        .unwrap_or_else(|| path.to_string());
    let mut css = String::new();
    for entry in epub.manifest().styles() {
        if let Ok(text) = entry.read_str() {
            css.push_str(&text);
            css.push('\n');
        }
    }
    let mut reader = epub.reader();
    let mut chapters = Vec::new();
    while let Some(item) = reader.read_next() {
        let Ok(data) = item else { continue };
        let href = data.manifest_entry().href().to_string();
        let raw = data.content();
        let plain = plain_text(raw);
        if skip_href(&href) || plain.chars().count() < 1500 {
            continue;
        }
        chapters.push(Chapter {
            title: href.clone(),
            href,
            html: page_html(&safe_css(&css), &body_inner(raw)),
            flow_html: flow_html(&safe_css(&css), &body_inner(raw)),
            plain,
        });
        if chapters.len() == 4 {
            break;
        }
    }
    if chapters.is_empty() {
        return Err(format!("no chapter text in {path}"));
    }
    Ok(Book { title, chapters })
}

fn skip_href(href: &str) -> bool {
    let href = href.to_ascii_lowercase();
    [
        "imprint",
        "colophon",
        "titlepage",
        "cover",
        "toc",
        "nav",
        "license",
        "uncopyright",
        "dedication",
    ]
    .iter()
    .any(|part| href.contains(part))
}

fn body_inner(xhtml: &str) -> String {
    let lower = xhtml.to_ascii_lowercase();
    let Some(start) = lower.find("<body") else {
        return xhtml.to_string();
    };
    let Some(gt) = xhtml[start..].find('>') else {
        return xhtml.to_string();
    };
    let from = start + gt + 1;
    let end = lower[from..].find("</body>").map(|i| from + i).unwrap_or(xhtml.len());
    xhtml[from..end].to_string()
}

fn safe_css(css: &str) -> String {
    let mut out = String::new();
    let mut rest = css;
    while let Some(at) = rest.find("url(") {
        out.push_str(&rest[..at]);
        rest = &rest[at + 4..];
        if let Some(end) = rest.find(')') {
            rest = &rest[end + 1..];
        }
    }
    out.push_str(rest);
    out.lines()
        .filter(|line| !line.trim_start().starts_with("@import"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn plain_text(xhtml: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in xhtml.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn flow_html(css: &str, chapter: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><style>
{css}
html, body {{ margin: 0; background: #f4f1ea; color: #1c1915; }}
#page {{ box-sizing: border-box; width: {PAGE_W}px; padding: 48px 56px; }}
</style></head>
<body><div id="page">{chapter}</div></body></html>"#
    )
}

pub fn page_html(css: &str, chapter: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><style>
{css}
html, body {{ margin: 0; height: 100%; background: #f4f1ea; color: #1c1915; }}
#page {{
  box-sizing: border-box;
  height: 100vh;
  padding: 48px 56px;
  column-width: calc(100vw - 112px);
  column-gap: 0;
  column-fill: auto;
  overflow-x: scroll;
  overflow-y: hidden;
}}
</style></head>
<body><div id="page">{chapter}</div></body></html>"#
    )
}

#[cfg(feature = "blitz")]
pub struct BlitzPages {
    pub columns: bool,
    pub count: usize,
    pub raster_ms: Vec<f64>,
    pub first: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[cfg(feature = "blitz")]
pub fn paginate_blitz(html: &str) -> Result<BlitzPages, String> {
    use anyrender::{PaintScene as _, render_to_buffer};
    use anyrender_vello_cpu::VelloCpuImageRenderer;
    use blitz_dom::util::Color;
    use blitz_dom::DocumentConfig;
    use blitz_html::HtmlDocument;
    use blitz_paint::paint_scene;
    use blitz_traits::shell::{ColorScheme, Viewport};
    use peniko::Fill;
    use peniko::kurbo::Rect;
    let scale = 2.0_f64;
    let width = PAGE_W;
    let height = PAGE_H;
    let mut document = HtmlDocument::from_html(
        html,
        DocumentConfig {
            base_url: Some("https://spike.invalid/book/".into()),
            viewport: Some(Viewport::new(
                (width as f64 * scale) as u32,
                (height as f64 * scale) as u32,
                scale as f32,
                ColorScheme::Light,
            )),
            ..DocumentConfig::default()
        },
    );
    let started = Instant::now();
    document.resolve(0.0);
    let resolve_ms = started.elapsed().as_secs_f64() * 1000.0;
    let laid_out = document.root_element().final_layout().size;
    let page_w = width as f32 * scale as f32;
    let page_h = height as f32 * scale as f32;
    let columns = laid_out.width > page_w * 1.4;
    let count = if columns {
        (laid_out.width / page_w).round().max(1.0) as usize
    } else {
        (laid_out.height / page_h).ceil().max(1.0) as usize
    };
    let render_w = (width as f64 * scale) as u32;
    let render_h = (height as f64 * scale) as u32;
    let mut raster_ms = vec![resolve_ms];
    let mut first = Vec::new();
    for page in 0..count.min(8) {
        let started = Instant::now();
        let (x_offset, y_offset) = if columns {
            ((page as f64 * width as f64 * scale) as u32, 0)
        } else {
            (0, (page as f64 * height as f64 * scale) as u32)
        };
        let buffer = render_to_buffer::<VelloCpuImageRenderer, _>(
            |scene| {
                scene.fill(
                    Fill::NonZero,
                    Default::default(),
                    Color::WHITE,
                    Default::default(),
                    &Rect::new(0.0, 0.0, render_w as f64, render_h as f64),
                );
                paint_scene(
                    scene,
                    document.as_mut(),
                    scale,
                    render_w,
                    render_h,
                    x_offset,
                    y_offset,
                );
            },
            render_w,
            render_h,
        );
        raster_ms.push(started.elapsed().as_secs_f64() * 1000.0);
        if page == 0 {
            first = buffer;
        }
    }
    Ok(BlitzPages {
        columns,
        count,
        raster_ms,
        first,
        width: render_w,
        height: render_h,
    })
}

pub fn anchor_in_chapter(href: &str, plain: &str, needle: &str) -> Option<Anchor> {
    let byte = plain.find(needle)?;
    let start = plain[..byte].chars().count() as u32;
    let end = start + needle.chars().count() as u32;
    Some(Anchor::Epub {
        spine_href: href.into(),
        start,
        end,
        text: needle.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_uses_the_chapter_text_not_a_page_number() {
        let plain = "It is a truth universally acknowledged";
        let anchor = anchor_in_chapter("chapter-1.xhtml", plain, "truth universally").unwrap();
        let Anchor::Epub { start, end, spine_href, .. } = anchor else {
            panic!("epub");
        };
        assert_eq!(spine_href, "chapter-1.xhtml");
        let got: String = plain
            .chars()
            .skip(start as usize)
            .take((end - start) as usize)
            .collect();
        assert_eq!(got, "truth universally");
    }
}
