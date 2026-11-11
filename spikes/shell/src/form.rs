//! The shell is one form. `step` is the only writer. `place` is the geometry.
//! `mount` paints that geometry and does not invent a second one.

use std::path::PathBuf;

use nsh_core::prelude::*;

const SIDEBAR_W: f64 = crate::tokens::SIDEBAR_W;
const CHAT_W: f64 = crate::tokens::CHAT_W;
const INSET: f64 = crate::tokens::INSET;
const RADIUS: f64 = crate::tokens::CARD_RADIUS;

pub const FIELD_HEX: u32 = crate::tokens::FIELD_TOP;

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
    pub position: String,
    pub opened: String,
    pub unavailable: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookList {
    pub label: String,
    pub count: u32,
    pub color: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChatLine {
    pub kind: String,
    pub text: String,
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
    pub lists: Vec<BookList>,
    pub chat: Vec<ChatLine>,
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
    pub dot: Option<u32>,
    pub badge: Option<(u32, u32)>,
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
    for list in &library.lists {
        let mut item = row(
            SelectId::Source(list.label.clone()),
            &list.label,
            Some(list.count),
            "",
            "书单",
            &library.selected,
        );
        item.dot = Some(list.color);
        out.push(item);
    }
    for source in &library.sources {
        let (icon, badge) = source_face(&source.label);
        let mut item = row(
            SelectId::Source(source.label.clone()),
            &source.label,
            Some(source.count),
            icon,
            "来源",
            &library.selected,
        );
        item.badge = badge;
        out.push(item);
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
        "sliders",
        "",
        &library.selected,
    ));
    out
}

fn source_face(label: &str) -> (&'static str, Option<(u32, u32)>) {
    match label {
        "Apple Books" => (
            "book-open",
            Some((crate::tokens::APPLE_FROM, crate::tokens::APPLE_TO)),
        ),
        "calibre" => (
            "library",
            Some((crate::tokens::CALIBRE_FROM, crate::tokens::CALIBRE_TO)),
        ),
        _ => ("folder", None),
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
        dot: None,
        badge: None,
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
    rgb(crate::tokens::INK).to_hsla()
}

fn quiet() -> Hsla {
    rgb(crate::tokens::QUIET).to_hsla()
}

fn icon_ink() -> Hsla {
    rgb(crate::tokens::ICON).to_hsla()
}

fn faint() -> Hsla {
    rgb(crate::tokens::FAINT).to_hsla()
}

fn wash(alpha: f32) -> Hsla {
    hsla(0.0, 0.0, 0.0, alpha)
}

macro_rules! line {
    ($text:expr, $size:expr, $weight:expr, $color:expr, $track:expr) => {
        text($text)
            .font_family(SharedString::from(crate::tokens::FONT_STACK))
            .text_size(px($size))
            .font_weight($weight)
            .text_color($color)
            .letter_spacing(px(crate::tokens::track($size, $track)))
    };
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
        root = root
            .bg_linear_to_b()
            .gradient_from(rgb(crate::tokens::FIELD_TOP).to_hsla())
            .gradient_to(rgb(crate::tokens::FIELD_BOTTOM).to_hsla());
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
                    .bg(rgb(crate::tokens::CARD).to_hsla())
                    .shadow(crate::tokens::card_shadows()),
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
        root = root.child(chat_pane(rect, form, &library));
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
        .child(sidebar_top(placed.traffic, form));
    let mut groups = view()
        .flex_1()
        .flex_col()
        .px(px(crate::tokens::SIDE_PAD_X))
        .pb(px(crate::tokens::SIDE_PAD_BOTTOM));
    groups = groups.child(search_field());
    for label in &placed.groups {
        groups = groups.child(group_label(label));
        for row in placed.rows.iter().filter(|row| row.group == *label) {
            groups = groups.child(side_row(row.clone(), selected, menu));
        }
    }
    column = column.child(groups);
    if let Some(settings) = placed.rows.iter().find(|row| row.id == SelectId::Settings) {
        column = column.child(
            view()
                .px(px(crate::tokens::SIDE_PAD_X))
                .pb(px(crate::tokens::SIDE_PAD_BOTTOM))
                .child(side_row(settings.clone(), selected, menu)),
        );
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
        .h(px(crate::tokens::TOP_ROW_H))
        .flex()
        .flex_row()
        .items_center()
        .child(view().w(px(leading as f32)).h(px(crate::tokens::TOP_ROW_H)))
        .child(sidebar_toggle_button(0.0, form))
        .into_element()
}

fn sidebar_toggle_button(traffic: f64, form: Signal<Form>) -> AnyElement {
    let mut button = view()
        .w(px(crate::tokens::TOGGLE_SIZE))
        .h(px(crate::tokens::TOGGLE_SIZE))
        .rounded(px(crate::tokens::TOGGLE_RADIUS))
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
        button = button
            .absolute()
            .left(px(traffic as f32))
            .top(px(crate::tokens::TOGGLE_TOP));
    }
    if let Some(path) = crate::icons::glyph_ink("sidebar-left", "6E6E73") {
        button = button.child(
            image(format!("file://{}", path.display())).size(px(crate::tokens::SIDEBAR_ICON)),
        );
    }
    button.into_element()
}

fn panel_toggle_button(form: Signal<Form>, floating: bool) -> AnyElement {
    let mut button = view()
        .w(px(crate::tokens::TOGGLE_SIZE))
        .h(px(crate::tokens::TOGGLE_SIZE))
        .rounded(px(crate::tokens::TOGGLE_RADIUS))
        .flex()
        .items_center()
        .justify_center()
        .accessible(true)
        .label("对话")
        .on_press(move || {
            form.set(step(form.get(), Command::TogglePanel));
        });
    if floating {
        button = button
            .absolute()
            .right(px(crate::tokens::CHAT_PAD_X))
            .top(px(crate::tokens::TOGGLE_TOP));
    }
    if let Some(path) = crate::icons::glyph_ink("sidebar-right", "6E6E73") {
        button = button
            .child(image(format!("file://{}", path.display())).size(px(crate::tokens::PANEL_ICON)));
    }
    button.into_element()
}

fn search_field() -> AnyElement {
    view()
        .mb(px(crate::tokens::SEARCH_MB))
        .h(px(crate::tokens::SEARCH_H))
        .rounded(px(crate::tokens::SEARCH_RADIUS))
        .bg(wash(crate::tokens::SEARCH_ALPHA))
        .px(px(crate::tokens::SEARCH_PAD_X))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(crate::tokens::SEARCH_GAP))
        .child(glyph_box("search", false, crate::tokens::ROW_ICON, None))
        .child(line!(
            "搜索书名、作者、笔记",
            crate::tokens::SEARCH_SIZE,
            crate::tokens::BODY_WEIGHT,
            quiet(),
            crate::tokens::BODY_TRACK_EM
        ))
        .into_element()
}

