use std::collections::BTreeSet;

use spike_bold_word_starts::{bold_prefix_len, LetterRun};

fn count_letters(len: usize, byte: u8) -> usize {
    let token = vec![byte; len];
    let token = std::str::from_utf8(&token).unwrap();
    bold_prefix_len(LetterRun::parse(token).unwrap())
}

#[test]
fn agreed_lengths_are_not_an_exact_fifth() {
    for len in 7..=128 {
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

    let src = include_str!("../src/lib.rs").to_ascii_lowercase();
    assert!(src.contains("pub fn bold_prefix_len(word: letterrun<'_>) -> usize"));
    for needle in ["freq", "period", "zipf"] {
        assert!(
            !src.contains(needle),
            "lib.rs must not take a {needle} input"
        );
    }
}
