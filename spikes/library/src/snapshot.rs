use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rusqlite::Connection;

use crate::Notice;

static COPY_SERIAL: AtomicU64 = AtomicU64::new(0);

pub(crate) struct Snapshot {
    conn: Connection,
    dir: PathBuf,
}

impl Snapshot {
    pub(crate) fn connection(&self) -> &Connection {
        &self.conn
    }
}

impl Drop for Snapshot {
    fn drop(&mut self) {
        drop(std::mem::replace(
            &mut self.conn,
            Connection::open_in_memory().expect("in-memory sqlite"),
        ));
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn prune_old_copies(support: &Path) {
    let base = support.join("library-copies");
    let Ok(entries) = std::fs::read_dir(&base) else {
        return;
    };
    for entry in entries.flatten() {
        let _ = std::fs::remove_dir_all(entry.path());
    }
}

pub(crate) fn snapshot(src: &Path, support: &Path) -> Result<Snapshot, Notice> {
    if !src.is_file() {
        return Err(Notice::CopyFailed);
    }
    prune_old_copies(support);
    let serial = COPY_SERIAL.fetch_add(1, Ordering::Relaxed);
    let dir = support
        .join("library-copies")
        .join(format!("{}-{serial}", std::process::id()));
    if std::fs::create_dir_all(&dir).is_err() {
        return Err(Notice::CopyFailed);
    }
    let copy_failed = |dir: &Path| {
        let _ = std::fs::remove_dir_all(dir);
        Err(Notice::CopyFailed)
    };
    let dest = dir.join(
        src.file_name()
            .map(|name| name.to_owned())
            .unwrap_or_default(),
    );
    if std::fs::copy(src, &dest).is_err() {
        return copy_failed(&dir);
    }
    for suffix in ["-wal", "-shm"] {
        let sidecar = {
            let mut path = src.as_os_str().to_owned();
            path.push(suffix);
            PathBuf::from(path)
        };
        if sidecar.is_file()
            && std::fs::copy(
                &sidecar,
                dir.join(
                    sidecar
                        .file_name()
                        .map(|name| name.to_owned())
                        .unwrap_or_default(),
                ),
            )
            .is_err()
        {
            return copy_failed(&dir);
        }
    }
    let conn = match Connection::open_with_flags(&dest, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
    {
        Ok(conn) => conn,
        Err(_) => return copy_failed(&dir),
    };
    Ok(Snapshot { conn, dir })
}