fn group_label(label: &str) -> AnyElement {
    view()
        .pt(px(crate::tokens::LABEL_MT))
        .pb(px(crate::tokens::LABEL_MB))
        .px(px(crate::tokens::LABEL_PX))
        .child(line!(
            label.to_string(),
            crate::tokens::LABEL_SIZE,
            crate::tokens::LABEL_WEIGHT,
            quiet(),
            crate::tokens::LABEL_TRACK_EM
        ))
        .into_element()
}

fn side_row(row: SideRow, selected: Signal<SelectId>, menu: Signal<bool>) -> AnyElement {
    let id = row.id.clone();
    let dotted = row.dot.is_some();
    let add = matches!(row.id, SelectId::AddSource);
    let weight = if row.selected {
        crate::tokens::ROW_WEIGHT_ON
    } else {
        crate::tokens::BODY_WEIGHT
    };
    let color = if add { quiet() } else { ink() };
    let mut line = view()
        .h(px(crate::tokens::ROW_H))
        .rounded(px(crate::tokens::ROW_RADIUS))
        .pl(px(crate::tokens::ROW_PAD_L))
        .pr(px(crate::tokens::ROW_PAD_R))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(crate::tokens::ROW_GAP))
        .accessible(true)
        .label(row.label.clone())
        .on_press(move || {
            if add {
                menu.set(!menu.get());
                return;
            }
            if dotted {
                return;
            }
            menu.set(false);
            selected.set(id.clone());
        });
    if row.selected {
        line = line.bg(wash(crate::tokens::ROW_ON_ALPHA));
    }
    line = line.child(row_mark(&row));
    line = line.child(view().flex_1().child(line!(
        row.label.clone(),
        crate::tokens::BODY_SIZE,
        weight,
        color,
        crate::tokens::BODY_TRACK_EM
    )));
    if let Some(count) = row.count {
        line = line.child(line!(
            count.to_string(),
            crate::tokens::COUNT_ROW_SIZE,
            crate::tokens::BODY_WEIGHT,
            quiet(),
            crate::tokens::BODY_TRACK_EM
        ));
    }
    line.into_element()
}

