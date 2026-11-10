use std::cell::RefCell;
use std::rc::Rc;

use nsh_platform_macos::objc2::rc::Retained;
use nsh_platform_macos::objc2::{define_class, msg_send, AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly};
use nsh_platform_macos::objc2_app_kit::{NSResponder, NSView};
use nsh_platform_macos::objc2_foundation::{NSObjectProtocol, NSPoint, NSRect, NSSize, NSString};
use objc2_app_kit::{
    NSAttributedStringNSStringDrawing, NSBezierPath, NSBitmapImageFileType, NSBitmapImageRep, NSColor,
    NSFont, NSFontAttributeName, NSForegroundColorAttributeName, NSImage, NSImageSymbolConfiguration,
    NSLineBreakMode, NSMutableParagraphStyle, NSParagraphStyleAttributeName, NSTextAlignment,
};
use objc2_foundation::{NSDictionary, NSMutableAttributedString, NSRange};

use crate::panels::{Layout, Line, Link, Note, PlacedCard, Rect as Band, RightMode, Thread, Variant};

#[derive(Clone)]
pub struct Paint {
    pub variant: Variant,
    pub chinese: bool,
    pub hover: f64,
    pub mockup: String,
    pub bare: bool,
    pub page: Band,
    pub left: Option<Band>,
    pub right: Option<Band>,
    pub drawer: Option<Band>,
    pub notes: Vec<Note>,
    pub placed: Vec<PlacedCard>,
    pub plate: String,
    pub ready: bool,
    pub window_w: f64,
    pub window_h: f64,
    pub layout: Layout,
}

impl Default for Paint {
    fn default() -> Self {
        Self {
            variant: Variant::Card,
            chinese: false,
            hover: 0.0,
            mockup: String::new(),
            bare: false,
            page: Band { x: 0.0, y: 0.0, w: 0.0, h: 0.0 },
            left: None,
            right: None,
            drawer: None,
            notes: Vec::new(),
            placed: Vec::new(),
            plate: String::new(),
            ready: false,
            window_w: 0.0,
            window_h: 0.0,
            layout: Layout {
                max_columns: 1,
                left_docked: false,
                right: RightMode::Closed,
            },
        }
    }
}

struct Ivars {
    paint: Rc<RefCell<Paint>>,
    plate: RefCell<Option<(String, Retained<NSImage>)>>,
}

define_class!(
    #[unsafe(super(NSView, NSResponder, objc2::runtime::NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = Ivars]
    struct OverlayView;

    unsafe impl NSObjectProtocol for OverlayView {}

    impl OverlayView {
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool {
            true
        }

        #[unsafe(method(isOpaque))]
        fn is_opaque(&self) -> bool {
            false
        }

        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, _dirty: NSRect) {
            let paint = self.ivars().paint.borrow().clone();
            draw_paint(&paint, &self.ivars().plate);
        }
    }
);

pub fn install(parent: &NSView, paint: Rc<RefCell<Paint>>) -> Retained<NSView> {
    let mtm = MainThreadMarker::new().expect("main");
    let view = mtm.alloc::<OverlayView>().set_ivars(Ivars {
        paint,
        plate: RefCell::new(None),
    });
    let view: Retained<OverlayView> = unsafe {
        msg_send![
            super(view),
            initWithFrame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(10.0, 10.0))
        ]
    };
    unsafe {
        let _: () = msg_send![&*view, setWantsLayer: true];
    }
    let ns = Retained::into_super(view);
    unsafe {
        let _: () = msg_send![parent, addSubview: &*ns];
    }
    ns
}

pub fn place(overlay: &NSView, bounds: NSRect) {
    unsafe {
        let _: () = msg_send![overlay, setFrame: bounds];
    }
}

