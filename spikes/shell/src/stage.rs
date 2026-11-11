use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use nsh_platform_macos::objc2::rc::Retained;
use nsh_platform_macos::objc2::runtime::AnyObject;
use nsh_platform_macos::objc2::{
    define_class, msg_send, sel, DefinedClass, MainThreadMarker, MainThreadOnly,
};
use nsh_platform_macos::objc2_app_kit::{NSEvent, NSResponder, NSView, NSWindow, NSWindowButton};
use nsh_platform_macos::objc2_foundation::{NSObjectProtocol, NSPoint, NSRect, NSSize};
use nsh_platform_macos::{ComponentContext, Components, NativeComponent};
use objc2_core_graphics::CGContext;
use serde_json::{json, Value};
use spike_pdf::{paint_highlight, select_points, Document, PageText, Rgba};
use spike_report::{Anchor, PdfEngine, PixelScale};

use crate::bitmap;

struct Ivars {
    state: Rc<RefCell<State>>,
}

define_class!(
    #[unsafe(super(NSView, NSResponder, objc2::runtime::NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = Ivars]
    struct StageView;

    unsafe impl NSObjectProtocol for StageView {}

    impl StageView {
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool {
            true
        }

        #[unsafe(method(pumpChrome))]
        fn pump_chrome(&self) {
            let _: () = unsafe { msg_send![self, setNeedsDisplay: true] };
        }

        #[unsafe(method(pumpLights))]
        fn pump_lights(&self) {
            crate::launch::install_document_open();
            let window: Option<Retained<NSWindow>> = unsafe { msg_send![self, window] };
            if let Some(window) = window {
                let queued = crate::launch::take_opens();
                if !queued.is_empty() {
                    let mut state = self.ivars().state.borrow_mut();
                    for path in queued {
                        state.open_requested(self, &path);
                    }
                }
                let (chrome, page, before) = {
                    let state = self.ivars().state.borrow();
                    (
                        state.chrome.clone(),
                        state.page_box(),
                        (
                            state.chrome.bar_w.get(),
                            state.chrome.bar_gen.get(),
                            state.chrome.bar_dirty.get(),
                        ),
                    )
                };
                crate::chrome::sync(&window, &chrome, page);
                let after = (chrome.bar_w.get(), chrome.bar_gen.get(), chrome.bar_dirty.get());
                if before != after {
                    let _: () = unsafe { msg_send![self, setNeedsDisplay: true] };
                }
                if crate::chrome::shell_on() {
                    let state = self.ivars().state.borrow();
                    let key = (
                        state.stage_w.round() as i32,
                        state.stage_h.round() as i32,
                        state.chrome.bar_w.get().round() as i32,
                    );
                    drop(state);
                    thread_local! {
                        static LAST: std::cell::Cell<(i32, i32, i32)> = const { std::cell::Cell::new((0, 0, 0)) };
                    }
                    let changed = LAST.with(|last| {
                        if last.get() == key || key.0 < 2 || key.1 < 2 {
                            false
                        } else {
                            last.set(key);
                            true
                        }
                    });
                    if changed {
                        self.ivars().state.borrow().publish();
                    }
                }
                self.ivars().state.borrow().note_probe(self);
            }
            let _: () = unsafe {
                msg_send![
                    self,
                    performSelector: sel!(pumpLights),
                    withObject: std::ptr::null::<objc2::runtime::AnyObject>(),
                    afterDelay: 0.05_f64
                ]
            };
        }

        #[unsafe(method(pollSettings))]
        fn poll_settings(&self) {
            let changed = self.ivars().state.borrow_mut().reload_trim_from_file();
            if changed {
                let _: () = unsafe { msg_send![self, setNeedsDisplay: true] };
            }
            let _: () = unsafe {
                msg_send![
                    self,
                    performSelector: sel!(pollSettings),
                    withObject: std::ptr::null::<objc2::runtime::AnyObject>(),
                    afterDelay: 0.4_f64
                ]
            };
        }

        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, _dirty: NSRect) {
            let state = self.ivars().state.clone();
            state.borrow_mut().draw(self);
        }

        #[unsafe(method(layout))]
        fn layout_view(&self) {
            let _: () = unsafe { msg_send![super(self), layout] };
            let bounds: NSRect = unsafe { msg_send![self, bounds] };
            let window: Option<Retained<NSWindow>> = unsafe { msg_send![self, window] };
            let fullscreen = window.as_ref().is_some_and(|window| {
                window.styleMask().contains(objc2_app_kit::NSWindowStyleMask::FullScreen)
            });
            let (chrome, page) = {
                let mut state = self.ivars().state.borrow_mut();
                if state.chrome.full.get() != fullscreen {
                    state.chrome.mark_bar_dirty();
                }
                state.chrome.full.set(fullscreen);
                state.stage_w = bounds.size.width;
                state.stage_h = bounds.size.height;
                state.chrome.laying_out.set(true);
                state.place_page();
                (state.chrome.clone(), state.page_box())
            };
            if let Some(window) = window {
                crate::chrome::sync(&window, &chrome, page);
            }
            chrome.laying_out.set(false);
        }

        #[unsafe(method(scrollWheel:))]
        fn scroll_wheel(&self, event: &NSEvent) {
            let dy: f64 = unsafe { msg_send![event, scrollingDeltaY] };
            self.ivars().state.borrow_mut().scroll(dy);
            unsafe { msg_send![self, setNeedsDisplay: true] }
        }

        #[unsafe(method(magnifyWithEvent:))]
        fn magnify(&self, event: &NSEvent) {
            let mag: f64 = unsafe { msg_send![event, magnification] };
            self.ivars().state.borrow_mut().pinch(mag);
            unsafe { msg_send![self, setNeedsDisplay: true] }
        }

        #[unsafe(method(mouseDown:))]
        fn mouse_down(&self, event: &NSEvent) {
            let point = point_in(self, event);
            let placed = self
                .ivars()
                .state
                .borrow()
                .placed
                .as_ref()
                .map(|item| (item.0, item.1, item.2, item.3));
            if crate::chrome::should_drag_margin(point.0, point.1, placed) {
                let window: Option<Retained<NSWindow>> = unsafe { msg_send![self, window] };
                if let Some(window) = window {
                    let _: () = unsafe { msg_send![&*window, performWindowDragWithEvent: event] };
                }
                return;
            }
            self.ivars().state.borrow_mut().drag = Some((point, point));
        }

        #[unsafe(method(mouseDragged:))]
        fn mouse_dragged(&self, event: &NSEvent) {
            let point = point_in(self, event);
            if let Some((start, _)) = self.ivars().state.borrow().drag {
                self.ivars().state.borrow_mut().drag = Some((start, point));
            }
            unsafe { msg_send![self, setNeedsDisplay: true] }
        }

        #[unsafe(method(mouseUp:))]
        fn mouse_up(&self, event: &NSEvent) {
            let point = point_in(self, event);
            let state = self.ivars().state.clone();
            let mut state = state.borrow_mut();
            if let Some((start, _)) = state.drag.take() {
                state.finish_drag(start, point);
            }
            drop(state);
            unsafe { msg_send![self, setNeedsDisplay: true] }
        }
    }
);

