pub const MIN_COST: f32 = -1000.0;
pub const MAX_COST: f32 = 1000.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Box,
    Glue,
    Penalty,
}

#[derive(Clone, Debug)]
pub struct Atom {
    pub kind: Kind,
    pub width: f32,
    pub stretch: f32,
    pub shrink: f32,
    pub penalty: f32,
    pub flagged: bool,
}

impl Atom {
    pub fn box_(width: f32) -> Self {
        Self {
            kind: Kind::Box,
            width,
            stretch: 0.0,
            shrink: 0.0,
            penalty: 0.0,
            flagged: false,
        }
    }

    pub fn glue(width: f32, stretch: f32, shrink: f32) -> Self {
        Self {
            kind: Kind::Glue,
            width,
            stretch,
            shrink,
            penalty: 0.0,
            flagged: false,
        }
    }

    pub fn penalty(cost: f32, width: f32, flagged: bool) -> Self {
        Self {
            kind: Kind::Penalty,
            width,
            stretch: 0.0,
            shrink: 0.0,
            penalty: cost,
            flagged,
        }
    }

    pub fn forced() -> Self {
        Self::penalty(MIN_COST, 0.0, false)
    }
}

#[derive(Clone, Debug)]
pub struct BrokenLine {
    pub start_item: usize,
    pub end_item: usize,
    pub ratio: f32,
    pub hyphen: bool,
    pub overflow: bool,
}

#[derive(Clone, Debug)]
struct Node {
    index: isize,
    line: usize,
    fitness: u8,
    total: f32,
    prev: Option<usize>,
}

pub fn break_paragraph(items: &[Atom], measure: impl Fn(usize) -> f32, max_ratio: f32) -> Vec<BrokenLine> {
    if items.is_empty() {
        return Vec::new();
    }
    let mut limit = max_ratio.max(0.5);
    for _ in 0..8 {
        if let Some(lines) = attempt(items, &measure, limit) {
            return lines;
        }
        limit *= 1.6;
    }
    attempt(items, &measure, 8.0).unwrap_or_else(|| force_anywhere(items, &measure))
}

fn attempt(items: &[Atom], measure: &impl Fn(usize) -> f32, max_ratio: f32) -> Option<Vec<BrokenLine>> {
    let mut nodes = vec![Node {
        index: -1,
        line: 0,
        fitness: 1,
        total: 0.0,
        prev: None,
    }];
    let mut active = vec![0usize];
    let sums = Sums::new(items);

    for b in 0..items.len() {
        if !feasible(items, b) {
            continue;
        }
        let mut best: [Option<usize>; 4] = [None; 4];
        let mut dead = Vec::new();
        for (slot, &ai) in active.iter().enumerate() {
            let node = &nodes[ai];
            let metrics = sums.metrics(items, node.index, b as isize);
            let ideal = measure(node.line);
            let ratio = adjustment(metrics.actual, ideal, metrics.stretch, metrics.shrink);
            let too_tight = ratio < -1.0;
            if items[b].kind == Kind::Penalty && items[b].penalty <= MIN_COST || too_tight {
                dead.push(slot);
            }
            if ratio < -1.0 || ratio > max_ratio {
                continue;
            }
            let mut demerits = demerits_of(ratio, items[b].kind, items[b].penalty);
            if items[b].flagged && node.index >= 0 && items[node.index as usize].flagged {
                demerits += 10_000.0;
            }
            let fitness = fitness_of(ratio);
            if node.index >= 0 && fitness.abs_diff(node.fitness) > 1 {
                demerits += 10_000.0;
            }
            let candidate = Node {
                index: b as isize,
                line: node.line + 1,
                fitness,
                total: node.total + demerits,
                prev: Some(ai),
            };
            let replace = match best[fitness as usize] {
                None => true,
                Some(id) => candidate.total < nodes[id].total,
            };
            if replace {
                best[fitness as usize] = Some(nodes.len());
                nodes.push(candidate);
            }
        }
        for slot in dead.into_iter().rev() {
            active.swap_remove(slot);
        }
        for id in best.into_iter().flatten() {
            active.push(id);
        }
        if active.is_empty() {
            return None;
        }
    }

    let mut best: Option<usize> = None;
    for &ai in &active {
        if nodes[ai].index == items.len() as isize - 1
            && best.is_none_or(|id| nodes[ai].total < nodes[id].total)
        {
            best = Some(ai);
        }
    }
    let best = best?;
    let mut chain = Vec::new();
    let mut cursor = Some(best);
    while let Some(id) = cursor {
        let node = &nodes[id];
        if node.index >= 0 {
            chain.push(node.index as usize);
        }
        cursor = node.prev;
    }
    chain.reverse();
    Some(lines_from(items, &chain, measure))
}

