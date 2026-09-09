use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogCategory {
    Chart,
    Gps,
    Ais,
    Nmea,
    Route,
    Render,
    Database,
    Alarm,
    Import,
    System,
}

#[derive(Debug, Clone, Default)]
pub struct Diagnostics {
    pub fps: Option<f32>,
    pub memory_bytes: Option<u64>,
    pub charts_loaded: usize,
    pub gps_valid: bool,
    pub ais_targets: usize,
    pub nmea_messages_per_second: f32,
    pub render_time_ms: Option<f32>,
    pub cache_bytes: u64,
    pub errors: u64,
    pub counters: BTreeMap<String, u64>,
}

impl Diagnostics {
    pub fn increment(&mut self, key: impl Into<String>) {
        *self.counters.entry(key.into()).or_insert(0) += 1;
    }

    pub fn record_error(&mut self) {
        self.errors = self.errors.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_increment() {
        let mut diagnostics = Diagnostics::default();
        diagnostics.increment("gps.fix");
        diagnostics.increment("gps.fix");
        assert_eq!(diagnostics.counters["gps.fix"], 2);
    }
}
