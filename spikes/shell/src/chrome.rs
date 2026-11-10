use std::cell::{Cell, RefCell};

use nsh_platform_macos::objc2::rc::Retained;
use nsh_platform_macos::objc2::runtime::{AnyObject, ProtocolObject};
use nsh_platform_macos::objc2::{msg_send, sel, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSBackingStoreType, NSBox, NSBoxType, NSButton, NSColor, NSEventType, NSFont, NSImage,
    NSScreen, NSSlider, NSTextField, NSTitlePosition, NSTitlebarSeparatorStyle,
    NSToolbarDelegate, NSToolbarDisplayMode, NSToolbarFlexibleSpaceItemIdentifier, NSToolbarItem,
    NSToolbarItemGroup, NSToolbarItemGroupControlRepresentation, NSToolbarItemGroupSelectionMode,
    NSView, NSWindow, NSWindowButton, NSWindowCollectionBehavior, NSWindowStyleMask,
    NSFontWeightRegular, NSFontWeightSemibold,
};
use objc2_foundation::{NSArray, NSPoint, NSRect, NSSize, NSString};

pub const TOP: f64 = crate::reading::WINDOW_TOP_MAT;
pub const FIT: f64 = 0.88;
pub const TOOLBAR_ORDER: [&str; 5] = [
    "books.contents",
    "books.bookmarks",
    "books.search",
    "books.add-bookmark",
    "books.notes",
];

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
    pub design_visible: Cell<(f64, f64)>,
    chevron_alpha: Cell<f64>,
    scrub_alpha: Cell<f64>,
    last_fade: Cell<Option<std::time::Instant>>,
    toolbar_ready: Cell<bool>,
    items_requested: Cell<bool>,
    pub tracking: Cell<bool>,
    pub scrub_page: Cell<u32>,
    scrub_pages: Cell<u32>,
    pub jump_origin: Cell<u32>,
    pub jump_previous: Cell<u32>,
    ticks: RefCell<Vec<(u32, String)>>,
    field: RefCell<Option<Retained<NSTextField>>>,
    previous: RefCell<Option<Retained<NSButton>>>,
    next: RefCell<Option<Retained<NSButton>>>,
    panel: RefCell<Option<Retained<NSWindow>>>,
    panel_body: RefCell<Option<Retained<NSTextField>>>,
    scrub: RefCell<Option<Retained<NSSlider>>>,
    preview: RefCell<Option<Retained<NSTextField>>>,
    tick_views: RefCell<Vec<Retained<NSBox>>>,
    back_previous: RefCell<Option<Retained<NSButton>>>,
    back_origin: RefCell<Option<Retained<NSButton>>>,
    pdf_num: RefCell<Option<Retained<NSTextField>>>,
    pdf_rest: RefCell<Option<Retained<NSTextField>>>,
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
            design_visible: Cell::new((0.0, 0.0)),
            chevron_alpha: Cell::new(0.0),
            scrub_alpha: Cell::new(0.0),
            last_fade: Cell::new(None),
            toolbar_ready: Cell::new(false),
            items_requested: Cell::new(false),
            tracking: Cell::new(false),
            scrub_page: Cell::new(1),
            scrub_pages: Cell::new(1),
            jump_origin: Cell::new(0),
            jump_previous: Cell::new(0),
            ticks: RefCell::new(Vec::new()),
            field: RefCell::new(None),
            previous: RefCell::new(None),
            next: RefCell::new(None),
            panel: RefCell::new(None),
            panel_body: RefCell::new(None),
            scrub: RefCell::new(None),
            preview: RefCell::new(None),
            tick_views: RefCell::new(Vec::new()),
            back_previous: RefCell::new(None),
            back_origin: RefCell::new(None),
            pdf_num: RefCell::new(None),
            pdf_rest: RefCell::new(None),
        }
    }
}

impl Runtime {
    pub fn set_progress(&self, page: u32, pages: u32) {
        let pages = pages.max(1);
        self.scrub_page.set(page.clamp(1, pages));
        self.scrub_pages.set(pages);
    }

    pub fn set_ticks(&self, ticks: Vec<(u32, String)>) {
        *self.ticks.borrow_mut() = ticks;
    }

    pub fn note_jump(&self, from: u32) {
        if self.jump_origin.get() == 0 {
            self.jump_origin.set(from);
        }
        self.jump_previous.set(from);
    }

