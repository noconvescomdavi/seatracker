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

#[derive(Debug, Clone, PartialEq)]
pub enum AisMessage {
    PositionReport(AisTarget),
    Unsupported { message_type: u8, mmsi: Option<u32> },
}

impl AisTarget {
    pub fn is_stale(&self, now_ms: u64, stale_after_ms: u64) -> bool {
        now_ms.saturating_sub(self.received_at_ms) > stale_after_ms
    }
}

fn sixbit(c: u8) -> Option<u8> {
    let mut v = c.checked_sub(48)?;
    if v > 40 { v = v.checked_sub(8)?; }
    (v < 64).then_some(v)
}

fn payload_bits(payload: &str, fill_bits: u8) -> Result<Vec<bool>, String> {
    if fill_bits > 5 { return Err("invalid AIS fill bits".into()); }
    let mut bits = Vec::with_capacity(payload.len()*6);
    for c in payload.bytes() {
        let v = sixbit(c).ok_or("invalid AIS payload character")?;
        for shift in (0..6).rev() { bits.push(((v >> shift) & 1) != 0); }
    }
    for _ in 0..fill_bits { bits.pop(); }
    Ok(bits)
}

fn ubits(bits:&[bool], start:usize, len:usize)->Option<u64>{
    let mut out=0u64;
    for i in start..start+len {
        out=(out<<1) | (*bits.get(i)? as u64);
    }
    Some(out)
}

fn sbits(bits:&[bool], start:usize, len:usize)->Option<i64>{
    let u=ubits(bits,start,len)?;
    let sign=1u64<<(len-1);
    if u & sign != 0 { Some((u as i64) - ((1u64<<len) as i64)) } else { Some(u as i64) }
}

pub fn parse_vdm_vdo(sentence: &str, received_at_ms: u64) -> Result<AisMessage, String> {
    let body = sentence.trim().trim_start_matches('!');
    let body = body.split('*').next().unwrap_or(body);
    let f:Vec<&str>=body.split(',').collect();
    if f.len() < 7 { return Err("short AIS sentence".into()); }
    if !f[0].ends_with("VDM") && !f[0].ends_with("VDO") { return Err("not AIS VDM/VDO".into()); }
    if f[1] != "1" { return Err("multi-fragment AIS not supported by single-sentence parser".into()); }

    let fill:u8=f[6].parse().map_err(|_|"invalid fill bits")?;
    let bits=payload_bits(f[5],fill)?;
    let message_type=ubits(&bits,0,6).ok_or("missing message type")? as u8;
    let mmsi=ubits(&bits,8,30).map(|v|v as u32);

    match message_type {
        1|2|3 => {
            let mmsi=mmsi.ok_or("missing MMSI")?;
            let nav=ubits(&bits,38,4).map(|v|v as u8);
            let sog_raw=ubits(&bits,50,10).unwrap_or(1023);
            let lon_raw=sbits(&bits,61,28).ok_or("missing longitude")?;
            let lat_raw=sbits(&bits,89,27).ok_or("missing latitude")?;
            let cog_raw=ubits(&bits,116,12).unwrap_or(3600);
            let hdg_raw=ubits(&bits,128,9).unwrap_or(511);

            let longitude = if lon_raw.abs() <= 108_600_000 { Some(lon_raw as f64 / 600_000.0) } else { None };
            let latitude = if lat_raw.abs() <= 54_600_000 { Some(lat_raw as f64 / 600_000.0) } else { None };

            Ok(AisMessage::PositionReport(AisTarget {
                mmsi,
                latitude,
                longitude,
                sog_knots:(sog_raw < 1023).then_some(sog_raw as f32 / 10.0),
                cog_deg:(cog_raw < 3600).then_some(cog_raw as f32 / 10.0),
                heading_deg:(hdg_raw < 511).then_some(hdg_raw as u16),
                navigation_status:nav,
                received_at_ms,
            }))
        }
        _ => Ok(AisMessage::Unsupported { message_type, mmsi })
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

    #[test]
    fn target_staleness() {
        let t=AisTarget{mmsi:123456789,latitude:None,longitude:None,sog_knots:None,cog_deg:None,heading_deg:None,navigation_status:None,received_at_ms:1000};
        assert!(t.is_stale(5000,3000));
    }

    #[test]
    fn parses_single_fragment_position_report() {
        let s="!AIVDM,1,1,,A,15Muq?P0000G?t@E>4p@wvN20<0u,0*00";
        let m=parse_vdm_vdo(s,1234).unwrap();
        match m {
            AisMessage::PositionReport(t) => {
                assert!(t.mmsi > 0);
                assert_eq!(t.received_at_ms,1234);
            }
            _ => panic!()
        }
    }
}
