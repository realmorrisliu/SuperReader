use std::cell::{Cell, RefCell};

use nsh_platform_macos::objc2::rc::Retained;
use nsh_platform_macos::objc2::{msg_send, MainThreadMarker};
use objc2_app_kit::{
    NSColor, NSFont, NSScreen, NSTextField, NSTitlebarSeparatorStyle, NSWindow, NSWindowButton,
    NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

pub const SIDE: f64 = 52.0;
pub const FOOTER: f64 = 36.0;
pub const BAR: f64 = 32.0;
pub const TOP_OVERLAY: f64 = 44.0;
pub const BOTTOM_OVERLAY: f64 = 24.0;
pub const TOP_BAR: f64 = 22.0;
pub const BOTTOM_BAR: f64 = 14.0;
pub const FIT: f64 = 0.88;
pub const LIGHTS_BOTTOM: f64 = 28.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Off,
    Lights,
    Hover,
    Bar,
}

pub struct Mats {
    pub side: f64,
    pub top: f64,
    pub bottom: f64,
    pub bar: f64,
    pub footer: f64,
    pub lock: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub window_w: f64,
    pub window_h: f64,
    pub page_w: f64,
    pub page_h: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct PageBox {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub stage_w: f64,
    pub stage_h: f64,
    pub ratio_w: f64,
    pub ratio_h: f64,
}

pub struct Runtime {
    settled: Cell<bool>,
    stable: Cell<u8>,
    attempts: Cell<u8>,
    shown: Cell<bool>,
    full: Cell<bool>,
    fullscreen_sent: Cell<bool>,
    lights: Cell<f64>,
    ratio_key: Cell<(i32, i32)>,
    page: Cell<PageBox>,
    title: RefCell<String>,
    styled: Cell<bool>,
    entering: Cell<bool>,
    field: RefCell<Option<Retained<NSTextField>>>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self {
            settled: Cell::new(false),
            stable: Cell::new(0),
            attempts: Cell::new(0),
            shown: Cell::new(false),
            full: Cell::new(false),
            fullscreen_sent: Cell::new(false),
            lights: Cell::new(0.0),
            ratio_key: Cell::new((0, 0)),
            page: Cell::new(PageBox {
                x: 0.0,
                y: 0.0,
                w: 0.0,
                h: 0.0,
                stage_w: 0.0,
                stage_h: 0.0,
                ratio_w: 0.0,
                ratio_h: 0.0,
            }),
            title: RefCell::new(String::new()),
            styled: Cell::new(false),
            entering: Cell::new(false),
            field: RefCell::new(None),
        }
    }
}

impl Runtime {
    pub fn set_title(&self, title: &str) {
        *self.title.borrow_mut() = title.to_string();
    }
}

pub fn parse_kind(raw: &str) -> Kind {
    match raw.trim().to_ascii_lowercase().as_str() {
        "a" => Kind::Lights,
        "b" => Kind::Hover,
        "c" => Kind::Bar,
        _ => Kind::Off,
    }
}

pub fn kind_from_env() -> Kind {
    std::env::var("SPIKE_CHROME")
        .ok()
        .as_deref()
        .map(parse_kind)
        .unwrap_or(Kind::Off)
}

pub fn letter(kind: Kind) -> &'static str {
    match kind {
        Kind::Off => "",
        Kind::Lights => "a",
        Kind::Hover => "b",
        Kind::Bar => "c",
    }
}

pub fn title_face(title: &str) -> &'static str {
    if title
        .chars()
        .any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
    {
        "Songti SC"
    } else {
        "Iowan Old Style"
    }
}

pub fn mats(kind: Kind) -> Option<Mats> {
    match kind {
        Kind::Off => None,
        Kind::Lights => Some(Mats {
            side: SIDE,
            top: TOP_OVERLAY,
            bottom: BOTTOM_OVERLAY,
            bar: 0.0,
            footer: FOOTER,
            lock: true,
        }),
        Kind::Hover => Some(Mats {
            side: SIDE,
            top: TOP_OVERLAY,
            bottom: BOTTOM_OVERLAY,
            bar: 0.0,
            footer: FOOTER,
            lock: false,
        }),
        Kind::Bar => Some(Mats {
            side: SIDE,
            top: TOP_BAR,
            bottom: BOTTOM_BAR,
            bar: BAR,
            footer: FOOTER,
            lock: true,
        }),
    }
}

