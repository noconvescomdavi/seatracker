use serde_json::Value;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SignalKState {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub sog_knots: Option<f64>,
    pub cog_true_deg: Option<f64>,
    pub heading_true_deg: Option<f64>,
    pub depth_below_transducer_m: Option<f64>,
    pub apparent_wind_angle_deg: Option<f64>,
    pub apparent_wind_speed_knots: Option<f64>,
}

impl SignalKState {
    pub fn apply_delta_json(&mut self, json: &str) -> Result<usize, String> {
        let root: Value = serde_json::from_str(json).map_err(|error| error.to_string())?;
        let updates = root
            .get("updates")
            .and_then(Value::as_array)
            .ok_or("Signal K delta missing updates")?;

        let mut applied = 0usize;
        for update in updates {
            let Some(values) = update.get("values").and_then(Value::as_array) else {
                continue;
            };
            for item in values {
                let Some(path) = item.get("path").and_then(Value::as_str) else {
                    continue;
                };
                let value = item.get("value").unwrap_or(&Value::Null);
                match path {
                    "navigation.position" => {
                        let lat = value.get("latitude").and_then(Value::as_f64);
                        let lon = value.get("longitude").and_then(Value::as_f64);
                        if let (Some(lat), Some(lon)) = (lat, lon)
                            && (-90.0..=90.0).contains(&lat)
                            && (-180.0..=180.0).contains(&lon)
                        {
                            self.latitude = Some(lat);
                            self.longitude = Some(lon);
                            applied += 1;
                        }
                    }
                    "navigation.speedOverGround" => {
                        if let Some(mps) = value.as_f64() {
                            self.sog_knots = Some(mps * 1.943_844_492_440_6);
                            applied += 1;
                        }
                    }
                    "navigation.courseOverGroundTrue" => {
                        if let Some(rad) = value.as_f64() {
                            self.cog_true_deg = Some(rad.to_degrees().rem_euclid(360.0));
                            applied += 1;
                        }
                    }
                    "navigation.headingTrue" => {
                        if let Some(rad) = value.as_f64() {
                            self.heading_true_deg = Some(rad.to_degrees().rem_euclid(360.0));
                            applied += 1;
                        }
                    }
                    "environment.depth.belowTransducer" => {
                        if let Some(meters) = value.as_f64() {
                            self.depth_below_transducer_m = Some(meters);
                            applied += 1;
                        }
                    }
                    "environment.wind.angleApparent" => {
                        if let Some(rad) = value.as_f64() {
                            self.apparent_wind_angle_deg = Some(rad.to_degrees());
                            applied += 1;
                        }
                    }
                    "environment.wind.speedApparent" => {
                        if let Some(mps) = value.as_f64() {
                            self.apparent_wind_speed_knots = Some(mps * 1.943_844_492_440_6);
                            applied += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(applied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_navigation_delta() {
        let json = r#"{"updates":[{"values":[
          {"path":"navigation.position","value":{"latitude":-22.9,"longitude":-43.2}},
          {"path":"navigation.speedOverGround","value":5.14444}
        ]}]}"#;
        let mut state = SignalKState::default();
        assert_eq!(state.apply_delta_json(json).unwrap(), 2);
        assert_eq!(state.latitude, Some(-22.9));
        assert!((state.sog_knots.unwrap() - 10.0).abs() < 0.01);
    }
}
