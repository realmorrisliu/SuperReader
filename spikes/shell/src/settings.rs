use std::path::{Path, PathBuf};

use crate::reading;

pub fn path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join("Library/Application Support/SuperReader/settings.json")
}

pub fn load(path: &Path) -> String {
    let Ok(text) = std::fs::read_to_string(path) else {
        return reading::DEFAULT_TRIM.to_string();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return reading::DEFAULT_TRIM.to_string();
    };
    let id = value.get("trim").and_then(|item| item.as_str()).unwrap_or("");
    reading::trim(id).id.to_string()
}

pub fn store(path: &Path, id: &str) -> Result<(), String> {
    let id = reading::trim(id).id;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|err| err.to_string())?;
    }
    let body = format!("{{\"trim\":\"{id}\"}}\n");
    std::fs::write(path, body).map_err(|err| err.to_string())
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
        let _ = std::fs::remove_dir_all(&dir);
    }
}
