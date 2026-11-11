use std::fs;
use std::path::PathBuf;

use rusqlite::Connection;

use crate::fixture;
use crate::{
    dropped_open, file_uri_path, open, Availability, Catalog, Home, Import, ImportStep, Notice,
    OpenKind, OpenSurfaces, Scope, SourceName,
};

fn scope_dir(root: PathBuf, support: PathBuf) -> Scope {
    Scope {
        support,
        override_root: Some(root),
        remembered: None,
        home: Home::Closed,
    }
}

fn unique_tag() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|stamp| stamp.as_nanos())
        .unwrap_or(0)
}

fn temp_support() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "spike-library-support-{}-{}",
        std::process::id(),
        unique_tag()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn temp_fixture() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "spike-library-fixture-{}-{}",
        std::process::id(),
        unique_tag()
    ));
    let _ = fs::remove_dir_all(&dir);
    fixture::write_apple(&dir);
    dir
}

#[test]
fn committed_fixture_lists_six_titles() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/apple");
    if !root.join("BKLibrary-fixture.sqlite").is_file() {
        fixture::write_apple(&root);
    }
    let support = temp_support();
    let catalog = open(scope_dir(root, support));
    let titles: Vec<_> = catalog.books().iter().map(|book| book.title()).collect();
    assert!(titles.contains(&"Plain Essay"));
    assert!(titles.contains(&"Field Guide"));
    assert!(titles.contains(&"Folder Book"));
    assert!(titles.contains(&"Cloud Copy"));
    assert!(titles.contains(&"Store Sample"));
    assert!(titles.contains(&"Spoken Sample"));
    assert!(!titles.contains(&"Series Sample"));
    let tally = catalog.import_tally();
    assert_eq!(tally.ready, 3);
    assert_eq!(tally.store, 1);
    assert_eq!(tally.cloud, 1);
    assert!(catalog.settled());
}

#[test]
fn import_catalogues_one_book_per_step() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/apple");
    if !root.join("BKLibrary-fixture.sqlite").is_file() {
        fixture::write_apple(&root);
    }
    let support = temp_support();
    let mut import = Import::begin(scope_dir(root, support));
    let mut count = 0;
    loop {
        match import.step() {
            ImportStep::Book(book) => {
                count += 1;
                assert!(!book.title().is_empty());
                assert!(!Catalog::streaming(vec![book]).settled());
            }
            ImportStep::Finished(catalog) => {
                assert!(catalog.settled());
                assert_eq!(catalog.books().len(), count);
                assert_eq!(count, 6);
                assert_eq!(catalog.import_tally().ready, 3);
                break;
            }
        }
    }
}

