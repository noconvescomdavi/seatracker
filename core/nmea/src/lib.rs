use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConnectionKind {
    Serial,
    TcpClient,
    TcpServer,
    Udp,
    Bluetooth,
    UsbSerial,
    Replay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionConfig {
    pub id: String,
    pub kind: ConnectionKind,
    pub endpoint: String,
    pub enabled: bool,
    pub priority: u8,
}

#[derive(Debug, Clone, Default)]
pub struct NmeaSourceStats {
    pub received: u64,
    pub valid: u64,
    pub invalid: u64,
    pub last_received_at_ms: Option<u64>,
}

#[derive(Debug, Default)]
pub struct NmeaRouter {
    sources: BTreeMap<String, ConnectionConfig>,
    stats: BTreeMap<String, NmeaSourceStats>,
}

impl NmeaRouter {
    pub fn add_source(&mut self, config: ConnectionConfig) -> Result<(), String> {
        if config.id.trim().is_empty() {
            return Err("connection id cannot be empty".into());
        }
        self.sources.insert(config.id.clone(), config);
        Ok(())
    }

    pub fn remove_source(&mut self, id: &str) {
        self.sources.remove(id);
        self.stats.remove(id);
    }

    pub fn sources(&self) -> impl Iterator<Item = &ConnectionConfig> {
        self.sources.values()
    }

    pub fn stats(&self, id: &str) -> Option<&NmeaSourceStats> {
        self.stats.get(id)
    }

    pub fn ingest(
        &mut self,
        source_id: &str,
        sentence: &str,
        received_at_ms: u64,
    ) -> Result<NmeaMessage, String> {
        let config = self
            .sources
            .get(source_id)
            .ok_or_else(|| format!("unknown NMEA source: {source_id}"))?;
        if !config.enabled {
            return Err(format!("NMEA source disabled: {source_id}"));
        }

        let stats = self.stats.entry(source_id.to_string()).or_default();
        stats.received = stats.received.saturating_add(1);
        stats.last_received_at_ms = Some(received_at_ms);

        match parse(sentence) {
            Ok(message) => {
                stats.valid = stats.valid.saturating_add(1);
                Ok(message)
            }
            Err(error) => {
                stats.invalid = stats.invalid.saturating_add(1);
                Err(error)
            }
        }
    }

    pub fn preferred_source(&self) -> Option<&ConnectionConfig> {
        self.sources
            .values()
            .filter(|source| source.enabled)
            .min_by_key(|source| source.priority)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NmeaMessage {
    Rmc {
        lat: f64,
        lon: f64,
        sog_knots: f32,
        cog_deg: f32,
        valid: bool,
    },
    Gga {
        lat: f64,
        lon: f64,
        fix_quality: u8,
        satellites: u8,
        hdop: Option<f32>,
    },
    Gll {
        lat: f64,
        lon: f64,
        valid: bool,
    },
    Vtg {
        cog_deg: Option<f32>,
        sog_knots: Option<f32>,
    },
    Hdt {
        heading_true: f32,
    },
    Hdg {
        heading_magnetic: f32,
        deviation: Option<f32>,
        variation: Option<f32>,
    },
    Gsa {
        fix_type: u8,
        hdop: Option<f32>,
        vdop: Option<f32>,
        pdop: Option<f32>,
    },
    Gsv {
        satellites_in_view: Option<u8>,
    },
    Dbt {
        depth_m: f32,
    },
    Dpt {
        depth_m: f32,
        offset_m: Option<f32>,
    },
    Mtw {
        temperature_c: f32,
    },
    Mwv {
        angle_deg: f32,
        speed_knots: f32,
        relative: bool,
        valid: bool,
    },
    Mwd {
        direction_true_deg: f32,
        speed_knots: f32,
    },
    Rsa {
        starboard_rudder_deg: Option<f32>,
        port_rudder_deg: Option<f32>,
    },
    Vlw {
        total_nm: Option<f32>,
        trip_nm: Option<f32>,
    },
    Unsupported(String),
}

pub fn encode_sentence(body: &str) -> String {
    let checksum = body.as_bytes().iter().fold(0u8, |acc, byte| acc ^ byte);
    format!("${body}*{checksum:02X}")
}

pub fn encode_xte_nm(error_nm: f64, steer_right: bool) -> String {
    let direction = if steer_right { "R" } else { "L" };
    encode_sentence(&format!("GPXTE,A,A,{:.3},{direction},N", error_nm.abs()))
}

pub fn encode_rmb(
    xte_nm: f64,
    steer_right: bool,
    origin_id: &str,
    destination_id: &str,
    destination_lat: f64,
    destination_lon: f64,
    range_nm: f64,
    bearing_deg: f64,
    closing_velocity_knots: f64,
    arrival: bool,
) -> String {
    let direction = if steer_right { "R" } else { "L" };
    let (lat_value, lat_hemi) = encode_coord(destination_lat, true);
    let (lon_value, lon_hemi) = encode_coord(destination_lon, false);
    let arrival_flag = if arrival { "A" } else { "V" };
    encode_sentence(&format!(
        "GPRMB,A,{:.3},{direction},{origin_id},{destination_id},{lat_value},{lat_hemi},{lon_value},{lon_hemi},{:.3},{:.1},{:.2},{arrival_flag}",
        xte_nm.abs(),
        range_nm.max(0.0),
        bearing_deg.rem_euclid(360.0),
        closing_velocity_knots.max(0.0),
    ))
}

pub fn encode_apb(
    xte_nm: f64,
    steer_right: bool,
    bearing_origin_to_destination_deg: f64,
    bearing_present_to_destination_deg: f64,
    heading_to_steer_deg: f64,
    destination_id: &str,
    arrival_circle_entered: bool,
) -> String {
    let direction = if steer_right { "R" } else { "L" };
    let arrival = if arrival_circle_entered { "A" } else { "V" };
    encode_sentence(&format!(
        "GPAPB,A,A,{:.3},{direction},N,{arrival},V,{:.1},T,{destination_id},{:.1},T,{:.1},T",
        xte_nm.abs(),
        bearing_origin_to_destination_deg.rem_euclid(360.0),
        bearing_present_to_destination_deg.rem_euclid(360.0),
        heading_to_steer_deg.rem_euclid(360.0),
    ))
}

fn encode_coord(value: f64, latitude: bool) -> (String, &'static str) {
    let hemisphere = if latitude {
        if value < 0.0 { "S" } else { "N" }
    } else if value < 0.0 {
        "W"
    } else {
        "E"
    };
    let absolute = value.abs();
    let degrees = absolute.floor();
    let minutes = (absolute - degrees) * 60.0;
    let formatted = if latitude {
        format!("{degrees:02.0}{minutes:07.4}")
    } else {
        format!("{degrees:03.0}{minutes:07.4}")
    };
    (formatted, hemisphere)
}

pub fn validate_checksum(sentence: &str) -> bool {
    let s = sentence.trim();
    let Some(star) = s.rfind('*') else {
        return false;
    };
    let payload = s.strip_prefix('$').unwrap_or(s);
    let payload = &payload[..star.saturating_sub(if s.starts_with('$') { 1 } else { 0 })];
    let expected = u8::from_str_radix(&s[star + 1..].chars().take(2).collect::<String>(), 16);
    let Ok(expected) = expected else { return false };
    payload.as_bytes().iter().fold(0u8, |acc, b| acc ^ b) == expected
}

fn coord(value: &str, hemi: &str) -> Option<f64> {
    if value.is_empty() {
        return None;
    }
    let raw: f64 = value.parse().ok()?;
    let deg = (raw / 100.0).floor();
    let min = raw - deg * 100.0;
    let mut dec = deg + min / 60.0;
    if matches!(hemi, "S" | "W") {
        dec = -dec;
    }
    Some(dec)
}

fn signed(value: Option<&&str>, dir: Option<&&str>) -> Option<f32> {
    let mut v: f32 = value?.parse().ok()?;
    if matches!(dir.copied(), Some("W") | Some("S")) {
        v = -v;
    }
    Some(v)
}

pub fn parse(sentence: &str) -> Result<NmeaMessage, String> {
    let s = sentence.trim();
    if !validate_checksum(s) {
        return Err("invalid checksum".into());
    }
    let star = s.rfind('*').ok_or("missing checksum")?;
    let fields: Vec<&str> = s[1..star].split(',').collect();
    let kind = fields.first().ok_or("empty sentence")?;
    let msg = &kind[kind.len().saturating_sub(3)..];

    match msg {
        "RMC" => {
            let lat = coord(
                fields.get(3).copied().unwrap_or(""),
                fields.get(4).copied().unwrap_or(""),
            )
            .ok_or("bad latitude")?;
            let lon = coord(
                fields.get(5).copied().unwrap_or(""),
                fields.get(6).copied().unwrap_or(""),
            )
            .ok_or("bad longitude")?;
            Ok(NmeaMessage::Rmc {
                lat,
                lon,
                sog_knots: fields.get(7).and_then(|v| v.parse().ok()).unwrap_or(0.0),
                cog_deg: fields.get(8).and_then(|v| v.parse().ok()).unwrap_or(0.0),
                valid: fields.get(2).copied() == Some("A"),
            })
        }
        "GGA" => {
            let lat = coord(
                fields.get(2).copied().unwrap_or(""),
                fields.get(3).copied().unwrap_or(""),
            )
            .ok_or("bad latitude")?;
            let lon = coord(
                fields.get(4).copied().unwrap_or(""),
                fields.get(5).copied().unwrap_or(""),
            )
            .ok_or("bad longitude")?;
            Ok(NmeaMessage::Gga {
                lat,
                lon,
                fix_quality: fields.get(6).and_then(|v| v.parse().ok()).unwrap_or(0),
                satellites: fields.get(7).and_then(|v| v.parse().ok()).unwrap_or(0),
                hdop: fields.get(8).and_then(|v| v.parse().ok()),
            })
        }
        "GLL" => {
            let lat = coord(
                fields.get(1).copied().unwrap_or(""),
                fields.get(2).copied().unwrap_or(""),
            )
            .ok_or("bad latitude")?;
            let lon = coord(
                fields.get(3).copied().unwrap_or(""),
                fields.get(4).copied().unwrap_or(""),
            )
            .ok_or("bad longitude")?;
            Ok(NmeaMessage::Gll {
                lat,
                lon,
                valid: fields.get(6).copied() == Some("A"),
            })
        }
        "VTG" => Ok(NmeaMessage::Vtg {
            cog_deg: fields.get(1).and_then(|v| v.parse().ok()),
            sog_knots: fields.get(5).and_then(|v| v.parse().ok()),
        }),
        "HDT" => Ok(NmeaMessage::Hdt {
            heading_true: fields
                .get(1)
                .and_then(|v| v.parse().ok())
                .ok_or("bad heading")?,
        }),
        "HDG" => Ok(NmeaMessage::Hdg {
            heading_magnetic: fields
                .get(1)
                .and_then(|v| v.parse().ok())
                .ok_or("bad magnetic heading")?,
            deviation: signed(fields.get(2), fields.get(3)),
            variation: signed(fields.get(4), fields.get(5)),
        }),
        "GSA" => Ok(NmeaMessage::Gsa {
            fix_type: fields.get(2).and_then(|v| v.parse().ok()).unwrap_or(1),
            pdop: fields.get(15).and_then(|v| v.parse().ok()),
            hdop: fields.get(16).and_then(|v| v.parse().ok()),
            vdop: fields.get(17).and_then(|v| v.parse().ok()),
        }),
        "GSV" => Ok(NmeaMessage::Gsv {
            satellites_in_view: fields.get(3).and_then(|v| v.parse().ok()),
        }),
        "DBT" => {
            let depth_m = fields
                .get(3)
                .and_then(|v| v.parse().ok())
                .or_else(|| {
                    fields
                        .get(1)
                        .and_then(|v| v.parse::<f32>().ok())
                        .map(|ft| ft * 0.3048)
                })
                .ok_or("bad depth")?;
            Ok(NmeaMessage::Dbt { depth_m })
        }
        "DPT" => Ok(NmeaMessage::Dpt {
            depth_m: fields
                .get(1)
                .and_then(|v| v.parse().ok())
                .ok_or("bad depth")?,
            offset_m: fields.get(2).and_then(|v| v.parse().ok()),
        }),
        "MTW" => Ok(NmeaMessage::Mtw {
            temperature_c: fields
                .get(1)
                .and_then(|v| v.parse().ok())
                .ok_or("bad water temperature")?,
        }),
        "MWV" => {
            let unit = fields.get(4).copied().unwrap_or("N");
            let raw_speed: f32 = fields
                .get(3)
                .and_then(|v| v.parse().ok())
                .ok_or("bad wind speed")?;
            let speed_knots = match unit {
                "N" => raw_speed,
                "M" => raw_speed * 1.943_844,
                "K" => raw_speed / 1.852,
                _ => return Err("unsupported wind speed unit".into()),
            };
            Ok(NmeaMessage::Mwv {
                angle_deg: fields
                    .get(1)
                    .and_then(|v| v.parse().ok())
                    .ok_or("bad wind angle")?,
                speed_knots,
                relative: fields.get(2).copied() == Some("R"),
                valid: fields.get(5).copied() != Some("V"),
            })
        }
        "MWD" => {
            let speed_knots = fields
                .get(5)
                .and_then(|v| v.parse().ok())
                .or_else(|| {
                    fields
                        .get(7)
                        .and_then(|v| v.parse::<f32>().ok())
                        .map(|mps| mps * 1.943_844)
                })
                .ok_or("bad true wind speed")?;
            Ok(NmeaMessage::Mwd {
                direction_true_deg: fields
                    .get(1)
                    .and_then(|v| v.parse().ok())
                    .ok_or("bad true wind direction")?,
                speed_knots,
            })
        }
        "RSA" => Ok(NmeaMessage::Rsa {
            starboard_rudder_deg: fields.get(1).and_then(|v| v.parse().ok()),
            port_rudder_deg: fields.get(3).and_then(|v| v.parse().ok()),
        }),
        "VLW" => Ok(NmeaMessage::Vlw {
            total_nm: fields.get(1).and_then(|v| v.parse().ok()),
            trip_nm: fields.get(3).and_then(|v| v.parse().ok()),
        }),
        _ => Ok(NmeaMessage::Unsupported(msg.into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rmc() {
        let s = "$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A";
        match parse(s).unwrap() {
            NmeaMessage::Rmc {
                valid, sog_knots, ..
            } => {
                assert!(valid);
                assert!((sog_knots - 22.4).abs() < 0.01);
            }
            _ => panic!(),
        }
    }

    #[test]
    fn rejects_bad_checksum() {
        assert!(parse("$GPRMC,1,A,1,N,1,E,1,1,1*00").is_err());
    }

    #[test]
    fn encodes_autopilot_sentences_with_valid_checksums() {
        let xte = encode_xte_nm(0.12, true);
        assert!(validate_checksum(&xte));

        let rmb = encode_rmb(0.12, true, "A", "B", -22.9, -43.2, 3.4, 87.0, 8.0, false);
        assert!(validate_checksum(&rmb));

        let apb = encode_apb(0.12, true, 90.0, 88.0, 87.0, "B", false);
        assert!(validate_checksum(&apb));
    }

    #[test]
    fn router_tracks_sources_and_priority() {
        let mut router = NmeaRouter::default();
        router
            .add_source(ConnectionConfig {
                id: "udp".into(),
                kind: ConnectionKind::Udp,
                endpoint: "0.0.0.0:10110".into(),
                enabled: true,
                priority: 20,
            })
            .unwrap();
        router
            .add_source(ConnectionConfig {
                id: "gps".into(),
                kind: ConnectionKind::UsbSerial,
                endpoint: "/dev/ttyUSB0".into(),
                enabled: true,
                priority: 10,
            })
            .unwrap();

        assert_eq!(router.preferred_source().unwrap().id, "gps");
        assert!(
            router
                .ingest(
                    "udp",
                    "$GPRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A",
                    1000
                )
                .is_ok()
        );
        assert_eq!(router.stats("udp").unwrap().valid, 1);
    }
}
