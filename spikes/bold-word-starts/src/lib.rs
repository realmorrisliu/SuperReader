pub const BOLD_WORD_STARTS_LABEL: (&str, &str) = ("Bold Word Starts", "词首加粗");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LetterRun<'a>(&'a str);

impl<'a> LetterRun<'a> {
    pub fn parse(token: &'a str) -> Option<Self> {
        if token.is_empty() || !token.bytes().all(|byte| byte.is_ascii_alphabetic()) {
            return None;
        }
        Some(Self(token))
    }

    pub fn as_str(self) -> &'a str {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WordSpan<'a> {
    at: usize,
    run: LetterRun<'a>,
}

impl<'a> WordSpan<'a> {
    pub fn at(self) -> usize {
        self.at
    }

    pub fn run(self) -> LetterRun<'a> {
        self.run
    }
}

pub struct Words<'a> {
    text: &'a str,
    index: usize,
}

impl<'a> Iterator for Words<'a> {
    type Item = WordSpan<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let rest = &self.text[self.index..];
        let mut start = None;
        let mut end = self.index;
        for (offset, ch) in rest.char_indices() {
            let abs = self.index + offset;
            if ch.is_ascii_alphabetic() {
                if start.is_none() {
                    start = Some(abs);
                }
                end = abs + ch.len_utf8();
            } else if start.is_some() {
                break;
            }
        }
        let at = start?;
        self.index = end;
        Some(WordSpan {
            at,
            run: LetterRun(&self.text[at..end]),
        })
    }
}

pub fn words(text: &str) -> Words<'_> {
    Words { text, index: 0 }
}

pub fn bold_prefix_len(word: LetterRun<'_>) -> usize {
    let len = word.0.len();
    if len <= 6 {
        onset_len(word.0)
    } else {
        tier_len(len)
    }
}

fn onset_len(letters: &str) -> usize {
    let bytes = letters.as_bytes();
    let len = bytes.len();
    let mut index = 0;
    while index < len && !is_vowel(bytes[index], index) {
        index += 1;
    }
    if index < len {
        index += 1;
    }
    index.clamp(1, len)
}

fn is_vowel(byte: u8, index: usize) -> bool {
    match byte.to_ascii_lowercase() {
        b'a' | b'e' | b'i' | b'o' | b'u' => true,
        b'y' => index > 0,
        _ => false,
    }
}

fn tier_len(len: usize) -> usize {
    match len {
        7 => 3,
        9 => 4,
        n => (n * 45).div_ceil(100),
    }
}
