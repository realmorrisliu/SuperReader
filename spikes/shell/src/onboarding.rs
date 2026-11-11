//! Empty shelf. Preview states stay on `SPIKE_ONBOARDING`. The live shelf
//! uses the same layout and calls the import actions.

use std::path::PathBuf;
use std::sync::OnceLock;

use nsh_core::prelude::*;

use crate::icons::{self, OnboardingGlyphs};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Preview {
    Empty,
    Importing,
    Done,
    Denied,
}

pub fn preview() -> Option<Preview> {
    match std::env::var("SPIKE_ONBOARDING")
        .ok()
        .as_deref()
        .map(str::trim)
    {
        Some("empty") => Some(Preview::Empty),
        Some("importing") => Some(Preview::Importing),
        Some("done") => Some(Preview::Done),
        Some("denied") => Some(Preview::Denied),
        _ => None,
    }
}

struct Copy {
    hint: &'static str,
    import: &'static str,
    calibre: &'static str,
    folder: &'static str,
    choose: &'static str,
    importing: &'static str,
    done: &'static str,
    store: &'static str,
    cloud: &'static str,
}

fn chinese() -> bool {
    std::env::var("SPIKE_LANG")
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default()
        .to_ascii_lowercase()
        .starts_with("zh")
}

fn copy() -> Copy {
    if chinese() {
        Copy {
            hint: "把书拖到这里打开",
            import: "从 Apple Books 导入",
            calibre: "calibre 书库",
            folder: "文件夹",
            choose: "选择文件夹",
            importing: "正在导入",
            done: "已导入 3 本，1 本商店加密不可用，1 本尚未从 iCloud 下载",
            store: "商店加密",
            cloud: "尚未下载",
        }
    } else {
        Copy {
            hint: "Drop a book to open it",
            import: "Import from Apple Books",
            calibre: "calibre library",
            folder: "Folder",
            choose: "Choose Folder",
            importing: "Importing",
            done: "Imported 3, 1 store-protected unavailable, 1 not downloaded from iCloud",
            store: "Store",
            cloud: "Not downloaded",
        }
    }
}

fn glyphs() -> Option<&'static OnboardingGlyphs> {
    static CELL: OnceLock<Option<OnboardingGlyphs>> = OnceLock::new();
    CELL.get_or_init(|| {
        let dir = crate::settings::support_dir().join("onboarding");
        icons::write_onboarding_glyphs(&dir)
    })
    .as_ref()
}

pub fn screen(mode: Preview) -> AnyElement {
    match mode {
        Preview::Empty => empty_state(false, || {}, || {}, || {}),
        Preview::Denied => empty_state(true, || {}, || {}, || {}),
        Preview::Importing => arriving(true),
        Preview::Done => arriving(false),
    }
}

pub fn live_empty(
    denied: bool,
    on_import: impl Fn() + Clone + 'static,
    on_calibre: impl Fn() + Clone + 'static,
    on_folder: impl Fn() + Clone + 'static,
) -> AnyElement {
    empty_state(denied, on_import, on_calibre, on_folder)
}

/// Quiet line for a finished import. Zero store and cloud counts drop out.
pub fn result_line(ready: usize, store: usize, cloud: usize) -> String {
    if chinese() {
        let mut parts = vec![format!("已导入 {ready} 本")];
        if store > 0 {
            parts.push(format!("{store} 本商店加密不可用"));
        }
        if cloud > 0 {
            parts.push(format!("{cloud} 本尚未从 iCloud 下载"));
        }
        return parts.join("，");
    }
    let mut parts = vec![format!("Imported {ready}")];
    if store > 0 {
        parts.push(format!("{store} store-protected unavailable"));
    }
    if cloud > 0 {
        parts.push(format!("{cloud} not downloaded from iCloud"));
    }
    parts.join(", ")
}