fn row_mark(row: &SideRow) -> AnyElement {
    if let Some(color) = row.dot {
        return view()
            .w(px(crate::tokens::ROW_ICON_SLOT))
            .flex()
            .items_center()
            .justify_center()
            .child(
                view()
                    .size(px(crate::tokens::DOT))
                    .rounded(px(crate::tokens::DOT / 2.0))
                    .bg(rgb(color).to_hsla()),
            )
            .into_element();
    }
    if let Some((from, to)) = row.badge {
        return view()
            .w(px(crate::tokens::ROW_ICON_SLOT))
            .flex()
            .items_center()
            .justify_center()
            .child(
                view()
                    .size(px(crate::tokens::SOURCE_BADGE))
                    .rounded(px(crate::tokens::SOURCE_BADGE_RADIUS))
                    .bg_linear_to_b()
                    .gradient_from(rgb(from).to_hsla())
                    .gradient_to(rgb(to).to_hsla())
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(glyph_box(
                        row.icon,
                        false,
                        crate::tokens::SOURCE_GLYPH,
                        Some("FFFFFF"),
                    )),
            )
            .into_element();
    }
    let size = if row.icon == "plus" {
        crate::tokens::ADD_ICON
    } else if row.id == SelectId::Settings {
        crate::tokens::SETTINGS_ICON
    } else {
        crate::tokens::ROW_ICON
    };
    let hex = if row.selected {
        "1D1D1F"
    } else if matches!(row.id, SelectId::AddSource) {
        "8E8E93"
    } else {
        "6E6E73"
    };
    view()
        .w(px(crate::tokens::ROW_ICON_SLOT))
        .flex()
        .items_center()
        .justify_center()
        .child(glyph_box(row.icon, row.selected, size, Some(hex)))
        .into_element()
}

fn glyph_box(name: &str, solid: bool, size: f32, hex: Option<&str>) -> AnyElement {
    if name.is_empty() {
        return view().size(px(size)).into_element();
    }
    let key = if solid {
        format!("{name}-solid")
    } else {
        name.to_string()
    };
    let ink = hex.unwrap_or("1D1D1F");
    let path = crate::icons::glyph_ink(&key, ink).or_else(|| crate::icons::glyph_ink(name, ink));
    match path {
        Some(path) => image(format!("file://{}", path.display()))
            .size(px(size))
            .into_element(),
        None => view().size(px(size)).into_element(),
    }
}

fn chat_pane(rect: Rect, form: Signal<Form>, library: &LibraryIn) -> AnyElement {
    let chip = library
        .continue_reading
        .first()
        .map(|card| card.title.clone())
        .unwrap_or_default();
    view()
        .absolute()
        .left(px(rect.x as f32))
        .top(px(rect.y as f32))
        .w(px(rect.w as f32))
        .h(px(rect.h as f32))
        .flex()
        .flex_col()
        .child(chat_head(form))
        .child(chat_thread(library))
        .child(composer(&chip))
        .into_element()
}

fn chat_head(form: Signal<Form>) -> AnyElement {
    view()
        .h(px(crate::tokens::CHAT_HEAD_H))
        .px(px(crate::tokens::CHAT_PAD_X))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(crate::tokens::CHAT_HEAD_GAP))
        .child(avatar())
        .child(
            view()
                .flex_1()
                .flex()
                .flex_row()
                .items_baseline()
                .gap(px(crate::tokens::NAME_GAP))
                .child(line!(
                    "Slowpage",
                    crate::tokens::NAME_SIZE,
                    crate::tokens::NAME_WEIGHT,
                    ink(),
                    crate::tokens::BODY_TRACK_EM
                ))
                .child(line!(
                    "你的阅读助手",
                    crate::tokens::SUB_SIZE,
                    crate::tokens::BODY_WEIGHT,
                    quiet(),
                    crate::tokens::BODY_TRACK_EM
                )),
        )
        .child(
            view()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(crate::tokens::TOOL_CLUSTER_GAP))
                .child(icon_button("edit", "新对话", crate::tokens::PANEL_ICON))
                .child(panel_toggle_button(form, false)),
        )
        .into_element()
}