pub fn vertical(mats: &Mats) -> f64 {
    mats.bar + mats.top + mats.bottom + mats.footer
}

pub fn fit(
    kind: Kind,
    ratio_w: f64,
    ratio_h: f64,
    visible_w: f64,
    visible_h: f64,
) -> Option<Frame> {
    let mats = mats(kind)?;
    let ratio = if ratio_h <= 0.0 {
        1.0
    } else {
        (ratio_w / ratio_h).max(0.05)
    };
    let vertical = vertical(&mats);
    let max_w = (visible_w * FIT).max(1.0);
    let max_h = (visible_h * FIT).max(1.0);
    let from_h = max_h - vertical;
    let from_w = (max_w - mats.side * 2.0) / ratio;
    let page_h = from_h.min(from_w).max(1.0);
    let page_w = page_h * ratio;
    Some(Frame {
        window_w: page_w + mats.side * 2.0,
        window_h: page_h + vertical,
        page_w,
        page_h,
    })
}

pub fn content_box(stage_w: f64, stage_h: f64) -> Option<(f64, f64, f64, f64)> {
    let mats = mats(kind_from_env())?;
    let box_w = (stage_w - mats.side * 2.0).max(1.0);
    let box_h = (stage_h - mats.top - mats.bottom).max(1.0);
    Some((box_w, box_h, mats.side, mats.top))
}

pub fn page_clears_titlebar(kind: Kind, page_y: f64) -> bool {
    let Some(mats) = mats(kind) else {
        return true;
    };
    let page_top = mats.bar + page_y;
    let title_bottom = if mats.bar > 0.0 {
        mats.bar
    } else {
        LIGHTS_BOTTOM
    };
    page_top + 0.5 >= title_bottom
}

pub fn should_drag_margin(x: f64, y: f64, placed: Option<(i32, i32, i32, i32)>) -> bool {
    if kind_from_env() == Kind::Off {
        return false;
    }
    let Some((left, top, width, height)) = placed else {
        return false;
    };
    let right = f64::from(left + width);
    let bottom = f64::from(top + height);
    x < f64::from(left) || y < f64::from(top) || x > right || y > bottom
}

pub fn opening_size() -> Option<(i32, i32)> {
    let kind = kind_from_env();
    if kind == Kind::Off {
        return None;
    }
    let mtm = MainThreadMarker::new()?;
    let screen = NSScreen::mainScreen(mtm)?;
    let visible = screen.visibleFrame();
    let id = crate::settings::load(&crate::settings::path());
    let item = crate::reading::trim(&id);
    let frame = fit(
        kind,
        f64::from(item.width_mm),
        f64::from(item.height_mm),
        visible.size.width,
        visible.size.height,
    )?;
    Some((frame.window_w.round() as i32, frame.window_h.round() as i32))
}

struct Release<'a>(&'a Cell<bool>);

impl Drop for Release<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

