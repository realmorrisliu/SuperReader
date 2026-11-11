use nsh_platform_macos::objc2::rc::Retained;
use nsh_platform_macos::objc2::AnyThread;
use objc2_app_kit::NSImage;
use objc2_foundation::{NSData, NSSize};

const LIST: &str = include_str!("../../../assets/icons/list.svg");
const SETTINGS: &str = include_str!("../../../assets/icons/settings.svg");
const SETTINGS_SOLID: &str = include_str!("../../../assets/icons/settings-solid.svg");
const PLUS: &str = include_str!("../../../assets/icons/plus.svg");
const PLUS_SOLID: &str = include_str!("../../../assets/icons/plus-solid.svg");
const FILE: &str = include_str!("../../../assets/icons/file.svg");
const FILE_SOLID: &str = include_str!("../../../assets/icons/file-solid.svg");
const CHECK: &str = include_str!("../../../assets/icons/check.svg");
const CHECK_SOLID: &str = include_str!("../../../assets/icons/check-solid.svg");
const BOOK_OPEN: &str = include_str!("../../../assets/icons/book-open.svg");
const BOOK_OPEN_SOLID: &str = include_str!("../../../assets/icons/book-open-solid.svg");
const SIDEBAR_RIGHT: &str = include_str!("../../../assets/icons/sidebar-right.svg");
const LIBRARY_SOLID: &str = include_str!("../../../assets/icons/library-solid.svg");
const BOOKMARK_SOLID: &str = include_str!("../../../assets/icons/bookmark-solid.svg");
const NOTE_SOLID: &str = include_str!("../../../assets/icons/note-solid.svg");
const FOLDER_SOLID: &str = include_str!("../../../assets/icons/folder-solid.svg");
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

pub fn glyph_file(name: &str) -> Option<std::path::PathBuf> {
    let svg = named_svg(name)?;
    let dir = crate::settings::support_dir().join("shell-glyphs");
    std::fs::create_dir_all(&dir).ok()?;
    let path = dir.join(format!("{name}.png"));
    if !path.is_file() {
        let png = raster_png(svg, 48).ok()?;
        std::fs::write(&path, png).ok()?;
    }
    Some(path)
}

fn named_svg(name: &str) -> Option<&'static str> {
    Some(match name {
        "list" => LIST,
        "search" => SEARCH,
        "bookmark" => BOOKMARK,
        "bookmark-plus" => BOOKMARK_PLUS,
        "note" => NOTE,
        "book" => BOOK,
        "folder" => FOLDER,
        "library" => LIBRARY,
        "settings" => SETTINGS,
        "settings-solid" => SETTINGS_SOLID,
        "plus" => PLUS,
        "plus-solid" => PLUS_SOLID,
        "file" => FILE,
        "file-solid" => FILE_SOLID,
        "check" => CHECK,
        "check-solid" => CHECK_SOLID,
        "book-open" => BOOK_OPEN,
        "book-open-solid" => BOOK_OPEN_SOLID,
        "sidebar-right" => SIDEBAR_RIGHT,
        "library-solid" => LIBRARY_SOLID,
        "bookmark-solid" => BOOKMARK_SOLID,
        "note-solid" => NOTE_SOLID,
        "folder-solid" => FOLDER_SOLID,
        _ => return None,
    })
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