    pub fn clear_jump(&self) {
        self.jump_origin.set(0);
        self.jump_previous.set(0);
    }
}

impl Runtime {
    pub fn set_title(&self, title: &str) {
        *self.title.borrow_mut() = title.to_string();
    }
}

fn side_inset(width: f64) -> f64 {
    (width * crate::reading::TEXT_SIDE_RATIO).max(1.0)
}

pub fn fit(ratio_w: f64, ratio_h: f64, visible_w: f64, visible_h: f64) -> Frame {
    let ratio = if ratio_h <= 0.0 {
        1.0
    } else {
        (ratio_w / ratio_h).max(0.05)
    };
    let max_w = (visible_w * FIT).max(1.0);
    let max_h = (visible_h * FIT).max(1.0);
    let inner = (1.0 - 2.0 * crate::reading::TEXT_SIDE_RATIO).max(0.2);
    let from_h = max_h - TOP;
    let from_w = max_w * inner / ratio;
    let page_h = from_h.min(from_w).max(1.0);
    let page_w = page_h * ratio;
    let side = page_w * crate::reading::TEXT_SIDE_RATIO / inner;
    Frame {
        window_w: page_w + side * 2.0,
        window_h: page_h + TOP,
        page_w,
        page_h,
    }
}

pub fn content_box(stage_w: f64, stage_h: f64) -> (f64, f64, f64, f64) {
    let side = side_inset(stage_w);
    let box_w = (stage_w - side * 2.0).max(1.0);
    let box_h = (stage_h - TOP).max(1.0);
    (box_w, box_h, side, TOP)
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

pub fn sync(
    window: &NSWindow,
    delegate: Option<&ProtocolObject<dyn NSToolbarDelegate>>,
    host: &NSView,
    rt: &Runtime,
    page: PageBox,
) {
    if rt.entering.replace(true) {
        return;
    }
    let _release = Release(&rt.entering);
    rt.page.set(page);
    install_toolbar(window, delegate, rt);
    style_window(window, rt);
    let fullscreen = window.styleMask().contains(NSWindowStyleMask::FullScreen);
    if fullscreen {
        fade_titlebar(window, 1.0);
        place_chevrons(host, rt, page, 1.0);
        let steps = fade_steps(rt);
        let scrub = approach(rt.scrub_alpha.get(), scrub_target(window, rt), steps);
        rt.scrub_alpha.set(scrub);
        place_scrub(host, rt, scrub);
        rt.lights.set(1.0);
        rt.chevron_alpha.set(1.0);
        rt.full.set(true);
        place_title(window, rt, hover_target(window));
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
            let frame = fit(
                page.ratio_w,
                page.ratio_h,
                visible.size.width,
                visible.size.height,
            );
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
    let steps = fade_steps(rt);
    let next = approach(rt.lights.get(), hover_target(window), steps);
    rt.lights.set(next);
    let chevrons = approach(rt.chevron_alpha.get(), chrome_target(window), steps);
    rt.chevron_alpha.set(chevrons);
    let scrub = approach(rt.scrub_alpha.get(), scrub_target(window, rt), steps);
    rt.scrub_alpha.set(scrub);
    fade_titlebar(window, next);
    place_chevrons(host, rt, page, chevrons);
    place_scrub(host, rt, scrub);
    place_title(window, rt, next);
    if rt.settled.get() && !rt.fullscreen_sent.get() && flag("SPIKE_FULLSCREEN") {
        window.toggleFullScreen(None);
        rt.fullscreen_sent.set(true);
    }
    write_json(window, rt, false);
}

fn install_toolbar(
    window: &NSWindow,
    delegate: Option<&ProtocolObject<dyn NSToolbarDelegate>>,
    rt: &Runtime,
) {
    if rt.toolbar_ready.get() {
        return;
    }
    let Some(toolbar) = window.toolbar() else {
        return;
    };
    let Some(delegate) = delegate else {
        return;
    };
    if !rt.items_requested.replace(true) {
        toolbar.setDelegate(Some(delegate));
        toolbar.setDisplayMode(NSToolbarDisplayMode::IconOnly);
        toolbar.setAllowsUserCustomization(false);
        unsafe {
            let _: () = msg_send![&*toolbar, setAutosavesConfiguration: false];
        }
        if toolbar.items().is_empty() {
            let leading = NSString::from_str("books.leading");
            let add = NSString::from_str("books.add-bookmark");
            let notes = NSString::from_str("books.notes");
            let space = unsafe { &*NSToolbarFlexibleSpaceItemIdentifier };
            let ids: [&NSString; 4] = [&leading, space, &add, &notes];
            for (index, identifier) in ids.iter().enumerate() {
                toolbar.insertItemWithItemIdentifier_atIndex(identifier, index as isize);
            }
        }
    }
    rt.toolbar_ready.set(!toolbar.items().is_empty());
}

fn style_window(window: &NSWindow, rt: &Runtime) {
    if rt.styled.replace(true) {
        return;
    }
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

fn fade_steps(rt: &Runtime) -> f64 {
    let now = std::time::Instant::now();
    let dt = rt
        .last_fade
        .get()
        .map(|at| now.saturating_duration_since(at).as_secs_f64())
        .unwrap_or(0.05);
    rt.last_fade.set(Some(now));
    if dt < 0.016 {
        0.0
    } else {
        (dt / 0.05).clamp(1.0, 8.0)
    }
}

fn approach(current: f64, target: f64, steps: f64) -> f64 {
    if steps <= 0.0 {
        return current;
    }
    let delta = (target - current).clamp(-0.25 * steps, 0.25 * steps);
    (current + delta).clamp(0.0, 1.0)
}

fn fade_titlebar(window: &NSWindow, alpha: f64) {
    let Some(button) = window.standardWindowButton(NSWindowButton::CloseButton) else {
        return;
    };
    let container: Option<Retained<NSView>> = unsafe { msg_send![&*button, superview] };
    let Some(container) = container else {
        return;
    };
    unsafe {
        let _: () = msg_send![&*container, setAlphaValue: alpha];
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
        field.setTextColor(Some(&NSColor::labelColor()));
        field.setFont(Some(&NSFont::systemFontOfSize_weight(
            13.0,
            unsafe { NSFontWeightSemibold },
        )));
        if let Some(content) = window.contentView() {
            content.addSubview(&field);
        }
        *slot = Some(field);
    }
    let Some(field) = slot.as_ref() else {
        return;
    };
    unsafe {
        let value = NSString::from_str(&shown);
        let _: () = msg_send![&*field, setStringValue: &*value];
        let _: () = msg_send![&*field, setAlignment: objc2_app_kit::NSTextAlignment(1)];
        let _: () = msg_send![&*field, setAlphaValue: alpha];
        let _: () = msg_send![&*field, setHidden: alpha < 0.02];
    }
    if let Some(content) = window.contentView() {
        let bounds = content.bounds();
        let width = (bounds.size.width - 180.0).max(40.0);
        let x = (bounds.size.width - width) / 2.0;
        let y = bounds.size.height - 34.0;
        unsafe {
            let _: () = msg_send![
                &*field,
                setFrame: NSRect::new(NSPoint::new(x, y), NSSize::new(width, 20.0))
            ];
        }
    }
}

fn chrome_target(window: &NSWindow) -> f64 {
    if near_top(window) || near_side(window) {
        1.0
    } else {
        0.0
    }
}

fn near_side(window: &NSWindow) -> bool {
    let mouse = objc2_app_kit::NSEvent::mouseLocation();
    let frame = window.frame();
    let x = mouse.x - frame.origin.x;
    mouse.y >= frame.origin.y
        && mouse.y <= frame.origin.y + frame.size.height
        && (x <= 72.0 || x >= frame.size.width - 72.0)
}

fn toolbar_order(window: &NSWindow) -> Vec<String> {
    let Some(toolbar) = window.toolbar() else {
        return Vec::new();
    };
    let mut order = Vec::new();
    for item in toolbar.items().iter() {
        let id = item.itemIdentifier().to_string();
        if id.contains("Space") {
            continue;
        }
        if id == "books.leading" {
            order.extend(TOOLBAR_ORDER[..3].iter().map(|item| (*item).to_string()));
            continue;
        }
        order.push(id);
    }
    order
}

struct ChromeCopy {
    contents: &'static str,
    bookmarks: &'static str,
    search: &'static str,
    add: &'static str,
    notes: &'static str,
    notes_body: &'static str,
    previous: &'static str,
    next: &'static str,
}

fn chrome_copy() -> ChromeCopy {
    if prefers_zh() {
        ChromeCopy {
            contents: "目录",
            bookmarks: "书签",
            search: "搜索",
            add: "添加书签",
            notes: "笔记",
            notes_body: "划线和批注会显示在这里。提及 @alan 还不在这个版本里。",
            previous: "上一页",
            next: "下一页",
        }
    } else {
        ChromeCopy {
            contents: "Table of Contents",
            bookmarks: "Bookmarks",
            search: "Search",
            add: "Add Bookmark",
            notes: "Notes",
            notes_body: "Highlights and comments will show up here. Mentioning @alan is not in this build.",
            previous: "Previous Page",
            next: "Next Page",
        }
    }
}

fn prefers_zh() -> bool {
    let languages = objc2_foundation::NSLocale::preferredLanguages();
    languages
        .iter()
        .next()
        .map(|lang| lang.to_string().to_lowercase().starts_with("zh"))
        .unwrap_or(false)
}

fn symbol(name: &str, label: &str, _mtm: MainThreadMarker) -> Retained<NSImage> {
    let image = NSImage::imageWithSystemSymbolName_accessibilityDescription(
        &NSString::from_str(name),
        Some(&NSString::from_str(label)),
    );
    image.unwrap_or_else(|| {
        NSImage::imageWithSystemSymbolName_accessibilityDescription(
            &NSString::from_str("circle"),
            Some(&NSString::from_str(label)),
        )
        .expect("sf symbol")
    })
}

fn stamp(item: &NSToolbarItem, label: &str, target: &AnyObject) {
    let text = NSString::from_str(label);
    item.setLabel(&text);
    item.setPaletteLabel(&text);
    item.setToolTip(Some(&text));
    item.setBordered(true);
    unsafe {
        item.setTarget(Some(target));
        item.setAction(Some(sel!(chromeItem:)));
        let _: () = msg_send![&*item, setAccessibilityLabel: &*text];
    }
}

pub fn toolbar_item(
    identifier: &NSString,
    target: &AnyObject,
    mtm: MainThreadMarker,
) -> Option<Retained<NSToolbarItem>> {
    let id = identifier.to_string();
    let copy = chrome_copy();
    if id == "books.leading" {
        let images = NSArray::from_retained_slice(&[
            symbol("list.bullet", copy.contents, mtm),
            symbol("bookmark", copy.bookmarks, mtm),
            symbol("magnifyingglass", copy.search, mtm),
        ]);
        let group = unsafe {
            NSToolbarItemGroup::groupWithItemIdentifier_images_selectionMode_labels_target_action(
                identifier,
                &images,
                NSToolbarItemGroupSelectionMode::Momentary,
                None,
                Some(target),
                Some(sel!(chromeItem:)),
                mtm,
            )
        };
        group.setControlRepresentation(NSToolbarItemGroupControlRepresentation::Expanded);
        let labels = [copy.contents, copy.bookmarks, copy.search];
        for (item, label) in group.subitems().iter().zip(labels) {
            stamp(&item, label, target);
        }
        return Some(Retained::into_super(group));
    }
    let (symbol_name, label) = match id.as_str() {
        "books.add-bookmark" => ("bookmark.fill", copy.add),
        "books.notes" => ("note.text", copy.notes),
        _ => return None,
    };
    let item = NSToolbarItem::initWithItemIdentifier(NSToolbarItem::alloc(mtm), identifier);
    item.setImage(Some(&symbol(symbol_name, label, mtm)));
    stamp(&item, label, target);
    Some(item)
}

pub fn default_item_identifiers() -> Retained<NSArray<NSString>> {
    let leading = NSString::from_str("books.leading");
    let add = NSString::from_str("books.add-bookmark");
    let notes = NSString::from_str("books.notes");
    NSArray::from_slice(&[
        &*leading,
        unsafe { &*NSToolbarFlexibleSpaceItemIdentifier },
        &*add,
        &*notes,
    ])
}

fn host_target(host: &NSView) -> &AnyObject {
    unsafe { &*std::ptr::from_ref(host).cast::<AnyObject>() }
}

fn place_chevrons(host: &NSView, rt: &Runtime, page: PageBox, alpha: f64) {
    let mtm = MainThreadMarker::from(host);
    let copy = chrome_copy();
    let target = host_target(host);
    if rt.previous.borrow().is_none() {
        let button = chevron("chevron.left", copy.previous, -1, target, mtm);
        host.addSubview(&button);
        *rt.previous.borrow_mut() = Some(button);
    }
    if rt.next.borrow().is_none() {
        let button = chevron("chevron.right", copy.next, 1, target, mtm);
        host.addSubview(&button);
        *rt.next.borrow_mut() = Some(button);
    }
    let bounds = host.bounds();
    let y = (bounds.size.height - 52.0) / 2.0;
    let left_x = ((page.x - 28.0) / 2.0).max(8.0);
    let right_margin = bounds.size.width - (page.x + page.w);
    let right_x = page.x + page.w + ((right_margin - 28.0) / 2.0).max(8.0);
    place_button(host, rt.previous.borrow().as_ref(), left_x, y, alpha);
    place_button(host, rt.next.borrow().as_ref(), right_x, y, alpha);
}

fn chevron(
    name: &str,
    label: &str,
    tag: isize,
    target: &AnyObject,
    mtm: MainThreadMarker,
) -> Retained<NSButton> {
    let image = symbol(name, label, mtm);
    let button = unsafe {
        NSButton::buttonWithImage_target_action(&image, Some(target), Some(sel!(turnPage:)), mtm)
    };
    button.setBordered(false);
    button.setContentTintColor(Some(&NSColor::secondaryLabelColor()));
    let text = NSString::from_str(label);
    unsafe {
        let _: () = msg_send![&*button, setTag: tag];
        let _: () = msg_send![&*button, setToolTip: &*text];
        let _: () = msg_send![&*button, setAccessibilityLabel: &*text];
    }
    button
}

fn place_button(host: &NSView, button: Option<&Retained<NSButton>>, x: f64, y: f64, alpha: f64) {
    let Some(button) = button else {
        return;
    };
    unsafe {
        let _: () = msg_send![
            host,
            addSubview: &**button,
            positioned: 1isize,
            relativeTo: std::ptr::null::<NSView>()
        ];
        let _: () = msg_send![
            &**button,
            setFrame: NSRect::new(NSPoint::new(x, y), NSSize::new(28.0, 52.0))
        ];
        let _: () = msg_send![&**button, setAlphaValue: alpha];
        let _: () = msg_send![&**button, setHidden: alpha < 0.02];
    }
}

pub fn present_placeholder(window: &NSWindow, rt: &Runtime, identifier: &str) {
    let copy = chrome_copy();
    let (title, body) = match identifier {
        "books.contents" | "0" => (
            copy.contents.to_string(),
            format!("{}.", missing(copy.contents)),
        ),
        "books.bookmarks" | "1" => (
            copy.bookmarks.to_string(),
            format!("{}.", missing(copy.bookmarks)),
        ),
        "books.search" | "2" => (copy.search.to_string(), format!("{}.", missing(copy.search))),
        "books.add-bookmark" => (copy.add.to_string(), format!("{}.", missing(copy.add))),
        "books.notes" => (copy.notes.to_string(), copy.notes_body.to_string()),
        _ => return,
    };
    let mtm = MainThreadMarker::from(&*window);
    if rt.panel.borrow().is_none() {
        let panel = unsafe {
            NSWindow::initWithContentRect_styleMask_backing_defer(
                NSWindow::alloc(mtm),
                NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(320.0, 140.0)),
                NSWindowStyleMask::Titled | NSWindowStyleMask::Closable | NSWindowStyleMask::UtilityWindow,
                NSBackingStoreType::Buffered,
                false,
            )
        };
        unsafe {
            panel.setReleasedWhenClosed(false);
        }
        let field = NSTextField::labelWithString(&NSString::from_str(""), mtm);
        field.setFont(Some(&objc2_app_kit::NSFont::systemFontOfSize(13.0)));
        field.setTextColor(Some(&NSColor::secondaryLabelColor()));
        unsafe {
            let _: () = msg_send![
                &*field,
                setFrame: NSRect::new(NSPoint::new(20.0, 36.0), NSSize::new(280.0, 64.0))
            ];
        }
        if let Some(content) = panel.contentView() {
            content.addSubview(&field);
        }
        *rt.panel_body.borrow_mut() = Some(field);
        *rt.panel.borrow_mut() = Some(panel);
    }
    let panel = rt.panel.borrow();
    let Some(panel) = panel.as_ref() else {
        return;
    };
    panel.setTitle(&NSString::from_str(&title));
    if let Some(field) = rt.panel_body.borrow().as_ref() {
        unsafe {
            let value = NSString::from_str(&body);
            let _: () = msg_send![&**field, setStringValue: &*value];
        }
    }
    panel.center();
    panel.orderFront(None);
}

fn missing(label: &str) -> String {
    if prefers_zh() {
        format!("{label}还不在这个版本里")
    } else {
        format!("{label} is not in this build")
    }
}

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
        "side": side_inset(frame.size.width),
        "side_ratio": crate::reading::TEXT_SIDE_RATIO,
        "fullscreen": fullscreen,
        "lights": rt.lights.get(),
        "chevrons": rt.chevron_alpha.get(),
        "scrub": rt.scrub_alpha.get(),
        "scrub_page": rt.scrub_page.get(),
        "scrub_pages": rt.scrub_pages.get(),
        "tick_count": rt.ticks.borrow().len(),
        "jump_back": rt.jump_origin.get() > 0,
        "items": toolbar_order(window),
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

fn scrub_target(window: &NSWindow, rt: &Runtime) -> f64 {
    if rt.tracking.get() || near_bottom(window) {
        1.0
    } else {
        0.0
    }
}

fn near_bottom(window: &NSWindow) -> bool {
    let mouse = objc2_app_kit::NSEvent::mouseLocation();
    let frame = window.frame();
    mouse.x >= frame.origin.x
        && mouse.x <= frame.origin.x + frame.size.width
    && mouse.y >= frame.origin.y
    && mouse.y <= frame.origin.y + 64.0
}

fn place_scrub(host: &NSView, rt: &Runtime, alpha: f64) {
    let mtm = MainThreadMarker::from(host);
    let target = host_target(host);
    if rt.scrub.borrow().is_none() {
        let slider = unsafe {
            NSSlider::sliderWithTarget_action(Some(target), Some(sel!(scrub:)), mtm)
        };
        slider.setMinValue(1.0);
        slider.setMaxValue(1.0);
        slider.setContinuous(true);
        let label = NSString::from_str(if prefers_zh() {
            "页码选取器"
        } else {
            "Page picker"
        });
        unsafe {
            let _: () = msg_send![&*slider, setAccessibilityLabel: &*label];
        }
        lift(host, &slider);
        *rt.scrub.borrow_mut() = Some(slider);
    }
    let pages = rt.scrub_pages.get().max(1) as f64;
    let page = rt.scrub_page.get().clamp(1, pages as u32) as f64;
    if let Some(slider) = rt.scrub.borrow().as_ref() {
        slider.setMaxValue(pages);
        if !rt.tracking.get() {
            slider.setDoubleValue(page);
        }
        let bounds = host.bounds();
        let y = (bounds.size.height - 24.0).max(0.0);
        place_view(host, slider, 8.0, y, (bounds.size.width - 16.0).max(40.0), 16.0, alpha);
    }
    place_ticks(host, rt, alpha);
    place_preview(host, rt, alpha);
    place_return(host, rt, alpha);
}

fn place_ticks(host: &NSView, rt: &Runtime, alpha: f64) {
    let ticks = rt.ticks.borrow().clone();
    let pages = rt.scrub_pages.get().max(1);
    let mut views = rt.tick_views.borrow_mut();
    let mtm = MainThreadMarker::from(host);
    while views.len() < ticks.len() {
        let tick = NSBox::initWithFrame(
            NSBox::alloc(mtm),
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(2.0, 8.0)),
        );
        tick.setBoxType(NSBoxType::Custom);
        tick.setTitlePosition(NSTitlePosition::NoTitle);
        tick.setBorderWidth(0.0);
        tick.setFillColor(&NSColor::secondaryLabelColor());
        lift(host, &tick);
        views.push(tick);
    }
    let bounds = host.bounds();
    let track = (bounds.size.width - 16.0).max(40.0);
    let y = bounds.size.height - 20.0;
    for (index, tick) in views.iter().enumerate() {
        let show = alpha >= 0.02 && index < ticks.len();
        let fraction = if pages <= 1 {
            0.0
        } else {
            f64::from(ticks.get(index).map(|(page, _)| page.saturating_sub(1)).unwrap_or(0))
                / f64::from(pages - 1)
        };
        let x = 8.0 + fraction * track;
        place_view(host, tick, x, y, 2.0, 8.0, if show { alpha } else { 0.0 });
    }
}

fn place_preview(host: &NSView, rt: &Runtime, alpha: f64) {
    let mtm = MainThreadMarker::from(host);
    if rt.preview.borrow().is_none() {
        let field = NSTextField::labelWithString(&NSString::from_str(""), mtm);
        field.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
            12.0,
            unsafe { NSFontWeightSemibold },
        )));
        field.setTextColor(Some(&NSColor::labelColor()));
        lift(host, &field);
        *rt.preview.borrow_mut() = Some(field);
    }
    let Some(field) = rt.preview.borrow().clone() else {
        return;
    };
    let shown = rt.tracking.get() && alpha >= 0.02;
    if shown {
        let page = rt.scrub_page.get();
        let pages = rt.scrub_pages.get().max(1);
        let title = chapter_title(rt, page);
        let (num, _) = crate::reading::footer_zh(page, pages);
        let text = if title.is_empty() {
            num
        } else {
            format!("{title}  {num}")
        };
        unsafe {
            let value = NSString::from_str(&text);
            let _: () = msg_send![&*field, setStringValue: &*value];
            let _: () = msg_send![&*field, setAlignment: objc2_app_kit::NSTextAlignment(1)];
        }
    }
    let bounds = host.bounds();
    let width = (bounds.size.width - 80.0).max(40.0);
    let x = (bounds.size.width - width) / 2.0;
    place_view(
        host,
        &field,
        x,
        bounds.size.height - 48.0,
        width,
        18.0,
        if shown { 1.0 } else { 0.0 },
    );
}

