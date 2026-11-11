# Full-form tokens

The approved B mockup stylesheet is the source for these numbers. `spikes/shell/src/tokens.rs` is the only place the shell reads them. The window in that stylesheet is 1440 by 900 points.

The card in the stylesheet starts at the sidebar's right edge. The shell keeps a 6 point inset on every side of the card. `shell-card-inset` measures that gap, and `b-rows.png` shows the stylesheet's flush card. The inset stays.

## Window

| CSS | Value | Rust |
| --- | --- | --- |
| `html, body` width / height | 1440px, 900px | `WINDOW_W`, `WINDOW_H` |
| `body` font-family | system stack, SF Pro and PingFang SC | `FONT_STACK` |
| `body` font-size, color, tracking | 13px, `#1d1d1f`, -0.005em | `BODY_SIZE`, `INK`, `BODY_TRACK_EM` |
| `.win` background | `#EEEEF0` to `#E9E9EC` | `FIELD_TOP`, `FIELD_BOTTOM` |
| `.side` width | 220px | `SIDEBAR_W` |
| `.agent` width | 320px | `CHAT_W` |
| `.card` top / bottom | 6px | `INSET` |
| `.card` radius | 12px | `CARD_RADIUS` |
| `.card` fill | `#fff` | `CARD` |
| `.card` shadow | 0.5px ring, 1px/2px, 6px/20px | `card_shadows` |

## Sidebar

| CSS | Value | Rust |
| --- | --- | --- |
| `.side` padding | 52px 10px 12px | `SIDE_PAD_TOP`, `SIDE_PAD_X`, `SIDE_PAD_BOTTOM` |
| `.side-top` height | 52px | `TOP_ROW_H` |
| `.tb` | 28px, radius 7px | `TOGGLE_SIZE`, `TOGGLE_RADIUS` |
| sidebar glyph | 18px, IconMCP `sidebar-left` | `SIDEBAR_ICON` |
| `.sfield` | height 28, radius 8, pad 8, gap 6, 12.5px, `#8E8E93` | `SEARCH_H`, `SEARCH_RADIUS`, `SEARCH_PAD_X`, `SEARCH_GAP`, `SEARCH_SIZE`, `QUIET` |
| `.sfield` fill | `rgba(0,0,0,.05)` | `SEARCH_ALPHA` |
| `.slabel` | 11px / 600, tracking 0.04em, margin 14 0 4 | `LABEL_SIZE`, `LABEL_WEIGHT`, `LABEL_TRACK_EM`, `LABEL_MT`, `LABEL_MB` |
| `.row` | height 28, radius 7, pad 8/10, gap 8, 13px | `ROW_H`, `ROW_RADIUS`, `ROW_PAD_L`, `ROW_PAD_R`, `ROW_GAP`, `BODY_SIZE` |
| `.row.on` | `rgba(0,0,0,.07)`, label 600 | `ROW_ON_ALPHA`, `ROW_WEIGHT_ON` |
| `.row .rn` | 12px `#8E8E93` | `COUNT_ROW_SIZE`, `QUIET` |
| `.row.add` | `#8E8E93`, plus 14px | `QUIET`, `ADD_ICON` |
| `.dot` | 9px | `DOT` |
| `.srcic` | 16px, radius 4 | `SOURCE_BADGE`, `SOURCE_BADGE_RADIUS` |
| Apple Books badge | `#FFA53D` to `#F27A1A` | `APPLE_FROM`, `APPLE_TO` |
| calibre badge | `#6B5A4A` to `#4A3C30` | `CALIBRE_FROM`, `CALIBRE_TO` |
| `.side-foot` | height 28, gap 8, 13px `#6E6E73` | `FOOT_H`, `FOOT_GAP`, `BODY_SIZE`, `ICON` |
| settings glyph | IconMCP `sliders` | row icon `sliders` |

An empty **书单** group still shows its label. The headless check `shell-sidebar` reads that label. The fixture fills the three lists from the mockup when `SUPERREADER_FORM_FIXTURE=1`.

## Card header and shelf