fn opened_at_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|stamp| stamp.as_millis() as i64)
        .unwrap_or(0)
}

fn point_in(view: &StageView, event: &NSEvent) -> (f64, f64) {
    let window_point: NSPoint = unsafe { msg_send![event, locationInWindow] };
    let point: NSPoint = unsafe {
        msg_send![view, convertPoint: window_point, fromView: std::ptr::null::<NSView>()]
    };
    (point.x, point.y)
}

struct PdfView {
    doc: Document,
    sizes: Vec<(f32, f32)>,
    text: HashMap<usize, PageText>,
    cache: HashMap<(usize, u8), Rgba>,
    frame_ms: Vec<f64>,
}

struct State {
    context: ComponentContext,
    mode: Mode,
    scale: PixelScale,
    offset: f64,
    pinch: f64,
    drag: Option<((f64, f64), (f64, f64))>,
    pdf: Option<PdfView>,
    book_kind: &'static str,
    book_path: String,
    product: bool,
    saved_page: u32,
    page_total: Option<u32>,
    epub_page: usize,
    anchor: Option<Anchor>,
    anchor_page: Option<u32>,
    web: Option<crate::web::WebChapter>,
    web_pages: usize,
    trim_id: String,
    declared: Option<(f64, f64)>,
    stage_w: f64,
    stage_h: f64,
    placed: Option<(i32, i32, i32, i32, String)>,
    placed_zoom: f64,
    await_hold: bool,
    hold_ready: bool,
    settled: bool,
    hold_since: Option<Instant>,
    shown_at: Option<Instant>,
    footer_at: Option<Instant>,
    play_count: u8,
    play_at: Option<Instant>,
    turn_pending: bool,
    seen_frames: usize,
    goto: Option<usize>,
    title_settled: bool,
    chrome: Rc<crate::chrome::Runtime>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Empty,
    Pdfium,
    Hayro,
    WebKit,
    Blitz,
    Text,
}