fn icon_button(name: &str, label: &str, size: f32) -> AnyElement {
    let mut button = view()
        .w(px(crate::tokens::TOGGLE_SIZE))
        .h(px(crate::tokens::TOGGLE_SIZE))
        .rounded(px(crate::tokens::TOGGLE_RADIUS))
        .flex()
        .items_center()
        .justify_center()
        .accessible(true)
        .label(label.to_string());
    if let Some(path) = crate::icons::glyph_ink(name, "6E6E73") {
        button = button.child(image(format!("file://{}", path.display())).size(px(size)));
    }
    button.into_element()
}

fn avatar() -> AnyElement {
    view()
        .size(px(crate::tokens::AVATAR))
        .rounded(px(crate::tokens::AVATAR / 2.0))
        .bg_linear_to_br()
        .gradient_from(rgb(crate::tokens::AVATAR_FROM).to_hsla())
        .gradient_to(rgb(crate::tokens::AVATAR_TO).to_hsla())
        .flex()
        .items_center()
        .justify_center()
        .child(line!(
            "慢",
            crate::tokens::AVATAR_MARK_SIZE,
            crate::tokens::AVATAR_MARK_WEIGHT,
            rgb(crate::tokens::CARD).to_hsla(),
            0.0
        ))
        .into_element()
}

fn chat_thread(library: &LibraryIn) -> AnyElement {
    let mut column = view()
        .flex_1()
        .flex()
        .flex_col()
        .px(px(crate::tokens::CHAT_PAD_X))
        .pt(px(crate::tokens::CONV_PT))
        .pb(px(crate::tokens::COMPOSER_CLEAR))
        .gap(px(crate::tokens::CONV_GAP));
    for line in &library.chat {
        column = column.child(chat_line(line));
    }
    column.into_element()
}

fn chat_line(line: &ChatLine) -> AnyElement {
    match line.kind.as_str() {
        "time" => view()
            .flex()
            .justify_center()
            .child(line!(
                line.text.clone(),
                crate::tokens::TIME_RULE_SIZE,
                crate::tokens::BODY_WEIGHT,
                faint(),
                crate::tokens::BODY_TRACK_EM
            ))
            .into_element(),
        "user" => view()
            .flex()
            .flex_row()
            .justify_end()
            .child(
                view()
                    .max_w(px(crate::tokens::BUBBLE_MAX))
                    .rounded(px(crate::tokens::BUBBLE_RADIUS))
                    .px(px(crate::tokens::BUBBLE_PX))
                    .py(px(crate::tokens::BUBBLE_PY))
                    .bg(rgb(crate::tokens::CARD).to_hsla())
                    .shadow(crate::tokens::bubble_shadows())
                    .child(line!(
                        line.text.clone(),
                        crate::tokens::MSG_SIZE,
                        crate::tokens::BODY_WEIGHT,
                        ink(),
                        crate::tokens::BODY_TRACK_EM
                    )),
            )
            .into_element(),
        "chips" => {
            let mut row = view().flex().flex_row().gap(px(crate::tokens::CHIP_GAP));
            for chip in line.text.split('\n').filter(|chip| !chip.is_empty()) {
                row = row.child(
                    view()
                        .rounded(px(crate::tokens::CHIP_RADIUS))
                        .px(px(crate::tokens::CHIP_PX))
                        .py(px(crate::tokens::CHIP_PY))
                        .bg(hsla(0.0, 0.0, 1.0, 0.55))
                        .shadow(crate::tokens::chip_shadows())
                        .child(line!(
                            chip.to_string(),
                            crate::tokens::CHIP_SIZE,
                            crate::tokens::BODY_WEIGHT,
                            rgb(crate::tokens::CHIP_INK).to_hsla(),
                            crate::tokens::BODY_TRACK_EM
                        )),
                );
            }
            row.into_element()
        }
        _ => line!(
            line.text.clone(),
            crate::tokens::MSG_SIZE,
            crate::tokens::BODY_WEIGHT,
            ink(),
            crate::tokens::BODY_TRACK_EM
        )
        .into_element(),
    }
}

