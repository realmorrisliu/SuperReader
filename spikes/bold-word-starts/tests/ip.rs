use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use spike_bold_word_starts::{bold_prefix_len, LetterRun};

fn count_letters(len: usize, byte: u8) -> usize {
    let token = vec![byte; len];
    let token = std::str::from_utf8(&token).unwrap();
    bold_prefix_len(LetterRun::parse(token).unwrap())
}

#[test]
fn agreed_lengths_are_not_an_exact_fifth() {
    for len in 7..=512 {
        let count = count_letters(len, b'a');
        assert_eq!(count, count_letters(len, b'b'));
        assert_ne!(count * 5, len * 2);
        assert_ne!(count * 5, len * 3);
    }
}

#[test]
fn short_lengths_are_not_the_uniform_one_then_two_table() {
    let alphabet = b"aby";
    let mut sets = Vec::new();
    for len in 1..=4 {
        let mut seen = BTreeSet::new();
        let total = alphabet.len().pow(len as u32);
        for index in 0..total {
            let mut token = String::new();
            let mut rest = index;
            for _ in 0..len {
                token.push(alphabet[rest % alphabet.len()] as char);
                rest /= alphabet.len();
            }
            seen.insert(bold_prefix_len(LetterRun::parse(&token).unwrap()));
        }
        sets.push(seen);
    }
    let claimed = vec![
        BTreeSet::from([1]),
        BTreeSet::from([1]),
        BTreeSet::from([1]),
        BTreeSet::from([2]),
    ];
    assert_ne!(sets, claimed);
}

#[test]
fn prefix_takes_only_the_word() {
    fn accepts_word_only(f: fn(LetterRun<'static>) -> usize) {
        let _ = f;
    }
    accepts_word_only(bold_prefix_len);

    let src = read_src();
    assert!(src.contains("pub fn bold_prefix_len(word: letterrun<'_>) -> usize"));
    for needle in ["freq", "period", "zipf"] {
        assert!(!src.contains(needle), "src must not take a {needle} input");
    }
}

#[test]
fn public_fns_are_the_word_and_the_count() {
    let mut names = BTreeSet::new();
    for line in read_src().lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("pub fn ") {
            let name = rest.split(['(', '<']).next().unwrap();
            names.insert(name.to_string());
        }
        if line.starts_with("pub const ") {
            assert!(
                line.contains("bold_word_starts_label"),
                "unexpected public const: {line}"
            );
        }
    }
    assert_eq!(
        names,
        BTreeSet::from([
            "as_str".to_string(),
            "at".to_string(),
            "bold_prefix_len".to_string(),
            "parse".to_string(),
            "run".to_string(),
            "words".to_string(),
        ])
    );
}

fn read_src() -> String {
    fn walk(dir: &Path, out: &mut String) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, out);
            } else {
                out.push_str(&fs::read_to_string(&path).unwrap().to_ascii_lowercase());
                out.push('\n');
            }
        }
    }

    let mut out = String::new();
    walk(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"), &mut out);
    out
}
