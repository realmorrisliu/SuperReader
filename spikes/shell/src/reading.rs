pub const FRAME_FLOOR_MS: f64 = 16.7;
pub const PAPER: &str = "#f3eee4";
pub const SLIDE_MS: u16 = 160;
pub const TITLE_DELAY_MS: u32 = 400;
pub const TITLE_OVER_MS: u32 = 1;
pub const FONT_STACK: &str = "\"Iowan Old Style\", \"Songti SC\", serif";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Trim {
    pub id: &'static str,
    pub label: &'static str,
    pub width_mm: u16,
    pub height_mm: u16,
}

pub const TRIMS: [Trim; 3] = [
    Trim {
        id: "32kai",
        label: "32开",
        width_mm: 130,
        height_mm: 184,
    },
    Trim {
        id: "paperback",
        label: "Paperback",
        width_mm: 129,
        height_mm: 198,
    },
    Trim {
        id: "trade",
        label: "US Trade",
        width_mm: 140,
        height_mm: 216,
    },
];

pub const PAGE_OVERRIDES: &[(&str, &str)] = &[
    (
        "html and body background, margin, width, max-width, and height",
        "A publisher body width or margin shrinks the column box, so the chapter stays one page.",
    ),
    (
        "body columns, overflow, and page padding",
        "The page box, the measure, and the page margins belong to SuperReader.",
    ),
    (
        "direct children width, height, side margins, and float",
        "A wrapper with a fixed width or height becomes a single column.",
    ),
    (
        "position fixed rewritten to absolute",
        "A fixed element paints on every column.",
    ),
    (
        "a two pixel clip on the right edge",
        "The next column otherwise draws a dark line on the page edge.",
    ),
];

pub const DEFAULT_TRIM: &str = "paperback";

pub fn trim(id: &str) -> &'static Trim {
    TRIMS
        .iter()
        .find(|item| item.id == id)
        .unwrap_or_else(|| TRIMS.iter().find(|item| item.id == DEFAULT_TRIM).unwrap())
}

pub fn page_ratio(declared: Option<(f64, f64)>, item: &Trim) -> (f64, f64, &'static str) {
    if let Some((width, height)) = declared {
        if width > 0.0 && height > 0.0 {
            return (width, height, "declared");
        }
    }
    (f64::from(item.width_mm), f64::from(item.height_mm), "trim")
}

pub fn fit_page(window_w: f64, window_h: f64, ratio_w: f64, ratio_h: f64) -> (f64, f64, f64, f64) {
    let window_w = window_w.max(1.0);
    let window_h = window_h.max(1.0);
    let ratio_w = ratio_w.max(1.0);
    let ratio_h = ratio_h.max(1.0);
    let page = ratio_w / ratio_h;
    let (width, height) = if window_w / window_h > page {
        (window_h * page, window_h)
    } else {
        (window_w, window_w / page)
    };
    (
        ((window_w - width) / 2.0).max(0.0),
        ((window_h - height) / 2.0).max(0.0),
        width,
        height,
    )
}

pub fn ratio_within(width: f64, height: f64, want_w: f64, want_h: f64) -> bool {
    if height <= 0.0 || want_h <= 0.0 {
        return false;
    }
    let got = width / height;
    let want = want_w / want_h;
    (got - want).abs() / want <= 0.01
}

struct Face {
    size_px: u8,
    line_height: &'static str,
    measure_ch: u8,
    margin_top: u16,
    margin_side: u8,
    margin_bottom: u16,
    cjk: bool,
}

fn cjk_lang(lang: &str) -> bool {
    let lang = lang.trim().to_ascii_lowercase();
    lang.starts_with("zh") || lang.starts_with("ja")
}

fn face(lang: &str) -> Face {
    if cjk_lang(lang) {
        Face {
            size_px: 20,
            line_height: "1.70",
            measure_ch: 64,
            margin_top: 72,
            margin_side: 11,
            margin_bottom: 64,
            cjk: true,
        }
    } else {
        Face {
            size_px: 22,
            line_height: "1.48",
            measure_ch: 48,
            margin_top: 88,
            margin_side: 14,
            margin_bottom: 72,
            cjk: false,
        }
    }
}

