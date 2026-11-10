use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

pub const BREAK_ALGO: &str = "kp-1";

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BreakKey {
    pub index: u32,
    pub width: i32,
    pub first: i32,
    pub font: String,
    pub indent: i32,
    pub lang: String,
    pub incoming: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CutKind {
    Glue,
    Hyphen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cut {
    pub at: u32,
    pub kind: CutKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParagraphPlan {
    pub text: String,
    pub cuts: Vec<Cut>,
    pub gap: Vec<i32>,
    pub outgoing: u8,
    pub native: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredParagraph {
    pub key: BreakKey,
    pub plan: ParagraphPlan,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PageStamp {
    Trim(String),
    Declared { width: i32, height: i32 },
}

impl PageStamp {
    pub fn resolve(trim_id: &str, declared: Option<(f64, f64)>) -> Self {
        if let Some((width, height)) = declared {
            if width > 0.0 && height > 0.0 {
                return PageStamp::Declared {
                    width: tenth(width),
                    height: tenth(height),
                };
            }
        }
        PageStamp::Trim(crate::reading::trim(trim_id).id.to_string())
    }

    pub fn token(&self) -> String {
        match self {
            PageStamp::Trim(id) => format!("trim:{id}"),
            PageStamp::Declared { width, height } => format!("fixed:{width}x{height}"),
        }
    }
}

pub fn tenth(px: f64) -> i32 {
    (px * 10.0).round() as i32
}

pub fn book_id(files: &HashMap<String, Vec<u8>>) -> String {
    let mut keys: Vec<&String> = files.keys().collect();
    keys.sort();
    let mut hasher = Sha256::new();
    for key in keys {
        let bytes = &files[key];
        hasher.update((key.len() as u64).to_le_bytes());
        hasher.update(key.as_bytes());
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    }
    hex_encode(&hasher.finalize())
}

pub struct BreakStore {
    dir: PathBuf,
    book: String,
    stamp: String,
    chapters: BTreeMap<String, BTreeMap<BreakKey, ParagraphPlan>>,
}

impl BreakStore {
    pub fn open(support_dir: &Path) -> Self {
        let dir = support_dir.join("breaks");
        let _ = fs::create_dir_all(&dir);
        BreakStore {
            dir,
            book: String::new(),
            stamp: String::new(),
            chapters: BTreeMap::new(),
        }
    }

    pub fn prepare(&mut self, book: &str, stamp: &PageStamp) {
        self.book = book.to_string();
        self.stamp = stamp.token();
        self.chapters.clear();
        let path = self.path();
        let tmp = self.tmp_path();
        let _ = fs::remove_file(&tmp);
        let Ok(bytes) = fs::read(&path) else {
            return;
        };
        match parse_book(&bytes, BREAK_ALGO, &self.stamp) {
            Some(chapters) => self.chapters = chapters,
            None => {
                let _ = fs::remove_file(&path);
            }
        }
    }

    pub fn slice_json(&self, href: &str) -> String {
        let records = self.chapters.get(href).map(records_json).unwrap_or_else(|| "[]".into());
        format!(
            "{{\"algo\":{},\"stamp\":{},\"records\":{records}}}",
            json_string(BREAK_ALGO),
            json_string(&self.stamp),
        )
    }

    pub fn harvest(&mut self, href: &str, algo: &str, stamp: &str, plans: Vec<StoredParagraph>) {
        if algo != BREAK_ALGO || stamp != self.stamp || self.book.is_empty() {
            return;
        }
        let chapter = self.chapters.entry(href.to_string()).or_default();
        for plan in plans {
            if plan.plan.outgoing > 2 || plan.key.incoming > 2 {
                continue;
            }
            if plan.plan.gap.len() != plan.plan.cuts.len() + 1 {
                continue;
            }
            if !cuts_increase(&plan.plan.cuts) {
                continue;
            }
            chapter.insert(plan.key, plan.plan);
        }
        self.write();
    }

    pub fn invalidate(&mut self) {
        if !self.book.is_empty() {
            let _ = fs::remove_file(self.path());
            let _ = fs::remove_file(self.tmp_path());
        }
        self.chapters.clear();
    }

    pub fn retarget(&mut self, stamp: &PageStamp) {
        self.stamp = stamp.token();
        self.invalidate();
    }

    fn write(&self) {
        if self.book.is_empty() {
            return;
        }
        let body = serialize_book(BREAK_ALGO, &self.stamp, &self.chapters);
        let tmp = self.tmp_path();
        let path = self.path();
        if fs::write(&tmp, &body).is_err() {
            return;
        }
        let _ = fs::rename(&tmp, &path);
    }

    fn path(&self) -> PathBuf {
        self.dir.join(format!("{}.json", self.book))
    }

    fn tmp_path(&self) -> PathBuf {
        self.dir.join(format!(".{}.json.tmp", self.book))
    }
}

fn cuts_increase(cuts: &[Cut]) -> bool {
    cuts.windows(2).all(|pair| pair[0].at < pair[1].at)
}

fn parse_book(
    bytes: &[u8],
    algo: &str,
    stamp: &str,
) -> Option<BTreeMap<String, BTreeMap<BreakKey, ParagraphPlan>>> {
    let value: serde_json::Value = serde_json::from_slice(bytes).ok()?;
    if value.get("algo").and_then(|item| item.as_str()) != Some(algo) {
        return None;
    }
    if value.get("stamp").and_then(|item| item.as_str()) != Some(stamp) {
        return None;
    }
    let chapters = value.get("chapters")?.as_object()?;
    let mut out = BTreeMap::new();
    for (href, records) in chapters {
        let Some(records) = records.as_array() else {
            continue;
        };
        let mut plans = BTreeMap::new();
        for record in records {
            if let Some(stored) = parse_record(record) {
                plans.insert(stored.key, stored.plan);
            }
        }
        out.insert(href.clone(), plans);
    }
    Some(out)
}

fn parse_record(value: &serde_json::Value) -> Option<StoredParagraph> {
    let incoming = value.get("in")?.as_u64()? as u8;
    let outgoing = value.get("out")?.as_u64()? as u8;
    if incoming > 2 || outgoing > 2 {
        return None;
    }
    let cuts = parse_cuts(value.get("cuts")?)?;
    let gap = parse_gap(value.get("gap")?)?;
    if gap.len() != cuts.len() + 1 || !cuts_increase(&cuts) {
        return None;
    }
    let key = BreakKey {
        index: value.get("i")?.as_u64()? as u32,
        width: value.get("w")?.as_i64()? as i32,
        first: value.get("first")?.as_i64()? as i32,
        font: value.get("font")?.as_str()?.to_string(),
        indent: value.get("indent")?.as_i64()? as i32,
        lang: value.get("lang")?.as_str()?.to_string(),
        incoming,
    };
    Some(StoredParagraph {
        key,
        plan: ParagraphPlan {
            text: value.get("text")?.as_str()?.to_string(),
            cuts,
            gap,
            outgoing,
            native: value.get("native").and_then(|item| item.as_bool()).unwrap_or(false),
        },
    })
}

fn parse_cuts(value: &serde_json::Value) -> Option<Vec<Cut>> {
    let items = value.as_array()?;
    let mut cuts = Vec::with_capacity(items.len());
    for item in items {
        let pair = item.as_array()?;
        let at = pair.first()?.as_u64()? as u32;
        let kind = match pair.get(1)?.as_str()? {
            "g" => CutKind::Glue,
            "h" => CutKind::Hyphen,
            _ => return None,
        };
        cuts.push(Cut { at, kind });
    }
    Some(cuts)
}

fn parse_gap(value: &serde_json::Value) -> Option<Vec<i32>> {
    let items = value.as_array()?;
    items
        .iter()
        .map(|item| item.as_i64().map(|n| n as i32))
        .collect()
}

fn records_json(plans: &BTreeMap<BreakKey, ParagraphPlan>) -> String {
    let mut records = Vec::new();
    for (key, plan) in plans {
        records.push(record_json(key, plan));
    }
    format!("[{}]", records.join(","))
}

fn record_json(key: &BreakKey, plan: &ParagraphPlan) -> String {
    let cuts = plan
        .cuts
        .iter()
        .map(|cut| {
            let kind = match cut.kind {
                CutKind::Glue => "g",
                CutKind::Hyphen => "h",
            };
            format!("[{},{}]", cut.at, json_string(kind))
        })
        .collect::<Vec<_>>()
        .join(",");
    let gap = plan
        .gap
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"i\":{},\"w\":{},\"first\":{},\"font\":{},\"indent\":{},\"lang\":{},\"in\":{},\"text\":{},\"out\":{}{flag},\"cuts\":[{cuts}],\"gap\":[{gap}]}}",
        key.index,
        key.width,
        key.first,
        json_string(&key.font),
        key.indent,
        json_string(&key.lang),
        key.incoming,
        json_string(&plan.text),
        plan.outgoing,
        flag = if plan.native { ",\"native\":true" } else { "" },
    )
}

fn serialize_book(
    algo: &str,
    stamp: &str,
    chapters: &BTreeMap<String, BTreeMap<BreakKey, ParagraphPlan>>,
) -> String {
    let mut body = Vec::new();
    for (href, plans) in chapters {
        body.push(format!("{}:{}", json_string(href), records_json(plans)));
    }
    format!(
        "{{\"algo\":{},\"stamp\":{},\"chapters\":{{{}}}}}\n",
        json_string(algo),
        json_string(stamp),
        body.join(",")
    )
}

fn json_string(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_else(|_| "\"\"".into())
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0xf) as usize] as char);
    }
    out
}

