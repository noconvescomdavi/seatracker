use seatracker_navigation::DataValidity;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionFix {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude_m: Option<f64>,
    pub sog_knots: Option<f32>,
    pub cog_deg: Option<f32>,
    pub heading_deg: Option<f32>,
    pub hdop: Option<f32>,
    pub satellites: Option<u8>,
    pub timestamp_ms: u64,
    pub received_at_ms: u64,
    pub validity: DataValidity,
}

impl PositionFix {
    pub fn is_fresh(&self, now_ms: u64, max_age_ms: u64) -> bool {
        self.validity == DataValidity::Valid
            && now_ms.saturating_sub(self.received_at_ms) <= max_age_ms
    }

    pub fn validated(mut self) -> Self {
        if !self.latitude.is_finite()
            || !self.longitude.is_finite()
            || !(-90.0..=90.0).contains(&self.latitude)
            || !(-180.0..=180.0).contains(&self.longitude)
        {
            self.validity = DataValidity::Invalid;
        }
        self
    }
}

pub trait PositionProvider {
    fn provider_name(&self) -> &'static str;
    fn latest_fix(&self) -> Option<PositionFix>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionSource {
    AndroidLocation,
    NmeaSerial,
    NmeaTcp,
    NmeaUdp,
    WindowsLocation,
    Replay,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_coordinates_are_rejected() {
        let fix = PositionFix {
            latitude: 100.0,
            longitude: 10.0,
            altitude_m: None,
            sog_knots: None,
            cog_deg: None,
            heading_deg: None,
            hdop: None,
            satellites: None,
            timestamp_ms: 0,
            received_at_ms: 0,
            validity: DataValidity::Valid,
        }
        .validated();
        assert_eq!(fix.validity, DataValidity::Invalid);
    }
}
