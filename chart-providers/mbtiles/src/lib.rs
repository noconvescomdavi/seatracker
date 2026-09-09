use rusqlite::{Connection, OpenFlags};
use seatracker_charts::ChartProvider;
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Default)]
pub struct MbTilesProvider;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MbTilesMetadata {
    pub values: BTreeMap<String, String>,
}

impl MbTilesMetadata {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }
}

impl MbTilesProvider {
    pub const SQLITE_MAGIC: &'static [u8] = b"SQLite format 3\0";

    pub fn is_sqlite(header: &[u8]) -> bool {
        header.starts_with(Self::SQLITE_MAGIC)
    }

    pub fn open_read_only(path: impl AsRef<Path>) -> Result<Connection, String> {
        Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| e.to_string())
    }

    pub fn read_metadata(conn: &Connection) -> Result<MbTilesMetadata, String> {
        let mut stmt = conn
            .prepare("SELECT name, value FROM metadata ORDER BY name")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query([]).map_err(|e| e.to_string())?;
        let mut values = BTreeMap::new();
        while let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let name: String = row.get(0).map_err(|e| e.to_string())?;
            let value: String = row.get(1).map_err(|e| e.to_string())?;
            values.insert(name, value);
        }
        Ok(MbTilesMetadata { values })
    }

    pub fn read_tile(
        conn: &Connection,
        z: u32,
        x: u32,
        y_tms: u32,
    ) -> Result<Option<Vec<u8>>, String> {
        let mut stmt = conn.prepare(
            "SELECT tile_data FROM tiles WHERE zoom_level=?1 AND tile_column=?2 AND tile_row=?3 LIMIT 1"
        ).map_err(|e| e.to_string())?;
        let mut rows = stmt.query((z, x, y_tms)).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let data: Vec<u8> = row.get(0).map_err(|e| e.to_string())?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    pub fn xyz_to_tms_y(z: u32, y_xyz: u32) -> Option<u32> {
        let n = 1u64.checked_shl(z)?;
        let y = n.checked_sub(1)?.checked_sub(y_xyz as u64)?;
        u32::try_from(y).ok()
    }
}

impl ChartProvider for MbTilesProvider {
    fn provider_name(&self) -> &'static str {
        "MBTilesProvider"
    }

    fn can_open(&self, header: &[u8], extension: Option<&str>) -> bool {
        extension
            .map(|e| e.eq_ignore_ascii_case("mbtiles"))
            .unwrap_or(false)
            || Self::is_sqlite(header)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_sqlite_magic() {
        assert!(MbTilesProvider::is_sqlite(b"SQLite format 3\0rest"));
    }

    #[test]
    fn converts_xyz_to_tms() {
        assert_eq!(MbTilesProvider::xyz_to_tms_y(2, 0), Some(3));
        assert_eq!(MbTilesProvider::xyz_to_tms_y(2, 3), Some(0));
    }

    #[test]
    fn reads_metadata_and_tile() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE metadata (name TEXT, value TEXT);
             CREATE TABLE tiles (zoom_level INTEGER, tile_column INTEGER, tile_row INTEGER, tile_data BLOB);
             INSERT INTO metadata VALUES ('name','Demo'),('format','png');
             INSERT INTO tiles VALUES (1,0,1,x'010203');"
        ).unwrap();

        let meta = MbTilesProvider::read_metadata(&conn).unwrap();
        assert_eq!(meta.get("name"), Some("Demo"));
        assert_eq!(
            MbTilesProvider::read_tile(&conn, 1, 0, 1).unwrap(),
            Some(vec![1, 2, 3])
        );
    }
}
