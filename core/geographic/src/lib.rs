pub const METERS_PER_NAUTICAL_MILE: f64 = 1852.0;

pub fn meters_to_nm(meters: f64) -> f64 {
    meters / METERS_PER_NAUTICAL_MILE
}
pub fn nm_to_meters(nm: f64) -> f64 {
    nm * METERS_PER_NAUTICAL_MILE
}

pub fn normalize_longitude(mut lon: f64) -> f64 {
    while lon > 180.0 {
        lon -= 360.0;
    }
    while lon < -180.0 {
        lon += 360.0;
    }
    lon
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nm_roundtrip() {
        assert!((meters_to_nm(nm_to_meters(12.3)) - 12.3).abs() < 1e-12);
    }
    #[test]
    fn longitude_wrap() {
        assert_eq!(normalize_longitude(190.0), -170.0);
    }
}
