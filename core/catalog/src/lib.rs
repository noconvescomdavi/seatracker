use rusqlite::{Connection, OpenFlags, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChartFormat {
    S57,
    S63,
    Kap,
    MbTiles,
    Cm93,
    Nv2,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChartRecord {
    pub id: String,
    pub file_name: String,
    pub path: String,
    pub format: ChartFormat,
    pub byte_len: u64,
    pub sha256: String,
    pub min_lat: Option<f64>,
    pub min_lon: Option<f64>,
    pub max_lat: Option<f64>,
    pub max_lon: Option<f64>,
    pub scale: Option<u32>,
    pub edition: Option<String>,
    pub quality: Option<f32>,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChartAdmission {
    Accepted(ChartFormat),
    NeedsValidation(ChartFormat),
    Rejected(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Viewport {
    pub min_lat: f64,
    pub min_lon: f64,
    pub max_lat: f64,
    pub max_lon: f64,
    pub target_scale: Option<u32>,
}

impl ChartRecord {
    pub fn intersects(&self, viewport: &Viewport) -> bool {
        let (Some(min_lat), Some(min_lon), Some(max_lat), Some(max_lon)) =
            (self.min_lat, self.min_lon, self.max_lat, self.max_lon)
        else {
            return false;
        };
        min_lat <= viewport.max_lat
            && max_lat >= viewport.min_lat
            && min_lon <= viewport.max_lon
            && max_lon >= viewport.min_lon
    }
}

pub fn detect_format(path: &str, head: &[u8]) -> ChartFormat {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if head.starts_with(b"SQLite format 3\0") {
        return ChartFormat::MbTiles;
    }

    match ext.as_str() {
        "000" => ChartFormat::S57,
        "os63" => ChartFormat::S63,
        "kap" => ChartFormat::Kap,
        "mbtiles" => ChartFormat::MbTiles,
        "nv2" => ChartFormat::Nv2,
        "cm93" => ChartFormat::Cm93,
        _ => ChartFormat::Unknown,
    }
}

pub fn admit_chart(path: &str, head: &[u8]) -> ChartAdmission {
    let format = detect_format(path, head);
    match format {
        ChartFormat::MbTiles if !head.starts_with(b"SQLite format 3\0") => {
            ChartAdmission::Rejected(
                "MBTiles extension present but SQLite signature is missing".into(),
            )
        }
        ChartFormat::Kap => {
            let looks_like_bsb = head.windows(4).take(4096).any(|w| w == b"BSB/");
            if looks_like_bsb {
                ChartAdmission::Accepted(ChartFormat::Kap)
            } else {
                ChartAdmission::NeedsValidation(ChartFormat::Kap)
            }
        }
        ChartFormat::S57 => ChartAdmission::NeedsValidation(ChartFormat::S57),
        ChartFormat::S63 => ChartAdmission::NeedsValidation(ChartFormat::S63),
        ChartFormat::Nv2 => ChartAdmission::NeedsValidation(ChartFormat::Nv2),
        ChartFormat::Cm93 => ChartAdmission::NeedsValidation(ChartFormat::Cm93),
        ChartFormat::MbTiles => ChartAdmission::Accepted(ChartFormat::MbTiles),
        ChartFormat::Unknown => {
            ChartAdmission::Rejected("unsupported or unknown chart format".into())
        }
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

pub fn make_record(path: &str, bytes: &[u8]) -> ChartRecord {
    let name = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path)
        .to_string();
    let digest = sha256_hex(bytes);

    ChartRecord {
        id: digest.clone(),
        file_name: name,
        path: path.to_string(),
        format: detect_format(path, &bytes[..bytes.len().min(4096)]),
        byte_len: bytes.len() as u64,
        sha256: digest,
        min_lat: None,
        min_lon: None,
        max_lat: None,
        max_lon: None,
        scale: None,
        edition: None,
        quality: None,
        enabled: true,
    }
}

pub fn select_quilt<'a>(records: &'a [ChartRecord], viewport: &Viewport) -> Vec<&'a ChartRecord> {
    let mut candidates: Vec<_> = records
        .iter()
        .filter(|record| record.enabled && record.intersects(viewport))
        .collect();

    candidates.sort_by(|a, b| {
        let scale_score = |record: &ChartRecord| -> u64 {
            match (record.scale, viewport.target_scale) {
                (Some(scale), Some(target)) => scale.abs_diff(target) as u64,
                (Some(_), None) => 0,
                _ => u64::MAX / 2,
            }
        };
        scale_score(a)
            .cmp(&scale_score(b))
            .then_with(|| b.quality.partial_cmp(&a.quality).unwrap_or(Ordering::Equal))
            .then_with(|| a.file_name.cmp(&b.file_name))
    });
    candidates
}

pub struct ChartDatabase {
    conn: Connection,
}

impl ChartDatabase {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        let db = Self { conn };
        db.initialize()?;
        Ok(db)
    }

    pub fn open_read_only(path: impl AsRef<Path>) -> Result<Self, String> {
        let conn = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| e.to_string())?;
        Ok(Self { conn })
    }

    fn initialize(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                "PRAGMA journal_mode=WAL;
                 CREATE TABLE IF NOT EXISTS charts (
                   id TEXT PRIMARY KEY,
                   file_name TEXT NOT NULL,
                   path TEXT NOT NULL,
                   format TEXT NOT NULL,
                   byte_len INTEGER NOT NULL,
                   sha256 TEXT NOT NULL,
                   min_lat REAL,
                   min_lon REAL,
                   max_lat REAL,
                   max_lon REAL,
                   scale INTEGER,
                   edition TEXT,
                   quality REAL,
                   enabled INTEGER NOT NULL DEFAULT 1
                 );
                 CREATE INDEX IF NOT EXISTS idx_charts_bounds
                   ON charts(min_lat, max_lat, min_lon, max_lon);",
            )
            .map_err(|e| e.to_string())
    }

    pub fn upsert(&self, record: &ChartRecord) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO charts
                 (id,file_name,path,format,byte_len,sha256,min_lat,min_lon,max_lat,max_lon,scale,edition,quality,enabled)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14)
                 ON CONFLICT(id) DO UPDATE SET
                 file_name=excluded.file_name,path=excluded.path,format=excluded.format,
                 byte_len=excluded.byte_len,sha256=excluded.sha256,
                 min_lat=excluded.min_lat,min_lon=excluded.min_lon,max_lat=excluded.max_lat,max_lon=excluded.max_lon,
                 scale=excluded.scale,edition=excluded.edition,quality=excluded.quality,enabled=excluded.enabled",
                params![
                    record.id,
                    record.file_name,
                    record.path,
                    format!("{:?}", record.format),
                    record.byte_len as i64,
                    record.sha256,
                    record.min_lat,
                    record.min_lon,
                    record.max_lat,
                    record.max_lon,
                    record.scale.map(i64::from),
                    record.edition,
                    record.quality,
                    if record.enabled { 1 } else { 0 }
                ],
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    pub fn count(&self) -> Result<u64, String> {
        self.conn
            .query_row("SELECT COUNT(*) FROM charts", [], |row| {
                row.get::<_, i64>(0)
            })
            .map(|value| value.max(0) as u64)
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_mbtiles_magic() {
        assert_eq!(
            detect_format("chart.bin", b"SQLite format 3\0anything"),
            ChartFormat::MbTiles
        );
    }

    #[test]
    fn rejects_fake_mbtiles() {
        assert!(matches!(
            admit_chart("chart.mbtiles", b"not sqlite"),
            ChartAdmission::Rejected(_)
        ));
    }

    #[test]
    fn keeps_nv2_unvalidated() {
        assert_eq!(
            admit_chart("1U794XL.nv2", b"unknown"),
            ChartAdmission::NeedsValidation(ChartFormat::Nv2)
        );
    }

    #[test]
    fn quilt_prefers_nearest_scale() {
        let mut a = make_record("a.kap", b"BSB/");
        a.min_lat = Some(-10.0);
        a.max_lat = Some(0.0);
        a.min_lon = Some(-20.0);
        a.max_lon = Some(-10.0);
        a.scale = Some(50_000);

        let mut b = a.clone();
        b.id = "b".into();
        b.file_name = "b.kap".into();
        b.scale = Some(100_000);

        let selected = select_quilt(
            &[a, b],
            &Viewport {
                min_lat: -5.0,
                min_lon: -15.0,
                max_lat: -4.0,
                max_lon: -14.0,
                target_scale: Some(60_000),
            },
        );
        assert_eq!(selected[0].scale, Some(50_000));
    }

    #[test]
    fn sqlite_catalog_upsert() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE charts (
               id TEXT PRIMARY KEY,file_name TEXT NOT NULL,path TEXT NOT NULL,format TEXT NOT NULL,
               byte_len INTEGER NOT NULL,sha256 TEXT NOT NULL,min_lat REAL,min_lon REAL,max_lat REAL,max_lon REAL,
               scale INTEGER,edition TEXT,quality REAL,enabled INTEGER NOT NULL DEFAULT 1
             );",
        )
        .unwrap();
        let db = ChartDatabase { conn };
        let record = make_record("chart.kap", b"BSB/NA=Demo");
        db.upsert(&record).unwrap();
        assert_eq!(db.count().unwrap(), 1);
    }
}
