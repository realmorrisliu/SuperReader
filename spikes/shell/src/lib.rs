mod bitmap;
mod breaks;
mod chrome;
mod neighbor;
mod reading;
mod settings;
mod stage;
mod web;

use nsh_base::text_view;
use nsh_core::desktop::{MenuRole, TitleBarStyle};
use nsh_core::element::desktop::{MenuItem, set_application_menu, set_title_bar};
use nsh_core::platform::Os;
use nsh_core::prelude::*;
use serde_json::json;

pub use chrome::opening_size;
pub use web::mark_process_start;

pub use stage::register;

#[native_module("spike-shell")]
fn install(registry: &mut Registry) {
    registry.register::<SpikeApp>();
}

#[component]
pub fn SpikeApp() -> impl IntoElement {
    set_title_bar(TitleBarStyle::Hidden, Some(px(chrome::TOP as f32)));
    let stage = element_ref();
    let title = signal("SuperReader spike".to_string());
    let trim_id = signal(settings::load(&settings::path()));
    effect({
        let trim_id = trim_id.clone();
        let stage = stage.clone();
        move || {
            let current = trim_id.get();
            let stage = stage.clone();
            set_application_menu(reading_menu(&current, move |id: String| {
                stage.command("trim", json!(id));
            }));
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
    let bars_row = signal(false);
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
    view()
        .flex_1()
        .bg(rgb(0xf3eee4).to_hsla())
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
            view().flex_1().child(
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
                        move |event: &NativeEvent| {
                            if event.name.as_ref() != "status" {
                                return;
                            }
                            title.set(event.data["title"].as_str().unwrap_or("").to_string());
                            variant.set(event.data["variant"].as_str().unwrap_or("").to_string());
                            left.set(event.data["left"].as_str().unwrap_or("").to_string());
                            right.set(event.data["right"].as_str().unwrap_or("").to_string());
                            show_text.set(event.data["show_text"].as_bool().unwrap_or(false));
                            let faded = event.data["title_opacity"].as_f64().unwrap_or(1.0) as f32;
                            title_rest.set(faded);
                            if !title_hover.get() {
                                title_opacity.set(faded);
                            }
                            footer_opacity
                                .set(event.data["footer_opacity"].as_f64().unwrap_or(1.0) as f32);
                            bars_row.set(event.data["bars_row"].as_bool().unwrap_or(false));
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
            ),
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

fn reading_menu(current: &str, on_trim: impl Fn(String) + Clone + 'static) -> Vec<MenuItem> {
    let mut menus = Vec::new();
    if os() == Os::MacOs {
        menus.push(MenuItem::submenu(
            "SuperReader spike",
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
    menus.push(MenuItem::submenu("View", view_items));
    menus
}