fn composer(book: &str) -> AnyElement {
    let mut tools = view()
        .mt(px(crate::tokens::COMPOSER_ROW_MT))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(crate::tokens::COMPOSER_ROW_GAP))
        .child(icon_button("paperclip", "附件", crate::tokens::ATTACH_ICON));
    if !book.is_empty() {
        tools = tools.child(
            view()
                .h(px(crate::tokens::SCOPE_H))
                .rounded(px(crate::tokens::SCOPE_RADIUS))
                .px(px(crate::tokens::SCOPE_PX))
                .gap(px(crate::tokens::SCOPE_GAP))
                .flex()
                .flex_row()
                .items_center()
                .bg(rgb(crate::tokens::SCOPE_BG).to_hsla())
                .child(glyph_box(
                    "book-open",
                    false,
                    crate::tokens::SOURCE_GLYPH,
                    Some("6E6E73"),
                ))
                .child(line!(
                    book.to_string(),
                    crate::tokens::SUB_SIZE,
                    crate::tokens::BODY_WEIGHT,
                    icon_ink(),
                    crate::tokens::BODY_TRACK_EM
                )),
        );
    }
    tools = tools.child(view().flex_1()).child(send_button());
    view()
        .absolute()
        .left(px(crate::tokens::COMPOSER_INSET))
        .right(px(crate::tokens::COMPOSER_INSET))
        .bottom(px(crate::tokens::COMPOSER_INSET))
        .rounded(px(crate::tokens::COMPOSER_RADIUS))
        .pt(px(crate::tokens::COMPOSER_PT))
        .pr(px(crate::tokens::COMPOSER_PR))
        .pb(px(crate::tokens::COMPOSER_PB))
        .pl(px(crate::tokens::COMPOSER_PL))
        .bg(rgb(crate::tokens::CARD).to_hsla())
        .shadow(crate::tokens::composer_shadows())
        .accessible(true)
        .label("问问 Slowpage…")
        .child(line!(
            "问问 Slowpage…",
            crate::tokens::BODY_SIZE,
            crate::tokens::BODY_WEIGHT,
            faint(),
            crate::tokens::BODY_TRACK_EM
        ))
        .child(tools)
        .into_element()
}

fn send_button() -> AnyElement {
    let mut button = view()
        .size(px(crate::tokens::SEND))
        .rounded(px(crate::tokens::SEND / 2.0))
        .bg(rgb(crate::tokens::SEND_BG).to_hsla())
        .flex()
        .items_center()
        .justify_center()
        .accessible(true)
        .label("发送");
    if let Some(path) = crate::icons::glyph_ink("arrow-up", "FFFFFF") {
        button = button
            .child(image(format!("file://{}", path.display())).size(px(crate::tokens::SEND_ICON)));
    }
    button.into_element()
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
        .bg(rgb(crate::tokens::CARD).to_hsla())
        .overflow_hidden()
        .flex()
        .flex_col();
    if !matches!(face, CardFace::Page) {
        body = body.child(card_toolbar());
    }
    let mut column = view()
        .flex_1()
        .flex()
        .flex_col()
        .px(px(crate::tokens::CARD_PAD_X))
        .pt(px(crate::tokens::CARD_PAD_TOP));
    if !matches!(face, CardFace::Page) {
        column = column.child(card_header(&title, library.book_count));
    }
    if let Some(notice) = &library.notice {
        column = column.child(
            line!(
                notice.clone(),
                crate::tokens::BODY_SIZE,
                crate::tokens::BODY_WEIGHT,
                quiet(),
                crate::tokens::BODY_TRACK_EM
            )
            .accessible(true)
            .label(notice.clone()),
        );
    }
    column = match face {
        CardFace::EmptyPeers => column.child(empty_peers(on_peer)),
        CardFace::Grid => column.child(book_grid(
            library,
            (rect.w as f32 - crate::tokens::CARD_PAD_X * 2.0).max(crate::tokens::SHELF_COVER_W),
            on_open,
        )),
        CardFace::Placeholder | CardFace::Page => column,
    };
    body.child(column).into_element()
}

