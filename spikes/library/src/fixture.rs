use std::fs;
use std::path::Path;

use rusqlite::Connection;

const PRIDE_CHAPTER: &str = "It is a truth universally acknowledged, that a single man in possession of a good fortune, must be in want of a wife. However little known the feelings or views of such a man may be on his first entering a neighbourhood, this truth is so well fixed in the minds of the surrounding families, that he is considered as the rightful property of some one or other of their daughters. My dear Mr. Bennet, said his lady to him one day, have you heard that Netherfield Park is let at last? Mr. Bennet replied that he had not. But it is, returned she; for Mrs. Long has just been here, and she told me all about it. Mr. Bennet made no answer. Do you not want to know who has taken it? cried his wife impatiently. You want to tell me, and I have no objection to hearing it. This was invitation enough. Why, my dear, you must know, Mrs. Long says that Netherfield is taken by a young man of large fortune from the north of England; that he came down on Monday in a chaise and four to see the place, and was so much delighted with it, that he agreed with Mr. Morris immediately; that he is to take possession before Michaelmas, and some of his servants are to be in the house by the end of next week. What is his name? Bingley. Is he married or single? Oh! single, my dear, to be sure! A single man of large fortune; four or five thousand a year. What a fine thing for our girls! How so? How can it affect them? My dear Mr. Bennet, replied his wife, how can you be so tiresome! You must know that I am thinking of his marrying one of them. Is that his design in settling here? Design! nonsense, how can you talk so! But it is very likely that he may fall in love with one of them, and therefore you must visit him as soon as he comes. ";

const MINIMAL_PDF: &[u8] = include_bytes!("../fixtures/apple/Field Guide.pdf");
const PLAIN_ESSAY: &[u8] = include_bytes!("../fixtures/apple/Plain Essay.epub");

const MINIMAL_PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
    0xde, 0x00, 0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08, 0xd7, 0x63, 0xf8, 0xcf, 0xc0, 0x00,
    0x00, 0x00, 0x02, 0x00, 0x01, 0xe2, 0x21, 0xbc, 0x33, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e,
    0x44, 0xae, 0x42, 0x60, 0x82,
];

/// Apple-shaped tree for tests and committed fixtures.
pub fn write_apple(dir: &Path) {
    fs::create_dir_all(dir).expect("fixture dir");
    write_apple_files(dir);
    write_apple_sqlite(dir);
}

fn write_apple_files(dir: &Path) {
    fs::write(dir.join("Plain Essay.epub"), PLAIN_ESSAY).unwrap();
    fs::write(dir.join("cover.png"), MINIMAL_PNG).unwrap();
    fs::write(dir.join("Field Guide.pdf"), MINIMAL_PDF).unwrap();

    let folder = dir.join("Folder Book");
    fs::create_dir_all(folder.join("META-INF")).unwrap();
    fs::create_dir_all(folder.join("OEBPS")).unwrap();
    fs::write(
        folder.join("META-INF/container.xml"),
        r#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
    )
    .unwrap();
    fs::write(
        folder.join("OEBPS/content.opf"),
        r#"<?xml version="1.0"?>
<package version="2.0" xmlns="http://www.idpf.org/2007/opf" unique-identifier="uuid_id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:title>Folder Book</dc:title>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
    <item id="chap" href="chapter.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="chap"/>
  </spine>
</package>"#,
    )
    .unwrap();
    let chapter = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml"><body><p>{PRIDE_CHAPTER}</p></body></html>"#
    );
    assert!(PRIDE_CHAPTER.len() > 1500);
    fs::write(folder.join("OEBPS/chapter.xhtml"), chapter).unwrap();

    fs::write(dir.join("Spoken Sample.m4b"), b"ftypM4A ").unwrap();

    fs::write(dir.join(".Cloud Copy.epub.icloud"), b"icloud-placeholder").unwrap();
}

fn write_apple_sqlite(dir: &Path) {
    let path = dir.join("BKLibrary-fixture.sqlite");
    let _ = fs::remove_file(&path);
    let conn = Connection::open(&path).expect("sqlite");
    conn.execute_batch(
        r"
        CREATE TABLE ZBKLIBRARYASSET (
            Z_PK INTEGER PRIMARY KEY,
            ZASSETID TEXT,
            ZTITLE TEXT,
            ZAUTHOR TEXT,
            ZPATH TEXT,
            ZSTOREID TEXT,
            ZCONTENTTYPE INTEGER
        );
        ",
    )
    .unwrap();
    let insert = |id: &str, title: &str, author: &str, file: &str, store: &str, ctype: i64| {
        conn.execute(
            "INSERT INTO ZBKLIBRARYASSET (ZASSETID, ZTITLE, ZAUTHOR, ZPATH, ZSTOREID, ZCONTENTTYPE) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![id, title, author, file, store, ctype],
        )
        .unwrap();
    };
    insert(
        "essay-1",
        "Plain Essay",
        "Sample Author",
        "Plain Essay.epub",
        "",
        1,
    );
    insert(
        "guide-1",
        "Field Guide",
        "Sample Author",
        "Field Guide.pdf",
        "",
        3,
    );
    insert(
        "folder-1",
        "Folder Book",
        "Jane Austen",
        "Folder Book",
        "",
        1,
    );
    insert("cloud-1", "Cloud Copy", "", "Cloud Copy.epub", "", 1);
    insert(
        "store-1",
        "Store Sample",
        "",
        "Store Sample.pdf",
        "1234567890",
        3,
    );
    insert("spoken-1", "Spoken Sample", "", "Spoken Sample.m4b", "", 6);
    insert("series-1", "Series Sample", "", "Series Sample.epub", "", 5);
}

pub fn write_calibre(dir: &Path) {
    fs::create_dir_all(dir).unwrap();
    let book_dir = dir.join("Harbor Notes");
    fs::create_dir_all(&book_dir).unwrap();
    fs::write(
        book_dir.join("Harbor Notes.epub"),
        b"PK\x03\x04calibre-epub",
    )
    .unwrap();
    let cover_bytes = MINIMAL_PNG;
    fs::write(book_dir.join("cover.jpg"), cover_bytes).unwrap();
    let conn = Connection::open(dir.join("metadata.db")).expect("metadata");
    conn.execute_batch(
        r"
        CREATE TABLE books (id INTEGER PRIMARY KEY, title TEXT, path TEXT);
        CREATE TABLE authors (id INTEGER PRIMARY KEY, name TEXT);
        CREATE TABLE books_authors_link (book INTEGER, author INTEGER);
        CREATE TABLE data (id INTEGER PRIMARY KEY, book INTEGER, format TEXT, name TEXT);
        ",
    )
    .unwrap();
    conn.execute(
        "INSERT INTO books (id, title, path) VALUES (1, 'Harbor Notes', 'Harbor Notes')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO authors (id, name) VALUES (1, 'Fixture Author')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO books_authors_link (book, author) VALUES (1, 1)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO data (book, format, name) VALUES (1, 'EPUB', 'Harbor Notes')",
        [],
    )
    .unwrap();
}
