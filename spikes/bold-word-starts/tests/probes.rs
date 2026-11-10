use spike_bold_word_starts::{bold_prefix_len, words, LetterRun, BOLD_WORD_STARTS_LABEL};

fn count(token: &str) -> usize {
    bold_prefix_len(LetterRun::parse(token).unwrap())
}

fn spans(text: &str) -> Vec<(usize, String, usize)> {
    words(text)
        .map(|span| {
            let run = span.run();
            (span.at(), run.as_str().to_string(), bold_prefix_len(run))
        })
        .collect()
}

#[test]
fn fifteen_probe_words_match_the_locked_counts() {
    let probes = [
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
    let counts: Vec<usize> = probes.iter().copied().map(count).collect();
    assert_eq!(counts, vec![1, 2, 2, 2, 3, 2, 3, 4, 4, 5, 5, 6, 6, 7, 7]);
}

#[test]
fn onset_follows_the_consonants_and_the_next_vowel() {
    assert_eq!(count("y"), 1);
    assert_eq!(count("by"), 2);
    assert_eq!(count("yellow"), 2);
    assert_eq!(count("rhythm"), 3);
    assert_eq!(count("at"), 1);
    assert_eq!(count("that"), 3);
    assert_eq!(count("pm"), 2);
}

#[test]
fn vowel_initial_long_words_take_the_tier_count() {
    assert_eq!(count("experience"), 5);
    assert_eq!(count("universally"), 5);
    assert_eq!(count("understanding"), 6);
    assert_eq!(count("information"), 5);
    assert_ne!(count("information"), 1);
}

#[test]
fn digits_are_unmarked_gaps() {
    assert!(LetterRun::parse("A1").is_none());
    assert!(LetterRun::parse("3pm").is_none());
    assert!(words("42").next().is_none());
    assert_eq!(spans("3pm"), vec![(1, "pm".to_string(), 2)]);
    assert_eq!(spans("COVID19"), vec![(0, "COVID".to_string(), 2)]);
    assert_eq!(
        spans("A1B"),
        vec![(0, "A".to_string(), 1), (2, "B".to_string(), 1)]
    );
}

#[test]
fn hyphen_and_apostrophe_split_the_word() {
    assert!(LetterRun::parse("well-known").is_none());
    assert!(LetterRun::parse("don't").is_none());
    assert_eq!(
        spans("well-known"),
        vec![(0, "well".to_string(), 2), (5, "known".to_string(), 3)]
    );
    assert_eq!(
        spans("don't"),
        vec![(0, "don".to_string(), 2), (4, "t".to_string(), 1)]
    );
}

#[test]
fn han_text_yields_no_run() {
    assert!(words("范愛農").next().is_none());
    assert_eq!(spans("范Love農"), vec![(3, "Love".to_string(), 2)]);
}

#[test]
fn label_is_the_product_name() {
    assert_eq!(BOLD_WORD_STARTS_LABEL, ("Bold Word Starts", "词首加粗"));
}