impl State {
    fn paint_shell(&self, bounds: NSRect) {
        let Some(ctx) = current_context() else {
            return;
        };
        unsafe {
            let (red, green, blue) = if crate::chrome::shell_on() {
                (235.0 / 255.0, 235.0 / 255.0, 238.0 / 255.0)
            } else {
                crate::reading::paper_rgb()
            };
            CGContext::set_rgb_fill_color(Some(&ctx), red, green, blue, 1.0);
            CGContext::fill_rect(Some(&ctx), bounds);
            let bar = self.chrome.bar_w.get();
            if bar > 1.0 && !crate::chrome::shell_on() {
                CGContext::set_rgb_fill_color(
                    Some(&ctx),
                    228.0 / 255.0,
                    228.0 / 255.0,
                    230.0 / 255.0,
                    1.0,
                );
                CGContext::fill_rect(
                    Some(&ctx),
                    NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(bar, bounds.size.height)),
                );
            }
        }
    }

    fn note_probe(&self, view: &StageView) {
        let window: Option<Retained<NSWindow>> = unsafe { msg_send![view, window] };
        let Some(window) = window else {
            return;
        };
        let (mode, pages, page) = match self.mode {
            Mode::Empty => ("empty", 0, 0),
            Mode::WebKit => ("epub", self.web_pages, self.epub_page + 1),
            Mode::Pdfium | Mode::Hayro => (
                "pdf",
                self.pdf.as_ref().map(|pdf| pdf.sizes.len()).unwrap_or(0),
                self.current_page() + 1,
            ),
            Mode::Text | Mode::Blitz => ("other", 0, 0),
        };
        let close_w = window
            .standardWindowButton(NSWindowButton::CloseButton)
            .map(|button| {
                let view: &NSView = unsafe { &*std::ptr::from_ref(&*button).cast::<NSView>() };
                view.frame().size.width
            })
            .unwrap_or(0.0);
        let toolbar = window.toolbar().is_some_and(|toolbar| {
            toolbar
                .items()
                .iter()
                .any(|item| item.itemIdentifier().to_string() == "books.contents")
        });
        let body = json!({
            "window_title": window.title().to_string(),
            "mode": mode,
            "path": self.book_path,
            "page": page,
            "pages": pages,
            "bar_w": self.chrome.bar_w.get(),
            "lights": self.chrome.lights.get(),
            "close_w": close_w,
            "toolbar": toolbar,
            "product": self.product,
        });
        let dir = "/tmp/superreader-spike";
        let _ = std::fs::create_dir_all(dir);
        let text = serde_json::to_string(&body).unwrap_or_default();
        let path = format!("{dir}/reader.json");
        let tmp = format!("{path}.tmp");
        if std::fs::write(&tmp, text).is_ok() {
            let _ = std::fs::rename(&tmp, path);
        }
    }

    fn draw(&mut self, view: &StageView) {
        self.poll_web();
        self.note_probe(view);
        let bounds: NSRect = unsafe { msg_send![view, bounds] };
        self.paint_shell(bounds);
        if self.mode == Mode::Empty {
            return;
        }
        if self.mode == Mode::WebKit {
            self.stage_w = bounds.size.width;
            self.stage_h = bounds.size.height;
            self.place_page();
            let playing = std::env::var("SPIKE_PLAY").as_deref() == Ok("1")
                && self.play_count < 3
                && !self.turn_pending;
            let need_page = self.goto.is_some_and(|target| {
                self.web
                    .as_ref()
                    .and_then(|web| web.inbox.borrow().pages)
                    .map(|(page, _)| page < target)
                    .unwrap_or(true)
            });
            if self.chrome_moving() || playing || need_page {
                let _: () = unsafe {
                    msg_send![
                        view,
                        performSelector: sel!(pumpChrome),
                        withObject: std::ptr::null::<objc2::runtime::AnyObject>(),
                        afterDelay: 0.016_f64
                    ]
                };
            }
            return;
        }
        let mtm = MainThreadMarker::new().expect("main");
        let ctx = unsafe {
            let current: Option<Retained<objc2_app_kit::NSGraphicsContext>> =
                msg_send![objc2::class!(NSGraphicsContext), currentContext];
            current.and_then(|g| -> Option<Retained<CGContext>> { msg_send![&*g, CGContext] })
        };
        let Some(ctx) = ctx else {
            return;
        };
        let _ = mtm;
        if !matches!(self.mode, Mode::Pdfium | Mode::Hayro) {
            return;
        }
        if self.web.is_some() {
            self.dismiss_web();
        }
        if self.product {
            self.draw_fitted_pdf(&ctx, bounds);
            return;
        }
        let (origin_x, origin_y) = self.pdf_origin();
        let display = self.scale.display() as f64;
        let Some(pdf) = self.pdf.as_mut() else {
            return;
        };
        let mut y = origin_y - self.offset;
        for index in 0..pdf.sizes.len() {
            let (pw, ph) = pdf.sizes[index];
            let dw = pw as f64 * display;
            let dh = ph as f64 * display;
            let next = y + dh;
            if next > 0.0 && y < bounds.size.height {
                let key = (index, scale_key(self.scale));
                if !pdf.cache.contains_key(&key) {
                    let started = std::time::Instant::now();
                    if let Ok(image) = pdf.doc.raster(index, self.scale) {
                        pdf.frame_ms.push(started.elapsed().as_secs_f64() * 1000.0);
                        pdf.cache.insert(key, image);
                    }
                }
                if let Some(image) = pdf.cache.get(&key).cloned() {
                    let mut shown = image;
                    if self.anchor_page == Some(index as u32) {
                        if let Some(Anchor::Pdf { quads, .. }) = &self.anchor {
                            paint_highlight(&mut shown, quads, self.scale);
                        }
                    }
                    let bytes = bitmap::flip_rows(shown.width, &shown.bytes);
                    if let Some(cg) = bitmap::rgba_image(
                        shown.width,
                        shown.height,
                        bytes,
                        matches!(self.mode, Mode::Hayro),
                    ) {
                        let rect = NSRect::new(NSPoint::new(origin_x, y), NSSize::new(dw, dh));
                        unsafe { CGContext::draw_image(Some(&ctx), rect, Some(&cg)) };
                    }
                }
            }
            y = next + 16.0;
            if y > bounds.size.height + 40.0 {
                break;
            }
        }
    }

    fn scroll(&mut self, dy: f64) {
        if matches!(self.mode, Mode::Pdfium | Mode::Hayro) {
            self.offset = (self.offset - dy).max(0.0);
            self.note_page();
            self.publish();
        }
    }

    fn pinch(&mut self, mag: f64) {
        self.pinch += mag;
        if self.pinch > 0.12 {
            self.set_scale(PixelScale::Zoom2x);
            self.pinch = 0.0;
        } else if self.pinch < -0.12 {
            self.set_scale(PixelScale::Retina1x);
            self.pinch = 0.0;
        }
    }

    fn set_scale(&mut self, scale: PixelScale) {
        if self.scale == scale {
            return;
        }
        self.scale = scale;
        if let Some(pdf) = self.pdf.as_mut() {
            pdf.cache.clear();
        }
        self.publish();
    }

    fn finish_drag(&mut self, start: (f64, f64), end: (f64, f64)) {
        let display = self.scale.display() as f64;
        let (origin_x, origin_y) = self.pdf_origin();
        let Some(pdf) = self.pdf.as_mut() else {
            return;
        };
        let (page, x0, y0) = match locate(
            pdf,
            self.offset,
            start.0,
            start.1,
            display,
            origin_x,
            origin_y,
        ) {
            Some(v) => v,
            None => return,
        };
        let Some((_, x1, y1)) = locate(pdf, self.offset, end.0, end.1, display, origin_x, origin_y)
        else {
            return;
        };
        let text = pdf.text.entry(page).or_insert_with(|| pdf.doc.text(page));
        if let Some(mut anchor) = select_points(text, (x0, y0), (x1, y1)) {
            if let Anchor::Pdf { page: slot, .. } = &mut anchor {
                *slot = page as u32;
            }
            self.anchor_page = Some(page as u32);
            self.anchor = Some(anchor);
            self.publish();
        }
    }

    fn command(&mut self, name: &str, args: &Value) {
        match name {
            "pdf" => {
                let engine = if args.as_str() == Some("hayro") {
                    PdfEngine::Hayro
                } else {
                    PdfEngine::Pdfium
                };
                self.mode = if engine == PdfEngine::Hayro {
                    Mode::Hayro
                } else {
                    Mode::Pdfium
                };
                self.load_pdf(engine);
            }
            "zoom" => {
                let scale = if args.as_str() == Some("zoom-2x") {
                    PixelScale::Zoom2x
                } else {
                    PixelScale::Retina1x
                };
                self.set_scale(scale);
            }
            "book" => {
                self.book_path.clear();
                self.book_kind = if args.as_str() == Some("zhaohua") {
                    "zhaohua"
                } else {
                    "pride"
                };
                self.publish();
            }
            "bold" => {
                let on = args.as_bool().unwrap_or(false);
                let _ = crate::settings::store_bold(&crate::settings::path(), on);
                if let Some(web) = &self.web {
                    web.word_starts_changed();
                }
            }
            "epub" => {
                self.mode = match args.as_str() {
                    Some("blitz") => Mode::Blitz,
                    Some("text") => Mode::Text,
                    _ => Mode::WebKit,
                };
                self.publish();
            }
            "page" => {
                let delta = args.as_i64().unwrap_or(0);
                if self.mode == Mode::WebKit {
                    if let Some(web) = &self.web {
                        web.turn(delta as i32);
                    }
                } else if matches!(self.mode, Mode::Pdfium | Mode::Hayro) {
                    self.turn_pdf(delta);
                } else if delta < 0 {
                    self.epub_page = self.epub_page.saturating_sub(1);
                } else {
                    self.epub_page += 1;
                }
                self.publish();
            }
            _ => {}
        }
    }

    fn select_phrase(&mut self, phrase: &str) {
        let Some(pdf) = self.pdf.as_mut() else {
            return;
        };
        let text = pdf.text.entry(0).or_insert_with(|| pdf.doc.text(0));
        let Some(byte) = text.plain.find(phrase) else {
            return;
        };
        let start = text.plain[..byte].chars().count() as u32;
        let end = start + phrase.chars().count() as u32;
        let quads: Vec<spike_report::Quad> = text
            .glyphs
            .iter()
            .filter(|glyph| glyph.index >= start && glyph.index < end)
            .map(|glyph| glyph.quad)
            .collect();
        self.anchor_page = Some(0);
        self.anchor = Some(Anchor::Pdf {
            page: 0,
            start,
            end,
            text: phrase.into(),
            quads,
        });
        self.publish();
    }

    fn drag_two_lines(&mut self) {
        let Some(pdf) = self.pdf.as_mut() else {
            return;
        };
        let text = pdf.text.entry(0).or_insert_with(|| pdf.doc.text(0));
        let Some(at) = text.plain.find("Recent") else {
            return;
        };
        let start = text.plain[..at].chars().count() as u32;
        let Some(first) = text.glyphs.iter().find(|glyph| glyph.index == start) else {
            return;
        };
        let y0 = first.quad.y;
        let mut end = start + 20;
        let mut seen_next = false;
        for glyph in text.glyphs.iter().filter(|glyph| glyph.index > start) {
            if !seen_next && glyph.quad.y > y0 + 6.0 {
                seen_next = true;
            }
            if seen_next {
                end = glyph.index;
                if glyph.index > start + 80 && glyph.quad.y > y0 + 6.0 {
                    break;
                }
            }
        }
        let Some(last) = text.glyphs.iter().find(|glyph| glyph.index == end) else {
            return;
        };
        if let Some(mut anchor) = select_points(
            text,
            (first.quad.x + 1.0, first.quad.y + first.quad.h * 0.5),
            (last.quad.x + 1.0, last.quad.y + last.quad.h * 0.5),
        ) {
            if let Anchor::Pdf { page, .. } = &mut anchor {
                *page = 0;
            }
            self.anchor_page = Some(0);
            self.anchor = Some(anchor);
            let display = self.scale.display() as f64;
            self.offset = (first.quad.y as f64 * display - 36.0).max(0.0);
            if let Some(Anchor::Pdf { text, .. }) = &self.anchor {
                let _ = std::fs::write("/tmp/superreader-spike/drag.txt", text);
            }
        }
    }

    fn load_pdf(&mut self, engine: PdfEngine) {
        let path = spike_report::gpt3_path();
        let _ = self.load_pdf_at(engine, &path, 1);
        self.publish();
    }

    fn draw_fitted_pdf(&mut self, ctx: &CGContext, bounds: NSRect) {
        let index = self.current_page();
        let Some(pdf) = self.pdf.as_mut() else {
            return;
        };
        if pdf.sizes.is_empty() {
            return;
        }
        let index = index.min(pdf.sizes.len() - 1);
        let (pw, ph) = pdf.sizes[index];
        if pw < 1.0 || ph < 1.0 {
            return;
        }
        let (box_w, box_h, left, top) =
            crate::chrome::content_box(bounds.size.width, bounds.size.height);
        let (x, y, dw, dh) =
            spike_report::fitted_pdf_page(left, top, box_w, box_h, f64::from(pw), f64::from(ph));
        let key = (index, scale_key(self.scale));
        if !pdf.cache.contains_key(&key) {
            if let Ok(image) = pdf.doc.raster(index, self.scale) {
                pdf.cache.insert(key, image);
            }
        }
        let Some(image) = pdf.cache.get(&key).cloned() else {
            return;
        };
        let bytes = bitmap::flip_rows(image.width, &image.bytes);
        if let Some(cg) = bitmap::rgba_image(
            image.width,
            image.height,
            bytes,
            matches!(self.mode, Mode::Hayro),
        ) {
            let rect = NSRect::new(NSPoint::new(x, y), NSSize::new(dw, dh));
            unsafe { CGContext::draw_image(Some(ctx), rect, Some(&cg)) };
        }
    }

    fn load_pdf_at(&mut self, engine: PdfEngine, path: &str, page: u32) -> bool {
        let bytes = std::fs::read(path).unwrap_or_default();
        if bytes.is_empty() {
            self.pdf = None;
            return false;
        }
        let lib = spike_report::pdfium_lib();
        let opened = Document::open(engine, bytes.clone(), &lib)
            .map(|(doc, _, _)| (doc, engine))
            .or_else(|_| {
                let other = if engine == PdfEngine::Hayro {
                    PdfEngine::Pdfium
                } else {
                    PdfEngine::Hayro
                };
                Document::open(other, bytes, &lib).map(|(doc, _, _)| (doc, other))
            });
        let Ok((doc, used)) = opened else {
            self.pdf = None;
            return false;
        };
        self.page_total = Some(doc.page_count() as u32);
        let count = doc.page_count().min(80);
        let sizes: Vec<_> = (0..count).map(|i| doc.page_size_pt(i)).collect();
        if sizes.is_empty() || sizes.iter().all(|(w, h)| *w < 1.0 || *h < 1.0) {
            self.pdf = None;
            return false;
        }
        self.mode = match used {
            PdfEngine::Hayro => Mode::Hayro,
            PdfEngine::Pdfium => Mode::Pdfium,
        };
        self.offset = page_offset(&sizes, page, self.scale);
        self.anchor = None;
        self.pdf = Some(PdfView {
            doc,
            sizes,
            text: HashMap::new(),
            cache: HashMap::new(),
            frame_ms: Vec::new(),
        });
        true
    }

    fn turn_pdf(&mut self, delta: i64) {
        let Some(pdf) = self.pdf.as_ref() else {
            return;
        };
        let count = pdf.sizes.len();
        if count == 0 {
            return;
        }
        let current = self.current_page();
        let next = if delta < 0 {
            current.saturating_sub(1)
        } else {
            (current + 1).min(count - 1)
        };
        let sizes = pdf.sizes.clone();
        self.offset = page_offset(&sizes, next as u32 + 1, self.scale);
        self.note_page();
    }

    fn open_at(&mut self, view: &StageView, path: &str, page: u32) -> bool {
        let Some(book) = crate::books::parse_book(path) else {
            return false;
        };
        if !crate::books::can_open(&book.path) {
            return false;
        }
        let kind = match book.kind {
            crate::books::BookKind::Pdf => spike_library::OpenKind::Pdf,
            crate::books::BookKind::Epub if std::path::Path::new(&book.path).is_dir() => {
                spike_library::OpenKind::EpubDir
            }
            crate::books::BookKind::Epub => spike_library::OpenKind::EpubFile,
        };
        let page = page.max(1);
        self.page_total = None;
        let plan = spike_library::OpenSurfaces::closed().open(kind, page);
        self.dismiss_web();
        self.pdf = None;
        self.anchor = None;
        self.book_path = book.path.clone();
        let opened = if plan.pdf {
            let engine = match std::env::var("SPIKE_ENGINE").as_deref() {
                Ok("hayro") => PdfEngine::Hayro,
                _ => PdfEngine::Pdfium,
            };
            self.mode = Mode::Pdfium;
            self.load_pdf_at(engine, &book.path, plan.page)
        } else if plan.web {
            self.mode = Mode::WebKit;
            self.goto = if plan.page > 1 {
                Some(plan.page as usize)
            } else {
                None
            };
            self.epub_page = 0;
            self.web_pages = 1;
            self.shown_at = None;
            self.placed = None;
            self.open_web(view);
            self.web.is_some()
        } else {
            false
        };
        if !opened {
            self.mode = Mode::Empty;
            self.book_path.clear();
            self.publish();
            return false;
        }
        self.saved_page = page;
        let _ = crate::settings::record_open(
            &crate::settings::path(),
            &book.path,
            page,
            self.page_total,
            opened_at_ms(),
        );
        self.publish();
        true
    }

    fn open_requested(&mut self, view: &StageView, path: &str) {
        let page = crate::settings::load_place(&crate::settings::path())
            .filter(|place| place.path == path)
            .map(|place| place.page)
            .unwrap_or(1);
        self.open_at(view, path, page);
    }

    fn dismiss_web(&mut self) {
        if let Some(web) = self.web.take() {
            web.dismiss();
        }
    }

    fn note_page(&mut self) {
        let Some(page) = self.settled_page() else {
            return;
        };
        if self.book_path.is_empty() || page == self.saved_page {
            return;
        }
        self.saved_page = page;
        let path = self.book_path.clone();
        let _ = crate::settings::record_progress(
            &crate::settings::path(),
            &path,
            page,
            self.page_total,
        );
    }

    fn settled_page(&self) -> Option<u32> {
        match self.mode {
            Mode::Pdfium | Mode::Hayro => Some(self.current_page() as u32 + 1),
            Mode::WebKit => {
                let shown = self.epub_page as u32 + 1;
                if self.goto.is_some_and(|target| (shown as usize) < target) {
                    None
                } else {
                    Some(shown)
                }
            }
            Mode::Empty | Mode::Text | Mode::Blitz => None,
        }
    }

    fn epub_path(&self) -> String {
        if !self.book_path.is_empty() {
            return self.book_path.clone();
        }
        match self.book_kind {
            "zhaohua" => spike_report::zhaohua_path(),
            "styled" | "publisher" | "plain-en" | "plain-zh" | "fixed" => {
                format!("{}/{}.epub", spike_report::fixtures_dir(), self.book_kind)
            }
            _ => spike_report::pride_path(),
        }
    }

    fn poll_web(&mut self) {
        self.finish_hold();
        let Some(web) = &self.web else {
            return;
        };
        let (reload, changed) = {
            let inbox = web.inbox.borrow();
            let mut changed = false;
            if let Some((page, pages)) = inbox.pages {
                let page = page.saturating_sub(1);
                if self.epub_page != page || self.web_pages != pages.max(1) {
                    changed = true;
                    self.footer_at = Some(Instant::now());
                }
                self.epub_page = page;
                self.web_pages = pages.max(1);
                if self.shown_at.is_none() && pages > 1 {
                    self.shown_at = Some(Instant::now());
                    self.footer_at = Some(Instant::now());
                }
            }
            if inbox.anchor.is_some() && self.anchor != inbox.anchor {
                self.anchor = inbox.anchor.clone();
                changed = true;
            }
            (inbox.reload && !inbox.did_reload, changed)
        };
        if !self.book_path.is_empty() && self.mode == Mode::WebKit {
            let shown = self.epub_page as u32 + 1;
            let still_turning = self.goto.is_some_and(|target| (shown as usize) < target);
            if !still_turning && shown != self.saved_page {
                let path = self.book_path.clone();
                self.saved_page = shown;
                let _ = crate::settings::record_progress(
                    &crate::settings::path(),
                    &path,
                    shown,
                    self.page_total,
                );
            }
        }
        if changed || self.chrome_moving() {
            self.publish();
        }
        if crate::reading::title_alpha(elapsed_ms(self.shown_at)) == 0.0 && self.shown_at.is_some()
        {
            self.title_settled = true;
        }
        if reload {
            web.inbox.borrow_mut().did_reload = true;
            web.inbox.borrow_mut().reload = false;
            web.reload();
        }
        if std::env::var("SPIKE_TURN").as_deref() == Ok("1") && self.frame_settled() {
            let ready = web
                .inbox
                .borrow()
                .pages
                .map(|(_, pages)| pages > 1)
                .unwrap_or(false);
            let already = web.inbox.borrow().scripted_turn;
            if ready && !already {
                web.inbox.borrow_mut().scripted_turn = true;
                self.turn_pending = true;
                web.turn(1);
            }
        }
        if let Some(target) = self.goto {
            let (page, count) = web.inbox.borrow().pages.unwrap_or((1, 1));
            if count > 1 && page < target && !self.turn_pending {
                self.turn_pending = true;
                self.play_at = Some(Instant::now());
                web.turn(1);
            }
            if self.turn_pending
                && self
                    .play_at
                    .map(|at| at.elapsed().as_millis() > 700)
                    .unwrap_or(false)
            {
                self.turn_pending = false;
            }
        }
        if std::env::var("SPIKE_PLAY").as_deref() == Ok("1") {
            let ready = web
                .inbox
                .borrow()
                .pages
                .map(|(_, pages)| pages > 1)
                .unwrap_or(false);
            let shown = self
                .shown_at
                .map(|at| at.elapsed().as_millis())
                .unwrap_or(0);
            let due = self
                .play_at
                .map(|at| at.elapsed().as_millis() >= 900)
                .unwrap_or(shown >= 1200);
            if ready && self.frame_settled() && self.play_count < 3 && due && !self.turn_pending {
                self.play_count += 1;
                self.play_at = Some(Instant::now());
                self.turn_pending = true;
                web.turn(1);
            }
        }
        let frame_count = web.inbox.borrow().frame_ms.len();
        if frame_count > self.seen_frames {
            self.seen_frames = frame_count;
            self.turn_pending = false;
        }
        let inbox = web.inbox.borrow();
        let href = web.href();
        crate::web::write_inbox(&inbox, &href);
    }

    fn finish_hold(&mut self) {
        if !self.await_hold {
            return;
        }
        let offset = self.web.as_ref().and_then(|web| web.inbox.borrow().hold);
        let timed_out = self
            .hold_since
            .map(|at| at.elapsed().as_millis() > 700)
            .unwrap_or(false);
        if offset.is_none() && !timed_out {
            return;
        }
        if let Some(web) = &self.web {
            web.inbox.borrow_mut().hold = None;
        }
        self.await_hold = false;
        let offset = offset.unwrap_or(0);
        self.hold_ready = true;
        self.place_page();
        if let Some(web) = &self.web {
            web.reveal(offset);
            if let (Some(start), Some(end)) = (
                self.anchor.as_ref().and_then(anchor_start),
                self.anchor.as_ref().and_then(anchor_end),
            ) {
                web.repaint(start, end);
            }
        }
    }

    fn open_web(&mut self, parent: &StageView) {
        let path = self.epub_path();
        let parent: &AnyObject = unsafe { &*(parent as *const StageView).cast() };
        match crate::web::WebChapter::open(parent, &path) {
            Ok(chapter) => {
                self.declared = chapter.declared;
                self.placed = None;
                self.web = Some(chapter);
            }
            Err(err) => eprintln!("webkit {err}"),
        }
    }

    fn chrome_moving(&self) -> bool {
        // The window's content height is still changing after the title fades.
        // A native page stops drawing then and keeps the opening frame; the
        // line breaker happens to invalidate again and picks up the final
        // frame, so the two screenshots were different pages.
        self.mode == Mode::WebKit
            && self
                .shown_at
                .map(|at| at.elapsed().as_millis() < 2200)
                .unwrap_or(true)
    }

    fn frame_settled(&self) -> bool {
        self.shown_at
            .map(|at| at.elapsed().as_millis() >= 2200)
            .unwrap_or(false)
            && !self.await_hold
    }

    fn apply_trim(&mut self, id: &str) {
        let id = crate::reading::trim(id).id.to_string();
        if self.trim_id == id {
            return;
        }
        let _ = crate::settings::store(&crate::settings::path(), &id);
        if let Some(web) = &self.web {
            web.trim_changed(&id);
        }
        self.trim_id = id;
        self.placed = None;
        self.place_page();
        self.publish();
    }

    fn reload_trim_from_file(&mut self) -> bool {
        let id = crate::settings::load(&crate::settings::path());
        if id == self.trim_id {
            return false;
        }
        self.apply_trim(&id);
        true
    }

    fn page_box(&self) -> crate::chrome::PageBox {
        let item = crate::reading::trim(&self.trim_id);
        let (rw, rh, _) = crate::reading::page_ratio(self.declared, item);
        let placed = self
            .placed
            .clone()
            .map(|item| (item.0, item.1, item.2, item.3))
            .unwrap_or((0, 0, 0, 0));
        crate::chrome::PageBox {
            x: f64::from(placed.0),
            y: f64::from(placed.1),
            w: f64::from(placed.2),
            h: f64::from(placed.3),
            stage_w: self.stage_w,
            stage_h: self.stage_h,
            ratio_w: rw,
            ratio_h: rh,
        }
    }

    fn place_page(&mut self) {
        if self.await_hold || self.stage_w < 1.0 || self.stage_h < 1.0 {
            return;
        }
        let item = crate::reading::trim(&self.trim_id);
        let (rw, rh, source) = crate::reading::page_ratio(self.declared, item);
        let (box_w, box_h, left, top) = crate::chrome::content_box(self.stage_w, self.stage_h);
        let (ix, iy, w, h) = crate::reading::fit_page(box_w, box_h, rw, rh);
        let bar = self.chrome.bar_w.get();
        let x = crate::chrome::content_page_x(self.stage_w, w, bar).unwrap_or(left + ix);
        let y = top + iy;
        let design = crate::chrome::reading_page(
            rw,
            rh,
            self.chrome.full.get(),
            &self.chrome.design_visible,
        );
        let zoom = if self.chrome.full.get() && design.1 > 1.0 {
            (h / design.1).max(1.0)
        } else {
            1.0
        };
        let key = (
            x.round() as i32,
            y.round() as i32,
            w.round() as i32,
            h.round() as i32,
            self.trim_id.clone(),
        );
        if self.web.is_none() {
            return;
        }
        if self.placed.as_ref() == Some(&key) && (self.placed_zoom - zoom).abs() < 0.01 {
            return;
        }
        let reflowable = self.declared.is_none();
        if self.settled && reflowable && !self.hold_ready {
            self.await_hold = true;
            self.hold_since = Some(Instant::now());
            if let Some(web) = &self.web {
                web.hold();
            }
            return;
        }
        self.hold_ready = false;
        self.settled = true;
        self.placed = Some(key);
        let Some(web) = &self.web else {
            return;
        };
        self.placed_zoom = zoom;
        web.place_at(x, y, w, h, zoom);
        web.set_page_target(layout_target(w, zoom));
        {
            let mut inbox = web.inbox.borrow_mut();
            inbox.page_w = w;
            inbox.page_h = h;
            inbox.page_x = x;
            inbox.bar_w = self.chrome.bar_w.get();
            inbox.stage_w = self.stage_w;
            inbox.trim = self.trim_id.clone();
            inbox.page_source = source.to_string();
        }
        let href = web.href();
        crate::web::write_inbox(&web.inbox.borrow(), &href);
    }

    fn publish(&self) {
        let mut title_opacity = 1.0_f32;
        let mut footer_opacity = 1.0_f32;
        let mut bars_row = false;
        let (title, variant, left, right, show_text, html) = match self.mode {
            Mode::Empty => {
                bars_row = true;
                (
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    false,
                    String::new(),
                )
            }
            Mode::Pdfium | Mode::Hayro => {
                let pages = self.pdf.as_ref().map(|p| p.sizes.len()).unwrap_or(0);
                let current = self.current_page().saturating_add(1);
                let left_pages = pages.saturating_sub(current);
                let title = if self.book_path.is_empty() || self.book_path.ends_with("gpt3.pdf") {
                    "Language Models are Few-Shot Learners".to_string()
                } else {
                    std::path::Path::new(&self.book_path)
                        .file_stem()
                        .and_then(|name| name.to_str())
                        .unwrap_or("PDF")
                        .to_string()
                };
                // The installed app uses the EPUB toolbar. The dev spike keeps
                // the engine label and the page row.
                if self.product {
                    bars_row = true;
                }
                let variant = if self.product {
                    String::new()
                } else {
                    format!(
                        "{}  {}",
                        match self.mode {
                            Mode::Hayro => "hayro",
                            _ => "pdfium",
                        },
                        self.scale.name()
                    )
                };
                let left = if self.product {
                    String::new()
                } else {
                    format!("{current}/{pages}")
                };
                let right = if self.product {
                    String::new()
                } else {
                    format!("{left_pages} pages left")
                };
                (title, variant, left, right, false, String::new())
            }
            Mode::WebKit => {
                let pages = self.web_pages.max(1);
                let page = (self.epub_page % pages) + 1;
                let path = self.epub_path();
                let title = self
                    .web
                    .as_ref()
                    .map(|web| web.book_title.clone())
                    .filter(|title| !title.is_empty())
                    .or_else(|| spike_epub::open_book(&path).ok().map(|book| book.title))
                    .unwrap_or_default();
                title_opacity = crate::reading::title_alpha(elapsed_ms(self.shown_at));
                footer_opacity = 1.0;
                bars_row = true;
                let (left, right) = crate::reading::footer_zh(page as u32, pages as u32);
                (title, String::new(), left, right, false, String::new())
            }
            Mode::Text | Mode::Blitz => {
                let label = match self.mode {
                    Mode::Text => "text view",
                    Mode::WebKit => "webkit",
                    _ => "blitz",
                };
                let path = self.epub_path();
                let book = spike_epub::open_book(&path).ok();
                let title = book.as_ref().map(|b| b.title.clone()).unwrap_or_default();
                let chapter = book.as_ref().and_then(|b| b.chapters.first());
                let html = chapter
                    .map(|c| format!("<p>{}</p>", c.plain))
                    .unwrap_or_default();
                let pages = chapter
                    .map(|c| (c.plain.chars().count() / 1200).max(1))
                    .unwrap_or(1);
                let page = (self.epub_page % pages) + 1;
                (
                    title,
                    label.to_string(),
                    format!("{page}/{pages}"),
                    pages_left(pages.saturating_sub(page)),
                    self.mode == Mode::Text,
                    html,
                )
            }
        };
        let anchor = self
            .anchor
            .as_ref()
            .map(|a| serde_json::to_value(a).unwrap_or(Value::Null))
            .unwrap_or(Value::Null);
        self.chrome.set_title(&title);
        if self.mode == Mode::WebKit {
            if let Some(web) = &self.web {
                web.set_footer(&left, &right);
            }
        }
        self.context.emit(
            "status",
            json!({
                "title": title,
                "variant": variant,
                "left": left,
                "right": right,
                "show_text": show_text,
                "html": html,
                "anchor": anchor,
                "title_opacity": title_opacity,
                "footer_opacity": footer_opacity,
                "bars_row": bars_row,
                "trim": self.trim_id,
                "empty": self.mode == Mode::Empty,
                "stage_w": self.stage_w,
                "stage_h": self.stage_h,
                "bar_w": self.chrome.bar_w.get(),
            }),
        );
    }

    fn current_page(&self) -> usize {
        let Some(pdf) = self.pdf.as_ref() else {
            return 0;
        };
        let display = self.scale.display() as f64;
        let mut y = 0.0;
        for (index, (_, ph)) in pdf.sizes.iter().enumerate() {
            let dh = *ph as f64 * display;
            if self.offset < y + dh {
                return index;
            }
            y += dh + 16.0;
        }
        0
    }
}