fn card_toolbar() -> AnyElement {
    view()
        .h(px(crate::tokens::TOOLBAR_H))
        .px(px(crate::tokens::TOOLBAR_PX))
        .gap(px(crate::tokens::TOOLBAR_GAP))
        .flex()
        .flex_row()
        .items_center()
        .child(view().flex_1())
        .child(sort_control())
        .child(view_toggle())
        .into_element()
}

fn sort_control() -> AnyElement {
    let mut control = view()
        .h(px(crate::tokens::TOOL_H))
        .px(px(crate::tokens::TOOL_PX))
        .gap(px(crate::tokens::TOOL_GAP))
        .rounded(px(crate::tokens::TOOL_RADIUS))
        .flex()
        .flex_row()
        .items_center()
        .accessible(true)
        .label("最近阅读");
    if let Some(path) = crate::icons::glyph_ink("sort-desc", "6E6E73") {
        control = control
            .child(image(format!("file://{}", path.display())).size(px(crate::tokens::SORT_ICON)));
    }
    control = control.child(line!(
        "最近阅读",
        crate::tokens::TOOL_SIZE,
        crate::tokens::BODY_WEIGHT,
        icon_ink(),
        crate::tokens::BODY_TRACK_EM
    ));
    if let Some(path) = crate::icons::glyph_ink("chevron-down", "6E6E73") {
        control = control.child(
            image(format!("file://{}", path.display())).size(px(crate::tokens::CHEVRON_ICON)),
        );
    }
    control.into_element()
}

fn view_toggle() -> AnyElement {
    view()
        .p(px(crate::tokens::SEG_PAD))
        .rounded(px(crate::tokens::SEG_RADIUS))
        .bg(wash(crate::tokens::SEG_ALPHA))
        .flex()
        .flex_row()
        .child(view_button("grid", "网格", true))
        .child(view_button("list", "列表", false))
        .into_element()
}

fn view_button(name: &str, label: &str, on: bool) -> AnyElement {
    let mut button = view()
        .w(px(crate::tokens::SEG_BTN_W))
        .h(px(crate::tokens::SEG_BTN_H))
        .rounded(px(crate::tokens::SEG_BTN_RADIUS))
        .flex()
        .items_center()
        .justify_center()
        .accessible(true)
        .label(label.to_string());
    if on {
        button = button
            .bg(rgb(crate::tokens::CARD).to_hsla())
            .shadow(crate::tokens::seg_shadows());
    }
    let hex = if on { "1D1D1F" } else { "6E6E73" };
    if let Some(path) = crate::icons::glyph_ink(name, hex) {
        button = button
            .child(image(format!("file://{}", path.display())).size(px(crate::tokens::VIEW_ICON)));
    }
    button.into_element()
}

fn card_header(title: &str, count: u32) -> AnyElement {
    view()
        .h(px(crate::tokens::TITLE_ROW_H))
        .flex()
        .flex_row()
        .items_baseline()
        .gap(px(crate::tokens::TITLE_GAP))
        .child(line!(
            title.to_string(),
            crate::tokens::H1_SIZE,
            crate::tokens::H1_WEIGHT,
            ink(),
            crate::tokens::H1_TRACK_EM
        ))
        .child(line!(
            format!("{count} 本"),
            crate::tokens::COUNT_SIZE,
            crate::tokens::COUNT_WEIGHT,
            quiet(),
            0.0
        ))
        .into_element()
}

