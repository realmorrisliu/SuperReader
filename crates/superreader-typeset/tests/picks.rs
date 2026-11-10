use superreader_typeset::{
    load_chapter, ChapterInput, Typesetter, TEX_HYPHEN_LEFT_MIN, TEX_HYPHEN_PENALTY,
    TEX_HYPHEN_RIGHT_MIN,
};

fn fonts() -> Option<Typesetter> {
    let dir = "/tmp/superreader-fixtures";
    let names = [
        "EBGaramond-Regular.ttf",
        "EBGaramond-Italic.ttf",
        "NotoSerifCJKsc-Regular.otf",
    ];
    let missing: Vec<&str> = names
        .into_iter()
        .filter(|name| !std::path::Path::new(dir).join(name).is_file())
        .collect();
    if !missing.is_empty() {
        let message = format!(
            "fixture fonts are not in {dir} ({}); run just fixtures-fonts",
            missing.join(", ")
        );
        if std::env::var_os("CI").is_some() {
            panic!("{message}");
        }
        let name = std::thread::current().name().unwrap_or("picks").to_string();
        let line = format!("skip {name}: {message}\n");
        if let Ok(mut err) = std::fs::OpenOptions::new().write(true).open("/dev/stderr") {
            let _ = std::io::Write::write_all(&mut err, line.as_bytes());
        }
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
            measure: None,
            font_size: None,
            tune: superreader_typeset::BreakTune::default(),
            bold_word_starts: false,
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
    let xhtml =
        format!(r#"<html><body><p style="text-indent: 0">「春</p><p>{body}</p></body></html>"#);
    let layout = lay(&setter, &xhtml, "");
    let box_ = layout.content_box(0);
    let all = glyphs(&layout);
    let ends: Vec<_> = all
        .iter()
        .filter(|glyph| glyph.line_end && is_close(glyph.ch))
        .collect();
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
    let starts: Vec<_> = all
        .iter()
        .filter(|glyph| glyph.line_start && is_open(glyph.ch))
        .collect();
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
    assert!(
        !offsets.is_empty(),
        "expected a hyphen in the fixture, lines {}",
        layout.report().line_count
    );
    assert!(
        offsets
            .iter()
            .all(|offset| offset % 12 == 2 || offset % 12 == 6),
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
    let chapter = load_chapter(
        "/tmp/superreader-fixtures/zhaohua.epub",
        "25271-0-0.txt.html",
    )
    .unwrap();
    let layout = lay(&setter, &chapter.xhtml, &chapter.stylesheet);
    let report = layout.report();
    assert_eq!(
        report.kinsoku_violations, 0,
        "kinsoku {}",
        report.kinsoku_violations
    );
    assert_eq!(
        report.overflow_lines, 0,
        "overflow {}",
        report.overflow_lines
    );
    assert!(report.page_count > 1, "pages {}", report.page_count);
}

#[test]
fn pride_justified_lines_reach_the_measure() {
    let Some(setter) = fonts() else { return };
    let chapter =
        load_chapter("/tmp/superreader-fixtures/pride-se.epub", "chapter-1.xhtml").unwrap();
    let layout = lay(&setter, &chapter.xhtml, &chapter.stylesheet);
    let mut short = Vec::new();
    for page in 0..layout.report().page_count {
        let right = layout.content_box(page).right;
        let page_glyphs: Vec<_> = layout
            .glyph_runs(page)
            .into_iter()
            .flat_map(|run| run.glyphs)
            .collect();
        let mut lines: Vec<Vec<superreader_typeset::PlacedGlyph>> = Vec::new();
        for glyph in page_glyphs {
            if let Some(line) = lines
                .iter_mut()
                .find(|line| (line[0].baseline_y - glyph.baseline_y).abs() < 0.5)
            {
                line.push(glyph);
            } else {
                lines.push(vec![glyph]);
            }
        }
        for line in lines {
            let ink = line
                .iter()
                .map(|glyph| glyph.x + glyph.advance)
                .fold(0.0_f32, f32::max);
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

fn paragraph_texts(layout: &superreader_typeset::ChapterLayout) -> Vec<String> {
    let mut lines: Vec<(f32, String, bool)> = Vec::new();
    for glyph in glyphs(layout) {
        if lines
            .last()
            .is_none_or(|(y, _, _)| (glyph.baseline_y - y).abs() > 0.5)
        {
            lines.push((glyph.baseline_y, String::new(), glyph.paragraph_start));
        }
        lines.last_mut().unwrap().1.push(glyph.ch);
    }
    let mut paras = Vec::new();
    for (_, text, start) in lines {
        if start || paras.is_empty() {
            paras.push(text);
        } else {
            paras.last_mut().unwrap().push_str(&text);
        }
    }
    paras
}

#[test]
fn pride_chapter_blocks_match_source_paragraphs() {
    let Some(setter) = fonts() else { return };
    let chapter =
        load_chapter("/tmp/superreader-fixtures/pride-se.epub", "chapter-1.xhtml").unwrap();
    let paragraphs = chapter.xhtml.matches("<p>").count() + chapter.xhtml.matches("<p ").count();
    let headings = chapter.xhtml.matches("<h2").count();
    let layout = lay(&setter, &chapter.xhtml, &chapter.stylesheet);
    let paras = paragraph_texts(&layout);
    assert_eq!(paragraphs, 34, "source paragraphs");
    assert_eq!(headings, 1, "chapter heading");
    assert_eq!(
        paras.len(),
        paragraphs + headings,
        "blocks {:?}",
        paras
            .iter()
            .map(|p| p.chars().take(24).collect::<String>())
            .collect::<Vec<_>>()
    );
    assert!(
        paras
            .iter()
            .any(|text| text.contains("Mydear") && text.contains("Mr.") && text.contains("Bennet")),
        "Mr. Bennet was split"
    );
    assert!(
        paras
            .iter()
            .any(|text| text.contains("You") && text.contains("wanttotellme")),
        "the italic You was split"
    );
}

#[test]
fn fi_ligature_advance_matches_the_shaped_width() {
    let Some(setter) = fonts() else { return };
    for word in ["first", "fixed", "Netherfield"] {
        let layout = lay(
            &setter,
            &format!(
                "<html><body><p style=\"text-align:left;text-indent:0;hyphens:none\">{word}</p></body></html>"
            ),
            "",
        );
        let all = glyphs(&layout);
        assert!(
            all.iter()
                .all(|glyph| glyph.glyph_id != 0 || glyph.advance < 0.05),
            "{word} kept a width on an undrawn ligature part"
        );
        let drawn: Vec<_> = all
            .into_iter()
            .filter(|glyph| glyph.glyph_id != 0)
            .collect();
        assert!(drawn.len() >= 2, "{word} glyphs {}", drawn.len());
        for pair in drawn.windows(2) {
            let gap = pair[1].x - (pair[0].x + pair[0].advance);
            assert!(
                gap.abs() < 0.6,
                "{word} {} to {} gap {gap:.2}",
                pair[0].ch,
                pair[1].ch
            );
        }
        let sum: f32 = drawn.iter().map(|glyph| glyph.advance).sum();
        let span = drawn.last().unwrap().x + drawn.last().unwrap().advance - drawn[0].x;
        assert!((sum - span).abs() < 0.6, "{word} sum {sum} span {span}");
    }
}

#[test]
fn ambiguous_quotes_take_no_cjk_latin_glue() {
    let Some(setter) = fonts() else { return };
    let xhtml = "<html><body><p>叫做《一覺》連這“一覺”也沒有</p></body></html>";
    let layout = lay(&setter, xhtml, "");
    let all = glyphs(&layout);
    let at = |ch: char, skip: usize| all.iter().filter(|glyph| glyph.ch == ch).nth(skip).unwrap();
    let open = at('“', 0);
    let prev = all
        .iter()
        .take_while(|glyph| glyph.ch != '“')
        .filter(|glyph| glyph.glyph_id != 0)
        .last()
        .unwrap();
    let glue = open.x - (prev.x + prev.advance);
    assert!(glue.abs() < 0.75, "quote glue {glue:.2} after {}", prev.ch);
    let close = at('”', 0);
    let after = all
        .iter()
        .skip_while(|glyph| glyph.ch != '”')
        .nth(1)
        .unwrap();
    let after_glue = after.x - (close.x + close.advance);
    assert!(
        after_glue.abs() < 0.75,
        "closing quote glue {after_glue:.2}"
    );
    let bracket = at('《', 0);
    let before = all
        .iter()
        .take_while(|glyph| glyph.ch != '《')
        .filter(|glyph| glyph.glyph_id != 0)
        .last()
        .unwrap();
    let aki = bracket.x - (before.x + before.advance);
    let em = bracket.size;
    assert!(
        (aki - em * 0.5).abs() < 1.5,
        "《 aki {aki:.2} em {em} after {}",
        before.ch
    );
    let source = "叫做《一覺》";
    assert!(!source.chars().any(|ch| ch.is_whitespace()));
}

#[test]
fn word_gap_counts_a_break_that_is_not_the_paragraph_end() {
    let Some(setter) = fonts() else { return };
    let wide = Some(800.0);
    let only_last = lay_at(
        &setter,
        "<html><body><p>alpha beta gamma</p></body></html>",
        wide,
    );
    assert_eq!(
        only_last.line_stats().scored_lines,
        0,
        "a one-line paragraph is the last line"
    );
    let broken = lay_at(
        &setter,
        "<html><body><p>alpha beta gamma<br/>end</p></body></html>",
        wide,
    );
    assert_eq!(
        broken.line_stats().scored_lines,
        1,
        "the line before the break is not the paragraph's last line"
    );
    let p95 = broken.report().word_gap_p95;
    assert!(p95 > 1.0, "the break line is in the sample, p95 {p95}");
}

#[test]
fn word_gap_skips_cjk_paragraphs() {
    let Some(setter) = fonts() else { return };
    let latin = "one two three four five six seven eight nine ten eleven twelve thirteen";
    let plain = lay_at(
        &setter,
        &format!("<html><body><p>{latin}</p></body></html>"),
        Some(220.0),
    );
    assert!(
        plain.line_stats().scored_lines > 0,
        "latin lines are in the sample"
    );
    let mixed = lay_at(
        &setter,
        &format!("<html><body><p>{latin} 字</p></body></html>"),
        Some(220.0),
    );
    assert_eq!(
        mixed.line_stats().scored_lines,
        0,
        "a CJK paragraph is out of the sample"
    );
    assert_eq!(mixed.report().word_gap_count, 0);
}

#[test]
fn word_gap_skips_last_lines_single_words_and_forced_breaks() {
    let Some(setter) = fonts() else { return };
    let body = "one two three four five six seven eight nine ten eleven twelve thirteen fourteen";
    let base = format!("<html><body><p>{body} {body}</p></body></html>");
    let extra = format!(
        "<html><body><p>{body} {body}</p><p>Only</p><p>Stop<br/>here now</p></body></html>"
    );
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
    let glyph = layout
        .glyph_runs(0)
        .into_iter()
        .next()
        .unwrap()
        .glyphs
        .into_iter()
        .next()
        .unwrap();
    let caret = layout
        .hit_test(
            0,
            superreader_typeset::Point {
                x: glyph.x + 0.5,
                y: glyph.baseline_y,
            },
        )
        .unwrap();
    assert_eq!(caret.node, glyph.node);
    assert_eq!(caret.offset, glyph.offset);
    let rects = layout.selection_rects(
        0,
        superreader_typeset::CaretRange {
            start: superreader_typeset::Caret {
                node: glyph.node,
                offset: 0,
            },
            end: superreader_typeset::Caret {
                node: glyph.node,
                offset: 5,
            },
        },
    );
    assert!(!rects.is_empty());
}

fn lay_at(
    setter: &Typesetter,
    xhtml: &str,
    measure: Option<f32>,
) -> superreader_typeset::ChapterLayout {
    lay_tuned(
        setter,
        xhtml,
        measure,
        superreader_typeset::BreakTune::default(),
    )
}

fn lay_tuned(
    setter: &Typesetter,
    xhtml: &str,
    measure: Option<f32>,
    tune: superreader_typeset::BreakTune,
) -> superreader_typeset::ChapterLayout {
    setter
        .compose(ChapterInput {
            xhtml,
            stylesheet: "",
            base_href: "",
            measure,
            font_size: Some(22.0),
            tune,
            bold_word_starts: false,
        })
        .unwrap()
}

fn justified_line(
    layout: &superreader_typeset::ChapterLayout,
) -> Vec<superreader_typeset::PlacedGlyph> {
    let mut lines: Vec<Vec<superreader_typeset::PlacedGlyph>> = Vec::new();
    for glyph in glyphs(layout) {
        let slot = lines
            .iter()
            .position(|line| (line[0].baseline_y - glyph.baseline_y).abs() < 0.5);
        if let Some(slot) = slot {
            lines[slot].push(glyph);
        } else {
            lines.push(vec![glyph]);
        }
    }
    lines
        .into_iter()
        .find(|line| line.iter().any(|glyph| glyph.line_end))
        .expect("a justified line")
}

#[test]
fn font_expansion_scales_advances_and_the_caret_follows() {
    let Some(setter) = fonts() else { return };
    let xhtml = "<html><body><p>It is a truth universally acknowledged, that a single man in possession of a good fortune must be in want of a wife today.</p></body></html>";
    let base = lay_tuned(
        &setter,
        xhtml,
        Some(180.0),
        superreader_typeset::BreakTune::default(),
    );
    let tune = superreader_typeset::BreakTune {
        font_expand: 0.02,
        ..superreader_typeset::BreakTune::default()
    };
    let expanded = lay_tuned(&setter, xhtml, Some(180.0), tune);
    let base_line = justified_line(&base);
    let line = justified_line(&expanded);
    assert!(
        glyphs(&expanded)
            .iter()
            .any(|glyph| (glyph.x_scale - 1.0).abs() > 0.004),
        "scale stayed 1"
    );
    let moved = line.iter().any(|glyph| {
        base_line.iter().any(|other| {
            other.offset == glyph.offset && (other.advance - glyph.advance).abs() > 0.05
        })
    });
    assert!(moved, "advances did not move");
    let glyph = line.iter().find(|glyph| !glyph.hyphen).unwrap();
    let hit = expanded
        .hit_test(
            0,
            superreader_typeset::Point {
                x: glyph.x + glyph.advance * 0.3,
                y: glyph.baseline_y,
            },
        )
        .unwrap();
    assert_eq!(hit.offset, glyph.offset);
    let next = line
        .iter()
        .find(|other| other.offset > glyph.offset && !other.hyphen)
        .unwrap();
    let after = expanded
        .hit_test(
            0,
            superreader_typeset::Point {
                x: glyph.x + glyph.advance + 0.15,
                y: glyph.baseline_y,
            },
        )
        .unwrap();
    assert_eq!(after.offset, next.offset, "hit past the scaled advance");
    let end = line.iter().rev().find(|glyph| !glyph.hyphen).unwrap();
    let rects = expanded.selection_rects(
        0,
        superreader_typeset::CaretRange {
            start: superreader_typeset::Caret {
                node: glyph.node,
                offset: glyph.offset,
            },
            end: superreader_typeset::Caret {
                node: end.node,
                offset: end.offset + end.span,
            },
        },
    );
    let span = end.x + end.advance - glyph.x;
    let width: f32 = rects.iter().map(|rect| rect.width).sum();
    assert!((width - span).abs() < 1.0, "selection {width} span {span}");
}

#[test]
fn letter_spacing_changes_advances_without_scaling_the_outline() {
    let Some(setter) = fonts() else { return };
    let xhtml = "<html><body><p>It is a truth universally acknowledged, that a single man in possession of a good fortune must be in want of a wife today.</p></body></html>";
    let base = lay_tuned(
        &setter,
        xhtml,
        Some(180.0),
        superreader_typeset::BreakTune::default(),
    );
    let tune = superreader_typeset::BreakTune {
        letter_space: 0.01,
        ..superreader_typeset::BreakTune::default()
    };
    let tracked = lay_tuned(&setter, xhtml, Some(180.0), tune);
    let line = justified_line(&tracked);
    assert!(
        line.iter().all(|glyph| (glyph.x_scale - 1.0).abs() < 1e-6),
        "letter-spacing scaled an outline"
    );
    let base_line = justified_line(&base);
    let moved = line.iter().any(|glyph| {
        base_line.iter().any(|other| {
            other.offset == glyph.offset && (other.advance - glyph.advance).abs() > 0.04
        })
    });
    assert!(moved, "letter-spacing left the advances alone");
    let glyph = line.iter().find(|glyph| !glyph.hyphen).unwrap();
    let hit = tracked
        .hit_test(
            0,
            superreader_typeset::Point {
                x: glyph.x + glyph.advance * 0.3,
                y: glyph.baseline_y,
            },
        )
        .unwrap();
    assert_eq!(hit.offset, glyph.offset);
}

#[test]
fn tolerance_tune_keeps_hyphen_runs_to_two() {
    let Some(setter) = fonts() else { return };
    let chapter =
        load_chapter("/tmp/superreader-fixtures/pride-se.epub", "chapter-1.xhtml").unwrap();
    let tune = superreader_typeset::BreakTune {
        tolerance: true,
        ..superreader_typeset::BreakTune::default()
    };
    let layout = setter
        .compose(ChapterInput {
            xhtml: &chapter.xhtml,
            stylesheet: &chapter.stylesheet,
            base_href: &chapter.href,
            measure: Some(343.47),
            font_size: Some(22.0),
            tune,
            bold_word_starts: false,
        })
        .unwrap();
    assert!(
        layout.line_stats().longest_hyphen_run <= 2,
        "run {}",
        layout.line_stats().longest_hyphen_run
    );
    assert_eq!(layout.report().overflow_lines, 0);
}
