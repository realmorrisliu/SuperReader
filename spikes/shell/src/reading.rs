use std::fmt;

pub const FRAME_FLOOR_MS: f64 = 16.7;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Choice {
    Legacy,
    A,
    B,
    C,
}

#[derive(Debug)]
pub struct ChoiceError;

impl fmt::Display for ChoiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SPIKE_READING must be a, b, or c")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Paragraph {
    Indent { tenths: u8 },
    Space { thousandths: u16 },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Align {
    Left,
    Justify,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FooterChrome {
    Stay { thousandths: u16 },
    Hide { after_ms: u32, over_ms: u32 },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Turn {
    Cut,
    Slide { ms: u16 },
    Curl { ms: u16 },
}

#[derive(Clone, Copy)]
pub struct View {
    pub letter: char,
    pub opinion: &'static str,
    pub latin: &'static str,
    pub cjk: &'static str,
    pub size_px: u8,
    pub line_height: &'static str,
    pub measure_ch: u8,
    pub paragraph: Paragraph,
    pub align: Align,
    pub hyphens: bool,
    pub paper: &'static str,
    pub margin_top_px: u16,
    pub margin_side_vw: u8,
    pub margin_bottom_px: u16,
    pub title_delay_ms: u32,
    pub title_over_ms: u32,
    pub footer: FooterChrome,
    pub turn: Turn,
}

const VIEWS: [View; 3] = [
    View {
        letter: 'A',
        opinion: "A quiet justified page in Georgia and Songti, with a first-line indent and an instant cut.",
        latin: "Georgia",
        cjk: "Songti SC",
        size_px: 20,
        line_height: "1.70",
        measure_ch: 64,
        paragraph: Paragraph::Indent { tenths: 20 },
        align: Align::Justify,
        hyphens: true,
        paper: "#f3eee4",
        margin_top_px: 72,
        margin_side_vw: 11,
        margin_bottom_px: 64,
        title_delay_ms: 200,
        title_over_ms: 500,
        footer: FooterChrome::Stay { thousandths: 420 },
        turn: Turn::Cut,
    },
    View {
        letter: 'B',
        opinion: "An airy ragged page in New York and PingFang, with space between paragraphs and a short slide.",
        latin: "New York",
        cjk: "PingFang SC",
        size_px: 18,
        line_height: "1.85",
        measure_ch: 52,
        paragraph: Paragraph::Space { thousandths: 850 },
        align: Align::Left,
        hyphens: false,
        paper: "#f7f5ef",
        margin_top_px: 56,
        margin_side_vw: 8,
        margin_bottom_px: 48,
        title_delay_ms: 200,
        title_over_ms: 400,
        footer: FooterChrome::Hide {
            after_ms: 4000,
            over_ms: 400,
        },
        turn: Turn::Slide { ms: 160 },
    },
    View {
        letter: 'C',
        opinion: "A narrow page in Iowan Old Style and PingFang, with a paper curl.",
        latin: "Iowan Old Style",
        cjk: "PingFang SC",
        size_px: 22,
        line_height: "1.48",
        measure_ch: 48,
        paragraph: Paragraph::Indent { tenths: 20 },
        align: Align::Justify,
        hyphens: true,
        paper: "#fbf9f3",
        margin_top_px: 88,
        margin_side_vw: 14,
        margin_bottom_px: 72,
        title_delay_ms: 300,
        title_over_ms: 500,
        footer: FooterChrome::Stay { thousandths: 500 },
        turn: Turn::Curl { ms: 280 },
    },
];

pub fn choose(raw: Option<&str>) -> Result<Choice, ChoiceError> {
    match raw {
        None => Ok(Choice::Legacy),
        Some("a") => Ok(Choice::A),
        Some("b") => Ok(Choice::B),
        Some("c") => Ok(Choice::C),
        Some(_) => Err(ChoiceError),
    }
}

pub fn cycle(current: Choice) -> Choice {
    match current {
        Choice::Legacy | Choice::C => Choice::A,
        Choice::A => Choice::B,
        Choice::B => Choice::C,
    }
}

pub fn view(choice: Choice) -> Option<&'static View> {
    match choice {
        Choice::Legacy => None,
        Choice::A => Some(&VIEWS[0]),
        Choice::B => Some(&VIEWS[1]),
        Choice::C => Some(&VIEWS[2]),
    }
}

pub fn all() -> &'static [View] {
    &VIEWS
}

