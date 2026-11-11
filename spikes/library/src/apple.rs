use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::disk::from_disk;
use crate::sealed::Sealed;
use crate::snapshot::snapshot;
use crate::{
    install_cover, resolve_under_roots, sort_by_id, Availability, Book, BookId, Catalog,
    ImportStep, LibrarySource, Notice, Scope, Unreadable,
};

pub(crate) struct AppleBooks {
    scope: Scope,
}

impl AppleBooks {
    pub(crate) fn new(scope: Scope) -> Self {
        Self { scope }
    }

    fn layout(&self) -> Result<(PathBuf, Vec<PathBuf>), Notice> {
        if let Some(root) = &self.scope.override_root {
            return Ok((root.clone(), vec![root.clone()]));
        }
        let home = crate::home_dir(&self.scope).ok_or(Notice::NoLibrary)?;
        let library_dir =
            home.join("Library/Containers/com.apple.iBooksX/Data/Documents/BKLibrary");
        let mut roots = vec![
            home.join("Library/Mobile Documents/iCloud~com~apple~iBooks/Documents"),
            home.join("Library/Containers/com.apple.BKAgentService/Data/Documents/iBooks/Books"),
        ];
        roots.retain(|path| path.is_dir() || path.parent().is_some());
        Ok((library_dir, roots))
    }

    fn pick_sqlite(dir: &Path) -> Result<PathBuf, Notice> {
        let mut matches: Vec<PathBuf> = fs::read_dir(dir)
            .map_err(|err| {
                if err.kind() == std::io::ErrorKind::PermissionDenied {
                    Notice::PermissionDenied
                } else {
                    Notice::NoLibrary
                }
            })?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(OsStr::to_str)
                    .is_some_and(|name| name.starts_with("BKLibrary-") && name.ends_with(".sqlite"))
            })
            .collect();
        if matches.is_empty() {
            return Err(Notice::NoLibrary);
        }
        matches.sort_by(|left, right| {
            let left_time = fs::metadata(left)
                .and_then(|meta| meta.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            let right_time = fs::metadata(right)
                .and_then(|meta| meta.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            right_time.cmp(&left_time)
        });
        Ok(matches.remove(0))
    }

    fn table_columns(conn: &rusqlite::Connection, table: &str) -> Result<Vec<String>, Notice> {
        let mut stmt = conn
            .prepare(&format!("PRAGMA table_info({table})"))
            .map_err(|_| Notice::Unreadable)?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|_| Notice::Unreadable)?;
        Ok(rows.filter_map(Result::ok).collect())
    }
}

impl Sealed for AppleBooks {}

impl LibrarySource for AppleBooks {
    fn load(&self) -> Catalog {
        match Cursor::open(&self.scope) {
            Ok(cursor) => drain(cursor),
            Err(notice) => Catalog::from_parts(Vec::new(), Some(notice)),
        }
    }
}

#[derive(Clone)]
struct Raw {
    asset_id: String,
    title: String,
    author: Option<String>,
    path: Option<String>,
    store_id: Option<String>,
    content_type: i64,
}

/// One Apple Books row at a time. The sqlite snapshot is read in `open`.
/// `step` classifies that row against disk.
pub(crate) struct Cursor {
    support: PathBuf,
    override_root: Option<PathBuf>,
    book_roots: Vec<PathBuf>,
    rows: Vec<Raw>,
    index: usize,
    books: Vec<Book>,
}

impl Cursor {
    pub(crate) fn open(scope: &Scope) -> Result<Self, Notice> {
        let books = AppleBooks::new(scope.clone());
        let (library_dir, book_roots) = books.layout()?;
        if !library_dir.is_dir() {
            return Err(Notice::NoLibrary);
        }
        let sqlite_path = AppleBooks::pick_sqlite(&library_dir)?;
        let snap = snapshot(&sqlite_path, &scope.support)?;
        let conn = snap.connection();
        let columns = AppleBooks::table_columns(conn, "ZBKLIBRARYASSET")?;
        if !columns.iter().any(|name| name == "ZSTOREID") {
            return Err(Notice::Unreadable);
        }
        let sql =
            "SELECT ZASSETID, ZTITLE, ZAUTHOR, ZPATH, ZSTOREID, ZCONTENTTYPE FROM ZBKLIBRARYASSET";
        let mut stmt = conn.prepare(sql).map_err(|_| Notice::Unreadable)?;
        let rows = stmt
            .query_map([], |row| {
                Ok(Raw {
                    asset_id: row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                    title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    author: row.get::<_, Option<String>>(2)?,
                    path: row.get::<_, Option<String>>(3)?,
                    store_id: row.get::<_, Option<String>>(4)?,
                    content_type: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
                })
            })
            .map_err(|_| Notice::Unreadable)?;
        let rows = rows
            .filter_map(Result::ok)
            .filter(|raw| raw.content_type != 5 && !raw.asset_id.is_empty())
            .collect();
        Ok(Self {
            support: scope.support.clone(),
            override_root: scope.override_root.clone(),
            book_roots,
            rows,
            index: 0,
            books: Vec::new(),
        })
    }

    pub(crate) fn step(&mut self) -> ImportStep {
        let Some(raw) = self.rows.get(self.index).cloned() else {
            sort_by_id(&mut self.books);
            return ImportStep::Finished(Catalog::from_parts(
                std::mem::take(&mut self.books),
                None,
            ));
        };
        self.index += 1;
        let book = self.classify(raw);
        self.books.push(book.clone());
        ImportStep::Book(book)
    }

    fn classify(&self, raw: Raw) -> Book {
        let title = if raw.title.is_empty() {
            "Untitled".to_string()
        } else {
            raw.title
        };
        let author = raw.author.filter(|value| !value.is_empty());
        let id = BookId::apple(&raw.asset_id);
        if raw
            .store_id
            .as_deref()
            .is_some_and(|value| !value.is_empty())
        {
            return Book::new(id, title, author, None, Availability::Store);
        }
        let Some(raw_path) = raw.path.filter(|value| !value.is_empty()) else {
            return Book::new(
                id,
                title,
                author,
                None,
                Availability::Unreadable(Unreadable::Missing),
            );
        };
        let resolved = if let Some(root) = &self.override_root {
            let path = Path::new(&raw_path);
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                root.join(raw_path)
            }
        } else {
            match resolve_under_roots(Path::new(&raw_path), &self.book_roots) {
                Some(path) => path,
                None => {
                    return Book::new(
                        id,
                        title,
                        author,
                        None,
                        Availability::Unreadable(Unreadable::Denied),
                    );
                }
            }
        };
        let roots = if let Some(root) = &self.override_root {
            vec![root.clone()]
        } else {
            self.book_roots.clone()
        };
        let availability = from_disk(&resolved, &roots, raw.content_type == 6);
        let cover =
            cover_sibling(&resolved).and_then(|src| install_cover(&src, &self.support, &id));
        Book::new(id, title, author, cover, availability)
    }
}

fn drain(mut cursor: Cursor) -> Catalog {
    loop {
        match cursor.step() {
            ImportStep::Book(_) => {}
            ImportStep::Finished(catalog) => return catalog,
        }
    }
}

fn cover_sibling(book: &Path) -> Option<PathBuf> {
    let dir = book.parent()?;
    for name in ["cover.jpg", "cover.png"] {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}
