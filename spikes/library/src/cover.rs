use std::path::{Path, PathBuf};

use crate::{install_cover, store_cover_bytes, BookId, CoverFile};

/// Embedded EPUB cover first. A sibling `cover.jpg` or `cover.png` is the fallback.
/// PDF page 1 is rastered by the shell, which calls `Catalog::attach_png`.
pub(crate) fn resolve_cover(book: &Path, support: &Path, id: &BookId) -> Option<CoverFile> {
    if let Some(bytes) = embedded_epub(book) {
        return store_cover_bytes(&bytes, support, id);
    }
    sibling(book).and_then(|src| install_cover(&src, support, id))
}

fn embedded_epub(book: &Path) -> Option<Vec<u8>> {
    let epub = book.is_dir()
        || book
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("epub"));
    if !epub {
        return None;
    }
    spike_epub::cover_bytes(book)
}

fn sibling(book: &Path) -> Option<PathBuf> {
    let dir = book.parent()?;
    for name in ["cover.jpg", "cover.png"] {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
