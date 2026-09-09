use std::collections::HashMap;

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

#[derive(Debug, Clone)]
struct FragmentGroup {
    total: u8,
    channel: String,
    payloads: Vec<Option<String>>,
    fill_bits: u8,
    updated_at_ms: u64,
}

#[derive(Debug, Default)]
pub struct AisAssembler {
    groups: HashMap<String, FragmentGroup>,
}

impl AisAssembler {
    pub fn push(
        &mut self,
        sentence: &str,
        received_at_ms: u64,
    ) -> Result<Option<AisMessage>, String> {
        let body = sentence.trim().trim_start_matches('!');
        let body = body.split('*').next().unwrap_or(body);
        let f: Vec<&str> = body.split(',').collect();

        if f.len() < 7 {
            return Err("short AIS sentence".into());
        }
        if !f[0].ends_with("VDM") && !f[0].ends_with("VDO") {
            return Err("not AIS VDM/VDO".into());
        }

        let total: u8 = f[1].parse().map_err(|_| "invalid fragment count")?;
        let number: u8 = f[2].parse().map_err(|_| "invalid fragment number")?;
        if total == 0 || number == 0 || number > total {
            return Err("invalid AIS fragment numbering".into());
        }

        if total == 1 {
            return parse_vdm_vdo(sentence, received_at_ms).map(Some);
        }

        let seq = f[3];
        if seq.is_empty() {
            return Err("multi-fragment AIS requires sequential message id".into());
        }

        let channel = f[4].to_string();
        let fill_bits: u8 = f[6].parse().map_err(|_| "invalid fill bits")?;
        let key = format!("{seq}:{channel}");

        let group = self
            .groups
            .entry(key.clone())
            .or_insert_with(|| FragmentGroup {
                total,
                channel: channel.clone(),
                payloads: vec![None; total as usize],
                fill_bits: 0,
                updated_at_ms: received_at_ms,
            });

        if group.total != total || group.channel != channel {
            self.groups.remove(&key);
            return Err("AIS fragment group mismatch".into());
        }

        group.updated_at_ms = received_at_ms;
        group.payloads[(number - 1) as usize] = Some(f[5].to_string());
        if number == total {
            group.fill_bits = fill_bits;
        }

        if group.payloads.iter().any(Option::is_none) {
            return Ok(None);
        }

        let payload = group
            .payloads
            .iter()
            .map(|part| part.as_deref().unwrap_or(""))
            .collect::<String>();
        let fill = group.fill_bits;
        self.groups.remove(&key);

        decode_payload(&payload, fill, received_at_ms).map(Some)
    }

    pub fn discard_stale(&mut self, now_ms: u64, stale_after_ms: u64) {
        self.groups
            .retain(|_, group| now_ms.saturating_sub(group.updated_at_ms) <= stale_after_ms);
    }

    pub fn pending_groups(&self) -> usize {
        self.groups.len()
    }
}

impl AisTarget {
    pub fn is_stale(&self, now_ms: u64, stale_after_ms: u64) -> bool {
        now_ms.saturating_sub(self.received_at_ms) > stale_after_ms
    }

    pub fn has_valid_motion(&self) -> bool {
        self.latitude.is_some()
            && self.longitude.is_some()
            && self.sog_knots.is_some()
            && self.cog_deg.is_some()
    }
}

fn sixbit(c: u8) -> Option<u8> {
    let mut v = c.checked_sub(48)?;
    if v > 40 {
        v = v.checked_sub(8)?;
    }
    (v < 64).then_some(v)
}

fn payload_bits(payload: &str, fill_bits: u8) -> Result<Vec<bool>, String> {
    if fill_bits > 5 {
        return Err("invalid AIS fill bits".into());
    }
    let mut bits = Vec::with_capacity(payload.len() * 6);
    for c in payload.bytes() {
        let v = sixbit(c).ok_or("invalid AIS payload character")?;
        for shift in (0..6).rev() {
            bits.push(((v >> shift) & 1) != 0);
        }
    }
    for _ in 0..fill_bits {
        bits.pop();
    }
    Ok(bits)
}

fn ubits(bits: &[bool], start: usize, len: usize) -> Option<u64> {
    let mut out = 0_u64;
    for i in start..start + len {
        out = (out << 1) | (*bits.get(i)? as u64);
    }
    Some(out)
}

fn sbits(bits: &[bool], start: usize, len: usize) -> Option<i64> {
    let u = ubits(bits, start, len)?;
    let sign = 1_u64 << (len - 1);
    if u & sign != 0 {
        Some((u as i64) - ((1_u64 << len) as i64))
    } else {
        Some(u as i64)
    }
}

