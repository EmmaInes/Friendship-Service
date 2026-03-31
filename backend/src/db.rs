use rusqlite::Connection;
use std::path::Path;

mod embedded_migrations {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

pub fn init(db_path: &str) -> Connection {
    let needs_create = !Path::new(db_path).exists();
    let mut conn = Connection::open(db_path).expect("Failed to open database");

    if needs_create {
        tracing::info!("Creating new database at {}", db_path);
    }

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 5000;",
    )
    .expect("Failed to set PRAGMAs");

    embedded_migrations::migrations::runner()
        .run(&mut conn)
        .expect("Failed to run migrations");

    tracing::info!("Database ready");
    conn
}
