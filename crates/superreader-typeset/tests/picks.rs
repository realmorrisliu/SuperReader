use superreader_typeset::{
    load_chapter, ChapterInput, Typesetter, TEX_HYPHEN_LEFT_MIN, TEX_HYPHEN_PENALTY, TEX_HYPHEN_RIGHT_MIN,
};

fn fonts() -> Option<Typesetter> {
    let dir = "/tmp/superreader-fixtures";
    if !std::path::Path::new(dir).join("EBGaramond-Regular.ttf").is_file() {
        return None;
    }
    Some(
        Typesetter::open(
            &format!("{dir}/EBGaramond-Regular.ttf"),
            &format!("{dir}/EBGaramond-Italic.ttf"),
            &format!("{dir}/NotoSerifCJKsc-Regular.otf"),
        )
        .unwrap(),
    )
}

fn lay(setter: &Typesetter, xhtml: &str, css: &str) -> superreader_typeset::ChapterLayout {
    setter
        .compose(ChapterInput {
            xhtml,
            stylesheet: css,
            base_href: "",
        })
        .unwrap()
}

fn glyphs(layout: &superreader_typeset::ChapterLayout) -> Vec<superreader_typeset::PlacedGlyph> {
    (0..layout.report().page_count)
        .flat_map(|page| layout.glyph_runs(page))
        .flat_map(|run| run.glyphs)
        .collect()
}

fn is_close(ch: char) -> bool {
    "」』）〉》】〕〗〙〛”’。、，．！？；：".contains(ch)
}

fn is_open(ch: char) -> bool {
    "「『（〈《【〔〖〘〚“‘".contains(ch)
}

#[test]
fn jlreq_hang_drops_edge_aki_and_hangs_closing_marks() {
    let Some(setter) = fonts() else { return };
    let mut body = String::new();
    for _ in 0..14 {
        body.push_str("春夏秋冬春夏秋冬春夏秋冬春夏。「引」");
    }
    let xhtml = format!(
        r#"<html><body><p style="text-indent: 0">「春</p><p>{body}</p></body></html>"#
    );
    let layout = lay(&setter, &xhtml, "");
    let box_ = layout.content_box(0);
    let all = glyphs(&layout);
    let ends: Vec<_> = all.iter().filter(|glyph| glyph.line_end && is_close(glyph.ch)).collect();
    assert!(!ends.is_empty(), "no line-end closing mark");
    for glyph in &ends {
        let ink_right = glyph.x + glyph.advance;
        assert!(
            ink_right > box_.right + 0.5,
            "closing {:?} ink {ink_right} did not hang past {}",
            glyph.ch,
            box_.right
        );
    }
    let starts: Vec<_> = all.iter().filter(|glyph| glyph.line_start && is_open(glyph.ch)).collect();
    assert!(!starts.is_empty(), "no line-start opening mark");
    for glyph in &starts {
        assert!(
            glyph.x <= box_.left + 2.0,
            "opening {:?} starts at {} instead of the content edge {}",
            glyph.ch,
            glyph.x,
            box_.left
        );
        assert!(glyph.x >= box_.left - 1.0, "opening hangs off the left");
    }
    assert_eq!(layout.report().kinsoku_violations, 0);
}

#[test]
fn tex_hyphen_defaults_break_hyphenation_at_two_and_six() {
    assert_eq!(TEX_HYPHEN_PENALTY, 50.0);
    assert_eq!(TEX_HYPHEN_LEFT_MIN, 2);
    assert_eq!(TEX_HYPHEN_RIGHT_MIN, 3);
    let Some(setter) = fonts() else { return };
    let xhtml = "<html><body><p style=\"font-size: 34px\">hyphenation hyphenation hyphenation hyphenation hyphenation hyphenation</p></body></html>";
    let layout = lay(&setter, xhtml, "");
    let offsets: Vec<u32> = glyphs(&layout)
        .iter()
        .filter(|glyph| glyph.hyphen)
        .map(|glyph| glyph.offset)
        .collect();
    assert!(!offsets.is_empty(), "expected a hyphen in the fixture, lines {}", layout.report().line_count);
    assert!(
        offsets.iter().all(|offset| offset % 12 == 2 || offset % 12 == 6),
        "offsets {offsets:?}"
    );
}

