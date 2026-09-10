use seatracker_geographic::{great_circle_distance_m, meters_to_nm};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PositionSample {
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Odometer {
    pub trip_nm: f64,
    pub total_nm: f64,
    last: Option<PositionSample>,
}

impl Odometer {
    pub fn update(&mut self, sample: PositionSample, max_step_nm: f64) -> bool {
        let Some(last) = self.last.replace(sample) else {
            return false;
        };
        if sample.timestamp_ms <= last.timestamp_ms {
            self.last = Some(last);
            return false;
        }
        let distance_nm = meters_to_nm(great_circle_distance_m(
            last.latitude,
            last.longitude,
            sample.latitude,
            sample.longitude,
        ));
        if !distance_nm.is_finite() || distance_nm <= 0.0 || distance_nm > max_step_nm {
            return false;
        }
        self.trip_nm += distance_nm;
        self.total_nm += distance_nm;
        true
    }

    pub fn reset_trip(&mut self) {
        self.trip_nm = 0.0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerState {
    Stopped,
    Running,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CountdownTimer {
    duration_ms: u64,
    started_at_ms: Option<u64>,
}

impl CountdownTimer {
    pub fn new(duration_ms: u64) -> Self {
        Self {
            duration_ms,
            started_at_ms: None,
        }
    }

    pub fn start(&mut self, now_ms: u64) {
        self.started_at_ms = Some(now_ms);
    }

    pub fn stop(&mut self) {
        self.started_at_ms = None;
    }

    pub fn remaining_ms(&self, now_ms: u64) -> u64 {
        let Some(started) = self.started_at_ms else {
            return self.duration_ms;
        };
        self.duration_ms
            .saturating_sub(now_ms.saturating_sub(started))
    }

    pub fn state(&self, now_ms: u64) -> TimerState {
        let Some(started) = self.started_at_ms else {
            return TimerState::Stopped;
        };
        if now_ms.saturating_sub(started) >= self.duration_ms {
            TimerState::Expired
        } else {
            TimerState::Running
        }
    }
}

pub fn eta_ms(now_ms: u64, distance_nm: f64, speed_knots: f64) -> Option<u64> {
    if !distance_nm.is_finite()
        || !speed_knots.is_finite()
        || distance_nm < 0.0
        || speed_knots <= 0.0
    {
        return None;
    }
    let travel_ms = distance_nm / speed_knots * 3_600_000.0;
    if travel_ms > u64::MAX as f64 {
        return None;
    }
    now_ms.checked_add(travel_ms.round() as u64)
}

pub fn fuel_required(distance_nm: f64, speed_knots: f64, burn_per_hour: f64) -> Option<f64> {
    if distance_nm < 0.0 || speed_knots <= 0.0 || burn_per_hour < 0.0 {
        return None;
    }
    Some(distance_nm / speed_knots * burn_per_hour)
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DeviationTable {
    points: Vec<(f64, f64)>,
}

impl DeviationTable {
    pub fn new(mut points: Vec<(f64, f64)>) -> Self {
        points.retain(|(heading, deviation)| heading.is_finite() && deviation.is_finite());
        points.sort_by(|a, b| a.0.total_cmp(&b.0));
        Self { points }
    }

    pub fn deviation_deg(&self, heading_deg: f64) -> Option<f64> {
        if self.points.is_empty() {
            return None;
        }
        let heading = heading_deg.rem_euclid(360.0);
        if self.points.len() == 1 {
            return Some(self.points[0].1);
        }

        for pair in self.points.windows(2) {
            let (h0, d0) = pair[0];
            let (h1, d1) = pair[1];
            if heading >= h0 && heading <= h1 && h1 > h0 {
                let t = (heading - h0) / (h1 - h0);
                return Some(d0 + (d1 - d0) * t);
            }
        }

        let (last_h, last_d) = *self.points.last()?;
        let (first_h, first_d) = self.points[0];
        let wrapped_heading = if heading < first_h { heading + 360.0 } else { heading };
        let wrapped_first = first_h + 360.0;
        let span = wrapped_first - last_h;
        if span <= 0.0 {
            return Some(last_d);
        }
        let t = (wrapped_heading - last_h) / span;
        Some(last_d + (first_d - last_d) * t)
    }

    pub fn magnetic_to_compass(&self, magnetic_deg: f64) -> Option<f64> {
        let deviation = self.deviation_deg(magnetic_deg)?;
        Some((magnetic_deg - deviation).rem_euclid(360.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn odometer_accumulates_short_valid_steps() {
        let mut odo = Odometer::default();
        odo.update(
            PositionSample {
                latitude: 0.0,
                longitude: 0.0,
                timestamp_ms: 1,
            },
            5.0,
        );
        assert!(odo.update(
            PositionSample {
                latitude: 0.0,
                longitude: 0.01,
                timestamp_ms: 2,
            },
            5.0,
        ));
        assert!(odo.trip_nm > 0.5);
    }

    #[test]
    fn countdown_expires() {
        let mut timer = CountdownTimer::new(1_000);
        timer.start(100);
        assert_eq!(timer.state(500), TimerState::Running);
        assert_eq!(timer.state(1_100), TimerState::Expired);
    }

    #[test]
    fn deviation_interpolates() {
        let table = DeviationTable::new(vec![(0.0, 0.0), (90.0, 4.0)]);
        let value = table.deviation_deg(45.0).unwrap();
        assert!((value - 2.0).abs() < 1e-9);
    }
}
