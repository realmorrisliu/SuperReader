use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BookKind {
    Epub,
    Pdf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookFile {
    pub path: String,
    pub kind: BookKind,
}

pub fn parse_book(path: &str) -> Option<BookFile> {
    let path = path.trim();
    if path.is_empty() {
        return None;
    }
    let file = Path::new(path);
    if file.is_dir() && file.join("META-INF/container.xml").is_file() {
        return Some(BookFile {
            path: path.to_string(),
            kind: BookKind::Epub,
        });
    }
    let kind = match file
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("epub") => BookKind::Epub,
        Some("pdf") => BookKind::Pdf,
        _ => return None,
    };
    Some(BookFile {
        path: path.to_string(),
        kind,
    })
}

pub fn path_from_file_url(uri: &str) -> Option<String> {
    let rest = uri.strip_prefix("file://")?;
    let rest = rest.strip_prefix("localhost").unwrap_or(rest);
    let rest = if let Some(stripped) = rest.strip_prefix("//") {
        stripped
    } else {
        rest
    };
    if !rest.starts_with('/') {
        return None;
    }
    let path = percent_decode(rest)?;
    let path = path.trim_end_matches('/');
    parse_book(path).map(|book| book.path)
}

pub fn can_open(path: &str) -> bool {
    let Some(book) = parse_book(path) else {
        return false;
    };
    let path = Path::new(&book.path);
    match book.kind {
        BookKind::Pdf => path.is_file(),
        BookKind::Epub => {
            path.is_file() || (path.is_dir() && path.join("META-INF/container.xml").is_file())
        }
    }
}

pub fn in_app_bundle() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let Some(macos) = exe.parent() else {
        return false;
    };
    macos.file_name().is_some_and(|name| name == "MacOS")
        && macos
            .parent()
            .and_then(|contents| contents.file_name())
            .is_some_and(|name| name == "Contents")
}

fn percent_decode(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return None;
            }
            let hi = hex(bytes[index + 1])?;
            let lo = hex(bytes[index + 2])?;
            out.push((hi << 4) | lo);
            index += 3;
        } else {
            out.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(out).ok()
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn books_are_epub_or_pdf_paths() {
        assert_eq!(parse_book("/tmp/Pride.EPUB").unwrap().kind, BookKind::Epub);
        assert_eq!(parse_book("/tmp/paper.pdf").unwrap().kind, BookKind::Pdf);
        assert!(parse_book("/tmp/notes.txt").is_none());
        assert!(parse_book("  ").is_none());
    }

    #[test]
    fn file_urls_keep_the_book_path() {
        assert_eq!(
            path_from_file_url("file:///tmp/pride-se.epub").as_deref(),
            Some("/tmp/pride-se.epub")
        );
        assert_eq!(
            path_from_file_url("file://localhost/tmp/My%20Book.pdf").as_deref(),
            Some("/tmp/My Book.pdf")
        );
        assert!(path_from_file_url("file:///tmp/notes.txt").is_none());
        assert!(path_from_file_url("https://example.com/a.epub").is_none());
        assert_eq!(
            path_from_file_url("file:///tmp/Book.epub/").as_deref(),
            Some("/tmp/Book.epub")
        );
    }

    #[test]
    fn an_epub_package_is_a_folder_with_a_container() {
        let root = std::env::temp_dir().join(format!("slowpage-epub-{}", std::process::id()));
        let folder = root.join("Book.epub");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(folder.join("META-INF")).unwrap();
        std::fs::write(folder.join("META-INF/container.xml"), "<container/>").unwrap();
        assert!(can_open(folder.to_str().unwrap()));
        assert!(!can_open("/tmp/notes.txt"));
        let _ = std::fs::remove_dir_all(&root);
    }
}
