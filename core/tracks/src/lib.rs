use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackPoint {
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp_ms: u64,
    pub cog_deg: Option<f32>,
    pub sog_knots: Option<f32>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrackState {
    Stopped,
    Recording,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackRecorder {
    pub name: String,
    pub state: TrackState,
    pub points: Vec<TrackPoint>,
    pub max_points: usize,
}

impl TrackRecorder {
    pub fn new(name: impl Into<String>, max_points: usize) -> Self {
        Self {
            name: name.into(),
            state: TrackState::Stopped,
            points: Vec::new(),
            max_points: max_points.max(2),
        }
    }

    pub fn start(&mut self) {
        self.state = TrackState::Recording;
    }

    pub fn pause(&mut self) {
        if self.state == TrackState::Recording {
            self.state = TrackState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == TrackState::Paused {
            self.state = TrackState::Recording;
        }
    }

    pub fn stop(&mut self) {
        self.state = TrackState::Stopped;
    }

    pub fn push(&mut self, point: TrackPoint) -> bool {
        if self.state != TrackState::Recording {
            return false;
        }
        if let Some(last) = self.points.last()
            && point.timestamp_ms <= last.timestamp_ms
        {
            return false;
        }
        if self.points.len() == self.max_points {
            self.points.remove(0);
        }
        self.points.push(point);
        true
    }

    pub fn replay(&self) -> impl Iterator<Item = &TrackPoint> {
        self.points.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_records_only_while_active() {
        let mut track = TrackRecorder::new("Trip", 10);
        assert!(!track.push(TrackPoint {
            latitude: 0.0,
            longitude: 0.0,
            timestamp_ms: 1,
            cog_deg: None,
            sog_knots: None,
        }));
        track.start();
        assert!(track.push(TrackPoint {
            latitude: 0.0,
            longitude: 0.0,
            timestamp_ms: 2,
            cog_deg: Some(90.0),
            sog_knots: Some(10.0),
        }));
        track.pause();
        assert!(!track.push(TrackPoint {
            latitude: 0.0,
            longitude: 0.1,
            timestamp_ms: 3,
            cog_deg: Some(90.0),
            sog_knots: Some(10.0),
        }));
    }
}
