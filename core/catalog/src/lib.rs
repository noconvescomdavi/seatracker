use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChartAdmission {
    Accepted(ChartFormat),
    NeedsValidation(ChartFormat),
    Rejected(String),
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
        ChartFormat::Nv2 => ChartAdmission::NeedsValidation(ChartFormat::Nv2),
        ChartFormat::Cm93 => ChartAdmission::NeedsValidation(ChartFormat::Cm93),
        ChartFormat::S63 => ChartAdmission::NeedsValidation(ChartFormat::S63),
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
        format: detect_format(path, &bytes[..bytes.len().min(4096)]),
        byte_len: bytes.len() as u64,
        sha256: digest,
        min_lat: None,
        min_lon: None,
        max_lat: None,
        max_lon: None,
        scale: None,
        edition: None,
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
}