#[test]
fn cjk_latin_quarter_em_on_of_zhaohua_and_mid_line() {
    let Some(setter) = fonts() else { return };
    let xhtml = "<html><body><p>Read of 朝花夕拾 today</p></body></html>";
    let layout = lay(&setter, xhtml, "");
    let all = glyphs(&layout);
    let text: String = all.iter().map(|glyph| glyph.ch).collect();
    assert!(text.contains('朝'), "{text}");
    let gap = |left: char, right: char| -> f32 {
        let left_g = all.iter().find(|glyph| glyph.ch == left).unwrap();
        let right_g = all.iter().find(|glyph| glyph.ch == right).unwrap();
        right_g.x - (left_g.x + left_g.advance)
    };
    let latin = gap('d', 'o');
    let mixed = gap('f', '朝');
    let em = all.iter().find(|glyph| glyph.ch == '朝').unwrap().size;
    let extra = mixed - latin;
    assert!(
        (extra - em * 0.25).abs() < 1.5,
        "extra {extra} em {em} latin {latin} mixed {mixed}"
    );
    let zhao = all.iter().find(|glyph| glyph.ch == '朝').unwrap();
    assert!(!zhao.line_start, "朝 should sit mid-line");
}

#[test]
fn publisher_hyphens_none_beats_the_fallback() {
    let Some(setter) = fonts() else { return };
    let xhtml = "<html><body><p style=\"font-size: 80px\">hyphenation hyphenation hyphenation</p></body></html>";
    let layout = lay(&setter, xhtml, "p { hyphens: none }");
    assert!(glyphs(&layout).iter().all(|glyph| !glyph.hyphen));
}

#[test]
fn publisher_hanging_punctuation_none_beats_the_fallback() {
    let Some(setter) = fonts() else { return };
    let mut body = String::new();
    for _ in 0..14 {
        body.push_str("春夏秋冬春夏秋冬春夏秋冬春夏。「引」");
    }
    let xhtml = format!("<html><body><p>{body}</p></body></html>");
    let layout = lay(&setter, &xhtml, "p { hanging-punctuation: none }");
    let box_ = layout.content_box(0);
    let ends: Vec<_> = glyphs(&layout)
        .iter()
        .filter(|glyph| glyph.line_end && is_close(glyph.ch))
        .cloned()
        .collect();
    assert!(!ends.is_empty(), "no line-end closing mark");
    for glyph in &ends {
        let ink_right = glyph.x + glyph.advance;
        assert!(
            ink_right <= box_.right + 0.8,
            "publisher none still hangs {:?} at {ink_right} past {}",
            glyph.ch,
            box_.right
        );
    }
}

#[test]
fn mixed_phonetic_line_does_not_overflow() {
    let Some(setter) = fonts() else { return };
    let mut body = String::new();
    for _ in 0..30 {
        body.push_str("la者“的”也；“兒”");
    }
    let xhtml = format!("<html><body><p>{body}</p></body></html>");
    let layout = lay(&setter, &xhtml, "");
    assert_eq!(layout.report().kinsoku_violations, 0, "kinsoku");
    assert_eq!(layout.report().overflow_lines, 0, "overflow");
    let text: String = glyphs(&layout).iter().map(|glyph| glyph.ch).collect();
    assert!(text.contains('兒'));
}

#[test]
fn zhaohua_has_no_kinsoku_violations_or_overflow() {
    let Some(setter) = fonts() else { return };
    let chapter = load_chapter("/tmp/superreader-fixtures/zhaohua.epub", "25271-0-0.txt.html").unwrap();
    let layout = lay(&setter, &chapter.xhtml, &chapter.stylesheet);
    let report = layout.report();
    assert_eq!(report.kinsoku_violations, 0, "kinsoku {}", report.kinsoku_violations);
    assert_eq!(report.overflow_lines, 0, "overflow {}", report.overflow_lines);
    assert!(report.page_count > 1, "pages {}", report.page_count);
}

