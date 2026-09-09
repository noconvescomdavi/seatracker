use seatracker_routes::Route;

#[derive(Debug, Clone)]
pub struct VoyageLeg {
    pub from: String,
    pub to: String,
    pub course_deg: f64,
    pub distance_nm: f64,
    pub speed_knots: f64,
    pub departure_ms: Option<u64>,
    pub notes: Option<String>,
}

impl VoyageLeg {
    pub fn ttg_hours(&self) -> Option<f64> {
        if self.speed_knots <= 0.0 {
            None
        } else {
            Some(self.distance_nm / self.speed_knots)
        }
    }

    pub fn eta_ms(&self) -> Option<u64> {
        let departure = self.departure_ms?;
        let delta_ms = self.ttg_hours()? * 3_600_000.0;
        if !delta_ms.is_finite() || delta_ms < 0.0 || delta_ms > u64::MAX as f64 {
            return None;
        }
        departure.checked_add(delta_ms.round() as u64)
    }
}

pub fn plan_route(route: &Route, speed_knots: f64, departure_ms: Option<u64>) -> Vec<VoyageLeg> {
    let route_legs = route.legs();
    route
        .waypoints
        .windows(2)
        .zip(route_legs)
        .scan(departure_ms, |next_departure, (pair, leg)| {
            let voyage_leg = VoyageLeg {
                from: pair[0].name.clone(),
                to: pair[1].name.clone(),
                course_deg: leg.course_deg,
                distance_nm: leg.distance_nm,
                speed_knots,
                departure_ms: *next_departure,
                notes: pair[1].notes.clone(),
            };
            *next_departure = voyage_leg.eta_ms();
            Some(voyage_leg)
        })
        .collect()
}

pub fn total_distance_nm(legs: &[VoyageLeg]) -> f64 {
    legs.iter().map(|leg| leg.distance_nm).sum()
}

pub fn total_ttg_hours(legs: &[VoyageLeg]) -> Option<f64> {
    let mut total = 0.0;
    for leg in legs {
        total += leg.ttg_hours()?;
    }
    Some(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use seatracker_routes::Waypoint;

    #[test]
    fn voyage_totals() {
        let leg = VoyageLeg {
            from: "A".into(),
            to: "B".into(),
            course_deg: 90.0,
            distance_nm: 20.0,
            speed_knots: 10.0,
            departure_ms: Some(1_000),
            notes: None,
        };
        assert_eq!(total_distance_nm(std::slice::from_ref(&leg)), 20.0);
        assert_eq!(total_ttg_hours(&[leg]), Some(2.0));
    }

    #[test]
    fn route_plan_has_eta() {
        let route = Route {
            name: "R".into(),
            waypoints: vec![
                Waypoint {
                    name: "A".into(),
                    latitude: 0.0,
                    longitude: 0.0,
                    notes: None,
                    arrival_radius_nm: None,
                },
                Waypoint {
                    name: "B".into(),
                    latitude: 0.0,
                    longitude: 1.0,
                    notes: None,
                    arrival_radius_nm: None,
                },
            ],
        };
        let plan = plan_route(&route, 10.0, Some(0));
        assert_eq!(plan.len(), 1);
        assert!(plan[0].eta_ms().unwrap() > 0);
    }
}