fn pdf_origin_of(product: bool, bar_w: f64) -> (f64, f64) {
    if product {
        (bar_w.max(24.0), crate::chrome::TOP)
    } else {
        (24.0, 0.0)
    }
}

impl State {
    fn pdf_origin(&self) -> (f64, f64) {
        pdf_origin_of(self.product, self.chrome.bar_w.get())
    }
}

fn locate(
    pdf: &PdfView,
    offset: f64,
    x: f64,
    y: f64,
    display: f64,
    origin_x: f64,
    origin_y: f64,
) -> Option<(usize, f32, f32)> {
    let mut top = origin_y - offset;
    for (index, (pw, ph)) in pdf.sizes.iter().enumerate() {
        let dh = *ph as f64 * display;
        let bottom = top + dh;
        if y >= top && y <= bottom {
            let px = ((x - origin_x) / display) as f32;
            let py = ((y - top) / display) as f32;
            if px >= 0.0 && px <= *pw {
                return Some((index, px, py));
            }
        }
        top = bottom + 16.0;
    }
    None
}

fn elapsed_ms(at: Option<Instant>) -> u32 {
    at.map(|stamp| stamp.elapsed().as_millis().min(u128::from(u32::MAX)) as u32)
        .unwrap_or(0)
}

fn pages_left(left: usize) -> String {
    if left == 1 {
        "1 page left in this chapter".into()
    } else {
        format!("{left} pages left in this chapter")
    }
}

