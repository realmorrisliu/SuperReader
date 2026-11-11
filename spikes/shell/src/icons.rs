use nsh_platform_macos::objc2::rc::Retained;
use nsh_platform_macos::objc2::AnyThread;
use objc2_app_kit::NSImage;
use objc2_foundation::{NSData, NSSize};

const LIST: &str = include_str!("../../../assets/icons/list.svg");
const BOOKMARK: &str = include_str!("../../../assets/icons/bookmark.svg");
const SEARCH: &str = include_str!("../../../assets/icons/search.svg");
const BOOKMARK_PLUS: &str = include_str!("../../../assets/icons/bookmark-plus.svg");
const NOTE: &str = include_str!("../../../assets/icons/note.svg");
const BOOK: &str = include_str!("../../../assets/icons/book.svg");
const FOLDER: &str = include_str!("../../../assets/icons/folder.svg");
const LIBRARY: &str = include_str!("../../../assets/icons/library.svg");
const POINTS: f64 = 18.0;
const PIXELS: u32 = 54;

pub fn contents_image() -> Option<Retained<NSImage>> {
    image(LIST)
}

pub fn bookmark_image() -> Option<Retained<NSImage>> {
    image(BOOKMARK)
}

pub fn search_image() -> Option<Retained<NSImage>> {
    image(SEARCH)
}

pub fn add_bookmark_image() -> Option<Retained<NSImage>> {
    image(BOOKMARK_PLUS)
}

pub fn notes_image() -> Option<Retained<NSImage>> {
    image(NOTE)
}

pub struct OnboardingGlyphs {
    pub book: std::path::PathBuf,
    pub folder: std::path::PathBuf,
    pub library: std::path::PathBuf,
}

/// Black outline PNGs for the NSH shelf. The toolbar keeps template NSImages.
pub fn write_onboarding_glyphs(dir: &std::path::Path) -> Option<OnboardingGlyphs> {
    std::fs::create_dir_all(dir).ok()?;
    Some(OnboardingGlyphs {
        book: write_glyph(dir, "book", BOOK)?,
        folder: write_glyph(dir, "folder", FOLDER)?,
        library: write_glyph(dir, "library", LIBRARY)?,
    })
}

fn write_glyph(dir: &std::path::Path, name: &str, svg: &str) -> Option<std::path::PathBuf> {
    let png = raster_png(svg, 32).ok()?;
    let path = dir.join(format!("{name}.png"));
    std::fs::write(&path, png).ok()?;
    Some(path)
}

fn image(svg: &str) -> Option<Retained<NSImage>> {
    let png = raster_png(svg, PIXELS).ok()?;
    let data = NSData::with_bytes(&png);
    let image = NSImage::initWithData(NSImage::alloc(), &data)?;
    image.setSize(NSSize::new(POINTS, POINTS));
    image.setTemplate(true);
    Some(image)
}

fn raster_png(svg: &str, px: u32) -> Result<Vec<u8>, String> {
    let pixmap = raster(svg, px)?;
    pixmap.encode_png().map_err(|err| err.to_string())
}

fn raster(svg: &str, px: u32) -> Result<resvg::tiny_skia::Pixmap, String> {
    let painted = paint(svg);
    let tree = resvg::usvg::Tree::from_str(&painted, &resvg::usvg::Options::default())
        .map_err(|err| err.to_string())?;
    let pixmap = draw(&tree, px)?;
    let ink = pixmap.pixels().iter().filter(|px| px.alpha() > 0).count();
    if ink == 0 {
        return Err("icon raster is empty".into());
    }
    Ok(pixmap)
}

fn paint(svg: &str) -> String {
    // usvg leaves currentColor unresolved, which drops the stroke. A template
    // image has to be black so AppKit can tint it.
    svg.replace("currentColor", "#000000")
}

fn draw(tree: &resvg::usvg::Tree, px: u32) -> Result<resvg::tiny_skia::Pixmap, String> {
    let size = tree.size();
    let width = size.width();
    let height = size.height();
    if width <= 0.0 || height <= 0.0 {
        return Err("svg has no size".into());
    }
    let scale = px as f32 / width;
    let w = (width * scale).ceil().max(1.0) as u32;
    let h = (height * scale).ceil().max(1.0) as u32;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h).ok_or("pixmap")?;
    resvg::render(
        tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_toolbar_icon_rasterizes_to_ink() {
        for svg in [
            LIST,
            BOOKMARK,
            SEARCH,
            BOOKMARK_PLUS,
            NOTE,
            BOOK,
            FOLDER,
            LIBRARY,
        ] {
            let pixmap = raster(svg, 32).unwrap();
            let ink = pixmap.pixels().iter().filter(|px| px.alpha() > 0).count();
            assert!(ink > 20, "ink pixels {ink}");
        }
    }
}