fn chapter_title(rt: &Runtime, page: u32) -> String {
    let ticks = rt.ticks.borrow();
    let mut title = String::new();
    for (at, name) in ticks.iter() {
        if *at <= page {
            title = name.clone();
        }
    }
    title
}

fn place_return(host: &NSView, rt: &Runtime, alpha: f64) {
    let mtm = MainThreadMarker::from(host);
    let copy = jump_copy();
    let target = host_target(host);
    if rt.back_previous.borrow().is_none() {
        let button = chevron("arrow.uturn.backward", copy.previous, -1, target, mtm);
        unsafe {
            button.setAction(Some(sel!(jumpBack:)));
        }
        lift(host, &button);
        *rt.back_previous.borrow_mut() = Some(button);
    }
    if rt.back_origin.borrow().is_none() {
        let button = chevron("arrow.uturn.forward", copy.origin, 1, target, mtm);
        unsafe {
            button.setAction(Some(sel!(jumpBack:)));
        }
        lift(host, &button);
        *rt.back_origin.borrow_mut() = Some(button);
    }
    let bounds = host.bounds();
    let y = (bounds.size.height - 52.0).max(0.0);
    let live = alpha >= 0.02 && rt.jump_origin.get() > 0;
    let shown = if live { alpha } else { 0.0 };
    if let Some(button) = rt.back_previous.borrow().as_ref() {
        place_view(host, button, 16.0, y, 28.0, 22.0, shown);
    }
    if let Some(button) = rt.back_origin.borrow().as_ref() {
        place_view(
            host,
            button,
            (bounds.size.width - 44.0).max(16.0),
            y,
            28.0,
            22.0,
            shown,
        );
    }
}

