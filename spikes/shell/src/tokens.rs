//! Full-form measures from the approved B mockup stylesheet.
//! Paint reads these constants. A second copy of a size does not belong in a view.

use nsh_core::color::rgba;
use nsh_core::prelude::*;
use nsh_core::BoxShadow;

pub const FONT_STACK: &str =
    "-apple-system, BlinkMacSystemFont, PingFang SC, Helvetica Neue, sans-serif";

pub const INK: u32 = 0x001D_1D1F;
pub const QUIET: u32 = 0x008E_8E93;
pub const ICON: u32 = 0x006E_6E73;
pub const FAINT: u32 = 0x00A1_A1A6;
pub const CHIP_INK: u32 = 0x003A_3A3F;
pub const CITE: u32 = 0x0055_555B;
pub const FIELD_TOP: u32 = 0x00EE_EEF0;
pub const FIELD_BOTTOM: u32 = 0x00E9_E9EC;
pub const CARD: u32 = 0x00FF_FFFF;
pub const CONTINUE_BG: u32 = 0x00F5_F5F7;
pub const SCOPE_BG: u32 = 0x00F2_F2F4;
pub const SEND_BG: u32 = 0x00D8_D8DD;
pub const AVATAR_FROM: u32 = 0x00F0_B267;
pub const AVATAR_TO: u32 = 0x00D2_693C;
pub const APPLE_FROM: u32 = 0x00FF_A53D;
pub const APPLE_TO: u32 = 0x00F2_7A1A;
pub const CALIBRE_FROM: u32 = 0x006B_5A4A;
pub const CALIBRE_TO: u32 = 0x004A_3C30;
pub const LIST_WANT: u32 = 0x00E0_773A;
pub const LIST_LU: u32 = 0x005B_8DEF;
pub const LIST_EN: u32 = 0x007D_8C6A;

pub const BODY_TRACK_EM: f32 = -0.005;
pub const H1_TRACK_EM: f32 = -0.01;
pub const LABEL_TRACK_EM: f32 = 0.04;
pub const TAG_TRACK_EM: f32 = 0.04;

pub const H1_SIZE: f32 = 26.0;
pub const H1_WEIGHT: FontWeight = FontWeight::BOLD;
pub const COUNT_SIZE: f32 = 13.0;
pub const COUNT_WEIGHT: FontWeight = FontWeight::NORMAL;
pub const SECTION_SIZE: f32 = 15.0;
pub const SECTION_WEIGHT: FontWeight = FontWeight::SEMIBOLD;
pub const SECTION_NOTE_SIZE: f32 = 12.0;
pub const BODY_SIZE: f32 = 13.0;
pub const BODY_WEIGHT: FontWeight = FontWeight::NORMAL;
pub const ROW_WEIGHT_ON: FontWeight = FontWeight::SEMIBOLD;
pub const SEARCH_SIZE: f32 = 12.5;
pub const LABEL_SIZE: f32 = 11.0;
pub const LABEL_WEIGHT: FontWeight = FontWeight::SEMIBOLD;
pub const COUNT_ROW_SIZE: f32 = 12.0;
pub const TOOL_SIZE: f32 = 12.5;
pub const SHELF_TITLE_SIZE: f32 = 12.5;
pub const SHELF_TITLE_WEIGHT: FontWeight = FontWeight::MEDIUM;
pub const SHELF_META_SIZE: f32 = 11.5;
pub const CONTINUE_TITLE_SIZE: f32 = 13.0;
pub const CONTINUE_TITLE_WEIGHT: FontWeight = FontWeight::SEMIBOLD;
pub const CONTINUE_META_SIZE: f32 = 12.0;
pub const PROGRESS_SIZE: f32 = 11.0;
pub const TIME_SIZE: f32 = 11.0;
pub const TAG_SIZE: f32 = 9.0;
pub const TAG_SM_SIZE: f32 = 8.5;
pub const TAG_WEIGHT: FontWeight = FontWeight::SEMIBOLD;
pub const NAME_SIZE: f32 = 13.0;
pub const NAME_WEIGHT: FontWeight = FontWeight::SEMIBOLD;
pub const SUB_SIZE: f32 = 11.5;
pub const AVATAR_MARK_SIZE: f32 = 13.0;
pub const AVATAR_MARK_WEIGHT: FontWeight = FontWeight::BOLD;
pub const MSG_SIZE: f32 = 13.0;
pub const CHIP_SIZE: f32 = 12.0;
pub const CITE_SIZE: f32 = 11.5;
pub const TIME_RULE_SIZE: f32 = 11.0;

