use std::fs;
use std::path::Path;

const EYE_SCALE: f64 = 1.3;

struct Slot {
    name: &'static str,
    px: u32,
    small: bool,
    preview: Option<&'static str>,
}

const SLOTS: &[Slot] = &[
    Slot {
        name: "icon_16x16.png",
        px: 16,
        small: true,
        preview: Some("Slowpage-icon-16.png"),
    },
    Slot {
        name: "icon_16x16@2x.png",
        px: 32,
        small: false,
        preview: None,
    },
    Slot {
        name: "icon_32x32.png",
        px: 32,
        small: true,
        preview: Some("Slowpage-icon-32.png"),
    },
    Slot {
        name: "icon_32x32@2x.png",
        px: 64,
        small: false,
        preview: None,
    },
    Slot {
        name: "icon_128x128.png",
        px: 128,
        small: false,
        preview: Some("Slowpage-icon-128.png"),
    },
    Slot {
        name: "icon_128x128@2x.png",
        px: 256,
        small: false,
        preview: None,
    },
    Slot {
        name: "icon_256x256.png",
        px: 256,
        small: false,
        preview: None,
    },
    Slot {
        name: "icon_256x256@2x.png",
        px: 512,
        small: false,
        preview: None,
    },
    Slot {
        name: "icon_512x512.png",
        px: 512,
        small: false,
        preview: None,
    },
    Slot {
        name: "icon_512x512@2x.png",
        px: 1024,
        small: false,
        preview: Some("Slowpage-icon-1024.png"),
    },
];

pub fn write_set(svg: &str, iconset: &Path, preview: &Path) -> Result<(), String> {
    fs::create_dir_all(iconset).map_err(|err| err.to_string())?;
    fs::create_dir_all(preview).map_err(|err| err.to_string())?;
    let small = small_variant(svg)?;
    for slot in SLOTS {
        let source = if slot.small { small.as_str() } else { svg };
        let png = raster_png(source, slot.px)?;
        fs::write(iconset.join(slot.name), &png).map_err(|err| err.to_string())?;
        if let Some(name) = slot.preview {
            fs::write(preview.join(name), &png).map_err(|err| err.to_string())?;
        }
    }
    Ok(())
}

pub fn raster_png(svg: &str, px: u32) -> Result<Vec<u8>, String> {
    let pixmap = raster(svg, px)?;
    pixmap.encode_png().map_err(|err| err.to_string())
}

pub fn raster(svg: &str, px: u32) -> Result<resvg::tiny_skia::Pixmap, String> {
    let tree = resvg::usvg::Tree::from_str(svg, &resvg::usvg::Options::default())
        .map_err(|err| err.to_string())?;
    let size = tree.size();
    let width = size.width();
    let height = size.height();
    if width <= 0.0 || height <= 0.0 {
        return Err("svg has no size".into());
    }
    let scale = px as f32 / width;
    let w = (width * scale).ceil().max(1.0) as u32;
    let h = (height * scale).ceil().max(1.0) as u32;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h).ok_or("pixmap")?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    let ink = pixmap.pixels().iter().filter(|px| px.alpha() > 0).count();
    if ink == 0 {
        return Err("icon raster is empty".into());
    }
    Ok(pixmap)
}

/// 16 and 32 point @1x. The page sliver and the back rim are thinner than a
/// pixel there, so this keeps the slab and draws the eyes wider.
pub fn small_variant(svg: &str) -> Result<String, String> {
    let face = between(svg, "id=\"face\" d=\"", "\"")?;
    let edge: Vec<&str> = svg
        .lines()
        .map(str::trim)
        .filter(|line| line.contains("fill=\"#E36103\""))
        .collect();
    let face_use: Vec<&str> = svg
        .lines()
        .map(str::trim)
        .filter(|line| line.contains("fill=\"#FC7401\""))
        .collect();
    let rects: Vec<&str> = svg
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("<rect "))
        .collect();
    if edge.is_empty() || face_use.len() != 1 || rects.len() != 2 {
        return Err("icon svg is missing the slab or the eyes".into());
    }
    let mut eyes = String::new();
    for rect in rects {
        eyes.push_str("    ");
        eyes.push_str(&widen_eye(rect)?);
        eyes.push('\n');
    }
    Ok(format!(
        concat!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"1024\" height=\"1024\" viewBox=\"0 0 1024 1024\">\n",
            "  <defs>\n",
            "    <path id=\"face\" d=\"{face}\"/>\n",
            "  </defs>\n",
            "  {edge}\n",
            "  {face_use}\n",
            "  <g fill=\"#FCFCFC\">\n",
            "{eyes}",
            "  </g>\n",
            "</svg>\n",
        ),
        face = face,
        edge = edge.join("\n  "),
        face_use = face_use[0],
        eyes = eyes,
    ))
}