fn force_anywhere(items: &[Atom], measure: &impl Fn(usize) -> f32) -> Vec<BrokenLine> {
    let mut breaks = Vec::new();
    let mut width = 0.0;
    let mut line = 0;
    let mut start = 0usize;
    for (i, item) in items.iter().enumerate() {
        if item.kind == Kind::Box {
            width += item.width;
        }
        if width > measure(line) && i > start {
            breaks.push(i.saturating_sub(1));
            width = item.width;
            line += 1;
            start = i;
        }
    }
    breaks.push(items.len() - 1);
    lines_from(items, &breaks, measure)
}

fn lines_from(items: &[Atom], breaks: &[usize], measure: impl Fn(usize) -> f32) -> Vec<BrokenLine> {
    let mut lines = Vec::new();
    let mut prev: isize = -1;
    for (line_no, &end) in breaks.iter().enumerate() {
        let start = content_start(items, prev, end);
        let metrics = raw_metrics(items, start, end);
        let ideal = measure(line_no);
        let mut ratio = adjustment(metrics.0, ideal, metrics.2, metrics.1);
        let overflow = ratio < -1.0 || (metrics.2 <= 0.01 && metrics.0 > ideal + 0.5);
        if !ratio.is_finite() {
            ratio = if metrics.0 > ideal { -1.0 } else { 0.0 };
        }
        let hyphen = items[end].kind == Kind::Penalty && items[end].flagged;
        lines.push(BrokenLine {
            start_item: start,
            end_item: end,
            ratio,
            hyphen,
            overflow,
        });
        prev = end as isize;
    }
    lines
}

struct Metrics {
    actual: f32,
    stretch: f32,
    shrink: f32,
}

struct Sums {
    box_w: Vec<f32>,
    glue_w: Vec<f32>,
    glue_stretch: Vec<f32>,
    glue_shrink: Vec<f32>,
}

impl Sums {
    fn new(items: &[Atom]) -> Self {
        let n = items.len();
        let mut sums = Self {
            box_w: vec![0.0; n + 1],
            glue_w: vec![0.0; n + 1],
            glue_stretch: vec![0.0; n + 1],
            glue_shrink: vec![0.0; n + 1],
        };
        for (i, item) in items.iter().enumerate() {
            sums.box_w[i + 1] = sums.box_w[i] + if item.kind == Kind::Box { item.width } else { 0.0 };
            sums.glue_w[i + 1] = sums.glue_w[i] + if item.kind == Kind::Glue { item.width } else { 0.0 };
            sums.glue_stretch[i + 1] =
                sums.glue_stretch[i] + if item.kind == Kind::Glue { item.stretch } else { 0.0 };
            sums.glue_shrink[i + 1] =
                sums.glue_shrink[i] + if item.kind == Kind::Glue { item.shrink } else { 0.0 };
        }
        sums
    }

    fn metrics(&self, items: &[Atom], start_break: isize, end_break: isize) -> Metrics {
        let start = content_start(items, start_break, end_break as usize) as isize;
        if start > end_break {
            return Metrics { actual: 0.0, stretch: 0.0, shrink: 0.0 };
        }
        let end = end_break as usize;
        let start = start as usize;
        let interior = (start + 1).min(end);
        let penalty_w = if items[end].kind == Kind::Penalty { items[end].width } else { 0.0 };
        Metrics {
            actual: self.box_w[end + 1] - self.box_w[start]
                + (self.glue_w[end] - self.glue_w[interior])
                + penalty_w,
            stretch: self.glue_stretch[end] - self.glue_stretch[interior],
            shrink: self.glue_shrink[end] - self.glue_shrink[interior],
        }
    }
}