fn scale_key(scale: PixelScale) -> u8 {
    match scale {
        PixelScale::Retina1x => 1,
        PixelScale::Zoom2x => 2,
    }
}

struct Stage {
    view: Retained<StageView>,
    state: Rc<RefCell<State>>,
}

impl Stage {
    fn new(context: ComponentContext) -> Self {
        let mtm = MainThreadMarker::new().expect("main");
        let state = Rc::new(RefCell::new(State {
            context,
            mode: Mode::Pdfium,
            scale: PixelScale::Retina1x,
            offset: 0.0,
            pinch: 0.0,
            drag: None,
            pdf: None,
            book_kind: "pride",
            book_path: String::new(),
            product: false,
            saved_page: 0,
            page_total: None,
            epub_page: 0,
            anchor: None,
            anchor_page: None,
            web: None,
            web_pages: 1,
            trim_id: crate::settings::load(&crate::settings::path()),
            declared: None,
            stage_w: 0.0,
            stage_h: 0.0,
            placed: None,
            placed_zoom: 1.0,
            await_hold: false,
            hold_ready: false,
            settled: false,
            hold_since: None,
            shown_at: None,
            footer_at: None,
            play_count: 0,
            play_at: None,
            turn_pending: false,
            seen_frames: 0,
            goto: std::env::var("SPIKE_AT")
                .ok()
                .and_then(|raw| raw.parse().ok()),
            title_settled: false,
            chrome: Rc::new(crate::chrome::Runtime::default()),
        }));
        let view = mtm.alloc::<StageView>().set_ivars(Ivars {
            state: state.clone(),
        });
        let view: Retained<StageView> = unsafe {
            msg_send![super(view), initWithFrame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0))]
        };
        unsafe { msg_send![&*view, setWantsLayer: true] }
        let product = crate::books::in_app_bundle();
        {
            let mut state = state.borrow_mut();
            state.product = product;
            state.chrome.product.set(product);
            if product {
                state.chrome.lights.set(1.0);
                state.mode = Mode::Empty;
            }
        }
        let shelf = crate::launches_full();
        if shelf {
            crate::chrome::engage_shell();
            state.borrow_mut().mode = Mode::Empty;
            state.borrow_mut().publish();
        } else if product {
            let mut state = state.borrow_mut();
            if let Ok(path) = std::env::var("SUPERREADER_OPEN_PATH") {
                let path = path.trim();
                if !path.is_empty() {
                    state.open_requested(&view, path);
                } else {
                    state.publish();
                }
            } else if let Some(place) = crate::settings::load_place(&crate::settings::path()) {
                state.open_at(&view, &place.path, place.page);
            } else {
                state.publish();
            }
        }
        if !product && !shelf {
            let engine = match std::env::var("SPIKE_ENGINE").as_deref() {
                Ok("hayro") => PdfEngine::Hayro,
                _ => PdfEngine::Pdfium,
            };
            if std::env::var("SPIKE_SCALE").as_deref() == Ok("zoom-2x") {
                state.borrow_mut().scale = PixelScale::Zoom2x;
            }
            state.borrow_mut().mode = if engine == PdfEngine::Hayro {
                Mode::Hayro
            } else {
                Mode::Pdfium
            };
            state.borrow_mut().load_pdf(engine);
            if std::env::var("SPIKE_DRAG").as_deref() == Ok("1") {
                state.borrow_mut().drag_two_lines();
            }
            if std::env::var("SPIKE_MODE").as_deref() == Ok("webkit") {
                state.borrow_mut().mode = Mode::WebKit;
                state.borrow_mut().book_kind = book_kind_from_env();
                state.borrow_mut().open_web(&view);
                state.borrow_mut().publish();
            }
            if std::env::var("SPIKE_MODE").as_deref() == Ok("text") {
                state.borrow_mut().mode = Mode::Text;
                state.borrow_mut().book_kind = book_kind_from_env();
                state.borrow_mut().publish();
            }
            if std::env::var("SPIKE_SELECT").as_deref() == Ok("1") {
                state.borrow_mut().select_phrase("Abstract");
            }
        }
        let _: () = unsafe {
            msg_send![
                &*view,
                performSelector: sel!(pumpLights),
                withObject: std::ptr::null::<AnyObject>(),
                afterDelay: 0.05_f64
            ]
        };
        let _: () = unsafe {
            msg_send![
                &*view,
                performSelector: sel!(pollSettings),
                withObject: std::ptr::null::<AnyObject>(),
                afterDelay: 0.4_f64
            ]
        };
        Stage { view, state }
    }
}

