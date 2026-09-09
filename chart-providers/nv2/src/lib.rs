use seatracker_charts::ChartProvider;

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
                            note: String::from_utf8_lossy(&bytes[s..index]).to_string(),
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
        out
    }
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
}