| CSS | Value | Rust |
| --- | --- | --- |
| `.card-tb` | height 40, pad 12, gap 6 | `TOOLBAR_H`, `TOOLBAR_PX`, `TOOLBAR_GAP` |
| `.ctl` | height 28, pad 8, gap 5, 12.5px `#6E6E73` | `TOOL_H`, `TOOL_PX`, `TOOL_GAP`, `TOOL_SIZE`, `ICON` |
| sort / chevron | 15px, 11px | `SORT_ICON`, `CHEVRON_ICON` |
| `.vseg` | pad 2, radius 8, button 28 by 22, radius 6 | `SEG_PAD`, `SEG_RADIUS`, `SEG_BTN_W`, `SEG_BTN_H`, `SEG_BTN_RADIUS` |
| `.vseg` track | `rgba(0,0,0,.05)` | `SEG_ALPHA` |
| `.vseg .on` shadow | 0.5px ring and 1px/2px | `seg_shadows` |
| `.card-body` padding | 4px 40px 0 | `CARD_PAD_TOP`, `CARD_PAD_X` |
| `.title-row` | height 36, gap 10 | `TITLE_ROW_H`, `TITLE_GAP` |
| `h1` | 26px / 700, tracking -0.01em | `H1_SIZE`, `H1_WEIGHT`, `H1_TRACK_EM` |
| `.h1-sub` | 13px / 400 `#8E8E93` | `COUNT_SIZE`, `COUNT_WEIGHT`, `QUIET` |
| `.sec` | 15px / 600, margin 28 0 12, gap 8 | `SECTION_SIZE`, `SECTION_WEIGHT`, `SECTION_MT`, `SECTION_MB`, `SECTION_GAP` |
| `.sec-n` | 12px / 400 `#8E8E93` | `SECTION_NOTE_SIZE`, `BODY_WEIGHT`, `QUIET` |
| `.cr-row` gap | 16px, at most 3 cards | `CONTINUE_GAP`, `CONTINUE_MAX` |
| `.cr` | pad 12, radius 12, gap 14, `#F5F5F7` | `CONTINUE_PAD`, `CONTINUE_RADIUS`, `CONTINUE_INNER_GAP`, `CONTINUE_BG` |
| continue cover | 52 by 78 | `CONTINUE_COVER_W`, `CONTINUE_COVER_H` |
| `.cr-t` | 13px / 600 | `CONTINUE_TITLE_SIZE`, `CONTINUE_TITLE_WEIGHT` |
| `.cr-a` | 12px `#8E8E93`, margin-top 2 | `CONTINUE_META_SIZE`, `QUIET`, `CONTINUE_META_MT` |
| `.bar` | height 3, radius 2, track `rgba(0,0,0,.08)`, fill `#1d1d1f` | `BAR_H`, `BAR_RADIUS`, `BAR_TRACK_ALPHA`, `INK` |
| `.cr-p` | 11px, gap 8 | `PROGRESS_SIZE`, `BAR_GAP` |
| `.cr-w` | 11px `#A1A1A6` | `TIME_SIZE`, `FAINT` |
| `.grid` row gap | 28px | `SHELF_ROW_GAP` |
| shelf cover | 128 by 192, radius 3 | `SHELF_COVER_W`, `SHELF_COVER_H`, `SHELF_RADIUS` |
| column count | 5 at the mockup content width of 808 | `shelf_columns`, `SHELF_MIN_GAP` |
| cover shadow | 0.5px ring, 1px/2px, 6px/14px spread -4 | `cover_shadows` |
| `.b-t` | 12.5px / 500, margin-top 10 | `SHELF_TITLE_SIZE`, `SHELF_TITLE_WEIGHT`, `SHELF_TITLE_MT` |
| `.b-a` | 11.5px `#8E8E93`, gap 6, margin-top 1 | `SHELF_META_SIZE`, `QUIET`, `SHELF_META_GAP`, `SHELF_META_MT` |
| `.pct` | 11px `#8E8E93`, on the author line | `PROGRESS_SIZE` |
| `.tag` | 9px / 600, radius 3, pad 3, 0.5px border | `TAG_SIZE`, `TAG_WEIGHT`, `TAG_RADIUS`, `TAG_PX`, `TAG_BORDER` |
| `.tag.sm` | 8.5px | `TAG_SM_SIZE` |

`shelf_columns(808)` is 5. The gap between those covers is 42 points, which is the stylesheet's `justify-content: space-between` on five 128-point columns. A narrower card uses fewer columns. It does not stop at two.

## Chat

