#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeatherField {
    Wind,
    Pressure,
    Waves,
    Current,
    Precipitation,
    AirTemperature,
    SeaTemperature,
    CloudCover,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VectorSample {
    pub u: f32,
    pub v: f32,
}

impl VectorSample {
    pub fn speed(&self) -> f32 {
        (self.u * self.u + self.v * self.v).sqrt()
    }

    pub fn direction_to_deg(&self) -> f32 {
        let deg = self.u.atan2(self.v).to_degrees();
        deg.rem_euclid(360.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WeatherValue {
    Scalar(f32),
    Vector(VectorSample),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeatherSample {
    pub field: WeatherField,
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp_ms: u64,
    pub value: WeatherValue,
}

#[derive(Debug, Clone, Default)]
pub struct WeatherDataset {
    pub source: String,
    pub model: Option<String>,
    pub generated_at_ms: Option<u64>,
    pub samples: Vec<WeatherSample>,
}

impl WeatherDataset {
    pub fn fields(&self) -> Vec<WeatherField> {
        let mut fields = Vec::new();
        for sample in &self.samples {
            if !fields.contains(&sample.field) {
                fields.push(sample.field);
            }
        }
        fields
    }

    pub fn nearest(
        &self,
        field: WeatherField,
        latitude: f64,
        longitude: f64,
        timestamp_ms: u64,
    ) -> Option<&WeatherSample> {
        self.samples
            .iter()
            .filter(|sample| sample.field == field)
            .min_by_key(|sample| {
                let dt = sample.timestamp_ms.abs_diff(timestamp_ms) as u128;
                let dlat = ((sample.latitude - latitude).abs() * 1_000_000.0) as u128;
                let dlon = ((sample.longitude - longitude).abs() * 1_000_000.0) as u128;
                dt.saturating_add(dlat).saturating_add(dlon)
            })
    }
}

pub trait GribProvider {
    fn provider_name(&self) -> &'static str;
    fn can_open(&self, header: &[u8], extension: Option<&str>) -> bool;
    fn load(&self, bytes: &[u8]) -> Result<WeatherDataset, String>;
}

pub fn looks_like_grib(bytes: &[u8]) -> bool {
    bytes.windows(4).take(64).any(|window| window == b"GRIB")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_grib_signature() {
        assert!(looks_like_grib(b"xxxxGRIByyyy"));
    }

    #[test]
    fn vector_speed() {
        let sample = VectorSample { u: 3.0, v: 4.0 };
        assert_eq!(sample.speed(), 5.0);
    }
}
