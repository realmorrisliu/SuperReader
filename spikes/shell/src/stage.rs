use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Instant;

use nsh_platform_macos::objc2::rc::Retained;
use nsh_platform_macos::objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use nsh_platform_macos::objc2_app_kit::{NSEvent, NSResponder, NSView};
use nsh_platform_macos::objc2_foundation::{NSObjectProtocol, NSPoint, NSRect, NSSize};
use nsh_platform_macos::{ComponentContext, Components, NativeComponent};
use objc2_core_graphics::CGContext;
use serde_json::{Value, json};
use spike_pdf::{Document, PageText, Rgba, paint_highlight, select_points};
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

        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, _dirty: NSRect) {
            let state = self.ivars().state.clone();
            state.borrow_mut().draw(self);
        }

        #[unsafe(method(layout))]
        fn layout_view(&self) {
            let _: () = unsafe { msg_send![super(self), layout] };
            let bounds: NSRect = unsafe { msg_send![self, bounds] };
            let state = self.ivars().state.borrow();
            if let Some(web) = &state.web {
                web.place(bounds.size.width, bounds.size.height);
            }
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
    epub_page: usize,
    anchor: Option<Anchor>,
    anchor_page: Option<u32>,
    web: Option<crate::web::WebChapter>,
    web_pages: usize,
    choice: Rc<Cell<crate::reading::Choice>>,
    shown_at: Option<Instant>,
    footer_at: Option<Instant>,
    play_count: u8,
    play_at: Option<Instant>,
    turn_pending: bool,
    seen_frames: usize,
    goto: Option<usize>,
    title_settled: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Pdfium,
    Hayro,
    WebKit,
    Blitz,
    Text,
}