pub const SIDEBAR_W: f64 = 220.0;
pub const CHAT_W: f64 = 320.0;
pub const INSET: f64 = 6.0;
pub const CARD_RADIUS: f64 = 12.0;
pub const WINDOW_W: f64 = 1440.0;
pub const WINDOW_H: f64 = 900.0;

pub const SIDE_PAD_X: f32 = 10.0;
pub const SIDE_PAD_TOP: f32 = 52.0;
pub const SIDE_PAD_BOTTOM: f32 = 12.0;
pub const TOP_ROW_H: f32 = 52.0;
pub const TOGGLE_SIZE: f32 = 28.0;
pub const TOGGLE_RADIUS: f32 = 7.0;
pub const TOGGLE_TOP: f32 = 12.0;
pub const SEARCH_H: f32 = 28.0;
pub const SEARCH_RADIUS: f32 = 8.0;
pub const SEARCH_PAD_X: f32 = 8.0;
pub const SEARCH_GAP: f32 = 6.0;
pub const SEARCH_MB: f32 = 8.0;
pub const LABEL_H: f32 = 16.0;
pub const LABEL_MT: f32 = 14.0;
pub const LABEL_MB: f32 = 4.0;
pub const LABEL_PX: f32 = 10.0;
pub const ROW_H: f32 = 28.0;
pub const ROW_RADIUS: f32 = 7.0;
pub const ROW_PAD_L: f32 = 8.0;
pub const ROW_PAD_R: f32 = 10.0;
pub const ROW_GAP: f32 = 8.0;
pub const ROW_ICON_SLOT: f32 = 18.0;
pub const ROW_ICON: f32 = 16.0;
pub const DOT: f32 = 9.0;
pub const SOURCE_BADGE: f32 = 16.0;
pub const SOURCE_BADGE_RADIUS: f32 = 4.0;
pub const SOURCE_GLYPH: f32 = 11.0;
pub const ADD_ICON: f32 = 14.0;
pub const FOOT_H: f32 = 28.0;
pub const FOOT_GAP: f32 = 8.0;
pub const FOOT_PX: f32 = 8.0;
pub const SETTINGS_ICON: f32 = 16.0;

pub const TOOLBAR_H: f32 = 40.0;
pub const TOOLBAR_PX: f32 = 12.0;
pub const TOOLBAR_GAP: f32 = 6.0;
pub const TOOL_H: f32 = 28.0;
pub const TOOL_PX: f32 = 8.0;
pub const TOOL_GAP: f32 = 5.0;
pub const TOOL_RADIUS: f32 = 7.0;
pub const SEG_PAD: f32 = 2.0;
pub const SEG_RADIUS: f32 = 8.0;
pub const SEG_BTN_W: f32 = 28.0;
pub const SEG_BTN_H: f32 = 22.0;
pub const SEG_BTN_RADIUS: f32 = 6.0;
pub const SORT_ICON: f32 = 15.0;
pub const CHEVRON_ICON: f32 = 11.0;
pub const VIEW_ICON: f32 = 15.0;
pub const SIDEBAR_ICON: f32 = 18.0;
pub const PANEL_ICON: f32 = 18.0;

