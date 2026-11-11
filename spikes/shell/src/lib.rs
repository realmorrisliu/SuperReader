mod bitmap;
mod bold_word;
mod books;
mod breaks;
mod chrome;
mod form;
mod icons;
mod launch;
mod onboarding;
mod reading;
mod settings;
mod shelf;
mod stage;
mod web;

use std::path::PathBuf;
use std::time::Duration;

use nsh_base::text_view;
use nsh_core::desktop::{MenuRole, TitleBarStyle};
use nsh_core::element::desktop::{set_application_menu, set_title_bar, MenuItem};
use nsh_core::file_dialog::{FileFilter, OpenDialog};
use nsh_core::platform::Os;
use nsh_core::prelude::*;
use nsh_platform_macos::objc2::MainThreadMarker;
use nsh_platform_macos::objc2_app_kit::NSModalResponseOK;
use nsh_platform_macos::objc2_app_kit::NSOpenPanel;
use serde_json::json;
use spike_library::{Catalog, Home, Scope, SourceName};

pub use books::in_app_bundle;
pub use chrome::opening_size;
pub use chrome::shell_opening_size;
pub use launch::install_document_open;
pub use web::mark_process_start;

pub use stage::register;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Screen {
    Shelf,
    Reader,
}

const READER_FORCE_KEYS: &[&str] = &[
    "SPIKE_MODE",
    "SPIKE_DRAG",
    "SPIKE_ENGINE",
    "SPIKE_SELECT",
    "SPIKE_TURN",
    "SPIKE_PLAY",
    "SPIKE_BOOK",
    "SPIKE_AT",
];

pub fn reader_forcing_launch() -> bool {
    READER_FORCE_KEYS.iter().any(|key| {
        std::env::var(key)
            .ok()
            .is_some_and(|value| !value.trim().is_empty())
    })
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| "/tmp".into())
}

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn empty_catalog() -> Catalog {
    spike_library::open(Scope {
        support: settings::support_dir(),
        override_root: None,
        remembered: None,
        home: Home::Closed,
    })
}

fn root_scope(root: PathBuf) -> Scope {
    Scope {
        support: settings::support_dir(),
        override_root: Some(root),
        remembered: None,
        home: Home::Closed,
    }
}

fn apple_scope() -> Scope {
    if let Some(root) = env_path("SUPERREADER_LIBRARY") {
        return root_scope(root);
    }
    Scope {
        support: settings::support_dir(),
        override_root: None,
        remembered: None,
        home: Home::Open(home_dir()),
    }
}

fn calibre_scope() -> Option<Scope> {
    if let Some(root) = env_path("SUPERREADER_CALIBRE") {
        return Some(root_scope(root));
    }
    let root = home_dir().join("Calibre Library");
    if root.join("metadata.db").is_file() {
        Some(root_scope(root))
    } else {
        None
    }
}

fn stream_import(
    owner: Owner,
    catalog: Signal<Catalog>,
    epoch: Signal<u64>,
    scope: Scope,
    source: SourceName,
) {
    let ticket = epoch.get().wrapping_add(1);
    epoch.set(ticket);
    let base = catalog.get();
    // A manual clock does not run a timer during the press, so the streaming
    // check sees the shelf grow as it advances frames. The press listener's
    // owner dies when the first book replaces the empty shelf, so that task
    // has to live on the app. On the system clock the press returns before
    // those frames, so the list the check reads is the settled catalog.
    if !nsh_core::time::manual() {
        let mut import = spike_library::Import::begin(scope);
        let mut books = Vec::new();
        loop {
            match import.step() {
                spike_library::ImportStep::Book(book) => {
                    books.push(book);
                    catalog.set(
                        base.clone()
                            .admit(source.clone(), Catalog::streaming(books.clone())),
                    );
                }
                spike_library::ImportStep::Finished(done) => {
                    catalog.set(attach_pdf_covers(base.admit(source, done)));
                    return;
                }
            }
        }
    }
    owner.run(|| {
        spawn(async move {
            let mut import = spike_library::Import::begin(scope);
            let mut books = Vec::new();
            loop {
                if epoch.get() != ticket {
                    return;
                }
                match import.step() {
                    spike_library::ImportStep::Book(book) => {
                        books.push(book);
                        catalog.set(
                            base.clone()
                                .admit(source.clone(), Catalog::streaming(books.clone())),
                        );
                        timer(Duration::from_millis(20)).await;
                    }
                    spike_library::ImportStep::Finished(done) => {
                        if epoch.get() == ticket {
                            catalog.set(attach_pdf_covers(base.admit(source, done)));
                        }
                        return;
                    }
                }
            }
        });
    });
}