#[test]
fn pride_justified_lines_reach_the_measure() {
    let Some(setter) = fonts() else { return };
    let chapter = load_chapter("/tmp/superreader-fixtures/pride-se.epub", "chapter-1.xhtml").unwrap();
    let layout = lay(&setter, &chapter.xhtml, &chapter.stylesheet);
    let mut short = Vec::new();
    for page in 0..layout.report().page_count {
        let right = layout.content_box(page).right;
        let page_glyphs: Vec<_> = layout.glyph_runs(page).into_iter().flat_map(|run| run.glyphs).collect();
        let mut lines: Vec<Vec<superreader_typeset::PlacedGlyph>> = Vec::new();
        for glyph in page_glyphs {
            if let Some(line) = lines.iter_mut().find(|line| (line[0].baseline_y - glyph.baseline_y).abs() < 0.5) {
                line.push(glyph);
            } else {
                lines.push(vec![glyph]);
            }
        }
        for line in lines {
            let ink = line.iter().map(|glyph| glyph.x + glyph.advance).fold(0.0_f32, f32::max);
            if !line.iter().any(|glyph| glyph.line_end) {
                continue;
            }
            if ink < right - 1.5 {
                let text: String = line.iter().map(|glyph| glyph.ch).collect();
                short.push(format!("{:.1} {text}", right - ink));
            }
        }
    }
    assert!(short.is_empty(), "short {}", short.join(" | "));
}

#[test]
fn a_word_wider_than_the_measure_hyphenates_instead_of_overflowing() {
    let Some(setter) = fonts() else { return };
    let word = "pneumonoultramicroscopicsilicovolcanoconiosis";
    let alone = format!("<html><body><p>{word}</p></body></html>");
    let alone_layout = lay(&setter, &alone, "");
    assert_eq!(alone_layout.report().overflow_lines, 0);
    assert!(alone_layout.report().hyphenated_lines >= 1);
    let sentence = "<html><body><p style=\"font-size: 60px\">Hello antidisestablishmentarianism today</p></body></html>";
    let sentence_layout = lay(&setter, sentence, "");
    assert_eq!(sentence_layout.report().overflow_lines, 0);
    assert!(sentence_layout.report().hyphenated_lines >= 1);
}

#[test]
fn word_gap_skips_last_lines_single_words_and_forced_breaks() {
    let Some(setter) = fonts() else { return };
    let body = "one two three four five six seven eight nine ten eleven twelve thirteen fourteen";
    let base = format!("<html><body><p>{body} {body}</p></body></html>");
    let extra = format!("<html><body><p>{body} {body}</p><p>Only</p><p>Stop<br/>here now</p></body></html>");
    let plain = lay(&setter, &base, "");
    let with_extras = lay(&setter, &extra, "");
    let left = plain.report().word_gap_p95;
    let right = with_extras.report().word_gap_p95;
    assert!((left - right).abs() < 1e-9, "p95 {left} vs {right}");
    assert!(left > 1.0, "p95 {left}");
    assert!((plain.report().word_gap_max - with_extras.report().word_gap_max).abs() < 1e-9);
}

#[test]
fn caret_hit_test_returns_the_glyph_under_the_point() {
    let Some(setter) = fonts() else { return };
    let layout = lay(&setter, "<html><body><p>Hello there.</p></body></html>", "");
    let glyph = layout.glyph_runs(0).into_iter().next().unwrap().glyphs.into_iter().next().unwrap();
    let caret = layout
        .hit_test(0, superreader_typeset::Point { x: glyph.x + 0.5, y: glyph.baseline_y })
        .unwrap();
    assert_eq!(caret.node, glyph.node);
    assert_eq!(caret.offset, glyph.offset);
    let rects = layout.selection_rects(
        0,
        superreader_typeset::CaretRange {
            start: superreader_typeset::Caret { node: glyph.node, offset: 0 },
            end: superreader_typeset::Caret { node: glyph.node, offset: 5 },
        },
    );
    assert!(!rects.is_empty());
}
