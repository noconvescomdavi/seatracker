pub mod measurement;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DataValidity {
    #[default]
    Unknown,
    Invalid,
    Stale,
    Unavailable,
    Valid,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NavigationState {
    pub latitude: f64,
    pub longitude: f64,
    pub sog_knots: f32,
    pub cog_deg: f32,
    pub heading_deg: Option<f32>,
    pub fix_quality: Option<u8>,
    pub satellites: Option<u8>,
    pub hdop: Option<f32>,
    pub timestamp_ms: Option<u64>,
    pub validity: DataValidity,
}

impl NavigationState {
    pub fn is_usable(&self) -> bool {
        self.validity == DataValidity::Valid
            && self.latitude.is_finite()
            && self.longitude.is_finite()
            && (-90.0..=90.0).contains(&self.latitude)
            && (-180.0..=180.0).contains(&self.longitude)
    }

    pub fn mark_stale_if_older_than(&mut self, now_ms: u64, max_age_ms: u64) {
        if let Some(timestamp) = self.timestamp_ms {
            if now_ms.saturating_sub(timestamp) > max_age_ms && self.validity == DataValidity::Valid
            {
                self.validity = DataValidity::Stale;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OwnShipConfig {
    pub loa_m: f32,
    pub beam_m: f32,
    pub draft_m: f32,
    pub gps_offset_forward_m: f32,
    pub gps_offset_starboard_m: f32,
}

impl Default for OwnShipConfig {
    fn default() -> Self {
        Self {
            loa_m: 10.0,
            beam_m: 3.0,
            draft_m: 1.0,
            gps_offset_forward_m: 0.0,
            gps_offset_starboard_m: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_state_is_not_usable() {
        let mut state = NavigationState {
            latitude: -22.9,
            longitude: -43.2,
            timestamp_ms: Some(1000),
            validity: DataValidity::Valid,
            ..NavigationState::default()
        };
        state.mark_stale_if_older_than(5000, 3000);
        assert_eq!(state.validity, DataValidity::Stale);
        assert!(!state.is_usable());
    }
}
