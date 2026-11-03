mod bitmap;
mod reading;
mod stage;
mod web;

use nsh_base::text_view;
use nsh_core::prelude::*;
use serde_json::json;

pub use stage::register;

#[native_module("spike-shell")]
fn install(registry: &mut Registry) {
    registry.register::<SpikeApp>();
}

#[component]
pub fn SpikeApp() -> impl IntoElement {
    let stage = element_ref();
    let title = signal("SuperReader spike".to_string());
    let variant = signal(String::new());
    let left = signal(String::new());
    let right = signal(String::new());
    let html = signal(String::new());
    let show_text = signal(false);
    let title_opacity = signal(1.0_f32);
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
    on_key("0", {
        let stage = stage.clone();
        move |_| stage.command("reading", json!("cycle"))
    });
    view()
        .flex_1()
        .bg(hsla(40.0, 0.2, 0.96, 1.0))
        .child(
            view()
                .h(px(44.0))
                .px_4()
                .flex()
                .flex_row()
                .items_center()
                .justify_between()
                .opacity(title_opacity)
                .child(text(move || title.get()).text_size(px(15.0)))
                .child(text(move || variant.get()).text_size(px(13.0))),
        )
        .child(
            view().flex_1().child(
                native("reader.stage").element_ref(&stage).flex_1().on_native_event({
                    let title = title.clone();
                    let variant = variant.clone();
                    let left = left.clone();
                    let right = right.clone();
                    let html = html.clone();
                    let show_text = show_text.clone();
                    let title_opacity = title_opacity.clone();
                    let footer_opacity = footer_opacity.clone();
                    let bars_row = bars_row.clone();
                    move |event: &NativeEvent| {
                        if event.name.as_ref() != "status" {
                            return;
                        }
                        title.set(event.data["title"].as_str().unwrap_or("").to_string());
                        variant.set(event.data["variant"].as_str().unwrap_or("").to_string());
                        left.set(event.data["left"].as_str().unwrap_or("").to_string());
                        right.set(event.data["right"].as_str().unwrap_or("").to_string());
                        show_text.set(event.data["show_text"].as_bool().unwrap_or(false));
                        title_opacity.set(event.data["title_opacity"].as_f64().unwrap_or(1.0) as f32);
                        footer_opacity.set(event.data["footer_opacity"].as_f64().unwrap_or(1.0) as f32);
                        bars_row.set(event.data["bars_row"].as_bool().unwrap_or(false));
                        if let Some(body) = event.data["html"].as_str() {
                            html.set(body.to_string());
                        }
                    }
                }),
            ),
        )
        .child(when(
            move || show_text.get(),
            {
                let html = html.clone();
                move || {
                    text_view()
                        .html(move || SharedString::from(html.get()))
                        .selectable(true)
                        .p_6()
                        .bg(hsla(40.0, 0.25, 0.97, 1.0))
                }
            },
        ))
        .child(when(
            {
                let bars_row = bars_row.clone();
                move || bars_row.get()
            },
            {
                let left = left.clone();
                let right = right.clone();
                let footer_opacity = footer_opacity.clone();
                move || {
                    view()
                        .h(px(36.0))
                        .px_4()
                        .flex()
                        .flex_row()
                        .items_center()
                        .opacity(footer_opacity)
                        .child(view().w(px(120.0)))
                        .child(
                            view().flex_1().flex().flex_row().justify_center().child(
                                text(move || left.get())
                                    .text_size(px(15.0))
                                    .text_color(hsla(0.0, 0.0, 0.16, 1.0)),
                            ),
                        )
                        .child(
                            text(move || right.get())
                                .text_size(px(15.0))
                                .text_color(hsla(0.0, 0.0, 0.16, 1.0)),
                        )
                }
            },
        ))
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