pub fn export_png(paint: &Paint, path: &str) -> bool {
    if paint.window_w < 2.0 || paint.window_h < 2.0 || path.is_empty() {
        return false;
    }
    let image = NSImage::initWithSize(
        NSImage::alloc(),
        NSSize::new(paint.window_w, paint.window_h),
    );
    image.lockFocusFlipped(true);
    paper_color().setFill();
    NSBezierPath::fillRect(NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(paint.window_w, paint.window_h),
    ));
    let slot = RefCell::new(None);
    draw_paint(paint, &slot);
    image.unlockFocus();
    let Some(tiff) = image.TIFFRepresentation() else {
        return false;
    };
    let Some(rep) = NSBitmapImageRep::imageRepWithData(&tiff) else {
        return false;
    };
    let props = NSDictionary::new();
    let png = unsafe { rep.representationUsingType_properties(NSBitmapImageFileType::PNG, &props) };
    let Some(png) = png else {
        return false;
    };
    std::fs::write(path, png.to_vec()).is_ok()
}

fn draw_paint(paint: &Paint, plate_slot: &RefCell<Option<(String, Retained<NSImage>)>>) {
    if paint.page.w < 2.0 {
        return;
    }
    if !paint.plate.is_empty() {
        draw_plate(paint, plate_slot);
    }
    if paint.bare {
        draw_toolbar(paint);
        draw_badge(paint);
        return;
    }
    let panel = paint.drawer.or(paint.right);
    draw_highlights(paint);
    if paint.variant.link() == Link::Hairline {
        if let Some(panel) = panel {
            draw_connectors(paint, panel);
        }
    }
    if let Some(left) = paint.left {
        draw_left(left);
    }
    if let Some(panel) = panel {
        let floating = paint.drawer.is_some();
        draw_panel(panel, floating);
        draw_notes(paint, panel);
        draw_input(paint, panel);
    }
    draw_toolbar(paint);
    draw_badge(paint);
}

fn draw_plate(paint: &Paint, plate_slot: &RefCell<Option<(String, Retained<NSImage>)>>) {
    let mut slot = plate_slot.borrow_mut();
    let stale = slot.as_ref().is_none_or(|(path, _)| path != &paint.plate);
    if stale {
        let loaded = NSImage::initWithContentsOfFile(NSImage::alloc(), &NSString::from_str(&paint.plate));
        *slot = loaded.map(|image| (paint.plate.clone(), image));
    }
    let Some((_, image)) = slot.as_ref() else {
        return;
    };
    image.drawInRect(ns(paint.page));
}

fn draw_highlights(paint: &Paint) {
    let alpha = if paint.variant.link() == Link::Hairline { 0.16 } else { 0.42 };
    let color = NSColor::colorWithRed_green_blue_alpha(0.96, 0.72, 0.18, alpha);
    for note in &paint.notes {
        let band = line_band(paint.page, &note.line);
        fill_round(inflate(ns(band), 1.0, 1.0), 3.0, &color);
    }
}

fn draw_connectors(paint: &Paint, panel: Band) {
    let color = NSColor::colorWithRed_green_blue_alpha(0.35, 0.30, 0.24, 0.55);
    color.setStroke();
    for note in &paint.notes {
        let Some(placed) = paint.placed.iter().find(|item| item.id == note.id) else {
            continue;
        };
        let line = line_band(paint.page, &note.line);
        let y0 = line.y + line.h * 0.5;
        let x0 = line.x + line.w;
        let x1 = panel.x + 8.0;
        let y1 = placed.y + 16.0;
        let mid = x0 + ((x1 - x0) * 0.55).max(8.0);
        stroke_line(x0, y0, mid, y0);
        stroke_line(mid, y0, mid, y1);
        stroke_line(mid, y1, x1, y1);
    }
}