pub fn clean_lang(lang: &str) -> String {
    let lang = lang.trim();
    if lang.is_empty() {
        return "en".to_string();
    }
    lang.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

pub fn fallback_css(lang: &str) -> String {
    let face = face(lang);
    let cjk = if face.cjk {
        "text-autospace: normal; line-break: strict;"
    } else {
        ""
    };
    format!(
        r#"@layer spike-fallback {{
  :where(html, body) {{
    font-family: {stack};
    font-size: {size}px;
    line-height: {leading};
    text-align: justify;
    hyphens: auto;
    -webkit-hyphens: auto;
    -webkit-hyphenate-limit-before: 2;
    -webkit-hyphenate-limit-after: 3;
    -webkit-hyphenate-limit-lines: 2;
    hanging-punctuation: first allow-end last;
    {cjk}
  }}
  :where(p, li, blockquote) {{
    text-indent: 2em;
    margin-top: 0;
    margin-bottom: 0;
  }}
}}
"#,
        stack = FONT_STACK,
        size = face.size_px,
        leading = face.line_height,
        cjk = cjk,
    )
}

pub fn page_css(lang: &str) -> String {
    let face = face(lang);
    format!(
        r#"
html, body {{
  background: {paper} !important;
  color: #1c1915 !important;
  margin: 0 !important;
  width: auto !important;
  max-width: none !important;
  min-width: 0 !important;
  height: 100vh !important;
}}
body {{
  box-sizing: border-box !important;
  margin-left: 0 !important;
  margin-right: 0 !important;
  height: 100vh !important;
  padding: {top}px 0 {bottom}px !important;
  column-gap: 0 !important;
  column-fill: auto !important;
  overflow: hidden !important;
}}
html {{
  overflow: hidden !important;
}}
p, li, blockquote, h1, h2, h3 {{
  padding-left: max({side}%, calc((100% - {measure}ch) / 2)) !important;
  padding-right: max({side}%, calc((100% - {measure}ch) / 2)) !important;
  box-sizing: border-box !important;
}}
body > * {{
  width: auto !important;
  max-width: none !important;
  height: auto !important;
  min-height: 0 !important;
  margin-left: 0 !important;
  margin-right: 0 !important;
  float: none !important;
}}
body::-webkit-scrollbar {{ display: none; }}
"#,
        paper = PAPER,
        top = face.margin_top,
        bottom = face.margin_bottom,
        side = face.margin_side,
        measure = face.measure_ch,
    )
}

pub fn prepare_html(html: &str, lang: &str) -> String {
    let lang = clean_lang(lang);
    let mut html = html.to_string();
    set_lang(&mut html, &lang);
    let style = format!(
        "<style id=\"spike-fallback\">{}</style>",
        fallback_css(&lang)
    );
    let lower = html.to_ascii_lowercase();
    if let Some(at) = lower.find("<head>") {
        html.insert_str(at + "<head>".len(), &style);
    } else if let Some(at) = lower.find("<html") {
        if let Some(end) = html[at..].find('>') {
            html.insert_str(at + end + 1, &style);
        } else {
            html.insert_str(0, &style);
        }
    } else {
        html.insert_str(0, &style);
    }
    html
}

fn set_lang(html: &mut String, lang: &str) {
    let lower = html.to_ascii_lowercase();
    let Some(start) = lower.find("<html") else {
        return;
    };
    let Some(end_rel) = html[start..].find('>') else {
        return;
    };
    let end = start + end_rel;
    let tag = html[start..end].to_string();
    let tag_lower = tag.to_ascii_lowercase();
    if let Some(rel) = tag_lower.find("lang=") {
        let value_at = start + rel + "lang=".len();
        let bytes = html.as_bytes();
        if value_at >= bytes.len() {
            return;
        }
        let quote = bytes[value_at];
        if quote == b'"' || quote == b'\'' {
            let close = html[value_at + 1..]
                .find(quote as char)
                .map(|i| value_at + 1 + i);
            if let Some(close) = close {
                html.replace_range(value_at + 1..close, lang);
            }
        }
    } else {
        html.insert_str(start + "<html".len(), &format!(" lang=\"{lang}\""));
    }
}