struct JumpCopy {
    previous: &'static str,
    origin: &'static str,
}

fn jump_copy() -> JumpCopy {
    if prefers_zh() {
        JumpCopy {
            previous: "上次查看的页面",
            origin: "返回开始的页面",
        }
    } else {
        JumpCopy {
            previous: "Page you last viewed",
            origin: "Page you started on",
        }
    }
}

pub fn scrub_released(event_type: NSEventType) -> bool {
    event_type == NSEventType::LeftMouseUp
}

pub fn place_pdf_footer(
    host: &NSView,
    rt: &Runtime,
    band: f64,
    page_x: f64,
    page_w: f64,
    num: &str,
    rest: &str,
    visible: bool,
) {
    let mtm = MainThreadMarker::from(host);
    if rt.pdf_num.borrow().is_none() {
        let field = footer_field(mtm);
        lift(host, &field);
        *rt.pdf_num.borrow_mut() = Some(field);
    }
    if rt.pdf_rest.borrow().is_none() {
        let field = footer_field(mtm);
        lift(host, &field);
        *rt.pdf_rest.borrow_mut() = Some(field);
    }
    let bounds = host.bounds();
    let y = (bounds.size.height - band).max(0.0);
    let height = band.max(18.0);
    let right = (page_x + page_w).min(bounds.size.width);
    let alpha = if visible { 1.0 } else { 0.0 };
    if let Some(field) = rt.pdf_num.borrow().as_ref() {
        set_label(field, num);
        unsafe {
            let _: () = msg_send![&**field, setAlignment: objc2_app_kit::NSTextAlignment(1)];
        }
        place_view(host, field, page_x, y, (right - page_x).max(40.0), height, alpha);
    }
    if let Some(field) = rt.pdf_rest.borrow().as_ref() {
        set_label(field, rest);
        unsafe {
            let _: () = msg_send![&**field, setAlignment: objc2_app_kit::NSTextAlignment(2)];
        }
        let width = 180.0_f64.min((right - page_x).max(40.0));
        place_view(host, field, (right - width).max(0.0), y, width, height, alpha);
    }
}

