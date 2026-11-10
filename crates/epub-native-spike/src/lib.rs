mod breaker;
mod face;
mod hyphen;
mod layout;
mod parse;

pub use face::Faces;
pub use hyphen::Hyphenator;
pub use layout::{encode, side_by_side, Caret, Choices, Hyphenation, LaidOut, Punctuation, Stats, PAGE_H, PAGE_W};
pub use parse::{parse_chapter, parse_chapter_opts, Document};

pub struct Engine {
    faces: Faces,
    hyphen: Hyphenator,
}

impl Engine {
    pub fn open(latin: &str, italic: &str, cjk: &str) -> Result<Self, String> {
        Ok(Self {
            faces: Faces::load(latin, italic, cjk)?,
            hyphen: Hyphenator::load(),
        })
    }

    pub fn faces(&self) -> &Faces {
        &self.faces
    }

    pub fn layout(&self, xhtml: &str, css: &str, choices: Choices, spine_step: u32, cjk: bool) -> LaidOut {
        let doc = parse_chapter(xhtml, css, cjk);
        self.layout_doc(&doc, choices, spine_step, cjk)
    }

    pub fn layout_doc(&self, doc: &Document, choices: Choices, spine_step: u32, cjk: bool) -> LaidOut {
        LaidOut::lay(&self.faces, &self.hyphen, doc, choices, spine_step, cjk)
    }
}
