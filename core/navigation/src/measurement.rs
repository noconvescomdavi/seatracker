pub fn ebl_endpoint(lat: f64, lon: f64, bearing_deg: f64, range_nm: f64) -> (f64, f64) {
    let r_nm = 3440.065_f64;
    let d = range_nm / r_nm;
    let brg = bearing_deg.to_radians();
    let lat1 = lat.to_radians();
    let lon1 = lon.to_radians();
    let lat2 = (lat1.sin()*d.cos() + lat1.cos()*d.sin()*brg.cos()).asin();
    let lon2 = lon1 + (brg.sin()*d.sin()*lat1.cos()).atan2(d.cos()-lat1.sin()*lat2.sin());
    (lat2.to_degrees(), ((lon2.to_degrees()+540.0)%360.0)-180.0)
}

pub fn initial_bearing_deg(lat1:f64, lon1:f64, lat2:f64, lon2:f64) -> f64 {
    let a=lat1.to_radians(); let b=lat2.to_radians(); let dl=(lon2-lon1).to_radians();
    let y=dl.sin()*b.cos();
    let x=a.cos()*b.sin()-a.sin()*b.cos()*dl.cos();
    (y.atan2(x).to_degrees()+360.0)%360.0
}

pub fn haversine_nm(lat1:f64, lon1:f64, lat2:f64, lon2:f64) -> f64 {
    let r_nm=3440.065_f64;
    let dlat=(lat2-lat1).to_radians(); let dlon=(lon2-lon1).to_radians();
    let a=(dlat/2.0).sin().powi(2)+lat1.to_radians().cos()*lat2.to_radians().cos()*(dlon/2.0).sin().powi(2);
    2.0*r_nm*a.sqrt().asin()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn ebl_vrm_roundtrip() {
        let p=ebl_endpoint(-22.9,-43.2,90.0,10.0);
        assert!((haversine_nm(-22.9,-43.2,p.0,p.1)-10.0).abs()<0.02);
    }
}
