use serde_json::Value;

pub const DESIGN_W: f64 = 460.0;
pub const LEFT_W: f64 = 72.0;
pub const RIGHT_W: f64 = 300.0;
pub const TOP_MAT: f64 = 48.0;
pub const MAC_THREE: f64 = 1280.0;
pub const MAC_TWO: f64 = 1000.0;
pub const CARD_GAP: f64 = 8.0;
pub const SHEET_GAP: f64 = 12.0;
pub const SHEET_RADIUS: f64 = 14.0;
pub const BAR_SHARE: f64 = 0.08;
pub const PAGE_SHARE: f64 = 0.55;
pub const PANEL_FLOOR: f64 = 220.0;

pub fn paper_rgb() -> (f64, f64, f64) {
    (243.0 / 255.0, 238.0 / 255.0, 228.0 / 255.0)
}

pub fn ground_rgb() -> (f64, f64, f64) {
    (226.0 / 255.0, 220.0 / 255.0, 210.0 / 255.0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choice {
    pub left_open: bool,
    pub right_open: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    Mac,
    IpadLandscape,
    IpadPortrait,
    IpadSplit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RightMode {
    Closed,
    Docked,
    Drawer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    pub max_columns: u8,
    pub left_docked: bool,
    pub right: RightMode,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageSize {
    pub w: f64,
    pub h: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Center {
    Reading,
    Library,
    Agents,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CenterAction {
    NowReading,
    Library,
    Agents,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    Margin,
    Card,
    Quote,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Link {
    Highlight,
    Hairline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Thread {
    Plain,
    Nested,
    Ruled,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct Note {
    pub id: u8,
    pub line: Line,
    pub body: String,
    pub reply: String,
}

#[derive(Clone, Copy, Debug)]
pub struct AnchorCard {
    pub id: u8,
    pub line_y: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct PlacedCard {
    pub id: u8,
    pub y: f64,
    pub h: f64,
    pub snapped: bool,
}

#[derive(Clone, Debug)]
pub struct Geometry {
    pub layout: Layout,
    pub page: Rect,
    pub sheet: Option<Rect>,
    pub left: Option<Rect>,
    pub right: Option<Rect>,
    pub drawer: Option<Rect>,
}

pub fn enabled() -> bool {
    flag("SPIKE_PANELS")
}

pub fn hover_forced() -> bool {
    flag("SPIKE_PANELS_HOVER")
}

pub fn crowd() -> bool {
    flag("SPIKE_PANELS_CROWD")
}

pub fn bare() -> bool {
    flag("SPIKE_PANELS_BARE")
}

pub fn window_size() -> (f64, f64) {
    (env_f64("SPIKE_PANELS_W", 1440.0), env_f64("SPIKE_PANELS_H", 920.0))
}

pub fn choice_from_env() -> Choice {
    Choice {
        left_open: flag("SPIKE_PANELS_LEFT"),
        right_open: flag("SPIKE_PANELS_RIGHT"),
    }
}

pub fn variant_from_env() -> Variant {
    match std::env::var("SPIKE_PANELS_VARIANT").as_deref() {
        Ok("margin") => Variant::Margin,
        Ok("quote") => Variant::Quote,
        _ => Variant::Card,
    }
}

pub fn surface_from_env() -> Surface {
    match std::env::var("SPIKE_PANELS_SURFACE").as_deref() {
        Ok("ipad-landscape") => Surface::IpadLandscape,
        Ok("ipad-portrait") => Surface::IpadPortrait,
        Ok("ipad-split") => Surface::IpadSplit,
        _ => Surface::Mac,
    }
}

pub fn plate_path() -> String {
    std::env::var("SPIKE_PANELS_PLATE").unwrap_or_default()
}

pub fn lines_path() -> String {
    std::env::var("SPIKE_PANELS_LINES").unwrap_or_default()
}

pub fn export_path() -> String {
    std::env::var("SPIKE_PANELS_EXPORT").unwrap_or_default()
}

pub fn variant_named(name: &str) -> Option<Variant> {
    match name {
        "margin" => Some(Variant::Margin),
        "card" => Some(Variant::Card),
        "quote" => Some(Variant::Quote),
        _ => None,
    }
}

impl Variant {
    pub fn link(self) -> Link {
        match self {
            Variant::Card => Link::Hairline,
            Variant::Margin | Variant::Quote => Link::Highlight,
        }
    }

    pub fn thread(self) -> Thread {
        match self {
            Variant::Margin => Thread::Plain,
            Variant::Card => Thread::Nested,
            Variant::Quote => Thread::Ruled,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Variant::Margin => "margin",
            Variant::Card => "card",
            Variant::Quote => "quote",
        }
    }
}

pub fn card_metrics(variant: Variant) -> (f64, f64) {
    let expanded = match variant {
        Variant::Margin => 128.0,
        Variant::Card => 168.0,
        Variant::Quote => 196.0,
    };
    (48.0, expanded)
}

pub fn max_columns(surface: Surface, width: f64) -> u8 {
    match surface {
        Surface::IpadLandscape => 3,
        Surface::IpadPortrait => 2,
        Surface::IpadSplit => 1,
        Surface::Mac if width >= MAC_THREE => 3,
        Surface::Mac if width >= MAC_TWO => 2,
        Surface::Mac => 1,
    }
}

pub fn resolve(choice: Choice, surface: Surface, width: f64) -> Layout {
    let max_columns = max_columns(surface, width);
    let slots = max_columns.saturating_sub(1);
    let right = if !choice.right_open {
        RightMode::Closed
    } else if slots >= 1 {
        RightMode::Docked
    } else {
        RightMode::Drawer
    };
    let used = u8::from(right == RightMode::Docked);
    Layout {
        max_columns,
        left_docked: choice.left_open && slots > used,
        right,
    }
}

pub fn apply_center(_current: Center, action: CenterAction) -> Center {
    match action {
        CenterAction::NowReading => Center::Reading,
        CenterAction::Library => Center::Library,
        CenterAction::Agents => Center::Agents,
    }
}

pub fn design_page(ratio_w: f64, ratio_h: f64) -> PageSize {
    let ratio_w = if ratio_w <= 0.0 { 129.0 } else { ratio_w };
    let ratio_h = if ratio_h <= 0.0 { 198.0 } else { ratio_h };
    PageSize {
        w: DESIGN_W,
        h: DESIGN_W * ratio_h / ratio_w,
    }
}

pub fn geometry(
    window_w: f64,
    window_h: f64,
    design: PageSize,
    choice: Choice,
    surface: Surface,
) -> Geometry {
    let layout = resolve(choice, surface, window_w);
    let inset = layout.left_docked || layout.right == RightMode::Docked;
    let left_w = if layout.left_docked {
        (window_w * BAR_SHARE).max(LEFT_W)
    } else {
        0.0
    };
    let rest = (window_w - left_w).max(1.0);
    let page_col = if layout.right == RightMode::Docked {
        page_column(rest, design.w)
    } else {
        rest
    };
    let right_w = if layout.right == RightMode::Docked {
        (rest - page_col).max(1.0)
    } else {
        0.0
    };
    let (page, sheet) = if inset {
        let sheet = Rect {
            x: left_w + SHEET_GAP,
            y: SHEET_GAP,
            w: (page_col - SHEET_GAP * 2.0).max(1.0),
            h: (window_h - SHEET_GAP * 2.0).max(1.0),
        };
        (fit_page(sheet, design), Some(sheet))
    } else {
        let slot = Rect {
            x: 0.0,
            y: TOP_MAT,
            w: window_w.max(1.0),
            h: (window_h - TOP_MAT).max(1.0),
        };
        (fit_page(slot, design), None)
    };
    let left = if layout.left_docked {
        Some(Rect {
            x: 0.0,
            y: 0.0,
            w: left_w,
            h: window_h,
        })
    } else {
        None
    };
    let right = if layout.right == RightMode::Docked {
        Some(Rect {
            x: window_w - right_w,
            y: 0.0,
            w: right_w,
            h: window_h,
        })
    } else {
        None
    };
    let drawer = if layout.right == RightMode::Drawer {
        Some(Rect {
            x: window_w - RIGHT_W - 16.0,
            y: TOP_MAT + 12.0,
            w: RIGHT_W,
            h: (window_h - TOP_MAT - 28.0).max(1.0),
        })
    } else {
        None
    };
    Geometry {
        layout,
        page,
        sheet,
        left,
        right,
        drawer,
    }
}

fn page_column(rest: f64, design_w: f64) -> f64 {
    let natural = rest * PAGE_SHARE;
    let needed = design_w + SHEET_GAP * 2.0;
    if rest >= needed + PANEL_FLOOR {
        natural.max(needed)
    } else {
        natural
    }
}

fn fit_page(slot: Rect, design: PageSize) -> Rect {
    let scale = (slot.w / design.w).min(slot.h / design.h).min(1.0);
    let page_w = design.w * scale;
    let page_h = design.h * scale;
    Rect {
        x: slot.x + (slot.w - page_w) / 2.0,
        y: slot.y + (slot.h - page_h) / 2.0,
        w: page_w,
        h: page_h,
    }
}

pub fn place_cards(
    cards: &[AnchorCard],
    selected: u8,
    collapsed: f64,
    expanded: f64,
    gap: f64,
    ceiling: f64,
) -> Vec<PlacedCard> {
    let mut items: Vec<PlacedCard> = cards
        .iter()
        .map(|card| {
            let snapped = card.id == selected;
            PlacedCard {
                id: card.id,
                y: card.line_y.max(ceiling),
                h: if snapped { expanded } else { collapsed },
                snapped,
            }
        })
        .collect();
    items.sort_by(|a, b| a.y.total_cmp(&b.y).then(a.id.cmp(&b.id)));
    let Some(sel_at) = items.iter().position(|item| item.snapped) else {
        flow_down(&mut items, gap);
        return items;
    };
    let selected_card = items[sel_at];
    let mut above = items[..sel_at].to_vec();
    let mut below = items[sel_at + 1..].to_vec();
    let mut next_top = selected_card.y - gap;
    for card in above.iter_mut().rev() {
        if card.y + card.h > next_top {
            card.y = next_top - card.h;
        }
        if card.y < ceiling {
            card.y = ceiling;
        }
        next_top = card.y - gap;
    }
    let mut stay = Vec::new();
    for card in above {
        if card.y + card.h + gap > selected_card.y + 0.1 {
            below.insert(0, card);
        } else {
            stay.push(card);
        }
    }
    flow_down(&mut stay, gap);
    let mut still = Vec::new();
    for card in stay {
        if card.y + card.h + gap > selected_card.y + 0.1 {
            below.insert(0, card);
        } else {
            still.push(card);
        }
    }
    let mut cursor = selected_card.y + selected_card.h + gap;
    for card in &mut below {
        if card.y < cursor {
            card.y = cursor;
        }
        cursor = card.y + card.h + gap;
    }
    let mut out = still;
    out.push(selected_card);
    out.extend(below);
    out
}

fn flow_down(items: &mut [PlacedCard], gap: f64) {
    let mut cursor = f64::NEG_INFINITY;
    for item in items.iter_mut() {
        if item.y < cursor {
            item.y = cursor;
        }
        cursor = item.y + item.h + gap;
    }
}

pub fn pick_lines(lines: &[Line], crowd: bool) -> Vec<Line> {
    if lines.is_empty() {
        return Vec::new();
    }
    if crowd {
        let start = if lines.len() > 6 { 2 } else { 0 };
        let end = (start + 5).min(lines.len());
        return lines[start..end].to_vec();
    }
    let indexes: Vec<usize> = if lines.len() >= 13 {
        vec![1, 6, 12]
    } else if lines.len() >= 3 {
        vec![1, lines.len() / 2, lines.len() - 2]
    } else {
        (0..lines.len()).collect()
    };
    let mut out = Vec::new();
    let mut seen = Vec::new();
    for index in indexes {
        if seen.contains(&index) {
            continue;
        }
        if let Some(line) = lines.get(index) {
            seen.push(index);
            out.push(line.clone());
        }
    }
    if out.is_empty() {
        out.push(lines[0].clone());
    }
    out
}

pub fn line_text_repeats(text: &str) -> bool {
    let head: String = text.chars().take(18).collect();
    if head.chars().count() < 18 {
        return false;
    }
    text.matches(head.as_str()).count() > 1
}

pub fn lines_plausible(lines: &[Line], page_w: f64) -> bool {
    if lines.len() < 3 || page_w <= 1.0 {
        return false;
    }
    let wide = lines.iter().any(|line| line.w > page_w * 0.45);
    let clean = lines.iter().all(|line| !line_text_repeats(&line.text));
    wide && clean
}

pub fn notes_for(chinese: bool, lines: &[Line]) -> Vec<Note> {
    let bodies: &[&str] = if chinese {
        &[
            "示例。这一句是回忆的入口。",
            "示例。园子比人事更清楚。",
            "示例。他在写离开，也在写留下。",
            "示例。花是给记忆用的。",
            "示例。句子很短，时间很长。",
        ]
    } else {
        &[
            "Sample. The joke is the premise.",
            "Sample. She is counting, not admiring.",
            "Sample. The room does the talking.",
            "Sample. Manners, used as strategy.",
            "Sample. The sentence smiles and cuts.",
        ]
    };
    let reply = if chinese {
        "@alan  后面几段都在给这一句找证据。"
    } else {
        "@alan  The chapter keeps the promise this line makes."
    };
    lines
        .iter()
        .enumerate()
        .map(|(index, line)| Note {
            id: index as u8,
            line: line.clone(),
            body: bodies[index % bodies.len()].to_string(),
            reply: reply.to_string(),
        })
        .collect()
}

pub fn scale_lines(lines: &[Line], from: PageSize, to: PageSize) -> Vec<Line> {
    if from.w <= 0.0 || from.h <= 0.0 {
        return lines.to_vec();
    }
    let sx = to.w / from.w;
    let sy = to.h / from.h;
    lines
        .iter()
        .map(|line| Line {
            x: line.x * sx,
            y: line.y * sy,
            w: line.w * sx,
            h: line.h * sy,
            text: line.text.clone(),
        })
        .collect()
}

pub fn load_lines(path: &str) -> Option<(PageSize, Vec<Line>)> {
    let text = std::fs::read_to_string(path).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    let page = PageSize {
        w: value["page_w"].as_f64()?,
        h: value["page_h"].as_f64()?,
    };
    let lines = value["lines"].as_array()?.iter().filter_map(line_from_json).collect();
    Some((page, lines))
}

pub fn line_from_json(value: &Value) -> Option<Line> {
    Some(Line {
        x: value["x"].as_f64()?,
        y: value["y"].as_f64()?,
        w: value["w"].as_f64()?,
        h: value["h"].as_f64()?,
        text: value["text"].as_str().unwrap_or("").to_string(),
    })
}

pub fn right_name(mode: RightMode) -> &'static str {
    match mode {
        RightMode::Closed => "closed",
        RightMode::Docked => "docked",
        RightMode::Drawer => "drawer",
    }
}

fn flag(name: &str) -> bool {
    std::env::var(name).ok().as_deref() == Some("1")
}

fn env_f64(name: &str, fallback: f64) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|raw| raw.parse().ok())
        .filter(|value: &f64| value.is_finite() && *value > 1.0)
        .unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn both() -> Choice {
        Choice {
            left_open: true,
            right_open: true,
        }
    }

    fn design() -> PageSize {
        design_page(129.0, 198.0)
    }

    fn close(got: f64, want: f64) {
        assert!((got - want).abs() < 0.05, "{got} vs {want}");
    }

    #[test]
    fn narrowing_closes_the_left_bar_then_the_right_panel_becomes_a_drawer() {
        let choice = both();
        let wide = resolve(choice, Surface::Mac, 1440.0);
        let medium = resolve(choice, Surface::Mac, 1120.0);
        let narrow = resolve(choice, Surface::Mac, 780.0);
        let restored = resolve(choice, Surface::Mac, 1440.0);
        assert_eq!(wide.max_columns, 3);
        assert!(wide.left_docked);
        assert_eq!(wide.right, RightMode::Docked);
        assert_eq!(medium.max_columns, 2);
        assert!(!medium.left_docked);
        assert_eq!(medium.right, RightMode::Docked);
        assert_eq!(narrow.max_columns, 1);
        assert!(!narrow.left_docked);
        assert_eq!(narrow.right, RightMode::Drawer);
        assert_eq!(restored, wide);
    }

    #[test]
    fn a_closed_panel_stays_closed_on_a_wide_window() {
        let choice = Choice {
            left_open: false,
            right_open: false,
        };
        let wide = resolve(choice, Surface::Mac, 2000.0);
        assert_eq!(wide.max_columns, 3);
        assert!(!wide.left_docked);
        assert_eq!(wide.right, RightMode::Closed);
    }

    #[test]
    fn the_left_bar_can_dock_when_the_right_panel_is_closed() {
        let choice = Choice {
            left_open: true,
            right_open: false,
        };
        let medium = resolve(choice, Surface::Mac, 1120.0);
        assert!(medium.left_docked);
        assert_eq!(medium.right, RightMode::Closed);
    }

    #[test]
    fn ipad_class_sets_the_max_even_when_the_width_would_not() {
        let choice = both();
        assert_eq!(resolve(choice, Surface::Mac, 1180.0).max_columns, 2);
        let land = resolve(choice, Surface::IpadLandscape, 1180.0);
        assert_eq!(land.max_columns, 3);
        assert!(land.left_docked);
        assert_eq!(land.right, RightMode::Docked);
        let portrait = resolve(choice, Surface::IpadPortrait, 820.0);
        assert_eq!(portrait.max_columns, 2);
        assert!(!portrait.left_docked);
        assert_eq!(portrait.right, RightMode::Docked);
        assert_eq!(
            resolve(choice, Surface::Mac, 820.0).right,
            RightMode::Drawer
        );
        let split = resolve(choice, Surface::IpadSplit, 375.0);
        assert_eq!(split.max_columns, 1);
        assert_eq!(split.right, RightMode::Drawer);
        assert!(!split.left_docked);
    }

    #[test]
    fn opening_a_panel_recenters_the_same_page() {
        let design = design();
        let shut = geometry(
            1440.0,
            920.0,
            design,
            Choice {
                left_open: false,
                right_open: false,
            },
            Surface::Mac,
        );
        let open = geometry(
            1440.0,
            920.0,
            design,
            Choice {
                left_open: false,
                right_open: true,
            },
            Surface::Mac,
        );
        close(open.page.w, shut.page.w);
        close(open.page.h, shut.page.h);
        close(open.page.w, design.w);
        assert!(open.page.x < shut.page.x);
    }

    #[test]
    fn the_mac_sequence_keeps_one_page_size() {
        let design = design();
        for width in [1440.0, 1120.0, 780.0] {
            let geo = geometry(width, 920.0, design, both(), Surface::Mac);
            close(geo.page.w, design.w);
            close(geo.page.h, design.h);
        }
    }

    #[test]
    fn a_drawer_does_not_move_the_page() {
        let design = design();
        let bare = geometry(
            780.0,
            920.0,
            design,
            Choice {
                left_open: false,
                right_open: false,
            },
            Surface::Mac,
        );
        let drawer = geometry(780.0, 920.0, design, both(), Surface::Mac);
        close(drawer.page.x, bare.page.x);
        close(drawer.page.w, bare.page.w);
        assert!(drawer.drawer.is_some());
        assert!(bare.drawer.is_none());
    }

    #[test]
    fn a_split_window_scales_the_page_and_keeps_the_trim() {
        let design = design();
        let geo = geometry(375.0, 812.0, design, both(), Surface::IpadSplit);
        assert!(geo.page.w < design.w - 1.0);
        let want = design.w / design.h;
        let got = geo.page.w / geo.page.h;
        assert!((got - want).abs() < 0.001, "{got} vs {want}");
        assert!(geo.page.w <= 375.0 + 0.1);
    }

    #[test]
    fn ipad_portrait_and_landscape_fit_the_design_page() {
        let design = design();
        let land = geometry(1180.0, 820.0, design, both(), Surface::IpadLandscape);
        let portrait = geometry(820.0, 1180.0, design, both(), Surface::IpadPortrait);
        close(land.page.w, design.w);
        close(land.page.h, design.h);
        close(portrait.page.w, design.w);
        close(portrait.page.h, design.h);
        assert!(land.sheet.is_some());
        assert!(portrait.sheet.is_some());
        let split = geometry(375.0, 812.0, design, both(), Surface::IpadSplit);
        assert!(split.sheet.is_none());
    }

    #[test]
    fn a_docked_page_is_an_inset_sheet_and_the_immersive_page_is_flush() {
        let design = design();
        let companion = geometry(
            1204.0,
            830.0,
            design,
            Choice {
                left_open: false,
                right_open: true,
            },
            Surface::Mac,
        );
        let sheet = companion.sheet.expect("companion sheet");
        close(sheet.x, SHEET_GAP);
        close(sheet.y, SHEET_GAP);
        close(sheet.y + sheet.h, 830.0 - SHEET_GAP);
        let panel = companion.right.expect("panel");
        close(sheet.x + sheet.w + SHEET_GAP, panel.x);
        let share = panel.w / 1204.0;
        assert!((share - (1.0 - PAGE_SHARE)).abs() < 0.02, "{share}");
        close(companion.page.w, design.w);
        assert!(companion.page.x >= sheet.x - 0.1);
        assert!(companion.page.x + companion.page.w <= sheet.x + sheet.w + 0.1);
        let side = companion.page.x - sheet.x;
        assert!(side > SHEET_RADIUS, "sheet corners stay outside the page, {side}");

        let full = geometry(1280.0, 830.0, design, both(), Surface::Mac);
        let bar = full.left.expect("bar");
        let bar_share = bar.w / 1280.0;
        assert!((bar_share - BAR_SHARE).abs() < 0.001, "{bar_share}");
        close(bar.y, 0.0);
        close(bar.h, 830.0);
        assert!(full.sheet.is_some());
        close(full.page.w, design.w);

        let shut = geometry(
            680.0,
            800.0,
            design,
            Choice {
                left_open: false,
                right_open: false,
            },
            Surface::Mac,
        );
        assert!(shut.sheet.is_none());
        let drawer = geometry(680.0, 800.0, design, both(), Surface::Mac);
        assert!(drawer.sheet.is_none());
        close(drawer.page.x, shut.page.x);
        close(drawer.page.w, shut.page.w);
    }

    #[test]
    fn a_selected_card_stays_on_its_line_and_the_crowd_flows_down() {
        let cards = [
            AnchorCard { id: 0, line_y: 100.0 },
            AnchorCard { id: 1, line_y: 120.0 },
            AnchorCard { id: 2, line_y: 140.0 },
        ];
        let placed = place_cards(&cards, 1, 36.0, 140.0, 8.0, 0.0);
        let at = |id: u8| placed.iter().find(|item| item.id == id).unwrap();
        close(at(1).y, 120.0);
        assert!(at(1).snapped);
        close(at(0).y, 76.0);
        close(at(2).y, 268.0);
    }

    #[test]
    fn a_card_that_cannot_sit_above_the_selection_flows_below_it() {
        let cards = [
            AnchorCard { id: 0, line_y: 10.0 },
            AnchorCard { id: 1, line_y: 40.0 },
        ];
        let placed = place_cards(&cards, 1, 36.0, 100.0, 8.0, 0.0);
        let at = |id: u8| placed.iter().find(|item| item.id == id).unwrap();
        close(at(1).y, 40.0);
        close(at(0).y, 148.0);
    }

    #[test]
    fn separated_cards_stay_on_their_lines() {
        let cards = [
            AnchorCard { id: 0, line_y: 100.0 },
            AnchorCard { id: 1, line_y: 200.0 },
            AnchorCard { id: 2, line_y: 400.0 },
        ];
        let placed = place_cards(&cards, 1, 36.0, 120.0, 8.0, 0.0);
        let at = |id: u8| placed.iter().find(|item| item.id == id).unwrap();
        close(at(0).y, 100.0);
        close(at(1).y, 200.0);
        close(at(2).y, 400.0);
    }

    #[test]
    fn crowded_picks_are_consecutive_and_sparse_picks_are_not() {
        let lines: Vec<Line> = (0..20)
            .map(|index| Line {
                x: 0.0,
                y: f64::from(index) * 20.0,
                w: 100.0,
                h: 16.0,
                text: format!("line {index}"),
            })
            .collect();
        let crowded = pick_lines(&lines, true);
        assert_eq!(crowded.len(), 5);
        close(crowded[1].y - crowded[0].y, 20.0);
        let sparse = pick_lines(&lines, false);
        assert_eq!(sparse.len(), 3);
        assert!(sparse[1].y > sparse[0].y + 40.0);
        let short = pick_lines(&lines[..9], false);
        assert_eq!(short.len(), 3);
        assert!(short[1].y > short[0].y + 20.0);
        assert!(short[2].y > short[1].y + 20.0);
    }

    #[test]
    fn repeated_line_text_is_not_a_real_line() {
        let junk = "It is a truth univer It is a truth univer";
        assert!(line_text_repeats(junk));
        assert!(!line_text_repeats("It is a truth universally acknowledged"));
        let bad = vec![
            Line { x: 64.0, y: 10.0, w: 200.0, h: 30.0, text: junk.into() },
            Line { x: 64.0, y: 42.0, w: 300.0, h: 30.0, text: junk.into() },
            Line { x: 64.0, y: 74.0, w: 280.0, h: 30.0, text: junk.into() },
        ];
        assert!(!lines_plausible(&bad, 460.0));
        let good: Vec<Line> = (0..4)
            .map(|index| Line {
                x: 64.0,
                y: f64::from(index) * 32.0,
                w: 320.0,
                h: 30.0,
                text: format!("a real line of body text {index}"),
            })
            .collect();
        assert!(lines_plausible(&good, 460.0));
    }

    #[test]
    fn sample_notes_name_themselves_and_alan() {
        let lines = vec![Line {
            x: 0.0,
            y: 10.0,
            w: 40.0,
            h: 16.0,
            text: "It is a truth".into(),
        }];
        let english = notes_for(false, &lines);
        assert!(english[0].body.starts_with("Sample."));
        assert!(english[0].reply.contains("@alan"));
        let chinese = notes_for(true, &lines);
        assert!(chinese[0].body.starts_with("示例"));
        assert!(chinese[0].reply.contains("@alan"));
    }

    #[test]
    fn the_three_card_variants_differ_in_link_and_thread() {
        assert_eq!(Variant::Margin.link(), Link::Highlight);
        assert_eq!(Variant::Card.link(), Link::Hairline);
        assert_eq!(Variant::Quote.link(), Link::Highlight);
        assert_eq!(Variant::Margin.thread(), Thread::Plain);
        assert_eq!(Variant::Card.thread(), Thread::Nested);
        assert_eq!(Variant::Quote.thread(), Thread::Ruled);
        assert_ne!(Variant::Margin.link(), Variant::Card.link());
        assert_ne!(Variant::Card.thread(), Variant::Quote.thread());
    }

    #[test]
    fn now_reading_returns_to_the_page_after_another_function() {
        let library = apply_center(Center::Reading, CenterAction::Library);
        assert_eq!(library, Center::Library);
        assert_eq!(
            apply_center(library, CenterAction::NowReading),
            Center::Reading
        );
        assert_eq!(
            apply_center(Center::Reading, CenterAction::Agents),
            Center::Agents
        );
    }

    #[test]
    fn line_positions_scale_with_the_page() {
        let lines = vec![Line {
            x: 10.0,
            y: 20.0,
            w: 30.0,
            h: 4.0,
            text: "a".into(),
        }];
        let scaled = scale_lines(
            &lines,
            PageSize { w: 100.0, h: 200.0 },
            PageSize { w: 50.0, h: 100.0 },
        );
        close(scaled[0].x, 5.0);
        close(scaled[0].y, 10.0);
        close(scaled[0].w, 15.0);
        assert_eq!(scaled[0].text, "a");
    }
}