impl State {
    fn draw(&mut self, view: &StageView) {
        self.poll_web();
        let bounds: NSRect = unsafe { msg_send![view, bounds] };
        if self.mode == Mode::WebKit {
            if let Some(web) = &self.web {
                web.place(bounds.size.width, bounds.size.height);
            }
            let playing = std::env::var("SPIKE_PLAY").as_deref() == Ok("1")
                && self.play_count < 3
                && !self.turn_pending;
            let need_page = self.goto.is_some_and(|target| {
                self.web.as_ref().and_then(|web| web.inbox.borrow().pages).map(|(page, _)| page < target).unwrap_or(true)
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
        unsafe {
            CGContext::set_rgb_fill_color(Some(&ctx), 0.96, 0.95, 0.93, 1.0);
            CGContext::fill_rect(Some(&ctx), bounds);
        }
        if !matches!(self.mode, Mode::Pdfium | Mode::Hayro) {
            return;
        }
        let Some(pdf) = self.pdf.as_mut() else {
            return;
        };
        let display = self.scale.display() as f64;
        let mut y = -self.offset;
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
                        let rect = NSRect::new(NSPoint::new(24.0, y), NSSize::new(dw, dh));
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
        let Some(pdf) = self.pdf.as_mut() else {
            return;
        };
        let display = self.scale.display() as f64;
        let (page, x0, y0) = match locate(pdf, self.offset, start.0, start.1, display) {
            Some(v) => v,
            None => return,
        };
        let Some((_, x1, y1)) = locate(pdf, self.offset, end.0, end.1, display) else {
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
                self.mode = if engine == PdfEngine::Hayro { Mode::Hayro } else { Mode::Pdfium };
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
                self.book_kind = if args.as_str() == Some("zhaohua") { "zhaohua" } else { "pride" };
                self.publish();
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
        let bytes = std::fs::read(spike_report::gpt3_path()).unwrap_or_default();
        if bytes.is_empty() {
            self.pdf = None;
            self.publish();
            return;
        }
        if let Ok((doc, _, _)) = Document::open(engine, bytes, &spike_report::pdfium_lib()) {
            let count = doc.page_count().min(80);
            let sizes: Vec<_> = (0..count).map(|i| doc.page_size_pt(i)).collect();
            self.pdf = Some(PdfView {
                doc,
                sizes,
                text: HashMap::new(),
                cache: HashMap::new(),
                frame_ms: Vec::new(),
            });
            self.offset = 0.0;
            self.anchor = None;
        }
        self.publish();
    }

    fn poll_web(&mut self) {
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
        if changed || self.chrome_moving() {
            self.publish();
        }
        if let Some(view) = crate::reading::view(self.choice.get()) {
            let (title_alpha, _) = crate::reading::alphas(
                view,
                elapsed_ms(self.shown_at),
                elapsed_ms(self.footer_at),
            );
            if title_alpha == 0.0 {
                self.title_settled = true;
            }
        }
        if reload {
            web.inbox.borrow_mut().did_reload = true;
            web.inbox.borrow_mut().reload = false;
            web.reload();
        }
        let viewing = crate::reading::view(self.choice.get()).is_some();
        if std::env::var("SPIKE_TURN").as_deref() == Ok("1") {
            if viewing {
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
            } else {
                let turns = web.inbox.borrow().turn_ms.len();
                if turns == 0 && web.inbox.borrow().pages.is_some() {
                    web.turn(1);
                }
            }
        }
        if let Some(target) = self.goto {
            let (page, count) = web.inbox.borrow().pages.unwrap_or((1, 1));
            if viewing && count > 1 && page < target && !self.turn_pending {
                self.turn_pending = true;
                self.play_at = Some(Instant::now());
                web.turn(1);
            }
            if self.turn_pending
                && self.play_at.map(|at| at.elapsed().as_millis() > 700).unwrap_or(false)
            {
                self.turn_pending = false;
            }
        }
        if viewing && std::env::var("SPIKE_PLAY").as_deref() == Ok("1") {
            let ready = web
                .inbox
                .borrow()
                .pages
                .map(|(_, pages)| pages > 1)
                .unwrap_or(false);
            let shown = self.shown_at.map(|at| at.elapsed().as_millis()).unwrap_or(0);
            let due = self
                .play_at
                .map(|at| at.elapsed().as_millis() >= 900)
                .unwrap_or(shown >= 1200);
            if ready && self.play_count < 3 && due && !self.turn_pending {
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
        crate::web::write_inbox(&inbox, web.href());
    }

    fn open_web(&mut self, parent: &StageView) {
        let path = if self.book_kind == "zhaohua" {
            spike_report::zhaohua_path()
        } else {
            spike_report::pride_path()
        };
        let parent: &objc2::runtime::AnyObject = unsafe { &*(parent as *const StageView).cast() };
        match crate::web::WebChapter::open(parent, &path, self.choice.clone()) {
            Ok(chapter) => self.web = Some(chapter),
            Err(err) => eprintln!("webkit {err}"),
        }
    }

    fn chrome_moving(&self) -> bool {
        let Some(view) = crate::reading::view(self.choice.get()) else {
            return false;
        };
        if !self.title_settled {
            return true;
        }
        crate::reading::chrome_moving(view, elapsed_ms(self.shown_at), elapsed_ms(self.footer_at))
    }

    fn publish(&self) {
        let mut title_opacity = 1.0_f32;
        let mut footer_opacity = 1.0_f32;
        let mut bars_row = false;
        let (title, variant, left, right, show_text, html) = match self.mode {
            Mode::Pdfium | Mode::Hayro => {
                let pages = self.pdf.as_ref().map(|p| p.sizes.len()).unwrap_or(0);
                let current = self.current_page().saturating_add(1);
                let left_pages = pages.saturating_sub(current);
                (
                    "Language Models are Few-Shot Learners".to_string(),
                    format!(
                        "{}  {}",
                        match self.mode {
                            Mode::Hayro => "hayro",
                            _ => "pdfium",
                        },
                        self.scale.name()
                    ),
                    format!("{current}/{pages}"),
                    format!("{left_pages} pages left"),
                    false,
                    String::new(),
                )
            }
            Mode::WebKit => {
                let pages = self.web_pages.max(1);
                let page = (self.epub_page % pages) + 1;
                let path = if self.book_kind == "zhaohua" {
                    spike_report::zhaohua_path()
                } else {
                    spike_report::pride_path()
                };
                let title = spike_epub::open_book(&path)
                    .map(|book| book.title)
                    .unwrap_or_default();
                if let Some(view) = crate::reading::view(self.choice.get()) {
                    let (title_alpha, footer_alpha) = crate::reading::alphas(
                        view,
                        elapsed_ms(self.shown_at),
                        elapsed_ms(self.footer_at),
                    );
                    title_opacity = title_alpha;
                    footer_opacity = footer_alpha;
                    bars_row = true;
                    let (left, right) = crate::reading::footer_zh(page as u32, pages as u32);
                    (
                        title,
                        view.letter.to_string(),
                        left,
                        right,
                        false,
                        String::new(),
                    )
                } else {
                    (
                        title,
                        "webkit".to_string(),
                        format!("{page}/{pages}"),
                        pages_left(pages.saturating_sub(page)),
                        false,
                        String::new(),
                    )
                }
            }
            Mode::Text | Mode::Blitz => {
                let label = match self.mode {
                    Mode::Text => "text view",
                    Mode::WebKit => "webkit",
                    _ => "blitz",
                };
                let path = if self.book_kind == "zhaohua" {
                    spike_report::zhaohua_path()
                } else {
                    spike_report::pride_path()
                };
                let book = spike_epub::open_book(&path).ok();
                let title = book.as_ref().map(|b| b.title.clone()).unwrap_or_default();
                let chapter = book.as_ref().and_then(|b| b.chapters.first());
                let html = chapter
                    .map(|c| format!("<p>{}</p>", c.plain))
                    .unwrap_or_default();
                let pages = chapter.map(|c| (c.plain.chars().count() / 1200).max(1)).unwrap_or(1);
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

fn locate(pdf: &PdfView, offset: f64, x: f64, y: f64, display: f64) -> Option<(usize, f32, f32)> {
    let mut top = -offset;
    for (index, (pw, ph)) in pdf.sizes.iter().enumerate() {
        let dh = *ph as f64 * display;
        let bottom = top + dh;
        if y >= top && y <= bottom {
            let px = ((x - 24.0) / display) as f32;
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
            epub_page: 0,
            anchor: None,
            anchor_page: None,
            web: None,
            web_pages: 1,
            choice: Rc::new(Cell::new(crate::reading::Choice::Legacy)),
            shown_at: None,
            footer_at: None,
            play_count: 0,
            play_at: None,
            turn_pending: false,
            seen_frames: 0,
            goto: std::env::var("SPIKE_AT").ok().and_then(|raw| raw.parse().ok()),
            title_settled: false,
        }));
        let view = mtm.alloc::<StageView>().set_ivars(Ivars {
            state: state.clone(),
        });
        let view: Retained<StageView> =
            unsafe { msg_send![super(view), initWithFrame: NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0))] };
        unsafe { msg_send![&*view, setWantsLayer: true] }
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
        let choice = match crate::reading::choose(std::env::var("SPIKE_READING").ok().as_deref()) {
            Ok(choice) => choice,
            Err(err) => {
                eprintln!("{err}");
                std::process::exit(1);
            }
        };
        state.borrow_mut().choice.set(choice);
        let read = choice != crate::reading::Choice::Legacy
            || std::env::var("SPIKE_MODE").as_deref() == Ok("webkit");
        if read {
            state.borrow_mut().mode = Mode::WebKit;
            if std::env::var("SPIKE_BOOK").as_deref() == Ok("zhaohua") {
                state.borrow_mut().book_kind = "zhaohua";
            }
            state.borrow_mut().open_web(&view);
            state.borrow_mut().publish();
        }
        if std::env::var("SPIKE_MODE").as_deref() == Ok("text") {
            state.borrow_mut().mode = Mode::Text;
            if std::env::var("SPIKE_BOOK").as_deref() == Ok("zhaohua") {
                state.borrow_mut().book_kind = "zhaohua";
            }
            state.borrow_mut().publish();
        }
        if std::env::var("SPIKE_SELECT").as_deref() == Ok("1") {
            state.borrow_mut().select_phrase("Abstract");
        }
        Stage { view, state }
    }
}

impl NativeComponent for Stage {
    fn view(&self) -> Retained<NSView> {
        Retained::into_super(self.view.clone())
    }

    fn set_prop(&mut self, _name: &str, _value: Option<&nsh_core::JsonValue>) {}

    fn command(&mut self, name: &str, args: &nsh_core::JsonValue) {
        if name == "reading" {
            let next = crate::reading::cycle(self.state.borrow().choice.get());
            self.state.borrow_mut().choice.set(next);
            self.state.borrow_mut().shown_at = Some(Instant::now());
            self.state.borrow_mut().footer_at = Some(Instant::now());
            self.state.borrow_mut().play_count = 0;
            self.state.borrow_mut().play_at = None;
            self.state.borrow_mut().turn_pending = false;
            self.state.borrow_mut().seen_frames = 0;
            self.state.borrow_mut().title_settled = false;
            if next != crate::reading::Choice::Legacy {
                self.state.borrow_mut().mode = Mode::WebKit;
            }
            if self.state.borrow().mode == Mode::WebKit {
                if self.state.borrow().web.is_none() {
                    self.state.borrow_mut().open_web(&self.view);
                } else if let Some(web) = &self.state.borrow().web {
                    web.reload();
                }
            }
            self.state.borrow_mut().publish();
        } else {
            self.state.borrow_mut().command(name, args);
        }
        unsafe { msg_send![&*self.view, setNeedsDisplay: true] }
    }
}

pub fn register(components: &Components) {
    components.register(
        "reader.stage",
        Rc::new(|context| Box::new(Stage::new(context))),
    );
}
