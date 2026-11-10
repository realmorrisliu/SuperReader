//! Port of `spikes/justify/vendor/tex-linebreak/layout.js` `breakLines`.
//!
//! The adjustment cap is a hard maximum. When every active line is looser
//! than that cap and a finite ratio still exists, this returns
//! [`MaxAdjustmentExceeded`] instead of raising the cap. The caller runs
//! TeX's three passes and does not multiply the limit.

const MIN_COST: f64 = -1000.0;
const MAX_COST: f64 = 1000.0;
const MIN_ADJUSTMENT_RATIO: f64 = -1.0;
const PARAGRAPH_START: isize = -1;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Box,
    Glue,
    Penalty,
}

#[derive(Clone, Copy, Debug)]
pub struct Item {
    pub kind: Kind,
    pub width: f64,
    pub stretch: f64,
    pub shrink: f64,
    pub cost: f64,
    pub flagged: bool,
}

impl Item {
    pub fn box_(width: f64) -> Self {
        Self {
            kind: Kind::Box,
            width,
            stretch: 0.0,
            shrink: 0.0,
            cost: 0.0,
            flagged: false,
        }
    }

    pub fn glue(width: f64, stretch: f64, shrink: f64) -> Self {
        Self {
            kind: Kind::Glue,
            width,
            stretch,
            shrink,
            cost: 0.0,
            flagged: false,
        }
    }

    pub fn penalty(width: f64, cost: f64, flagged: bool) -> Self {
        Self {
            kind: Kind::Penalty,
            width,
            stretch: 0.0,
            shrink: 0.0,
            cost,
            flagged,
        }
    }

