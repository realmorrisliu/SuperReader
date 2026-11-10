use std::cell::{Cell, RefCell};

use nsh_platform_macos::objc2::rc::Retained;
use nsh_platform_macos::objc2::runtime::{AnyObject, NSObjectProtocol, ProtocolObject};
use nsh_platform_macos::objc2::{
    define_class, msg_send, DefinedClass, MainThreadMarker, MainThreadOnly,
};
use objc2_app_kit::{
    NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua,
    NSColor, NSFont, NSFontWeightRegular, NSImage, NSImageAlignment, NSImageScaling,
    NSImageSymbolConfiguration, NSImageView, NSScreen, NSTextField, NSTitlebarSeparatorStyle,
    NSToolbar, NSToolbarDelegate, NSToolbarDisplayMode, NSToolbarItem, NSView, NSWindow,
    NSWindowButton, NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_foundation::{NSArray, NSPoint, NSRect, NSSize, NSString};

#[path = "icons.rs"]
mod icons;

pub const SIDE: f64 = 52.0;
pub const TOP: f64 = 48.0;
pub const FIT: f64 = 0.88;

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
    pub full: Cell<bool>,
    fullscreen_sent: Cell<bool>,
    lights: Cell<f64>,
    ratio_key: Cell<(i32, i32)>,
    page: Cell<PageBox>,
    title: RefCell<String>,
    styled: Cell<bool>,
    entering: Cell<bool>,
    field: RefCell<Option<Retained<NSTextField>>>,
    contents_bar: RefCell<Option<Retained<ContentsBar>>>,
    icon_gap: Cell<f64>,
    icon_dy: Cell<f64>,
    pub design_visible: Cell<(f64, f64)>,
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
            contents_bar: RefCell::new(None),
            icon_gap: Cell::new(f64::NAN),
            icon_dy: Cell::new(f64::NAN),
            design_visible: Cell::new((0.0, 0.0)),
        }
    }
}

