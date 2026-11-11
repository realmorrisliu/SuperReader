use std::fs;
use std::path::Path;

use rusqlite::Connection;

const PRIDE_CHAPTER: &str = "It is a truth universally acknowledged, that a single man in possession of a good fortune, must be in want of a wife. However little known the feelings or views of such a man may be on his first entering a neighbourhood, this truth is so well fixed in the minds of the surrounding families, that he is considered as the rightful property of some one or other of their daughters. My dear Mr. Bennet, said his lady to him one day, have you heard that Netherfield Park is let at last? Mr. Bennet replied that he had not. But it is, returned she; for Mrs. Long has just been here, and she told me all about it. Mr. Bennet made no answer. Do you not want to know who has taken it? cried his wife impatiently. You want to tell me, and I have no objection to hearing it. This was invitation enough. Why, my dear, you must know, Mrs. Long says that Netherfield is taken by a young man of large fortune from the north of England; that he came down on Monday in a chaise and four to see the place, and was so much delighted with it, that he agreed with Mr. Morris immediately; that he is to take possession before Michaelmas, and some of his servants are to be in the house by the end of next week. What is his name? Bingley. Is he married or single? Oh! single, my dear, to be sure! A single man of large fortune; four or five thousand a year. What a fine thing for our girls! How so? How can it affect them? My dear Mr. Bennet, replied his wife, how can you be so tiresome! You must know that I am thinking of his marrying one of them. Is that his design in settling here? Design! nonsense, how can you talk so! But it is very likely that he may fall in love with one of them, and therefore you must visit him as soon as he comes. ";

const COVER_W: u32 = 80;
const COVER_H: u32 = 120;

/// Apple-shaped tree for tests and committed fixtures.
pub fn write_apple(dir: &Path) {
    fs::create_dir_all(dir).expect("fixture dir");
    write_apple_files(dir);
    write_apple_sqlite(dir);
}

fn write_apple_files(dir: &Path) {
    let _ = fs::remove_file(dir.join("cover.png"));
    let _ = fs::remove_file(dir.join("cover.jpg"));
    let essay_cover = solid_png(COVER_W, COVER_H, 47, 79, 62);
    fs::write(
        dir.join("Plain Essay.epub"),
        epub_with_cover("plain-essay", "Plain Essay", &essay_cover),
    )
    .unwrap();
    fs::write(dir.join("Field Guide.pdf"), colored_pdf(0.10, 0.10, 0.12)).unwrap();

    let folder = dir.join("Folder Book");
    fs::create_dir_all(folder.join("META-INF")).unwrap();
    fs::create_dir_all(folder.join("OEBPS")).unwrap();
    let folder_cover = solid_png(COVER_W, COVER_H, 139, 58, 58);
    fs::write(folder.join("OEBPS/cover.png"), &folder_cover).unwrap();
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
<package version="3.0" xmlns="http://www.idpf.org/2007/opf" unique-identifier="uuid_id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="uuid_id">folder-book</dc:identifier>
    <dc:title>Folder Book</dc:title>
    <dc:language>en</dc:language>
    <meta name="cover" content="cover-img"/>
  </metadata>
  <manifest>
    <item id="cover-img" href="cover.png" media-type="image/png" properties="cover-image"/>
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

pub fn write_store_only(dir: &Path) {
    fs::create_dir_all(dir).unwrap();
    let path = dir.join("BKLibrary-store.sqlite");
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
    conn.execute(
        "INSERT INTO ZBKLIBRARYASSET (ZASSETID, ZTITLE, ZAUTHOR, ZPATH, ZSTOREID, ZCONTENTTYPE) VALUES ('store-1', 'Store Sample', 'Sample Author', 'Store Sample.pdf', '1234567890', 3)",
        [],
    )
    .unwrap();
}

pub fn write_calibre(dir: &Path) {
    fs::create_dir_all(dir).unwrap();
    let _ = fs::remove_file(dir.join("metadata.db"));
    let book_dir = dir.join("Harbor Notes");
    fs::create_dir_all(&book_dir).unwrap();
    let _ = fs::remove_file(book_dir.join("cover.jpg"));
    let _ = fs::remove_file(book_dir.join("cover.png"));
    let cover = solid_png(COVER_W, COVER_H, 30, 58, 95);
    fs::write(
        book_dir.join("Harbor Notes.epub"),
        epub_with_cover("harbor-notes", "Harbor Notes", &cover),
    )
    .unwrap();
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

fn epub_with_cover(id: &str, title: &str, cover: &[u8]) -> Vec<u8> {
    let opf = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="uid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="uid">{id}</dc:identifier>
    <dc:title>{title}</dc:title>
    <dc:language>en</dc:language>
    <meta name="cover" content="cover-img"/>
  </metadata>
  <manifest>
    <item id="cover-img" href="cover.png" media-type="image/png" properties="cover-image"/>
    <item id="chap" href="chapter.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="chap"/>
  </spine>
</package>"#
    );
    let chapter = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml"><body><p>{PRIDE_CHAPTER}</p></body></html>"#
    );
    let container = r#"<?xml version="1.0"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;
    zip_stored(&[
        ("mimetype", b"application/epub+zip"),
        ("META-INF/container.xml", container.as_bytes()),
        ("OEBPS/content.opf", opf.as_bytes()),
        ("OEBPS/cover.png", cover),
        ("OEBPS/chapter.xhtml", chapter.as_bytes()),
    ])
}

