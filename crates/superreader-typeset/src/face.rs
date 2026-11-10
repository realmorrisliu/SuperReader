use harfrust::{
    shape, Buffer, Direction, Feature, Font, Language, Script, ShapeOptions, ShaperFont, Tag,
};
use skrifa::instance::{LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::{FontRef, GlyphId, MetadataProvider};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Transform};

pub struct Faces {
    latin: OwnedFace,
    italic: OwnedFace,
    cjk: OwnedFace,
}

struct OwnedFace {
    bytes: &'static [u8],
    font: Font,
}

#[derive(Clone, Debug)]
pub struct ShapedGlyph {
    pub gid: u32,
    pub advance: f32,
    pub x_offset: f32,
    pub char_index: usize,
}

pub struct ShapeOpts {
    pub italic: bool,
    pub cjk: bool,
    pub size: f32,
    pub weight: u16,
    pub small_caps: bool,
    pub oldstyle: bool,
}

impl Faces {
    pub fn load(latin: &str, italic: &str, cjk: &str) -> Result<Self, String> {
        Ok(Self {
            latin: load_face(latin)?,
            italic: load_face(italic)?,
            cjk: load_face(cjk)?,
        })
    }

    pub fn shape(&self, text: &str, opts: &ShapeOpts) -> Vec<ShapedGlyph> {
        if text.is_empty() {
            return Vec::new();
        }
        let face = if opts.cjk {
            &self.cjk
        } else if opts.italic {
            &self.italic
        } else {
            &self.latin
        };
        let font = if !opts.cjk && opts.weight >= 650 {
            face.font
                .instance_builder()
                .variations([("wght", 650.0)])
                .build()
        } else {
            face.font.clone()
        };
        let mut shaper = ShaperFont::new(&font);
        let scale = (opts.size * 64.0).round().max(64.0) as i32;
        shaper.set_scale(scale);
        let mut buffer = Buffer::new();
        buffer.push_str(text);
        buffer.set_direction(Direction::LeftToRight);
        buffer.set_script(Some(if opts.cjk { Script::HAN } else { Script::LATIN }));
        let lang = if opts.cjk { b"zh" as &[u8] } else { b"en" };
        if let Some(language) = Language::new(lang) {
            buffer.set_language(Some(language));
        }
        let mut features = Vec::new();
        if opts.small_caps {
            features.push(Feature::new(Tag::new(b"smcp"), 1, ..));
        }
        if opts.oldstyle {
            features.push(Feature::new(Tag::new(b"onum"), 1, ..));
        }
        let options = ShapeOptions::new()
            .features(&features)
            .point_size(Some(opts.size));
        if shape(&shaper, &mut buffer, options).is_err() {
            return fallback_glyphs(text, opts.size);
        }
        let mut glyphs = Vec::new();
        for (info, pos) in buffer.glyph_infos().iter().zip(buffer.glyph_positions()) {
            let cluster = info.cluster as usize;
            let char_index = text.get(..cluster).map(|prefix| prefix.chars().count()).unwrap_or(0);
            glyphs.push(ShapedGlyph {
                gid: info.glyph_id,
                advance: pos.x_advance as f32 / 64.0,
                x_offset: pos.x_offset as f32 / 64.0,
                char_index,
            });
        }
        if glyphs.is_empty() {
            fallback_glyphs(text, opts.size)
        } else {
            glyphs
        }
    }

    pub fn draw(
        &self,
        pixmap: &mut Pixmap,
        face: u8,
        gid: u32,
        x: f32,
        baseline: f32,
        size: f32,
        color: Color,
    ) {
        if gid == 0 {
            return;
        }
        let owned = match face {
            1 => &self.italic,
            2 => &self.cjk,
            _ => &self.latin,
        };
        let Ok(font) = FontRef::new(owned.bytes) else {
            return;
        };
        let Some(outline) = font.outline_glyphs().get(GlyphId::new(gid)) else {
            return;
        };
        let mut pen = Pen {
            path: PathBuilder::new(),
            x,
            baseline,
        };
        let settings = DrawSettings::unhinted(Size::new(size), LocationRef::default());
        if outline.draw(settings, &mut pen).is_err() {
            return;
        }
        let Some(path) = pen.path.finish() else {
            return;
        };
        let mut paint = Paint::default();
        paint.set_color(color);
        paint.anti_alias = true;
        pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
    }
}

fn fallback_glyphs(text: &str, size: f32) -> Vec<ShapedGlyph> {
    text.chars()
        .enumerate()
        .map(|(char_index, _)| ShapedGlyph {
            gid: 0,
            advance: size * 0.5,
            x_offset: 0.0,
            char_index,
        })
        .collect()
}

struct Pen {
    path: PathBuilder,
    x: f32,
    baseline: f32,
}

impl OutlinePen for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.path.move_to(self.x + x, self.baseline - y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.path.line_to(self.x + x, self.baseline - y);
    }
    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.path
            .quad_to(self.x + cx, self.baseline - cy, self.x + x, self.baseline - y);
    }
    fn curve_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.path.cubic_to(
            self.x + c1x,
            self.baseline - c1y,
            self.x + c2x,
            self.baseline - c2y,
            self.x + x,
            self.baseline - y,
        );
    }
    fn close(&mut self) {
        self.path.close();
    }
}

fn load_face(path: &str) -> Result<OwnedFace, String> {
    let bytes = std::fs::read(path).map_err(|err| format!("{path}: {err}"))?;
    let bytes: &'static [u8] = Box::leak(bytes.into_boxed_slice());
    let font = Font::new(bytes, 0).ok_or_else(|| format!("font {path} did not parse"))?;
    Ok(OwnedFace { bytes, font })
}

pub fn paper() -> Color {
    Color::from_rgba8(0xff, 0xff, 0xff, 0xff)
}

pub fn ink() -> Color {
    Color::from_rgba8(0x1c, 0x19, 0x15, 0xff)
}
