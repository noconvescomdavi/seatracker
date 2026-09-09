use seatracker_geographic::{destination_point, nm_to_meters};
use seatracker_gps::PositionFix;
use seatracker_navigation::DataValidity;

#[derive(Debug, Clone)]
pub struct NavigationSimulator {
    pub latitude: f64,
    pub longitude: f64,
    pub cog_deg: f64,
    pub sog_knots: f64,
    pub heading_deg: f64,
    pub timestamp_ms: u64,
}

impl NavigationSimulator {
    pub fn step(&mut self, delta_ms: u64) -> PositionFix {
        let hours = delta_ms as f64 / 3_600_000.0;
        let distance_nm = self.sog_knots * hours;
        let (lat, lon) = destination_point(
            self.latitude,
            self.longitude,
            self.cog_deg,
            nm_to_meters(distance_nm),
        );
        self.latitude = lat;
        self.longitude = lon;
        self.timestamp_ms = self.timestamp_ms.saturating_add(delta_ms);
        PositionFix {
            latitude: lat,
            longitude: lon,
            altitude_m: None,
            sog_knots: Some(self.sog_knots as f32),
            cog_deg: Some(self.cog_deg as f32),
            heading_deg: Some(self.heading_deg as f32),
            hdop: Some(0.8),
            satellites: Some(12),
            timestamp_ms: self.timestamp_ms,
            received_at_ms: self.timestamp_ms,
            validity: DataValidity::Valid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advances_ship_position() {
        let mut sim = NavigationSimulator {
            latitude: 0.0,
            longitude: 0.0,
            cog_deg: 90.0,
            sog_knots: 10.0,
            heading_deg: 90.0,
            timestamp_ms: 0,
        };
        let fix = sim.step(3_600_000);
        assert!((fix.longitude - 0.1665).abs() < 0.01);
    }
}