fn decode_payload(payload: &str, fill_bits: u8, received_at_ms: u64) -> Result<AisMessage, String> {
    let bits = payload_bits(payload, fill_bits)?;
    let message_type = ubits(&bits, 0, 6).ok_or("missing message type")? as u8;
    let mmsi = ubits(&bits, 8, 30).map(|v| v as u32);

    match message_type {
        1 | 2 | 3 => {
            let mmsi = mmsi.ok_or("missing MMSI")?;
            let nav = ubits(&bits, 38, 4).map(|v| v as u8);
            let sog_raw = ubits(&bits, 50, 10).unwrap_or(1023);
            let lon_raw = sbits(&bits, 61, 28).ok_or("missing longitude")?;
            let lat_raw = sbits(&bits, 89, 27).ok_or("missing latitude")?;
            let cog_raw = ubits(&bits, 116, 12).unwrap_or(3600);
            let hdg_raw = ubits(&bits, 128, 9).unwrap_or(511);

            let longitude = (lon_raw.abs() <= 108_600_000).then_some(lon_raw as f64 / 600_000.0);
            let latitude = (lat_raw.abs() <= 54_600_000).then_some(lat_raw as f64 / 600_000.0);

            Ok(AisMessage::PositionReport(AisTarget {
                mmsi,
                latitude,
                longitude,
                sog_knots: (sog_raw < 1023).then_some(sog_raw as f32 / 10.0),
                cog_deg: (cog_raw < 3600).then_some(cog_raw as f32 / 10.0),
                heading_deg: (hdg_raw < 511).then_some(hdg_raw as u16),
                navigation_status: nav,
                received_at_ms,
            }))
        }
        _ => Ok(AisMessage::Unsupported { message_type, mmsi }),
    }
}

pub fn parse_vdm_vdo(sentence: &str, received_at_ms: u64) -> Result<AisMessage, String> {
    let body = sentence.trim().trim_start_matches('!');
    let body = body.split('*').next().unwrap_or(body);
    let f: Vec<&str> = body.split(',').collect();

    if f.len() < 7 {
        return Err("short AIS sentence".into());
    }
    if !f[0].ends_with("VDM") && !f[0].ends_with("VDO") {
        return Err("not AIS VDM/VDO".into());
    }
    if f[1] != "1" {
        return Err("multi-fragment AIS requires AisAssembler".into());
    }

    let fill: u8 = f[6].parse().map_err(|_| "invalid fill bits")?;
    decode_payload(f[5], fill, received_at_ms)
}

pub fn cpa_tcpa_nm(
    own_lat: f64,
    own_lon: f64,
    own_sog_kn: f64,
    own_cog_deg: f64,
    tgt_lat: f64,
    tgt_lon: f64,
    tgt_sog_kn: f64,
    tgt_cog_deg: f64,
) -> Option<(f64, f64)> {
    let mean_lat = ((own_lat + tgt_lat) * 0.5).to_radians();
    let dx_nm = (tgt_lon - own_lon) * 60.0 * mean_lat.cos();
    let dy_nm = (tgt_lat - own_lat) * 60.0;
    let velocity = |speed: f64, course: f64| {
        let radians = course.to_radians();
        (speed * radians.sin(), speed * radians.cos())
    };
    let (ovx, ovy) = velocity(own_sog_kn, own_cog_deg);
    let (tvx, tvy) = velocity(tgt_sog_kn, tgt_cog_deg);
    let rvx = tvx - ovx;
    let rvy = tvy - ovy;
    let vv = rvx * rvx + rvy * rvy;
    if vv < 1e-9 {
        return None;
    }
    let tcpa_h = -(dx_nm * rvx + dy_nm * rvy) / vv;
    let cx = dx_nm + rvx * tcpa_h;
    let cy = dy_nm + rvy * tcpa_h;
    Some(((cx * cx + cy * cy).sqrt(), tcpa_h * 60.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_staleness() {
        let target = AisTarget {
            mmsi: 123_456_789,
            latitude: None,
            longitude: None,
            sog_knots: None,
            cog_deg: None,
            heading_deg: None,
            navigation_status: None,
            received_at_ms: 1000,
        };
        assert!(target.is_stale(5000, 3000));
    }

    #[test]
    fn parses_single_fragment_position_report() {
        let sentence = "!AIVDM,1,1,,A,15Muq?P0000G?t@E>4p@wvN20<0u,0*00";
        let message = parse_vdm_vdo(sentence, 1234).unwrap();
        match message {
            AisMessage::PositionReport(target) => {
                assert!(target.mmsi > 0);
                assert_eq!(target.received_at_ms, 1234);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn assembler_rejects_fragment_without_sequence_id() {
        let mut assembler = AisAssembler::default();
        let result = assembler.push(
            "!AIVDM,2,1,,A,55NBsi02;R@4L@E>221@E=B1HE=<Dh0000000016?4pN?88888888888880,0*00",
            1,
        );
        assert!(result.is_err());
    }

    #[test]
    fn assembler_discards_stale_groups() {
        let mut assembler = AisAssembler::default();
        let _ = assembler.push(
            "!AIVDM,2,1,7,A,55NBsi02;R@4L@E>221@E=B1HE=<Dh0000000016?4pN?88888888888880,0*00",
            1000,
        );
        assert_eq!(assembler.pending_groups(), 1);
        assembler.discard_stale(5000, 3000);
        assert_eq!(assembler.pending_groups(), 0);
    }
}
