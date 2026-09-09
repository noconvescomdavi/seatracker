use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChartFormat { S57, S63, Kap, MbTiles, Cm93, Nv2, Unknown }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartRecord {
    pub id: String,
    pub file_name: String,
    pub format: ChartFormat,
    pub byte_len: u64,
    pub sha256: String,
    pub min_lat: Option<f64>,
    pub min_lon: Option<f64>,
    pub max_lat: Option<f64>,
    pub max_lon: Option<f64>,
    pub scale: Option<u32>,
    pub edition: Option<String>,
}

pub fn detect_format(path: &str, head: &[u8]) -> ChartFormat {
    let ext = Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "000" => ChartFormat::S57,
        "kap" => ChartFormat::Kap,
        "mbtiles" => ChartFormat::MbTiles,
        "nv2" => ChartFormat::Nv2,
        "cm93" => ChartFormat::Cm93,
        _ if head.starts_with(b"SQLite format 3") => ChartFormat::MbTiles,
        _ => ChartFormat::Unknown
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

pub fn make_record(path: &str, bytes: &[u8]) -> ChartRecord {
    let name = Path::new(path).file_name().and_then(|n| n.to_str()).unwrap_or(path).to_string();
    let digest = sha256_hex(bytes);
    ChartRecord {
        id: digest.clone(),
        file_name: name,
        format: detect_format(path, &bytes[..bytes.len().min(64)]),
        byte_len: bytes.len() as u64,
        sha256: digest,
        min_lat: None, min_lon: None, max_lat: None, max_lon: None,
        scale: None, edition: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn detects_mbtiles_magic() {
        assert_eq!(detect_format("chart.bin", b"SQLite format 3\0anything"), ChartFormat::MbTiles);
    }
    #[test] fn detects_nv2_extension() {
        assert_eq!(detect_format("1U794XL.nv2", b"unknown"), ChartFormat::Nv2);
    }
}