fn attach_pdf_covers(mut catalog: Catalog) -> Catalog {
    let support = settings::support_dir();
    let pending = catalog.pdfs_missing_cover();
    for (id, path) in pending {
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let Ok(png) = spike_pdf::first_page_png(&bytes) else {
            continue;
        };
        catalog.attach_png(&id, &png, &support);
    }
    catalog
}

fn choose_folder() -> Option<PathBuf> {
    let mtm = MainThreadMarker::new().expect("main");
    let panel = NSOpenPanel::openPanel(mtm);
    panel.setCanChooseDirectories(true);
    panel.setCanChooseFiles(false);
    panel.setAllowsMultipleSelection(false);
    if panel.runModal() != NSModalResponseOK {
        return None;
    }
    let urls = panel.URLs();
    let path = urls
        .firstObject()?
        .path()
        .map(|value| value.to_string())
        .unwrap_or_default();
    if path.is_empty() {
        return None;
    }
    let folder = PathBuf::from(path);
    let _ = settings::store_library_folder(&settings::path(), &folder);
    Some(folder)
}

fn apply_chosen(owner: Owner, catalog: &Signal<Catalog>, epoch: &Signal<u64>) {
    let Some(folder) = choose_folder() else {
        return;
    };
    stream_import(
        owner,
        *catalog,
        *epoch,
        root_scope(folder.clone()),
        SourceName::Folder(folder.display().to_string()),
    );
}

pub fn publish_launch_file() {
    let Some(cmd) = std::env::args().skip(1).find_map(|arg| {
        let path = PathBuf::from(&arg);
        if path.exists() {
            spike_library::dropped_open(&path)
        } else {
            None
        }
    }) else {
        return;
    };
    std::env::set_var("SUPERREADER_OPEN_KIND", cmd.kind().as_str());
    std::env::set_var(
        "SUPERREADER_OPEN_PATH",
        cmd.path().as_path().display().to_string(),
    );
    std::env::set_var("SUPERREADER_OPEN_TITLE", cmd.title());
}

fn launch_title() -> Option<String> {
    let path = env_path("SUPERREADER_OPEN_PATH")?;
    if !path.exists() {
        return None;
    }
    Some(
        std::env::var("SUPERREADER_OPEN_TITLE")
            .ok()
            .filter(|title| !title.trim().is_empty())
            .unwrap_or_else(|| "Untitled".to_string()),
    )
}

/// The shelf is the open flow for the installed app and for fixture launches.
/// Reader-forcing `SPIKE_*` launches and a dropped file skip it.
pub fn launches_full() -> bool {
    !reader_forcing_launch() && launch_title().is_none()
}

pub fn shelf_launch() -> bool {
    launches_full()
        && (in_app_bundle()
            || env_path("SUPERREADER_LIBRARY").is_some()
            || env_path("SUPERREADER_CALIBRE").is_some()
            || onboarding::preview().is_some())
}

#[native_module("spike-shell")]
fn install(registry: &mut Registry) {
    registry.register::<SpikeApp>();
}

