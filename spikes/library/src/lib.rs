//! spike-library. Read `open`, then `Catalog` and `Book`.

mod apple;
mod calibre;
mod disk;
mod snapshot;

use std::path::{Path, PathBuf};

/// Inputs the shell already has. No field is a database file, a permission
/// result, or a store id.
#[derive(Clone, Debug)]
pub struct Scope {
    pub support: PathBuf,
    pub override_root: Option<PathBuf>,
    pub remembered: Option<PathBuf>,
    pub home: Home,
}

/// Whether an impl may resolve paths under the user's home directory.
#[derive(Clone, Debug)]
pub enum Home {
    /// Impls must not read the home directory.
    Closed,
    /// The home directory. Each impl appends its own library path.
    Open(PathBuf),
}

/// Shelf entry. Returns an owned catalog. The impl does not escape.
pub fn open(scope: Scope) -> Catalog {
    finish(Import::begin(scope))
}

fn finish(mut import: Import) -> Catalog {
    loop {
        match import.step() {
            ImportStep::Book(_) => {}
            ImportStep::Finished(catalog) => return catalog,
        }
    }
}

/// One catalogued book, or the settled catalog when nothing remains.
pub enum ImportStep {
    Book(Book),
    Finished(Catalog),
}

/// Reads a library one book at a time. `step` classifies the next book
/// before it returns that book.
pub struct Import {
    source: ImportSource,
}

enum ImportSource {
    Apple(apple::Cursor),
    Calibre(calibre::Cursor),
    Ready(Option<Catalog>),
}

impl Import {
    pub fn begin(scope: Scope) -> Self {
        if let Some(ref root) = scope.override_root {
            if calibre::root_has_metadata_db(root) {
                return Self::calibre(scope);
            }
            return Self::apple(scope);
        }
        if matches!(scope.home, Home::Closed) {
            return Self::ready(Catalog::from_parts(Vec::new(), None));
        }
        match apple::Cursor::open(&scope) {
            Err(Notice::NoLibrary) => Self::calibre(scope),
            Err(Notice::PermissionDenied) if scope.remembered.is_some() => Self::calibre(scope),
            Err(notice) => Self::ready(Catalog::from_parts(Vec::new(), Some(notice))),
            Ok(cursor) => Self {
                source: ImportSource::Apple(cursor),
            },
        }
    }

    pub fn step(&mut self) -> ImportStep {
        match &mut self.source {
            ImportSource::Apple(cursor) => cursor.step(),
            ImportSource::Calibre(cursor) => cursor.step(),
            ImportSource::Ready(catalog) => ImportStep::Finished(
                catalog
                    .take()
                    .unwrap_or_else(|| Catalog::from_parts(Vec::new(), None)),
            ),
        }
    }

    fn apple(scope: Scope) -> Self {
        match apple::Cursor::open(&scope) {
            Ok(cursor) => Self {
                source: ImportSource::Apple(cursor),
            },
            Err(notice) => Self::ready(Catalog::from_parts(Vec::new(), Some(notice))),
        }
    }

    fn calibre(scope: Scope) -> Self {
        match calibre::Cursor::open(&scope) {
            Ok(cursor) => Self {
                source: ImportSource::Calibre(cursor),
            },
            Err(notice) => Self::ready(Catalog::from_parts(Vec::new(), Some(notice))),
        }
    }

    fn ready(catalog: Catalog) -> Self {
        Self {
            source: ImportSource::Ready(Some(catalog)),
        }
    }
}

/// Owned list. Fields stay private. Child modules use `from_parts`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    books: Vec<Book>,
    notice: Option<Notice>,
    settled: bool,
}

impl Catalog {
    pub(crate) fn from_parts(books: Vec<Book>, notice: Option<Notice>) -> Self {
        Self {
            books,
            notice,
            settled: true,
        }
    }

    /// Rows seen so far. The quiet result line waits until `settled`.
    pub fn streaming(mut books: Vec<Book>) -> Self {
        sort_by_id(&mut books);
        Self {
            books,
            notice: None,
            settled: false,
        }
    }

    pub fn settled(&self) -> bool {
        self.settled
    }

    #[cfg(test)]
    pub(crate) fn notice(&self) -> Option<Notice> {
        self.notice
    }

    pub fn books(&self) -> &[Book] {
        &self.books
    }