fn draw_left(bar: Band) {
    let paper = paper_color();
    paper.setFill();
    NSBezierPath::fillRect(ns(bar));
    let hair = NSColor::colorWithRed_green_blue_alpha(0.0, 0.0, 0.0, 0.12);
    hair.setStroke();
    stroke_line(bar.x + bar.w - 0.5, 0.0, bar.x + bar.w - 0.5, bar.h);
    let items = [
        ("book.closed", "Now reading", true),
        ("books.vertical", "Library", false),
        ("person.2", "Agents", false),
    ];
    let mut y = 62.0;
    for (index, (symbol, label, on)) in items.iter().enumerate() {
        if index == 1 {
            y += 14.0;
        }
        if *on {
            let pill = Band { x: 8.0, y: y - 4.0, w: bar.w - 16.0, h: 52.0 };
            fill_round(ns(pill), 10.0, &NSColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.72));
        }
        draw_symbol(symbol, Band { x: (bar.w - 20.0) / 2.0, y, w: 20.0, h: 20.0 }, 15.0);
        draw_text(
            label,
            Band { x: 4.0, y: y + 22.0, w: bar.w - 8.0, h: 14.0 },
            10.0,
            0.23,
            &ink(0.72),
            false,
            NSTextAlignment::Center,
        );
        y += 58.0;
    }
}

fn draw_panel(panel: Band, floating: bool) {
    if floating {
        let shadow = NSColor::colorWithRed_green_blue_alpha(0.15, 0.12, 0.08, 0.16);
        fill_round(inflate(ns(panel), 6.0, 8.0), 18.0, &shadow);
        fill_round(ns(panel), 16.0, &paper_color());
    } else {
        paper_color().setFill();
        NSBezierPath::fillRect(ns(panel));
        let hair = NSColor::colorWithRed_green_blue_alpha(0.0, 0.0, 0.0, 0.12);
        hair.setStroke();
        stroke_line(panel.x + 0.5, 0.0, panel.x + 0.5, panel.h);
    }
    draw_text(
        "Notes",
        Band { x: panel.x + 16.0, y: panel.y + 58.0, w: 80.0, h: 18.0 },
        13.0,
        0.3,
        &ink(1.0),
        false,
        NSTextAlignment::Left,
    );
    let chip_band = Band { x: panel.x + panel.w - 78.0, y: panel.y + 56.0, w: 62.0, h: 20.0 };
    fill_round(ns(chip_band), 10.0, &NSColor::colorWithRed_green_blue_alpha(0.25, 0.2, 0.14, 0.08));
    draw_text("Sample", chip_band, 11.0, 0.23, &ink(0.8), false, NSTextAlignment::Center);
}

fn draw_notes(paint: &Paint, panel: Band) {
    let input_top = panel.y + panel.h - 60.0;
    for note in &paint.notes {
        let Some(placed) = paint.placed.iter().find(|item| item.id == note.id) else {
            continue;
        };
        if placed.y > input_top {
            continue;
        }
        let card = Band {
            x: panel.x + 14.0,
            y: placed.y,
            w: panel.w - 28.0,
            h: placed.h.min(input_top - placed.y),
        };
        draw_card(paint, note, card, placed.snapped);
    }
}

