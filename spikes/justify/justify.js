(function () {
  __TEX_LAYOUT__
  const tex = createTexLayout();
  const PATTERNS = {
    "en-us": __PATTERNS_US__,
    "en-gb": __PATTERNS_GB__
  };
  const JUSTIFY_ON = __JUSTIFY_ON__;
  const BREAK_ALGO = "__BREAK_ALGO__";
  // Publisher Literata missed the gap bar at paperback and US Trade, so those paragraphs stay on WebKit.
  const OWN_PUBLISHER = false;
  const BREAKS = {
    // word-spacing fills each owned line, so ratio 1 is a gap of twice a normal space.
    // A filled gap is 1 + ratio spaces. 0.75 stays under the 2.0 max.
    maxAdjustmentRatio: 0.75,
    initialMaxAdjustmentRatio: 0.5,
    doubleHyphenPenalty: 10000,
    adjacentLooseTightPenalty: 10000
  };
  const HYPHEN_COST = 200;
  const LEFT_MIN = 2;
  const RIGHT_MIN = 3;
  const ALLOW = {
    EM: 1, I: 1, STRONG: 1, B: 1, ABBR: 1, SPAN: 1, A: 1, SMALL: 1, CITE: 1,
    Q: 1, DFN: 1, CODE: 1, KBD: 1, SAMP: 1, SUB: 1, SUP: 1, BDO: 1, BDI: 1,
    U: 1, S: 1, MARK: 1
  };
  const FORBIDDEN = "，。、．；：？！…—·・」』】〉》〕〗〙〛）］｝､｡";
  const HANGING = ".,;:!?)]}'\"”’»‐-–—";

  let installed = false;
  let host = null;
  let latest = null;
  let generation = 0;
  let pageKey = "";
  let passToken = 0;
  let waiter = null;
  let canvas;
  let canvasCtx;
  const origins = new WeakMap();
  let originsReady = false;
  const dictCache = new Map();
  const pointCache = new Map();
  let tries = null;
  let exceptions = null;
  let hyphenTail = 0;
  let cacheHits = 0;
  let cacheLive = 0;
  let missLog = [];
  const storedPlans = new Map();
  if (window.__spikeBreaks && window.__spikeBreaks.records) {
    window.__spikeBreaks.records.forEach(function (row) {
      storedPlans.set(storedKey(row), row);
    });
  }
  window.spikeBreaksClear = function () {
    storedPlans.clear();
  };

  function install(next) {
    host = next;
    installed = true;
    ensureStyle();
  }

  function onPageBoxChanged() {
    if (!installed) throw new Error("install first");
    const token = ++passToken;
    if (!placed()) {
      publish(blank(false));
      return;
    }
    const key = window.innerWidth.toFixed(1);
    if (key === pageKey && latest && latest.ready && latest.tail === "done" && typeof window.spikeCover !== "number") {
      publish(latest);
      return;
    }
    const applyStarted = performance.now();
    const list = candidates();
    ensureOrigins(list);
    hyphenTail = 0;
    cacheHits = 0;
    cacheLive = 0;
    missLog = [];
    padCache.clear();
    const paint = JUSTIFY_ON && window.spikeClearPaint ? window.spikeClearPaint() : null;
    let wrote = false;
    const held = typeof window.spikeHeld === "number" ? window.spikeHeld : 0;
    const cover = typeof window.spikeCover === "number" ? window.spikeCover : 0;
    const needRight = Math.max(cover, 0) + window.innerWidth * 2;
    const starts = startOffsets(list);
    let seenHeld = false;
    let stop = list.length;
    for (let i = 0; i < list.length; i++) {
      if (token !== passToken) return;
      const result = commitParagraph(list[i], i);
      if (result === "owned" || result === "native") wrote = true;
      const start = starts.get(list[i]);
      if (!seenHeld && (start === undefined || start >= held || containsOffset(list[i], held))) seenHeld = true;
      if (seenHeld && reaches(list[i], needRight)) {
        stop = i + 1;
        break;
      }
    }
    if (token !== passToken) return;
    if (wrote) generation += 1;
    pageKey = key;
    const counts = countMarks();
    publish(Object.assign({
      generation: generation,
      ready: true,
      break_source: breakSource(),
      live_calls: cacheLive,
      apply_ms: performance.now() - applyStarted,
      misses: missLog.slice(0, 8),
      settled_width: window.innerWidth,
      skipped: Math.max(0, list.length - counts.owned - counts.fallen_back),
      tail: "pending"
    }, counts, blankScore()));
    if (paint && window.spikePaint) window.spikePaint(paint.start, paint.end);
    scheduleTail(token, paint, stop, list);
  }

  function whenVisibleCommitted() {
    if (!installed) return Promise.reject(new Error("install first"));
    if (matches(latest)) return Promise.resolve(latest);
    return new Promise(function (resolve) {
      waiter = { resolve: resolve, token: passToken };
      const mine = waiter;
      setTimeout(function () {
        if (waiter === mine) {
          waiter = null;
          resolve(latest || blank(false));
        }
      }, 1000);
    });
  }

  function matches(report) {
    return !!(report && report.ready && Math.abs(report.settled_width - window.innerWidth) < 1);
  }

  function publish(report) {
    latest = report;
    if (waiter && matches(report)) {
      const resolve = waiter.resolve;
      waiter = null;
      resolve(report);
    }
    if (host) host.onReport(report);
  }

  function blank(ready) {
    return {
      generation: generation,
      ready: ready,
      settled_width: window.innerWidth,
      owned: 0,
      skipped: 0,
      fallen_back: 0,
      gap_ratio: null,
      gap_p95: null,
      loose_lines: 0,
      justified_lines: 0,
      rivers: 0,
      hyphen_lines: 0,
      hyphen_run: 0,
      overflow_lines: 0,
      forbidden_starts: 0,
      flush_err: 0,
      tail: "pending"
    };
  }

  function blankScore() {
    return {
      gap_ratio: null,
      gap_p95: null,
      loose_lines: 0,
      justified_lines: 0,
      rivers: 0,
      hyphen_lines: 0,
      hyphen_run: 0,
      overflow_lines: 0,
      forbidden_starts: 0,
      flush_err: 0,
      box_err: 0
    };
  }

  function placed() {
    if (window.innerWidth < 8) return false;
    if (document.fonts && document.fonts.status !== "loaded") return false;
    if (typeof window.spikePageW !== "number") return false;
    if (Math.abs(window.innerWidth - window.spikePageW) >= 1.5) return false;
    return true;
  }

  function candidates() {
    return Array.prototype.slice.call(document.body.querySelectorAll("p, li, blockquote"));
  }

  function tenthPx(px) {
    return Math.round(px * 10);
  }

  function fingerprint(text) {
    let hash = 0x811c9dc5;
    for (let i = 0; i < text.length; i++) {
      hash ^= text.charCodeAt(i);
      hash = Math.imul(hash, 0x01000193);
    }
    return (hash >>> 0).toString(16);
  }

  function storedKey(row) {
    return [row.i, row.w, row.first, row.font, row.indent, String(row.lang || "").toLowerCase(), row.in].join("\t");
  }

  function measureKey(index, measure, lang, incoming) {
    return storedKey({
      i: index,
      w: tenthPx(measure.width),
      first: tenthPx(measure.first),
      font: measure.font,
      indent: tenthPx(parseFloat(measure.indent) || 0),
      lang: lang,
      in: incoming
    });
  }

  function insertAt(p, offset, kind, hyphenWidth) {
    const walker = document.createTreeWalker(p, NodeFilter.SHOW_TEXT);
    let seen = 0;
    while (walker.nextNode()) {
      const node = walker.currentNode;
      const next = seen + node.textContent.length;
      if (offset <= next) {
        insertCut({ node: node, offset: offset - seen, kind: kind, width: hyphenWidth });
        return;
      }
      seen = next;
    }
    throw new Error("cut");
  }

  function liveFields(index, measure, incoming, before) {
    return {
      i: index,
      w: tenthPx(measure.width),
      first: tenthPx(measure.first),
      font: measure.font,
      indent: tenthPx(parseFloat(measure.indent) || 0),
      lang: (langOf(pOf(index)) || "").toLowerCase(),
      in: incoming,
      text: fingerprint(before),
      raw_w: measure.width,
      inner: window.innerWidth,
      dpr: window.devicePixelRatio || 1
    };
  }

  function pOf(index) {
    return candidates()[index];
  }

  function noteMiss(reason, live, row) {
    if (missLog.length >= 12) return;
    const diff = [];
    if (row) {
      ["w", "first", "font", "indent", "lang", "in", "text"].forEach(function (field) {
        if (row[field] !== live[field] && String(row[field]) !== String(live[field])) diff.push(field);
      });
    }
    missLog.push({ reason: reason, diff: diff, live: live, stored: row || null });
  }

  function applyStored(p, index, incoming, measure, before) {
    const live = liveFields(index, measure, incoming, before);
    live.lang = (langOf(p) || "").toLowerCase();
    const row = storedPlans.get(measureKey(index, measure, langOf(p), incoming));
    if (!row) {
      let sameIndex = null;
      storedPlans.forEach(function (item) {
        if (sameIndex || item.i !== index) return;
        sameIndex = item;
      });
      noteMiss(sameIndex ? "key" : "absent", live, sameIndex);
      return null;
    }
    if (row.native) return "native";
    if (row.text !== fingerprint(before)) {
      noteMiss("text", live, row);
      return null;
    }
    try {
      const cuts = (row.cuts || []).slice().sort(function (a, b) { return b[0] - a[0]; });
      cuts.forEach(function (cut) {
        insertAt(p, cut[0], cut[1] === "h" ? "hyphen" : "glue", measure.hyphen);
      });
      if (p.querySelector("br[data-kp]")) wrapLines(p, 0);
      const spans = p.querySelectorAll("span[data-kp-line]");
      for (let s = 0; s < spans.length && row.gap && s < row.gap.length; s++) {
        if (lastLineSpan(spans[s])) continue;
        spans[s].style.wordSpacing = (row.gap[s] / 1000).toFixed(3) + "px";
      }
      if (!p.hasAttribute("data-kp-ws")) {
        p.setAttribute("data-kp-ws", p.style.whiteSpace || "");
        p.setAttribute("data-kp-hy", p.style.hyphens || "");
      }
      p.style.whiteSpace = "nowrap";
      p.style.hyphens = "manual";
      p.style.webkitHyphens = "manual";
      if (p.textContent !== before) throw new Error("offset space changed");
      const groups = lineGroups(p);
      if (hyphenRun(groups) > 2) throw new Error("hyphen run");
      const past = inkPast(p, groups);
      if (past) throw new Error(past);
      return row.out || 0;
    } catch (err) {
      noteMiss(String(err && err.message || err), live, row);
      restore(p);
      p.removeAttribute("data-kp");
      p.removeAttribute("data-kp-key");
      p.removeAttribute("data-kp-in");
      p.removeAttribute("data-kp-out");
      return null;
    }
  }

  function harvestCuts(p) {
    const cuts = [];
    let seen = 0;
    function walk(el) {
      for (let i = 0; i < el.childNodes.length; i++) {
        const node = el.childNodes[i];
        if (node.nodeType === 3) {
          seen += node.textContent.length;
        } else if (node.nodeType === 1 && node.tagName === "BR" && node.hasAttribute("data-kp")) {
          const prev = node.previousElementSibling;
          const hyphen = !!(prev && prev.hasAttribute && prev.hasAttribute("data-kp-hyphen"));
          cuts.push([seen, hyphen ? "h" : "g"]);
        } else if (node.nodeType === 1) {
          walk(node);
        }
      }
    }
    walk(p);
    return cuts;
  }

  let unstored = [];

  function readPlans(list) {
    const plans = [];
    unstored = [];
    let incoming = 0;
    for (let i = 0; i < list.length; i++) {
      const p = list[i];
      if (p.getAttribute("data-kp") === "native") {
        const measure = lineMeasure(p);
        plans.push({
          i: i,
          w: tenthPx(measure.width),
          first: tenthPx(measure.first),
          font: measure.font,
          indent: tenthPx(parseFloat(measure.indent) || 0),
          lang: (langOf(p) || "").toLowerCase(),
          in: incoming,
          text: fingerprint(p.textContent || ""),
          out: 0,
          native: true,
          cuts: [],
          gap: [0]
        });
        continue;
      }
      if (p.getAttribute("data-kp") !== "owned") continue;
      const recorded = p.getAttribute("data-kp-in");
      if (recorded !== null && recorded !== "") incoming = Number(recorded) || 0;
      const measure = lineMeasure(p);
      const cuts = harvestCuts(p);
      const spans = p.querySelectorAll("span[data-kp-line]");
      const gap = [];
      if (!spans.length) gap.push(0);
      for (let s = 0; s < spans.length; s++) {
        gap.push(Math.round((parseFloat(spans[s].style.wordSpacing) || 0) * 1000));
      }
      if (gap.length !== cuts.length + 1) {
        if (unstored.length < 8) unstored.push({ i: i, cuts: cuts.length, gaps: gap.length });
        continue;
      }
      const out = Math.min(2, Number(p.getAttribute("data-kp-out")) || trailingFrom(lineGroups(p)) || 0);
      plans.push({
        i: i,
        w: tenthPx(measure.width),
        first: tenthPx(measure.first),
        font: measure.font,
        indent: tenthPx(parseFloat(measure.indent) || 0),
        lang: (langOf(p) || "").toLowerCase(),
        in: incoming,
        text: fingerprint(p.textContent || ""),
        out: out,
        cuts: cuts,
        gap: gap
      });
      incoming = out;
    }
    return plans;
  }

  function postPlans(list) {
    if (!window.webkit || !window.webkit.messageHandlers || !window.webkit.messageHandlers.spike) return;
    window.webkit.messageHandlers.spike.postMessage(JSON.stringify({
      kind: "breaks",
      algo: BREAK_ALGO,
      stamp: window.__spikeStamp || "",
      plans: readPlans(list)
    }));
  }

  function breakSource() {
    if (!cacheHits && !cacheLive) return null;
    if (!cacheLive) return "cached";
    if (!cacheHits) return "live";
    return "mixed";
  }

  function commitParagraph(p, index) {
    if (!JUSTIFY_ON) return "skip";
    const measure = lineMeasure(p);
    const key = measure.width.toFixed(1) + "|" + measure.font + "|" + measure.indent;
    if (p.getAttribute("data-kp-key") === key && (p.getAttribute("data-kp") === "owned" || p.getAttribute("data-kp") === "native")) {
      if (p.getAttribute("data-kp") === "owned") {
        const out = p.getAttribute("data-kp-out");
        if (out !== null && out !== "") hyphenTail = Number(out) || 0;
      }
      return "skip";
    }
    restore(p);
    const reason = rejectionReason(p, measure);
    if (reason) {
      p.removeAttribute("data-kp");
      p.removeAttribute("data-kp-key");
      p.removeAttribute("data-kp-in");
      p.removeAttribute("data-kp-out");
      return "skip";
    }
    const before = p.textContent;
    const incoming = hyphenTail;
    const applied = applyStored(p, index, incoming, measure, before);
    if (applied === "native") {
      p.setAttribute("data-kp", "native");
      p.setAttribute("data-kp-key", key);
      cacheHits += 1;
      return "native";
    }
    if (typeof applied === "number") {
      p.setAttribute("data-kp", "owned");
      p.setAttribute("data-kp-key", key);
      p.setAttribute("data-kp-in", String(incoming));
      p.setAttribute("data-kp-out", String(applied));
      hyphenTail = applied;
      cacheHits += 1;
      return "owned";
    }
    const typeBefore = typeSnapshot(p);
    p.setAttribute("data-kp", "open");
    p.setAttribute("data-kp-key", key);
    try {
      let anchored = itemize(p, false);
      const lengths = lineLengths(measure, anchored.length);
      let breaks;
      try {
        breaks = tex.breakLines(anchored.map(function (entry) { return entry.item; }), lengths, BREAKS);
      } catch (err) {
        if (!(err instanceof tex.MaxAdjustmentExceededError)) throw err;
        anchored = itemize(p, true);
        if (hyphenTail >= 1) {
          for (let h = 0; h < anchored.length; h++) {
            const item = anchored[h].item;
            if (item && item.type === "penalty" && item.flagged) {
              anchored[h].item = Object.assign({}, item, { cost: tex.MAX_COST });
              break;
            }
          }
        }
        const again = lineLengths(measure, anchored.length);
        breaks = tex.breakLines(anchored.map(function (entry) { return entry.item; }), again, BREAKS);
        breaks = forbidThirdHyphen(anchored, again, breaks);
      }
      writePlan(p, anchored, breaks, measure);
      if (p.textContent !== before) throw new Error("offset space changed");
      const typeAfter = typeSnapshot(p);
      if (typeAfter.fontFamily !== typeBefore.fontFamily || typeAfter.fontSize !== typeBefore.fontSize || typeAfter.lineHeight !== typeBefore.lineHeight || typeAfter.textIndent !== typeBefore.textIndent || typeAfter.textAlign !== typeBefore.textAlign) {
        throw new Error("type changed");
      }
      const groups = lineGroups(p);
      if (hyphenRun(groups) > 2) throw new Error("hyphen run");
      const past = inkPast(p, groups);
      if (past) throw new Error(past);
      p.setAttribute("data-kp", "owned");
      p.setAttribute("data-kp-in", String(incoming));
      hyphenTail = trailingFrom(groups);
      p.setAttribute("data-kp-out", String(hyphenTail));
      cacheLive += 1;
      return "owned";
    } catch (err) {
      restore(p);
      p.setAttribute("data-kp", "native");
      p.setAttribute("data-kp-key", key);
      p.removeAttribute("data-kp-in");
      p.removeAttribute("data-kp-out");
      cacheLive += 1;
      return "native";
    }
  }

  function rejectionReason(p, measure) {
    if (!(measure.width >= 8)) return "unplaced";
    const tag = p.tagName;
    if (tag !== "P" && tag !== "LI" && tag !== "BLOCKQUOTE") return "not-paragraph";
    const cs = getComputedStyle(p);
    if (cs.textAlign !== "justify") return "not-justify";
    if (origins.get(p) === "publisher" && !OWN_PUBLISHER) return "publisher-align";
    if (isCjkParagraph(p)) return "cjk";
    if (cs.whiteSpace === "pre" || cs.whiteSpace === "pre-wrap" || cs.whiteSpace === "pre-line") return "preformatted";
    if (notSimple(p)) return "not-simple";
    if (!(measure.space > 0)) return "unplaced";
    return null;
  }

  function notSimple(p) {
    const letter = getComputedStyle(p, "::first-letter");
    const size = parseFloat(getComputedStyle(p).fontSize) || 0;
    const letterSize = parseFloat(letter.fontSize) || 0;
    if (letter.float !== "none" || (size && letterSize > size * 1.5)) return true;
    const nodes = p.querySelectorAll("*");
    for (let i = 0; i < nodes.length; i++) {
      const el = nodes[i];
      const tag = el.tagName;
      if (tag === "BR" || tag === "IMG" || tag === "SVG" || tag === "RUBY" || tag === "TABLE") return true;
      if (!ALLOW[tag]) return true;
      const cs = getComputedStyle(el);
      if (cs.display !== "inline" || cs.float !== "none" || cs.position !== "static") return true;
    }
    return false;
  }

  function isCjkParagraph(p) {
    const lang = langOf(p).toLowerCase();
    if (lang.indexOf("zh") === 0 || lang.indexOf("ja") === 0 || lang.indexOf("ko") === 0) return true;
    const text = p.textContent || "";
    for (let i = 0; i < text.length; i++) {
      const code = text.charCodeAt(i);
      if ((code >= 0x3400 && code <= 0x9FFF) || (code >= 0xF900 && code <= 0xFAFF) || (code >= 0x3040 && code <= 0x30FF) || (code >= 0x31F0 && code <= 0x31FF) || (code >= 0xFF66 && code <= 0xFF9D) || (code >= 0x1100 && code <= 0x11FF) || (code >= 0x3130 && code <= 0x318F) || (code >= 0xAC00 && code <= 0xD7AF) || (code >= 0x3000 && code <= 0x303F)) {
        return true;
      }
    }
    return false;
  }

  function langOf(p) {
    let el = p;
    while (el) {
      if (el.lang) return el.lang;
      el = el.parentElement;
    }
    return "";
  }

  function ensureOrigins(list) {
    if (originsReady) return;
    const sheets = ["spike-fallback", "spike-fallback-late"].map(function (id) {
      return document.getElementById(id);
    }).filter(Boolean);
    if (!sheets.length) {
      list.forEach(function (p) { origins.set(p, "publisher"); });
      originsReady = true;
      return;
    }
    const prev = sheets.map(function (sheet) { return sheet.disabled; });
    try {
      sheets.forEach(function (sheet) { sheet.disabled = true; });
      list.forEach(function (p) {
        let origin = "fallback";
        try {
          if (getComputedStyle(p).textAlign === "justify") origin = "publisher";
        } catch (err) {
          origin = "publisher";
        }
        origins.set(p, origin);
      });
    } catch (err) {
      list.forEach(function (p) { origins.set(p, "publisher"); });
    } finally {
      sheets.forEach(function (sheet, index) { sheet.disabled = prev[index]; });
    }
    originsReady = true;
  }

  function restore(p) {
    p.querySelectorAll("br[data-kp], span[data-kp-hyphen]").forEach(function (node) {
      node.remove();
    });
    const lines = Array.prototype.slice.call(p.querySelectorAll("span[data-kp-line]"));
    for (let i = lines.length - 1; i >= 0; i--) {
      const span = lines[i];
      const parent = span.parentNode;
      while (span.firstChild) parent.insertBefore(span.firstChild, span);
      parent.removeChild(span);
    }
    if (p.hasAttribute("data-kp-ws")) {
      const whiteSpace = p.getAttribute("data-kp-ws");
      if (whiteSpace) p.style.whiteSpace = whiteSpace;
      else p.style.removeProperty("white-space");
      p.removeAttribute("data-kp-ws");
    }
    if (p.hasAttribute("data-kp-hy")) {
      const hyphens = p.getAttribute("data-kp-hy");
      if (hyphens) p.style.hyphens = hyphens;
      else p.style.removeProperty("hyphens");
      p.style.removeProperty("-webkit-hyphens");
      p.removeAttribute("data-kp-hy");
    }
    p.removeAttribute("data-kp");
    p.removeAttribute("data-kp-key");
    p.normalize();
  }

  function specifiedPad() {
    const el = document.getElementById("spike-page");
    const text = el ? el.textContent : "";
    const match = text.match(/padding-left:\s*([^;!]+)/);
    return match ? match[1].trim() : "";
  }


  function lineMeasure(p) {
    const cs = getComputedStyle(p);
    let width = parseFloat(cs.width) || 0;
    const pad = (parseFloat(cs.paddingLeft) || 0) + (parseFloat(cs.paddingRight) || 0);
    const border = (parseFloat(cs.borderLeftWidth) || 0) + (parseFloat(cs.borderRightWidth) || 0);
    if (cs.boxSizing === "border-box") width -= pad + border;
    const indent = parseFloat(cs.textIndent) || 0;
    const font = fontOf(cs);
    const opener = ((p.textContent || "").match(/\S/) || [""])[0];
    const hangsFirst = (cs.hangingPunctuation || "").indexOf("first") !== -1 && "“\"'‘«".indexOf(opener) !== -1;
    const hang = hangsFirst ? measureText(font, opener) : 0;
    return {
      width: width,
      first: Math.max(1, width - Math.max(0, indent - hang)),
      space: measureText(fontOf(cs), " "),
      hyphen: Math.max(1, measureText(fontOf(cs), "-")),
      font: fontOf(cs),
      indent: cs.textIndent || "0px"
    };
  }

  function lineLengths(measure, slots) {
    const widths = [];
    const count = Math.max(slots, 1);
    for (let i = 0; i < count; i++) widths.push(i === 0 ? measure.first : measure.width);
    return widths;
  }

  function itemize(p, hyphenate) {
    const items = [];
    const lang = langOf(p);
    walk(p, hyphenate, lang, items);
    items.push({ item: { type: "glue", width: 0, stretch: 1000, shrink: 0 }, node: null, start: 0, end: 0 });
    items.push({ item: tex.forcedBreak(), node: null, start: 0, end: 0 });
    return items;
  }

  function walk(el, hyphenate, lang, items) {
    for (let i = 0; i < el.childNodes.length; i++) {
      const child = el.childNodes[i];
      if (child.nodeType === 3) {
        addText(child, hyphenate, lang, items);
      } else if (child.nodeType === 1) {
        const cs = getComputedStyle(child);
        if (child.tagName === "ABBR" || cs.whiteSpace === "nowrap" || cs.whiteSpace === "pre") addAtomic(child, items);
        else walk(child, hyphenate, lang, items);
      }
    }
  }

  function addAtomic(el, items) {
    let width = el.getBoundingClientRect().width;
    if (!(width > 0)) width = measureText(fontOf(getComputedStyle(el)), el.textContent || "");
    items.push({ item: { type: "box", width: width }, node: el, start: 0, end: el.childNodes.length });
  }

  function domScale(font) {
    const key = "domscale\n" + font;
    if (widthCache.has(key)) return widthCache.get(key);
    const sample = "Hmmmwii ";
    const probe = document.createElement("span");
    probe.textContent = sample;
    probe.style.position = "absolute";
    probe.style.left = "0";
    probe.style.top = "0";
    probe.style.whiteSpace = "pre";
    probe.style.font = font;
    document.documentElement.appendChild(probe);
    const dom = probe.getBoundingClientRect().width || 0;
    probe.remove();
    const canvas = measureText(font, sample) || 1;
    const scale = dom > 0 ? dom / canvas : 1;
    widthCache.set(key, scale);
    return scale;
  }

  function addText(node, hyphenate, lang, items) {
    const text = node.textContent || "";
    const font = fontOf(getComputedStyle(node.parentElement || node.parentNode));
    const scale = domScale(font);
    const space = measureText(font, " ") * scale;
    const shrink = Math.max(0, space - 3);
    const hyphen = Math.max(1, measureText(font, "-") * scale);
    let i = 0;
    while (i < text.length) {
      const ch = text.charAt(i);
      if (ch === "\u00A0") {
        let j = i + 1;
        while (j < text.length && text.charAt(j) === "\u00A0") j++;
        items.push({ item: { type: "box", width: measureText(font, text.slice(i, j)) * scale }, node: node, start: i, end: j });
        i = j;
        continue;
      }
      if (isSpace(ch)) {
        let j = i + 1;
        while (j < text.length && isSpace(text.charAt(j))) j++;
        items.push({ item: { type: "glue", width: space, stretch: space, shrink: shrink }, node: node, start: i, end: j });
        i = j;
        continue;
      }
      let j = i + 1;
      while (j < text.length && text.charAt(j) !== "\u00A0" && !isSpace(text.charAt(j))) j++;
      const word = text.slice(i, j);
      if (hyphenate) pushWord(node, i, word, font, hyphen, lang, items);
      else items.push({ item: { type: "box", width: measureText(font, word) * scale }, node: node, start: i, end: j });
      i = j;
    }
  }

  function pushWord(node, start, word, font, hyphen, lang, items) {
    const points = hyphenPoints(word, lang, font);
    const bounds = points.concat([word.length]);
    let prev = 0;
    for (let n = 0; n < bounds.length; n++) {
      const at = bounds[n];
      const piece = word.slice(prev, at);
      if (piece) {
        items.push({ item: { type: "box", width: measureText(font, piece) * domScale(font) }, node: node, start: start + prev, end: start + at });
      }
      if (at < word.length) {
        items.push({
          item: { type: "penalty", width: hyphen, cost: HYPHEN_COST, flagged: true },
          node: node,
          start: start + at,
          end: start + at
        });
      }
      prev = at;
    }
  }

  function spaceNode(node) {
    if (node.nodeType !== 3 || !node.textContent) return false;
    for (let i = 0; i < node.textContent.length; i++) {
      if (!isSpace(node.textContent.charAt(i))) return false;
    }
    return true;
  }

  function isSpace(ch) {
    return ch === " " || ch === "\n" || ch === "\t" || ch === "\r" || ch === "\f";
  }

  function hyphenPoints(word, lang, font) {
    const match = /[A-Za-z]{5,}/.exec(word);
    if (!match) return [];
    const raw = pointsFor(match[0], lang, font);
    const base = match.index;
    const out = [];
    for (let i = 0; i < raw.length; i++) {
      const at = base + raw[i];
      if (at > 0 && at < word.length) out.push(at);
    }
    return out;
  }

  function pointsFor(word, lang, font) {
    const key = (lang || "") + "\n" + font + "\n" + word;
    if (pointCache.has(key)) return pointCache.get(key);
    let points = liangPoints(word, lang);
    if (!points.length && dictionary(lang, font)) points = webkitPoints(word, lang, font);
    pointCache.set(key, points);
    return points;
  }

  function dictionary(lang, font) {
    const key = (lang || "") + "\n" + font;
    if (dictCache.has(key)) return dictCache.get(key);
    const points = webkitPoints("hyphenation", lang, font);
    const ok = points.length > 0;
    dictCache.set(key, ok);
    if (ok) pointCache.set(key + "\nhyphenation", points);
    return ok;
  }

  function webkitPoints(word, lang, font) {
    const host = document.createElement("div");
    host.setAttribute("aria-hidden", "true");
    host.lang = lang || "en";
    host.style.cssText = "position:absolute;left:-10000px;top:0;width:1px;hyphens:auto;-webkit-hyphens:auto;-webkit-hyphenate-limit-before:2;-webkit-hyphenate-limit-after:3;-webkit-hyphenate-limit-lines:2;white-space:normal;overflow-wrap:normal;word-break:normal;line-height:1.2;";
    host.style.font = font;
    host.textContent = word;
    document.documentElement.appendChild(host);
    try {
      const node = host.firstChild;
      if (!node) return [];
      const points = [];
      let lineTop = null;
      for (let i = 0; i < word.length; i++) {
        const range = document.createRange();
        range.setStart(node, i);
        range.setEnd(node, i + 1);
        const rect = range.getClientRects()[0];
        if (!rect || !rect.height) continue;
        if (lineTop === null) lineTop = rect.top;
        else if (rect.top > lineTop + 0.5) {
          if (i >= LEFT_MIN && i <= word.length - RIGHT_MIN) points.push(i);
          lineTop = rect.top;
        }
      }
      return points;
    } catch (err) {
      return [];
    } finally {
      host.remove();
    }
  }

  function liangPoints(word, lang) {
    const table = patternTable(lang);
    if (!table) return [];
    const lower = word.toLowerCase();
    const known = table.exceptions.get(lower);
    if (known) return known;
    const padded = "." + lower + ".";
    const scores = new Array(padded.length).fill(0);
    for (let i = 0; i < padded.length; i++) {
      let node = table.trie;
      for (let j = i; j < padded.length; j++) {
        node = node.kids[padded.charAt(j)];
        if (!node) break;
        if (node.points) {
          for (let k = 0; k < node.points.length; k++) {
            const at = i + k;
            if (at < scores.length && node.points[k] > scores[at]) scores[at] = node.points[k];
          }
        }
      }
    }
    const indexes = [];
    for (let i = 0; i < padded.length; i++) {
      if (i > LEFT_MIN && i < padded.length - RIGHT_MIN && scores[i] % 2 === 1) indexes.push(i - 1);
    }
    return indexes;
  }

  function patternTable(lang) {
    ensurePatterns();
    const key = patternKey(lang);
    if (!key || !tries[key]) return null;
    return { trie: tries[key], exceptions: exceptions[key] };
  }

  function patternKey(lang) {
    const value = (lang || "").toLowerCase();
    if (value.indexOf("en-gb") === 0) return "en-gb";
    if (value.indexOf("en") === 0) return "en-us";
    return "";
  }

  function ensurePatterns() {
    if (tries) return;
    tries = {};
    exceptions = {};
    Object.keys(PATTERNS).forEach(function (key) {
      const pack = PATTERNS[key];
      tries[key] = compileTrie(pack.patterns || []);
      const map = new Map();
      (pack.exceptions || []).forEach(function (definition) {
        map.set(definition.replace(/-/g, "").toLowerCase(), exceptionPoints(definition));
      });
      exceptions[key] = map;
    });
  }

  function compileTrie(patterns) {
    const root = { points: null, kids: Object.create(null) };
    for (let i = 0; i < patterns.length; i++) {
      const pattern = patterns[i];
      const chars = pattern.replace(/[0-9]/g, "");
      const points = pattern.split(/\D/).map(function (part) { return part === "" ? 0 : Number(part); });
      let node = root;
      for (let c = 0; c < chars.length; c++) {
        const ch = chars.charAt(c);
        node = node.kids[ch] || (node.kids[ch] = { points: null, kids: Object.create(null) });
      }
      node.points = points;
    }
    return root;
  }

  function exceptionPoints(definition) {
    const points = [];
    let bare = 0;
    for (let i = 0; i < definition.length; i++) {
      if (definition.charAt(i) === "-") points.push(bare);
      else bare += 1;
    }
    return points;
  }

  function forbidThirdHyphen(anchored, lengths, breaks) {
    for (let guard = 0; guard < 8; guard++) {
      let run = 0;
      let banned = false;
      for (let i = 1; i < breaks.length - 1; i++) {
        const entry = anchored[breaks[i]];
        const item = entry && entry.item;
        if (!(item && item.type === "penalty" && item.flagged)) {
          run = 0;
          continue;
        }
        run += 1;
        if (run < 3) continue;
        const extra = anchored[breaks[i]];
        if (extra && extra.item) extra.item = Object.assign({}, extra.item, { cost: tex.MAX_COST });
        banned = true;
        break;
      }
      if (!banned) return breaks;
      try {
        breaks = tex.breakLines(anchored.map(function (row) { return row.item; }), lengths, BREAKS);
      } catch (err) {
        return breaks;
      }
    }
    return breaks;
  }

  function hyphenRun(groups) {
    let run = 0;
    let max = 0;
    for (let i = 0; i < groups.length; i++) {
      if (groups[i].hyphen) {
        run += 1;
        if (run > max) max = run;
      } else run = 0;
    }
    return max;
  }

  function trailingFrom(groups) {
    let tail = 0;
    for (let i = groups.length - 1; i >= 0; i--) {
      if (!groups[i].hyphen) break;
      tail += 1;
    }
    return tail;
  }

  function writePlan(p, anchored, breakpoints, measure) {
    ensureStyle();
    const cuts = [];
    for (let b = 0; b < breakpoints.length - 2; b++) {
      const entry = anchored[breakpoints[b + 1]];
      if (!entry || !entry.node || entry.node.nodeType !== 3) continue;
      const kind = entry.item.type === "penalty" && entry.item.flagged ? "hyphen" : "glue";
      cuts.push({
        node: entry.node,
        offset: entry.start,
        kind: kind,
        width: measure.hyphen
      });
    }
    cuts.sort(function (a, b) {
      if (a.node === b.node) return b.offset - a.offset;
      const pos = a.node.compareDocumentPosition(b.node);
      if (pos & Node.DOCUMENT_POSITION_FOLLOWING) return 1;
      if (pos & Node.DOCUMENT_POSITION_PRECEDING) return -1;
      return 0;
    });
    cuts.forEach(function (cut) { insertCut(cut); });
    if (p.querySelector("br[data-kp]")) wrapLines(p, 0);
    if (!p.hasAttribute("data-kp-ws")) {
      p.setAttribute("data-kp-ws", p.style.whiteSpace || "");
      p.setAttribute("data-kp-hy", p.style.hyphens || "");
    }
    p.style.whiteSpace = "nowrap";
    p.style.hyphens = "manual";
    p.style.webkitHyphens = "manual";
    fitSpacing(p);
  }

  function fitSpacing(p) {
    const spans = p.querySelectorAll("span[data-kp-line]");
    for (let pass = 0; pass < 8; pass++) {
      const edges = contentEdges(p);
      let worst = 0;
      for (let s = 0; s < spans.length; s++) {
        const span = spans[s];
        if (lastLineSpan(span)) continue;
        const rect = span.getBoundingClientRect();
        if (!(rect.width > 0)) continue;
        const edge = edgeAt(edges, rect.top + rect.height / 2, rect.left + rect.width / 2);
        if (!edge) continue;
        const shortfall = edge.right - rect.right;
        if (Math.abs(shortfall) > worst) worst = Math.abs(shortfall);
        if (Math.abs(shortfall) < 0.25) continue;
        const gaps = (span.textContent.match(/ /g) || []).length || 1;
        const current = parseFloat(span.style.wordSpacing) || 0;
        span.style.wordSpacing = (current + shortfall / gaps).toFixed(3) + "px";
      }
      if (worst < 0.25) break;
    }
  }

  function lastLineSpan(span) {
    let node = span.nextSibling;
    while (node) {
      if (node.nodeType === 1 && node.tagName === "BR" && node.hasAttribute("data-kp")) return false;
      node = node.nextSibling;
    }
    return true;
  }

  function insertCut(cut) {
    const node = cut.node;
    const parent = node.parentNode;
    if (!parent) return;
    if (cut.kind === "hyphen") {
      const right = node.splitText(cut.offset);
      const hyphen = document.createElement("span");
      hyphen.setAttribute("data-kp-hyphen", "");
      hyphen.style.width = cut.width + "px";
      parent.insertBefore(hyphen, right);
      const br = document.createElement("br");
      br.setAttribute("data-kp", "");
      parent.insertBefore(br, right);
      return;
    }
    const after = cut.offset < node.length ? node.splitText(cut.offset) : node.nextSibling;
    const br = document.createElement("br");
    br.setAttribute("data-kp", "");
    parent.insertBefore(br, after);
  }

  function wrapLines(el, lineNo) {
    const kids = Array.prototype.slice.call(el.childNodes);
    let bucket = [];
    function flush(before) {
      let start = 0;
      let end = bucket.length;
      while (start < end && spaceNode(bucket[start])) start += 1;
      while (end > start && spaceNode(bucket[end - 1])) end -= 1;
      const kept = bucket.slice(start, end).filter(function (node) {
        return !(node.nodeType === 3 && node.textContent === "");
      });
      const anchor = start < bucket.length ? bucket[start] : before;
      bucket = [];
      if (!kept.length) return;
      const span = document.createElement("span");
      span.setAttribute("data-kp-line", String(lineNo));
      el.insertBefore(span, anchor || null);
      kept.forEach(function (node) { span.appendChild(node); });
    }
    for (let i = 0; i < kids.length; i++) {
      const node = kids[i];
      if (node.nodeType === 1 && node.tagName === "BR" && node.hasAttribute("data-kp")) {
        flush(node);
        lineNo += 1;
        continue;
      }
      if (node.nodeType === 1 && node.querySelector && node.querySelector("br[data-kp]")) {
        flush(node);
        lineNo = wrapLines(node, lineNo);
        continue;
      }
      bucket.push(node);
    }
    flush(null);
    return lineNo;
  }

  function inkPast(p, lines) {
    const edges = contentEdges(p);
    const limit = parseFloat(getComputedStyle(p).lineHeight);
    const spans = p.querySelectorAll("span[data-kp-line]");
    for (let i = 0; i < spans.length; i++) {
      const rects = spans[i].getClientRects();
      for (let r = 0; r < rects.length; r++) {
        if (isFinite(limit) && rects[r].height > limit + 1) return "tall";
      }
    }
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      const edge = edgeAt(edges, line.top + (line.height || 0) / 2, (line.minLeft + line.maxRight) / 2);
      if (!edge) continue;
      if (line.maxRight > edge.right + 0.75) return "right " + (line.maxRight - edge.right).toFixed(2);
      // The first glyph's side bearing sits about 0.9px left of the content edge
      // on the wider trims. That is the same ink native leaves; it is not a line
      // that escaped the column.
      const hang = line.first && "“\"'‘«".indexOf(line.first) !== -1 && line.minLeft >= edge.left - 12;
      if (line.minLeft < edge.left - 1.25 && !hang) return "left " + (edge.left - line.minLeft).toFixed(2);
    }
    return "";
  }

  function contentEdges(p) {
    const cs = getComputedStyle(p);
    const padL = parseFloat(cs.paddingLeft) || 0;
    const padR = parseFloat(cs.paddingRight) || 0;
    const borderL = parseFloat(cs.borderLeftWidth) || 0;
    const borderR = parseFloat(cs.borderRightWidth) || 0;
    return Array.prototype.map.call(p.getClientRects(), function (rect) {
      return {
        top: rect.top,
        bottom: rect.bottom,
        left: rect.left + padL + borderL,
        right: rect.right - padR - borderR
      };
    });
  }

  function edgeAt(edges, y, x) {
    let best = null;
    let bestDist = Infinity;
    for (let i = 0; i < edges.length; i++) {
      const edge = edges[i];
      if (y < edge.top - 2 || y > edge.bottom + 2) continue;
      const dx = x < edge.left ? edge.left - x : x > edge.right ? x - edge.right : 0;
      if (dx < bestDist) {
        best = edge;
        bestDist = dx;
      }
    }
    return best;
  }

  function typeSnapshot(p) {
    const cs = getComputedStyle(p);
    return {
      fontFamily: cs.fontFamily,
      fontSize: cs.fontSize,
      lineHeight: cs.lineHeight,
      textIndent: cs.textIndent,
      textAlign: cs.textAlign
    };
  }

  function naturalSpace(p) {
    const cs = getComputedStyle(p);
    const key = "natspace\n" + fontOf(cs) + "\n" + cs.letterSpacing + "\n" + cs.fontVariantNumeric + "\n" + cs.fontFeatureSettings;
    if (widthCache.has(key)) return widthCache.get(key);
    const probe = document.createElement("span");
    probe.textContent = "n n";
    probe.style.position = "absolute";
    probe.style.whiteSpace = "pre";
    probe.style.font = fontOf(cs);
    probe.style.letterSpacing = cs.letterSpacing;
    probe.style.fontVariantNumeric = cs.fontVariantNumeric;
    probe.style.fontFeatureSettings = cs.fontFeatureSettings;
    document.documentElement.appendChild(probe);
    let width = 0;
    if (probe.firstChild) {
      const range = document.createRange();
      range.setStart(probe.firstChild, 1);
      range.setEnd(probe.firstChild, 2);
      width = range.getBoundingClientRect().width;
    }
    probe.remove();
    if (!(width > 0)) width = measureText(fontOf(cs), " ");
    widthCache.set(key, width);
    return width;
  }

  function scoreChapter(doc) {
    const lines = [];
    let flush = 0;
    let boxErr = 0;
    const root = doc.body;
    const stride = window.innerWidth || 1;
    const paras = root.querySelectorAll("p, li, blockquote");
    for (let i = 0; i < paras.length; i++) {
        const p = paras[i];
        if (p.querySelector("p, li, blockquote")) continue;
        const cjk = isCjkParagraph(p);
        const align = getComputedStyle(p).textAlign;
        const owned = p.getAttribute("data-kp") === "owned";
        const space = naturalSpace(p) || 1;
        const indent = Math.max(0, parseFloat(getComputedStyle(p).textIndent) || 0);
        const formula = nativePad(p);
        const fragments = p.getClientRects();
        for (let f = 0; f < fragments.length; f++) {
          if (fragments[f].width > 0) boxErr = Math.max(boxErr, Math.abs(fragments[f].width - stride));
        }
        const cs = getComputedStyle(p);
        boxErr = Math.max(boxErr, Math.abs((parseFloat(cs.paddingLeft) || 0) - formula.padL), Math.abs((parseFloat(cs.paddingRight) || 0) - formula.padR));
        const edges = contentEdges(p);
        const groups = lineGroups(p);
        const lineSpans = p.querySelectorAll("span[data-kp-line]");
        for (let s = 0; s < lineSpans.length; s++) {
          const span = lineSpans[s];
          if (lastLineSpan(span)) continue;
          const rect = span.getBoundingClientRect();
          if (!(rect.width > 0) || !owned) continue;
          const origin = fragmentLeft(fragments, rect);
          if (origin == null) continue;
          const contentLeft = origin + formula.padL;
          const contentRight = origin + stride - formula.padR;
          const leftErr = Math.min(Math.abs(rect.left - contentLeft), Math.abs(rect.left - (contentLeft + indent)));
          flush = Math.max(flush, leftErr, Math.abs(rect.right - contentRight));
        }
        for (let g = 0; g < groups.length; g++) {
          const line = groups[g];
          if (!(line.maxRight > line.minLeft) || !(line.height > 0)) continue;
          const midX = (line.minLeft + line.maxRight) / 2;
          lines.push({
            cjk: cjk,
            justify: align === "justify",
            owned: owned,
            space: space,
            indent: g === 0 ? indent : 0,
            last: g === groups.length - 1,
            gaps: line.gaps,
            hyphen: line.hyphen,
            first: line.first,
            second: line.second || "",
            lastChar: line.lastChar || "",
            maxRight: line.maxRight,
            minLeft: line.minLeft,
            top: line.top,
            height: line.height,
            edges: edges,
            column: Math.max(0, Math.floor(midX / stride))
          });
        }
    }
    const ratios = [];
    let loose = 0;
    let justified = 0;
    let hyphenLines = 0;
    let hyphenRun = 0;
    let hyphenStreak = 0;
    let overflow = 0;
    let forbidden = 0;
    let column = null;
    const byColumn = {};
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (column !== line.column) {
        hyphenStreak = 0;
        column = line.column;
      }
      const pair = (line.first === "—" || line.first === "…") && line.first === line.second;
      if (!line.owned && line.first && FORBIDDEN.indexOf(line.first) !== -1 && !pair) forbidden += 1;
      if (line.cjk || !line.justify) {
        hyphenStreak = 0;
        continue;
      }
      if (!line.owned) {
        hyphenStreak = 0;
        continue;
      }
      const edge = edgeAt(line.edges, line.top + (line.height || 0) / 2, (line.minLeft + line.maxRight) / 2);
      if (edge) {
        const hangingOpen = line.first && "“\"'‘«".indexOf(line.first) !== -1 && line.minLeft >= edge.left - 12;
        if (line.maxRight > edge.right + 0.75 || (line.minLeft < edge.left - 1.25 && !hangingOpen)) overflow += 1;
      }
      if (line.hyphen) {
        hyphenStreak += 1;
        if (hyphenStreak > hyphenRun) hyphenRun = hyphenStreak;
      } else hyphenStreak = 0;
      if (line.last || line.gaps.length < 2) continue;
      justified += 1;
      let widest = 0;
      const wide = [];
      for (let g = 0; g < line.gaps.length; g++) {
        const ratio = line.gaps[g].gap / line.space;
        ratios.push(ratio);
        if (ratio > widest) widest = ratio;
        if (line.gaps[g].gap > line.space * 1.5) wide.push(line.gaps[g].center);
      }
      if (widest > 1.5) loose += 1;
      if (line.hyphen) hyphenLines += 1;
      const bucket = byColumn[line.column] || (byColumn[line.column] = []);
      bucket.push({ centers: wide, space: line.space });
    }
    let rivers = 0;
    Object.keys(byColumn).forEach(function (key) {
      const count = countRivers(byColumn[key]);
      if (count > rivers) rivers = count;
    });
    return {
      gap_ratio: ratios.length ? Math.max.apply(null, ratios) : null,
      gap_p95: percentile(ratios, 0.95),
      loose_lines: loose,
      justified_lines: justified,
      rivers: rivers,
      hyphen_lines: hyphenLines,
      hyphen_run: hyphenRun,
      overflow_lines: overflow,
      forbidden_starts: forbidden,
      flush_err: Math.round(flush * 100) / 100,
      box_err: Math.round(boxErr * 100) / 100
    };
  }

  function fragmentLeft(fragments, rect) {
    let best = null;
    let bestDist = Infinity;
    const midY = rect.top + rect.height / 2;
    const midX = rect.left + rect.width / 2;
    for (let i = 0; i < fragments.length; i++) {
      const frag = fragments[i];
      if (midY < frag.top - 2 || midY > frag.bottom + 2) continue;
      const dx = midX < frag.left ? frag.left - midX : midX > frag.right ? midX - frag.right : 0;
      if (dx < bestDist) {
        best = frag.left;
        bestDist = dx;
      }
    }
    return best;
  }

  const padCache = new Map();

  function nativePad(p) {
    const cs = getComputedStyle(p);
    const pageW = window.innerWidth || 1;
    const spec = specifiedPad();
    const key = pageW.toFixed(2) + "\n" + spec + "\n" + fontOf(cs) + "\n" + cs.fontVariantNumeric + "\n" + cs.fontFeatureSettings + "\n" + cs.letterSpacing;
    if (padCache.has(key)) return padCache.get(key);
    const probe = document.createElement("div");
    probe.style.position = "absolute";
    probe.style.visibility = "hidden";
    probe.style.boxSizing = "border-box";
    probe.style.width = pageW + "px";
    probe.style.font = fontOf(cs);
    probe.style.fontVariantNumeric = cs.fontVariantNumeric;
    probe.style.fontFeatureSettings = cs.fontFeatureSettings;
    probe.style.letterSpacing = cs.letterSpacing;
    if (spec) {
      probe.style.paddingLeft = spec;
      probe.style.paddingRight = spec;
    }
    document.documentElement.appendChild(probe);
    const pcs = getComputedStyle(probe);
    const value = {
      padL: parseFloat(pcs.paddingLeft) || 0,
      padR: parseFloat(pcs.paddingRight) || 0,
      pageW: pageW
    };
    probe.remove();
    padCache.set(key, value);
    return value;
  }

  function lineGroups(p) {
    const lines = [];
    let line = null;
    function open(rect) {
      line = {
        top: rect.top,
        height: rect.height,
        minLeft: rect.left,
        maxRight: rect.right,
        gaps: [],
        pending: false,
        prevRight: null,
        first: null,
        lastChar: null,
        hyphen: false
      };
      lines.push(line);
    }
    const walker = document.createTreeWalker(p, NodeFilter.SHOW_TEXT);
    while (walker.nextNode()) {
      const node = walker.currentNode;
      const text = node.textContent || "";
      for (let i = 0; i < text.length; i++) {
        const ch = text.charAt(i);
        if (isSpace(ch)) {
          if (line) line.pending = true;
          continue;
        }
        const range = document.createRange();
        range.setStart(node, i);
        range.setEnd(node, i + 1);
        const rect = range.getClientRects()[0];
        if (!rect || !rect.width) continue;
        const jump = line && line.prevRight != null && (rect.left > line.prevRight + 48 || rect.left + 1 < line.minLeft);
        if (!line || jump || Math.abs(rect.top - line.top) > Math.max(3, (line.height || 0) * 0.6)) open(rect);
        if (!line.first && ch !== "\u00A0") line.first = ch;
        else if (line.first && !line.second && ch !== "\u00A0") line.second = ch;
        if (line.pending && line.prevRight != null && ch !== "\u00A0") {
          const gap = rect.left - line.prevRight;
          if (gap > 0.1) line.gaps.push({ gap: gap, center: (line.prevRight + rect.left) / 2 });
        }
        line.pending = false;
        line.prevRight = rect.right;
        line.maxRight = Math.max(line.maxRight, rect.right);
        line.minLeft = Math.min(line.minLeft, rect.left);
        line.height = Math.max(line.height, rect.height);
        if (ch !== "\u00A0") line.lastChar = ch;
      }
    }
    const marks = p.querySelectorAll("span[data-kp-hyphen]");
    for (let i = 0; i < marks.length; i++) {
      const rects = marks[i].getClientRects();
      for (let r = 0; r < rects.length; r++) {
        const rect = rects[r];
        if (!(rect.width > 0) || !(rect.height > 0)) continue;
        const mid = rect.top + rect.height / 2;
        let best = null;
        let bestDist = Infinity;
        for (let n = 0; n < lines.length; n++) {
          const line = lines[n];
          if (rect.left > line.maxRight + rect.width + 4) continue;
          if (rect.right < line.minLeft - 2) continue;
          const center = line.top + (line.height || 0) / 2;
          const dist = Math.abs(mid - center);
          if (dist < bestDist) {
            best = line;
            bestDist = dist;
          }
        }
        if (best && bestDist <= Math.max(4, (best.height || 0) * 0.8)) {
          best.hyphen = true;
          best.maxRight = Math.max(best.maxRight, rect.right);
        }
      }
    }
    return lines;
  }

  function countRivers(lines) {
    let rivers = 0;
    let run = 0;
    let prev = null;
    for (let i = 0; i < lines.length; i++) {
      const centers = lines[i].centers;
      if (!prev || !centers.length) {
        if (run >= 4) rivers += 1;
        run = centers.length ? 1 : 0;
        prev = centers.length ? centers : null;
        continue;
      }
      const space = lines[i].space || 1;
      let hit = false;
      for (let a = 0; a < centers.length && !hit; a++) {
        for (let b = 0; b < prev.length; b++) {
          if (Math.abs(centers[a] - prev[b]) <= space) hit = true;
        }
      }
      if (hit) {
        run += 1;
        prev = centers;
      } else {
        if (run >= 4) rivers += 1;
        run = 1;
        prev = centers;
      }
    }
    if (run >= 4) rivers += 1;
    return rivers;
  }

  function percentile(values, p) {
    if (!values.length) return null;
    const sorted = values.slice().sort(function (a, b) { return a - b; });
    const index = Math.min(sorted.length - 1, Math.max(0, Math.ceil(p * sorted.length) - 1));
    return sorted[index];
  }

  function countMarks() {
    let owned = 0;
    let fallen = 0;
    const nodes = document.body.querySelectorAll("p[data-kp], li[data-kp], blockquote[data-kp]");
    for (let i = 0; i < nodes.length; i++) {
      const state = nodes[i].getAttribute("data-kp");
      if (state === "owned") owned += 1;
      else if (state === "native") fallen += 1;
    }
    return { owned: owned, fallen_back: fallen };
  }

  function scheduleTail(token, paint, start, list) {
    let index = start;
    function step() {
      if (token !== passToken) return;
      let n = 0;
      while (index < list.length && n < 4) {
        if (token !== passToken) return;
        commitParagraph(list[index], index);
        index += 1;
        n += 1;
      }
      if (index < list.length) {
        idle(step);
        return;
      }
      if (token !== passToken || !latest) return;
      if (window.spikeBusy) {
        idle(step);
        return;
      }
      const counts = countMarks();
      const score = scoreChapter(document);
      readPlans(list);
      publish(Object.assign({}, latest, counts, score, {
        skipped: Math.max(0, list.length - counts.owned - counts.fallen_back),
        tail: "done",
        break_source: breakSource(),
        live_calls: cacheLive,
        misses: missLog.slice(0, 8),
        unstored: unstored.slice(0, 8),
        generation: generation
      }));
      if (paint && window.spikePaint) window.spikePaint(paint.start, paint.end);
      postPlans(list);
    }
    idle(step);
  }

  function idle(fn) {
    if (window.requestIdleCallback) window.requestIdleCallback(function () { fn(); }, { timeout: 200 });
    else setTimeout(fn, 16);
  }

  function startOffsets(list) {
    const wanted = new Set(list);
    const map = new Map();
    const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    let seen = 0;
    while (walker.nextNode()) {
      const node = walker.currentNode;
      const parent = node.parentElement && node.parentElement.closest("p, li, blockquote");
      if (parent && wanted.has(parent) && !map.has(parent)) map.set(parent, seen);
      seen += node.textContent.length;
    }
    return map;
  }

  function containsOffset(p, offset) {
    const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
    let seen = 0;
    while (walker.nextNode()) {
      const node = walker.currentNode;
      const next = seen + node.textContent.length;
      if (next > offset) return p.contains(node);
      seen = next;
    }
    return false;
  }

  function reaches(p, right) {
    const rects = p.getClientRects();
    for (let i = 0; i < rects.length; i++) {
      if (rects[i].right >= right) return true;
    }
    return false;
  }

  function fontOf(cs) {
    const bits = [];
    if (cs.fontStyle && cs.fontStyle !== "normal") bits.push(cs.fontStyle);
    if (cs.fontWeight && cs.fontWeight !== "400" && cs.fontWeight !== "normal") bits.push(cs.fontWeight);
    bits.push(cs.fontSize || "16px");
    bits.push(cs.fontFamily || "serif");
    return bits.join(" ");
  }

  const widthCache = new Map();

  function measureText(font, text) {
    const sample = text || " ";
    const key = font + "\n" + sample;
    if (widthCache.has(key)) return widthCache.get(key);
    if (!canvas) {
      canvas = document.createElement("canvas");
      canvasCtx = canvas.getContext("2d");
    }
    if (!canvasCtx) return sample.length * 8;
    canvasCtx.font = font || "16px serif";
    const width = canvasCtx.measureText(sample).width;
    widthCache.set(key, width);
    return width;
  }

  function ensureStyle() {
    if (document.getElementById("spike-kp")) return;
    const style = document.createElement("style");
    style.id = "spike-kp";
    style.textContent = "span[data-kp-hyphen]{display:inline-block;height:1px;background:currentColor;vertical-align:0.32em;-webkit-user-select:none;user-select:none;pointer-events:none;}span[data-kp-line]{display:inline;}";
    document.documentElement.appendChild(style);
  }

  window.spikeJustify = {
    install: install,
    onPageBoxChanged: onPageBoxChanged,
    whenVisibleCommitted: whenVisibleCommitted
  };
})();