#[test]
fn dropped_open_accepts_epub_pdf_and_exploded_dir() {
    let dir = std::env::temp_dir().join(format!(
        "spike-library-drop-{}-{}",
        std::process::id(),
        unique_tag()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let epub = dir.join("Note Sheet.epub");
    fs::write(&epub, b"PK\x03\x04").unwrap();
    let cmd = dropped_open(&epub).unwrap();
    assert_eq!(cmd.kind(), OpenKind::EpubFile);
    assert_eq!(cmd.title(), "Note Sheet");
    let pdf = dir.join("Note Sheet.pdf");
    fs::write(&pdf, b"%PDF").unwrap();
    assert_eq!(dropped_open(&pdf).unwrap().kind(), OpenKind::Pdf);
    let exploded = dir.join("Loose Book");
    fs::create_dir_all(exploded.join("META-INF")).unwrap();
    fs::write(exploded.join("META-INF/container.xml"), b"<c/>").unwrap();
    let cmd = dropped_open(&exploded).unwrap();
    assert_eq!(cmd.kind(), OpenKind::EpubDir);
    assert_eq!(cmd.title(), "Loose Book");
    let placeholder = dir.join(".Cloud Copy.epub.icloud");
    fs::write(&placeholder, b"icloud").unwrap();
    assert!(dropped_open(&placeholder).is_none());
    let audio = dir.join("Spoken Sample.m4b");
    fs::write(&audio, b"audio").unwrap();
    assert!(dropped_open(&audio).is_none());
    assert!(dropped_open(&dir).is_none());
    let decoded = file_uri_path("file:///tmp/a%20b.epub").unwrap();
    assert_eq!(decoded, PathBuf::from("/tmp/a b.epub"));
    let local = file_uri_path("file://localhost/tmp/a.epub").unwrap();
    assert_eq!(local, PathBuf::from("/tmp/a.epub"));
    assert!(file_uri_path("https://example.com/a.epub").is_none());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn closed_variants_have_no_open_command() {
    let dir = temp_fixture();
    let support = temp_support();
    let catalog = open(scope_dir(dir, support));
    for book in catalog.books() {
        let label = book.unavailable_label();
        if label.is_some() {
            assert!(book.open_command().is_none(), "{}", book.title());
        }
    }
    let store = catalog
        .books()
        .iter()
        .find(|book| book.title() == "Store Sample")
        .expect("store");
    assert_eq!(store.unavailable_label(), Some("Store"));
    let cloud = catalog
        .books()
        .iter()
        .find(|book| book.title() == "Cloud Copy")
        .expect("cloud");
    assert_eq!(cloud.unavailable_label(), Some("Not downloaded"));
}

#[test]
fn icloud_placeholder_through_symlinked_root_is_not_downloaded() {
    let real = std::env::temp_dir().join(format!(
        "spike-library-real-{}-{}",
        std::process::id(),
        unique_tag()
    ));
    let link = std::env::temp_dir().join(format!(
        "spike-library-link-{}-{}",
        std::process::id(),
        unique_tag()
    ));
    let _ = fs::remove_dir_all(&real);
    let _ = fs::remove_file(&link);
    fixture::write_apple(&real);
    std::os::unix::fs::symlink(&real, &link).unwrap();
    let support = temp_support();
    let catalog = open(scope_dir(link.clone(), support));
    let cloud = catalog
        .books()
        .iter()
        .find(|book| book.title() == "Cloud Copy")
        .expect("cloud");
    assert_eq!(cloud.unavailable_label(), Some("Not downloaded"));
    let _ = fs::remove_file(&link);
    let _ = fs::remove_dir_all(&real);
}

#[test]
fn ready_shapes_open() {
    let dir = temp_fixture();
    let support = temp_support();
    let catalog = open(scope_dir(dir, support));
    let essay = catalog
        .books()
        .iter()
        .find(|book| book.title() == "Plain Essay")
        .unwrap();
    let cmd = essay.open_command().unwrap();
    assert_eq!(cmd.kind().as_str(), "epub-file");
    let folder = catalog
        .books()
        .iter()
        .find(|book| book.title() == "Folder Book")
        .unwrap();
    assert_eq!(folder.open_command().unwrap().kind().as_str(), "epub-dir");
    let guide = catalog
        .books()
        .iter()
        .find(|book| book.title() == "Field Guide")
        .unwrap();
    assert_eq!(guide.open_command().unwrap().kind().as_str(), "pdf");
}

#[test]
fn snapshot_leaves_source_bytes_alone() {
    let dir = temp_fixture();
    let db = dir.join("BKLibrary-fixture.sqlite");
    let before = fs::read(&db).unwrap();
    let support = temp_support();
    let _ = open(scope_dir(dir, support));
    let after = fs::read(db).unwrap();
    assert_eq!(before, after);
}

#[test]
fn missing_store_id_column_is_unreadable() {
    let dir = std::env::temp_dir().join(format!("spike-library-no-store-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let db = dir.join("BKLibrary-fixture.sqlite");
    let conn = Connection::open(&db).unwrap();
    conn.execute_batch(
        "CREATE TABLE ZBKLIBRARYASSET (ZASSETID TEXT, ZTITLE TEXT, ZAUTHOR TEXT, ZPATH TEXT, ZCONTENTTYPE INTEGER)",
    )
    .unwrap();
    conn.execute(
        "INSERT INTO ZBKLIBRARYASSET VALUES ('x', 'Plain Essay', '', 'Plain Essay.epub', 1)",
        [],
    )
    .unwrap();
    fs::write(dir.join("Plain Essay.epub"), b"epub").unwrap();
    let support = temp_support();
    let catalog = open(scope_dir(dir.clone(), support));
    assert_eq!(catalog.notice(), Some(Notice::Unreadable));
    assert!(catalog.books().is_empty());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn disk_epub_wins_over_audiobook_hint() {
    let dir = temp_fixture();
    let support = temp_support();
    let db = dir.join("BKLibrary-fixture.sqlite");
    let conn = Connection::open(&db).unwrap();
    conn.execute(
        "UPDATE ZBKLIBRARYASSET SET ZCONTENTTYPE = 6 WHERE ZTITLE = 'Plain Essay'",
        [],
    )
    .unwrap();
    drop(conn);
    let catalog = open(scope_dir(dir, support));
    let essay = catalog
        .books()
        .iter()
        .find(|book| book.title() == "Plain Essay")
        .unwrap();
    assert!(essay.open_command().is_some());
}

#[test]
fn copy_failure_returns_notice_and_no_books() {
    let dir = temp_fixture();
    let support_file =
        std::env::temp_dir().join(format!("spike-library-support-file-{}", std::process::id()));
    let _ = fs::remove_file(&support_file);
    fs::write(&support_file, b"x").unwrap();
    let catalog = open(scope_dir(dir, support_file.clone()));
    assert_eq!(catalog.notice(), Some(Notice::CopyFailed));
    assert!(catalog.books().is_empty());
    let _ = fs::remove_file(support_file);
}

#[test]
fn path_outside_override_root_is_not_openable() {
    let dir = temp_fixture();
    let db = dir.join("BKLibrary-fixture.sqlite");
    let conn = Connection::open(&db).unwrap();
    conn.execute(
        "INSERT INTO ZBKLIBRARYASSET (ZASSETID, ZTITLE, ZAUTHOR, ZPATH, ZSTOREID, ZCONTENTTYPE) VALUES ('escape', 'Escape Volume', '', '/etc/passwd', '', 3)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO ZBKLIBRARYASSET (ZASSETID, ZTITLE, ZAUTHOR, ZPATH, ZSTOREID, ZCONTENTTYPE) VALUES ('dotdot', 'Dot Dot Volume', '', 'missing/../../outside.epub', '', 1)",
        [],
    )
    .unwrap();
    drop(conn);
    let support = temp_support();
    let catalog = open(scope_dir(dir, support));
    let escape = catalog
        .books()
        .iter()
        .find(|book| book.title() == "Escape Volume")
        .unwrap();
    assert!(escape.open_command().is_none());
    let dotted = catalog
        .books()
        .iter()
        .find(|book| book.title() == "Dot Dot Volume")
        .unwrap();
    assert!(dotted.open_command().is_none());
}

#[test]
fn home_closed_without_override_reads_nothing() {
    let support = temp_support();
    let catalog = open(Scope {
        support,
        override_root: None,
        remembered: None,
        home: Home::Closed,
    });
    assert!(catalog.books().is_empty());
    assert!(catalog.notice().is_none());
}

#[test]
fn calibre_fixture_lists_title_author_and_cover() {
    let dir = std::env::temp_dir().join(format!("spike-calibre-fixture-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fixture::write_calibre(&dir);
    let support = temp_support();
    let catalog = open(scope_dir(dir.clone(), support.clone()));
    let book = catalog
        .books()
        .iter()
        .find(|book| book.title() == "Harbor Notes")
        .expect("harbor");
    assert_eq!(book.author(), Some("Fixture Author"));
    assert_eq!(book.source().label(), "calibre");
    let cover = fs::read(book.cover().expect("cover").as_path()).unwrap();
    assert!(cover.starts_with(b"\x89PNG"));
    let width = u32::from_be_bytes(cover[16..20].try_into().unwrap());
    let height = u32::from_be_bytes(cover[20..24].try_into().unwrap());
    assert_eq!((width, height), (80, 120));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn admit_keeps_the_other_source_and_a_second_import_refreshes_one() {
    let apple_dir = std::env::temp_dir().join(format!("spike-admit-apple-{}", std::process::id()));
    let calibre_dir =
        std::env::temp_dir().join(format!("spike-admit-calibre-{}", std::process::id()));
    let _ = fs::remove_dir_all(&apple_dir);
    let _ = fs::remove_dir_all(&calibre_dir);
    fixture::write_apple(&apple_dir);
    fixture::write_calibre(&calibre_dir);
    let support = temp_support();
    let apple = open(scope_dir(apple_dir, support.clone()));
    let calibre = open(scope_dir(calibre_dir, support));
    let union = apple.admit(SourceName::Calibre, calibre);
    assert!(union
        .books()
        .iter()
        .any(|book| book.title() == "Plain Essay"));
    assert!(union
        .books()
        .iter()
        .any(|book| book.title() == "Harbor Notes" && book.source().label() == "calibre"));
    let refreshed = union.admit(SourceName::Calibre, Catalog::from_parts(Vec::new(), None));
    assert!(refreshed
        .books()
        .iter()
        .any(|book| book.title() == "Plain Essay"));
    assert!(!refreshed
        .books()
        .iter()
        .any(|book| book.title() == "Harbor Notes"));
}

#[test]
fn an_epub_cover_comes_from_the_package_not_a_one_pixel_sibling() {
    let dir = std::env::temp_dir().join(format!("spike-cover-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fixture::write_apple(&dir);
    let support = temp_support();
    let catalog = open(scope_dir(dir, support));
    let essay = catalog
        .books()
        .iter()
        .find(|book| book.title() == "Plain Essay")
        .unwrap();
    let bytes = fs::read(essay.cover().unwrap().as_path()).unwrap();
    assert!(bytes.starts_with(b"\x89PNG"));
    assert!(bytes.len() > 69);
    let guide = catalog
        .books()
        .iter()
        .find(|book| book.title() == "Field Guide")
        .unwrap();
    assert!(guide.cover().is_none());
    assert_eq!(guide.open_command().unwrap().kind(), OpenKind::Pdf);
}

#[test]
fn override_metadata_db_selects_calibre_not_apple() {
    let dir = std::env::temp_dir().join(format!("spike-calibre-only-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fixture::write_calibre(&dir);
    let support = temp_support();
    let catalog = open(scope_dir(dir, support));
    assert!(catalog
        .books()
        .iter()
        .any(|book| book.title() == "Harbor Notes"));
    assert!(!catalog
        .books()
        .iter()
        .any(|book| book.title() == "Plain Essay"));
}

#[test]
fn no_library_home_falls_through_to_calibre() {
    let home = std::env::temp_dir().join(format!("spike-home-{}", std::process::id()));
    let _ = fs::remove_dir_all(&home);
    fs::create_dir_all(&home).unwrap();
    let calibre = home.join("Calibre Library");
    fixture::write_calibre(&calibre);
    let support = temp_support();
    let catalog = open(Scope {
        support,
        override_root: None,
        remembered: None,
        home: Home::Open(home.clone()),
    });
    assert!(catalog
        .books()
        .iter()
        .any(|book| book.title() == "Harbor Notes"));
    let _ = fs::remove_dir_all(&home);
}

#[test]
fn permission_denied_on_apple_does_not_list_calibre_in_same_tree() {
    let home = std::env::temp_dir().join(format!("spike-perm-home-{}", std::process::id()));
    let _ = fs::remove_dir_all(&home);
    fs::create_dir_all(&home).unwrap();
    let apple_dir = home.join("Library/Containers/com.apple.iBooksX/Data/Documents/BKLibrary");
    fs::create_dir_all(&apple_dir).unwrap();
    let calibre = home.join("Calibre Library");
    fixture::write_calibre(&calibre);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if fs::metadata(&apple_dir)
            .map(|meta| meta.permissions().mode() & 0o222 == 0)
            .unwrap_or(true)
        {
            let mut perms = fs::metadata(&apple_dir).unwrap().permissions();
            perms.set_mode(0o000);
            fs::set_permissions(&apple_dir, perms).unwrap();
            let support = temp_support();
            let catalog = open(Scope {
                support,
                override_root: None,
                remembered: None,
                home: Home::Open(home.clone()),
            });
            assert_eq!(catalog.notice(), Some(Notice::PermissionDenied));
            assert!(!catalog
                .books()
                .iter()
                .any(|book| book.title() == "Harbor Notes"));
            let mut perms = fs::metadata(&apple_dir).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&apple_dir, perms).unwrap();
        }
    }
    let _ = fs::remove_dir_all(&home);
}

#[test]
fn switching_epub_and_pdf_drops_the_other_surface() {
    let epub = OpenSurfaces::closed().open(OpenKind::EpubFile, 1);
    assert!(epub.web);
    assert!(!epub.pdf);
    let pdf = epub.open(OpenKind::Pdf, 4);
    assert!(!pdf.web);
    assert!(pdf.pdf);
    assert_eq!(pdf.page, 4);
    let back = pdf.open(OpenKind::EpubDir, 0);
    assert!(back.web);
    assert!(!back.pdf);
    assert_eq!(back.page, 1);
    let again = back.open(OpenKind::Pdf, 1);
    assert!(!again.web);
    assert!(again.pdf);
}

#[test]
fn store_variant_has_no_path_field() {
    fn assert_store() {
        let _ = Availability::Store;
    }
    assert_store();
}
