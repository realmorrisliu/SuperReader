pub const INSET: f64 = 6.0;
pub const RADIUS: f64 = 9.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Sidebar,
    BelowTitle,
    Paper,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    pub card: Rect,
    pub page: Rect,
    pub inset: f64,
    pub radius: f64,
    pub field: Field,
}

pub fn immersive_label(on: bool) -> &'static str {
    if on {
        "Exit Immersive Reading"
    } else {
        "Enter Immersive Reading"
    }
}

pub fn content_region(stage_w: f64, stage_h: f64, bar: f64, title_band: f64) -> Rect {
    let left = if bar > 1.0 { bar } else { 0.0 };
    Rect {
        x: left,
        y: title_band,
        w: (stage_w - left).max(1.0),
        h: (stage_h - title_band).max(1.0),
    }
}

pub fn windowed_card(stage_w: f64, stage_h: f64, bar: f64, title_band: f64) -> Rect {
    let region = content_region(stage_w, stage_h, bar, title_band);
    Rect {
        x: region.x + INSET,
        y: region.y + INSET,
        w: (region.w - INSET * 2.0).max(1.0),
        h: (region.h - INSET * 2.0).max(1.0),
    }
}

pub fn immersive_card(stage_w: f64, stage_h: f64) -> Rect {
    Rect {
        x: 0.0,
        y: 0.0,
        w: stage_w.max(1.0),
        h: stage_h.max(1.0),
    }
}

pub fn page_in_card(card: Rect, ratio_w: f64, ratio_h: f64) -> Rect {
    let (ix, iy, w, h) = fit_inside(card.w, card.h, ratio_w, ratio_h);
    Rect {
        x: card.x + ix,
        y: card.y + iy,
        w,
        h,
    }
}

pub fn layout(
    stage_w: f64,
    stage_h: f64,
    bar: f64,
    title_band: f64,
    ratio_w: f64,
    ratio_h: f64,
    immersive: bool,
    fullscreen: bool,
    fullscreen_page: Rect,
) -> Layout {
    if immersive {
        let card = immersive_card(stage_w, stage_h);
        return Layout {
            page: page_in_card(card, ratio_w, ratio_h),
            card,
            inset: 0.0,
            radius: 0.0,
            field: Field::Paper,
        };
    }
    if fullscreen {
        return Layout {
            card: fullscreen_page,
            page: fullscreen_page,
            inset: 0.0,
            radius: 0.0,
            field: Field::Sidebar,
        };
    }
    let card = windowed_card(stage_w, stage_h, bar, title_band);
    Layout {
        page: page_in_card(card, ratio_w, ratio_h),
        card,
        inset: INSET,
        radius: RADIUS,
        field: Field::BelowTitle,
    }
}

pub fn unflipped_child_y(card_h: f64, from_top: f64, page_h: f64) -> f64 {
    card_h - from_top - page_h
}

pub fn insets(card: Rect, region: Rect) -> (f64, f64, f64, f64) {
    let left = card.x - region.x;
    let top = card.y - region.y;
    let right = (region.x + region.w) - (card.x + card.w);
    let bottom = (region.y + region.h) - (card.y + card.h);
    (left, top, right, bottom)
}

fn fit_inside(box_w: f64, box_h: f64, ratio_w: f64, ratio_h: f64) -> (f64, f64, f64, f64) {
    let box_w = box_w.max(1.0);
    let box_h = box_h.max(1.0);
    let ratio_w = ratio_w.max(1.0);
    let ratio_h = ratio_h.max(1.0);
    let page = ratio_w / ratio_h;
    let (width, height) = if box_w / box_h > page {
        (box_h * page, box_h)
    } else {
        (box_w, box_w / page)
    };
    (
        ((box_w - width) / 2.0).max(0.0),
        ((box_h - height) / 2.0).max(0.0),
        width,
        height,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn near(got: f64, want: f64) {
        assert!((got - want).abs() < 0.01, "{got} vs {want}");
    }

    #[test]
    fn the_windowed_card_is_the_content_region_inset_six_on_every_side() {
        let stage_w = 1100.0;
        let stage_h = 830.0;
        let bar = 98.0;
        let title = 48.0;
        let card = windowed_card(stage_w, stage_h, bar, title);
        let region = content_region(stage_w, stage_h, bar, title);
        let (left, top, right, bottom) = insets(card, region);
        near(left, 6.0);
        near(top, 6.0);
        near(right, 6.0);
        near(bottom, 6.0);
        near(card.w, stage_w - bar - 12.0);
        near(card.h, stage_h - title - 12.0);
    }

    #[test]
    fn an_outset_around_the_page_is_not_this_card() {
        let card = windowed_card(1100.0, 830.0, 98.0, 48.0);
        let page = page_in_card(card, 129.0, 198.0);
        let outset_w = page.w + 12.0;
        assert!(
            card.w > outset_w + 100.0,
            "card {0} is still a sleeve around the page {outset_w}",
            card.w
        );
        assert!(page.x > card.x + 6.0);
        assert!(page.x + page.w < card.x + card.w - 6.0);
        near(page.y, card.y);
        near(page.h, card.h);
    }

    #[test]
    fn immersive_reading_is_the_stage_with_no_inset() {
        let stage_w = 1100.0;
        let stage_h = 830.0;
        let laid = layout(
            stage_w,
            stage_h,
            98.0,
            48.0,
            129.0,
            198.0,
            true,
            false,
            Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            },
        );
        assert_eq!(laid.field, Field::Paper);
        near(laid.inset, 0.0);
        near(laid.card.x, 0.0);
        near(laid.card.y, 0.0);
        near(laid.card.w, stage_w);
        near(laid.card.h, stage_h);
        near(laid.page.y, 0.0);
        near(laid.page.h, stage_h);
        let windowed = layout(
            stage_w,
            stage_h,
            98.0,
            48.0,
            129.0,
            198.0,
            false,
            false,
            Rect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 1.0,
            },
        );
        assert!(
            laid.page.w > windowed.page.w + 20.0,
            "immersive page {} stayed the windowed width {}",
            laid.page.w,
            windowed.page.w
        );
    }

    #[test]
    fn fullscreen_keeps_the_page_it_was_given() {
        let page = Rect {
            x: 200.0,
            y: 48.0,
            w: 640.0,
            h: 900.0,
        };
        let laid = layout(1400.0, 1000.0, 98.0, 48.0, 129.0, 198.0, false, true, page);
        assert_eq!(laid.field, Field::Sidebar);
        assert_eq!(laid.page, page);
        assert_eq!(laid.card, page);
        near(laid.inset, 0.0);
    }

    #[test]
    fn an_unflipped_card_puts_the_page_top_at_from_top() {
        near(unflipped_child_y(770.0, 0.0, 770.0), 0.0);
        near(unflipped_child_y(770.0, 10.0, 400.0), 360.0);
    }

    #[test]
    fn the_menu_title_follows_the_mode() {
        assert_eq!(immersive_label(false), "Enter Immersive Reading");
        assert_eq!(immersive_label(true), "Exit Immersive Reading");
    }
}
