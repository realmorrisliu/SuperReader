use spike_library::{Book, Catalog, OpenKind};

use crate::form::{CoverCard, LibraryCounts, LibraryIn, SelectId, SourceIn};
use crate::settings::ReadingRecord;

pub fn project(catalog: &Catalog, selected: SelectId, history: &[ReadingRecord]) -> LibraryIn {
    let ready: Vec<&Book> = catalog
        .books()
        .iter()
        .filter(|book| book.open_command().is_some())
        .collect();
    let counts = LibraryCounts {
        all: ready.len() as u32,
        reading: ready
            .iter()
            .filter(|book| record_for(book, history).is_some_and(|row| !finished(row)))
            .count() as u32,
        want: 0,
        done: ready
            .iter()
            .filter(|book| record_for(book, history).is_some_and(finished))
            .count() as u32,
        pdf: ready
            .iter()
            .filter(|book| {
                book.open_command()
                    .is_some_and(|command| command.kind() == OpenKind::Pdf)
            })
            .count() as u32,
        notes: 0,
    };
    let mut sources: Vec<SourceIn> = Vec::new();
    for book in catalog.books() {
        let label = book.source().label().to_string();
        let ready_book = book.open_command().is_some();
        if let Some(row) = sources.iter_mut().find(|row| row.label == label) {
            if ready_book {
                row.count += 1;
            }
        } else {
            sources.push(SourceIn {
                label,
                count: u32::from(ready_book),
            });
        }
    }
    let mut shown: Vec<&Book> = catalog
        .books()
        .iter()
        .filter(|book| shown_in(book, &selected, record_for(book, history)))
        .collect();
    shown.sort_by(|left, right| {
        let left_at = record_for(left, history)
            .map(|row| row.opened_at)
            .unwrap_or(0);
        let right_at = record_for(right, history)
            .map(|row| row.opened_at)
            .unwrap_or(0);
        right_at
            .cmp(&left_at)
            .then_with(|| left.title().cmp(right.title()))
    });
    let covers = shown
        .iter()
        .map(|book| card(book, history))
        .collect::<Vec<_>>();
    let continue_reading = covers
        .iter()
        .filter(|card| card.page.is_some())
        .cloned()
        .collect();
    let ready_shown = shown
        .iter()
        .filter(|book| book.open_command().is_some())
        .count() as u32;
    LibraryIn {
        empty: catalog.books().is_empty(),
        book_count: ready_shown,
        counts,
        sources,
        selected,
        continue_reading,
        covers,
        notice: notice_text(catalog),
    }
}

fn notice_text(catalog: &Catalog) -> Option<String> {
    match catalog.notice_label() {
        Some("No library") => Some("未找到 Apple Books 资料库".to_string()),
        Some(label) => Some(label.to_string()),
        None => None,
    }
}

fn shown_in(book: &Book, selected: &SelectId, record: Option<&ReadingRecord>) -> bool {
    if book.open_command().is_some() {
        return matches_selection(book, selected, record);
    }
    matches!(
        selected,
        SelectId::Source(label) if book.source().label() == label
    )
}

fn matches_selection(book: &Book, selected: &SelectId, record: Option<&ReadingRecord>) -> bool {
    match selected {
        SelectId::AllBooks => true,
        SelectId::Pdf => book
            .open_command()
            .is_some_and(|command| command.kind() == OpenKind::Pdf),
        SelectId::Reading => record.is_some_and(|row| !finished(row)),
        SelectId::Done => record.is_some_and(finished),
        SelectId::Source(label) => book.source().label() == label,
        SelectId::Want | SelectId::Notes | SelectId::AddSource | SelectId::Settings => false,
    }
}

fn finished(record: &ReadingRecord) -> bool {
    record
        .total
        .is_some_and(|total| total > 0 && record.page >= total)
}

fn record_for<'a>(book: &Book, history: &'a [ReadingRecord]) -> Option<&'a ReadingRecord> {
    let path = book_path(book)?;
    history.iter().find(|row| row.path == path)
}

fn book_path(book: &Book) -> Option<String> {
    Some(book.open_command()?.path().as_path().display().to_string())
}

fn card(book: &Book, history: &[ReadingRecord]) -> CoverCard {
    let record = record_for(book, history);
    CoverCard {
        title: book.title().to_string(),
        author: book.author().unwrap_or("").to_string(),
        cover: book.cover().map(|file| file.as_path().to_path_buf()),
        open_path: book_path(book),
        pdf: book
            .open_command()
            .is_some_and(|command| command.kind() == OpenKind::Pdf),
        percent: record.and_then(percent),
        page: record.map(|row| row.page),
        unavailable: book.unavailable_label().map(str::to_string),
    }
}

