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
