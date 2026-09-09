use rusqlite::{Connection, OpenFlags};
use seatracker_charts::{
    BoundingBox, ChartProvider, ProviderCapabilities, RasterLayer, SeaChartModel,
};
use std::collections::{BTreeMap, VecDeque};
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

    pub fn zoom_range(&self) -> (Option<u8>, Option<u8>) {
        (
            self.get("minzoom").and_then(|v| v.parse().ok()),
            self.get("maxzoom").and_then(|v| v.parse().ok()),
        )
    }

    pub fn bounds(&self) -> Option<BoundingBox> {
        let values: Vec<f64> = self
            .get("bounds")?
            .split(',')
            .filter_map(|v| v.trim().parse().ok())
            .collect();
        if values.len() != 4 {
            return None;
        }
        Some(BoundingBox {
            min_lon: values[0],
            min_lat: values[1],
            max_lon: values[2],
            max_lat: values[3],
        })
    }
}

#[derive(Debug)]
pub struct TileCache {
    capacity_bytes: usize,
    used_bytes: usize,
    items: VecDeque<((u32, u32, u32), Vec<u8>)>,
}

impl TileCache {
    pub fn new(capacity_bytes: usize) -> Self {
        Self {
            capacity_bytes,
            used_bytes: 0,
            items: VecDeque::new(),
        }
    }

    pub fn insert(&mut self, key: (u32, u32, u32), data: Vec<u8>) {
        if data.len() > self.capacity_bytes {
            return;
        }
        if let Some(index) = self.items.iter().position(|(item_key, _)| *item_key == key)
            && let Some((_, old)) = self.items.remove(index)
        {
            self.used_bytes = self.used_bytes.saturating_sub(old.len());
        }
        while self.used_bytes + data.len() > self.capacity_bytes {
            let Some((_, old)) = self.items.pop_front() else {
                break;
            };
            self.used_bytes = self.used_bytes.saturating_sub(old.len());
        }
        self.used_bytes += data.len();
        self.items.push_back((key, data));
    }

    pub fn get(&mut self, key: (u32, u32, u32)) -> Option<Vec<u8>> {
        let index = self
            .items
            .iter()
            .position(|(item_key, _)| *item_key == key)?;
        let item = self.items.remove(index)?;
        let data = item.1.clone();
        self.items.push_back(item);
        Some(data)
    }

    pub fn used_bytes(&self) -> usize {
        self.used_bytes
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
        let mut stmt = conn
            .prepare(
                "SELECT tile_data FROM tiles WHERE zoom_level=?1 AND tile_column=?2 AND tile_row=?3 LIMIT 1",
            )
            .map_err(|e| e.to_string())?;
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

    pub fn to_model(source: &str, metadata: &MbTilesMetadata) -> SeaChartModel {
        let (min_zoom, max_zoom) = metadata.zoom_range();
        let format = metadata.get("format").unwrap_or("unknown");
        SeaChartModel {
            chart_id: metadata.get("name").unwrap_or(source).to_string(),
            provider: "MBTilesProvider".into(),
            provider_version: "0.2.0".into(),
            source: source.to_string(),
            bounds: metadata.bounds(),
            raster_layers: vec![RasterLayer {
                id: "mbtiles-raster".into(),
                source: source.to_string(),
                bounds: metadata.bounds(),
                width_px: None,
                height_px: None,
                min_zoom,
                max_zoom,
                tile_size: Some(256),
                mime_type: match format {
                    "png" => Some("image/png".into()),
                    "jpg" | "jpeg" => Some("image/jpeg".into()),
                    "webp" => Some("image/webp".into()),
                    "pbf" => Some("application/x-protobuf".into()),
                    _ => None,
                },
            }],
            metadata: metadata.values.clone(),
            ..SeaChartModel::default()
        }
    }
}

impl ChartProvider for MbTilesProvider {
    fn provider_name(&self) -> &'static str {
        "MBTilesProvider"
    }
    fn provider_version(&self) -> &'static str {
        "0.2.0"
    }
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            raster: true,
            vector: true,
            updates: false,
            protected_content: false,
            object_info: false,
        }
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
    fn reads_metadata_tile_and_builds_model() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE metadata (name TEXT, value TEXT);
             CREATE TABLE tiles (zoom_level INTEGER, tile_column INTEGER, tile_row INTEGER, tile_data BLOB);
             INSERT INTO metadata VALUES
               ('name','Demo'),('format','png'),('minzoom','1'),('maxzoom','10'),('bounds','-44,-24,-42,-22');
             INSERT INTO tiles VALUES (1,0,1,x'010203');",
        )
        .unwrap();

        let meta = MbTilesProvider::read_metadata(&conn).unwrap();
        assert_eq!(meta.get("name"), Some("Demo"));
        assert!(meta.bounds().is_some());
        assert_eq!(
            MbTilesProvider::read_tile(&conn, 1, 0, 1).unwrap(),
            Some(vec![1, 2, 3])
        );
        assert_eq!(
            MbTilesProvider::to_model("demo.mbtiles", &meta).chart_id,
            "Demo"
        );
    }

    #[test]
    fn bounded_cache_evicts() {
        let mut cache = TileCache::new(5);
        cache.insert((1, 0, 0), vec![1, 2, 3]);
        cache.insert((1, 0, 1), vec![4, 5, 6]);
        assert!(cache.used_bytes() <= 5);
        assert!(cache.get((1, 0, 1)).is_some());
    }
}