pub fn sync(window: &NSWindow, rt: &Runtime, page: PageBox) {
    if rt.entering.replace(true) {
        return;
    }
    let _release = Release(&rt.entering);
    let kind = kind_from_env();
    let Some(mats) = mats(kind) else {
        return;
    };
    rt.page.set(page);
    style_window(window, rt);
    let fullscreen = window.styleMask().contains(NSWindowStyleMask::FullScreen);
    if fullscreen {
        window.setContentAspectRatio(NSSize::new(0.0, 0.0));
        paint_lights(window, 1.0);
        rt.lights.set(1.0);
        rt.full.set(true);
        let title_alpha = if kind == Kind::Hover {
            hover_target(window)
        } else {
            0.0
        };
        place_title(window, rt, kind, title_alpha);
        write_json(window, rt, kind, &mats, true);
        return;
    }
    if rt.full.replace(false) {
        rt.settled.set(false);
        rt.stable.set(0);
        rt.attempts.set(0);
    }
    let key = (page.ratio_w.round() as i32, page.ratio_h.round() as i32);
    if key.0 > 0 && key.1 > 0 && rt.ratio_key.get() != key {
        rt.ratio_key.set(key);
        rt.settled.set(false);
        rt.stable.set(0);
        rt.attempts.set(0);
    }
    if !rt.settled.get() && page.ratio_w > 0.0 && page.ratio_h > 0.0 {
        if let Some(screen) = window
            .screen()
            .or_else(|| MainThreadMarker::new().and_then(NSScreen::mainScreen))
        {
            let visible = screen.visibleFrame();
            if let Some(frame) = fit(
                kind,
                page.ratio_w,
                page.ratio_h,
                visible.size.width,
                visible.size.height,
            ) {
                if mats.lock {
                    window.setContentAspectRatio(NSSize::new(frame.window_w, frame.window_h));
                }
                let place = !rt.shown.get();
                let animate = rt.shown.get();
                apply_frame(window, &screen, &frame, animate, place);
                rt.shown.set(true);
                let got = window.frame().size;
                let close = (got.width - frame.window_w).abs() < 3.0
                    && (got.height - frame.window_h).abs() < 3.0;
                let tries = rt.attempts.get().saturating_add(1);
                rt.attempts.set(tries);
                if close {
                    let count = rt.stable.get().saturating_add(1);
                    rt.stable.set(count);
                    if count >= 3 || tries >= 12 {
                        rt.settled.set(true);
                    }
                } else if tries >= 12 {
                    rt.settled.set(true);
                } else {
                    rt.stable.set(0);
                }
            }
        }
    }
    let target = hover_target(window);
    if kind == Kind::Bar {
        paint_lights(window, 1.0);
        rt.lights.set(1.0);
    } else {
        let next = approach(rt.lights.get(), target);
        rt.lights.set(next);
        paint_lights(window, next);
    }
    let title_alpha = if kind == Kind::Hover {
        rt.lights.get()
    } else {
        0.0
    };
    place_title(window, rt, kind, title_alpha);
    if rt.settled.get() && !rt.fullscreen_sent.get() && flag("SPIKE_FULLSCREEN") {
        window.setContentAspectRatio(NSSize::new(0.0, 0.0));
        window.toggleFullScreen(None);
        rt.fullscreen_sent.set(true);
    }
    write_json(window, rt, kind, &mats, false);
}

fn style_window(window: &NSWindow, rt: &Runtime) {
    if rt.styled.replace(true) {
        return;
    }
    let color =
        NSColor::colorWithRed_green_blue_alpha(243.0 / 255.0, 238.0 / 255.0, 228.0 / 255.0, 1.0);
    window.setBackgroundColor(Some(&color));
    window.makeKeyAndOrderFront(None);
    let mtm = MainThreadMarker::from(&*window);
    let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
    unsafe {
        let _: () = msg_send![&*app, activateIgnoringOtherApps: true];
    }
    window.setTitlebarSeparatorStyle(NSTitlebarSeparatorStyle::None);
    window.setMovableByWindowBackground(true);
    window.setCollectionBehavior(
        window.collectionBehavior() | NSWindowCollectionBehavior::FullScreenPrimary,
    );
}

fn apply_frame(window: &NSWindow, screen: &NSScreen, frame: &Frame, animate: bool, place: bool) {
    let visible = screen.visibleFrame();
    let width = frame.window_w;
    let height = frame.window_h;
    let current = window.frame();
    let size_ok =
        (current.size.width - width).abs() < 2.0 && (current.size.height - height).abs() < 2.0;
    if size_ok && !place {
        return;
    }
    let x = if place {
        visible.origin.x + (visible.size.width - width) / 2.0
    } else {
        current.origin.x + (current.size.width - width) / 2.0
    };
    let y = if place {
        visible.origin.y + (visible.size.height - height) / 2.0
    } else {
        current.origin.y + current.size.height - height
    };
    let x = x.max(visible.origin.x);
    let y = y.max(visible.origin.y);
    window.setFrame_display_animate(
        NSRect::new(NSPoint::new(x, y), NSSize::new(width, height)),
        true,
        animate,
    );
}

