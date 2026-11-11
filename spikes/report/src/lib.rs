use std::path::Path;

use serde::{Deserialize, Serialize};

pub const FRAME_BUDGET_MS: f64 = 16.7;
pub const GPT3: &str = "/tmp/superreader-fixtures/gpt3.pdf";
pub const PRIDE: &str = "/tmp/superreader-fixtures/pride-se.epub";
pub const ZHAOHUA: &str = "/tmp/superreader-fixtures/zhaohua.epub";
pub const PDFIUM_LIB: &str = "/tmp/pdfium/lib";

pub fn fixtures_dir() -> String {
    std::env::var("SPIKE_FIXTURES").unwrap_or_else(|_| "/tmp/superreader-fixtures".into())
}

pub fn gpt3_path() -> String {
    std::env::var("SPIKE_GPT3").unwrap_or_else(|_| format!("{}/gpt3.pdf", fixtures_dir()))
}

pub fn pride_path() -> String {
    std::env::var("SPIKE_PRIDE").unwrap_or_else(|_| format!("{}/pride-se.epub", fixtures_dir()))
}

pub fn zhaohua_path() -> String {
    std::env::var("SPIKE_ZHAOHUA").unwrap_or_else(|_| format!("{}/zhaohua.epub", fixtures_dir()))
}

pub fn pdfium_lib() -> String {
    pdfium_search(
        std::env::var("SPIKE_PDFIUM").ok(),
        std::env::current_exe().ok().as_deref(),
    )
}

pub fn pdfium_search(override_dir: Option<String>, exe: Option<&Path>) -> String {
    if let Some(dir) = override_dir {
        let dir = dir.trim();
        if !dir.is_empty() {
            return dir.to_string();
        }
    }
    if let Some(exe) = exe {
        if let Some(macos) = exe.parent() {
            if macos.file_name().is_some_and(|name| name == "MacOS") {
                if let Some(contents) = macos.parent() {
                    let lib = contents.join("Frameworks").join("libpdfium.dylib");
                    if lib.is_file() {
                        return contents.join("Frameworks").to_string_lossy().into_owned();
                    }
                }
            }
        }
    }
    PDFIUM_LIB.into()
}

/// 2 device pixels per PDF point, drawn at the page's point size.
/// 4 device pixels per point, drawn at twice the point size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PixelScale {
    Retina1x,
    Zoom2x,
}

impl PixelScale {
    pub const ALL: [PixelScale; 2] = [PixelScale::Retina1x, PixelScale::Zoom2x];

    pub fn px_per_pt(self) -> f32 {
        match self {
            PixelScale::Retina1x => 2.0,
            PixelScale::Zoom2x => 4.0,
        }
    }

    pub fn display(self) -> f32 {
        match self {
            PixelScale::Retina1x => 1.0,
            PixelScale::Zoom2x => 2.0,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            PixelScale::Retina1x => "retina-1x",
            PixelScale::Zoom2x => "zoom-2x",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PdfEngine {
    Pdfium,
    Hayro,
}

impl PdfEngine {
    pub fn name(self) -> &'static str {
        match self {
            PdfEngine::Pdfium => "pdfium",
            PdfEngine::Hayro => "hayro",
        }
    }
}

/// Rectangle in page points. Origin is the top left of the page.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Quad {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Quad {
    pub fn pixels(self, scale: PixelScale) -> Quad {
        let s = scale.px_per_pt();
        Quad {
            x: self.x * s,
            y: self.y * s,
            w: self.w * s,
            h: self.h * s,
        }
    }

    pub fn hits(self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x <= self.x + self.w && y <= self.y + self.h
    }
}

/// What a later comment thread stores. Page points and character offsets
/// do not change when the bitmap is thrown away and drawn again.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Anchor {
    Pdf {
        page: u32,
        start: u32,
        end: u32,
        text: String,
        quads: Vec<Quad>,
    },
    Epub {
        spine_href: String,
        start: u32,
        end: u32,
        text: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PdfSample {
    pub engine: String,
    pub scale: String,
    pub release: bool,
    pub pages: usize,
    pub bind_ms: f64,
    pub open_ms: f64,
    pub first_raster_ms: f64,
    pub open_to_first_ms: f64,
    pub later_page_ms: Vec<f64>,
    pub errors: u32,
    pub rgba_ok: bool,
    pub text_hits: u32,
    pub text_chars: usize,
    pub glyph_boxes: usize,
    pub rss_after_first_kb: u64,
    pub rss_after_last_kb: u64,
    pub px_w: u32,
    pub px_h: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Summary {
    pub engine: String,
    pub scale: String,
    pub runs: usize,
    pub open_to_first_ms: Stat,
    pub first_raster_ms: Stat,
    pub later_page_ms: Stat,
    pub rss_after_last_kb: Stat,
    pub text_hits_min: u32,
    pub errors: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stat {
    pub median: f64,
    pub min: f64,
    pub max: f64,
}

pub fn stat(values: &[f64]) -> Stat {
    let mut xs: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if xs.is_empty() {
        return Stat {
            median: f64::NAN,
            min: f64::NAN,
            max: f64::NAN,
        };
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = xs.len() / 2;
    let median = if xs.len() % 2 == 1 {
        xs[mid]
    } else {
        xs[mid - 1]
    };
    Stat {
        median,
        min: xs[0],
        max: *xs.last().unwrap(),
    }
}

pub fn rss_kb() -> u64 {
    let Ok(out) = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &std::process::id().to_string()])
        .output()
    else {
        return 0;
    };
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

pub fn require_release() -> Result<(), String> {
    if cfg!(debug_assertions) {
        Err("spike measurements run only in --release".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_is_a_real_sample() {
        let s = stat(&[5.0, 1.0, 4.0, 2.0, 3.0]);
        assert_eq!(s.median, 3.0);
        assert_eq!(s.min, 1.0);
        assert_eq!(s.max, 5.0);
    }

    #[test]
    fn zoom_quads_stay_in_page_points() {
        let anchor = Anchor::Pdf {
            page: 2,
            start: 4,
            end: 9,
            text: "hello".into(),
            quads: vec![Quad {
                x: 10.0,
                y: 20.0,
                w: 30.0,
                h: 8.0,
            }],
        };
        let json = serde_json::to_string(&anchor).unwrap();
        let back: Anchor = serde_json::from_str(&json).unwrap();
        let Anchor::Pdf { quads, .. } = &back else {
            panic!("pdf anchor");
        };
        assert!((quads[0].pixels(PixelScale::Zoom2x).w - 120.0).abs() < 0.01);
        assert!((quads[0].w - 30.0).abs() < 0.01);
    }

    #[test]
    fn a_bundled_pdfium_wins_over_the_tmp_default() {
        let root = std::env::temp_dir().join(format!("superreader-pdfium-{}", std::process::id()));
        let exe = root.join("Contents/MacOS/Slowpage");
        let lib = root.join("Contents/Frameworks/libpdfium.dylib");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(exe.parent().unwrap()).unwrap();
        std::fs::create_dir_all(lib.parent().unwrap()).unwrap();
        std::fs::write(&lib, "dylib").unwrap();
        assert_eq!(
            pdfium_search(Some("  ".into()), Some(exe.as_path())),
            root.join("Contents/Frameworks").to_string_lossy()
        );
        assert_eq!(
            pdfium_search(Some("/tmp/pdfium/lib".into()), Some(exe.as_path())),
            "/tmp/pdfium/lib"
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