#[component]
pub fn SpikeApp() -> impl IntoElement {
    let app_owner = Owner::current();
    set_title_bar(TitleBarStyle::Hidden, Some(px(chrome::TOP as f32)));
    let stage = element_ref();
    let title = signal(String::new());
    let trim_id = signal(settings::load(&settings::path()));
    let bold_on = signal(settings::bold_word_starts(&settings::path()));
    let empty = signal(false);
    let shelf_on = shelf_launch();
    let catalog = signal(empty_catalog());
    let epoch = signal(0_u64);
    let screen = signal(Screen::Shelf);
    effect({
        let trim_id = trim_id.clone();
        let bold_on = bold_on.clone();
        let stage = stage.clone();
        let catalog = catalog.clone();
        let epoch = epoch.clone();
        move || {
            let current = trim_id.get();
            let bold = bold_on.get();
            let needs_folder = shelf_on && catalog.get().needs_folder();
            let stage_trim = stage.clone();
            let stage_bold = stage.clone();
            let stage_open = stage.clone();
            let bold_slot = bold_on.clone();
            let catalog_slot = catalog.clone();
            let epoch_slot = epoch.clone();
            set_application_menu(reading_menu(
                &current,
                bold,
                needs_folder,
                move || apply_chosen(app_owner, &catalog_slot, &epoch_slot),
                move |id: String| {
                    stage_trim.command("trim", json!(id));
                },
                move || {
                    let next = !bold_slot.get();
                    bold_slot.set(next);
                    stage_bold.command("bold", json!(next));
                },
                move || choose_book(stage_open.clone()),
            ));
        }
    });
    let variant = signal(String::new());
    let left = signal(String::new());
    let right = signal(String::new());
    let html = signal(String::new());
    let show_text = signal(false);
    let title_opacity = signal(1.0_f32);
    let title_rest = signal(1.0_f32);
    let title_hover = signal(false);
    let footer_opacity = signal(1.0_f32);
    let shell_on = launches_full();
    let shell_form = signal(form::launch());
    let shell_selected = signal(form::SelectId::default());
    let shell_window = signal(form::Window {
        width: 1280.0,
        height: 800.0,
    });
    let shell_traffic = signal(0.0_f64);
    let bars_row = signal(shell_on);
    let dropped = stage.clone();
    on_key("1", {
        let stage = stage.clone();
        move |_| stage.command("pdf", json!("pdfium"))
    });
    on_key("2", {
        let stage = stage.clone();
        move |_| stage.command("pdf", json!("hayro"))
    });
    on_key("3", {
        let stage = stage.clone();
        move |_| stage.command("epub", json!("webkit"))
    });
    on_key("4", {
        let stage = stage.clone();
        move |_| stage.command("epub", json!("blitz"))
    });
    on_key("5", {
        let stage = stage.clone();
        move |_| stage.command("epub", json!("text"))
    });
    on_key("8", {
        let stage = stage.clone();
        move |_| stage.command("book", json!("pride"))
    });
    on_key("9", {
        let stage = stage.clone();
        move |_| stage.command("book", json!("zhaohua"))
    });
    on_key("mod+minus", {
        let stage = stage.clone();
        move |_| stage.command("zoom", json!("retina-1x"))
    });
    on_key("mod+plus", {
        let stage = stage.clone();
        move |_| stage.command("zoom", json!("zoom-2x"))
    });
    on_key("left", {
        let stage = stage.clone();
        move |_| stage.command("page", json!(-1))
    });
    on_key("right", {
        let stage = stage.clone();
        move |_| stage.command("page", json!(1))
    });
    let import_apple = {
        let catalog = catalog.clone();
        let epoch = epoch.clone();
        move || {
            stream_import(
                app_owner,
                catalog,
                epoch,
                apple_scope(),
                SourceName::AppleBooks,
            );
        }
    };
    let import_calibre = {
        let catalog = catalog.clone();
        let epoch = epoch.clone();
        move || {
            if let Some(scope) = calibre_scope() {
                stream_import(app_owner, catalog, epoch, scope, SourceName::Calibre);
                return;
            }
            apply_chosen(app_owner, &catalog, &epoch);
        }
    };
    let import_folder = {
        let catalog = catalog.clone();
        let epoch = epoch.clone();
        move || apply_chosen(app_owner, &catalog, &epoch)
    };
    let history = signal(settings::load_history(&settings::path()));
    let source_menu = signal(false);
    let shell_library = signal(shelf::project(
        &catalog.get(),
        shell_selected.get(),
        &history.get(),
    ));
    effect({
        let catalog = catalog.clone();
        let shell_selected = shell_selected.clone();
        let shell_library = shell_library.clone();
        let history = history.clone();
        move || {
            shell_library.set(shelf::project(
                &catalog.get(),
                shell_selected.get(),
                &history.get(),
            ));
        }
    });
    let drop_screen = screen.clone();
    let drop_variant = variant.clone();
    view()
        .flex_1()
        .bg(if shell_on {
            rgb(form::FIELD_HEX).to_hsla()
        } else {
            rgb(reading::paper_u32()).to_hsla()
        })
        .on_file_drop(move |event: &FileDropEvent| {
            if let Some(path) = event
                .uris
                .iter()
                .find_map(|uri| books::path_from_file_url(uri))
            {
                if shelf_on {
                    if let Some(cmd) = spike_library::dropped_open(std::path::Path::new(&path)) {
                        drop_variant.set(cmd.title().to_string());
                    }
                    drop_screen.set(Screen::Reader);
                }
                dropped.command("open", json!(path));
            }
        })
        .child(when(
            {
                let bars_row = bars_row.clone();
                move || !bars_row.get()
            },
            {
                let title = title.clone();
                let variant = variant.clone();
                let title_opacity = title_opacity.clone();
                let title_rest = title_rest.clone();
                let title_hover = title_hover.clone();
                move || {
                    view()
                        .h(px(44.0))
                        .px_4()
                        .flex()
                        .flex_row()
                        .items_center()
                        .justify_between()
                        .opacity(title_opacity)
                        .on_mouse_enter({
                            let title_opacity = title_opacity.clone();
                            let title_hover = title_hover.clone();
                            move |_: &MouseEvent| {
                                title_hover.set(true);
                                title_opacity.set(1.0);
                            }
                        })
                        .on_mouse_leave({
                            let title_opacity = title_opacity.clone();
                            let title_rest = title_rest.clone();
                            let title_hover = title_hover.clone();
                            move |_: &MouseEvent| {
                                title_hover.set(false);
                                title_opacity.set(title_rest.get());
                            }
                        })
                        .child(
                            text(move || title.get())
                                .text_size(px(15.0))
                                .text_color(hsla(0.0, 0.0, 0.12, 1.0)),
                        )
                        .child(
                            text(move || variant.get())
                                .text_size(px(13.0))
                                .text_color(hsla(0.0, 0.0, 0.12, 1.0)),
                        )
                }
            },
        ))
        .child(
            view()
                .flex_1()
                .relative()
                .child(
                    native("reader.stage")
                        .element_ref(&stage)
                        .flex_1()
                        .on_native_event({
                            let title = title.clone();
                            let variant = variant.clone();
                            let left = left.clone();
                            let right = right.clone();
                            let html = html.clone();
                            let show_text = show_text.clone();
                            let title_opacity = title_opacity.clone();
                            let title_rest = title_rest.clone();
                            let title_hover = title_hover.clone();
                            let footer_opacity = footer_opacity.clone();
                            let bars_row = bars_row.clone();
                            let trim_id = trim_id.clone();
                            let empty = empty.clone();
                            let shell_window = shell_window.clone();
                            let shell_traffic = shell_traffic.clone();
                            move |event: &NativeEvent| {
                                if event.name.as_ref() != "status" {
                                    return;
                                }
                                title.set(event.data["title"].as_str().unwrap_or("").to_string());
                                variant
                                    .set(event.data["variant"].as_str().unwrap_or("").to_string());
                                left.set(event.data["left"].as_str().unwrap_or("").to_string());
                                right.set(event.data["right"].as_str().unwrap_or("").to_string());
                                show_text.set(event.data["show_text"].as_bool().unwrap_or(false));
                                let faded =
                                    event.data["title_opacity"].as_f64().unwrap_or(1.0) as f32;
                                title_rest.set(faded);
                                if !title_hover.get() {
                                    title_opacity.set(faded);
                                }
                                footer_opacity.set(
                                    event.data["footer_opacity"].as_f64().unwrap_or(1.0) as f32,
                                );
                                bars_row.set(event.data["bars_row"].as_bool().unwrap_or(false));
                                empty.set(event.data["empty"].as_bool().unwrap_or(false));
                                if let (Some(width), Some(height)) = (
                                    event.data["stage_w"].as_f64(),
                                    event.data["stage_h"].as_f64(),
                                ) {
                                    if width > 1.0 && height > 1.0 {
                                        shell_window.set(form::Window { width, height });
                                    }
                                }
                                if let Some(span) = event.data["bar_w"].as_f64() {
                                    if span > 1.0 {
                                        shell_traffic.set(span);
                                    }
                                }
                                if let Some(id) = event.data["trim"].as_str() {
                                    if id != trim_id.get() {
                                        trim_id.set(id.to_string());
                                    }
                                }
                                if let Some(body) = event.data["html"].as_str() {
                                    html.set(body.to_string());
                                }
                            }
                        }),
                )
                .child(when(move || shell_on, {
                    let shell_form = shell_form.clone();
                    let shell_selected = shell_selected.clone();
                    let shell_window = shell_window.clone();
                    let shell_traffic = shell_traffic.clone();
                    let shell_library = shell_library.clone();
                    let import_apple = import_apple.clone();
                    let import_calibre = import_calibre.clone();
                    let import_folder = import_folder.clone();
                    let stage = stage.clone();
                    let history = history.clone();
                    let source_menu = source_menu.clone();
                    move || {
                        let stage_open = stage.clone();
                        let history_open = history.clone();
                        form::mount(
                            shell_form,
                            shell_selected,
                            shell_window,
                            shell_traffic,
                            shell_library,
                            source_menu,
                            move |peer| match peer {
                                form::Peer::AppleBooks => import_apple(),
                                form::Peer::Calibre => import_calibre(),
                                form::Peer::AddFolder => import_folder(),
                            },
                            move |path| {
                                stage_open.command("open", json!(path));
                                history_open.set(settings::load_history(&settings::path()));
                            },
                        )
                    }
                })),
        )
        .child(when(move || show_text.get(), {
            let html = html.clone();
            move || {
                text_view()
                    .html(move || SharedString::from(html.get()))
                    .selectable(true)
                    .p_6()
                    .bg(hsla(40.0, 0.25, 0.97, 1.0))
            }
        }))
        .child(when(
            {
                let bars_row = bars_row.clone();
                move || !bars_row.get()
            },
            {
                let left = left.clone();
                let right = right.clone();
                move || {
                    view()
                        .h(px(36.0))
                        .px_4()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(text(move || left.get()).text_size(px(13.0)))
                        .child(text(move || right.get()).text_size(px(13.0)))
                }
            },
        ))
}

