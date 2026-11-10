use std::collections::HashMap;

const RAW: &str = include_str!("../../../spikes/justify/vendor/hyphenated-en-us/patterns.json");

pub struct Hyphenator {
    patterns: HashMap<String, Vec<u8>>,
    exceptions: HashMap<String, Vec<usize>>,
}

impl Hyphenator {
    pub fn load() -> Self {
        let mut patterns = HashMap::new();
        let mut exceptions = HashMap::new();
        let mut mode = 0;
        for quoted in quoted_strings(RAW) {
            if quoted == "patterns" {
                mode = 1;
                continue;
            }
            if quoted == "exceptions" {
                mode = 2;
                continue;
            }
            if quoted == "id" || quoted == "en-us" {
                continue;
            }
            if mode == 1 {
                let (letters, levels) = split_pattern(quoted);
                patterns.insert(letters, levels);
            } else if mode == 2 {
                let word: String = quoted.chars().filter(|c| *c != '-').collect();
                let mut breaks = Vec::new();
                let mut n = 0;
                for c in quoted.chars() {
                    if c == '-' {
                        breaks.push(n);
                    } else {
                        n += 1;
                    }
                }
                exceptions.insert(word, breaks);
            }
        }
        Self { patterns, exceptions }
    }

    /// Break indexes are character counts from the start of `word`.
    pub fn breaks(&self, word: &str, left_min: usize, right_min: usize) -> Vec<usize> {
        let lower: String = word.chars().flat_map(|c| c.to_lowercase()).collect();
        let chars: Vec<char> = lower.chars().collect();
        let n = chars.len();
        if n < left_min + right_min {
            return Vec::new();
        }
        if let Some(found) = self.exceptions.get(&lower) {
            return found
                .iter()
                .copied()
                .filter(|i| *i >= left_min && *i + right_min <= n)
                .collect();
        }
        let mut padded = Vec::with_capacity(n + 2);
        padded.push('.');
        padded.extend(chars.iter().copied());
        padded.push('.');
        let mut score = vec![0u8; n + 1];
        for start in 0..padded.len() {
            for end in start + 1..=padded.len() {
                let key: String = padded[start..end].iter().collect();
                let Some(levels) = self.patterns.get(&key) else {
                    continue;
                };
                for (i, level) in levels.iter().copied().enumerate() {
                    let at = start + i;
                    if at >= 1 && at - 1 <= n && level > score[at - 1] {
                        score[at - 1] = level;
                    }
                }
            }
        }
        let mut out = Vec::new();
        for i in left_min..=n.saturating_sub(right_min) {
            if score[i] % 2 == 1 {
                out.push(i);
            }
        }
        out
    }
}

fn split_pattern(pattern: &str) -> (String, Vec<u8>) {
    let mut letters = String::new();
    let mut levels = vec![0u8];
    for c in pattern.chars() {
        if c.is_ascii_digit() {
            if let Some(last) = levels.last_mut() {
                *last = c.to_digit(10).unwrap_or(0) as u8;
            }
        } else {
            letters.push(c);
            levels.push(0);
        }
    }
    (letters, levels)
}

fn quoted_strings(raw: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let bytes = raw.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            let start = i + 1;
            i += 1;
            while i < bytes.len() && bytes[i] != b'"' {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                i += 1;
            }
            if i <= raw.len() {
                out.push(&raw[start..i]);
            }
        }
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hyphenation_splits_into_hy_phen_ation() {
        let hyphen = Hyphenator::load();
        let breaks = hyphen.breaks("hyphenation", 2, 3);
        assert!(
            breaks.contains(&2) && breaks.contains(&6),
            "breaks {breaks:?}"
        );
        let acknowledged = hyphen.breaks("acknowledged", 3, 3);
        let possession = hyphen.breaks("possession", 3, 3);
        let universally = hyphen.breaks("universally", 3, 3);
        assert!(!acknowledged.is_empty(), "acknowledged {acknowledged:?}");
        assert!(!possession.is_empty(), "possession {possession:?}");
        assert!(!universally.is_empty(), "universally {universally:?}");
    }
}
