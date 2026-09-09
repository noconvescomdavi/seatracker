use rusqlite::{params, Connection};
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;

pub trait KeyValueStore {
    fn put_bytes(&mut self, key: &str, value: &[u8]) -> Result<(), String>;
    fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>, String>;
    fn delete(&mut self, key: &str) -> Result<(), String>;
}

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             CREATE TABLE IF NOT EXISTS kv (
               key TEXT PRIMARY KEY,
               value BLOB NOT NULL,
               updated_at INTEGER NOT NULL DEFAULT (unixepoch() * 1000)
             );",
        )
        .map_err(|e| e.to_string())?;
        Ok(Self { conn })
    }

    pub fn in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
        conn.execute_batch(
            "CREATE TABLE kv (
               key TEXT PRIMARY KEY,
               value BLOB NOT NULL,
               updated_at INTEGER NOT NULL DEFAULT 0
             );",
        )
        .map_err(|e| e.to_string())?;
        Ok(Self { conn })
    }
}

impl KeyValueStore for SqliteStore {
    fn put_bytes(&mut self, key: &str, value: &[u8]) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO kv(key,value) VALUES (?1,?2)
                 ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=(unixepoch()*1000)",
                params![key, value],
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn get_bytes(&self, key: &str) -> Result<Option<Vec<u8>>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM kv WHERE key=?1 LIMIT 1")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query([key]).map_err(|e| e.to_string())?;
        let Some(row) = rows.next().map_err(|e| e.to_string())? else {
            return Ok(None);
        };
        row.get(0).map(Some).map_err(|e| e.to_string())
    }

    fn delete(&mut self, key: &str) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM kv WHERE key=?1", [key])
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

pub fn put_json<T: Serialize>(
    store: &mut dyn KeyValueStore,
    key: &str,
    value: &T,
) -> Result<(), String> {
    let data = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    store.put_bytes(key, &data)
}

pub fn get_json<T: DeserializeOwned>(
    store: &dyn KeyValueStore,
    key: &str,
) -> Result<Option<T>, String> {
    let Some(data) = store.get_bytes(key)? else {
        return Ok(None);
    };
    serde_json::from_slice(&data)
        .map(Some)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlite_store_roundtrip() {
        let mut store = SqliteStore::in_memory().unwrap();
        store.put_bytes("a", b"b").unwrap();
        assert_eq!(store.get_bytes("a").unwrap(), Some(b"b".to_vec()));
        store.delete("a").unwrap();
        assert_eq!(store.get_bytes("a").unwrap(), None);
    }
}