fn widen_eye(tag: &str) -> Result<String, String> {
    let x = attr(tag, "x")?
        .parse::<f64>()
        .map_err(|err| err.to_string())?;
    let width = attr(tag, "width")?
        .parse::<f64>()
        .map_err(|err| err.to_string())?;
    let rx = attr(tag, "rx")?
        .parse::<f64>()
        .map_err(|err| err.to_string())?;
    let mut widened = tag.to_string();
    widened = replace_attr(&widened, "x", &num(x - width * (EYE_SCALE - 1.0) / 2.0))?;
    widened = replace_attr(&widened, "width", &num(width * EYE_SCALE))?;
    widened = replace_attr(&widened, "rx", &num(rx * EYE_SCALE))?;
    Ok(widened)
}

fn attr<'a>(tag: &'a str, name: &str) -> Result<&'a str, String> {
    let key = format!("{name}=\"");
    between(tag, &key, "\"")
}

fn replace_attr(tag: &str, name: &str, value: &str) -> Result<String, String> {
    let key = format!("{name}=\"");
    let start = tag.find(&key).ok_or_else(|| format!("missing {name}"))? + key.len();
    let end = start
        + tag[start..]
            .find('"')
            .ok_or_else(|| format!("truncated {name}"))?;
    let mut out = String::new();
    out.push_str(&tag[..start]);
    out.push_str(value);
    out.push_str(&tag[end..]);
    Ok(out)
}

fn between<'a>(text: &'a str, start_mark: &str, end_mark: &str) -> Result<&'a str, String> {
    let start = text
        .find(start_mark)
        .ok_or_else(|| format!("missing {start_mark}"))?
        + start_mark.len();
    let end = start
        + text[start..]
            .find(end_mark)
            .ok_or_else(|| format!("truncated {start_mark}"))?;
    Ok(&text[start..end])
}

fn num(value: f64) -> String {
    let rounded = (value * 1000.0).round() / 1000.0;
    let text = format!("{rounded:.3}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const SVG: &str = include_str!("../../../assets/icon/Slowpage.svg");

    #[test]
    fn source_keeps_the_chosen_drawing() {
        assert!(SVG.contains("M606.1 112.5"));
        assert!(SVG.contains("fill=\"#FC7401\""));
        assert!(SVG.contains("fill=\"#E15F02\""));
        assert!(SVG.contains("fill=\"#F9DEC6\""));
        assert!(SVG.contains("width=\"72.5\""));
    }

    #[test]
    fn small_variant_drops_the_sliver_and_widens_the_eyes() {
        let small = small_variant(SVG).unwrap();
        assert!(!small.contains("#E15F02"));
        assert!(!small.contains("#F9DEC6"));
        assert!(!small.contains("#F3CBAC"));
        assert!(small.contains("#FC7401"));
        assert!(small.contains("#E36103"));
        assert!(small.contains("width=\"94.25\""));
        assert!(small.contains("width=\"92.95\""));
        assert!(small.contains("rx=\"47.19\""));
        assert!(small.contains("rx=\"46.41\""));
    }

    fn tally(
        svg: &str,
        px: u32,
        pred: fn(&resvg::tiny_skia::PremultipliedColorU8) -> bool,
    ) -> usize {
        raster(svg, px)
            .unwrap()
            .pixels()
            .iter()
            .filter(|pixel| pred(pixel))
            .count()
    }

    fn orange(pixel: &resvg::tiny_skia::PremultipliedColorU8) -> bool {
        pixel.alpha() > 200
            && pixel.red() > 220
            && pixel.green() > 80
            && pixel.green() < 160
            && pixel.blue() < 40
    }

    fn cream(pixel: &resvg::tiny_skia::PremultipliedColorU8) -> bool {
        pixel.alpha() > 200 && pixel.red() > 230 && pixel.green() > 190 && pixel.blue() > 160
    }

    fn eye(pixel: &resvg::tiny_skia::PremultipliedColorU8) -> bool {
        pixel.alpha() > 200 && pixel.red() > 230 && pixel.green() > 230 && pixel.blue() > 230
    }

    #[test]
    fn full_icon_has_the_page_and_small_icon_has_wider_eyes() {
        assert!(tally(SVG, 128, orange) > 100);
        assert!(tally(SVG, 128, cream) > 20);
        let full_eyes = tally(SVG, 128, eye);
        assert!(full_eyes > 10);
        let small = small_variant(SVG).unwrap();
        assert!(tally(&small, 128, orange) > 100);
        assert!(tally(&small, 128, eye) > full_eyes);
    }
}
