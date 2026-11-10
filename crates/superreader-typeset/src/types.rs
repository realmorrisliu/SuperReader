use std::time::Duration;

pub const PAPERBACK_WIDTH: f32 = 534.0;
pub const PAPERBACK_HEIGHT: f32 = 819.0;

/// TeX hyphenation parameters used when the book leaves `hyphens` unset, or sets `auto`.
pub const TEX_HYPHEN_PENALTY: f64 = 50.0;
pub const TEX_HYPHEN_LEFT_MIN: usize = 2;
pub const TEX_HYPHEN_RIGHT_MIN: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FontId(pub u16);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Caret {
    pub node: u32,
    pub offset: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CaretRange {
    pub start: Caret,
    pub end: Caret,
}

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ContentBox {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Clone, Debug)]
pub struct PlacedGlyph {
    pub font: FontId,
    pub glyph_id: u32,
    pub x: f32,
    pub baseline_y: f32,
    pub advance: f32,
    pub node: u32,
    pub offset: u32,
    pub span: u32,
    pub ch: char,
    pub size: f32,
    pub line_end: bool,
    pub line_start: bool,
    pub paragraph_start: bool,
    pub hyphen: bool,
    /// Horizontal outline scale for this glyph. 1 is the font's own drawing.
    /// An experiment may set a line-level factor. Hit testing uses `advance`,
    /// which already includes the scale.
    pub x_scale: f32,
}

#[derive(Clone, Debug)]
pub struct GlyphRun {
    pub font: FontId,
    pub glyphs: Vec<PlacedGlyph>,
}

#[derive(Clone, Copy, Debug)]
pub struct SelectionRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug)]
pub struct LayoutReport {
    pub page_count: u32,
    /// Nearest-rank 95th percentile of drawn word space divided by the font's
    /// own natural space. The line set matches the product: justified lines
    /// only, excluding the last line of a paragraph, a line that ends on a
    /// forced break, and any line with fewer than two word spaces. A
    /// single-word line is in that last group. The gate uses this figure,
    /// not a value rescaled to another font's space.
    pub word_gap_p95: f64,
    pub word_gap_max: f64,
    pub hyphenated_lines: u32,
    pub line_count: u32,
    pub kinsoku_violations: u32,
    pub overflow_lines: u32,
    pub shape_ms: f64,
    pub break_ms: f64,
}

/// Counts from one layout. `lines_over_cap` is scored lines whose word gap
/// is above 1.75, the gap that a 0.75 adjustment ratio draws when stretch
/// equals the space. This does not change which lines the gate scores.
#[derive(Clone, Copy, Debug)]
pub struct LineStats {
    pub scored_lines: u32,
    pub lines_over_cap: u32,
    pub longest_hyphen_run: u32,
}

impl LayoutReport {
    pub fn warm_layout(&self) -> Duration {
        Duration::from_secs_f64((self.shape_ms + self.break_ms) / 1000.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ChapterInput<'a> {
    pub xhtml: &'a str,
    pub stylesheet: &'a str,
    pub base_href: &'a str,
    /// Content measure in px. `None` uses the paperback measure, 384.48 for
    /// Latin and 416.52 for CJK.
    pub measure: Option<f32>,
    /// Fallback font size in px. `None` uses 22 for Latin and 20 for CJK.
    /// A size the book sets in CSS still wins.
    pub font_size: Option<f32>,
    /// Experiment switches. `BreakTune::default` is the shipping breaker.
    pub tune: BreakTune,
}

/// Switches for a measurement run. Every default is the shipping breaker.
/// None of these fields moves the word-gap gate.
#[derive(Clone, Copy, Debug)]
pub struct BreakTune {
    /// Word-space shrink is one third of the space. Stretch stays the space.
    /// The default shrink is `max(0, space - 3)`.
    pub tex_shrink: bool,
    /// pdfTeX-style font expansion, as a fraction of each glyph's width.
    /// 0.02 is ±2 percent, one factor for the whole line. 0 is off.
    pub font_expand: f32,
    /// Extra advance per glyph, as a fraction of the em. 0.01 is ±1 percent.
    /// 0 is off.
    pub letter_space: f32,
    /// Try TeX pretolerance and tolerance, then a looseness of one line either
    /// way, and keep the set with the lowest finite peak. The hyphen penalty
    /// stays at the TeX default. A run of more than two hyphenated lines is
    /// rejected.
    pub tolerance: bool,
}

impl Default for BreakTune {
    fn default() -> Self {
        Self {
            tex_shrink: false,
            font_expand: 0.0,
            letter_space: 0.0,
            tolerance: false,
        }
    }
}

#[derive(Debug)]
pub enum TypesetError {
    Font(String),
    Epub(String),
    Empty,
}

impl std::fmt::Display for TypesetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Font(msg) | Self::Epub(msg) => f.write_str(msg),
            Self::Empty => f.write_str("chapter had no text"),
        }
    }
}

impl std::error::Error for TypesetError {}
