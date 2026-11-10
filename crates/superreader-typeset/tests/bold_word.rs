use spike_bold_word_starts::{bold_prefix_len, LetterRun};
use superreader_typeset::{load_chapter, ChapterInput, Typesetter, BOLD_FACE};

const PROBES: [&str; 15] = [
    "a",
    "be",
    "man",
    "good",
    "truth",
    "single",
    "fortune",
    "entering",
    "character",
    "experience",
    "universally",
    "acknowledged",
    "understanding",
    "characteristic",
    "representations",
];

fn bold_setter() -> Option<Typesetter> {
    let dir = "/tmp/superreader-fixtures";
    let names = [
        "EBGaramond-Regular.ttf",
        "EBGaramond-Italic.ttf",
        "EBGaramond-Bold.ttf",
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
        eprintln!("skip bold word: {message}");
        return None;
    }
    Some(
        Typesetter::open_bold(
            &format!("{dir}/EBGaramond-Regular.ttf"),
            &format!("{dir}/EBGaramond-Italic.ttf"),
            &format!("{dir}/EBGaramond-Bold.ttf"),
            &format!("{dir}/NotoSerifCJKsc-Regular.otf"),
        )
        .unwrap(),
    )
}

fn layout(setter: &Typesetter, xhtml: &str, on: bool) -> superreader_typeset::ChapterLayout {
    setter
        .compose(ChapterInput {
            xhtml,
            stylesheet: "",
            base_href: "",
            measure: None,
            font_size: Some(22.0),
            tune: superreader_typeset::BreakTune::default(),
            bold_word_starts: on,
        })
        .unwrap()
}

fn letters(layout: &superreader_typeset::ChapterLayout) -> Vec<superreader_typeset::PlacedGlyph> {
    (0..layout.report().page_count)
        .flat_map(|page| layout.glyph_runs(page))
        .flat_map(|run| run.glyphs)
        .filter(|glyph| !glyph.hyphen && glyph.ch != ' ')
        .collect()
}

#[test]
fn probe_prefixes_match_the_painter() {
    let Some(setter) = bold_setter() else {
        return;
    };
    let counts: Vec<usize> = PROBES
        .iter()
        .map(|word| bold_prefix_len(LetterRun::parse(word).unwrap()))
        .collect();
    assert_eq!(counts, vec![1, 2, 2, 2, 3, 2, 3, 4, 4, 5, 5, 6, 6, 7, 7]);
    let body = PROBES.join(" ");
    let xhtml =
        format!("<html><body><p style=\"text-align:left;text-indent:0\">{body}</p></body></html>");
    let glyphs = letters(&layout(&setter, &xhtml, true));
    let mut cursor = 0usize;
    for (word, count) in PROBES.iter().zip(counts) {
        let chars: Vec<char> = word.chars().collect();
        let slice = &glyphs[cursor..cursor + chars.len()];
        let drawn: String = slice.iter().map(|glyph| glyph.ch).collect();
        assert_eq!(drawn, *word);
        for (index, glyph) in slice.iter().enumerate() {
            let face = if index < count { BOLD_FACE } else { 0 };
            assert_eq!(glyph.font.0, u16::from(face), "{word} at {index}");
        }
        cursor += chars.len();
    }
    assert_eq!(cursor, glyphs.len());
}

#[test]
fn publisher_bold_and_cjk_stay_on_their_own_faces() {
    let Some(setter) = bold_setter() else {
        return;
    };
    let xhtml = "<html><body><p style=\"text-align:left;text-indent:0\"><b>good</b> 好 good</p></body></html>";
    let glyphs = letters(&layout(&setter, xhtml, true));
    let bold = glyphs
        .iter()
        .filter(|glyph| glyph.font.0 == u16::from(BOLD_FACE))
        .count();
    assert_eq!(bold, 2, "only the plain word's prefix is the Bold face");
    assert!(glyphs.iter().any(|glyph| glyph.ch == '好'));
    assert!(glyphs
        .iter()
        .filter(|glyph| glyph.ch == '好')
        .all(|glyph| glyph.font.0 != u16::from(BOLD_FACE)));
}