fn raw_metrics(items: &[Atom], start: usize, end: usize) -> (f32, f32, f32) {
    let mut actual = 0.0;
    let mut shrink = 0.0;
    let mut stretch = 0.0;
    for p in start..=end {
        let item = &items[p];
        match item.kind {
            Kind::Box => actual += item.width,
            Kind::Glue if p != start && p != end => {
                actual += item.width;
                shrink += item.shrink;
                stretch += item.stretch;
            }
            Kind::Penalty if p == end => actual += item.width,
            _ => {}
        }
    }
    (actual, shrink, stretch)
}

fn content_start(items: &[Atom], start_break: isize, end_break: usize) -> usize {
    let mut start = if start_break < 0 {
        0
    } else {
        let at = start_break as usize;
        at + if items[at].kind == Kind::Penalty { 1 } else { 0 }
    };
    while start < items.len() && start <= end_break && !starts_content(&items[start]) {
        start += 1;
    }
    start.min(end_break + 1)
}

fn starts_content(item: &Atom) -> bool {
    item.kind == Kind::Box || (item.kind == Kind::Penalty && item.penalty >= MAX_COST)
}

fn feasible(items: &[Atom], b: usize) -> bool {
    let item = &items[b];
    (item.kind == Kind::Glue && b > 0 && items[b - 1].kind == Kind::Box)
        || (item.kind == Kind::Penalty && item.penalty < MAX_COST)
}

fn adjustment(actual: f32, ideal: f32, stretch: f32, shrink: f32) -> f32 {
    if (actual - ideal).abs() < 0.05 {
        0.0
    } else if actual < ideal {
        if stretch <= 0.05 { 1.0e6 } else { (ideal - actual) / stretch }
    } else if shrink <= 0.05 {
        -1.0e6
    } else {
        (ideal - actual) / shrink
    }
}

fn demerits_of(ratio: f32, kind: Kind, penalty: f32) -> f32 {
    let badness = 100.0 * ratio.abs().powi(3);
    let penalty = if kind == Kind::Penalty { penalty } else { 0.0 };
    if penalty >= 0.0 {
        (1.0 + badness + penalty).powi(2)
    } else if penalty > MIN_COST {
        (1.0 + badness).powi(2) - penalty.powi(2)
    } else {
        (1.0 + badness).powi(2)
    }
}

fn fitness_of(ratio: f32) -> u8 {
    if ratio < -0.5 {
        0
    } else if ratio < 0.5 {
        1
    } else if ratio < 1.0 {
        2
    } else {
        3
    }
}

pub fn glue_width(item: &Atom, ratio: f32) -> f32 {
    let ratio = ratio.clamp(-1.0, 4.0);
    if ratio < 0.0 {
        item.width + ratio * item.shrink
    } else {
        item.width + ratio * item.stretch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_short_line_stays_one_line() {
        let mut items = Vec::new();
        for _ in 0..3 {
            items.push(Atom::box_(20.0));
            items.push(Atom::glue(10.0, 5.0, 3.0));
        }
        items.pop();
        items.push(Atom::forced());
        let lines = break_paragraph(&items, |_| 200.0, 0.75);
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(!lines[0].overflow);
    }

    #[test]
    fn a_long_paragraph_breaks_without_overflow() {
        let mut items = Vec::new();
        for _ in 0..12 {
            items.push(Atom::box_(30.0));
            items.push(Atom::glue(10.0, 5.0, 3.0));
        }
        items.pop();
        items.push(Atom::forced());
        let lines = break_paragraph(&items, |_| 100.0, 0.75);
        assert!(lines.len() >= 3, "{lines:?}");
        assert!(lines.iter().all(|line| !line.overflow), "{lines:?}");
    }

    #[test]
    fn a_short_following_word_is_not_left_for_the_next_line() {
        let words = [
            14.4, 12.5, 8.8, 44.1, 28.6, 25.0, 36.1, 17.7, 52.8, 48.5, 5.5, 33.9, 8.8,
        ];
        let mut items = Vec::new();
        for width in words {
            items.push(Atom::box_(width));
            items.push(Atom::glue(4.4, 2.2, 1.5));
        }
        items.pop();
        items.push(Atom::forced());
        let lines = break_paragraph(&items, |_| 384.5, 0.75);
        assert!(
            lines[0].end_item >= 24,
            "first line dropped a word that fits {lines:?}"
        );
    }
}
