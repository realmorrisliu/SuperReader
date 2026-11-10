#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Script {
    Latin,
    Han,
}

pub const BOLD_WORD_STARTS_LABEL: (&str, &str) = ("Bold Word Starts", "词首加粗");

pub fn label_line() -> String {
    format!(
        "{} · {}",
        BOLD_WORD_STARTS_LABEL.0, BOLD_WORD_STARTS_LABEL.1
    )
}

pub fn tier_bold_count(len: usize, script: Script) -> usize {
    if len == 0 || script != Script::Latin {
        return 0;
    }
    let n = match len {
        1 | 2 => 1,
        3 => 2,
        4 => 3,
        5 => 4,
        6 | 7 => 3,
        8 | 9 => 4,
        10 => 5,
        _ => clear_of_fifths(len, ceil_45(len).clamp(1, len)),
    };
    n.min(len)
}

pub fn syllable_bold_count(len: usize, breaks: &[usize], script: Script) -> usize {
    if len == 0 || script != Script::Latin {
        return 0;
    }
    let mut cuts: Vec<usize> = breaks
        .iter()
        .copied()
        .filter(|at| (1..len).contains(at))
        .collect();
    cuts.sort_unstable();
    cuts.dedup();
    let n = if cuts.is_empty() {
        len
    } else if len >= 9 {
        cuts.get(1).copied().unwrap_or(cuts[0])
    } else {
        cuts[0]
    };
    n.clamp(1, len)
}

pub fn onset_bold_count(word: &str, script: Script) -> usize {
    let letters: Vec<char> = word.chars().filter(|ch| ch.is_ascii_alphabetic()).collect();
    let len = letters.len();
    if len == 0 || script != Script::Latin {
        return 0;
    }
    let mut index = 0;
    while index < len && !is_vowel(letters[index], index) {
        index += 1;
    }
    if index < len {
        index += 1;
    }
    index.clamp(1, len)
}

pub fn frame_active() -> bool {
    match std::env::var("SPIKE_BOLD_WORD_STARTS") {
        Ok(value) => {
            let value = value.trim();
            !value.is_empty() && value != "0"
        }
        Err(_) => false,
    }
}

pub fn injection(lang: &str) -> Option<String> {
    let mode = std::env::var("SPIKE_BOLD_WORD_STARTS").ok()?;
    let mode = mode.trim();
    if !matches!(mode, "off" | "tiers" | "syllable" | "onset" | "preset") {
        return None;
    }
    let preset = if mode == "preset" {
        std::env::var("SPIKE_BOLD_WORD_STARTS_FILE")
            .ok()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .unwrap_or_default()
    } else {
        String::new()
    };
    let copy = std::env::var("SPIKE_BOLD_WORD_STARTS_COPY").ok().as_deref() == Some("1");
    Some(render_script(mode, lang, &preset, copy))
}

fn ceil_45(len: usize) -> usize {
    (len * 45).div_ceil(100)
}

fn clear_of_fifths(len: usize, n: usize) -> usize {
    if len < 5 || !is_fifth(len, n) {
        return n;
    }
    let up = n + 1;
    if up <= len && !is_fifth(len, up) {
        return up;
    }
    n.saturating_sub(1).max(1)
}

fn is_fifth(len: usize, n: usize) -> bool {
    n * 5 == len * 2 || n * 5 == len * 3
}

fn is_vowel(ch: char, index: usize) -> bool {
    match ch.to_ascii_lowercase() {
        'a' | 'e' | 'i' | 'o' | 'u' => true,
        'y' => index > 0,
        _ => false,
    }
}

fn cjk_lang(lang: &str) -> bool {
    let lang = lang.trim().to_ascii_lowercase();
    lang.starts_with("zh") || lang.starts_with("ja")
}

fn tier_table_json() -> String {
    let values: Vec<usize> = (0..=128)
        .map(|len| tier_bold_count(len, Script::Latin))
        .collect();
    format!("{values:?}")
}