fn draw_card(paint: &Paint, note: &Note, card: Band, open: bool) {
    match paint.variant {
        Variant::Card => {
            fill_round(ns(card), 10.0, &NSColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.78));
        }
        Variant::Quote => {
            let bar = NSColor::colorWithRed_green_blue_alpha(0.42, 0.34, 0.24, 0.45);
            bar.setFill();
            NSBezierPath::fillRect(ns(Band { x: card.x, y: card.y + 2.0, w: 2.0, h: card.h - 4.0 }));
        }
        Variant::Margin => {}
    }
    let inset = if paint.variant == Variant::Quote { 12.0 } else { 10.0 };
    let mut y = card.y + 8.0;
    let text_w = card.w - inset - 10.0;
    let x = card.x + inset;
    if open && paint.variant == Variant::Quote {
        let quote = clip_chars(&note.line.text, 90);
        draw_text(
            &quote,
            Band { x, y, w: text_w, h: 34.0 },
            12.0,
            0.0,
            &ink(0.62),
            true,
            NSTextAlignment::Left,
        );
        y += 36.0;
    }
    let body_h = if open { 36.0 } else { card.h - 12.0 };
    draw_text(
        &note.body,
        Band { x, y, w: text_w, h: body_h.max(16.0) },
        13.0,
        0.0,
        &ink(1.0),
        true,
        NSTextAlignment::Left,
    );
    if !open {
        return;
    }
    y += 40.0;
    match paint.variant.thread() {
        Thread::Plain => {
            draw_text(
                &note.reply,
                Band { x, y, w: text_w, h: 36.0 },
                12.0,
                0.0,
                &ink(0.62),
                true,
                NSTextAlignment::Left,
            );
        }
        Thread::Nested => {
            let nest = Band { x: x - 4.0, y, w: text_w + 8.0, h: 52.0 };
            fill_round(ns(nest), 8.0, &NSColor::colorWithRed_green_blue_alpha(0.2, 0.16, 0.1, 0.06));
            draw_text(
                &note.reply,
                Band { x: nest.x + 8.0, y: nest.y + 6.0, w: nest.w - 16.0, h: 40.0 },
                12.0,
                0.0,
                &ink(0.75),
                true,
                NSTextAlignment::Left,
            );
        }
        Thread::Ruled => {
            let hair = NSColor::colorWithRed_green_blue_alpha(0.0, 0.0, 0.0, 0.12);
            hair.setStroke();
            stroke_line(x, y, x + text_w, y);
            draw_text(
                &note.reply,
                Band { x, y: y + 8.0, w: text_w, h: 36.0 },
                12.0,
                0.0,
                &ink(0.62),
                true,
                NSTextAlignment::Left,
            );
        }
    }
}

fn draw_input(paint: &Paint, panel: Band) {
    let field = Band {
        x: panel.x + 14.0,
        y: panel.y + panel.h - 52.0,
        w: panel.w - 28.0,
        h: 36.0,
    };
    fill_round(ns(field), 10.0, &NSColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.7));
    let placeholder = if paint.chinese { "问这一页" } else { "Ask about this page" };
    draw_text(
        placeholder,
        Band { x: field.x + 12.0, y: field.y + 8.0, w: field.w - 24.0, h: 20.0 },
        13.0,
        0.0,
        &ink(0.45),
        false,
        NSTextAlignment::Left,
    );
}

fn draw_toolbar(paint: &Paint) {
    if paint.hover < 0.04 {
        return;
    }
    let y = 16.0;
    let mut x = if paint.left.is_some() { 88.0 } else { 80.0 };
    for symbol in ["list.bullet", "bookmark", "magnifyingglass"] {
        draw_symbol_alpha(symbol, Band { x, y, w: 18.0, h: 18.0 }, 15.0, paint.hover);
        x += 32.0;
    }
    let notes_x = paint.window_w - 36.0;
    draw_symbol_alpha(
        "bookmark.fill",
        Band { x: notes_x - 32.0, y, w: 18.0, h: 18.0 },
        15.0,
        paint.hover,
    );
    draw_symbol_alpha("note.text", Band { x: notes_x, y, w: 18.0, h: 18.0 }, 15.0, paint.hover);
}

fn draw_badge(paint: &Paint) {
    if paint.mockup.is_empty() {
        return;
    }
    let width = 188.0;
    let below = paint.page.y + paint.page.h + 8.0;
    let covers_a_card = paint.drawer.is_some_and(|drawer| {
        let x = (paint.window_w - width) / 2.0;
        x + width > drawer.x + 8.0
    });
    let (x, y) = if covers_a_card {
        ((paint.window_w - width) / 2.0, 8.0)
    } else if below + 22.0 < paint.window_h {
        ((paint.window_w - width) / 2.0, below)
    } else {
        ((paint.window_w - width) / 2.0, paint.window_h - 28.0)
    };
    let band = Band { x, y, w: width, h: 20.0 };
    fill_round(ns(band), 10.0, &NSColor::colorWithRed_green_blue_alpha(0.95, 0.93, 0.89, 0.92));
    draw_text(&paint.mockup, band, 11.0, 0.23, &ink(0.85), false, NSTextAlignment::Center);
}