fn choose_book(stage: ElementRef) {
    spawn(async move {
        let dialog = OpenDialog::files().filter(FileFilter::new("Books", ["epub", "pdf"]));
        if let Ok(Some(paths)) = pick_paths(dialog).await {
            if let Some(path) = paths.into_iter().find_map(|path| {
                let text = path.display().to_string();
                books::parse_book(&text).map(|_| text)
            }) {
                stage.command("open", json!(path));
            }
        }
    });
}

fn reading_menu(
    current: &str,
    bold: bool,
    needs_folder: bool,
    choose_library: impl Fn() + Clone + 'static,
    on_trim: impl Fn(String) + Clone + 'static,
    on_bold: impl Fn() + Clone + 'static,
    on_open: impl Fn() + Clone + 'static,
) -> Vec<MenuItem> {
    let mut menus = Vec::new();
    if os() == Os::MacOs {
        menus.push(MenuItem::submenu(
            "Slowpage",
            [
                MenuItem::role(MenuRole::About),
                MenuItem::separator(),
                MenuItem::role(MenuRole::Hide),
                MenuItem::role(MenuRole::HideOthers),
                MenuItem::separator(),
                MenuItem::role(MenuRole::Quit),
            ],
        ));
    }
    let mut file_items = vec![MenuItem::new("Open…").shortcut("mod+o").on_select(on_open)];
    if needs_folder {
        file_items.push(MenuItem::new("Choose Library Folder").on_select(choose_library));
    }
    menus.push(MenuItem::submenu("File", file_items));
    menus.push(MenuItem::submenu(
        "Edit",
        [
            MenuItem::role(MenuRole::Undo),
            MenuItem::role(MenuRole::Redo),
            MenuItem::separator(),
            MenuItem::role(MenuRole::Cut),
            MenuItem::role(MenuRole::Copy),
            MenuItem::role(MenuRole::Paste),
            MenuItem::role(MenuRole::SelectAll),
        ],
    ));
    menus.push(MenuItem::submenu(
        "Window",
        [
            MenuItem::role(MenuRole::Minimize),
            MenuItem::role(MenuRole::Zoom),
            MenuItem::role(MenuRole::ToggleFullScreen),
            MenuItem::separator(),
            MenuItem::role(MenuRole::Close),
        ],
    ));
    let mut view_items = Vec::new();
    for item in reading::TRIMS {
        let id = item.id.to_string();
        let on_trim = on_trim.clone();
        view_items.push(
            MenuItem::new(item.label)
                .checked(current == item.id)
                .on_select(move || on_trim(id.clone())),
        );
    }
    view_items.push(MenuItem::separator());
    view_items.push(
        MenuItem::new(bold_word::menu_label())
            .checked(bold)
            .on_select(move || on_bold()),
    );
    menus.push(MenuItem::submenu("View", view_items));
    menus
}