    /// Shelf text. None when the book list needs no extra line.
    pub fn notice_label(&self) -> Option<&'static str> {
        self.notice.map(Notice::label)
    }

    /// The shell offers folder choice when this is true.
    pub fn needs_folder(&self) -> bool {
        matches!(
            self.notice,
            Some(Notice::PermissionDenied | Notice::NotALibrary)
        )
    }

    /// Ready books, store-protected books, and iCloud placeholders.
    /// Audiobooks and other unreadable rows stay out of the counts.
    pub fn import_tally(&self) -> ImportTally {
        let mut tally = ImportTally::default();
        for book in &self.books {
            if book.open_command().is_some() {
                tally.ready += 1;
            } else if book.unavailable_label() == Some("Store") {
                tally.store += 1;
            } else if book.unavailable_label() == Some("Not downloaded") {
                tally.cloud += 1;
            }
        }
        tally
    }
}

/// Counts for the one quiet line under an import.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImportTally {
    pub ready: usize,
    pub store: usize,
    pub cloud: usize,
}

/// Opaque lookup key. Callers do not parse the inner string.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BookId(String);

impl BookId {
    pub(crate) fn apple(asset_id: &str) -> Self {
        Self(format!("apple:{asset_id}"))
    }

    pub(crate) fn calibre(book_id: i64) -> Self {
        Self(format!("calibre:{book_id}"))
    }
}

/// Absolute path the stage may open. Ready variants are the only holders.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookPath(PathBuf);

impl BookPath {
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub(crate) fn new(path: PathBuf) -> Self {
        Self(path)
    }
}

/// Cover copied under the support directory. Distinct from `BookPath`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoverFile(PathBuf);

impl CoverFile {
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Book {
    id: BookId,
    title: String,
    author: Option<String>,
    cover: Option<CoverFile>,
    availability: Availability,
}

impl Book {
    pub(crate) fn new(
        id: BookId,
        title: String,
        author: Option<String>,
        cover: Option<CoverFile>,
        availability: Availability,
    ) -> Self {
        Self {
            id,
            title,
            author,
            cover,
            availability,
        }
    }

    pub fn id(&self) -> &BookId {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }

    /// Path for an NSH image. None means the shelf draws a title tile.
    pub fn cover(&self) -> Option<&CoverFile> {
        self.cover.as_ref()
    }

    /// None unless the book is a ready epub file, epub directory, or pdf.
    pub fn open_command(&self) -> Option<OpenCommand> {
        let (kind, path) = match &self.availability {
            Availability::EpubFile(path) => (OpenKind::EpubFile, path.clone()),
            Availability::EpubDir(path) => (OpenKind::EpubDir, path.clone()),
            Availability::Pdf(path) => (OpenKind::Pdf, path.clone()),
            Availability::NotDownloaded
            | Availability::Store
            | Availability::Audiobook
            | Availability::Unreadable(_) => return None,
        };
        Some(OpenCommand {
            kind,
            path,
            title: self.title.clone(),
        })
    }

    pub fn unavailable_label(&self) -> Option<&'static str> {
        Some(match &self.availability {
            Availability::NotDownloaded => "Not downloaded",
            Availability::Store => "Store",
            Availability::Audiobook => "Audiobook",
            Availability::Unreadable(Unreadable::Missing) => "Missing",
            Availability::Unreadable(Unreadable::Unsupported) => "Unsupported",
            Availability::Unreadable(Unreadable::Denied) => "Not readable",
            Availability::EpubFile(_) | Availability::EpubDir(_) | Availability::Pdf(_) => {
                return None
            }
        })
    }
}

/// Arguments for the stage command `open`. Built only from a ready book.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenCommand {
    kind: OpenKind,
    path: BookPath,
    title: String,
}

impl OpenCommand {
    pub fn kind(&self) -> OpenKind {
        self.kind
    }

    pub fn path(&self) -> &BookPath {
        &self.path
    }

    pub fn title(&self) -> &str {
        &self.title
    }
}

/// The only kind list. The stage uses `parse` and does not restate these strings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenKind {
    EpubFile,
    EpubDir,
    Pdf,
}

impl OpenKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EpubFile => "epub-file",
            Self::EpubDir => "epub-dir",
            Self::Pdf => "pdf",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "epub-file" => Some(Self::EpubFile),
            "epub-dir" => Some(Self::EpubDir),
            "pdf" => Some(Self::Pdf),
            _ => None,
        }
    }
}

/// Which reading surface is up after an open. Opening always drops the other
/// surface first, so an EPUB cannot stay over a PDF and a PDF cannot stay
/// over an EPUB.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenSurfaces {
    pub web: bool,
    pub pdf: bool,
    pub page: u32,
}