impl Runtime {
    pub fn set_title(&self, title: &str) {
        *self.title.borrow_mut() = title.to_string();
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

pub fn fit(ratio_w: f64, ratio_h: f64, visible_w: f64, visible_h: f64) -> Frame {
    let ratio = if ratio_h <= 0.0 {
        1.0
    } else {
        (ratio_w / ratio_h).max(0.05)
    };
    let max_w = (visible_w * FIT).max(1.0);
    let max_h = (visible_h * FIT).max(1.0);
    let from_h = max_h - TOP;
    let from_w = (max_w - SIDE * 2.0) / ratio;
    let page_h = from_h.min(from_w).max(1.0);
    let page_w = page_h * ratio;
    Frame {
        window_w: page_w + SIDE * 2.0,
        window_h: page_h + TOP,
        page_w,
        page_h,
    }
}

pub fn content_box(stage_w: f64, stage_h: f64) -> (f64, f64, f64, f64) {
    let box_w = (stage_w - SIDE * 2.0).max(1.0);
    let box_h = (stage_h - TOP).max(1.0);
    (box_w, box_h, SIDE, TOP)
}

pub fn reading_page(
    ratio_w: f64,
    ratio_h: f64,
    full: bool,
    remembered: &Cell<(f64, f64)>,
) -> (f64, f64) {
    let Some(mtm) = MainThreadMarker::new() else {
        return (1.0, 1.0);
    };
    let Some(screen) = NSScreen::mainScreen(mtm) else {
        return (1.0, 1.0);
    };
    let visible = screen.visibleFrame().size;
    let saved = remembered.get();
    if !full && (saved.0 < 1.0 || visible.height <= saved.1 + 1.0) {
        remembered.set((visible.width, visible.height));
    }
    let saved = remembered.get();
    let (width, height) = if saved.0 > 1.0 {
        saved
    } else {
        (visible.width, visible.height)
    };
    let frame = fit(ratio_w, ratio_h, width, height);
    (frame.page_w, frame.page_h)
}

pub fn should_drag_margin(x: f64, y: f64, placed: Option<(i32, i32, i32, i32)>) -> bool {
    let Some((left, top, width, height)) = placed else {
        return false;
    };
    let right = f64::from(left + width);
    let bottom = f64::from(top + height);
    x < f64::from(left) || y < f64::from(top) || x > right || y > bottom
}

pub fn opening_size() -> (i32, i32) {
    if let Some((width, height)) = spike_window() {
        return (width.round() as i32, height.round() as i32);
    }
    let id = crate::settings::load(&crate::settings::path());
    let item = crate::reading::trim(&id);
    let Some(mtm) = MainThreadMarker::new() else {
        return (700, 1040);
    };
    let Some(screen) = NSScreen::mainScreen(mtm) else {
        return (700, 1040);
    };
    let visible = screen.visibleFrame();
    let frame = fit(
        f64::from(item.width_mm),
        f64::from(item.height_mm),
        visible.size.width,
        visible.size.height,
    );
    (frame.window_w.round() as i32, frame.window_h.round() as i32)
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
    rt.page.set(page);
    style_window(window, rt);
    let fullscreen = window.styleMask().contains(NSWindowStyleMask::FullScreen);
    if fullscreen {
        paint_lights(window, 1.0);
        rt.lights.set(1.0);
        rt.full.set(true);
        let shown = hover_target(window);
        place_title(window, rt, shown);
        place_contents(window, rt, shown);
        write_json(window, rt, true);
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
            let frame = if let Some((width, height)) = spike_window() {
                Frame {
                    window_w: width,
                    window_h: height,
                    page_w: (width - SIDE * 2.0).max(1.0),
                    page_h: (height - TOP).max(1.0),
                }
            } else {
                fit(
                    page.ratio_w,
                    page.ratio_h,
                    visible.size.width,
                    visible.size.height,
                )
            };
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
    let next = if flag("SPIKE_LIGHTS") {
        1.0
    } else {
        approach(rt.lights.get(), hover_target(window))
    };
    rt.lights.set(next);
    paint_lights(window, next);
    place_title(window, rt, next);
    place_contents(window, rt, next);
    if rt.settled.get() && !rt.fullscreen_sent.get() && flag("SPIKE_FULLSCREEN") {
        window.toggleFullScreen(None);
        rt.fullscreen_sent.set(true);
    }
    write_json(window, rt, false);
}

pub fn paint_paper_layer(view: &AnyObject) {
    unsafe {
        let _: () = msg_send![view, setWantsLayer: true];
        let layer: Retained<AnyObject> = msg_send![view, layer];
        let (red, green, blue) = crate::reading::paper_rgb();
        let color = objc2_core_graphics::CGColor::new_srgb(red, green, blue, 1.0);
        let _: () = msg_send![&layer, setBackgroundColor: &*color];
    }
}

fn style_window(window: &NSWindow, rt: &Runtime) {
    if rt.styled.replace(true) {
        return;
    }
    apply_appearance(window);
    window.setTitlebarSeparatorStyle(NSTitlebarSeparatorStyle::None);
    window.setMovableByWindowBackground(true);
    window.setCollectionBehavior(
        window.collectionBehavior() | NSWindowCollectionBehavior::FullScreenPrimary,
    );
    window.makeKeyAndOrderFront(None);
    let mtm = MainThreadMarker::from(&*window);
    let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
    unsafe {
        let _: () = msg_send![&*app, activateIgnoringOtherApps: true];
    }
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
    window.setFrame_display_animate(
        NSRect::new(
            NSPoint::new(x.max(visible.origin.x), y.max(visible.origin.y)),
            NSSize::new(width, height),
        ),
        true,
        animate,
    );
}

fn hover_target(window: &NSWindow) -> f64 {
    if near_top(window) {
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
        && mouse.y >= top - 56.0
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

fn place_title(window: &NSWindow, rt: &Runtime, alpha: f64) {
    let title = rt.title.borrow().clone();
    let shown = if title.is_empty() || title == "SuperReader spike" {
        String::new()
    } else {
        title
    };
    if !shown.is_empty() {
        window.setTitle(&NSString::from_str(&shown));
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
        let width = (bounds.size.width - 180.0).max(40.0);
        let x = (bounds.size.width - width) / 2.0;
        let y = bounds.size.height - 36.0;
        unsafe {
            let _: () = msg_send![
                &*field,
                setFrame: NSRect::new(NSPoint::new(x, y), NSSize::new(width, 18.0))
            ];
        }
    }
}

fn apply_appearance(window: &NSWindow) {
    // AppOptions pins the window to aqua. This overrides that for the icon shots.
    let Ok(mode) = std::env::var("SPIKE_APPEARANCE") else {
        return;
    };
    let name: &NSString = if mode == "dark" {
        unsafe { NSAppearanceNameDarkAqua }
    } else {
        unsafe { NSAppearanceNameAqua }
    };
    let Some(appearance) = NSAppearance::appearanceNamed(name) else {
        return;
    };
    window.setAppearance(Some(&appearance));
}

const CONTENTS_ID: &str = "superreader.contents";

fn place_contents(window: &NSWindow, rt: &Runtime, alpha: f64) {
    let Some(toolbar) = window.toolbar() else {
        return;
    };
    if std::env::var("SPIKE_ICON").ok().as_deref() == Some("0") {
        clear_contents(&toolbar);
        return;
    }
    let image = if std::env::var("SPIKE_ICON").ok().as_deref() == Some("sf") {
        reference_symbol()
    } else {
        icons::contents_image()
    };
    let Some(image) = image else {
        return;
    };
    ensure_contents(window, rt, &toolbar, image);
    let Some(view) = contents_view(&toolbar) else {
        return;
    };
    view.setHidden(alpha < 0.02);
    view.setAlphaValue(alpha);
    record_gap(window, rt, &view);
}

fn clear_contents(toolbar: &NSToolbar) {
    if contents_view(toolbar).is_none() {
        return;
    }
    toolbar.setItemIdentifiers(&NSArray::from_slice(&[] as &[&NSString]));
}

fn ensure_contents(window: &NSWindow, rt: &Runtime, toolbar: &NSToolbar, image: Retained<NSImage>) {
    if rt.contents_bar.borrow().is_some() {
        return;
    }
    let mtm = MainThreadMarker::from(&*window);
    let bar = mtm.alloc::<ContentsBar>().set_ivars(ContentsIvars {
        image,
        row: row_height(window),
    });
    let bar: Retained<ContentsBar> = unsafe { msg_send![super(bar), init] };
    toolbar.setAutosavesConfiguration(false);
    toolbar.setDisplayMode(NSToolbarDisplayMode::IconOnly);
    toolbar.setDelegate(Some(ProtocolObject::from_ref(&*bar)));
    let id = NSString::from_str(CONTENTS_ID);
    toolbar.setItemIdentifiers(&NSArray::from_slice(&[&*id]));
    *rt.contents_bar.borrow_mut() = Some(bar);
}

fn contents_view(toolbar: &NSToolbar) -> Option<Retained<NSView>> {
    toolbar.items().iter().find_map(|item| {
        if item.itemIdentifier().to_string() == CONTENTS_ID {
            item.view()
        } else {
            None
        }
    })
}

fn row_height(window: &NSWindow) -> f64 {
    let Some(button) = window.standardWindowButton(NSWindowButton::CloseButton) else {
        return 18.0;
    };
    let host: Option<Retained<NSView>> = unsafe { msg_send![&*button, superview] };
    let Some(host) = host else {
        return 18.0;
    };
    let height = host.frame().size.height;
    if height.is_finite() && height > 18.0 {
        height
    } else {
        18.0
    }
}

fn reference_symbol() -> Option<Retained<NSImage>> {
    let base = NSImage::imageWithSystemSymbolName_accessibilityDescription(
        &NSString::from_str("list.bullet"),
        None,
    )?;
    let weight = unsafe { NSFontWeightRegular };
    let config = NSImageSymbolConfiguration::configurationWithPointSize_weight(16.0, weight);
    let image = base.imageWithSymbolConfiguration(&config).unwrap_or(base);
    image.setTemplate(true);
    Some(image)
}

fn record_gap(window: &NSWindow, rt: &Runtime, view: &NSView) {
    let Some(zoom) = window.standardWindowButton(NSWindowButton::ZoomButton) else {
        return;
    };
    let zoom_bounds: NSRect = unsafe { msg_send![&*zoom, bounds] };
    let zoom_win: NSRect =
        unsafe { msg_send![&*zoom, convertRect: zoom_bounds, toView: std::ptr::null::<NSView>()] };
    let item_win = view.convertRect_toView(view.bounds(), None);
    let gap = item_win.origin.x - (zoom_win.origin.x + zoom_win.size.width);
    let zoom_mid = zoom_win.origin.y + zoom_win.size.height / 2.0;
    let item_mid = item_win.origin.y + item_win.size.height / 2.0;
    let dy = item_mid - zoom_mid;
    let first = rt.icon_gap.get().is_nan();
    rt.icon_gap.set(gap);
    rt.icon_dy.set(dy);
    if first {
        eprintln!("contents toolbar gap {gap:.1} pt, vertical delta {dy:.1} pt");
    }
}

struct ContentsIvars {
    image: Retained<NSImage>,
    row: f64,
}

define_class!(
    #[unsafe(super(objc2::runtime::NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = ContentsIvars]
    struct ContentsBar;

    unsafe impl NSObjectProtocol for ContentsBar {}

    unsafe impl NSToolbarDelegate for ContentsBar {}

    impl ContentsBar {
        #[unsafe(method_id(toolbar:itemForItemIdentifier:willBeInsertedIntoToolbar:))]
        fn item_for(
            &self,
            _toolbar: &NSToolbar,
            identifier: &NSString,
            _insert: bool,
        ) -> Option<Retained<NSToolbarItem>> {
            if identifier.to_string() != CONTENTS_ID {
                None
            } else {
                let mtm = MainThreadMarker::from(self);
                let item =
                    NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), identifier);
                let label = NSString::from_str("Contents");
                item.setLabel(&label);
                item.setPaletteLabel(&label);
                item.setBordered(false);
                let image = &self.ivars().image;
                let view = NSImageView::imageViewWithImage(image, mtm);
                view.setImageAlignment(NSImageAlignment::AlignCenter);
                view.setImageScaling(NSImageScaling::ScaleNone);
                let width = image.size().width.max(18.0);
                let height = self.ivars().row.max(18.0);
                view.setFrame(NSRect::new(
                    NSPoint::new(0.0, 0.0),
                    NSSize::new(width, height),
                ));
                let hosted = view.into_super().into_super();
                item.setView(Some(&hosted));
                Some(item)
            }
        }

        #[unsafe(method_id(toolbarDefaultItemIdentifiers:))]
        fn default_ids(&self, _toolbar: &NSToolbar) -> Retained<NSArray<NSString>> {
            let id = NSString::from_str(CONTENTS_ID);
            NSArray::from_slice(&[&*id])
        }

        #[unsafe(method_id(toolbarAllowedItemIdentifiers:))]
        fn allowed_ids(&self, _toolbar: &NSToolbar) -> Retained<NSArray<NSString>> {
            let id = NSString::from_str(CONTENTS_ID);
            NSArray::from_slice(&[&*id])
        }
    }
);

fn write_json(window: &NSWindow, rt: &Runtime, fullscreen: bool) {
    let page = rt.page.get();
    let frame = window.frame();
    let body = serde_json::json!({
        "window_w": frame.size.width,
        "window_h": frame.size.height,
        "page_x": page.x,
        "page_y": page.y,
        "page_w": page.w,
        "page_h": page.h,
        "stage_w": page.stage_w,
        "stage_h": page.stage_h,
        "top": TOP,
        "side": SIDE,
        "fullscreen": fullscreen,
        "lights": rt.lights.get(),
        "ratio_w": page.ratio_w,
        "ratio_h": page.ratio_h,
        "title": rt.title.borrow().clone(),
        "appearance": window.effectiveAppearance().name().to_string(),
        "icon_gap": finite_json(rt.icon_gap.get()),
        "icon_dy": finite_json(rt.icon_dy.get()),
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

fn finite_json(value: f64) -> serde_json::Value {
    if value.is_finite() {
        serde_json::json!(value)
    } else {
        serde_json::Value::Null
    }
}

fn spike_window() -> Option<(f64, f64)> {
    parse_spike_window(&std::env::var("SPIKE_WINDOW").ok()?)
}

fn parse_spike_window(raw: &str) -> Option<(f64, f64)> {
    let (width, height) = raw.split_once('x')?;
    let width: f64 = width.trim().parse().ok()?;
    let height: f64 = height.trim().parse().ok()?;
    if !(200.0..=4000.0).contains(&width) || !(200.0..=4000.0).contains(&height) {
        return None;
    }
    Some((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(got: f64, want: f64) {
        assert!((got - want).abs() < 0.02, "{got} vs {want}");
    }

    #[test]
    fn spike_window_parses_a_frame_and_rejects_a_partial_one() {
        assert_eq!(parse_spike_window("640x960"), Some((640.0, 960.0)));
        assert_eq!(parse_spike_window(" 700 x 800 "), Some((700.0, 800.0)));
        assert_eq!(parse_spike_window("640"), None);
        assert_eq!(parse_spike_window("10x10"), None);
    }

    #[test]
    fn paperback_on_a_1000_by_1200_frame_is_the_page_plus_the_top_mat() {
        let frame = fit(129.0, 198.0, 1000.0, 1200.0);
        close(frame.page_h, 1056.0 - TOP);
        close(frame.page_w, frame.page_h * 129.0 / 198.0);
        close(frame.window_h, frame.page_h + TOP);
        close(frame.window_w, frame.page_w + SIDE * 2.0);
        assert!(frame.window_w <= 1000.0 * FIT + 0.1);
        assert!(frame.window_h <= 1200.0 * FIT + 0.1);
    }

    #[test]
    fn the_page_starts_below_the_traffic_lights() {
        let (_, _, _, top) = content_box(800.0, 1000.0);
        assert!(top >= 44.0);
    }

    #[test]
    fn a_cjk_title_uses_songti() {
        assert_eq!(title_face("朝花夕拾"), "Songti SC");
        assert_eq!(title_face("Pride and Prejudice"), "Iowan Old Style");
    }
}
