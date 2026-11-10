pub const BOLD_WORD_STARTS_LABEL: (&str, &str) = ("Bold Word Starts", "词首加粗");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LetterRun<'a>(&'a str);

impl<'a> LetterRun<'a> {
    pub fn parse(token: &'a str) -> Option<Self> {
        let mut chars = token.chars();
        if !chars.next().is_some_and(is_latin_letter) {
            return None;
        }
        if chars.all(|ch| is_latin_letter(ch) || is_combining_mark(ch)) {
            Some(Self(token))
        } else {
            None
        }
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
    /// Byte offset of the run in the source text.
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
            let in_run = start.is_some();
            if is_latin_letter(ch) || (in_run && is_combining_mark(ch)) {
                if start.is_none() {
                    start = Some(abs);
                }
                end = abs + ch.len_utf8();
            } else if in_run {
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

/// Char count of the bold prefix, including combining marks on those letters.
///
/// The painter takes this many chars from the run. `WordSpan::at` is a byte
/// offset into the source text and is not added to this count. The rule's
/// length is the number of Latin letters. A combining mark does not add one.
pub fn bold_prefix_len(word: LetterRun<'_>) -> usize {
    let steps = letter_steps(word.0);
    let len = steps.len();
    let letters = if len <= 6 {
        onset_letters(&steps)
    } else {
        tier_len(len)
    };
    let byte_end = steps[letters - 1].end;
    word.0[..byte_end].chars().count()
}

struct LetterStep {
    ch: char,
    end: usize,
}

fn letter_steps(word: &str) -> Vec<LetterStep> {
    let mut steps = Vec::new();
    let mut chars = word.char_indices().peekable();
    while let Some((start, ch)) = chars.next() {
        if !is_latin_letter(ch) {
            continue;
        }
        let mut end = start + ch.len_utf8();
        while let Some(&(mark_at, mark)) = chars.peek() {
            if !is_combining_mark(mark) {
                break;
            }
            end = mark_at + mark.len_utf8();
            chars.next();
        }
        steps.push(LetterStep { ch, end });
    }
    steps
}

fn onset_letters(steps: &[LetterStep]) -> usize {
    let len = steps.len();
    let mut index = 0;
    while index < len && !is_vowel(steps[index].ch, index) {
        index += 1;
    }
    if index < len {
        index += 1;
    }
    index.clamp(1, len)
}

fn is_latin_letter(ch: char) -> bool {
    matches!(
        ch,
        'A'..='Z'
            | 'a'..='z'
            | '\u{00C0}'..='\u{00D6}'
            | '\u{00D8}'..='\u{00F6}'
            | '\u{00F8}'..='\u{00FF}'
            | '\u{0100}'..='\u{017F}'
            | '\u{0180}'..='\u{024F}'
            | '\u{1E00}'..='\u{1EFF}'
    )
}

fn is_combining_mark(ch: char) -> bool {
    matches!(ch, '\u{0300}'..='\u{036F}')
}

fn is_vowel(ch: char, index: usize) -> bool {
    matches!(
        ch,
        'a' | 'A'
            | 'à'
            | 'À'
            | 'á'
            | 'Á'
            | 'â'
            | 'Â'
            | 'ä'
            | 'Ä'
            | 'ã'
            | 'Ã'
            | 'å'
            | 'Å'
            | 'æ'
            | 'Æ'
            | 'e'
            | 'E'
            | 'é'
            | 'É'
            | 'è'
            | 'È'
            | 'ê'
            | 'Ê'
            | 'ë'
            | 'Ë'
            | 'i'
            | 'I'
            | 'í'
            | 'Í'
            | 'ì'
            | 'Ì'
            | 'î'
            | 'Î'
            | 'ï'
            | 'Ï'
            | 'o'
            | 'O'
            | 'ó'
            | 'Ó'
            | 'ò'
            | 'Ò'
            | 'ô'
            | 'Ô'
            | 'ö'
            | 'Ö'
            | 'õ'
            | 'Õ'
            | 'ø'
            | 'Ø'
            | 'œ'
            | 'Œ'
            | 'u'
            | 'U'
            | 'ú'
            | 'Ú'
            | 'ù'
            | 'Ù'
            | 'û'
            | 'Û'
            | 'ü'
            | 'Ü'
    ) || (index > 0 && matches!(ch, 'y' | 'Y'))
}

fn tier_len(len: usize) -> usize {
    match len {
        7 => 3,
        9 => 4,
        n => (n * 45).div_ceil(100),
    }
}