pub const CARD_PAD_X: f32 = 40.0;
pub const CARD_PAD_TOP: f32 = 4.0;
pub const TITLE_ROW_H: f32 = 36.0;
pub const TITLE_GAP: f32 = 10.0;
pub const SECTION_MT: f32 = 28.0;
pub const SECTION_MB: f32 = 12.0;
pub const SECTION_GAP: f32 = 8.0;

pub const CONTINUE_GAP: f32 = 16.0;
pub const CONTINUE_MAX: usize = 3;
pub const CONTINUE_PAD: f32 = 12.0;
pub const CONTINUE_RADIUS: f32 = 12.0;
pub const CONTINUE_TEXT_PT: f32 = 2.0;
pub const CONTINUE_META_MT: f32 = 2.0;
pub const CONTINUE_INNER_GAP: f32 = 14.0;
pub const CONTINUE_COVER_W: f32 = 52.0;
pub const CONTINUE_COVER_H: f32 = 78.0;
pub const BAR_H: f32 = 3.0;
pub const BAR_RADIUS: f32 = 2.0;
pub const BAR_GAP: f32 = 8.0;

pub const SHELF_COVER_W: f32 = 128.0;
pub const SHELF_COVER_H: f32 = 192.0;
pub const COVER_LABEL_PX: f32 = 8.0;
pub const SHELF_RADIUS: f32 = 3.0;
pub const SHELF_MIN_GAP: f32 = 24.0;
pub const SHELF_ROW_GAP: f32 = 28.0;
pub const SHELF_TITLE_MT: f32 = 10.0;
pub const SHELF_META_MT: f32 = 1.0;
pub const SHELF_META_GAP: f32 = 6.0;

pub const TAG_RADIUS: f32 = 3.0;
pub const TAG_PX: f32 = 3.0;
pub const TAG_BORDER: f32 = 0.5;

pub const CHAT_PAD_X: f32 = 16.0;
pub const CHAT_HEAD_H: f32 = 52.0;
pub const CHAT_HEAD_GAP: f32 = 10.0;
pub const AVATAR: f32 = 26.0;
pub const NAME_GAP: f32 = 6.0;
pub const TOOL_CLUSTER_GAP: f32 = 2.0;
pub const CONV_GAP: f32 = 14.0;
pub const CONV_PT: f32 = 12.0;
pub const BUBBLE_RADIUS: f32 = 14.0;
pub const BUBBLE_TAIL: f32 = 4.0;
pub const BUBBLE_PX: f32 = 12.0;
pub const BUBBLE_PY: f32 = 8.0;
pub const BUBBLE_MAX: f32 = 250.0;
pub const COMPOSER_CLEAR: f32 = 120.0;
pub const COMPOSER_INSET: f32 = 12.0;
pub const COMPOSER_RADIUS: f32 = 14.0;
pub const COMPOSER_PT: f32 = 12.0;
pub const COMPOSER_PR: f32 = 10.0;
pub const COMPOSER_PB: f32 = 8.0;
pub const COMPOSER_PL: f32 = 14.0;
pub const PLACEHOLDER_H: f32 = 20.0;
pub const COMPOSER_ROW_MT: f32 = 20.0;
pub const COMPOSER_ROW_GAP: f32 = 6.0;
pub const ATTACH: f32 = 24.0;
pub const ATTACH_ICON: f32 = 16.0;
pub const SCOPE_PX: f32 = 7.0;
pub const SCOPE_H: f32 = 22.0;
pub const SCOPE_RADIUS: f32 = 6.0;
pub const SCOPE_GAP: f32 = 4.0;
pub const SEND: f32 = 26.0;
pub const SEND_ICON: f32 = 14.0;
pub const CHIP_RADIUS: f32 = 999.0;
pub const CHIP_GAP: f32 = 6.0;
pub const CHIP_PX: f32 = 10.0;
pub const CHIP_PY: f32 = 4.0;
pub const ROW_ON_ALPHA: f32 = 0.07;
pub const SEARCH_ALPHA: f32 = 0.05;
pub const SEG_ALPHA: f32 = 0.05;
pub const BAR_TRACK_ALPHA: f32 = 0.08;
pub const TAG_BORDER_ALPHA: f32 = 0.25;

