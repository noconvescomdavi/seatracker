use seatracker_charts::ChartProvider;

#[derive(Debug, Default)]
pub struct KapProvider;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct KapMetadata {
    pub name: Option<String>,
    pub scale: Option<u32>,
    pub datum: Option<String>,
    pub projection: Option<String>,
    pub width_px: Option<u32>,
    pub height_px: Option<u32>,
    pub references: Vec<KapReference>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KapReference {
    pub pixel_x: f64,
    pub pixel_y: f64,
    pub latitude: f64,
    pub longitude: f64,
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
                    } else if let Some(w) = field.strip_prefix("RA=") {
                        let mut p = w.split(',');
                        out.width_px = p.next().and_then(|x| x.trim().parse().ok());
                        out.height_px = p.next().and_then(|x| x.trim().parse().ok());
                    }
                }
            } else if let Some(v) = line.strip_prefix("REF/") {
                let parts: Vec<_> = v.split(',').collect();
                if parts.len() >= 5 {
                    let parsed = (
                        parts[1].trim().parse::<f64>(),
                        parts[2].trim().parse::<f64>(),
                        parts[3].trim().parse::<f64>(),
                        parts[4].trim().parse::<f64>(),
                    );
                    if let (Ok(pixel_x), Ok(pixel_y), Ok(latitude), Ok(longitude)) = parsed {
                        out.references.push(KapReference {
                            pixel_x,
                            pixel_y,
                            latitude,
                            longitude,
                        });
                    }
                }
            }
            if line.as_bytes().contains(&0x1A) {
                break;
            }
        }
        out
    }

    pub fn affine_pixel_to_geo(meta: &KapMetadata, x: f64, y: f64) -> Option<(f64, f64)> {
        if meta.references.len() < 3 {
            return None;
        }
        let a = meta.references[0];
        let b = meta.references[1];
        let c = meta.references[2];

        let det = (b.pixel_x - a.pixel_x) * (c.pixel_y - a.pixel_y)
            - (c.pixel_x - a.pixel_x) * (b.pixel_y - a.pixel_y);
        if det.abs() < 1e-12 {
            return None;
        }

        let u = ((x - a.pixel_x) * (c.pixel_y - a.pixel_y)
            - (c.pixel_x - a.pixel_x) * (y - a.pixel_y))
            / det;
        let v = ((b.pixel_x - a.pixel_x) * (y - a.pixel_y)
            - (x - a.pixel_x) * (b.pixel_y - a.pixel_y))
            / det;

        let lat = a.latitude + u * (b.latitude - a.latitude) + v * (c.latitude - a.latitude);
        let lon = a.longitude + u * (b.longitude - a.longitude) + v * (c.longitude - a.longitude);
        Some((lat, lon))
    }
}

impl ChartProvider for KapProvider {
    fn provider_name(&self) -> &'static str {
        "KAPProvider"
    }

    fn can_open(&self, header: &[u8], extension: Option<&str>) -> bool {
        extension
            .map(|e| e.eq_ignore_ascii_case("kap"))
            .unwrap_or(false)
            || header.windows(4).take(4096).any(|w| w == b"BSB/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_header_fields_and_refs() {
        let h=b"BSB/NA=Test Chart\r\nKNP/SC=50000,GD=WGS84,PR=MERCATOR\r\nREF/1,0,0,-22.0,-43.0\r\nREF/2,100,0,-22.0,-42.0\r\nREF/3,0,100,-23.0,-43.0\r\n";
        let m = KapProvider::parse_header(h, 4096);
        assert_eq!(m.name.as_deref(), Some("Test Chart"));
        assert_eq!(m.scale, Some(50000));
        assert_eq!(m.datum.as_deref(), Some("WGS84"));
        assert_eq!(m.references.len(), 3);
        let p = KapProvider::affine_pixel_to_geo(&m, 50.0, 50.0).unwrap();
        assert!((p.0 + 22.5).abs() < 1e-9);
        assert!((p.1 + 42.5).abs() < 1e-9);
    }
}
