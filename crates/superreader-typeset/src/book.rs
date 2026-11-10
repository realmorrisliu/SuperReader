use crate::types::TypesetError;
use rbook::epub::Epub;
#[cfg(test)]
use std::path::Path;

pub struct LoadedChapter {
    pub href: String,
    pub xhtml: String,
    pub stylesheet: String,
}

pub fn load_chapter(epub_path: &str, href_suffix: &str) -> Result<LoadedChapter, TypesetError> {
    let epub = Epub::open(epub_path).map_err(|err| TypesetError::Epub(err.to_string()))?;
    let mut found = None;
    for entry in epub.spine().iter() {
        let Some(manifest) = entry.manifest_entry() else { continue };
        let href = manifest.href().as_str().to_string();
        if href.ends_with(href_suffix) {
            let xhtml = manifest.read_str().map_err(|err| TypesetError::Epub(err.to_string()))?;
            found = Some((href, xhtml));
            break;
        }
    }
    let Some((href, xhtml)) = found else {
        return Err(TypesetError::Epub(format!("spine has no {href_suffix}")));
    };
    let mut stylesheet = String::new();
    for raw in link_hrefs(&xhtml) {
        let joined = join_href(&href, &raw);
        if let Some(entry) = epub.manifest().by_href(&joined) {
            if let Ok(css) = entry.read_str() {
                stylesheet.push_str(&css);
                stylesheet.push('\n');
            }
        }
    }
    Ok(LoadedChapter { href, xhtml, stylesheet })
}

#[cfg(test)]
pub fn fixture_dir() -> &'static str {
    "/tmp/superreader-fixtures"
}

#[cfg(test)]
pub fn fonts_present() -> bool {
    Path::new(fixture_dir()).join("EBGaramond-Regular.ttf").is_file()
}

fn link_hrefs(xhtml: &str) -> Vec<String> {
    let mut out = Vec::new();
    let lower = xhtml.to_ascii_lowercase();
    let mut from = 0;
    while let Some(rel) = lower[from..].find("<link") {
        let start = from + rel;
        let Some(end) = xhtml[start..].find('>') else { break };
        let tag = &xhtml[start..start + end];
        if tag.to_ascii_lowercase().contains("stylesheet") {
            if let Some(href) = attr_href(tag) {
                out.push(href);
            }
        }
        from = start + end + 1;
    }
    out
}

fn attr_href(tag: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let pos = lower.find("href=")?;
    let rest = tag[pos + 5..].trim_start();
    let quote = rest.chars().next()?;
    if quote == '"' || quote == '\'' {
        let body = &rest[quote.len_utf8()..];
        let end = body.find(quote)?;
        Some(body[..end].to_string())
    } else {
        None
    }
}

fn join_href(base: &str, rel: &str) -> String {
    if rel.starts_with('/') {
        return rel.trim_start_matches('/').to_string();
    }
    let mut parts: Vec<&str> = base.rsplitn(2, '/').nth(1).unwrap_or("").split('/').filter(|p| !p.is_empty()).collect();
    for part in rel.split('/') {
        if part == ".." {
            parts.pop();
        } else if part != "." && !part.is_empty() {
            parts.push(part);
        }
    }
    parts.join("/")
}
