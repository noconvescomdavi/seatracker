use seatracker_nmea::NmeaMessage;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct InstrumentState {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub sog_knots: Option<f32>,
    pub cog_deg: Option<f32>,
    pub heading_true_deg: Option<f32>,
    pub heading_magnetic_deg: Option<f32>,
    pub depth_m: Option<f32>,
    pub water_temperature_c: Option<f32>,
    pub apparent_wind_angle_deg: Option<f32>,
    pub apparent_wind_speed_knots: Option<f32>,
    pub true_wind_direction_deg: Option<f32>,
    pub true_wind_speed_knots: Option<f32>,
    pub rudder_angle_deg: Option<f32>,
    pub trip_nm: Option<f32>,
    pub total_nm: Option<f32>,
    pub satellites: Option<u8>,
    pub hdop: Option<f32>,
}

impl InstrumentState {
    pub fn apply(&mut self, message: &NmeaMessage) {
        match message {
            NmeaMessage::Rmc {
                lat,
                lon,
                sog_knots,
                cog_deg,
                valid,
            } if *valid => {
                self.latitude = Some(*lat);
                self.longitude = Some(*lon);
                self.sog_knots = Some(*sog_knots);
                self.cog_deg = Some(*cog_deg);
            }
            NmeaMessage::Gga {
                lat,
                lon,
                satellites,
                hdop,
                ..
            } => {
                self.latitude = Some(*lat);
                self.longitude = Some(*lon);
                self.satellites = Some(*satellites);
                self.hdop = *hdop;
            }
            NmeaMessage::Vtg { cog_deg, sog_knots } => {
                if cog_deg.is_some() {
                    self.cog_deg = *cog_deg;
                }
                if sog_knots.is_some() {
                    self.sog_knots = *sog_knots;
                }
            }
            NmeaMessage::Hdt { heading_true } => self.heading_true_deg = Some(*heading_true),
            NmeaMessage::Hdg {
                heading_magnetic, ..
            } => {
                self.heading_magnetic_deg = Some(*heading_magnetic);
            }
            NmeaMessage::Dbt { depth_m } | NmeaMessage::Dpt { depth_m, .. } => {
                self.depth_m = Some(*depth_m);
            }
            NmeaMessage::Mtw { temperature_c } => self.water_temperature_c = Some(*temperature_c),
            NmeaMessage::Mwv {
                angle_deg,
                speed_knots,
                relative,
                valid,
            } if *valid && *relative => {
                self.apparent_wind_angle_deg = Some(*angle_deg);
                self.apparent_wind_speed_knots = Some(*speed_knots);
            }
            NmeaMessage::Mwd {
                direction_true_deg,
                speed_knots,
            } => {
                self.true_wind_direction_deg = Some(*direction_true_deg);
                self.true_wind_speed_knots = Some(*speed_knots);
            }
            NmeaMessage::Rsa {
                starboard_rudder_deg,
                ..
            } => {
                self.rudder_angle_deg = *starboard_rudder_deg;
            }
            NmeaMessage::Vlw { total_nm, trip_nm } => {
                self.total_nm = *total_nm;
                self.trip_nm = *trip_nm;
            }
            NmeaMessage::Gsv { satellites_in_view } => self.satellites = *satellites_in_view,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_depth() {
        let mut state = InstrumentState::default();
        state.apply(&NmeaMessage::Dbt { depth_m: 12.3 });
        assert_eq!(state.depth_m, Some(12.3));
    }
}
