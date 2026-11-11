//! The shell is one form. `step` is the only writer. `place` is the geometry.
//! `mount` paints that geometry and does not invent a second one.

use std::path::PathBuf;

use nsh_core::prelude::*;

const SIDEBAR_W: f64 = 220.0;
const CHAT_W: f64 = 320.0;
const INSET: f64 = 6.0;
const RADIUS: f64 = 12.0;

pub const FIELD_HEX: u32 = 0x00EB_EBEE;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Presence {
    Shown,
    Hidden,
}

impl Presence {
    fn flip(self) -> Self {
        match self {
            Presence::Shown => Presence::Hidden,
            Presence::Hidden => Presence::Shown,
        }
    }

    fn shown(self) -> bool {
        self == Presence::Shown
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Form {
    Full { sidebar: Presence, chat: Presence },
    PageAndPanel { sidebar: Presence, margin: Presence },
    Immersive,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Command {
    ToggleSidebar,
    TogglePanel,
}

pub fn launch() -> Form {
    Form::Full {
        sidebar: Presence::Shown,
        chat: Presence::Shown,
    }
}

pub fn step(form: Form, command: Command) -> Form {
    match (form, command) {
        (Form::Immersive, _) => Form::Immersive,
        (Form::Full { sidebar, chat }, Command::ToggleSidebar) => Form::Full {
            sidebar: sidebar.flip(),
            chat,
        },
        (Form::Full { sidebar, chat }, Command::TogglePanel) => Form::Full {
            sidebar,
            chat: chat.flip(),
        },
        (Form::PageAndPanel { sidebar, margin }, Command::ToggleSidebar) => Form::PageAndPanel {
            sidebar: sidebar.flip(),
            margin,
        },
        (Form::PageAndPanel { sidebar, margin }, Command::TogglePanel) => Form::PageAndPanel {
            sidebar,
            margin: margin.flip(),
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Window {
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LibraryCounts {
    pub all: u32,
    pub reading: u32,
    pub want: u32,
    pub done: u32,
    pub pdf: u32,
    pub notes: u32,
}

impl Default for LibraryCounts {
    fn default() -> Self {
        Self {
            all: 0,
            reading: 0,
            want: 0,
            done: 0,
            pdf: 0,
            notes: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceIn {
    pub label: String,
    pub count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoverCard {
    pub title: String,
    pub author: String,
    pub cover: Option<PathBuf>,
    pub open_path: Option<String>,
    pub pdf: bool,
    pub percent: Option<u8>,
    pub page: Option<u32>,
    pub unavailable: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LibraryIn {
    pub empty: bool,
    pub book_count: u32,
    pub counts: LibraryCounts,
    pub sources: Vec<SourceIn>,
    pub selected: SelectId,
    pub continue_reading: Vec<CoverCard>,
    pub covers: Vec<CoverCard>,
    pub notice: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SelectId {
    AllBooks,
    Reading,
    Want,
    Done,
    Pdf,
    Notes,
    AddSource,
    Settings,
    Source(String),
}

impl Default for SelectId {
    fn default() -> Self {
        Self::AllBooks
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlaceIn {
    pub form: Form,
    pub window: Window,
    pub traffic: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Placement {
    pub gray: bool,
    pub sidebar: Option<Rect>,
    pub card: CardBox,
    pub chat: Option<Rect>,
    pub traffic: f64,
    pub face: CardFace,
    pub rows: Vec<SideRow>,
    pub groups: Vec<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CardBox {
    Placed {
        rect: Rect,
        radius: f64,
        shadow: bool,
    },
    AwaitingMargin {
        left: f64,
        top: f64,
        bottom: f64,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardFace {
    EmptyPeers,
    Grid,
    Placeholder,
    Page,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SideRow {
    pub id: SelectId,
    pub label: String,
    pub count: Option<u32>,
    pub icon: &'static str,
    pub selected: bool,
    pub group: &'static str,
}

pub fn place(input: &PlaceIn, library: &LibraryIn) -> Placement {
    let window = input.window;
    let (sidebar_w, chat_w, inset, radius, shadow, gray) = match input.form {
        Form::Full { sidebar, chat } => (
            if sidebar.shown() { SIDEBAR_W } else { 0.0 },
            if chat.shown() { CHAT_W } else { 0.0 },
            INSET,
            RADIUS,
            true,
            true,
        ),
        Form::PageAndPanel { sidebar, margin } => {
            if margin.shown() {
                let left = if sidebar.shown() { SIDEBAR_W } else { 0.0 };
                return Placement {
                    gray: true,
                    sidebar: sidebar_rect(sidebar.shown(), window),
                    card: CardBox::AwaitingMargin {
                        left: left + INSET,
                        top: INSET,
                        bottom: INSET,
                    },
                    chat: None,
                    traffic: input.traffic,
                    face: CardFace::Page,
                    rows: rows(library),
                    groups: groups(),
                };
            }
            (
                if sidebar.shown() { SIDEBAR_W } else { 0.0 },
                0.0,
                INSET,
                RADIUS,
                true,
                true,
            )
        }
        Form::Immersive => (0.0, 0.0, 0.0, 0.0, false, false),
    };
    let rect = Rect {
        x: sidebar_w + inset,
        y: inset,
        w: (window.width - sidebar_w - chat_w - inset * 2.0).max(1.0),
        h: (window.height - inset * 2.0).max(1.0),
    };
    let face = match input.form {
        Form::Full { .. } => {
            if matches!(library.selected, SelectId::Settings) {
                CardFace::Placeholder
            } else if library.empty {
                CardFace::EmptyPeers
            } else {
                CardFace::Grid
            }
        }
        _ => CardFace::Page,
    };
    Placement {
        gray,
        sidebar: sidebar_rect(sidebar_w > 0.0, window),
        card: CardBox::Placed {
            rect,
            radius,
            shadow,
        },
        chat: if chat_w > 0.0 {
            Some(Rect {
                x: window.width - chat_w,
                y: 0.0,
                w: chat_w,
                h: window.height,
            })
        } else {
            None
        },
        traffic: input.traffic,
        face,
        rows: rows(library),
        groups: groups(),
    }
}

fn sidebar_rect(shown: bool, window: Window) -> Option<Rect> {
    if !shown {
        return None;
    }
    Some(Rect {
        x: 0.0,
        y: 0.0,
        w: SIDEBAR_W,
        h: window.height,
    })
}

fn groups() -> Vec<&'static str> {
    vec!["书库", "笔记", "书单", "来源"]
}

fn rows(library: &LibraryIn) -> Vec<SideRow> {
    let c = &library.counts;
    let mut out = vec![
        row(
            SelectId::AllBooks,
            "全部书籍",
            Some(c.all),
            "library",
            "书库",
            &library.selected,
        ),
        row(
            SelectId::Reading,
            "正在读",
            Some(c.reading),
            "book-open",
            "书库",
            &library.selected,
        ),
        row(
            SelectId::Want,
            "想读",
            Some(c.want),
            "bookmark",
            "书库",
            &library.selected,
        ),
        row(
            SelectId::Done,
            "已读完",
            Some(c.done),
            "check",
            "书库",
            &library.selected,
        ),
        row(
            SelectId::Pdf,
            "PDF",
            Some(c.pdf),
            "file",
            "书库",
            &library.selected,
        ),
        row(
            SelectId::Notes,
            "高亮与笔记",
            Some(c.notes),
            "note",
            "笔记",
            &library.selected,
        ),
    ];
    for source in &library.sources {
        out.push(row(
            SelectId::Source(source.label.clone()),
            &source.label,
            Some(source.count),
            source_icon(&source.label),
            "来源",
            &library.selected,
        ));
    }
    out.push(row(
        SelectId::AddSource,
        "添加来源",
        None,
        "plus",
        "来源",
        &library.selected,
    ));
    out.push(row(
        SelectId::Settings,
        "设置",
        None,
        "settings",
        "",
        &library.selected,
    ));
    out
}

fn source_icon(label: &str) -> &'static str {
    match label {
        "Apple Books" => "book",
        "calibre" => "library",
        _ => "folder",
    }
}

/// A connected source keeps its count. A denied import drops that label.
pub fn remember_source(rows: &mut Vec<SourceIn>, label: &str, count: u32, connected: bool) {
    if !connected {
        rows.retain(|row| row.label != label);
        return;
    }
    if let Some(row) = rows.iter_mut().find(|row| row.label == label) {
        row.count = count;
        return;
    }
    rows.push(SourceIn {
        label: label.to_string(),
        count,
    });
}

fn row(
    id: SelectId,
    label: &str,
    count: Option<u32>,
    icon: &'static str,
    group: &'static str,
    selected: &SelectId,
) -> SideRow {
    SideRow {
        selected: &id == selected,
        id,
        label: label.to_string(),
        count,
        icon,
        group,
    }
}

pub fn selected_title(id: &SelectId) -> String {
    match id {
        SelectId::AllBooks => "全部书籍".into(),
        SelectId::Reading => "正在读".into(),
        SelectId::Want => "想读".into(),
        SelectId::Done => "已读完".into(),
        SelectId::Pdf => "PDF".into(),
        SelectId::Notes => "高亮与笔记".into(),
        SelectId::AddSource => "添加来源".into(),
        SelectId::Settings => "设置".into(),
        SelectId::Source(label) => label.clone(),
    }
}

fn ink() -> Hsla {
    hsla(0.0, 0.0, 0.11, 1.0)
}

fn quiet() -> Hsla {
    hsla(0.0, 0.0, 0.48, 1.0)
}

fn field() -> Hsla {
    rgb(FIELD_HEX).to_hsla()
}

pub fn mount(
    form: Signal<Form>,
    selected: Signal<SelectId>,
    window: Signal<Window>,
    traffic: Signal<f64>,
    library: Signal<LibraryIn>,
    menu: Signal<bool>,
    on_peer: impl Fn(Peer) + Clone + 'static,
    on_open: impl Fn(String) + Clone + 'static,
) -> impl IntoElement {
    let form_b = form;
    let selected_b = selected;
    let window_b = window;
    let traffic_b = traffic;
    let library_b = library;
    let menu_b = menu;
    let on_peer_b = on_peer;
    let on_open_b = on_open;
    dynamic(move || {
        let current = form_b.get();
        let library = library_b.get();
        let placed = place(
            &PlaceIn {
                form: current,
                window: window_b.get(),
                traffic: traffic_b.get(),
            },
            &library,
        );
        write_layout(&placed);
        paint(
            placed,
            form_b,
            selected_b,
            menu_b,
            library,
            on_peer_b.clone(),
            on_open_b.clone(),
        )
    })
}

fn write_layout(placed: &Placement) {
    let card = match placed.card {
        CardBox::Placed { rect, .. } => {
            serde_json::json!([rect.x, rect.y, rect.w, rect.h])
        }
        CardBox::AwaitingMargin { .. } => serde_json::json!(null),
    };
    let sidebar = placed.sidebar.map(|rect| rect.w).unwrap_or(0.0);
    let chat = placed.chat.map(|rect| rect.w).unwrap_or(0.0);
    let body = serde_json::json!({
        "sidebar_w": sidebar,
        "chat_w": chat,
        "card": card,
        "gray": placed.gray,
    });
    let path = std::path::Path::new("/tmp/superreader-spike/shell-layout.json");
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, body.to_string());
}

fn paint(
    placed: Placement,
    form: Signal<Form>,
    selected: Signal<SelectId>,
    menu: Signal<bool>,
    library: LibraryIn,
    on_peer: impl Fn(Peer) + Clone + 'static,
    on_open: impl Fn(String) + Clone + 'static,
) -> AnyElement {
    let mut root = view().absolute().inset(px(0.0));
    if placed.gray {
        root = root.bg(field());
    }
    if let CardBox::Placed {
        rect,
        radius,
        shadow,
    } = placed.card
    {
        if shadow {
            root = root.child(
                view()
                    .absolute()
                    .left(px(rect.x as f32))
                    .top(px(rect.y as f32))
                    .w(px(rect.w as f32))
                    .h(px(rect.h as f32))
                    .rounded(px(radius as f32))
                    .bg(rgb(0x00FF_FFFF).to_hsla())
                    .shadow_lg(),
            );
        }
    }
    if let Some(rect) = placed.sidebar {
        root = root.child(sidebar_pane(
            rect,
            &placed,
            form,
            selected,
            menu,
            on_peer.clone(),
        ));
    }
    if let Some(rect) = placed.chat {
        root = root.child(chat_pane(rect, form));
    }
    if let CardBox::Placed { rect, radius, .. } = placed.card {
        root = root.child(card_face(
            rect,
            radius,
            placed.face,
            &library,
            on_peer,
            on_open,
        ));
    }
    if placed.sidebar.is_none() && !matches!(form.get(), Form::Immersive) {
        root = root.child(sidebar_toggle_button(placed.traffic, form));
    }
    if placed.chat.is_none() && matches!(form.get(), Form::Full { .. }) {
        root = root.child(panel_toggle_button(form, true));
    }
    root.into_element()
}

fn sidebar_pane(
    rect: Rect,
    placed: &Placement,
    form: Signal<Form>,
    selected: Signal<SelectId>,
    menu: Signal<bool>,
    on_peer: impl Fn(Peer) + Clone + 'static,
) -> AnyElement {
    let mut column = view()
        .absolute()
        .left(px(rect.x as f32))
        .top(px(rect.y as f32))
        .w(px(rect.w as f32))
        .h(px(rect.h as f32))
        .flex()
        .flex_col()
        .bg(field())
        .child(sidebar_top(placed.traffic, form));
    column = column.child(search_field());
    let mut groups = view().flex_1().flex_col().px(px(10.0)).pt(px(8.0));
    for label in &placed.groups {
        groups = groups.child(group_label(label));
        if *label == "书单" {
            continue;
        }
        for row in placed.rows.iter().filter(|row| row.group == *label) {
            groups = groups.child(side_row(row.clone(), selected, menu));
        }
    }
    column = column.child(groups);
    if let Some(settings) = placed.rows.iter().find(|row| row.id == SelectId::Settings) {
        column = column.child(view().px(px(10.0)).pb(px(12.0)).child(side_row(
            settings.clone(),
            selected,
            menu,
        )));
    }
    if menu.get() {
        column = column.child(source_menu(menu, on_peer));
    }
    column.into_element()
}

fn source_menu(menu: Signal<bool>, on_peer: impl Fn(Peer) + Clone + 'static) -> AnyElement {
    let choices = [
        ("Apple Books", Peer::AppleBooks),
        ("calibre", Peer::Calibre),
        ("添加文件夹", Peer::AddFolder),
    ];
    let mut list = view()
        .absolute()
        .left(px(16.0))
        .bottom(px(52.0))
        .w(px(188.0))
        .rounded(px(12.0))
        .bg(rgb(0x00FF_FFFF).to_hsla())
        .shadow_lg()
        .p(px(6.0))
        .flex()
        .flex_col();
    for (label, peer) in choices {
        let menu = menu.clone();
        let on_peer = on_peer.clone();
        list = list.child(
            view()
                .h(px(32.0))
                .rounded(px(8.0))
                .px(px(8.0))
                .flex()
                .items_center()
                .accessible(true)
                .label(label)
                .on_press(move || {
                    menu.set(false);
                    on_peer(peer);
                })
                .child(
                    text(label.to_string())
                        .text_size(px(13.0))
                        .text_color(ink()),
                ),
        );
    }
    list.into_element()
}

fn sidebar_top(traffic: f64, form: Signal<Form>) -> AnyElement {
    let leading = if traffic > 8.0 { traffic } else { 78.0 };
    view()
        .h(px(52.0))
        .flex()
        .flex_row()
        .items_center()
        .child(view().w(px(leading as f32)).h(px(52.0)))
        .child(sidebar_toggle_button(0.0, form))
        .into_element()
}

fn sidebar_toggle_button(traffic: f64, form: Signal<Form>) -> AnyElement {
    let mut button = view()
        .w(px(28.0))
        .h(px(28.0))
        .rounded(px(6.0))
        .flex()
        .items_center()
        .justify_center()
        .accessible(true)
        .label("侧边栏")
        .on_press(move || {
            let next = step(form.get(), Command::ToggleSidebar);
            form.set(next);
        });
    if traffic > 0.0 {
        button = button.absolute().left(px(traffic as f32)).top(px(12.0));
    }
    if let Some(path) = crate::icons::glyph_file("list") {
        button = button.child(image(format!("file://{}", path.display())).size(px(16.0)));
    }
    button.into_element()
}

fn panel_toggle_button(form: Signal<Form>, floating: bool) -> AnyElement {
    let mut button = view()
        .w(px(28.0))
        .h(px(28.0))
        .rounded(px(6.0))
        .flex()
        .items_center()
        .justify_center()
        .accessible(true)
        .label("对话")
        .on_press(move || {
            form.set(step(form.get(), Command::TogglePanel));
        });
    if floating {
        button = button.absolute().right(px(16.0)).top(px(12.0));
    }
    if let Some(path) = crate::icons::glyph_file("sidebar-right") {
        button = button.child(image(format!("file://{}", path.display())).size(px(16.0)));
    }
    button.into_element()
}

fn search_field() -> AnyElement {
    view()
        .mx(px(12.0))
        .mb(px(6.0))
        .h(px(32.0))
        .rounded(px(8.0))
        .bg(rgb(0x00E3_E3E8).to_hsla())
        .px(px(8.0))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(6.0))
        .child(glyph_box("search", false))
        .child(
            text("搜索书名、作者、笔记")
                .text_size(px(13.0))
                .text_color(quiet()),
        )
        .into_element()
}

fn group_label(label: &str) -> AnyElement {
    view()
        .pt(px(14.0))
        .pb(px(4.0))
        .px(px(8.0))
        .child(
            text(label.to_string())
                .text_size(px(11.0))
                .text_color(quiet()),
        )
        .into_element()
}

fn side_row(row: SideRow, selected: Signal<SelectId>, menu: Signal<bool>) -> AnyElement {
    let id = row.id.clone();
    let mut line = view()
        .h(px(32.0))
        .rounded(px(8.0))
        .px(px(8.0))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(8.0))
        .accessible(true)
        .label(row.label.clone())
        .on_press(move || {
            if matches!(id, SelectId::AddSource) {
                menu.set(!menu.get());
                return;
            }
            menu.set(false);
            selected.set(id.clone());
        });
    if row.selected {
        line = line.bg(rgb(0x00DC_DCE2).to_hsla());
    }
    line = line.child(glyph_box(row.icon, row.selected));
    line = line.child(
        text(row.label.clone())
            .text_size(px(13.0))
            .text_color(ink())
            .flex_1(),
    );
    if let Some(count) = row.count {
        line = line.child(
            text(count.to_string())
                .text_size(px(12.0))
                .text_color(quiet()),
        );
    }
    line.into_element()
}

fn glyph_box(name: &str, solid: bool) -> AnyElement {
    let key = if solid {
        format!("{name}-solid")
    } else {
        name.to_string()
    };
    let path = crate::icons::glyph_file(&key).or_else(|| crate::icons::glyph_file(name));
    match path {
        Some(path) => image(format!("file://{}", path.display()))
            .size(px(16.0))
            .into_element(),
        None => view().size(px(16.0)).into_element(),
    }
}

fn chat_pane(rect: Rect, form: Signal<Form>) -> AnyElement {
    view()
        .absolute()
        .left(px(rect.x as f32))
        .top(px(rect.y as f32))
        .w(px(rect.w as f32))
        .h(px(rect.h as f32))
        .flex()
        .flex_col()
        .bg(field())
        .px(px(16.0))
        .pt(px(16.0))
        .pb(px(16.0))
        .child(
            view()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(10.0))
                .child(avatar())
                .child(
                    view()
                        .flex_1()
                        .flex_col()
                        .child(
                            text("Slowpage")
                                .text_size(px(15.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(ink()),
                        )
                        .child(text("你的阅读助手").text_size(px(12.0)).text_color(quiet())),
                )
                .child(panel_toggle_button(form, false)),
        )
        .child(
            view()
                .flex_1()
                .pt(px(36.0))
                .child(text("还没有对话").text_size(px(13.0)).text_color(quiet())),
        )
        .child(chat_field())
        .into_element()
}

fn avatar() -> AnyElement {
    view()
        .size(px(36.0))
        .rounded(px(18.0))
        .bg(rgb(0x00E3_6A4A).to_hsla())
        .flex()
        .items_center()
        .justify_center()
        .child(
            text("阅")
                .text_size(px(14.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(hsla(0.0, 0.0, 1.0, 1.0)),
        )
        .into_element()
}

fn chat_field() -> AnyElement {
    view()
        .h(px(44.0))
        .rounded(px(22.0))
        .bg(rgb(0x00FF_FFFF).to_hsla())
        .px(px(16.0))
        .flex()
        .flex_row()
        .items_center()
        .accessible(true)
        .label("问问 Slowpage…")
        .child(
            text("问问 Slowpage…")
                .text_size(px(14.0))
                .text_color(quiet()),
        )
        .into_element()
}

#[derive(Clone, Copy)]
pub enum Peer {
    AppleBooks,
    Calibre,
    AddFolder,
}

fn card_face(
    rect: Rect,
    radius: f64,
    face: CardFace,
    library: &LibraryIn,
    on_peer: impl Fn(Peer) + Clone + 'static,
    on_open: impl Fn(String) + Clone + 'static,
) -> AnyElement {
    let title = selected_title(&library.selected);
    let mut body = view()
        .absolute()
        .left(px(rect.x as f32))
        .top(px(rect.y as f32))
        .w(px(rect.w as f32))
        .h(px(rect.h as f32))
        .rounded(px(radius as f32))
        .bg(rgb(0x00FF_FFFF).to_hsla())
        .overflow_hidden()
        .flex()
        .flex_col()
        .p(px(28.0));
    body = body.child(card_header(&title, library.book_count));
    if let Some(notice) = &library.notice {
        body = body.child(
            text(notice.clone())
                .text_size(px(14.0))
                .text_color(quiet())
                .accessible(true)
                .label(notice.clone()),
        );
    }
    body = match face {
        CardFace::EmptyPeers => body.child(empty_peers(on_peer)),
        CardFace::Grid => body.child(book_grid(
            library,
            (rect.w - 56.0).max(120.0) as f32,
            on_open,
        )),
        CardFace::Placeholder => body,
        CardFace::Page => body,
    };
    body.into_element()
}

fn card_header(title: &str, count: u32) -> AnyElement {
    view()
        .flex()
        .flex_col()
        .gap(px(4.0))
        .child(
            text(title.to_string())
                .text_size(px(22.0))
                .text_color(ink()),
        )
        .child(
            view()
                .flex()
                .flex_row()
                .items_center()
                .child(
                    text(format!("{count} 本"))
                        .text_size(px(13.0))
                        .text_color(quiet()),
                )
                .child(view().flex_1())
                .child(text("最近阅读").text_size(px(13.0)).text_color(quiet())),
        )
        .into_element()
}

fn book_grid(
    library: &LibraryIn,
    width: f32,
    on_open: impl Fn(String) + Clone + 'static,
) -> AnyElement {
    let mut column = view().flex_1().flex().flex_col().pt(px(22.0)).gap(px(18.0));
    if !library.continue_reading.is_empty() {
        column = column.child(section_label("继续阅读"));
        let mut stack = view().flex().flex_col().gap(px(10.0));
        for card in &library.continue_reading {
            stack = stack.child(continue_card(card, on_open.clone()));
        }
        column = column.child(stack);
    }
    column = column.child(section_label("书架"));
    let mut row = view().w(px(width)).flex().flex_row().gap(px(22.0));
    let mut placed = 0usize;
    for card in &library.covers {
        if placed == 2 {
            column = column.child(row);
            row = view().w(px(width)).flex().flex_row().gap(px(22.0));
            placed = 0;
        }
        row = row.child(cover_tile(card, on_open.clone()));
        placed += 1;
    }
    if placed > 0 {
        column = column.child(row);
    }
    column.into_element()
}

fn section_label(label: &str) -> AnyElement {
    text(label.to_string())
        .text_size(px(13.0))
        .text_color(quiet())
        .into_element()
}

fn continue_card(card: &CoverCard, on_open: impl Fn(String) + 'static) -> AnyElement {
    let path = card.open_path.clone();
    let progress = match card.percent {
        Some(percent) => format!("{percent}%"),
        None => card
            .page
            .map(|page| format!("第 {page} 页"))
            .unwrap_or_default(),
    };
    let mut tile = view()
        .w(px(220.0))
        .h(px(72.0))
        .rounded(px(12.0))
        .bg(rgb(0x00F6_F6F8).to_hsla())
        .px(px(10.0))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(10.0))
        .accessible(true)
        .label(format!("继续阅读 {}", card.title));
    if let Some(path) = path {
        tile = tile.on_press(move || on_open(path.clone()));
    }
    tile.child(cover_art(card, 36.0, 54.0))
        .child(
            view()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    text(card.title.clone())
                        .text_size(px(13.0))
                        .text_color(ink()),
                )
                .child(text(progress).text_size(px(12.0)).text_color(quiet())),
        )
        .into_element()
}

fn cover_tile(card: &CoverCard, on_open: impl Fn(String) + 'static) -> AnyElement {
    let path = card.open_path.clone();
    let mut tile = view()
        .w(px(112.0))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .accessible(true)
        .label(card.title.clone());
    if let Some(path) = path {
        tile = tile.on_press(move || on_open(path.clone()));
    }
    tile = tile
        .child(cover_art(card, 112.0, 168.0))
        .child(
            text(card.title.clone())
                .text_size(px(12.0))
                .text_color(quiet()),
        )
        .child(
            text(if card.author.is_empty() {
                " ".to_string()
            } else {
                card.author.clone()
            })
            .text_size(px(12.0))
            .text_color(quiet()),
        );
    if let Some(label) = &card.unavailable {
        tile = tile.child(text(label.clone()).text_size(px(12.0)).text_color(quiet()));
    }
    tile.into_element()
}

fn cover_art(card: &CoverCard, width: f32, height: f32) -> AnyElement {
    let mut art = view()
        .w(px(width))
        .h(px(height))
        .rounded(px(4.0))
        .overflow_hidden()
        .bg(rgb(fallback_color(&card.title)).to_hsla());
    if let Some(path) = &card.cover {
        art = art.child(
            image(format!("file://{}", path.display()))
                .w(px(width))
                .h(px(height)),
        );
    } else {
        art = art.child(
            view()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .px(px(8.0))
                .child(
                    text(card.title.clone())
                        .text_size(px(12.0))
                        .text_color(rgb(0x00FF_FFFF).to_hsla()),
                ),
        );
    }
    if card.pdf {
        art = art.child(
            view()
                .absolute()
                .right(px(6.0))
                .bottom(px(6.0))
                .rounded(px(4.0))
                .px(px(4.0))
                .bg(rgb(0x00FF_FFFF).to_hsla())
                .child(text("PDF").text_size(px(10.0)).text_color(ink())),
        );
    }
    art.into_element()
}

fn fallback_color(title: &str) -> u32 {
    let mut hash = 0u32;
    for byte in title.as_bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(u32::from(*byte));
    }
    const COLORS: [u32; 5] = [
        0x002F_4F3E,
        0x008B_3A3A,
        0x001E_3A5F,
        0x008A_5A2B,
        0x003D_4C5A,
    ];
    COLORS[(hash as usize) % COLORS.len()]
}

fn empty_peers(on_peer: impl Fn(Peer) + Clone + 'static) -> AnyElement {
    view()
        .flex_1()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(22.0))
        .child(
            text("从这些来源加入书籍")
                .text_size(px(13.0))
                .text_color(quiet()),
        )
        .child(
            view()
                .flex()
                .flex_row()
                .gap(px(16.0))
                .child(peer_tile(
                    "Apple Books",
                    "book",
                    Peer::AppleBooks,
                    on_peer.clone(),
                ))
                .child(peer_tile(
                    "calibre",
                    "library",
                    Peer::Calibre,
                    on_peer.clone(),
                ))
                .child(peer_tile("添加文件夹", "folder", Peer::AddFolder, on_peer)),
        )
        .into_element()
}

fn peer_tile(label: &str, icon: &str, peer: Peer, on_peer: impl Fn(Peer) + 'static) -> AnyElement {
    view()
        .w(px(148.0))
        .h(px(112.0))
        .rounded(px(12.0))
        .bg(rgb(0x00F6_F6F8).to_hsla())
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(10.0))
        .accessible(true)
        .label(label.to_string())
        .on_press(move || on_peer(peer))
        .child(glyph_box(icon, false))
        .child(
            text(label.to_string())
                .text_size(px(13.0))
                .text_color(ink()),
        )
        .into_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window() -> Window {
        Window {
            width: 1440.0,
            height: 900.0,
        }
    }

    fn empty_library() -> LibraryIn {
        LibraryIn {
            empty: true,
            book_count: 0,
            counts: LibraryCounts::default(),
            sources: Vec::new(),
            selected: SelectId::AllBooks,
            continue_reading: Vec::new(),
            covers: Vec::new(),
            notice: None,
        }
    }

    fn near(got: f64, want: f64) {
        assert!((got - want).abs() < 0.01, "{got} vs {want}");
    }

    #[test]
    fn launch_is_full_with_both_columns() {
        let form = launch();
        let placed = place(
            &PlaceIn {
                form,
                window: window(),
                traffic: 92.0,
            },
            &empty_library(),
        );
        let sidebar = placed.sidebar.expect("sidebar");
        near(sidebar.w, 220.0);
        let chat = placed.chat.expect("chat");
        near(chat.w, 320.0);
        let CardBox::Placed {
            rect,
            radius,
            shadow,
        } = placed.card
        else {
            panic!("card");
        };
        near(rect.x, 226.0);
        near(rect.y, 6.0);
        near(rect.w, 1440.0 - 220.0 - 320.0 - 12.0);
        near(rect.h, 900.0 - 12.0);
        near(radius, 12.0);
        assert!(shadow);
        assert_eq!(placed.face, CardFace::EmptyPeers);
        assert_eq!(placed.groups, ["书库", "笔记", "书单", "来源"]);
    }

    #[test]
    fn hiding_a_column_keeps_the_six_point_inset() {
        let hidden_side = step(launch(), Command::ToggleSidebar);
        let placed = place(
            &PlaceIn {
                form: hidden_side,
                window: window(),
                traffic: 92.0,
            },
            &empty_library(),
        );
        assert!(placed.sidebar.is_none());
        let CardBox::Placed { rect, .. } = placed.card else {
            panic!("card");
        };
        near(rect.x, 6.0);
        near(rect.w, 1440.0 - 320.0 - 12.0);
        let hidden_chat = step(launch(), Command::TogglePanel);
        let placed = place(
            &PlaceIn {
                form: hidden_chat,
                window: window(),
                traffic: 92.0,
            },
            &empty_library(),
        );
        assert!(placed.chat.is_none());
        let CardBox::Placed { rect, .. } = placed.card else {
            panic!("card");
        };
        near(rect.x, 226.0);
        near(rect.w, 1440.0 - 220.0 - 12.0);
        let right = window().width - (rect.x + rect.w);
        near(right, 6.0);
    }

    #[test]
    fn a_toggle_twice_restores_full_and_does_not_leave_it() {
        let once = step(launch(), Command::TogglePanel);
        assert!(matches!(
            once,
            Form::Full {
                chat: Presence::Hidden,
                ..
            }
        ));
        assert_eq!(step(once, Command::TogglePanel), launch());
        assert_eq!(
            step(Form::Immersive, Command::ToggleSidebar),
            Form::Immersive
        );
    }

    #[test]
    fn a_shown_margin_has_no_chat_width() {
        let form = Form::PageAndPanel {
            sidebar: Presence::Hidden,
            margin: Presence::Shown,
        };
        let placed = place(
            &PlaceIn {
                form,
                window: window(),
                traffic: 0.0,
            },
            &empty_library(),
        );
        assert!(placed.chat.is_none());
        assert!(matches!(placed.card, CardBox::AwaitingMargin { .. }));
    }

    #[test]
    fn a_connected_source_keeps_its_count_and_a_denial_drops_it() {
        let mut rows = Vec::new();
        remember_source(&mut rows, "Apple Books", 4, true);
        remember_source(&mut rows, "calibre", 6, true);
        assert_eq!(rows[0].count, 4);
        remember_source(&mut rows, "Apple Books", 5, true);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].count, 5);
        assert_eq!(rows[1].label, "calibre");
        assert_eq!(rows[1].count, 6);
        remember_source(&mut rows, "calibre", 0, false);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].label, "Apple Books");
    }

    #[test]
    fn books_fill_the_grid() {
        let mut library = empty_library();
        library.empty = false;
        library.book_count = 4;
        let placed = place(
            &PlaceIn {
                form: launch(),
                window: window(),
                traffic: 0.0,
            },
            &library,
        );
        assert_eq!(placed.face, CardFace::Grid);
    }
}