| CSS | Value | Rust |
| --- | --- | --- |
| `.ag-head` | height 52, gap 10, pad from `.agent` 16 | `CHAT_HEAD_H`, `CHAT_HEAD_GAP`, `CHAT_PAD_X` |
| `.avatar` | 26px, `#F0B267` to `#D2693C`, mark 13px / 700 | `AVATAR`, `AVATAR_FROM`, `AVATAR_TO`, `AVATAR_MARK_SIZE`, `AVATAR_MARK_WEIGHT` |
| `.ag-name` / `.ag-sub` | 13px / 600 and 11.5px / 400, gap 6 | `NAME_SIZE`, `NAME_WEIGHT`, `SUB_SIZE`, `NAME_GAP` |
| header icons | compose `edit`, panel `sidebar-right`, 18px | `PANEL_ICON` |
| `.conv` | gap 14, padding-top 12 | `CONV_GAP`, `CONV_PT` |
| `.ctx-line` | 11px `#A1A1A6` | `TIME_RULE_SIZE`, `FAINT` |
| `.msg-u` | radius 14, pad 8 12, max 250, white | `BUBBLE_RADIUS`, `BUBBLE_PY`, `BUBBLE_PX`, `BUBBLE_MAX`, `CARD` |
| bubble shadow | 0.5px ring, 1px/2px | `bubble_shadows` |
| `.msg-a` | 13px `#1d1d1f` | `MSG_SIZE`, `INK` |
| `.chip` | 12px, pad 4 10, radius 999 | `CHIP_SIZE`, `CHIP_PY`, `CHIP_PX`, `CHIP_RADIUS` |
| `.composer` | inset 12, radius 14, pad 12 10 8 14 | `COMPOSER_INSET`, `COMPOSER_RADIUS`, `COMPOSER_PT`, `COMPOSER_PR`, `COMPOSER_PB`, `COMPOSER_PL` |
| composer shadow | 0.5px ring, 2px/10px | `composer_shadows` |
| `.ph` | 13px `#A1A1A6` | `BODY_SIZE`, `FAINT` |
| `.cmp-row` | margin-top 20, gap 6 | `COMPOSER_ROW_MT`, `COMPOSER_ROW_GAP` |
| `.scope` | height 22, radius 6, `#F2F2F4`, 11.5px | `SCOPE_H`, `SCOPE_RADIUS`, `SCOPE_BG`, `SUB_SIZE` |
| `.send` | 26px, `#D8D8DD` | `SEND`, `SEND_BG` |
| paperclip / send glyph | 16px, 14px | `ATTACH_ICON`, `SEND_ICON` |

With no stored chat lines the thread is empty. The placeholder **问问 Slowpage…** stays on the composer. The fixture writes two exchanges when `SUPERREADER_FORM_FIXTURE=1`.

## Icons

IconMCP only. Outline, stroke 1.25, radius 2. Solid is the selected row. `assets/icons/manifest.toml` lists them. `scripts/icons-fetch.sh` (the `just icons-fetch` recipe) writes the SVGs.

| Mockup glyph | IconMCP name | File |
| --- | --- | --- |
| sidebar toggle | `sidebar-left` | `sidebar-left.svg` |
| search | `search` | `search.svg` |
| 全部书籍 | `library` | `library.svg`, `library-solid.svg` |
| 正在读 | `book-open` | `book-open.svg`, `book-open-solid.svg` |
| 想读 | `bookmark` | `bookmark.svg`, `bookmark-solid.svg` |
| 已读完 | `check` | `check.svg`, `check-solid.svg` |
| PDF | `file` | `file.svg`, `file-solid.svg` |
| 高亮与笔记 | `note` | `note.svg`, `note-solid.svg` |
| Apple Books badge | `book-open` | `book-open.svg` |
| calibre badge | `library` | `library.svg` |
| folder source | `folder` | `folder.svg`, `folder-solid.svg` |
| 添加来源 | `plus` | `plus.svg` |
| 设置 | `sliders` | `sliders.svg`, `sliders-solid.svg` |
| 最近阅读 | `sort-desc` | `sort-desc.svg` |
| chevron | `chevron-down` | `chevron-down.svg` |
| grid | `grid` | `grid.svg` |
| list | `list` | `list.svg` |
| new chat | `edit` | `edit.svg` |
| panel | `sidebar-right` | `sidebar-right.svg` |
| attach | `paperclip` | `paperclip.svg` |
| scope chip | `book-open` | `book-open.svg` |
| send | `arrow-up` | `arrow-up.svg` |

The reading toolbar still uses `list` for the contents control. The sidebar toggle does not.
