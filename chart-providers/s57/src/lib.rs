use seatracker_charts::{ChartProvider, ProviderCapabilities, SeaChartModel};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct S57Provider;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct S57Probe {
    pub looks_like_iso8211: bool,
    pub record_length: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Iso8211Field {
    pub tag: String,
    pub length: usize,
    pub position: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Iso8211Record {
    pub record_length: usize,
    pub base_address: usize,
    pub fields: Vec<Iso8211Field>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct S57Inventory {
    pub records: usize,
    pub tags: BTreeMap<String, usize>,
    pub truncated: bool,
}

impl S57Provider {
    pub fn probe(bytes: &[u8]) -> S57Probe {
        if bytes.len() < 24 {
            return S57Probe {
                looks_like_iso8211: false,
                record_length: None,
            };
        }
        let len = std::str::from_utf8(&bytes[0..5])
            .ok()
            .and_then(|s| s.parse::<usize>().ok());
        let base = std::str::from_utf8(&bytes[12..17])
            .ok()
            .and_then(|s| s.parse::<usize>().ok());
        let looks = matches!((len, base), (Some(record_len), Some(base_addr))
            if record_len >= 24 && base_addr >= 24 && base_addr <= record_len);
        S57Probe {
            looks_like_iso8211: looks,
            record_length: len,
        }
    }

    pub fn parse_record(bytes: &[u8]) -> Result<Iso8211Record, String> {
        if bytes.len() < 24 {
            return Err("ISO8211 leader shorter than 24 bytes".into());
        }
        let parse_num = |range: std::ops::Range<usize>| -> Result<usize, String> {
            let value = std::str::from_utf8(&bytes[range]).map_err(|_| "non ASCII leader")?;
            value
                .parse::<usize>()
                .map_err(|_| "invalid numeric leader field".into())
        };
        let record_length = parse_num(0..5)?;
        let base_address = parse_num(12..17)?;
        if record_length < 24 || record_length > bytes.len() || base_address > record_length {
            return Err("ISO8211 record bounds invalid".into());
        }

        let field_length_digits = (bytes[20] as char)
            .to_digit(10)
            .ok_or("invalid field length digit count")? as usize;
        let field_position_digits = (bytes[21] as char)
            .to_digit(10)
            .ok_or("invalid field position digit count")?
            as usize;
        let field_tag_digits = (bytes[23] as char)
            .to_digit(10)
            .ok_or("invalid field tag digit count")? as usize;
        let entry_len = field_tag_digits + field_length_digits + field_position_digits;
        if entry_len == 0 || base_address < 25 {
            return Err("invalid ISO8211 directory geometry".into());
        }

        let directory_end = base_address - 1;
        if directory_end > bytes.len() || bytes[directory_end] != 0x1e {
            return Err("ISO8211 directory terminator missing".into());
        }

        let mut fields = Vec::new();
        let mut offset = 24;
        while offset + entry_len <= directory_end {
            let tag = std::str::from_utf8(&bytes[offset..offset + field_tag_digits])
                .map_err(|_| "invalid field tag")?
                .trim()
                .to_string();
            let len_start = offset + field_tag_digits;
            let pos_start = len_start + field_length_digits;
            let length = std::str::from_utf8(&bytes[len_start..pos_start])
                .map_err(|_| "invalid field length")?
                .parse::<usize>()
                .map_err(|_| "invalid field length")?;
            let position =
                std::str::from_utf8(&bytes[pos_start..pos_start + field_position_digits])
                    .map_err(|_| "invalid field position")?
                    .parse::<usize>()
                    .map_err(|_| "invalid field position")?;

            let field_start = base_address
                .checked_add(position)
                .ok_or("field start overflow")?;
            let field_end = field_start
                .checked_add(length)
                .ok_or("field end overflow")?;
            if field_end > record_length {
                return Err("ISO8211 field outside record".into());
            }
            fields.push(Iso8211Field {
                tag,
                length,
                position,
            });
            offset += entry_len;
        }

        Ok(Iso8211Record {
            record_length,
            base_address,
            fields,
        })
    }

    pub fn inventory(bytes: &[u8], max_records: usize) -> S57Inventory {
        let mut inventory = S57Inventory::default();
        let mut offset = 0usize;

        while offset < bytes.len() && inventory.records < max_records {
            let remaining = &bytes[offset..];
            let probe = Self::probe(remaining);
            let Some(record_len) = probe.record_length else {
                inventory.truncated = true;
                break;
            };
            if !probe.looks_like_iso8211 || record_len > remaining.len() {
                inventory.truncated = true;
                break;
            }
            match Self::parse_record(&remaining[..record_len]) {
                Ok(record) => {
                    inventory.records += 1;
                    for field in record.fields {
                        *inventory.tags.entry(field.tag).or_insert(0) += 1;
                    }
                }
                Err(_) => {
                    inventory.truncated = true;
                    break;
                }
            }
            offset = match offset.checked_add(record_len) {
                Some(next) => next,
                None => {
                    inventory.truncated = true;
                    break;
                }
            };
        }
        if offset < bytes.len() && inventory.records >= max_records {
            inventory.truncated = true;
        }
        inventory
    }

    pub fn structural_model(source: &str, bytes: &[u8]) -> SeaChartModel {
        let inventory = Self::inventory(bytes, 100_000);
        let mut metadata = BTreeMap::new();
        metadata.insert("iso8211_records".into(), inventory.records.to_string());
        metadata.insert(
            "inventory_truncated".into(),
            inventory.truncated.to_string(),
        );
        for (tag, count) in inventory.tags {
            metadata.insert(format!("field.{tag}"), count.to_string());
        }
        SeaChartModel {
            chart_id: source.to_string(),
            provider: "S57Provider".into(),
            provider_version: "0.2.0".into(),
            source: source.to_string(),
            metadata,
            ..SeaChartModel::default()
        }
    }
}

impl ChartProvider for S57Provider {
    fn provider_name(&self) -> &'static str {
        "S57Provider"
    }
    fn provider_version(&self) -> &'static str {
        "0.2.0"
    }
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            raster: false,
            vector: true,
            updates: false,
            protected_content: false,
            object_info: true,
        }
    }
    fn can_open(&self, header: &[u8], extension: Option<&str>) -> bool {
        extension
            .map(|e| e.eq_ignore_ascii_case("000"))
            .unwrap_or(false)
            || Self::probe(header).looks_like_iso8211
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_exchange_set_extension() {
        assert!(S57Provider.can_open(b"", Some("000")));
    }

    #[test]
    fn rejects_tiny_unknown_buffer() {
        assert!(!S57Provider.can_open(b"abc", None));
    }

    #[test]
    fn malformed_record_is_rejected() {
        assert!(S57Provider::parse_record(b"00024bad").is_err());
    }
}