fn line_band(page: Band, line: &Line) -> Band {
    Band {
        x: page.x + line.x,
        y: page.y + line.y,
        w: line.w,
        h: line.h,
    }
}

fn draw_symbol(name: &str, band: Band, size: f64) {
    draw_symbol_alpha(name, band, size, 1.0);
}

fn draw_symbol_alpha(name: &str, band: Band, size: f64, alpha: f64) {
    let Some(image) = NSImage::imageWithSystemSymbolName_accessibilityDescription(&NSString::from_str(name), None) else {
        return;
    };
    let config = NSImageSymbolConfiguration::configurationWithPointSize_weight(size, 0.0);
    let tint = NSImageSymbolConfiguration::configurationWithHierarchicalColor(&ink(alpha));
    let config = config.configurationByApplyingConfiguration(&tint);
    let Some(image) = image.imageWithSymbolConfiguration(&config) else {
        return;
    };
    image.drawInRect(ns(band));
}

fn draw_text(text: &str, band: Band, size: f64, weight: f64, color: &NSColor, wrap: bool, align: NSTextAlignment) {
    if text.is_empty() || band.w < 2.0 || band.h < 2.0 {
        return;
    }
    let font = NSFont::systemFontOfSize_weight(size, weight);
    let style = NSMutableParagraphStyle::new();
    style.setAlignment(align);
    style.setLineBreakMode(if wrap { NSLineBreakMode::ByWordWrapping } else { NSLineBreakMode::ByTruncatingTail });
    let string = NSString::from_str(text);
    let attr = NSMutableAttributedString::initWithString(NSMutableAttributedString::alloc(), &string);
    let range = NSRange { location: 0, length: string.length() };
    unsafe {
        attr.addAttribute_value_range(NSFontAttributeName, font.as_ref(), range);
        attr.addAttribute_value_range(NSForegroundColorAttributeName, color, range);
        attr.addAttribute_value_range(NSParagraphStyleAttributeName, style.as_ref(), range);
    }
    attr.drawInRect(ns(band));
}

fn paper_color() -> Retained<NSColor> {
    NSColor::colorWithRed_green_blue_alpha(243.0 / 255.0, 238.0 / 255.0, 228.0 / 255.0, 1.0)
}

fn ink(alpha: f64) -> Retained<NSColor> {
    NSColor::colorWithRed_green_blue_alpha(0.16, 0.13, 0.10, alpha)
}

fn fill_round(rect: NSRect, radius: f64, color: &NSColor) {
    let path = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(rect, radius, radius);
    color.setFill();
    path.fill();
}

fn stroke_line(x0: f64, y0: f64, x1: f64, y1: f64) {
    let path = NSBezierPath::bezierPath();
    path.setLineWidth(1.0);
    path.moveToPoint(NSPoint::new(x0, y0));
    path.lineToPoint(NSPoint::new(x1, y1));
    path.stroke();
}

fn ns(band: Band) -> NSRect {
    NSRect::new(NSPoint::new(band.x, band.y), NSSize::new(band.w.max(0.0), band.h.max(0.0)))
}

fn inflate(rect: NSRect, dx: f64, dy: f64) -> NSRect {
    NSRect::new(
        NSPoint::new(rect.origin.x - dx, rect.origin.y - dy),
        NSSize::new(rect.size.width + dx * 2.0, rect.size.height + dy * 2.0),
    )
}

fn clip_chars(text: &str, max: usize) -> String {
    let trimmed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if trimmed.chars().count() <= max {
        return trimmed;
    }
    let clipped: String = trimmed.chars().take(max).collect();
    format!("{clipped}…")
}
