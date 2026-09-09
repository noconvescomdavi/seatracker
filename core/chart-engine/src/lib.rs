use seatracker_catalog::{ChartAdmission, ChartFormat, admit_chart};
use seatracker_charts::ChartProvider;
use seatracker_provider_kap::{KapMetadata, KapProvider};
use seatracker_provider_mbtiles::MbTilesProvider;
use seatracker_provider_s57::{S57Probe, S57Provider};
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum ChartInspection {
    S57 {
        admission: ChartAdmission,
        probe: S57Probe,
    },
    Kap {
        admission: ChartAdmission,
        metadata: KapMetadata,
    },
    MbTiles {
        admission: ChartAdmission,
    },
    Unsupported {
        admission: ChartAdmission,
    },
}

fn extension(path: &str) -> Option<&str> {
    Path::new(path).extension().and_then(|v| v.to_str())
}

pub fn inspect_chart(path: &str, bytes: &[u8]) -> ChartInspection {
    let head = &bytes[..bytes.len().min(64 * 1024)];
    let admission = admit_chart(path, head);
    let ext = extension(path);

    let s57 = S57Provider;
    if s57.can_open(head, ext) {
        return ChartInspection::S57 {
            admission,
            probe: S57Provider::probe(head),
        };
    }

    let kap = KapProvider;
    if kap.can_open(head, ext) {
        return ChartInspection::Kap {
            admission,
            metadata: KapProvider::parse_header(head, 64 * 1024),
        };
    }

    let mbtiles = MbTilesProvider;
    if mbtiles.can_open(head, ext) {
        return ChartInspection::MbTiles { admission };
    }

    ChartInspection::Unsupported { admission }
}

pub fn positively_identified_format(inspection: &ChartInspection) -> Option<ChartFormat> {
    match inspection {
        ChartInspection::S57 { probe, .. } if probe.looks_like_iso8211 => Some(ChartFormat::S57),
        ChartInspection::Kap { metadata, .. }
            if metadata.name.is_some() || !metadata.references.is_empty() =>
        {
            Some(ChartFormat::Kap)
        }
        ChartInspection::MbTiles {
            admission: ChartAdmission::Accepted(ChartFormat::MbTiles),
        } => Some(ChartFormat::MbTiles),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_kap_to_kap_provider() {
        let bytes = b"BSB/NA=Demo\r\nKNP/SC=10000,GD=WGS84,PR=MERCATOR\r\n";
        let inspection = inspect_chart("demo.kap", bytes);
        assert!(matches!(inspection, ChartInspection::Kap { .. }));
        assert_eq!(
            positively_identified_format(&inspection),
            Some(ChartFormat::Kap)
        );
    }

    #[test]
    fn fake_mbtiles_is_not_positive() {
        let inspection = inspect_chart("demo.mbtiles", b"not sqlite");
        assert_eq!(positively_identified_format(&inspection), None);
    }
}