impl NativeComponent for Stage {
    fn view(&self) -> Retained<NSView> {
        Retained::into_super(self.view.clone())
    }

    fn set_prop(&mut self, _name: &str, _value: Option<&nsh_core::JsonValue>) {}

    fn command(&mut self, name: &str, args: &nsh_core::JsonValue) {
        if name == "open" {
            let path = args
                .as_str()
                .map(str::to_string)
                .or_else(|| {
                    args.get("path")
                        .and_then(|value| value.as_str())
                        .map(str::to_string)
                })
                .unwrap_or_default();
            self.state.borrow_mut().open_requested(&self.view, &path);
        } else if name == "trim" {
            self.state
                .borrow_mut()
                .apply_trim(args.as_str().unwrap_or(""));
        } else {
            self.state.borrow_mut().command(name, args);
        }
        unsafe { msg_send![&*self.view, setNeedsDisplay: true] }
    }
}

fn page_offset(sizes: &[(f32, f32)], page: u32, scale: PixelScale) -> f64 {
    let display = scale.display() as f64;
    let index = page.saturating_sub(1) as usize;
    let mut y = 0.0;
    for (i, (_, height)) in sizes.iter().enumerate() {
        if i == index {
            return y;
        }
        y += *height as f64 * display + 16.0;
    }
    0.0
}