fn render_script(mode: &str, lang: &str, preset: &str, copy: bool) -> String {
    let size = if cjk_lang(lang) { "20px" } else { "22px" };
    let copy_flag = if copy { "true" } else { "false" };
    PAGE_SCRIPT
        .replace("@@MODE@@", mode)
        .replace("@@TABLE@@", &tier_table_json())
        .replace("@@SIZE@@", size)
        .replace("@@LABEL@@", &label_line())
        .replace("@@PRESET@@", preset)
        .replace("@@COPY@@", copy_flag)
}

const PAGE_SCRIPT: &str = r#"(function () {
  var MODE = "@@MODE@@";
  var TIER = @@TABLE@@;
  var LABEL = "@@LABEL@@";
  var DO_COPY = @@COPY@@;
  @@PRESET@@
  document.documentElement.setAttribute("data-bws-label", LABEL);
  document.documentElement.setAttribute("data-bws-mode", MODE);
  var style = document.createElement("style");
  style.id = "sr-bws-frame";
  style.textContent = "html,body{background:#FFFFFF !important;color:#111111 !important;font-family:\"Iowan Old Style\",\"Songti SC\",serif !important;font-size:@@SIZE@@ !important;}span.sr-bws{font-weight:600;}";
  document.documentElement.appendChild(style);

  function tierCount(len) {
    if (len <= 0) return 0;
    if (len >= TIER.length) len = TIER.length - 1;
    return TIER[len];
  }

  function syllableCount(word) {
    var letters = word.replace(/[^A-Za-z]/g, "");
    var len = letters.length;
    if (!len) return 0;
    var cuts = [];
    if (window.spikeSyllablePoints) {
      window.spikeSyllablePoints(letters).forEach(function (at) {
        if (at > 0 && at < len) cuts.push(at);
      });
    }
    cuts.sort(function (a, b) { return a - b; });
    var n = len;
    if (cuts.length) n = (len >= 9 && cuts.length >= 2) ? cuts[1] : cuts[0];
    if (n < 1) n = 1;
    if (n > len) n = len;
    return n;
  }

  function isVowel(ch, index) {
    var c = ch.toLowerCase();
    if (c === "a" || c === "e" || c === "i" || c === "o" || c === "u") return true;
    return c === "y" && index > 0;
  }

  function onsetCount(word) {
    var letters = word.replace(/[^A-Za-z]/g, "");
    var len = letters.length;
    if (!len) return 0;
    var i = 0;
    while (i < len && !isVowel(letters.charAt(i), i)) i += 1;
    if (i < len) i += 1;
    if (i < 1) i = 1;
    if (i > len) i = len;
    return i;
  }

  window.srBoldCount = function (word) {
    var letters = String(word || "").replace(/[^A-Za-z]/g, "");
    if (!letters) return 0;
    if (MODE === "off") return 0;
    if (MODE === "preset" && typeof window.srPresetCount === "function") return window.srPresetCount(letters);
    if (MODE === "syllable") return syllableCount(letters);
    if (MODE === "onset") return onsetCount(letters);
    return tierCount(letters.length);
  };

  function skipNode(node) {
    var parent = node.parentElement;
    if (!parent) return true;
    if (parent.closest("script, style, #spike-footer, .sr-bws")) return true;
    return false;
  }

  function paintNode(node) {
    if (skipNode(node)) return;
    var text = node.textContent || "";
    if (!/[A-Za-z]/.test(text)) return;
    var parent = node.parentNode;
    if (!parent) return;
    var frag = document.createDocumentFragment();
    var re = /[A-Za-z]+/g;
    var last = 0;
    var match;
    while ((match = re.exec(text))) {
      if (match.index > last) frag.appendChild(document.createTextNode(text.slice(last, match.index)));
      var word = match[0];
      var count = window.srBoldCount(word) || 0;
      if (count < 0) count = 0;
      if (count > word.length) count = word.length;
      if (count > 0) {
        var span = document.createElement("span");
        span.className = "sr-bws";
        span.style.fontWeight = "600";
        span.textContent = word.slice(0, count);
        frag.appendChild(span);
        if (count < word.length) frag.appendChild(document.createTextNode(word.slice(count)));
      } else {
        frag.appendChild(document.createTextNode(word));
      }
      last = match.index + word.length;
    }
    if (last < text.length) frag.appendChild(document.createTextNode(text.slice(last)));
    parent.replaceChild(frag, node);
  }

  function paintAll() {
    var walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    var nodes = [];
    while (walker.nextNode()) nodes.push(walker.currentNode);
    nodes.forEach(paintNode);
  }

  function lineRects() {
    var n = 0;
    var nodes = document.querySelectorAll("p, li, blockquote");
    for (var i = 0; i < nodes.length; i++) n += nodes[i].getClientRects().length;
    return n;
  }

  function pageCount() {
    if (window.spikeBox) return window.spikeBox().pages || 0;
    return 0;
  }

  function firstWords() {
    var paras = document.querySelectorAll("p");
    var p = null;
    for (var i = 0; i < paras.length; i++) {
      if ((paras[i].textContent || "").trim().length > 40) {
        p = paras[i];
        break;
      }
    }
    if (!p) return [];
    var words = [];
    var re = /[A-Za-z]+/g;
    var text = p.textContent || "";
    var match;
    while ((match = re.exec(text))) {
      var word = match[0];
      words.push({ w: word, len: word.length, n: window.srBoldCount(word) || 0 });
      if (words.length >= 80) break;
    }
    return words;
  }

  function post(body) {
    body.kind = "bws";
    body.mode = MODE;
    body.label = LABEL;
    if (!window.webkit || !window.webkit.messageHandlers || !window.webkit.messageHandlers.spike) return;
    window.webkit.messageHandlers.spike.postMessage(JSON.stringify(body));
  }

  function copyPlain() {
    var paras = document.querySelectorAll("p");
    var p = null;
    for (var i = 0; i < paras.length; i++) {
      if ((paras[i].textContent || "").indexOf("acknowledged") >= 0) {
        p = paras[i];
        break;
      }
    }
    if (!p) p = paras[0];
    if (!p) return { text: "", copied: false, markup: false };
    var range = document.createRange();
    range.selectNodeContents(p);
    var sel = window.getSelection();
    sel.removeAllRanges();
    sel.addRange(range);
    var text = sel.toString();
    var copied = false;
    try { copied = !!document.execCommand("copy"); } catch (err) { copied = false; }
    return { text: text, copied: copied, markup: text.indexOf("<") >= 0 };
  }

  var ran = false;
  function run() {
    if (ran) return;
    ran = true;
    var beforePages = pageCount();
    var beforeLines = lineRects();
    var beforeText = document.body.textContent || "";
    var held = window.spikeClearPaint ? window.spikeClearPaint() : null;
    if (MODE !== "off") paintAll();
    var afterText = document.body.textContent || "";
    if (held && window.spikePaint) window.spikePaint(held.start, held.end);
    setTimeout(function () {
      var copy = DO_COPY ? copyPlain() : { text: "", copied: false, markup: false };
      post({
        phase: "measured",
        pagesBefore: beforePages,
        linesBefore: beforeLines,
        pagesAfter: pageCount(),
        linesAfter: lineRects(),
        textSame: beforeText === afterText,
        words: firstWords(),
        copyText: copy.text,
        copied: copy.copied,
        copyMarkup: copy.markup,
        highlight: held
      });
    }, 700);
  }

  var last = -1;
  var stable = 0;
  var timer = setInterval(function () {
    var n = document.querySelectorAll("p[data-kp], li[data-kp], blockquote[data-kp]").length;
    if (n > 0 && n === last) stable += 1;
    else stable = 0;
    last = n;
    if (stable >= 4) {
      clearInterval(timer);
      run();
    }
  }, 150);
  setTimeout(function () {
    clearInterval(timer);
    run();
  }, 6000);
})();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn claimed_short_table() -> Vec<usize> {
        vec![1, 1, 1, 2]
    }

    #[test]
    fn tier_table_for_lengths_1_through_12() {
        let got: Vec<usize> = (1..=12)
            .map(|len| tier_bold_count(len, Script::Latin))
            .collect();
        assert_eq!(got, vec![1, 1, 2, 3, 4, 3, 3, 4, 4, 5, 5, 6]);
    }

    #[test]
    fn tier_table_is_not_the_claimed_short_table_and_avoids_fifth_ratios() {
        let got: Vec<usize> = (1..=4)
            .map(|len| tier_bold_count(len, Script::Latin))
            .collect();
        assert_ne!(got, claimed_short_table());
        assert_ne!(tier_bold_count(4, Script::Latin), 2);
        for len in 1..=30 {
            let n = tier_bold_count(len, Script::Latin);
            assert!(n <= len, "{len}");
            if len >= 5 {
                assert_ne!(n * 5, len * 2, "len {len} count {n}");
                assert_ne!(n * 5, len * 3, "len {len} count {n}");
                let ratio = n as f64 / len as f64;
                assert!((ratio - 0.4).abs() > 1e-9, "{len}");
                assert!((ratio - 0.6).abs() > 1e-9, "{len}");
            }
        }
    }

    #[test]
    fn count_takes_length_and_script_not_a_frequency() {
        assert_eq!(tier_bold_count(8, Script::Latin), 4);
        assert_eq!(tier_bold_count(8, Script::Han), 0);
        assert_eq!(syllable_bold_count(8, &[3], Script::Han), 0);
        assert_eq!(onset_bold_count("truth", Script::Han), 0);
        assert_eq!(label_line(), "Bold Word Starts · 词首加粗");
    }

    #[test]
    fn syllable_uses_the_first_cut_and_the_second_cut_on_long_words() {
        assert_eq!(syllable_bold_count(5, &[2], Script::Latin), 2);
        assert_eq!(syllable_bold_count(8, &[3, 6], Script::Latin), 3);
        assert_eq!(syllable_bold_count(9, &[3, 6], Script::Latin), 6);
        assert_eq!(syllable_bold_count(12, &[4, 8, 10], Script::Latin), 8);
        assert_eq!(syllable_bold_count(4, &[], Script::Latin), 4);
        assert_eq!(syllable_bold_count(1, &[], Script::Latin), 1);
    }

    #[test]
    fn onset_keeps_the_opening_consonants_and_the_following_vowel() {
        assert_eq!(onset_bold_count("a", Script::Latin), 1);
        assert_eq!(onset_bold_count("to", Script::Latin), 2);
        assert_eq!(onset_bold_count("the", Script::Latin), 3);
        assert_eq!(onset_bold_count("that", Script::Latin), 3);
        assert_eq!(onset_bold_count("pride", Script::Latin), 3);
        assert_eq!(onset_bold_count("truth", Script::Latin), 3);
        assert_eq!(onset_bold_count("strength", Script::Latin), 4);
        assert_eq!(onset_bold_count("yellow", Script::Latin), 2);
        assert_eq!(onset_bold_count("universally", Script::Latin), 1);
    }

    #[test]
    fn injection_embeds_the_tier_table_and_the_single_label() {
        let js = render_script("tiers", "en", "", false);
        let table = (0..=12)
            .map(|len| tier_bold_count(len, Script::Latin).to_string())
            .collect::<Vec<_>>()
            .join(", ");
        assert!(js.contains(&table), "{table}");
        assert!(js.contains("Bold Word Starts · 词首加粗"));
        assert_eq!(js.matches("Bold Word Starts").count(), 1);
    }
}