fn colored_pdf(r: f32, g: f32, b: f32) -> Vec<u8> {
    let content = format!("{r:.3} {g:.3} {b:.3} rg\n0 0 240 360 re f\n");
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Count 1 /Kids [3 0 R] >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 240 360] /Contents 4 0 R >>".to_string(),
        format!(
            "<< /Length {} >>\nstream\n{content}endstream",
            content.len()
        ),
    ];
    let mut body = String::from("%PDF-1.4\n");
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(body.len());
        body.push_str(&format!("{} 0 obj\n{object}\nendobj\n", index + 1));
    }
    let xref = body.len();
    body.push_str(&format!("xref\n0 {}\n", objects.len() + 1));
    body.push_str("0000000000 65535 f \n");
    for offset in offsets {
        body.push_str(&format!("{offset:010} 00000 n \n"));
    }
    body.push_str(&format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
        objects.len() + 1
    ));
    body.into_bytes()
}

fn solid_png(width: u32, height: u32, red: u8, green: u8, blue: u8) -> Vec<u8> {
    let mut row = Vec::with_capacity(1 + width as usize * 3);
    row.push(0);
    for _ in 0..width {
        row.extend_from_slice(&[red, green, blue]);
    }
    let mut raw = Vec::with_capacity(row.len() * height as usize);
    for _ in 0..height {
        raw.extend_from_slice(&row);
    }
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    let mut png = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    png.extend(png_chunk(b"IHDR", &ihdr));
    png.extend(png_chunk(b"IDAT", &zlib_store(&raw)));
    png.extend(png_chunk(b"IEND", &[]));
    png
}

fn png_chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut body = Vec::with_capacity(4 + data.len());
    body.extend_from_slice(kind);
    body.extend_from_slice(data);
    let crc = crc32(&body);
    let mut out = Vec::new();
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(&body);
    out.extend_from_slice(&crc.to_be_bytes());
    out
}

fn zlib_store(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    let mut index = 0;
    loop {
        let end = (index + 65535).min(data.len());
        let last = end == data.len();
        out.push(if last { 1 } else { 0 });
        let chunk = &data[index..end];
        let len = chunk.len() as u16;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(chunk);
        if last {
            break;
        }
        index = end;
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn zip_stored(files: &[(&str, &[u8])]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    for (name, data) in files {
        let offset = out.len() as u32;
        let crc = crc32(data);
        let name_bytes = name.as_bytes();
        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(name_bytes);
        out.extend_from_slice(data);
        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&(data.len() as u32).to_le_bytes());
        central.extend_from_slice(&(data.len() as u32).to_le_bytes());
        central.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u32.to_le_bytes());
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name_bytes);
    }
    let cd_start = out.len() as u32;
    out.extend_from_slice(&central);
    let cd_len = central.len() as u32;
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&cd_len.to_le_bytes());
    out.extend_from_slice(&cd_start.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;
    for &byte in data {
        a = (a + u32::from(byte)) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}
