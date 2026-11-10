use std::path::{Path, PathBuf};

use crate::reading;

pub fn path() -> PathBuf {
    support_dir().join("settings.json")
}

pub fn support_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("SUPERREADER_SUPPORT_DIR") {
        let dir = dir.trim();
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join("Library/Application Support/SuperReader")
}

fn read_value(path: &Path) -> serde_json::Value {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .filter(|value: &serde_json::Value| value.is_object())
        .unwrap_or_else(|| serde_json::json!({}))
}

fn write_value(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|err| err.to_string())?;
    }
    let body = serde_json::to_string(value).map_err(|err| err.to_string())?;
    std::fs::write(path, format!("{body}\n")).map_err(|err| err.to_string())
}

pub fn load(path: &Path) -> String {
    let value = read_value(path);
    let id = value
        .get("trim")
        .and_then(|item| item.as_str())
        .unwrap_or("");
    reading::trim(id).id.to_string()
}

pub fn bold_word_starts(path: &Path) -> bool {
    read_value(path)
        .get("boldWordStarts")
        .and_then(|item| item.as_bool())
        .unwrap_or(false)
}

pub fn store(path: &Path, id: &str) -> Result<(), String> {
    let id = reading::trim(id).id;
    let mut value = read_value(path);
    value["trim"] = serde_json::json!(id);
    write_value(path, &value)
}

pub fn store_bold(path: &Path, on: bool) -> Result<(), String> {
    let mut value = read_value(path);
    if value.get("trim").and_then(|item| item.as_str()).is_none() {
        value["trim"] = serde_json::json!(reading::DEFAULT_TRIM);
    }
    value["boldWordStarts"] = serde_json::json!(on);
    write_value(path, &value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_keeps_a_known_trim_and_rejects_unknown() {
        let dir = std::env::temp_dir().join(format!("superreader-settings-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("settings.json");
        assert_eq!(load(&file), "paperback");
        store(&file, "32kai").unwrap();
        assert_eq!(load(&file), "32kai");
        store(&file, "nope").unwrap();
        assert_eq!(load(&file), "paperback");
        assert!(!bold_word_starts(&file));
        store_bold(&file, true).unwrap();
        store(&file, "trade").unwrap();
        assert!(bold_word_starts(&file));
        assert_eq!(load(&file), "trade");
        store_bold(&file, false).unwrap();
        assert!(!bold_word_starts(&file));
        assert_eq!(load(&file), "trade");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn support_dir_follows_a_verify_override_and_ignores_a_blank_one() {
        let dir = std::env::temp_dir().join(format!("superreader-support-{}", std::process::id()));
        let previous = std::env::var("SUPERREADER_SUPPORT_DIR").ok();
        std::env::set_var("SUPERREADER_SUPPORT_DIR", &dir);
        assert_eq!(support_dir(), dir);
        std::env::set_var("SUPERREADER_SUPPORT_DIR", " ");
        assert!(support_dir().ends_with("SuperReader"));
        match previous {
            Some(value) => std::env::set_var("SUPERREADER_SUPPORT_DIR", value),
            None => std::env::remove_var("SUPERREADER_SUPPORT_DIR"),
        }
    }
}