fn book_kind_from_env() -> &'static str {
    match std::env::var("SPIKE_BOOK").as_deref() {
        Ok("zhaohua") => "zhaohua",
        Ok("styled") => "styled",
        Ok("publisher") => "publisher",
        Ok("plain-en") => "plain-en",
        Ok("plain-zh") => "plain-zh",
        Ok("fixed") => "fixed",
        _ => "pride",
    }
}

fn anchor_start(anchor: &Anchor) -> Option<u32> {
    match anchor {
        Anchor::Epub { start, .. } => Some(*start),
        Anchor::Pdf { .. } => None,
    }
}

fn anchor_end(anchor: &Anchor) -> Option<u32> {
    match anchor {
        Anchor::Epub { end, .. } => Some(*end),
        Anchor::Pdf { .. } => None,
    }
}

fn layout_target(placed_width: f64, zoom: f64) -> f64 {
    if zoom.is_finite() && zoom > 0.01 {
        placed_width / zoom
    } else {
        placed_width
    }
}

#[cfg(test)]
mod tests {
    use super::layout_target;

    #[test]
    fn a_smaller_placement_is_the_layout_target() {
        assert_eq!(layout_target(400.0, 1.0), 400.0);
        assert_eq!(layout_target(800.0, 2.0), 400.0);
        assert!(layout_target(400.0, 1.0) < 534.0);
    }
}

fn current_context() -> Option<Retained<CGContext>> {
    unsafe {
        let current: Option<Retained<objc2_app_kit::NSGraphicsContext>> =
            msg_send![objc2::class!(NSGraphicsContext), currentContext];
        current.and_then(|g| -> Option<Retained<CGContext>> { msg_send![&*g, CGContext] })
    }
}

pub fn register(components: &Components) {
    components.register(
        "reader.stage",
        Rc::new(|context| Box::new(Stage::new(context))),
    );
}
