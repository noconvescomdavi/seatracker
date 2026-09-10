use seatracker_nmea::{RmbData, encode_apb, encode_rmb, encode_xte_nm};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutopilotState {
    Disarmed,
    Armed,
    Engaged,
    Fault,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Guidance {
    pub xte_nm: f64,
    pub steer_right: bool,
    pub origin_id: String,
    pub destination_id: String,
    pub destination_lat: f64,
    pub destination_lon: f64,
    pub range_nm: f64,
    pub bearing_origin_to_destination_deg: f64,
    pub bearing_present_to_destination_deg: f64,
    pub heading_to_steer_deg: f64,
    pub closing_velocity_knots: f64,
    pub arrival_circle_entered: bool,
}

pub trait AutopilotTransport {
    fn name(&self) -> &'static str;
    fn send_sentence(&mut self, sentence: &str) -> Result<(), String>;
}

pub struct AutopilotController<T: AutopilotTransport> {
    state: AutopilotState,
    transport: T,
    last_fault: Option<String>,
}

impl<T: AutopilotTransport> AutopilotController<T> {
    pub fn new(transport: T) -> Self {
        Self {
            state: AutopilotState::Disarmed,
            transport,
            last_fault: None,
        }
    }

    pub fn state(&self) -> AutopilotState {
        self.state
    }

    pub fn arm(&mut self) {
        if self.state == AutopilotState::Disarmed {
            self.state = AutopilotState::Armed;
        }
    }

    pub fn engage(&mut self) -> Result<(), String> {
        if self.state != AutopilotState::Armed {
            return Err("autopilot must be armed before engagement".into());
        }
        self.state = AutopilotState::Engaged;
        Ok(())
    }

    pub fn disengage(&mut self) {
        self.state = AutopilotState::Disarmed;
    }

    pub fn send_guidance(&mut self, guidance: &Guidance) -> Result<(), String> {
        if self.state != AutopilotState::Engaged {
            return Err("autopilot output blocked while not engaged".into());
        }

        let sentences = [
            encode_xte_nm(guidance.xte_nm, guidance.steer_right),
            encode_rmb(&RmbData {
                xte_nm: guidance.xte_nm,
                steer_right: guidance.steer_right,
                origin_id: &guidance.origin_id,
                destination_id: &guidance.destination_id,
                destination_lat: guidance.destination_lat,
                destination_lon: guidance.destination_lon,
                range_nm: guidance.range_nm,
                bearing_deg: guidance.bearing_present_to_destination_deg,
                closing_velocity_knots: guidance.closing_velocity_knots,
                arrival: guidance.arrival_circle_entered,
            }),
            encode_apb(
                guidance.xte_nm,
                guidance.steer_right,
                guidance.bearing_origin_to_destination_deg,
                guidance.bearing_present_to_destination_deg,
                guidance.heading_to_steer_deg,
                &guidance.destination_id,
                guidance.arrival_circle_entered,
            ),
        ];

        for sentence in sentences {
            if let Err(error) = self.transport.send_sentence(&sentence) {
                self.last_fault = Some(error.clone());
                self.state = AutopilotState::Fault;
                return Err(error);
            }
        }
        Ok(())
    }

    pub fn last_fault(&self) -> Option<&str> {
        self.last_fault.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MemoryTransport {
        sent: Vec<String>,
    }

    impl AutopilotTransport for MemoryTransport {
        fn name(&self) -> &'static str {
            "memory"
        }

        fn send_sentence(&mut self, sentence: &str) -> Result<(), String> {
            self.sent.push(sentence.to_string());
            Ok(())
        }
    }

    #[test]
    fn output_requires_explicit_arm_and_engage() {
        let transport = MemoryTransport::default();
        let mut controller = AutopilotController::new(transport);
        let guidance = Guidance {
            xte_nm: 0.1,
            steer_right: true,
            origin_id: "A".into(),
            destination_id: "B".into(),
            destination_lat: -22.9,
            destination_lon: -43.2,
            range_nm: 2.0,
            bearing_origin_to_destination_deg: 90.0,
            bearing_present_to_destination_deg: 88.0,
            heading_to_steer_deg: 87.0,
            closing_velocity_knots: 8.0,
            arrival_circle_entered: false,
        };

        assert!(controller.send_guidance(&guidance).is_err());
        controller.arm();
        controller.engage().unwrap();
        assert!(controller.send_guidance(&guidance).is_ok());
        assert_eq!(controller.state(), AutopilotState::Engaged);
    }
}
