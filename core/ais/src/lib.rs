#[derive(Debug, Clone, PartialEq)]
pub struct AisTarget {
    pub mmsi: u32,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub sog_knots: Option<f32>,
    pub cog_deg: Option<f32>,
    pub heading_deg: Option<u16>,
    pub navigation_status: Option<u8>,
    pub received_at_ms: u64,
}

impl AisTarget {
    pub fn is_stale(&self, now_ms: u64, stale_after_ms: u64) -> bool {
        now_ms.saturating_sub(self.received_at_ms) > stale_after_ms
    }
}

pub fn cpa_tcpa_nm(
    own_lat: f64, own_lon: f64, own_sog_kn: f64, own_cog_deg: f64,
    tgt_lat: f64, tgt_lon: f64, tgt_sog_kn: f64, tgt_cog_deg: f64
) -> Option<(f64, f64)> {
    let mean_lat = ((own_lat + tgt_lat) * 0.5).to_radians();
    let dx_nm = (tgt_lon - own_lon) * 60.0 * mean_lat.cos();
    let dy_nm = (tgt_lat - own_lat) * 60.0;
    let v = |s:f64,c:f64| {
        let r=c.to_radians();
        (s*r.sin(), s*r.cos())
    };
    let (ovx,ovy)=v(own_sog_kn,own_cog_deg);
    let (tvx,tvy)=v(tgt_sog_kn,tgt_cog_deg);
    let rvx=tvx-ovx; let rvy=tvy-ovy;
    let vv=rvx*rvx+rvy*rvy;
    if vv < 1e-9 { return None; }
    let tcpa_h = -(dx_nm*rvx + dy_nm*rvy)/vv;
    let cx=dx_nm+rvx*tcpa_h; let cy=dy_nm+rvy*tcpa_h;
    Some(((cx*cx+cy*cy).sqrt(), tcpa_h*60.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn target_staleness() {
        let t=AisTarget{mmsi:123456789,latitude:None,longitude:None,sog_knots:None,cog_deg:None,heading_deg:None,navigation_status:None,received_at_ms:1000};
        assert!(t.is_stale(5000,3000));
    }
}