fn footer_field(mtm: MainThreadMarker) -> Retained<NSTextField> {
    let field = NSTextField::labelWithString(&NSString::from_str(""), mtm);
    field.setFont(Some(&NSFont::monospacedDigitSystemFontOfSize_weight(
        11.0,
        unsafe { NSFontWeightRegular },
    )));
    field.setTextColor(Some(&NSColor::colorWithRed_green_blue_alpha(
        60.0 / 255.0,
        60.0 / 255.0,
        67.0 / 255.0,
        0.72,
    )));
    field
}

fn set_label(field: &NSTextField, text: &str) {
    let value = NSString::from_str(text);
    unsafe {
        let _: () = msg_send![field, setStringValue: &*value];
    }
}

fn lift(host: &NSView, view: &NSView) {
    unsafe {
        let _: () = msg_send![
            host,
            addSubview: view,
            positioned: 1isize,
            relativeTo: std::ptr::null::<NSView>()
        ];
    }
}

fn place_view(host: &NSView, view: &NSView, x: f64, y: f64, w: f64, h: f64, alpha: f64) {
    let _ = host;
    unsafe {
        let _: () = msg_send![
            view,
            setFrame: NSRect::new(NSPoint::new(x, y), NSSize::new(w, h))
        ];
        let _: () = msg_send![view, setAlphaValue: alpha];
        let _: () = msg_send![view, setHidden: alpha < 0.02];
    }
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
    fn paperback_on_a_1000_by_1200_frame_keeps_the_trim_and_the_side_ratio() {
        let frame = fit(129.0, 198.0, 1000.0, 1200.0);
        close(frame.page_w / frame.page_h, 129.0 / 198.0);
        close(frame.window_h, frame.page_h + TOP);
        let side = (frame.window_w - frame.page_w) / 2.0;
        close(side / frame.window_w, crate::reading::TEXT_SIDE_RATIO);
        assert!(frame.window_w <= 1000.0 * FIT + 0.1);
        assert!(frame.window_h <= 1200.0 * FIT + 0.1);
    }

    #[test]
    fn the_page_starts_below_the_traffic_lights() {
        let (_, _, _, top) = content_box(800.0, 1000.0);
        assert!(top >= 44.0);
    }

    #[test]
    fn a_late_fade_tick_catches_up() {
        assert!((approach(0.0, 1.0, 0.0) - 0.0).abs() < 0.001);
        assert!((approach(0.0, 1.0, 1.0) - 0.25).abs() < 0.001);
        assert!((approach(0.5, 1.0, 4.0) - 1.0).abs() < 0.001);
    }

    #[test]
    fn the_toolbar_order_is_the_books_groups() {
        assert_eq!(
            TOOLBAR_ORDER,
            [
                "books.contents",
                "books.bookmarks",
                "books.search",
                "books.add-bookmark",
                "books.notes",
            ]
        );
    }
}