/// Row label. The crate's English word stays the source of the count.
pub fn shelf_label(english: &str) -> String {
    if !chinese() {
        return english.to_string();
    }
    match english {
        "Not downloaded" => "尚未下载".to_string(),
        "Store" => "商店加密".to_string(),
        "Audiobook" => "有声书".to_string(),
        "Missing" => "找不到".to_string(),
        "Unsupported" => "不支持".to_string(),
        "Not readable" => "无法阅读".to_string(),
        other => other.to_string(),
    }
}

fn empty_state(
    denied: bool,
    on_import: impl Fn() + 'static,
    on_calibre: impl Fn() + 'static,
    on_folder: impl Fn() + Clone + 'static,
) -> AnyElement {
    let copy = copy();
    let glyphs = glyphs();
    let primary_label = if denied { copy.choose } else { copy.import };
    let primary_icon = glyphs.and_then(|glyphs| {
        if denied {
            Some(glyphs.folder.clone())
        } else {
            Some(glyphs.book.clone())
        }
    });
    let mut column = view()
        .flex_1()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(20.0))
        .child(
            text(copy.hint.to_string())
                .text_size(px(13.0))
                .text_color(quiet()),
        )
        .child(if denied {
            action(primary_label, primary_icon, true, on_folder.clone())
        } else {
            action(primary_label, primary_icon, true, on_import)
        });
    let mut second = view().flex_row().items_center().gap(px(28.0));
    second = second.child(action(
        copy.calibre,
        glyphs.map(|glyphs| glyphs.library.clone()),
        false,
        on_calibre,
    ));
    if !denied {
        second = second.child(action(
            copy.folder,
            glyphs.map(|glyphs| glyphs.folder.clone()),
            false,
            on_folder,
        ));
    }
    column = column.child(second);
    column.into_any()
}

fn arriving(importing: bool) -> AnyElement {
    let copy = copy();
    let line = if importing { copy.importing } else { copy.done };
    let mut rows: Vec<AnyElement> = Vec::new();
    rows.push(proto_row("Harbor Light", None, false));
    rows.push(proto_row("Field Guide", None, false));
    if !importing {
        rows.push(proto_row("Folder Book", None, false));
        rows.push(proto_row("Store Sample", Some(copy.store), true));
        rows.push(proto_row("Cloud Copy", Some(copy.cloud), true));
    }
    view()
        .flex_1()
        .flex_col()
        .p_4()
        .gap_3()
        .child(
            text(line.to_string())
                .text_size(px(13.0))
                .text_color(quiet()),
        )
        .child(view().flex_col().gap_1().children(rows))
        .into_any()
}

fn proto_row(title: &str, label: Option<&str>, grey: bool) -> AnyElement {
    let title_color = if grey { quiet() } else { ink() };
    let mut column = view().flex_col().gap_1().px_2().py_2();
    column = column.child(
        text(title.to_string())
            .text_size(px(15.0))
            .text_color(title_color),
    );
    if let Some(label) = label {
        column = column.child(
            text(label.to_string())
                .text_size(px(12.0))
                .text_color(quiet()),
        );
    }
    column.into_any()
}

fn action(
    label: &str,
    icon: Option<PathBuf>,
    primary: bool,
    on_press: impl Fn() + 'static,
) -> AnyElement {
    let mut row = view()
        .flex_row()
        .items_center()
        .justify_center()
        .gap_2()
        .px(px(16.0))
        .py(px(8.0))
        .rounded(px(8.0));
    if primary {
        row = row
            .bg(hsla(0.0, 0.0, 0.96, 1.0))
            .border_1()
            .border_color(ink());
    }
    if let Some(path) = icon {
        row = row.child(image(format!("file://{}", path.display())).size(px(16.0)));
    }
    row = row.on_press(move |_| on_press());
    let color = if primary { ink() } else { quiet() };
    row.child(
        text(label.to_string())
            .text_size(px(15.0))
            .text_color(color),
    )
    .into_any()
}

fn ink() -> Hsla {
    hsla(0.0, 0.0, 0.12, 1.0)
}

fn quiet() -> Hsla {
    hsla(0.0, 0.0, 0.45, 1.0)
}
