use rusqlite::{Connection, Result};
use chrono::NaiveDate;

#[derive(Debug)]
struct Plant {
    id: u32,
    name: String,
    birthdate: NaiveDate
}

pub fn create_tables(conn: Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS plant (
            id    INTEGER PRIMARY KEY,
            name  TEXT NOT NULL,
            birthdate  TEXT
        )",
        (),
    )?;
    Ok(())
}