pub fn footer_zh(page: u32, pages: u32) -> (String, String) {
    let pages = pages.max(1);
    let page = page.clamp(1, pages);
    let left = pages.saturating_sub(page);
    (format!("{page}/{pages}页"), format!("本章还剩{left}页"))
}

pub fn title_alpha(shown_ms: u32) -> f32 {
    fade_down(shown_ms, TITLE_DELAY_MS, TITLE_OVER_MS).clamp(0.0, 1.0)
}

fn fade_down(elapsed: u32, delay: u32, over: u32) -> f32 {
    let value = if elapsed <= delay {
        1.0
    } else if over == 0 {
        0.0
    } else {
        1.0 - (elapsed - delay) as f32 / over as f32
    };
    if value < 0.08 { 0.0 } else { value }
}

pub fn frames_within_floor(frames: &[f64]) -> bool {
    let missed = FRAME_FLOOR_MS * 2.0;
    !frames.is_empty() && frames.iter().all(|ms| ms.is_finite() && *ms < missed)
}

pub fn justify_source(enabled: bool) -> String {
    include_str!("../../justify/justify.js")
        .replace(
            "__TEX_LAYOUT__",
            include_str!("../../justify/vendor/tex-linebreak/layout.js"),
        )
        .replace(
            "__PATTERNS_US__",
            include_str!("../../justify/vendor/hyphenated-en-us/patterns.json"),
        )
        .replace(
            "__PATTERNS_GB__",
            include_str!("../../justify/vendor/hyphenated-en-gb/patterns.json"),
        )
        .replace("__JUSTIFY_ON__", if enabled { "true" } else { "false" })
}

pub fn page_script(lang: &str) -> String {
    let lang = clean_lang(lang);
    let fallback = serde_json::to_string(&fallback_css(&lang)).unwrap_or_else(|_| "\"\"".to_string());
    let page = serde_json::to_string(&page_css(&lang)).unwrap_or_else(|_| "\"\"".to_string());
    SCRIPT
        .replace("__LANG__", &lang)
        .replace("__FALLBACK__", &fallback)
        .replace("__PAGE__", &page)
        .replace("__MS__", &SLIDE_MS.to_string())
        .replace("__PAPER__", PAPER)
}