pub fn footer_zh(page: u32, pages: u32) -> (String, String) {
    let pages = pages.max(1);
    let page = page.clamp(1, pages);
    let left = pages.saturating_sub(page);
    (format!("{page}/{pages}页"), format!("本章还剩{left}页"))
}

pub fn alphas(view: &View, shown_ms: u32, footer_ms: u32) -> (f32, f32) {
    let title = fade_down(shown_ms, view.title_delay_ms, view.title_over_ms);
    let footer = match view.footer {
        FooterChrome::Stay { thousandths } => thousandths as f32 / 1000.0,
        FooterChrome::Hide { after_ms, over_ms } => {
            if footer_ms <= after_ms {
                1.0
            } else {
                fade_down(footer_ms - after_ms, 0, over_ms)
            }
        }
    };
    (title.clamp(0.0, 1.0), footer.clamp(0.0, 1.0))
}

fn fade_down(elapsed: u32, delay: u32, over: u32) -> f32 {
    if elapsed <= delay {
        1.0
    } else if over == 0 {
        0.0
    } else {
        1.0 - (elapsed - delay) as f32 / over as f32
    }
}

pub fn chrome_moving(view: &View, shown_ms: u32, footer_ms: u32) -> bool {
    let (title, footer) = alphas(view, shown_ms, footer_ms);
    if title > 0.02 {
        return true;
    }
    match view.footer {
        FooterChrome::Stay { .. } => false,
        FooterChrome::Hide { after_ms, over_ms } => footer_ms < after_ms + over_ms && footer > 0.02,
    }
}

pub fn frames_within_floor(frames: &[f64]) -> bool {
    let missed = FRAME_FLOOR_MS * 2.0;
    !frames.is_empty() && frames.iter().all(|ms| ms.is_finite() && *ms < missed)
}

pub fn stylesheet(view: &View) -> String {
    let align = match view.align {
        Align::Left => "left",
        Align::Justify => "justify",
    };
    let hyphens = if view.hyphens { "auto" } else { "manual" };
    let (indent, gap) = match view.paragraph {
        Paragraph::Indent { tenths } => (format!("{}em", tenths as f32 / 10.0), "0".to_string()),
        Paragraph::Space { thousandths } => ("0".to_string(), format!("{}em", thousandths as f32 / 1000.0)),
    };
    let side = view.margin_side_vw;
    format!(
        r#"
html, body {{
  background: {paper} !important;
  color: #1c1915 !important;
  font-family: "{latin}", "{cjk}", serif !important;
  font-size: {size}px !important;
  line-height: {leading} !important;
  margin: 0 !important;
  height: 100% !important;
}}
body {{
  box-sizing: border-box !important;
  width: 100vw !important;
  height: 100vh !important;
  margin: 0 !important;
  padding: {top}px 0 {bottom}px !important;
  column-width: 100vw !important;
  column-gap: 0 !important;
  column-fill: auto !important;
  overflow-x: auto !important;
  overflow-y: hidden !important;
  text-align: {align} !important;
  hyphens: {hyphens} !important;
  -webkit-hyphens: {hyphens} !important;
}}
p, li, blockquote, h1, h2, h3 {{
  padding-left: max({side}vw, calc((100vw - {measure}ch) / 2)) !important;
  padding-right: max({side}vw, calc((100vw - {measure}ch) / 2)) !important;
  box-sizing: border-box !important;
}}
p, li, blockquote {{
  font-family: inherit !important;
  font-size: inherit !important;
  line-height: inherit !important;
  text-align: inherit !important;
  text-indent: {indent} !important;
  margin: 0 0 {gap} 0 !important;
  hyphens: inherit !important;
  -webkit-hyphens: inherit !important;
}}
h1, h2, h3 {{
  font-family: inherit !important;
  text-align: center !important;
  text-indent: 0 !important;
  font-weight: 600 !important;
  margin: 0 0 0.8em 0 !important;
}}
body::-webkit-scrollbar {{ display: none; }}
"#,
        paper = view.paper,
        latin = view.latin,
        cjk = view.cjk,
        size = view.size_px,
        leading = view.line_height,
        measure = view.measure_ch,
        top = view.margin_top_px,
        side = side,
        bottom = view.margin_bottom_px,
        align = align,
        hyphens = hyphens,
        indent = indent,
        gap = gap,
    )
}

