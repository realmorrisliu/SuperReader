# Apple Books on macOS

This note records what the `spike-library` crate reads today. Paths come from the spike brief and need a pass on a live Mac library before we treat them as final.

## Library database

- Directory: `~/Library/Containers/com.apple.iBooksX/Data/Documents/BKLibrary/`
- File pattern: `BKLibrary-*.sqlite` (newest modification time wins)
- Table: `ZBKLIBRARYASSET`
- Columns this reader uses (our names, not Apple’s):
  - Asset id (`ZASSETID`)
  - Title (`ZTITLE`)
  - Author (`ZAUTHOR`)
  - File path (`ZPATH`)
  - Store id (`ZSTOREID`) — non-empty means FairPlay store book; the file is never opened
  - Content type (`ZCONTENTTYPE`) — see below

If `ZSTOREID` is missing from the table, the catalog is unreadable with zero books.

Content type `5` is a series container, not a book, and is omitted from the shelf. Type `6` is an audiobook hint when nothing on disk is a readable EPUB or PDF ([independent mapping from cider-cli `books.rs`](https://docs.rs/cider-cli/latest/src/cider/sources/books.rs.html), not an Apple document). An EPUB file, an exploded EPUB directory (`META-INF/container.xml`), or a PDF on disk wins over the integer ([Kevin Aleman’s model overview](https://kevinaleman.com/the-apple-books-data-model-1f8h/)).

## Book files

Typical roots (verify on hardware):

- `~/Library/Mobile Documents/iCloud~com~apple~iBooks/Documents/`
- `~/Library/Containers/com.apple.BKAgentService/Data/Documents/iBooks/Books/`

A missing file with a sibling `.{name}.icloud` placeholder, or a path ending in `.icloud`, is listed as not downloaded. The placeholder is not read ([iCloud file status](https://support.apple.com/guide/mac-help/check-icloud-drive-file-folder-status-mac-mchlc994344b/mac), [robotsprocket on EPUB repair](https://robotsprocket.com/fixing-epub-files-from-apple-books/)).

`SUPERREADER_LIBRARY`, when set, is the Apple Books root the import button reads. The shelf does not open it at launch. `SUPERREADER_CALIBRE` is the folder the calibre button reads. A path passed to the app (`open -a` or a dropped file) skips the shelf when it is an EPUB file, an exploded EPUB directory, or a PDF.

## Permissions

The shelf starts empty. Nothing reads `~/Library/Containers/com.apple.iBooksX` until the user presses **Import from Apple Books**. The snapshot is read once, then each book is classified and shown before the next one. The quiet result line appears after the last book. If that read is permission denied, the same button becomes **Choose Folder** and opens a folder panel in place. A chosen folder is stored as `libraryFolder` in `settings.json`. Listing the container without Full Disk Access returns permission denied ([apple-books-mcp](https://github.com/ragmha/apple-books-mcp)).

## Calibre second source

A folder whose root contains `metadata.db` is a Calibre library. Field names follow the [Calibre database API](https://manual.calibre-ebook.com/db_api.html); the SQL and schema are ours, not copied from calibre.

Default when Apple Books is absent: `~/Calibre Library`.

## Read-only snapshot

The crate copies `BKLibrary-*.sqlite` (and `-wal`/`-shm` when present) into `{support}/library-copies/{pid}-{n}/`, opens the copy read-only with rusqlite, and deletes the directory on drop. The original files are never written.

## Annotations (later slice)

Not read in this tracer. For a future import, Kevin Aleman lists `AEAnnotation` under `~/Library/Containers/com.apple.iBooksX/Data/Documents/AEAnnotation/`, table `ZAEANNOTATION`, joined on `ZANNOTATIONASSETID = ZASSETID` ([same article](https://kevinaleman.com/the-apple-books-data-model-1f8h/)). Columns named there include annotation type, location, note text, highlight style, and creation time. Treat that list as independent observation, not Apple documentation.

## DRM and backups

Store ids gate FairPlay content. Do not open those files ([forum notes on EPUB export](https://forums.zotero.org/discussion/118029/epub-book-import-from-apple-books-library), [Mavericks-era extraction write-up](https://www.simon.me.uk/2014/690_extracting-ebooks-from-apple-ibooks-for-mavericks/), [Stack Exchange backup discussion](https://apple.stackexchange.com/questions/163131/what-is-the-best-way-to-backup-epub-and-pdf-books-in-ibooks)).

## Still missing

Writing to either library, metadata editing, plugins, book sources, search, sorting, filters, annotation import, and opening `gpt3.pdf` from a library row without the shelf `open` command.