pub fn track(size: f32, em: f32) -> f32 {
    size * em
}

/// Columns of 128-point covers. At the 1440-point mockup the content width is 808 and this returns 5.
pub fn shelf_columns(content_w: f32) -> u32 {
    if content_w < SHELF_COVER_W {
        return 1;
    }
    let count = ((content_w + SHELF_MIN_GAP) / (SHELF_COVER_W + SHELF_MIN_GAP)).floor() as u32;
    count.max(1)
}

pub fn shelf_gap(content_w: f32, columns: u32) -> f32 {
    if columns <= 1 {
        return 0.0;
    }
    let used = columns as f32 * SHELF_COVER_W;
    ((content_w - used) / (columns - 1) as f32).max(0.0)
}

pub fn card_content_width(window_w: f32) -> f32 {
    let card = window_w - SIDEBAR_W as f32 - CHAT_W as f32 - INSET as f32 * 2.0;
    (card - CARD_PAD_X * 2.0).max(0.0)
}

fn black(alpha: f32) -> nsh_core::color::Rgba {
    let byte = (alpha.clamp(0.0, 1.0) * 255.0).round() as u32;
    rgba(byte)
}

fn layer(x: f32, y: f32, blur: f32, spread: f32, alpha: f32) -> BoxShadow {
    BoxShadow {
        offset_x: px(x),
        offset_y: px(y),
        blur: px(blur),
        spread: px(spread),
        color: black(alpha).into(),
        inset: false,
    }
}

pub fn card_shadows() -> Vec<BoxShadow> {
    vec![
        layer(0.0, 0.0, 0.0, 0.5, 0.07),
        layer(0.0, 1.0, 2.0, 0.0, 0.04),
        layer(0.0, 6.0, 20.0, 0.0, 0.045),
    ]
}

pub fn cover_shadows() -> Vec<BoxShadow> {
    vec![
        layer(0.0, 0.0, 0.0, 0.5, 0.08),
        layer(0.0, 1.0, 2.0, 0.0, 0.12),
        layer(0.0, 6.0, 14.0, -4.0, 0.18),
    ]
}

pub fn bubble_shadows() -> Vec<BoxShadow> {
    vec![
        layer(0.0, 0.0, 0.0, 0.5, 0.05),
        layer(0.0, 1.0, 2.0, 0.0, 0.04),
    ]
}

pub fn composer_shadows() -> Vec<BoxShadow> {
    vec![
        layer(0.0, 0.0, 0.0, 0.5, 0.08),
        layer(0.0, 2.0, 10.0, 0.0, 0.05),
    ]
}

pub fn seg_shadows() -> Vec<BoxShadow> {
    vec![
        layer(0.0, 0.0, 0.0, 0.5, 0.06),
        layer(0.0, 1.0, 2.0, 0.0, 0.08),
    ]
}

pub fn chip_shadows() -> Vec<BoxShadow> {
    vec![layer(0.0, 0.0, 0.0, 0.5, 0.12)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_mockup_width_fits_five_covers() {
        let content = card_content_width(WINDOW_W as f32);
        assert!((content - 808.0).abs() < 0.1, "{content}");
        assert_eq!(shelf_columns(content), 5);
        let gap = shelf_gap(content, 5);
        assert!((gap - 42.0).abs() < 0.1, "{gap}");
    }

    #[test]
    fn a_narrow_card_does_not_wrap_at_two_by_fiat() {
        assert_eq!(shelf_columns(400.0), 2);
        assert_eq!(shelf_columns(128.0), 1);
        assert!(shelf_columns(656.0) >= 4);
    }
}
