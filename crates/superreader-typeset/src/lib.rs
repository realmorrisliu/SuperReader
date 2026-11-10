//! Headless chapter typesetter for the owned EPUB engine.
//!
//! The Mac view consumes glyph runs, carets, hit tests, and selection rects.
//! WKWebView remains the reader until the paperback gates pass.
//! icu_segmenter is Unicode-3.0. rbook is Apache-2.0. harfrust is MIT.
//! skrifa is MIT OR Apache-2.0. tiny-skia is BSD-3-Clause.

mod book;
mod compose;
mod face;
mod gate;
mod hyphen;
mod kp;
mod parse;
mod types;

pub use book::{load_chapter, LoadedChapter};
pub use compose::{ChapterLayout, Typesetter};
pub use face::list_faces;
pub use gate::{nearest_rank_p95, phase1, Phase1Failure, Phase1Input};
pub use types::*;