fn percent(record: &ReadingRecord) -> Option<u8> {
    let total = record.total?;
    if total == 0 || record.page == 0 {
        return None;
    }
    Some(((u64::from(record.page) * 100) / u64::from(total)).min(100) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::record_open;
    use spike_library::{fixture, open, Home, Scope};
    use std::path::PathBuf;

    fn scope(root: PathBuf, support: PathBuf) -> Scope {
        Scope {
            support,
            override_root: Some(root),
            remembered: None,
            home: Home::Closed,
        }
    }

    #[test]
    fn continue_reading_follows_last_opened_and_percent_needs_a_total() {
        let apple = std::env::temp_dir().join(format!("spike-shelf-apple-{}", std::process::id()));
        let calibre =
            std::env::temp_dir().join(format!("spike-shelf-calibre-{}", std::process::id()));
        let support =
            std::env::temp_dir().join(format!("spike-shelf-support-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&apple);
        let _ = std::fs::remove_dir_all(&calibre);
        let _ = std::fs::remove_dir_all(&support);
        fixture::write_apple(&apple);
        fixture::write_calibre(&calibre);
        let apple_catalog = open(scope(apple.clone(), support.clone()));
        let calibre_catalog = open(scope(calibre.clone(), support.clone()));
        let union = apple_catalog.admit(spike_library::SourceName::Calibre, calibre_catalog);
        let essay = union
            .books()
            .iter()
            .find(|book| book.title() == "Plain Essay")
            .unwrap();
        let guide = union
            .books()
            .iter()
            .find(|book| book.title() == "Field Guide")
            .unwrap();
        let settings = support.join("settings.json");
        record_open(
            &settings,
            &essay
                .open_command()
                .unwrap()
                .path()
                .as_path()
                .display()
                .to_string(),
            3,
            None,
            10,
        )
        .unwrap();
        record_open(
            &settings,
            &guide
                .open_command()
                .unwrap()
                .path()
                .as_path()
                .display()
                .to_string(),
            1,
            Some(1),
            20,
        )
        .unwrap();
        let history = crate::settings::load_history(&settings);
        let shelf = project(&union, SelectId::AllBooks, &history);
        assert_eq!(shelf.counts.all, 4);
        assert_eq!(shelf.counts.pdf, 1);
        assert_eq!(shelf.sources.len(), 2);
        assert_eq!(shelf.sources[0].label, "Apple Books");
        assert_eq!(shelf.sources[0].count, 3);
        assert_eq!(shelf.sources[1].label, "calibre");
        assert_eq!(shelf.continue_reading.len(), 2);
        assert_eq!(shelf.continue_reading[0].title, "Field Guide");
        assert_eq!(shelf.continue_reading[0].percent, Some(100));
        assert_eq!(shelf.continue_reading[1].title, "Plain Essay");
        assert_eq!(shelf.continue_reading[1].percent, None);
        assert_eq!(shelf.continue_reading[1].page, Some(3));
        let pdf = project(&union, SelectId::Pdf, &history);
        assert_eq!(pdf.book_count, pdf.counts.pdf);
        assert_eq!(pdf.covers.len(), 1);
        assert_eq!(pdf.covers[0].title, "Field Guide");
        let reading = project(&union, SelectId::Reading, &history);
        assert_eq!(reading.covers.len(), 1);
        assert_eq!(reading.covers[0].title, "Plain Essay");
        assert_eq!(reading.counts.reading, 1);
        let done = project(&union, SelectId::Done, &history);
        assert_eq!(done.covers.len(), 1);
        assert_eq!(done.covers[0].title, "Field Guide");
        let _ = std::fs::remove_dir_all(&apple);
        let _ = std::fs::remove_dir_all(&calibre);
        let _ = std::fs::remove_dir_all(&support);
    }

    #[test]
    fn a_store_only_apple_source_stays_listed_at_zero() {
        let dir = std::env::temp_dir().join(format!("spike-shelf-store-{}", std::process::id()));
        let support =
            std::env::temp_dir().join(format!("spike-shelf-store-support-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&support);
        fixture::write_store_only(&dir);
        let catalog = open(scope(dir.clone(), support.clone()));
        let shelf = project(&catalog, SelectId::Source("Apple Books".to_string()), &[]);
        assert_eq!(shelf.sources.len(), 1);
        assert_eq!(shelf.sources[0].label, "Apple Books");
        assert_eq!(shelf.sources[0].count, 0);
        assert_eq!(shelf.book_count, 0);
        assert_eq!(shelf.covers.len(), 1);
        assert_eq!(shelf.covers[0].title, "Store Sample");
        assert_eq!(shelf.covers[0].unavailable.as_deref(), Some("Store"));
        assert!(shelf.covers[0].open_path.is_none());
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&support);
    }
}