fn book_grid(
    library: &LibraryIn,
    width: f32,
    on_open: impl Fn(String) + Clone + 'static,
) -> AnyElement {
    let mut column = view().flex().flex_col();
    if !library.continue_reading.is_empty() {
        column = column
            .child(section_label("继续阅读", None))
            .child(continue_row(library, on_open.clone()));
    }
    let note = if matches!(library.selected, SelectId::AllBooks) {
        Some("按最近阅读")
    } else {
        None
    };
    column = column.child(section_label("书架", note));
    let columns = crate::tokens::shelf_columns(width).max(1);
    let gap = crate::tokens::shelf_gap(width, columns);
    let mut row = view()
        .w(px(width))
        .mb(px(crate::tokens::SHELF_ROW_GAP))
        .flex()
        .flex_row()
        .gap(px(gap));
    let mut placed = 0u32;
    for card in &library.covers {
        if placed == columns {
            column = column.child(row);
            row = view()
                .w(px(width))
                .mb(px(crate::tokens::SHELF_ROW_GAP))
                .flex()
                .flex_row()
                .gap(px(gap));
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

fn continue_row(library: &LibraryIn, on_open: impl Fn(String) + Clone + 'static) -> AnyElement {
    let mut row = view()
        .flex()
        .flex_row()
        .gap(px(crate::tokens::CONTINUE_GAP));
    for card in library
        .continue_reading
        .iter()
        .take(crate::tokens::CONTINUE_MAX)
    {
        row = row.child(continue_card(card, on_open.clone()));
    }
    row.into_element()
}

fn section_label(label: &str, note: Option<&str>) -> AnyElement {
    let mut row = view()
        .mt(px(crate::tokens::SECTION_MT))
        .mb(px(crate::tokens::SECTION_MB))
        .gap(px(crate::tokens::SECTION_GAP))
        .flex()
        .flex_row()
        .items_baseline()
        .child(line!(
            label.to_string(),
            crate::tokens::SECTION_SIZE,
            crate::tokens::SECTION_WEIGHT,
            ink(),
            crate::tokens::BODY_TRACK_EM
        ));
    if let Some(note) = note {
        row = row.child(line!(
            note.to_string(),
            crate::tokens::SECTION_NOTE_SIZE,
            crate::tokens::BODY_WEIGHT,
            quiet(),
            0.0
        ));
    }
    row.into_element()
}

fn continue_card(card: &CoverCard, on_open: impl Fn(String) + 'static) -> AnyElement {
    let path = card.open_path.clone();
    let mut tile = view()
        .flex_1()
        .rounded(px(crate::tokens::CONTINUE_RADIUS))
        .p(px(crate::tokens::CONTINUE_PAD))
        .gap(px(crate::tokens::CONTINUE_INNER_GAP))
        .bg(rgb(crate::tokens::CONTINUE_BG).to_hsla())
        .flex()
        .flex_row()
        .accessible(true)
        .label(format!("继续阅读 {}", card.title));
    if let Some(path) = path {
        tile = tile.on_press(move || on_open(path.clone()));
    }
    tile.child(cover_art(
        card,
        crate::tokens::CONTINUE_COVER_W,
        crate::tokens::CONTINUE_COVER_H,
    ))
    .child(
        view()
            .flex_1()
            .flex()
            .flex_col()
            .pt(px(crate::tokens::CONTINUE_TEXT_PT))
            .child(
                line!(
                    card.title.clone(),
                    crate::tokens::CONTINUE_TITLE_SIZE,
                    crate::tokens::CONTINUE_TITLE_WEIGHT,
                    ink(),
                    crate::tokens::BODY_TRACK_EM
                )
                .truncate(),
            )
            .child(meta_line(card, true))
            .child(view().flex_1())
            .child(if card.percent.is_some() {
                progress_line(card)
            } else {
                view().into_element()
            })
            .child(line!(
                card.opened.clone(),
                crate::tokens::TIME_SIZE,
                crate::tokens::BODY_WEIGHT,
                faint(),
                crate::tokens::BODY_TRACK_EM
            )),
    )
    .into_element()
}

fn progress_line(card: &CoverCard) -> AnyElement {
    let percent = card.percent.unwrap_or(0);
    let mut bar = view()
        .flex_1()
        .h(px(crate::tokens::BAR_H))
        .rounded(px(crate::tokens::BAR_RADIUS))
        .overflow_hidden()
        .bg(wash(crate::tokens::BAR_TRACK_ALPHA))
        .flex()
        .flex_row();
    if percent > 0 {
        bar = bar.child(
            view()
                .h(px(crate::tokens::BAR_H))
                .flex_grow(f32::from(percent))
                .bg(ink()),
        );
        bar = bar.child(view().flex_grow(f32::from(100 - percent)));
    }
    view()
        .gap(px(crate::tokens::BAR_GAP))
        .flex()
        .flex_row()
        .items_center()
        .child(bar)
        .child(line!(
            card.percent
                .map(|value| format!("{value}%"))
                .unwrap_or_default(),
            crate::tokens::PROGRESS_SIZE,
            crate::tokens::BODY_WEIGHT,
            quiet(),
            crate::tokens::BODY_TRACK_EM
        ))
        .into_element()
}

fn cover_tile(card: &CoverCard, on_open: impl Fn(String) + 'static) -> AnyElement {
    let path = card.open_path.clone();
    let mut tile = view()
        .w(px(crate::tokens::SHELF_COVER_W))
        .flex()
        .flex_col()
        .accessible(true)
        .label(card.title.clone());
    if let Some(path) = path {
        tile = tile.on_press(move || on_open(path.clone()));
    }
    tile = tile
        .child(cover_art(
            card,
            crate::tokens::SHELF_COVER_W,
            crate::tokens::SHELF_COVER_H,
        ))
        .child(
            view().mt(px(crate::tokens::SHELF_TITLE_MT)).child(
                line!(
                    card.title.clone(),
                    crate::tokens::SHELF_TITLE_SIZE,
                    crate::tokens::SHELF_TITLE_WEIGHT,
                    ink(),
                    crate::tokens::BODY_TRACK_EM
                )
                .truncate(),
            ),
        )
        .child(author_line(card));
    if let Some(label) = &card.unavailable {
        tile = tile.child(line!(
            label.clone(),
            crate::tokens::SHELF_META_SIZE,
            crate::tokens::BODY_WEIGHT,
            quiet(),
            crate::tokens::BODY_TRACK_EM
        ));
    }
    tile.into_element()
}

fn author_line(card: &CoverCard) -> AnyElement {
    let mut row = view()
        .mt(px(crate::tokens::SHELF_META_MT))
        .gap(px(crate::tokens::SHELF_META_GAP))
        .flex()
        .flex_row()
        .items_center()
        .child(view().flex_1().child(line!(
            if card.author.is_empty() {
                " ".to_string()
            } else {
                card.author.clone()
            },
            crate::tokens::SHELF_META_SIZE,
            crate::tokens::BODY_WEIGHT,
            quiet(),
            crate::tokens::BODY_TRACK_EM
        )));
    if card.pdf {
        row = row.child(pdf_tag(false));
    }
    if let Some(percent) = card.percent {
        row = row.child(line!(
            format!("{percent}%"),
            crate::tokens::PROGRESS_SIZE,
            crate::tokens::BODY_WEIGHT,
            quiet(),
            crate::tokens::BODY_TRACK_EM
        ));
    }
    row.into_element()
}

fn meta_line(card: &CoverCard, small_tag: bool) -> AnyElement {
    let mut row = view()
        .mt(px(crate::tokens::CONTINUE_META_MT))
        .gap(px(crate::tokens::SHELF_META_GAP))
        .flex()
        .flex_row()
        .items_center()
        .child(line!(
            card.position.clone(),
            crate::tokens::CONTINUE_META_SIZE,
            crate::tokens::BODY_WEIGHT,
            quiet(),
            crate::tokens::BODY_TRACK_EM
        ));
    if card.pdf {
        row = row.child(pdf_tag(small_tag));
    }
    row.into_element()
}

fn pdf_tag(small: bool) -> AnyElement {
    let size = if small {
        crate::tokens::TAG_SM_SIZE
    } else {
        crate::tokens::TAG_SIZE
    };
    view()
        .rounded(px(crate::tokens::TAG_RADIUS))
        .px(px(crate::tokens::TAG_PX))
        .border(px(crate::tokens::TAG_BORDER))
        .border_color(wash(crate::tokens::TAG_BORDER_ALPHA))
        .child(line!(
            "PDF",
            size,
            crate::tokens::TAG_WEIGHT,
            icon_ink(),
            crate::tokens::TAG_TRACK_EM
        ))
        .into_element()
}

fn cover_art(card: &CoverCard, width: f32, height: f32) -> AnyElement {
    let mut art = view()
        .w(px(width))
        .h(px(height))
        .rounded(px(crate::tokens::SHELF_RADIUS))
        .overflow_hidden()
        .shadow(crate::tokens::cover_shadows())
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
                .px(px(crate::tokens::COVER_LABEL_PX))
                .child(line!(
                    card.title.clone(),
                    crate::tokens::SHELF_TITLE_SIZE,
                    crate::tokens::SHELF_TITLE_WEIGHT,
                    rgb(crate::tokens::CARD).to_hsla(),
                    crate::tokens::BODY_TRACK_EM
                )),
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
        .child(glyph_box(
            icon,
            false,
            crate::tokens::ROW_ICON,
            Some("6E6E73"),
        ))
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
            lists: Vec::new(),
            chat: Vec::new(),
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