#[test]
fn the_bold_face_changes_the_drawn_prefix() {
    let Some(setter) = bold_setter() else {
        return;
    };
    let xhtml = "<html><body><p style=\"text-align:left;text-indent:0\">a</p></body></html>";
    let on = layout(&setter, xhtml, true);
    let off = layout(&setter, xhtml, false);
    let on_png = setter.png(&on, 0).unwrap();
    let off_png = setter.png(&off, 0).unwrap();
    assert_ne!(on_png, off_png);
    let on_letter = letters(&on);
    let off_letter = letters(&off);
    assert_eq!(on_letter[0].font.0, u16::from(BOLD_FACE));
    assert_ne!(
        on_letter[0].advance.to_bits(),
        off_letter[0].advance.to_bits()
    );
}

#[test]
fn the_flag_is_the_layout_input_the_crate_would_cache() {
    let off = ChapterInput {
        xhtml: "a",
        stylesheet: "",
        base_href: "",
        measure: None,
        font_size: None,
        tune: superreader_typeset::BreakTune::default(),
        bold_word_starts: false,
    };
    let on = ChapterInput {
        bold_word_starts: true,
        ..off
    };
    assert_ne!(off, on);
}

#[test]
fn flag_off_matches_a_setter_that_never_loaded_bold() {
    let Some(with_bold) = bold_setter() else {
        return;
    };
    let dir = "/tmp/superreader-fixtures";
    let plain = Typesetter::open(
        &format!("{dir}/EBGaramond-Regular.ttf"),
        &format!("{dir}/EBGaramond-Italic.ttf"),
        &format!("{dir}/NotoSerifCJKsc-Regular.otf"),
    )
    .unwrap();
    let xhtml = "<html><body><p>It is a truth universally acknowledged</p></body></html>";
    let left = letters(&layout(&plain, xhtml, false));
    let right = letters(&layout(&with_bold, xhtml, false));
    let left: Vec<_> = left
        .iter()
        .map(|glyph| {
            (
                glyph.ch,
                glyph.glyph_id,
                glyph.advance.to_bits(),
                glyph.font.0,
            )
        })
        .collect();
    let right: Vec<_> = right
        .iter()
        .map(|glyph| {
            (
                glyph.ch,
                glyph.glyph_id,
                glyph.advance.to_bits(),
                glyph.font.0,
            )
        })
        .collect();
    assert_eq!(left, right);
}

#[test]
fn pride_chapter_one_on_has_no_overflow() {
    let Some(setter) = bold_setter() else {
        return;
    };
    let chapter =
        load_chapter("/tmp/superreader-fixtures/pride-se.epub", "chapter-1.xhtml").unwrap();
    let input = |on| ChapterInput {
        xhtml: &chapter.xhtml,
        stylesheet: &chapter.stylesheet,
        base_href: &chapter.href,
        measure: None,
        font_size: None,
        tune: superreader_typeset::BreakTune::default(),
        bold_word_starts: on,
    };
    let off = setter.compose(input(false)).unwrap();
    let on = setter.compose(input(true)).unwrap();
    let off_report = off.report();
    let on_report = on.report();
    assert_eq!(
        (on_report.overflow_lines, on_report.kinsoku_violations),
        (0, 0),
        "off p95 {:.4} on p95 {:.4}",
        off_report.word_gap_p95,
        on_report.word_gap_p95
    );
    assert!(off_report.word_gap_count > 0 && on_report.word_gap_count > 0);
    let off_stats = off.line_stats();
    let on_stats = on.line_stats();
    eprintln!(
        "pride chapter 1 word-gap p95 off {:.4} on {:.4} lines over 1.75 off {} of {} on {} of {}",
        off_report.word_gap_p95,
        on_report.word_gap_p95,
        off_stats.lines_over_cap,
        off_stats.scored_lines,
        on_stats.lines_over_cap,
        on_stats.scored_lines
    );
}