fn hover_target(window: &NSWindow) -> f64 {
    if flag("SPIKE_CHROME_SHOW") || near_top(window) {
        1.0
    } else {
        0.0
    }
}

fn near_top(window: &NSWindow) -> bool {
    let mouse = objc2_app_kit::NSEvent::mouseLocation();
    let frame = window.frame();
    let top = frame.origin.y + frame.size.height;
    mouse.x >= frame.origin.x
        && mouse.x <= frame.origin.x + frame.size.width
        && mouse.y <= top
        && mouse.y >= top - 48.0
}

fn approach(current: f64, target: f64) -> f64 {
    let delta = (target - current).clamp(-0.25, 0.25);
    (current + delta).clamp(0.0, 1.0)
}

fn paint_lights(window: &NSWindow, alpha: f64) {
    for button in [
        NSWindowButton::CloseButton,
        NSWindowButton::MiniaturizeButton,
        NSWindowButton::ZoomButton,
    ] {
        let Some(button) = window.standardWindowButton(button) else {
            continue;
        };
        unsafe {
            if alpha < 0.02 {
                let _: () = msg_send![&*button, setAlphaValue: 0.0_f64];
                let _: () = msg_send![&*button, setHidden: true];
            } else {
                let _: () = msg_send![&*button, setHidden: false];
                let _: () = msg_send![&*button, setAlphaValue: alpha];
            }
        }
    }
}

fn place_title(window: &NSWindow, rt: &Runtime, kind: Kind, alpha: f64) {
    let title = rt.title.borrow().clone();
    let shown = if title.is_empty() || title == "SuperReader spike" {
        String::new()
    } else {
        title
    };
    if !shown.is_empty() {
        window.setTitle(&NSString::from_str(&shown));
    }
    if kind != Kind::Hover {
        return;
    }
    let mtm = MainThreadMarker::from(&*window);
    let mut slot = rt.field.borrow_mut();
    if slot.is_none() {
        let field = NSTextField::labelWithString(&NSString::from_str(""), mtm);
        field.setTextColor(Some(&NSColor::colorWithRed_green_blue_alpha(
            0.36, 0.33, 0.29, 1.0,
        )));
        if let Some(content) = window.contentView() {
            let view = Retained::into_super(field.clone());
            content.addSubview(&view);
        }
        *slot = Some(field);
    }
    let Some(field) = slot.as_ref() else {
        return;
    };
    let face = NSString::from_str(title_face(&shown));
    if let Some(font) = NSFont::fontWithName_size(&face, 13.0) {
        unsafe {
            let _: () = msg_send![&*field, setFont: &*font];
        }
    }
    unsafe {
        let value = NSString::from_str(&shown);
        let _: () = msg_send![&*field, setStringValue: &*value];
        let _: () = msg_send![&*field, setAlignment: objc2_app_kit::NSTextAlignment(1)];
        let _: () = msg_send![&*field, setAlphaValue: alpha];
    }
    if let Some(content) = window.contentView() {
        let bounds = content.bounds();
        let width = (bounds.size.width - 160.0).max(40.0);
        let x = (bounds.size.width - width) / 2.0;
        let y = bounds.size.height - 24.0;
        unsafe {
            let _: () = msg_send![
                &*field,
                setFrame: NSRect::new(NSPoint::new(x, y), NSSize::new(width, 18.0))
            ];
        }
    }
}

