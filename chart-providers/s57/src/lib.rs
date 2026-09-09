use seatracker_charts::ChartProvider;

#[derive(Debug, Default)]
pub struct S57Provider;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S57Probe {
    pub looks_like_iso8211: bool,
    pub record_length: Option<usize>,
}

impl S57Provider {
    pub fn probe(bytes: &[u8]) -> S57Probe {
        if bytes.len() < 24 {
            return S57Probe {
                looks_like_iso8211: false,
                record_length: None,
            };
        }
        let len = std::str::from_utf8(&bytes[0..5])
            .ok()
            .and_then(|s| s.parse::<usize>().ok());
        let looks = len
            .map(|n| n >= 24 && n <= bytes.len().max(n))
            .unwrap_or(false)
            && bytes[5].is_ascii_alphanumeric();
        S57Probe {
            looks_like_iso8211: looks,
            record_length: len,
        }
    }
}

impl ChartProvider for S57Provider {
    fn provider_name(&self) -> &'static str {
        "S57Provider"
    }
    fn can_open(&self, header: &[u8], extension: Option<&str>) -> bool {
        extension
            .map(|e| e.eq_ignore_ascii_case("000"))
            .unwrap_or(false)
            || Self::probe(header).looks_like_iso8211
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recognizes_exchange_set_extension() {
        assert!(S57Provider.can_open(b"", Some("000")));
    }
    #[test]
    fn rejects_tiny_unknown_buffer() {
        assert!(!S57Provider.can_open(b"abc", None));
    }
}
