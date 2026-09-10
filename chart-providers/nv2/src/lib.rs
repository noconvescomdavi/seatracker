use seatracker_charts::ChartProvider;
use std::collections::BTreeSet;

#[derive(Debug, Default)]
pub struct Nv2Provider;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceLevel {
    Confirmed,
    Probable,
    Hypothesis,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nv2Observation {
    pub offset: usize,
    pub length: usize,
    pub evidence: EvidenceLevel,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Nv2Profile {
    pub file_size: usize,
    pub navionics_signature: bool,
    pub marine_echart_signature: bool,
    pub printable_strings: usize,
    pub chart_titles: Vec<String>,
    pub chart_ids: Vec<String>,
    pub entropy_first_64k: f64,
}

impl Nv2Provider {
    pub fn printable_strings(
        bytes: &[u8],
        min_len: usize,
        max_results: usize,
    ) -> Vec<Nv2Observation> {
        let mut out = Vec::new();
        let mut start = None;
        for (index, byte) in bytes.iter().copied().enumerate() {
            let printable = byte.is_ascii_graphic() || byte == b' ';
            match (start, printable) {
                (None, true) => start = Some(index),
                (Some(s), false) => {
                    if index.saturating_sub(s) >= min_len {
                        out.push(Nv2Observation {
                            offset: s,
                            length: index - s,
                            evidence: EvidenceLevel::Confirmed,
                            note: String::from_utf8_lossy(&bytes[s..index]).trim().to_string(),
                        });
                        if out.len() >= max_results {
                            break;
                        }
                    }
                    start = None;
                }
                _ => {}
            }
        }
        if let Some(s) = start
            && bytes.len().saturating_sub(s) >= min_len
            && out.len() < max_results
        {
            out.push(Nv2Observation {
                offset: s,
                length: bytes.len() - s,
                evidence: EvidenceLevel::Confirmed,
                note: String::from_utf8_lossy(&bytes[s..]).trim().to_string(),
            });
        }
        out
    }

    pub fn profile(bytes: &[u8]) -> Nv2Profile {
        let observations = Self::printable_strings(bytes, 6, 20_000);
        let mut titles = BTreeSet::new();
        let mut ids = BTreeSet::new();
        let mut navionics_signature = false;
        let mut marine_echart_signature = false;

        for observation in &observations {
            let value = observation.note.trim();
            let upper = value.to_ascii_uppercase();
            if upper.contains("NAVIONICS") {
                navionics_signature = true;
            }
            if upper.contains("MARINE E-CHART") {
                marine_echart_signature = true;
            }
            if looks_like_chart_id(value) {
                ids.insert(value.to_string());
            } else if looks_like_title(value) {
                titles.insert(value.to_string());
            }
        }

        Nv2Profile {
            file_size: bytes.len(),
            navionics_signature,
            marine_echart_signature,
            printable_strings: observations.len(),
            chart_titles: titles.into_iter().take(256).collect(),
            chart_ids: ids.into_iter().take(512).collect(),
            entropy_first_64k: shannon_entropy(&bytes[..bytes.len().min(65_536)]),
        }
    }

    pub fn looks_like_supported_plain_metadata(bytes: &[u8]) -> bool {
        let profile = Self::profile(&bytes[..bytes.len().min(2 * 1024 * 1024)]);
        profile.navionics_signature || profile.marine_echart_signature
    }
}

fn looks_like_chart_id(value: &str) -> bool {
    let trimmed = value.trim_matches(|c: char| !c.is_ascii_alphanumeric());
    if !(6..=14).contains(&trimmed.len()) {
        return false;
    }
    let alpha = trimmed.chars().filter(|c| c.is_ascii_alphabetic()).count();
    let digits = trimmed.chars().filter(|c| c.is_ascii_digit()).count();
    alpha >= 1 && digits >= 4 && alpha + digits == trimmed.len()
}

fn looks_like_title(value: &str) -> bool {
    let value = value.trim_matches(|c: char| !c.is_ascii_graphic() && c != ' ');
    if !(6..=80).contains(&value.len()) {
        return false;
    }
    let letters = value.chars().filter(|c| c.is_ascii_alphabetic()).count();
    let spaces = value.chars().filter(|c| *c == ' ').count();
    letters >= 4 && spaces >= 1 && !value.contains("http") && !value.contains('@')
}

fn shannon_entropy(bytes: &[u8]) -> f64 {
    if bytes.is_empty() {
        return 0.0;
    }
    let mut counts = [0usize; 256];
    for byte in bytes {
        counts[*byte as usize] += 1;
    }
    let len = bytes.len() as f64;
    counts
        .iter()
        .filter(|count| **count > 0)
        .map(|count| {
            let p = *count as f64 / len;
            -p * p.log2()
        })
        .sum()
}

impl ChartProvider for Nv2Provider {
    fn provider_name(&self) -> &'static str {
        "NV2Provider"
    }

    fn can_open(&self, _header: &[u8], extension: Option<&str>) -> bool {
        matches!(extension, Some(ext) if ext.eq_ignore_ascii_case("nv2"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_only_confirmed_strings() {
        let observations = Nv2Provider::printable_strings(b"\0TEST123\0xx\0", 4, 10);
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].evidence, EvidenceLevel::Confirmed);
    }

    #[test]
    fn profiles_known_plain_metadata_without_claiming_geometry_support() {
        let sample = b"\x50\x80\xfe\0Marine e-chart\0Navionics\0S4110778\0PANAMA NORTH\0";
        let profile = Nv2Provider::profile(sample);
        assert!(profile.navionics_signature);
        assert!(profile.marine_echart_signature);
        assert!(profile.chart_ids.iter().any(|id| id.contains("S4110778")));
        assert!(
            profile
                .chart_titles
                .iter()
                .any(|title| title.contains("PANAMA NORTH"))
        );
    }

    #[test]
    fn entropy_is_bounded() {
        let entropy = shannon_entropy(b"AAAAABBBBBCCCCC");
        assert!((0.0..=8.0).contains(&entropy));
    }
}
