//! Deterministic training-vessel dynamics for Estibordo practical simulations.
//! Educational simulator only; not a certified ship-handling model.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VesselState {
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub heading_deg: f64,
    pub course_deg: f64,
    pub speed_kn: f64,
    pub rudder_deg: f64,
    pub engine: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Environment {
    pub current_set_deg: f64,
    pub current_drift_kn: f64,
    pub wind_from_deg: f64,
    pub wind_kn: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VesselModel {
    pub max_speed_kn: f64,
    pub max_astern_kn: f64,
    pub max_rudder_deg: f64,
    pub accel_kn_s: f64,
    pub decel_kn_s: f64,
    pub turn_rate_deg_s_at_full_rudder_10kn: f64,
    pub wind_leeway_factor: f64,
}

impl Default for VesselModel {
    fn default() -> Self {
        Self { max_speed_kn: 15.0, max_astern_kn: 5.0, max_rudder_deg: 35.0,
            accel_kn_s: 0.025, decel_kn_s: 0.045,
            turn_rate_deg_s_at_full_rudder_10kn: 1.8, wind_leeway_factor: 0.002 }
    }
}

fn norm360(v:f64)->f64 { ((v % 360.0)+360.0)%360.0 }
fn components(speed:f64, course_deg:f64)->(f64,f64) {
    let r=course_deg.to_radians(); (speed*r.sin(), speed*r.cos())
}
fn vector_course_speed(e:f64,n:f64)->(f64,f64) {
    (norm360(e.atan2(n).to_degrees()), (e*e+n*n).sqrt())
}

impl VesselModel {
    pub fn step(&self, mut s: VesselState, env: Environment, dt_s:f64)->VesselState {
        let dt=dt_s.clamp(0.0,5.0);
        let target=(s.engine.clamp(-1.0,1.0) * if s.engine>=0.0 {self.max_speed_kn} else {self.max_astern_kn});
        let rate=if target.abs()>s.speed_kn.abs(){self.accel_kn_s}else{self.decel_kn_s};
        let delta=(target-s.speed_kn).clamp(-rate*dt,rate*dt);
        s.speed_kn+=delta;
        let rudder=s.rudder_deg.clamp(-self.max_rudder_deg,self.max_rudder_deg);
        let turn=self.turn_rate_deg_s_at_full_rudder_10kn*(rudder/self.max_rudder_deg)*(s.speed_kn.abs()/10.0);
        s.heading_deg=norm360(s.heading_deg+turn*dt);
        let leeway=(env.wind_kn*self.wind_leeway_factor).min(8.0);
        let water_course=norm360(s.heading_deg + leeway*((env.wind_from_deg-s.heading_deg).to_radians().sin()));
        let (we,wn)=components(s.speed_kn,water_course);
        let (ce,cn)=components(env.current_drift_kn,env.current_set_deg);
        let (cog,sog)=vector_course_speed(we+ce,wn+cn);
        s.course_deg=cog;
        let distance_nm=sog*dt/3600.0;
        let angular=distance_nm/60.0;
        let r=cog.to_radians();
        s.lat_deg += angular*r.cos();
        let coslat=s.lat_deg.to_radians().cos().abs().max(0.01);
        s.lon_deg += angular*r.sin()/coslat;
        if s.lon_deg>180.0{s.lon_deg-=360.0} else if s.lon_deg < -180.0{s.lon_deg+=360.0}
        s
    }
}

#[derive(Debug,Clone,Copy,PartialEq)]
pub struct Assessment {
    pub elapsed_s:f64,
    pub max_xte_nm:f64,
    pub min_clearance_nm:f64,
    pub max_speed_kn:f64,
    pub violations:u32,
}
impl Default for Assessment {
    fn default()->Self{Self{elapsed_s:0.0,max_xte_nm:0.0,min_clearance_nm:f64::INFINITY,max_speed_kn:0.0,violations:0}}
}
impl Assessment {
    pub fn observe(&mut self,dt_s:f64,xte_nm:f64,clearance_nm:f64,speed_kn:f64,xte_limit:f64,min_clearance:f64,speed_limit:f64){
        self.elapsed_s+=dt_s.max(0.0);
        self.max_xte_nm=self.max_xte_nm.max(xte_nm.abs());
        self.min_clearance_nm=self.min_clearance_nm.min(clearance_nm);
        self.max_speed_kn=self.max_speed_kn.max(speed_kn.abs());
        if xte_nm.abs()>xte_limit || clearance_nm<min_clearance || speed_kn.abs()>speed_limit { self.violations+=1; }
    }
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test] fn current_changes_ground_track(){let m=VesselModel::default();let s=VesselState{lat_deg:-22.0,lon_deg:-42.0,heading_deg:90.0,course_deg:90.0,speed_kn:10.0,rudder_deg:0.0,engine:10.0/15.0};let o=m.step(s,Environment{current_set_deg:0.0,current_drift_kn:2.0,wind_from_deg:0.0,wind_kn:0.0},1.0);assert!(o.course_deg<90.0);}
 #[test] fn rudder_turns_heading(){let m=VesselModel::default();let s=VesselState{lat_deg:0.0,lon_deg:0.0,heading_deg:0.0,course_deg:0.0,speed_kn:10.0,rudder_deg:35.0,engine:10.0/15.0};assert!(m.step(s,Environment{current_set_deg:0.0,current_drift_kn:0.0,wind_from_deg:0.0,wind_kn:0.0},1.0).heading_deg>0.0);}
}
