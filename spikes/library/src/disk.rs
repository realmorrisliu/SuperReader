use std::path::{Component, Path, PathBuf};

use crate::{Availability, BookPath, Unreadable};

/// Compare roots after resolving symlinks. A missing book still has to be
/// judged from its existing parent: macOS temp dirs are symlinks, and
/// `canonicalize` on the missing file would leave the unreduced path.
fn logical_path(path: &Path) -> PathBuf {
    let mut pending = Vec::new();
    let mut cursor = path;
    let base = loop {
        if let Ok(canon) = cursor.canonicalize() {
            break canon;
        }
        let Some(parent) = cursor.parent() else {
            break cursor.to_path_buf();
        };
        if parent.as_os_str().is_empty() {
            break cursor.to_path_buf();
        }
        if let Some(name) = cursor.file_name() {
            pending.push(name.to_os_string());
        }
        cursor = parent;
    };
    let mut rebuilt = base;
    for name in pending.iter().rev() {
        rebuilt.push(name);
    }
    let mut out = PathBuf::new();
    for comp in rebuilt.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => match out.components().next_back() {
                Some(Component::Normal(_)) => {
                    out.pop();
                }
                Some(Component::ParentDir) | None => out.push(comp),
                Some(Component::RootDir | Component::Prefix(_) | Component::CurDir) => {}
            },
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn under_roots(path: &Path, allowed_roots: &[PathBuf]) -> bool {
    if allowed_roots.is_empty() {
        return true;
    }
    let canonical = logical_path(path);
    allowed_roots
        .iter()
        .any(|root| canonical.starts_with(logical_path(root)))
}

/// Shared disk classification. Store id and series rows stay in `apple`.
pub(crate) fn from_disk(
    path: &Path,
    allowed_roots: &[PathBuf],
    audiobook_hint: bool,
) -> Availability {
    if !under_roots(path, allowed_roots) {
        return Availability::Unreadable(Unreadable::Unsupported);
    }

    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with(".icloud"))
    {
        return Availability::NotDownloaded;
    }

    if !path.exists() {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        let placeholder = parent.join(format!(".{file_name}.icloud"));
        if placeholder.is_file() {
            return Availability::NotDownloaded;
        }
        return Availability::Unreadable(Unreadable::Missing);
    }

    if path.is_dir() {
        let marker = path.join("META-INF/container.xml");
        if marker.is_file() {
            return Availability::EpubDir(BookPath::new(path.to_path_buf()));
        }
        return Availability::Unreadable(Unreadable::Unsupported);
    }

    let lower = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match lower.as_str() {
        "epub" => Availability::EpubFile(BookPath::new(path.to_path_buf())),
        "pdf" => Availability::Pdf(BookPath::new(path.to_path_buf())),
        "m4b" | "m4a" | "aax" => Availability::Audiobook,
        _ if audiobook_hint => Availability::Audiobook,
        _ => Availability::Unreadable(Unreadable::Unsupported),
    }
}