fn write_json(window: &NSWindow, rt: &Runtime, kind: Kind, mats: &Mats, fullscreen: bool) {
    let page = rt.page.get();
    let frame = window.frame();
    let body = serde_json::json!({
        "kind": letter(kind),
        "lock": mats.lock,
        "window_w": frame.size.width,
        "window_h": frame.size.height,
        "page_x": page.x,
        "page_y": page.y,
        "page_w": page.w,
        "page_h": page.h,
        "stage_w": page.stage_w,
        "stage_h": page.stage_h,
        "bar": mats.bar,
        "top": mats.top,
        "bottom": mats.bottom,
        "side": mats.side,
        "footer": mats.footer,
        "fullscreen": fullscreen,
        "lights": rt.lights.get(),
        "ratio_w": page.ratio_w,
        "ratio_h": page.ratio_h,
        "title": rt.title.borrow().clone(),
    });
    let _ = std::fs::create_dir_all("/tmp/superreader-spike");
    let _ = std::fs::write(
        "/tmp/superreader-spike/chrome.json",
        serde_json::to_string_pretty(&body).unwrap_or_default(),
    );
}

fn flag(name: &str) -> bool {
    std::env::var(name).ok().as_deref() == Some("1")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(got: f64, want: f64) {
        assert!((got - want).abs() < 0.02, "{got} vs {want}");
    }

    #[test]
    fn side_mats_equal_vertical_chrome_so_a_lock_keeps_the_page_ratio() {
        for kind in [Kind::Lights, Kind::Hover, Kind::Bar] {
            let mats = mats(kind).unwrap();
            close(mats.side * 2.0, vertical(&mats));
        }
        assert!(mats(Kind::Lights).unwrap().lock);
        assert!(!mats(Kind::Hover).unwrap().lock);
        assert!(mats(Kind::Bar).unwrap().lock);
    }

    #[test]
    fn paperback_on_a_1000_by_1200_frame_matches_the_hand_fit() {
        let frame = fit(Kind::Lights, 129.0, 198.0, 1000.0, 1200.0).unwrap();
        close(frame.page_h, 952.0);
        close(frame.page_w, 952.0 * 129.0 / 198.0);
        close(frame.window_h, 1056.0);
        close(frame.window_w, frame.page_w + 104.0);
        let inner_w = frame.window_w - 104.0;
        let inner_h = frame.window_h - 104.0;
        close(inner_w / inner_h, 129.0 / 198.0);
        let scaled_w = frame.window_w * 1.4 - 104.0;
        let scaled_h = frame.window_h * 1.4 - 104.0;
        close(scaled_w / scaled_h, 129.0 / 198.0);
    }

    #[test]
    fn the_three_prototypes_open_at_the_same_size() {
        let a = fit(Kind::Lights, 130.0, 184.0, 1512.0, 982.0).unwrap();
        let b = fit(Kind::Hover, 130.0, 184.0, 1512.0, 982.0).unwrap();
        let c = fit(Kind::Bar, 130.0, 184.0, 1512.0, 982.0).unwrap();
        close(a.window_w, b.window_w);
        close(a.window_h, b.window_h);
        close(a.window_w, c.window_w);
        close(a.window_h, c.window_h);
        assert!(a.window_w <= 1512.0 * FIT + 0.1);
        assert!(a.window_h <= 982.0 * FIT + 0.1);
    }

    #[test]
    fn the_page_starts_below_the_traffic_lights() {
        assert!(page_clears_titlebar(Kind::Lights, TOP_OVERLAY));
        assert!(page_clears_titlebar(Kind::Hover, TOP_OVERLAY));
        assert!(page_clears_titlebar(Kind::Bar, TOP_BAR));
        assert!(!page_clears_titlebar(Kind::Lights, 0.0));
    }

    #[test]
    fn names_are_a_b_and_c() {
        assert_eq!(parse_kind("a"), Kind::Lights);
        assert_eq!(parse_kind("B"), Kind::Hover);
        assert_eq!(parse_kind("c"), Kind::Bar);
        assert_eq!(parse_kind("paperback"), Kind::Off);
        assert_eq!(letter(Kind::Bar), "c");
        assert_eq!(title_face("朝花夕拾"), "Songti SC");
        assert_eq!(title_face("Pride and Prejudice"), "Iowan Old Style");
    }
}