impl OpenSurfaces {
    pub fn closed() -> Self {
        Self {
            web: false,
            pdf: false,
            page: 1,
        }
    }

    pub fn open(self, kind: OpenKind, page: u32) -> Self {
        let _previous = self;
        let page = page.max(1);
        match kind {
            OpenKind::Pdf => Self {
                web: false,
                pdf: true,
                page,
            },
            OpenKind::EpubFile | OpenKind::EpubDir => Self {
                web: true,
                pdf: false,
                page,
            },
        }
    }
}

/// A file the shelf can open without a catalog: an EPUB file, an exploded
/// EPUB directory, or a PDF. Placeholders and other extensions return none.
pub fn dropped_open(path: &Path) -> Option<OpenCommand> {
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("icloud"))
    {
        return None;
    }
    let kind = if path.is_dir() {
        if path.join("META-INF/container.xml").is_file() {
            OpenKind::EpubDir
        } else {
            return None;
        }
    } else {
        match path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .as_deref()
        {
            Some("epub") => OpenKind::EpubFile,
            Some("pdf") => OpenKind::Pdf,
            _ => return None,
        }
    };
    let title = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|stem| !stem.is_empty())
        .unwrap_or("Untitled")
        .to_string();
    Some(OpenCommand {
        kind,
        path: BookPath::new(path.to_path_buf()),
        title,
    })
}

/// `file:` URL to a path. Encoded octets are decoded. Other schemes return none.
pub fn file_uri_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let rest = rest.strip_prefix("localhost").unwrap_or(rest);
    let rest = rest.strip_prefix("//").unwrap_or(rest);
    let decoded = percent_decode(rest);
    if decoded.is_empty() {
        None
    } else {
        Some(PathBuf::from(decoded))
    }
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(value) = u8::from_str_radix(
                std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap_or(""),
                16,
            ) {
                out.push(value);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub(crate) fn sort_by_id(books: &mut [Book]) {
    books.sort_by(|left, right| left.id.0.cmp(&right.id.0));
}

/// Copies a sibling image into `{support}/covers/` by temp file and rename.
/// The file name comes from `BookId` with the colon replaced. None on failure.
pub(crate) fn install_cover(src: &Path, support: &Path, id: &BookId) -> Option<CoverFile> {
    let covers = support.join("covers");
    if std::fs::create_dir_all(&covers).is_err() {
        return None;
    }
    let name = id.0.replace(':', "_");
    let dest = covers.join(&name);
    let tmp = covers.join(format!("{name}.part"));
    if std::fs::copy(src, &tmp).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return None;
    }
    if std::fs::rename(&tmp, &dest).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return None;
    }
    Some(CoverFile(dest))
}

mod sealed {
    pub trait Sealed {}
}

/// One method. Path resolution, permission, the snapshot, and DRM stay inside.
pub trait LibrarySource: sealed::Sealed {
    fn load(&self) -> Catalog;
}

/// Ready variants own a `BookPath`. The closed variants own none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Availability {
    EpubFile(BookPath),
    EpubDir(BookPath),
    Pdf(BookPath),
    NotDownloaded,
    Store,
    Audiobook,
    Unreadable(Unreadable),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Unreadable {
    Missing,
    Unsupported,
    Denied,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Notice {
    PermissionDenied,
    NoLibrary,
    CopyFailed,
    NotALibrary,
    Unreadable,
}

impl Notice {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::PermissionDenied => "Library access is off",
            Self::NoLibrary => "No library",
            Self::CopyFailed => "Library copy failed",
            Self::NotALibrary => "Chosen folder has no library",
            Self::Unreadable => "Library could not be read",
        }
    }
}

#[cfg(any(test, feature = "fixture"))]
pub mod fixture;

pub(crate) fn home_dir(scope: &Scope) -> Option<&PathBuf> {
    match &scope.home {
        Home::Open(path) => Some(path),
        Home::Closed => None,
    }
}

pub(crate) fn resolve_under_roots(path: &Path, roots: &[PathBuf]) -> Option<PathBuf> {
    if path.is_absolute() {
        for root in roots {
            if path.starts_with(root) {
                return Some(path.to_path_buf());
            }
        }
        return None;
    }
    for root in roots {
        let joined = root.join(path);
        if joined.starts_with(root) {
            return Some(joined);
        }
    }
    None
}

#[cfg(test)]
mod tests;
