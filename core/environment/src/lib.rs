#[derive(Debug, Clone)]
pub struct TideSample {
    pub station_id: String,
    pub timestamp_ms: u64,
    pub height_m: f64,
    pub source: String,
    pub valid_until_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct CurrentSample {
    pub station_id: String,
    pub timestamp_ms: u64,
    pub direction_true_deg: f32,
    pub speed_knots: f32,
    pub source: String,
    pub valid_until_ms: Option<u64>,
}

pub trait TideProvider {
    fn provider_name(&self) -> &'static str;
    fn samples(&self, station_id: &str, from_ms: u64, to_ms: u64) -> Result<Vec<TideSample>, String>;
}

pub trait CurrentProvider {
    fn provider_name(&self) -> &'static str;
    fn samples(&self, station_id: &str, from_ms: u64, to_ms: u64) -> Result<Vec<CurrentSample>, String>;
}
