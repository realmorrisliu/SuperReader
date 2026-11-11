use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::cover::resolve_cover;
use crate::disk::from_disk;
use crate::sealed::Sealed;
use crate::snapshot::snapshot;
use crate::{
    home_dir, sort_by_id, Availability, Book, BookId, Catalog, ImportStep, LibrarySource, Notice,
    Scope, SourceName, Unreadable,
};

pub(crate) struct Calibre {
    scope: Scope,
}

impl Calibre {
    pub(crate) fn new(scope: Scope) -> Self {
        Self { scope }
    }

    fn root(&self) -> Result<PathBuf, Notice> {
        if let Some(root) = &self.scope.override_root {
            if root_has_metadata_db(root) {
                return Ok(root.clone());
            }
            return Err(Notice::NotALibrary);
        }
        if let Some(remembered) = &self.scope.remembered {
            if root_has_metadata_db(remembered) {
                return Ok(remembered.clone());
            }
            return Err(Notice::NotALibrary);
        }
        let home = home_dir(&self.scope).ok_or(Notice::NoLibrary)?;
        let root = home.join("Calibre Library");
        if root_has_metadata_db(&root) {
            Ok(root)
        } else {
            Err(Notice::NoLibrary)
        }
    }
}

impl Sealed for Calibre {}

impl LibrarySource for Calibre {
    fn load(&self) -> Catalog {
        match Cursor::open(&self.scope) {
            Ok(cursor) => drain(cursor),
            Err(notice) => Catalog::from_parts(Vec::new(), Some(notice)),
        }
    }
}

#[derive(Clone)]
struct Pending {
    id: i64,
    title: String,
    author: Option<String>,
    folder: String,
    formats: Vec<(String, String)>,
}

/// One Calibre book at a time. The database rows are read in `open`.
/// `step` classifies that book's files.
pub(crate) struct Cursor {
    root: PathBuf,
    support: PathBuf,
    rows: Vec<Pending>,
    index: usize,
    books: Vec<Book>,
}

impl Cursor {
    pub(crate) fn open(scope: &Scope) -> Result<Self, Notice> {
        let library = Calibre::new(scope.clone());
        let root = library.root()?;
        let snap = snapshot(&root.join("metadata.db"), &scope.support)?;
        let conn = snap.connection();
        let book_sql = r"
            SELECT books.id, books.title, books.path,
                   group_concat(authors.name, ', ')
            FROM books
            LEFT JOIN books_authors_link ON books_authors_link.book = books.id
            LEFT JOIN authors ON authors.id = books_authors_link.author
            GROUP BY books.id
        ";
        let mut stmt = conn.prepare(book_sql).map_err(|_| Notice::Unreadable)?;
        let queried = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                ))
            })
            .map_err(|_| Notice::Unreadable)?;
        let mut entries: HashMap<i64, Pending> = HashMap::new();
        for row in queried {
            let (id, title, folder, authors) = row.map_err(|_| Notice::Unreadable)?;
            entries.insert(
                id,
                Pending {
                    id,
                    title: title
                        .filter(|value| !value.is_empty())
                        .unwrap_or_else(|| "Untitled".to_string()),
                    author: authors.filter(|value| !value.is_empty()),
                    folder: folder.unwrap_or_default(),
                    formats: Vec::new(),
                },
            );
        }
        let mut fmt_stmt = conn
            .prepare("SELECT book, format, name FROM data")
            .map_err(|_| Notice::Unreadable)?;
        let formats = fmt_stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .map_err(|_| Notice::Unreadable)?;
        for row in formats {
            let (book_id, format, stem) = row.map_err(|_| Notice::Unreadable)?;
            if let Some(pending) = entries.get_mut(&book_id) {
                if let (Some(format), Some(stem)) = (format, stem) {
                    if !format.is_empty() && !stem.is_empty() {
                        pending.formats.push((format, stem));
                    }
                }
            }
        }
        let mut rows: Vec<Pending> = entries.into_values().collect();
        rows.sort_by_key(|row| row.id);
        Ok(Self {
            root,
            support: scope.support.clone(),
            rows,
            index: 0,
            books: Vec::new(),
        })
    }

    pub(crate) fn step(&mut self) -> ImportStep {
        let Some(pending) = self.rows.get(self.index).cloned() else {
            sort_by_id(&mut self.books);
            return ImportStep::Finished(Catalog::from_parts(
                std::mem::take(&mut self.books),
                None,
            ));
        };
        self.index += 1;
        let book = self.classify(pending);
        self.books.push(book.clone());
        ImportStep::Book(book)
    }

    fn classify(&self, pending: Pending) -> Book {
        let id = BookId::calibre(pending.id);
        if pending.folder.is_empty() {
            return Book::new(
                id,
                SourceName::Calibre,
                pending.title,
                pending.author,
                None,
                Availability::Unreadable(Unreadable::Missing),
            );
        }
        let book_dir = self.root.join(&pending.folder);
        let mut availability = Availability::Unreadable(Unreadable::Missing);
        for (format, stem) in pending.formats {
            let file = book_dir.join(format!("{stem}.{}", format.to_ascii_lowercase()));
            let candidate = from_disk(&file, std::slice::from_ref(&self.root), false);
            if matches!(
                candidate,
                Availability::EpubFile(_) | Availability::EpubDir(_) | Availability::Pdf(_)
            ) {
                availability = candidate;
                break;
            }
        }
        let cover = match &availability {
            Availability::EpubFile(path)
            | Availability::EpubDir(path)
            | Availability::Pdf(path) => resolve_cover(path.as_path(), &self.support, &id),
            _ => resolve_cover(&book_dir, &self.support, &id),
        };
        Book::new(
            id,
            SourceName::Calibre,
            pending.title,
            pending.author,
            cover,
            availability,
        )
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

/// Filename check kept next to the Calibre root rule.
pub(crate) fn root_has_metadata_db(root: &Path) -> bool {
    root.join("metadata.db").is_file()
}