    pub fn forced() -> Self {
        Self::penalty(0.0, MIN_COST, false)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BreakOpts {
    pub max_adjustment_ratio: Option<f64>,
    pub initial_max_adjustment_ratio: f64,
    pub double_hyphen_penalty: f64,
    pub adjacent_loose_tight_penalty: f64,
    /// 0 keeps the lowest-demerit active node. Any other value asks for that
    /// many lines more, or fewer, than the lowest-demerit set, and falls back
    /// to it when no such node reached the end.
    pub looseness: i32,
}

impl Default for BreakOpts {
    fn default() -> Self {
        Self::product()
    }
}

impl BreakOpts {
    pub fn product() -> Self {
        Self {
            max_adjustment_ratio: Some(0.75),
            initial_max_adjustment_ratio: 0.5,
            double_hyphen_penalty: 10_000.0,
            adjacent_loose_tight_penalty: 10_000.0,
            looseness: 0,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct MaxAdjustmentExceeded;

struct Sums {
    box_width: Vec<f64>,
    glue_width: Vec<f64>,
    glue_stretch: Vec<f64>,
    glue_shrink: Vec<f64>,
}

struct Node {
    index: isize,
    line: usize,
    fitness: u8,
    fallback: bool,
    total: f64,
    prev: Option<usize>,
}

struct LineMetrics {
    actual: f64,
    stretch: f64,
    shrink: f64,
    start_offset: f64,
}

pub fn break_lines(
    items: &[Item],
    line_lengths: &[f64],
    opts: BreakOpts,
) -> Result<Vec<usize>, MaxAdjustmentExceeded> {
    break_lines_with(items, line_lengths, opts, opts.initial_max_adjustment_ratio)
}

fn break_lines_with(
    items: &[Item],
    line_lengths: &[f64],
    opts: BreakOpts,
    initial: f64,
) -> Result<Vec<usize>, MaxAdjustmentExceeded> {
    if items.is_empty() {
        return Ok(Vec::new());
    }
    let last = items.last().unwrap();
    if !(last.kind == Kind::Penalty && last.cost <= MIN_COST) {
        panic!("paragraph items must end with a forced break");
    }
    let negative_stretch = items.iter().any(|item| {
        item.kind == Kind::Glue && (item.stretch < 0.0 || item.shrink < 0.0)
    });
    let negative_widths = items.iter().any(|item| item.width < 0.0);
    let sums = prefix_sums(items);
    let suffix = suffix_floor(items, &sums, negative_stretch);
    let max_cap = opts.max_adjustment_ratio.unwrap_or(f64::INFINITY);
    let current_max = initial.min(max_cap);
    let next_content = next_content(items);

    let mut nodes = vec![Node {
        index: PARAGRAPH_START,
        line: 0,
        fitness: 0,
        fallback: false,
        total: 0.0,
        prev: None,
    }];
    let mut active = vec![0usize];
    let mut min_above = f64::INFINITY;

    for b in 0..items.len() {
        if !feasible(items, b) {
            continue;
        }
        let item = items[b];
        let mut last_active: Option<usize> = None;
        let mut best_keys: Vec<i64> = Vec::new();
        let mut best_nodes: Vec<usize> = Vec::new();
        let snapshot = active.clone();
        for a_id in snapshot {
            let a = &nodes[a_id];
            let ideal = line_len(line_lengths, a.line);
            let metrics = line_metrics(&sums, items, &next_content, a.index, b);
            let ratio = if metrics.actual < ideal {
                (ideal - metrics.actual) / metrics.stretch
            } else if metrics.actual > ideal {
                (ideal - metrics.actual) / metrics.shrink
            } else {
                0.0
            };
            if ratio > current_max {
                min_above = min_above.min(ratio);
            }
            let overfull = ideal + metrics.start_offset;
            let future = suffix[b];
            let can_prune = (!a.fallback || !negative_widths) && !negative_stretch;
            let forced = item.kind == Kind::Penalty && item.cost <= MIN_COST;
            if forced || (can_prune && ratio < MIN_ADJUSTMENT_RATIO && future > overfull) {
                active.retain(|&id| id != a_id);
                last_active = Some(a_id);
            }
            if ratio >= MIN_ADJUSTMENT_RATIO && ratio <= current_max {
                let badness = 100.0 * ratio.abs().powi(3);
                let penalty = if item.kind == Kind::Penalty { item.cost } else { 0.0 };
                let mut demerits = if penalty >= 0.0 {
                    (1.0 + badness + penalty).powi(2)
                } else if penalty > MIN_COST {
                    (1.0 + badness).powi(2) - penalty.powi(2)
                } else {
                    (1.0 + badness).powi(2)
                };
                let prev_item = if a.index == PARAGRAPH_START {
                    None
                } else {
                    Some(items[a.index as usize])
                };
                if item.kind == Kind::Penalty {
                    if let Some(prev) = prev_item {
                        if prev.kind == Kind::Penalty && item.flagged && prev.flagged {
                            demerits += opts.double_hyphen_penalty;
                        }
                    }
                }
                let fitness = if ratio < -0.5 {
                    0
                } else if ratio < 0.5 {
                    1
                } else if ratio < 1.0 {
                    2
                } else {
                    3
                };
                if a.index != PARAGRAPH_START && (fitness as i32 - a.fitness as i32).abs() > 1 {
                    demerits += opts.adjacent_loose_tight_penalty;
                }
                let node = Node {
                    index: b as isize,
                    line: a.line + 1,
                    fitness,
                    fallback: false,
                    total: a.total + demerits,
                    prev: Some(a_id),
                };
                let state_key = node.line as i64 * 4 + fitness as i64;
                let id = nodes.len();
                nodes.push(node);
                if let Some(pos) = best_keys.iter().position(|key| *key == state_key) {
                    let old = best_nodes[pos];
                    if nodes[id].total < nodes[old].total {
                        best_nodes[pos] = id;
                    }
                } else {
                    best_keys.push(state_key);
                    best_nodes.push(id);
                }
            }
        }
        for id in best_nodes {
            active.push(id);
        }
        if active.is_empty() {
            if min_above.is_finite() {
                if opts.max_adjustment_ratio == Some(current_max)
                    || (opts.max_adjustment_ratio.is_none() && current_max.is_infinite())
                {
                    return Err(MaxAdjustmentExceeded);
                }
                return break_lines_with(items, line_lengths, opts, min_above * 2.0);
            }
            let last = last_active.expect("fallback needs the last active node");
            let id = nodes.len();
            nodes.push(Node {
                index: b as isize,
                line: nodes[last].line + 1,
                fitness: 1,
                fallback: true,
                total: nodes[last].total + 1000.0,
                prev: Some(last),
            });
            active.push(id);
        }
    }

    let mut best: Option<usize> = None;
    for id in &active {
        if best.is_none() || nodes[*id].total < nodes[best.unwrap()].total {
            best = Some(*id);
        }
    }
    if opts.looseness != 0 {
        if let Some(id) = best {
            let want = nodes[id].line as i32 + opts.looseness;
            let mut alt: Option<usize> = None;
            for cand in &active {
                if nodes[*cand].fallback || nodes[*cand].line as i32 != want {
                    continue;
                }
                if alt.is_none() || nodes[*cand].total < nodes[alt.unwrap()].total {
                    alt = Some(*cand);
                }
            }
            if let Some(alt) = alt {
                best = Some(alt);
            }
        }
    }
    let mut output = Vec::new();
    let mut next = best;
    while let Some(id) = next {
        let index = nodes[id].index;
        output.push(if index == PARAGRAPH_START { 0 } else { index as usize });
        next = nodes[id].prev;
    }
    output.reverse();
    Ok(output)
}

pub(crate) fn adjustment_ratios(items: &[Item], line_lengths: &[f64], breakpoints: &[usize]) -> Vec<f64> {
    let mut ratios = Vec::new();
    for b in 0..breakpoints.len().saturating_sub(1) {
        let ideal = line_len(line_lengths, b);
        let start_break = if b == 0 { PARAGRAPH_START } else { breakpoints[b] as isize };
        let end = breakpoints[b + 1];
        let start = line_content_start(items, start_break, end);
        let mut actual = 0.0;
        let mut stretch = 0.0;
        let mut shrink = 0.0;
        if start <= end {
            for p in start..=end {
                let item = items[p];
                if item.kind == Kind::Box {
                    actual += item.width;
                } else if item.kind == Kind::Glue && p != start && p != end {
                    actual += item.width;
                    stretch += item.stretch;
                    shrink += item.shrink;
                } else if item.kind == Kind::Penalty && p == end {
                    actual += item.width;
                }
            }
        }
        let ratio = if actual < ideal {
            (ideal - actual) / stretch
        } else if actual > ideal {
            (ideal - actual) / shrink
        } else {
            0.0
        };
        ratios.push(ratio);
    }
    ratios
}

fn prefix_sums(items: &[Item]) -> Sums {
    let n = items.len();
    let mut sums = Sums {
        box_width: vec![0.0; n + 1],
        glue_width: vec![0.0; n + 1],
        glue_stretch: vec![0.0; n + 1],
        glue_shrink: vec![0.0; n + 1],
    };
    for (i, item) in items.iter().enumerate() {
        sums.box_width[i + 1] = sums.box_width[i] + if item.kind == Kind::Box { item.width } else { 0.0 };
        sums.glue_width[i + 1] = sums.glue_width[i] + if item.kind == Kind::Glue { item.width } else { 0.0 };
        sums.glue_stretch[i + 1] =
            sums.glue_stretch[i] + if item.kind == Kind::Glue { item.stretch } else { 0.0 };
        sums.glue_shrink[i + 1] =
            sums.glue_shrink[i] + if item.kind == Kind::Glue { item.shrink } else { 0.0 };
    }
    sums
}

fn suffix_floor(items: &[Item], sums: &Sums, negative_stretch: bool) -> Vec<f64> {
    let mut floor = vec![f64::INFINITY; items.len()];
    if negative_stretch {
        return floor;
    }
    for b in 0..items.len() {
        if feasible(items, b) {
            let penalty_width = if items[b].kind == Kind::Penalty { items[b].width } else { 0.0 };
            floor[b] = sums.box_width[b + 1] + sums.glue_width[b] - sums.glue_shrink[b] + penalty_width;
        }
    }
    let mut min_floor = f64::INFINITY;
    for i in (0..items.len()).rev() {
        let value = floor[i];
        if value.is_finite() {
            floor[i] = min_floor;
            min_floor = min_floor.min(value);
        }
    }
    floor
}

fn next_content(items: &[Item]) -> Vec<usize> {
    let mut next = vec![items.len(); items.len() + 1];
    for i in (0..items.len()).rev() {
        next[i] = if starts_with_content(&items[i]) { i } else { next[i + 1] };
    }
    next
}

fn starts_with_content(item: &Item) -> bool {
    item.kind == Kind::Box || (item.kind == Kind::Penalty && item.cost >= MAX_COST)
}

fn feasible(items: &[Item], b: usize) -> bool {
    let item = items[b];
    (item.kind == Kind::Glue && b > 0 && items[b - 1].kind == Kind::Box)
        || (item.kind == Kind::Penalty && item.cost < MAX_COST)
}

#[cfg(test)]
pub(crate) fn javascript_breaks(
    items: &[Item],
    lengths: &[f64],
    opts: BreakOpts,
) -> Result<Vec<usize>, MaxAdjustmentExceeded> {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let path = oracle_path();
    std::fs::write(&path, oracle_payload(items, lengths, opts)).expect("oracle input");
    let output = std::process::Command::new("node")
        .arg(format!("{dir}/tests/break_oracle.js"))
        .arg(&path)
        .output()
        .expect("node");
    if !output.status.success() {
        panic!("{}", String::from_utf8_lossy(&output.stderr));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    if text.contains("\"ok\":false") || text.contains("\"ok\": false") {
        Err(MaxAdjustmentExceeded)
    } else {
        let start = text.find('[').expect("breaks");
        let end = text.rfind(']').expect("breaks");
        Ok(text[start + 1..end]
            .split(',')
            .filter(|part| !part.trim().is_empty())
            .map(|part| part.trim().parse().unwrap())
            .collect())
    }
}

#[cfg(test)]
fn oracle_path() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let _ = std::fs::create_dir_all(format!("{dir}/target"));
    format!("{dir}/target/oracle-{id}-{}.json", std::process::id())
}

#[cfg(test)]
fn oracle_payload(items: &[Item], lengths: &[f64], opts: BreakOpts) -> String {
    let mut items_json = String::from("[");
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            items_json.push(',');
        }
        let kind = match item.kind {
            Kind::Box => "box",
            Kind::Glue => "glue",
            Kind::Penalty => "penalty",
        };
        items_json.push_str(&format!(
            "{{\"type\":\"{kind}\",\"width\":{},\"stretch\":{},\"shrink\":{},\"cost\":{},\"flagged\":{}}}",
            item.width, item.stretch, item.shrink, item.cost, item.flagged
        ));
    }
    items_json.push(']');
    let lengths: Vec<String> = lengths.iter().map(|n| n.to_string()).collect();
    let max = match opts.max_adjustment_ratio {
        Some(value) => value.to_string(),
        None => "null".into(),
    };
    format!(
        "{{\"items\":{items_json},\"lengths\":[{}],\"opts\":{{\"maxAdjustmentRatio\":{max},\"initialMaxAdjustmentRatio\":{},\"doubleHyphenPenalty\":{},\"adjacentLooseTightPenalty\":{}}}}}",
        lengths.join(","),
        opts.initial_max_adjustment_ratio,
        opts.double_hyphen_penalty,
        opts.adjacent_loose_tight_penalty
    )
}

fn line_len(lengths: &[f64], index: usize) -> f64 {
    if lengths.is_empty() {
        return 0.0;
    }
    if index < lengths.len() {
        lengths[index]
    } else {
        lengths[lengths.len() - 1]
    }
}

fn line_metrics(
    sums: &Sums,
    items: &[Item],
    next_content: &[usize],
    start_break: isize,
    end_break: usize,
) -> LineMetrics {
    let raw = if start_break == PARAGRAPH_START {
        next_content[0]
    } else {
        let start = start_break as usize;
        let skip = if items[start].kind == Kind::Penalty { 1 } else { 0 };
        next_content[start + skip]
    };
    let start = raw.min(end_break + 1);
    if start > end_break {
        return LineMetrics {
            actual: 0.0,
            stretch: 0.0,
            shrink: 0.0,
            start_offset: 0.0,
        };
    }
    let interior = (start + 1).min(end_break);
    let penalty_width = if items[end_break].kind == Kind::Penalty {
        items[end_break].width
    } else {
        0.0
    };
    LineMetrics {
        actual: sums.box_width[end_break + 1] - sums.box_width[start]
            + (sums.glue_width[end_break] - sums.glue_width[interior])
            + penalty_width,
        stretch: sums.glue_stretch[end_break] - sums.glue_stretch[interior],
        shrink: sums.glue_shrink[end_break] - sums.glue_shrink[interior],
        start_offset: sums.box_width[start] + sums.glue_width[start] - sums.glue_shrink[start],
    }
}

pub(crate) fn line_content_start(items: &[Item], start_break: isize, end_break: usize) -> usize {
    let mut start = if start_break == PARAGRAPH_START {
        0
    } else {
        let at = start_break as usize;
        at + if items[at].kind == Kind::Penalty { 1 } else { 0 }
    };
    while start < items.len() && !starts_with_content(&items[start]) {
        start += 1;
    }
    start.min(end_break + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn oracle(items: &[Item], lengths: &[f64], opts: BreakOpts) -> Oracle {
        let path = oracle_path();
        std::fs::write(&path, oracle_json(items, lengths, opts)).unwrap();
        let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let output = Command::new("node")
            .arg(format!("{dir}/tests/break_oracle.js"))
            .arg(&path)
            .output()
            .expect("node");
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        parse_oracle(&String::from_utf8_lossy(&output.stdout))
    }

    enum Oracle {
        Breaks(Vec<usize>),
        Exceeded,
    }

    fn parse_oracle(text: &str) -> Oracle {
        if text.contains("\"ok\":false") || text.contains("\"ok\": false") {
            return Oracle::Exceeded;
        }
        let start = text.find('[').unwrap();
        let end = text.find(']').unwrap();
        let nums = text[start + 1..end]
            .split(',')
            .filter(|part| !part.trim().is_empty())
            .map(|part| part.trim().parse().unwrap())
            .collect();
        Oracle::Breaks(nums)
    }

    fn oracle_json(items: &[Item], lengths: &[f64], opts: BreakOpts) -> String {
        let mut items_json = String::from("[");
        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                items_json.push(',');
            }
            let kind = match item.kind {
                Kind::Box => "box",
                Kind::Glue => "glue",
                Kind::Penalty => "penalty",
            };
            items_json.push_str(&format!(
                "{{\"type\":\"{kind}\",\"width\":{},\"stretch\":{},\"shrink\":{},\"cost\":{},\"flagged\":{}}}",
                item.width,
                item.stretch,
                item.shrink,
                item.cost,
                item.flagged
            ));
        }
        items_json.push(']');
        let lengths: Vec<String> = lengths.iter().map(|n| n.to_string()).collect();
        let max = match opts.max_adjustment_ratio {
            Some(value) => value.to_string(),
            None => "null".to_string(),
        };
        format!(
            "{{\"items\":{items_json},\"lengths\":[{}],\"opts\":{{\"maxAdjustmentRatio\":{max},\"initialMaxAdjustmentRatio\":{},\"doubleHyphenPenalty\":{},\"adjacentLooseTightPenalty\":{}}}}}",
            lengths.join(","),
            opts.initial_max_adjustment_ratio,
            opts.double_hyphen_penalty,
            opts.adjacent_loose_tight_penalty
        )
    }

    fn assert_same(items: &[Item], lengths: &[f64], opts: BreakOpts) {
        let rust = break_lines(items, lengths, opts);
        match oracle(items, lengths, opts) {
            Oracle::Breaks(js) => assert_eq!(rust.expect("rust breaks"), js),
            Oracle::Exceeded => assert!(rust.is_err(), "js threw and rust returned {rust:?}"),
        }
    }

    fn paragraph(words: &[f64], space: f64) -> Vec<Item> {
        let mut items = Vec::new();
        for (i, width) in words.iter().enumerate() {
            if i > 0 {
                items.push(Item::glue(space, space, (space - 3.0).max(0.0)));
            }
            items.push(Item::box_(*width));
        }
        items.push(Item::glue(0.0, 1000.0, 0.0));
        items.push(Item::forced());
        items
    }

    #[test]
    fn product_options_match_the_javascript_breaker() {
        let items = paragraph(&[40.0, 50.0, 30.0, 60.0, 45.0, 35.0, 55.0, 25.0], 10.0);
        assert_same(&items, &[120.0, 140.0, 140.0, 140.0, 140.0], BreakOpts::product());
    }

    #[test]
    fn a_cap_of_0_75_throws_when_the_only_legal_ratio_is_looser() {
        let items = paragraph(&[80.0, 80.0, 80.0], 10.0);
        assert_same(&items, &[100.0, 100.0, 100.0], BreakOpts::product());
    }

    #[test]
    fn hyphen_penalties_change_the_same_breakpoints() {
        let mut items = vec![
            Item::box_(70.0),
            Item::penalty(8.0, 50.0, true),
            Item::box_(40.0),
            Item::glue(10.0, 10.0, 7.0),
            Item::box_(70.0),
            Item::penalty(8.0, 50.0, true),
            Item::box_(40.0),
            Item::glue(0.0, 1000.0, 0.0),
            Item::forced(),
        ];
        assert_same(&items, &[100.0, 100.0, 100.0, 100.0], BreakOpts::product());
        items[1].cost = 1000.0;
        items[5].cost = 1000.0;
        assert_same(&items, &[100.0, 100.0, 100.0, 100.0], BreakOpts::product());
    }
}