pub fn plans_from_json(value: &serde_json::Value) -> Vec<StoredParagraph> {
    let Some(records) = value.get("plans").and_then(|item| item.as_array()) else {
        return Vec::new();
    };
    records.iter().filter_map(parse_record).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(index: u32, incoming: u8) -> StoredParagraph {
        StoredParagraph {
            key: BreakKey {
                index,
                width: 3838,
                first: 3400,
                font: "22px \"Iowan Old Style\"".into(),
                indent: 440,
                lang: "en".into(),
                incoming,
            },
            plan: ParagraphPlan {
                text: "ab12".into(),
                cuts: vec![Cut { at: 12, kind: CutKind::Glue }],
                gap: vec![120, 0],
                outgoing: 0,
                native: false,
            },
        }
    }

    fn temp_store(name: &str) -> (PathBuf, BreakStore) {
        let dir = std::env::temp_dir().join(format!(
            "superreader-breaks-{}-{}-{}",
            std::process::id(),
            name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let store = BreakStore::open(&dir);
        (dir, store)
    }

    #[test]
    fn harvest_round_trips_and_a_second_write_matches() {
        let (dir, mut store) = temp_store("round");
        let stamp = PageStamp::Trim("paperback".into());
        store.prepare("book", &stamp);
        store.harvest("/c1", BREAK_ALGO, &stamp.token(), vec![sample(0, 0), sample(1, 0)]);
        let first = fs::read(dir.join("breaks/book.json")).unwrap();
        store.harvest("/c1", BREAK_ALGO, &stamp.token(), vec![sample(0, 0)]);
        let second = fs::read(dir.join("breaks/book.json")).unwrap();
        assert_eq!(first, second);
        let mut again = BreakStore::open(&dir);
        again.prepare("book", &stamp);
        assert_eq!(again.chapters["/c1"].len(), 2);
        assert!(again.slice_json("/c1").contains("\"first\":3400"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_stamp_mismatch_drops_the_file_and_a_bad_row_is_skipped() {
        let (dir, mut store) = temp_store("stamp");
        let stamp = PageStamp::Trim("paperback".into());
        store.prepare("book", &stamp);
        let mut bad = sample(3, 0);
        bad.plan.gap = vec![1];
        store.harvest("/c1", BREAK_ALGO, &stamp.token(), vec![bad, sample(1, 0)]);
        assert_eq!(store.chapters["/c1"].len(), 1);
        store.harvest("/c1", "kp-0", &stamp.token(), vec![sample(4, 0)]);
        assert!(!store.chapters["/c1"].contains_key(&sample(4, 0).key));
        drop(store);
        let mut next = BreakStore::open(&dir);
        next.prepare("book", &PageStamp::Trim("trade".into()));
        assert!(next.chapters.is_empty());
        assert!(!dir.join("breaks/book.json").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn retarget_keeps_later_plans_and_drops_the_old_stamp() {
        let (dir, mut store) = temp_store("retarget");
        let paperback = PageStamp::Trim("paperback".into());
        store.prepare("book", &paperback);
        store.harvest("/c1", BREAK_ALGO, &paperback.token(), vec![sample(0, 0)]);
        let trade = PageStamp::Trim("trade".into());
        store.retarget(&trade);
        assert!(store.chapters.is_empty());
        store.harvest("/c1", BREAK_ALGO, &paperback.token(), vec![sample(1, 0)]);
        assert!(store.chapters.is_empty());
        store.harvest("/c1", BREAK_ALGO, &trade.token(), vec![sample(2, 0)]);
        assert_eq!(store.chapters["/c1"].len(), 1);
        let mut again = BreakStore::open(&dir);
        again.prepare("book", &trade);
        assert!(again.slice_json("/c1").contains("\"i\":2"));
        let mut old = BreakStore::open(&dir);
        old.prepare("book", &paperback);
        assert!(old.chapters.is_empty());
        assert!(!dir.join("breaks/book.json").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn invalidate_removes_the_file_and_a_second_call_stays_empty() {
        let (dir, mut store) = temp_store("drop");
        let stamp = PageStamp::resolve("nope", None);
        assert_eq!(stamp.token(), "trim:paperback");
        store.prepare("book", &stamp);
        store.harvest("/c1", BREAK_ALGO, &stamp.token(), vec![sample(0, 0)]);
        store.invalidate();
        store.invalidate();
        assert!(store.chapters.is_empty());
        assert!(!dir.join("breaks/book.json").exists());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn book_id_changes_when_a_byte_changes_and_ignores_map_order() {
        let mut files = HashMap::new();
        files.insert("/b".into(), b"beta".to_vec());
        files.insert("/a".into(), b"alpha".to_vec());
        let id = book_id(&files);
        let mut swapped = HashMap::new();
        swapped.insert("/a".into(), b"alpha".to_vec());
        swapped.insert("/b".into(), b"beta".to_vec());
        assert_eq!(id, book_id(&swapped));
        swapped.insert("/a".into(), b"alphb".to_vec());
        assert_ne!(id, book_id(&swapped));
        assert_eq!(PageStamp::resolve("paperback", Some((100.0, 200.0))).token(), "fixed:1000x2000");
    }
}
