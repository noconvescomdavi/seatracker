#[derive(Debug, Clone, PartialEq)]
pub struct TideSample {
    pub station_id: String,
    pub timestamp_ms: u64,
    pub height_m: f64,
    pub source: String,
    pub valid_until_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CurrentSample {
    pub station_id: String,
    pub timestamp_ms: u64,
    pub direction_true_deg: f32,
    pub speed_knots: f32,
    pub source: String,
    pub valid_until_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExtremumKind {
    HighWater,
    LowWater,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TideExtremum {
    pub kind: ExtremumKind,
    pub timestamp_ms: u64,
    pub height_m: f64,
}

pub trait TideProvider {
    fn provider_name(&self) -> &'static str;
    fn samples(
        &self,
        station_id: &str,
        from_ms: u64,
        to_ms: u64,
    ) -> Result<Vec<TideSample>, String>;
}

pub trait CurrentProvider {
    fn provider_name(&self) -> &'static str;
    fn samples(
        &self,
        station_id: &str,
        from_ms: u64,
        to_ms: u64,
    ) -> Result<Vec<CurrentSample>, String>;
}

pub fn tide_extrema(samples: &[TideSample]) -> Vec<TideExtremum> {
    samples
        .windows(3)
        .filter_map(|window| {
            let prev = &window[0];
            let current = &window[1];
            let next = &window[2];
            let kind = if current.height_m > prev.height_m && current.height_m > next.height_m {
                ExtremumKind::HighWater
            } else if current.height_m < prev.height_m && current.height_m < next.height_m {
                ExtremumKind::LowWater
            } else {
                return None;
            };
            Some(TideExtremum {
                kind,
                timestamp_ms: current.timestamp_ms,
                height_m: current.height_m,
            })
        })
        .collect()
}

pub fn sample_is_valid(valid_until_ms: Option<u64>, now_ms: u64) -> bool {
    valid_until_ms.map(|until| now_ms <= until).unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_high_water() {
        let samples = vec![
            TideSample {
                station_id: "A".into(),
                timestamp_ms: 0,
                height_m: 1.0,
                source: "x".into(),
                valid_until_ms: None,
            },
            TideSample {
                station_id: "A".into(),
                timestamp_ms: 1,
                height_m: 2.0,
                source: "x".into(),
                valid_until_ms: None,
            },
            TideSample {
                station_id: "A".into(),
                timestamp_ms: 2,
                height_m: 1.0,
                source: "x".into(),
                valid_until_ms: None,
            },
        ];
        assert_eq!(tide_extrema(&samples)[0].kind, ExtremumKind::HighWater);
    }
}