const SCRIPT: &str = r#"
(function () {
  const fallback = document.createElement('style');
  fallback.id = 'spike-fallback-late';
  fallback.textContent = __FALLBACK__;
  (document.head || document.documentElement).prepend(fallback);
  const page = document.createElement('style');
  page.id = 'spike-page';
  page.textContent = __PAGE__;
  (document.head || document.documentElement).appendChild(page);
  document.documentElement.lang = "__LANG__";
  if (document.body) document.body.lang = "__LANG__";
  document.querySelectorAll('*').forEach(function (el) {
    if (getComputedStyle(el).position === 'fixed') {
      el.style.setProperty('position', 'absolute', 'important');
    }
  });
  window.spikeOwnsBox = true;
  const DURATION = __MS__;
  const PAPER = "__PAPER__";
  const SWIPE = 48;
  const root = document.body;
  let busy = false;
  let queued = 0;
  let acc = 0;
  let armed = true;
  let shift = 0;
  let measured = false;
  let layoutReport = null;
  let justifyReady = false;
  function seek(x) {
    shift = x;
    root.scrollLeft = x;
    document.documentElement.scrollLeft = x;
    root.style.transform = "none";
  }
  function pageWidth() {
    return Math.max(1, Math.round(window.innerWidth));
  }
  function columnStride() {
    return pageWidth();
  }
  function fitColumns() {
    const stride = columnStride();
    root.style.setProperty('column-width', stride + 'px', 'important');
    root.style.setProperty('overflow', 'hidden', 'important');
    document.documentElement.style.setProperty('overflow', 'hidden', 'important');
    document.documentElement.style.setProperty('clip-path', 'inset(0)', 'important');
  }
  function reflowPage() {
    fitColumns();
    if (justifyReady && window.spikeJustify) window.spikeJustify.onPageBoxChanged();
  }
  fitColumns();
  window.addEventListener('resize', reflowPage);
  function column() {
    const w = columnStride();
    const pages = Math.max(1, Math.round(Math.max(root.scrollWidth, shift + w) / w));
    const page = Math.min(pages, Math.round(shift / w) + 1);
    return { page: page, pages: pages, w: w };
  }
  function marks() {
    const text = document.body.innerText || "";
    let tofu = 0;
    let gap = 0;
    for (let i = 0; i < text.length; i++) {
      const c = text.charCodeAt(i);
      if (c === 0xFFFD || c === 0x25A1 || (c >= 0xE000 && c <= 0xF8FF)) tofu++;
      if (i + 1 < text.length && cjk(c) && text[i + 1] === " ") {
        let j = i + 1;
        while (j < text.length && text[j] === " ") j++;
        if (j < text.length && cjk(text.charCodeAt(j))) gap++;
      }
    }
    return { tofu: tofu, cjk_gap: gap };
  }
  function cjk(c) {
    return (c >= 0x3400 && c <= 0x9FFF) || (c >= 0xF900 && c <= 0xFAFF) || (c >= 0x3001 && c <= 0x303F);
  }
  function sample() {
    const node = document.querySelector('p') || document.body;
    const cs = getComputedStyle(node);
    const body = getComputedStyle(document.body);
    let fontStatus = 'unknown';
    try {
      if (document.fonts && document.fonts.check('19px "Literata"')) fontStatus = 'loaded';
      else if (document.fonts) fontStatus = 'missing';
    } catch (err) {
      fontStatus = 'missing';
    }
    return {
      font: cs.fontFamily,
      size: cs.fontSize,
      leading: cs.lineHeight,
      indent: cs.textIndent,
      align: cs.textAlign,
      hanging: body.hangingPunctuation || body.webkitHangingPunctuation || "",
      autospace: body.textAutospace || "",
      fontStatus: fontStatus
    };
  }
  function gapRatio() {
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d');
    if (!ctx) return 0;
    ctx.font = getComputedStyle(document.body).font;
    const normal = ctx.measureText(' ').width || 1;
    let max = normal;
    const paras = document.querySelectorAll('p');
    const count = Math.min(paras.length, 6);
    for (let i = 0; i < count; i++) {
      const walker = document.createTreeWalker(paras[i], NodeFilter.SHOW_TEXT);
      while (walker.nextNode()) {
        const node = walker.currentNode;
        const text = node.textContent || '';
        let spaces = 0;
        for (let at = 0; at < text.length && spaces < 30; at++) {
          if (text[at] !== ' ' || at === 0 || at + 1 >= text.length) continue;
          if (text[at - 1] === ' ' || text[at + 1] === ' ') continue;
          const left = document.createRange();
          left.setStart(node, at - 1);
          left.setEnd(node, at);
          const right = document.createRange();
          right.setStart(node, at + 1);
          right.setEnd(node, at + 2);
          const a = left.getBoundingClientRect();
          const b = right.getBoundingClientRect();
          if (!a.width || !b.width || Math.abs(a.top - b.top) > 2) continue;
          if (a.top < 0 || a.bottom > window.innerHeight || a.left < 0 || b.right > window.innerWidth) continue;
          const gap = b.left - a.right;
          if (gap > max) max = gap;
          spaces++;
        }
      }
    }
    return max / normal;
  }
  function post(extra) {
    const box = column();
    const body = { page: box.page, pages: box.pages, sx: root.scrollLeft, view_w: window.innerWidth, leaf: !!document.getElementById("spike-leaf") };
    const mark = marks();
    body.tofu = mark.tofu;
    body.cjk_gap = mark.cjk_gap;
    if (layoutReport) {
      body.layout = layoutReport;
      body.gap_ratio = layoutReport.gap_ratio;
    }
    if (!measured && box.pages > 0) {
      measured = true;
      body.sample = sample();
      if (!layoutReport) body.gap_ratio = gapRatio();
      body.plain = (document.body.innerText || '').slice(0, 24000);
    }
    if (extra) {
      Object.keys(extra).forEach(function (key) { body[key] = extra[key]; });
    }
    window.webkit.messageHandlers.spike.postMessage(JSON.stringify(body));
  }
  function locate(offset) {
    const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    let seen = 0;
    while (walker.nextNode()) {
      const node = walker.currentNode;
      const next = seen + node.textContent.length;
      if (next > offset) return { node: node, off: Math.max(0, offset - seen) };
      seen = next;
    }
    return null;
  }
  function offsetOf(node, off) {
    const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    let seen = 0;
    while (walker.nextNode()) {
      if (walker.currentNode === node) return seen + off;
      seen += walker.currentNode.textContent.length;
    }
    return seen;
  }
  window.spikeHold = function () {
    const x = Math.max(24, Math.min(80, window.innerWidth * 0.2));
    const y = Math.max(48, Math.min(140, window.innerHeight * 0.18));
    const caret = document.caretRangeFromPoint(x, y) || document.caretRangeFromPoint(x, y + 40);
    let offset = 0;
    if (caret && caret.startContainer) {
      const node = caret.startContainer.nodeType === 3
        ? caret.startContainer
        : (caret.startContainer.firstChild || null);
      if (node && node.nodeType === 3) offset = offsetOf(node, caret.startOffset || 0);
    }
    post({ hold: offset });
    return offset;
  };
  window.spikeReveal = function (offset) {
    requestAnimationFrame(function () {
      requestAnimationFrame(function () {
        const found = locate(offset);
        if (!found) {
          post();
          return;
        }
        const prev = root.style.transform;
        root.style.transform = 'none';
        root.scrollLeft = 0;
        const range = document.createRange();
        range.setStart(found.node, found.off);
        range.setEnd(found.node, Math.min(found.node.textContent.length, found.off + 1));
        const rect = range.getBoundingClientRect();
        const w = columnStride();
        const index = Math.max(0, Math.floor((rect.left + 8) / w));
        root.style.transform = prev;
        seek(index * w);
        measured = false;
        post({ pinned: offset });
      });
    });
  };
  function settle(frames) {
    document.documentElement.style.transform = "none";
    const leaf = document.getElementById("spike-leaf");
    if (leaf) leaf.remove();
    post({ frames: frames });
    busy = false;
    const next = queued;
    queued = 0;
    if (next) begin(next);
  }
  window.spikeTurn = function (delta) {
    delta = delta < 0 ? -1 : 1;
    if (busy) {
      queued = delta;
      return "queued";
    }
    begin(delta);
    return "ok";
  };
  function begin(delta) {
    const box = column();
    const target = box.page + delta;
    if (target < 1 || target > box.pages) {
      post({ frames: [] });
      return;
    }
    busy = true;
    slide(box, delta);
  }
  function slide(box, delta) {
    seek(box.w * (box.page + delta - 1));
    const leaf = document.createElement("div");
    leaf.id = "spike-leaf";
    leaf.style.cssText = "position:fixed;inset:0;z-index:5;pointer-events:none;background:" + PAPER + ";will-change:transform";
    document.documentElement.appendChild(leaf);
    const frames = [];
    let last = 0;
    const t0 = performance.now();
    function frame(t) {
      if (last) frames.push(t - last);
      last = t;
      const u = Math.min(1, (t - t0) / DURATION);
      const eased = (1 - Math.cos(Math.PI * u)) / 2;
      leaf.style.transform = "translate3d(" + (delta * eased * box.w) + "px,0,0)";
      if (u < 1) {
        requestAnimationFrame(frame);
        return;
      }
      settle(frames);
    }
    requestAnimationFrame(frame);
  }
  window.addEventListener("keydown", function (event) {
    if (event.repeat) return;
    if (event.key !== "ArrowRight" && event.key !== "ArrowLeft") return;
    event.preventDefault();
    window.spikeTurn(event.key === "ArrowRight" ? 1 : -1);
  }, true);
  window.addEventListener("wheel", function (event) {
    if (Math.abs(event.deltaX) <= Math.abs(event.deltaY)) return;
    event.preventDefault();
    if (busy) return;
    acc += event.deltaX;
    if (!armed) {
      if (Math.abs(event.deltaX) < 1.5) {
        armed = true;
        acc = 0;
      }
      return;
    }
    if (Math.abs(acc) < SWIPE) return;
    const delta = acc > 0 ? 1 : -1;
    acc = 0;
    armed = false;
    window.spikeTurn(delta);
  }, { passive: false, capture: true });
  if (window.spikeJustify) {
    window.spikeJustify.install({
      onReport: function (report) {
        layoutReport = report;
        post();
      }
    });
    justifyReady = true;
    window.spikeJustify.onPageBoxChanged();
  }
  if (document.fonts && document.fonts.ready) {
    document.fonts.ready.then(function () {
      measured = false;
      if (window.spikeJustify) window.spikeJustify.onPageBoxChanged();
      else post();
    });
  }
  requestAnimationFrame(function () { post(); });
})();
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chinese_footer_matches_the_books_line() {
        let (left, right) = footer_zh(590, 1146);
        assert_eq!(left, "590/1146页");
        assert_eq!(right, "本章还剩556页");
        let (left, right) = footer_zh(2, 2);
        assert_eq!(left, "2/2页");
        assert_eq!(right, "本章还剩0页");
    }

    #[test]
    fn frame_floor_is_one_refresh() {
        assert!(frames_within_floor(&[8.3, 16.7, 17.0, 21.0]));
        assert!(!frames_within_floor(&[33.4]));
        assert!(!frames_within_floor(&[]));
    }

    #[test]
    fn fallbacks_stay_under_the_book() {
        let en = fallback_css("en-GB");
        assert!(en.contains("@layer spike-fallback"));
        assert!(en.contains(":where("));
        assert!(!en.contains("!important"));
        assert!(!en.contains("text-wrap"));
        assert!(en.contains("Iowan Old Style"));
        assert!(en.contains("Songti SC"));
        assert!(en.contains("22px"));
        assert!(en.contains("1.48"));
        assert!(en.contains("text-indent: 2em"));
        assert!(en.contains("-webkit-hyphenate-limit-before: 2"));
        assert!(en.contains("-webkit-hyphenate-limit-after: 3"));
        assert!(en.contains("-webkit-hyphenate-limit-lines: 2"));
        assert!(en.contains("hanging-punctuation: first allow-end last"));
        assert!(!en.contains("text-autospace"));
        let zh = fallback_css("zh-CN");
        assert!(zh.contains("20px"));
        assert!(zh.contains("1.70"));
        assert!(zh.contains("text-autospace: normal"));
        assert!(zh.contains("line-break: strict"));
        let ja = fallback_css("ja");
        assert!(ja.contains("text-autospace: normal"));
    }

    #[test]
    fn page_sheet_does_not_restyle_type() {
        let css = page_css("en");
        assert!(css.contains("overflow: hidden"));
        assert!(!css.contains("column-width: 100vw"));
        assert!(css.contains(PAPER));
        assert!(css.contains("48ch"));
        assert!(!css.contains("font-size"));
        assert!(!css.contains("font-family"));
        assert!(!css.contains("text-indent"));
        assert!(!css.contains("text-align"));
        assert!(!css.contains("line-height"));
        assert!(!css.contains("hyphens"));
        let zh = page_css("zh");
        assert!(zh.contains("64ch"));
        assert!(zh.contains("11%"));
    }

    #[test]
    fn lang_is_stamped_and_the_fallback_is_first_in_the_head() {
        let html = prepare_html("<html><head><style>p{font-size:19px}</style></head><body></body></html>", "en-GB");
        assert!(html.contains("<html lang=\"en-GB\">"));
        let fallback = html.find("spike-fallback").unwrap();
        let book = html.find("font-size:19px").unwrap();
        assert!(fallback < book);
        let replaced = prepare_html("<html lang=\"en\"><head></head></html>", "zh-Hans");
        assert!(replaced.contains("lang=\"zh-Hans\""));
        assert!(!replaced.contains("lang=\"en\""));
    }

    #[test]
    fn trim_fit_keeps_the_ratio_and_declared_size_wins() {
        let kai = trim("32kai");
        assert_eq!(trim("nope").id, "paperback");
        assert_eq!(DEFAULT_TRIM, "paperback");
        let (x, y, w, h) = fit_page(1100.0, 1400.0, f64::from(kai.width_mm), f64::from(kai.height_mm));
        assert!(ratio_within(w, h, 130.0, 184.0));
        assert!(x >= 0.0 && y >= 0.0);
        assert!(w <= 1100.0 + 0.01 && h <= 1400.0 + 0.01);
        let (rw, rh, source) = page_ratio(Some((1000.0, 1000.0)), trim("paperback"));
        assert_eq!(source, "declared");
        assert!(ratio_within(rw, rh, 1.0, 1.0));
        let (_, _, source) = page_ratio(None, trim("trade"));
        assert_eq!(source, "trim");
        assert_eq!(trim("trade").width_mm, 140);
    }

    #[test]
    fn justify_source_parses_and_stays_out_of_the_type_sheet() {
        let on = justify_source(true);
        let off = justify_source(false);
        assert!(on.contains("window.spikeJustify"));
        assert!(on.contains("const JUSTIFY_ON = true"));
        assert!(off.contains("const JUSTIFY_ON = false"));
        assert!(on.contains("const OWN_PUBLISHER = false"));
        assert!(!on.contains("justifyContent"));
        assert!(!on.contains("</script>"));
        assert!(!on.to_ascii_lowercase().contains("text-wrap"));
        let path = std::env::temp_dir().join("superreader-justify-check.js");
        std::fs::write(&path, &on).unwrap();
        let status = std::process::Command::new("node")
            .arg("--check")
            .arg(&path)
            .status()
            .expect("node");
        let _ = std::fs::remove_file(&path);
        assert!(status.success());
    }

    #[test]
    fn script_is_one_slide_and_names_no_forbidden_mode() {
        let script = page_script("en-GB");
        assert!(script.contains("160"));
        assert!(script.contains("slide") || script.contains("translate3d"));
        assert!(!script.contains("__LANG__"));
        assert!(!script.contains("</script>"));
        assert!(script.contains("en-GB"));
        let lower = format!("{} {}", fallback_css("en"), script).to_ascii_lowercase();
        assert!(!lower.contains("night"));
        assert!(!lower.contains("theme"));
        assert!(!lower.contains("spread"));
        assert!(!lower.contains("text-wrap"));
    }

    #[test]
    fn title_fades_and_the_readme_lists_the_overrides() {
        assert_eq!(title_alpha(0), 1.0);
        assert_eq!(title_alpha(900), 0.0);
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/prototypes/reading-view/README.md");
        let readme = std::fs::read_to_string(&path).unwrap_or_default();
        for (_, reason) in PAGE_OVERRIDES {
            assert!(readme.contains(reason), "missing {reason}");
        }
        assert!(readme.contains("Iowan Old Style"));
        assert!(readme.contains("Songti SC"));
        assert!(readme.contains("#f3eee4"));
        assert!(readme.contains("32kai"));
        assert!(readme.contains("paperback is the default"));
        assert!(readme.contains("130"));
        assert!(readme.contains("hanging-punctuation: first allow-end last"));
        assert!(readme.contains("text-autospace: normal"));
        assert!(readme.contains("do not set text-wrap to pretty"));
        let lower = readme.to_ascii_lowercase();
        assert!(!lower.contains("night"));
        assert!(!lower.contains("theme"));
        assert!(!lower.contains("spread"));
    }
}
