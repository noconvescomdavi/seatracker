#[derive(Debug, Clone, PartialEq)]
pub enum NmeaMessage {
    Rmc { lat: f64, lon: f64, sog_knots: f32, cog_deg: f32, valid: bool },
    Gga { lat: f64, lon: f64, fix_quality: u8, satellites: u8, hdop: Option<f32> },
    Vtg { cog_deg: Option<f32>, sog_knots: Option<f32> },
    Hdt { heading_true: f32 },
    Unsupported(String),
}

pub fn validate_checksum(sentence: &str) -> bool {
    let s = sentence.trim();
    let Some(star) = s.rfind('*') else { return false };
    let body = s.strip_prefix('$').unwrap_or(s);
    let payload = &body[..star.saturating_sub(if s.starts_with('$') { 1 } else { 0 })];
    let expected = u8::from_str_radix(&s[star+1..].chars().take(2).collect::<String>(), 16);
    let Ok(expected) = expected else { return false };
    payload.as_bytes().iter().fold(0u8, |acc,b| acc ^ b) == expected
}

fn coord(value: &str, hemi: &str) -> Option<f64> {
    if value.is_empty() { return None; }
    let raw: f64 = value.parse().ok()?;
    let deg = (raw / 100.0).floor();
    let min = raw - deg * 100.0;
    let mut dec = deg + min / 60.0;
    if matches!(hemi, "S" | "W") { dec = -dec; }
    Some(dec)
}

pub fn parse(sentence: &str) -> Result<NmeaMessage, String> {
    let s = sentence.trim();
    if !validate_checksum(s) { return Err("invalid checksum".into()); }
    let star = s.rfind('*').ok_or("missing checksum")?;
    let fields: Vec<&str> = s[1..star].split(',').collect();
    let kind = fields.get(0).ok_or("empty sentence")?;
    let msg = &kind[kind.len().saturating_sub(3)..];
    match msg {
        "RMC" => {
            let lat = coord(fields.get(3).copied().unwrap_or(""), fields.get(4).copied().unwrap_or("")).ok_or("bad latitude")?;
            let lon = coord(fields.get(5).copied().unwrap_or(""), fields.get(6).copied().unwrap_or("")).ok_or("bad longitude")?;
            Ok(NmeaMessage::Rmc {
                lat, lon,
                sog_knots: fields.get(7).and_then(|v| v.parse().ok()).unwrap_or(0.0),
                cog_deg: fields.get(8).and_then(|v| v.parse().ok()).unwrap_or(0.0),
                valid: fields.get(2).copied() == Some("A"),
            })
        }
        "GGA" => {
            let lat = coord(fields.get(2).copied().unwrap_or(""), fields.get(3).copied().unwrap_or("")).ok_or("bad latitude")?;
            let lon = coord(fields.get(4).copied().unwrap_or(""), fields.get(5).copied().unwrap_or("")).ok_or("bad longitude")?;
            Ok(NmeaMessage::Gga {
                lat, lon,
                fix_quality: fields.get(6).and_then(|v| v.parse().ok()).unwrap_or(0),
                satellites: fields.get(7).and_then(|v| v.parse().ok()).unwrap_or(0),
                hdop: fields.get(8).and_then(|v| v.parse().ok()),
            })
        }
        "VTG" => Ok(NmeaMessage::Vtg {
            cog_deg: fields.get(1).and_then(|v| v.parse().ok()),
            sog_knots: fields.get(5).and_then(|v| v.parse().ok()),
        }),
        "HDT" => Ok(NmeaMessage::Hdt {
            heading_true: fields.get(1).and_then(|v| v.parse().ok()).ok_or("bad heading")?,
        }),
        _ => Ok(NmeaMessage::Unsupported(msg.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_rmc() {
        let s = "$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A";
        match parse(s).unwrap() {
            NmeaMessage::Rmc { valid, sog_knots, .. } => { assert!(valid); assert!((sog_knots-22.4).abs()<0.01); }
            _ => panic!()
        }
    }
}
