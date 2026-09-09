#[derive(Debug, Clone, PartialEq)]
pub enum AlarmKind {
    PositionLost,
    AisStale,
    Cpa,
    Tcpa,
    AnchorWatch,
    Speed,
    Course,
}

#[derive(Debug, Clone)]
pub struct AlarmConfig {
    pub cpa_nm: f64,
    pub tcpa_minutes: f64,
    pub ais_stale_ms: u64,
}

impl Default for AlarmConfig {
    fn default() -> Self {
        Self { cpa_nm: 1.0, tcpa_minutes: 15.0, ais_stale_ms: 180_000 }
    }
}

pub fn collision_alarm(cpa_nm: f64, tcpa_minutes: f64, cfg: &AlarmConfig) -> bool {
    tcpa_minutes >= 0.0 && tcpa_minutes <= cfg.tcpa_minutes && cpa_nm <= cfg.cpa_nm
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn collision_threshold() {
        assert!(collision_alarm(0.5, 10.0, &AlarmConfig::default()));
        assert!(!collision_alarm(2.0, 10.0, &AlarmConfig::default()));
    }
}
