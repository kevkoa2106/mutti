use rusqlite::Connection;
use std::path::Path;

use lofty::file::TaggedFileExt;
use lofty::probe::Probe;
use lofty::tag::Accessor;

pub fn init(conn: &Connection) {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS tracks (
            id    INTEGER PRIMARY KEY,
            title  TEXT NOT NULL,
            artist TEXT NOT NULL,
            path   TEXT NOT NULL UNIQUE
        );",
    )
    .expect("Failed to create tracks table");
}

pub fn insert_tracks(conn: &Connection, paths: &[impl AsRef<Path>]) {
    let mut stmt = conn
        .prepare(
            "INSERT OR IGNORE INTO tracks (title, artist, path) VALUES (?1, ?2, ?3)",
        )
        .expect("Failed to prepare insert statement");

    for p in paths {
        let path = p.as_ref();
        let path_str = path.to_string_lossy();

        let (title, artist) = read_title_artist(path);
        stmt.execute((&title, &artist, &*path_str)).ok();
    }
}

pub struct Track {
    pub title: String,
    pub artist: String,
    pub path: String,
}

pub fn query_tracks(conn: &Connection) -> Vec<Track> {
    let mut stmt = conn
        .prepare("SELECT title, artist, path FROM tracks ORDER BY title")
        .expect("Failed to prepare query");

    stmt.query_map([], |row| {
        Ok(Track {
            title: row.get(0)?,
            artist: row.get(1)?,
            path: row.get(2)?,
        })
    })
    .expect("Failed to query tracks")
    .filter_map(|r| r.ok())
    .collect()
}

fn read_title_artist(path: &Path) -> (String, String) {
    let tagged = Probe::open(path).and_then(|p| p.read());
    let tag = tagged
        .as_ref()
        .ok()
        .and_then(|t| t.primary_tag().or(t.first_tag()));

    let title = tag
        .and_then(|t| t.title().map(|s| s.to_string()))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            path.file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string())
        });

    let artist = tag
        .and_then(|t| t.artist().map(|s| s.to_string()))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Unknown".to_string());

    (title, artist)
}
