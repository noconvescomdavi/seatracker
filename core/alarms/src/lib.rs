use seatracker_ais::{AisTarget, cpa_tcpa_nm};
use seatracker_navigation::NavigationState;

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
    pub anchor_radius_nm: f64,
    pub max_speed_knots: Option<f64>,
    pub course_deviation_deg: Option<f64>,
}

impl Default for AlarmConfig {
    fn default() -> Self {
        Self {
            cpa_nm: 1.0,
            tcpa_minutes: 15.0,
            ais_stale_ms: 180_000,
            anchor_radius_nm: 0.05,
            max_speed_knots: None,
            course_deviation_deg: None,
        }
    }
}

pub fn collision_alarm(cpa_nm: f64, tcpa_minutes: f64, cfg: &AlarmConfig) -> bool {
    tcpa_minutes >= 0.0 && tcpa_minutes <= cfg.tcpa_minutes && cpa_nm <= cfg.cpa_nm
}

pub fn collision_alarm_for_target(
    own: &NavigationState,
    target: &AisTarget,
    now_ms: u64,
    cfg: &AlarmConfig,
) -> Option<bool> {
    if !own.is_usable() || target.is_stale(now_ms, cfg.ais_stale_ms) || !target.has_valid_motion() {
        return None;
    }

    let (target_lat, target_lon, target_sog, target_cog) = (
        target.latitude?,
        target.longitude?,
        target.sog_knots? as f64,
        target.cog_deg? as f64,
    );

    let (cpa_nm, tcpa_minutes) = cpa_tcpa_nm(
        own.latitude,
        own.longitude,
        own.sog_knots as f64,
        own.cog_deg as f64,
        target_lat,
        target_lon,
        target_sog,
        target_cog,
    )?;

    Some(collision_alarm(cpa_nm, tcpa_minutes, cfg))
}

pub fn anchor_watch_alarm(
    anchor_lat: f64,
    anchor_lon: f64,
    own: &NavigationState,
    cfg: &AlarmConfig,
) -> Option<bool> {
    if !own.is_usable() {
        return None;
    }
    let distance_nm = seatracker_navigation::measurement::haversine_nm(
        anchor_lat,
        anchor_lon,
        own.latitude,
        own.longitude,
    );
    Some(distance_nm > cfg.anchor_radius_nm)
}

pub fn speed_alarm(own: &NavigationState, cfg: &AlarmConfig) -> Option<bool> {
    if !own.is_usable() {
        return None;
    }
    let limit = cfg.max_speed_knots?;
    Some(own.sog_knots as f64 > limit)
}

pub fn course_alarm(
    own: &NavigationState,
    desired_course_deg: f64,
    cfg: &AlarmConfig,
) -> Option<bool> {
    if !own.is_usable() {
        return None;
    }
    let limit = cfg.course_deviation_deg?;
    let delta = ((own.cog_deg as f64 - desired_course_deg + 540.0) % 360.0) - 180.0;
    Some(delta.abs() > limit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use seatracker_navigation::DataValidity;

    #[test]
    fn collision_threshold() {
        assert!(collision_alarm(0.5, 10.0, &AlarmConfig::default()));
        assert!(!collision_alarm(2.0, 10.0, &AlarmConfig::default()));
    }

    #[test]
    fn anchor_watch_detects_departure() {
        let own = NavigationState {
            latitude: 0.0,
            longitude: 0.01,
            validity: DataValidity::Valid,
            ..NavigationState::default()
        };
        let mut cfg = AlarmConfig::default();
        cfg.anchor_radius_nm = 0.2;
        assert_eq!(anchor_watch_alarm(0.0, 0.0, &own, &cfg), Some(true));
    }

    #[test]
    fn stale_target_never_generates_collision_alarm() {
        let own = NavigationState {
            latitude: 0.0,
            longitude: 0.0,
            sog_knots: 10.0,
            cog_deg: 0.0,
            validity: DataValidity::Valid,
            ..NavigationState::default()
        };
        let target = AisTarget {
            mmsi: 123,
            latitude: Some(0.1),
            longitude: Some(0.0),
            sog_knots: Some(10.0),
            cog_deg: Some(180.0),
            heading_deg: Some(180),
            navigation_status: Some(0),
            received_at_ms: 0,
        };

        assert_eq!(
            collision_alarm_for_target(&own, &target, 500_000, &AlarmConfig::default()),
            None
        );
    }
}
