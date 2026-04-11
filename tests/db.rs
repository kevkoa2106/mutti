use mutti::db::{init, insert_tracks, query_tracks};
use rusqlite::Connection;
use std::path::PathBuf;

fn fresh_db() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory db");
    init(&conn);
    conn
}

#[test]
fn init_creates_tracks_table() {
    let conn = fresh_db();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='tracks'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn init_is_idempotent() {
    let conn = fresh_db();
    // Calling init again should not error or drop existing data.
    init(&conn);
    init(&conn);
    let tracks = query_tracks(&conn);
    assert!(tracks.is_empty());
}

#[test]
fn insert_tracks_falls_back_to_filename_and_unknown_artist() {
    let conn = fresh_db();
    // A path that doesn't exist: tag probe fails → title = file stem,
    // artist = "Unknown".
    let paths = vec![PathBuf::from("/tmp/does-not-exist/hello world.mp3")];
    insert_tracks(&conn, &paths);

    let tracks = query_tracks(&conn);
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].title, "hello world");
    assert_eq!(tracks[0].artist, "Unknown");
    assert_eq!(tracks[0].path, "/tmp/does-not-exist/hello world.mp3");
}

#[test]
fn insert_tracks_dedupes_on_path() {
    let conn = fresh_db();
    let paths = vec![
        PathBuf::from("/tmp/x/one.mp3"),
        PathBuf::from("/tmp/x/one.mp3"),
        PathBuf::from("/tmp/x/two.mp3"),
    ];
    insert_tracks(&conn, &paths);
    // Re-insert the same list a second time; INSERT OR IGNORE should skip.
    insert_tracks(&conn, &paths);

    let tracks = query_tracks(&conn);
    assert_eq!(tracks.len(), 2);
}

#[test]
fn query_tracks_returns_rows_sorted_by_title() {
    let conn = fresh_db();
    let paths = vec![
        PathBuf::from("/tmp/x/charlie.mp3"),
        PathBuf::from("/tmp/x/alpha.mp3"),
        PathBuf::from("/tmp/x/bravo.mp3"),
    ];
    insert_tracks(&conn, &paths);

    let tracks = query_tracks(&conn);
    let titles: Vec<&str> = tracks.iter().map(|t| t.title.as_str()).collect();
    assert_eq!(titles, vec!["alpha", "bravo", "charlie"]);
}

#[test]
fn query_tracks_empty_db_returns_empty_vec() {
    let conn = fresh_db();
    assert!(query_tracks(&conn).is_empty());
}

#[test]
fn insert_tracks_accepts_empty_slice() {
    let conn = fresh_db();
    let paths: Vec<PathBuf> = Vec::new();
    insert_tracks(&conn, &paths);
    assert!(query_tracks(&conn).is_empty());
}
