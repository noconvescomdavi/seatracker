pub const METERS_PER_NAUTICAL_MILE: f64 = 1852.0;
pub const EARTH_RADIUS_M: f64 = 6_371_008.8;

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

pub fn normalize_bearing(mut bearing: f64) -> f64 {
    bearing %= 360.0;
    if bearing < 0.0 {
        bearing += 360.0;
    }
    bearing
}

pub fn great_circle_distance_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let p1 = lat1.to_radians();
    let p2 = lat2.to_radians();
    let dp = (lat2 - lat1).to_radians();
    let dl = normalize_longitude(lon2 - lon1).to_radians();
    let a = (dp / 2.0).sin().powi(2) + p1.cos() * p2.cos() * (dl / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    EARTH_RADIUS_M * c
}

pub fn initial_bearing_deg(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let p1 = lat1.to_radians();
    let p2 = lat2.to_radians();
    let dl = normalize_longitude(lon2 - lon1).to_radians();
    let y = dl.sin() * p2.cos();
    let x = p1.cos() * p2.sin() - p1.sin() * p2.cos() * dl.cos();
    normalize_bearing(y.atan2(x).to_degrees())
}

pub fn final_bearing_deg(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    normalize_bearing(initial_bearing_deg(lat2, lon2, lat1, lon1) + 180.0)
}

pub fn destination_point(lat: f64, lon: f64, bearing_deg: f64, distance_m: f64) -> (f64, f64) {
    let phi1 = lat.to_radians();
    let lambda1 = lon.to_radians();
    let theta = bearing_deg.to_radians();
    let delta = distance_m / EARTH_RADIUS_M;
    let phi2 = (phi1.sin() * delta.cos() + phi1.cos() * delta.sin() * theta.cos()).asin();
    let lambda2 = lambda1
        + (theta.sin() * delta.sin() * phi1.cos()).atan2(delta.cos() - phi1.sin() * phi2.sin());
    (phi2.to_degrees(), normalize_longitude(lambda2.to_degrees()))
}

pub fn rhumb_distance_m(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let p1 = lat1.to_radians();
    let p2 = lat2.to_radians();
    let dp = p2 - p1;
    let mut dl = normalize_longitude(lon2 - lon1).to_radians();
    if dl.abs() > std::f64::consts::PI {
        dl = if dl > 0.0 {
            -(2.0 * std::f64::consts::PI - dl)
        } else {
            2.0 * std::f64::consts::PI + dl
        };
    }
    let dpsi = ((p2 / 2.0 + std::f64::consts::FRAC_PI_4).tan()
        / (p1 / 2.0 + std::f64::consts::FRAC_PI_4).tan())
    .ln();
    let q = if dpsi.abs() > 1e-12 {
        dp / dpsi
    } else {
        p1.cos()
    };
    (dp * dp + q * q * dl * dl).sqrt() * EARTH_RADIUS_M
}

pub fn rhumb_bearing_deg(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let p1 = lat1.to_radians();
    let p2 = lat2.to_radians();
    let dpsi = ((p2 / 2.0 + std::f64::consts::FRAC_PI_4).tan()
        / (p1 / 2.0 + std::f64::consts::FRAC_PI_4).tan())
    .ln();
    let mut dl = normalize_longitude(lon2 - lon1).to_radians();
    if dl.abs() > std::f64::consts::PI {
        dl = if dl > 0.0 {
            -(2.0 * std::f64::consts::PI - dl)
        } else {
            2.0 * std::f64::consts::PI + dl
        };
    }
    normalize_bearing(dl.atan2(dpsi).to_degrees())
}

pub fn cross_track_error_m(start: (f64, f64), end: (f64, f64), point: (f64, f64)) -> f64 {
    let d13 = great_circle_distance_m(start.0, start.1, point.0, point.1) / EARTH_RADIUS_M;
    let t13 = initial_bearing_deg(start.0, start.1, point.0, point.1).to_radians();
    let t12 = initial_bearing_deg(start.0, start.1, end.0, end.1).to_radians();
    (d13.sin() * (t13 - t12).sin()).asin() * EARTH_RADIUS_M
}

pub fn along_track_distance_m(start: (f64, f64), end: (f64, f64), point: (f64, f64)) -> f64 {
    let d13 = great_circle_distance_m(start.0, start.1, point.0, point.1) / EARTH_RADIUS_M;
    let xt = cross_track_error_m(start, end, point) / EARTH_RADIUS_M;
    (d13.cos() / xt.cos()).acos() * EARTH_RADIUS_M
}

pub fn ttg_seconds(distance_m: f64, speed_knots: f64) -> Option<f64> {
    if !distance_m.is_finite() || !speed_knots.is_finite() || speed_knots <= 0.0 {
        return None;
    }
    Some(distance_m / (speed_knots * METERS_PER_NAUTICAL_MILE / 3600.0))
}

pub fn required_speed_knots(distance_m: f64, available_seconds: f64) -> Option<f64> {
    if distance_m < 0.0 || available_seconds <= 0.0 {
        return None;
    }
    Some(meters_to_nm(distance_m) / (available_seconds / 3600.0))
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
    #[test]
    fn known_equator_distance() {
        let d = great_circle_distance_m(0.0, 0.0, 0.0, 1.0);
        assert!((meters_to_nm(d) - 60.04).abs() < 0.2);
    }
    #[test]
    fn destination_roundtrip() {
        let p = destination_point(0.0, 0.0, 90.0, nm_to_meters(60.0));
        assert!(p.0.abs() < 0.01);
        assert!((p.1 - 0.999).abs() < 0.02);
    }
    #[test]
    fn ttg() {
        assert_eq!(
            ttg_seconds(nm_to_meters(10.0), 10.0).map(|v| v.round()),
            Some(3600.0)
        );
    }
}
