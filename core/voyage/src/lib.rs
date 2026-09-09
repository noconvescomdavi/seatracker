#[derive(Debug, Clone)]
pub struct VoyageLeg {
    pub from: String,
    pub to: String,
    pub course_deg: f64,
    pub distance_nm: f64,
    pub speed_knots: f64,
}

impl VoyageLeg {
    pub fn ttg_hours(&self) -> Option<f64> {
        if self.speed_knots <= 0.0 {
            None
        } else {
            Some(self.distance_nm / self.speed_knots)
        }
    }
}

pub fn total_distance_nm(legs: &[VoyageLeg]) -> f64 {
    legs.iter().map(|l| l.distance_nm).sum()
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
    #[test]
    fn voyage_totals() {
        let l = VoyageLeg {
            from: "A".into(),
            to: "B".into(),
            course_deg: 90.0,
            distance_nm: 20.0,
            speed_knots: 10.0,
        };
        assert_eq!(total_distance_nm(&[l.clone()]), 20.0);
        assert_eq!(total_ttg_hours(&[l]), Some(2.0));
    }
}
