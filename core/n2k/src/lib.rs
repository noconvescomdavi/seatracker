#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pgn {
    VesselHeading,
    RateOfTurn,
    Attitude,
    PositionRapidUpdate,
    CogSogRapidUpdate,
    GnssPositionData,
    WindData,
    WaterDepth,
    SpeedWaterReferenced,
    EngineParametersRapid,
    EngineParametersDynamic,
    FluidLevel,
    BatteryStatus,
    AisClassAPosition,
    AisClassBPosition,
    Unknown(u32),
}

impl Pgn {
    pub fn from_number(value: u32) -> Self {
        match value {
            127250 => Self::VesselHeading,
            127251 => Self::RateOfTurn,
            127257 => Self::Attitude,
            129025 => Self::PositionRapidUpdate,
            129026 => Self::CogSogRapidUpdate,
            129029 => Self::GnssPositionData,
            130306 => Self::WindData,
            128267 => Self::WaterDepth,
            128259 => Self::SpeedWaterReferenced,
            127488 => Self::EngineParametersRapid,
            127489 => Self::EngineParametersDynamic,
            127505 => Self::FluidLevel,
            127508 => Self::BatteryStatus,
            129038 => Self::AisClassAPosition,
            129039 => Self::AisClassBPosition,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum N2kObservation {
    Position { latitude: f64, longitude: f64 },
    CogSog { cog_true_deg: f32, sog_knots: f32 },
    Heading { heading_deg: f32, magnetic: bool },
    Depth { meters: f32 },
    Wind { angle_deg: f32, speed_knots: f32, apparent: bool },
    EngineRpm { instance: u8, rpm: f32 },
    BatteryVoltage { instance: u8, volts: f32 },
    Unknown { pgn: u32, payload: Vec<u8> },
}

pub trait N2kDecoder {
    fn decoder_name(&self) -> &'static str;
    fn decode(&self, pgn: u32, payload: &[u8]) -> Result<N2kObservation, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_common_pgns() {
        assert_eq!(Pgn::from_number(129025), Pgn::PositionRapidUpdate);
        assert_eq!(Pgn::from_number(130306), Pgn::WindData);
    }
}
