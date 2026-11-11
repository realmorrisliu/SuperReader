use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

use crate::books;

static OPEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn take_opens() -> Vec<String> {
    OPEN.lock()
        .map(|mut queued| std::mem::take(&mut *queued))
        .unwrap_or_default()
}

pub fn accept_path(path: &Path) {
    accept(&path.display().to_string());
}

fn queue(path: String) {
    let Ok(mut queued) = OPEN.lock() else {
        return;
    };
    if !queued.iter().any(|have| have == &path) {
        queued.push(path);
    }
}

fn accept(raw: &str) {
    let path = raw.trim().trim_end_matches('/');
    if books::parse_book(path).is_some() {
        trace(&format!("queued {path}"));
        queue(path.to_string());
    } else if !path.is_empty() {
        trace(&format!("rejected {path}"));
    }
}

fn trace(line: &str) {
    let dir = "/tmp/superreader-spike";
    let _ = std::fs::create_dir_all(dir);
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("{dir}/opens.log"))
    {
        let _ = writeln!(file, "{line}");
    }
}
