//! Phase 1 of the typesetter gate.
//!
//! One gap is the drawn inter-word space divided by the font's natural space.
//! The sample is every justified line. The paragraph's last line is out, and
//! so is a line with fewer than two spaces, a CJK line, and a line that is
//! not justified. Nearest-rank p95 matches the reader's `percentile`.
//!
//! The crate's all-lines p95 has to be no worse than the reader's all-lines
//! p95 from the same build, fonts, and width. The other conditions are zero
//! kinsoku violations, zero overflow, and at most 50 ms per chapter. Font
//! expansion and letter-spacing fail the gate. The recorded Pride numbers are
//! a baseline, not the bar. Tighten later.

pub const PHASE1_CHAPTER_MS: f64 = 50.0;

#[derive(Clone, Copy, Debug)]
pub struct Phase1Input {
    pub crate_p95: f64,
    pub reader_p95: Option<f64>,
    /// Spaces in the crate's sample. Zero means the chapter had nothing to score.
    pub crate_gaps: u32,
    pub kinsoku: u32,
    pub overflow: u32,
    pub chapter_ms: f64,
    pub font_expand: f32,
    pub letter_space: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase1Failure {
    FontExpand,
    LetterSpace,
    WorseThanReader,
    Kinsoku,
    Overflow,
    ChapterMs,
}

/// Nearest-rank 95th percentile. Same index as the reader's
/// `Math.ceil(0.95 * n) - 1`.
pub fn nearest_rank_p95(values: &mut [f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let index = ((values.len() as f64) * 0.95).ceil() as usize - 1;
    values[index.min(values.len() - 1)]
}

pub fn phase1(input: Phase1Input) -> Result<(), Vec<Phase1Failure>> {
    let mut failures = Vec::new();
    if input.font_expand != 0.0 {
        failures.push(Phase1Failure::FontExpand);
    }
    if input.letter_space != 0.0 {
        failures.push(Phase1Failure::LetterSpace);
    }
    match input.reader_p95 {
        Some(reader) if input.crate_p95 > reader => failures.push(Phase1Failure::WorseThanReader),
        None if input.crate_gaps > 0 => failures.push(Phase1Failure::WorseThanReader),
        _ => {}
    }
    if input.kinsoku > 0 {
        failures.push(Phase1Failure::Kinsoku);
    }
    if input.overflow > 0 {
        failures.push(Phase1Failure::Overflow);
    }
    if input.chapter_ms > PHASE1_CHAPTER_MS {
        failures.push(Phase1Failure::ChapterMs);
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn passing() -> Phase1Input {
        Phase1Input {
            crate_p95: 2.0,
            reader_p95: Some(3.4286),
            crate_gaps: 40,
            kinsoku: 0,
            overflow: 0,
            chapter_ms: 16.0,
            font_expand: 0.0,
            letter_space: 0.0,
        }
    }

    #[test]
    fn nearest_rank_matches_the_reader_index() {
        let mut four = vec![1.0, 2.0, 3.0, 4.0];
        assert_eq!(nearest_rank_p95(&mut four), 4.0);
        let mut twenty: Vec<f64> = (1..=20).map(|n| n as f64).collect();
        assert_eq!(nearest_rank_p95(&mut twenty), 19.0);
        assert_eq!(nearest_rank_p95(&mut []), 0.0);
        assert_eq!(nearest_rank_p95(&mut [1.5]), 1.5);
    }

    #[test]
    fn phase1_passes_when_the_crate_is_no_worse() {
        assert!(phase1(passing()).is_ok());
        let mut equal = passing();
        equal.crate_p95 = 3.4286;
        equal.reader_p95 = Some(3.4286);
        assert!(phase1(equal).is_ok());
        equal.chapter_ms = PHASE1_CHAPTER_MS;
        assert!(phase1(equal).is_ok());
    }

    #[test]
    fn phase1_fails_when_the_crate_is_looser_than_the_reader() {
        let mut input = passing();
        input.crate_p95 = 3.5;
        assert_eq!(
            phase1(input).unwrap_err(),
            vec![Phase1Failure::WorseThanReader]
        );
    }

    #[test]
    fn phase1_refuses_expansion_and_tracking() {
        let mut expand = passing();
        expand.font_expand = 0.02;
        expand.crate_p95 = 1.0;
        assert!(phase1(expand)
            .unwrap_err()
            .contains(&Phase1Failure::FontExpand));
        let mut track = passing();
        track.letter_space = 0.01;
        assert!(phase1(track)
            .unwrap_err()
            .contains(&Phase1Failure::LetterSpace));
    }

    #[test]
    fn phase1_fails_kinsoku_overflow_and_a_slow_chapter() {
        let mut input = passing();
        input.kinsoku = 1;
        input.overflow = 2;
        input.chapter_ms = 50.1;
        let failures = phase1(input).unwrap_err();
        assert!(failures.contains(&Phase1Failure::Kinsoku));
        assert!(failures.contains(&Phase1Failure::Overflow));
        assert!(failures.contains(&Phase1Failure::ChapterMs));
    }

    #[test]
    fn a_chapter_with_no_gaps_on_either_side_skips_the_p95_compare() {
        let mut input = passing();
        input.reader_p95 = None;
        input.crate_gaps = 0;
        input.crate_p95 = 0.0;
        assert!(phase1(input).is_ok());
        input.crate_gaps = 4;
        assert_eq!(
            phase1(input).unwrap_err(),
            vec![Phase1Failure::WorseThanReader]
        );
    }
}
