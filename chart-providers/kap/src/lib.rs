use seatracker_charts::ChartProvider;

#[derive(Debug, Default)]
pub struct KapProvider;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct KapMetadata {
    pub name: Option<String>,
    pub scale: Option<u32>,
    pub datum: Option<String>,
    pub projection: Option<String>,
}

impl KapProvider {
    pub fn parse_header(bytes: &[u8], max_header_bytes: usize) -> KapMetadata {
        let capped = &bytes[..bytes.len().min(max_header_bytes)];
        let text = String::from_utf8_lossy(capped);
        let mut out = KapMetadata::default();

        for line in text.lines() {
            if let Some(v) = line.strip_prefix("KNP/") {
                for field in v.split(',') {
                    if let Some(scale) = field.strip_prefix("SC=") {
                        out.scale = scale.trim().parse().ok();
                    } else if let Some(datum) = field.strip_prefix("GD=") {
                        out.datum = Some(datum.trim().to_string());
                    } else if let Some(proj) = field.strip_prefix("PR=") {
                        out.projection = Some(proj.trim().to_string());
                    }
                }
            } else if let Some(v) = line.strip_prefix("BSB/") {
                for field in v.split(',') {
                    if let Some(name) = field.strip_prefix("NA=") {
                        out.name = Some(name.trim().to_string());
                    }
                }
            }
            if line.as_bytes().contains(&0x1A) { break; }
        }
        out
    }
}

impl ChartProvider for KapProvider {
    fn provider_name(&self) -> &'static str { "KAPProvider" }
    fn can_open(&self, header: &[u8], extension: Option<&str>) -> bool {
        extension.map(|e| e.eq_ignore_ascii_case("kap")).unwrap_or(false)
            || header.windows(4).take(4096).any(|w| w == b"BSB/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_common_header_fields() {
        let h=b"BSB/NA=Test Chart,NU=1\r\nKNP/SC=50000,GD=WGS84,PR=MERCATOR\r\n";
        let m=KapProvider::parse_header(h, 4096);
        assert_eq!(m.name.as_deref(),Some("Test Chart"));
        assert_eq!(m.scale,Some(50000));
        assert_eq!(m.datum.as_deref(),Some("WGS84"));
    }
}