pub fn page_script(view: &View) -> String {
    let (motion, ms) = match view.turn {
        Turn::Cut => ("cut", 0),
        Turn::Slide { ms } => ("slide", ms),
        Turn::Curl { ms } => ("curl", ms),
    };
    let css = serde_json::to_string(&stylesheet(view)).unwrap_or_else(|_| "\"\"".to_string());
    SCRIPT
        .replace("__CSS__", &css)
        .replace("__MOTION__", motion)
        .replace("__MS__", &ms.to_string())
        .replace("__PAPER__", view.paper)
}

const SCRIPT: &str = r#"
(function () {
  const style = document.createElement('style');
  style.textContent = __CSS__;
  document.documentElement.appendChild(style);
  const MOTION = "__MOTION__";
  const DURATION = __MS__;
  const PAPER = "__PAPER__";
  const SWIPE = 48;
  const root = document.body;
  let busy = false;
  let queued = 0;
  let acc = 0;
  let armed = true;
  function column() {
    const w = Math.max(1, window.innerWidth);
    const pages = Math.max(1, Math.round(root.scrollWidth / w));
    const page = Math.min(pages, Math.round(root.scrollLeft / w) + 1);
    return { page: page, pages: pages, w: w };
  }
  function post(extra) {
    const box = column();
    const body = { page: box.page, pages: box.pages };
    if (extra) {
      Object.keys(extra).forEach(function (key) { body[key] = extra[key]; });
    }
    window.webkit.messageHandlers.spike.postMessage(JSON.stringify(body));
  }
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
      post(null);
      return;
    }
    busy = true;
    if (MOTION === "slide") slide(box, delta);
    else if (MOTION === "curl") curl(box, delta);
    else cut(box, delta);
  }
  function cut(box, delta) {
    const frames = [];
    let last = 0;
    requestAnimationFrame(function (t) {
      last = t;
      root.scrollLeft = box.w * (box.page + delta - 1);
      requestAnimationFrame(function (t2) {
        frames.push(t2 - last);
        settle(frames);
      });
    });
  }
  function slide(box, delta) {
    root.scrollLeft = box.w * (box.page + delta - 1);
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
  function curl(box, delta) {
    root.scrollLeft = box.w * (box.page + delta - 1);
    const leaf = document.createElement("div");
    leaf.id = "spike-leaf";
    leaf.style.cssText = "position:fixed;inset:0;z-index:5;pointer-events:none;background:" + PAPER + ";transform-origin:right center;will-change:transform;box-shadow:-24px 0 36px rgba(40,30,10,0.16)";
    document.documentElement.appendChild(leaf);
    const frames = [];
    let last = 0;
    const t0 = performance.now();
    const sign = delta > 0 ? 1 : -1;
    function frame(t) {
      if (last) frames.push(t - last);
      last = t;
      const u = Math.min(1, (t - t0) / DURATION);
      const eased = (1 - Math.cos(Math.PI * u)) / 2;
      leaf.style.transform = "perspective(1400px) rotateY(" + (sign * eased * 92) + "deg)";
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
})();
"#;

#[allow(dead_code)]
pub fn token_row(view: &View) -> String {
    let paragraph = match view.paragraph {
        Paragraph::Indent { tenths } => format!("indent {}em", tenths as f32 / 10.0),
        Paragraph::Space { thousandths } => format!("space {}em", thousandths as f32 / 1000.0),
    };
    let align = match view.align {
        Align::Left => "left",
        Align::Justify => "justify",
    };
    let hyphens = if view.hyphens { "auto" } else { "off" };
    let footer = match view.footer {
        FooterChrome::Stay { thousandths } => format!("stay {:.2}", thousandths as f32 / 1000.0),
        FooterChrome::Hide { after_ms, over_ms } => format!("hide after {after_ms}ms over {over_ms}ms"),
    };
    let turn = match view.turn {
        Turn::Cut => "cut".to_string(),
        Turn::Slide { ms } => format!("slide {ms}ms"),
        Turn::Curl { ms } => format!("curl {ms}ms"),
    };
    format!(
        "| {letter} | {latin} | {cjk} | {size}px | {leading} | {measure}ch | {paragraph} | {align} | {hyphens} | {paper} | {top}px {side}vw {bottom}px | fade {delay}ms {over}ms | {footer} | {turn} |",
        letter = view.letter,
        latin = view.latin,
        cjk = view.cjk,
        size = view.size_px,
        leading = view.line_height,
        measure = view.measure_ch,
        paragraph = paragraph,
        align = align,
        hyphens = hyphens,
        paper = view.paper,
        top = view.margin_top_px,
        side = view.margin_side_vw,
        bottom = view.margin_bottom_px,
        delay = view.title_delay_ms,
        over = view.title_over_ms,
        footer = footer,
        turn = turn,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chinese_footer_matches_the_books_line() {
        let (left, right) = footer_zh(590, 1146);
        assert_eq!(left, "590/1146页");
        assert_eq!(right, "本章还剩556页");
        let (_, right) = footer_zh(1112, 1146);
        assert_eq!(right, "本章还剩34页");
        let (left, right) = footer_zh(2, 2);
        assert_eq!(left, "2/2页");
        assert_eq!(right, "本章还剩0页");
    }

    #[test]
    fn frame_floor_is_one_refresh() {
        assert!(frames_within_floor(&[8.3, 16.7, 17.0, 21.0]));
        assert!(!frames_within_floor(&[33.4]));
        assert!(!frames_within_floor(&[]));
        assert!(!frames_within_floor(&[f64::NAN]));
    }

    #[test]
    fn choose_accepts_only_the_three_letters() {
        assert_eq!(choose(None).unwrap(), Choice::Legacy);
        assert_eq!(choose(Some("a")).unwrap(), Choice::A);
        assert_eq!(choose(Some("b")).unwrap(), Choice::B);
        assert_eq!(choose(Some("c")).unwrap(), Choice::C);
        assert!(choose(Some("A")).is_err());
        assert!(choose(Some("d")).is_err());
        assert_eq!(cycle(Choice::Legacy), Choice::A);
        assert_eq!(cycle(Choice::A), Choice::B);
        assert_eq!(cycle(Choice::B), Choice::C);
        assert_eq!(cycle(Choice::C), Choice::A);
    }

    #[test]
    fn each_view_is_one_column_on_its_own_paper() {
        for view in all() {
            let css = stylesheet(view);
            assert!(css.contains("column-width: 100vw"), "{}", view.letter);
            assert!(!css.contains("column-count"), "{}", view.letter);
            assert!(css.contains(view.paper), "{}", view.letter);
            assert!(css.contains(view.latin), "{}", view.letter);
            assert!(css.contains(view.cjk), "{}", view.letter);
            let script = page_script(view);
            assert!(!script.contains("__MOTION__"), "{}", view.letter);
            assert!(!script.contains("</script>"), "{}", view.letter);
            let lower = format!("{css} {script}").to_ascii_lowercase();
            assert!(!lower.contains("night"), "{}", view.letter);
            assert!(!lower.contains("theme"), "{}", view.letter);
            assert!(!lower.contains("spread"), "{}", view.letter);
        }
        assert!(page_script(&VIEWS[0]).contains("\"cut\""));
        assert!(page_script(&VIEWS[1]).contains("\"slide\""));
        assert!(page_script(&VIEWS[1]).contains("160"));
        assert!(page_script(&VIEWS[2]).contains("\"curl\""));
    }

    #[test]
    fn title_fades_and_the_hiding_footer_waits() {
        let a = &VIEWS[0];
        let (title, footer) = alphas(a, 0, 0);
        assert_eq!(title, 1.0);
        assert!((footer - 0.42).abs() < 0.001);
        let (title, _) = alphas(a, 700 + 900, 0);
        assert!(title <= 0.0);
        let b = &VIEWS[1];
        let (_, footer) = alphas(b, 0, 1000);
        assert_eq!(footer, 1.0);
        let (_, footer) = alphas(b, 0, 4000 + 400);
        assert!(footer <= 0.0);
    }

    #[test]
    fn readme_carries_each_opinion_and_token() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/prototypes/reading-view/README.md");
        let readme = std::fs::read_to_string(&path).unwrap_or_default();
        for view in all() {
            assert!(readme.contains(view.opinion), "{}", view.letter);
            assert!(readme.contains(&token_row(view)), "missing {}", token_row(view));
        }
        let lower = readme.to_ascii_lowercase();
        assert!(!lower.contains("night"));
        assert!(!lower.contains("theme"));
        assert!(!lower.contains("spread"));
    }
}
