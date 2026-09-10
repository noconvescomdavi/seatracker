mod cell;
pub use cell::*;

use seatracker_charts::{ChartProvider, ProviderCapabilities};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Default)]
pub struct Cm93Provider;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cm93Scale {
    Z,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

impl Cm93Scale {
    pub fn from_char(value: char) -> Option<Self> {
        match value.to_ascii_uppercase() {
            'Z' => Some(Self::Z),
            'A' => Some(Self::A),
            'B' => Some(Self::B),
            'C' => Some(Self::C),
            'D' => Some(Self::D),
            'E' => Some(Self::E),
            'F' => Some(Self::F),
            'G' => Some(Self::G),
            _ => None,
        }
    }

    pub fn native_scale(self) -> u32 {
        match self {
            Self::Z => 20_000_000,
            Self::A => 3_000_000,
            Self::B => 1_000_000,
            Self::C => 200_000,
            Self::D => 100_000,
            Self::E => 50_000,
            Self::F => 20_000,
            Self::G => 7_500,
        }
    }

    pub fn cell_step_thirds(self) -> i32 {
        match self {
            Self::Z => 120,
            Self::A => 60,
            Self::B => 30,
            Self::C => 12,
            Self::D => 3,
            Self::E | Self::F | Self::G => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cm93CellName {
    pub subcell: char,
    pub latitude_index: i32,
    pub longitude_index: i32,
    pub scale: Cm93Scale,
    pub compressed: bool,
}

impl Cm93CellName {
    pub fn parse(file_name: &str) -> Option<Self> {
        let mut name = file_name;
        let compressed = name.to_ascii_lowercase().ends_with(".xz");
        if compressed {
            name = name.get(..name.len().checked_sub(3)?)?;
        }

        let (stem, extension) = name.rsplit_once('.')?;
        if stem.len() != 8 || extension.len() != 1 {
            return None;
        }
        let scale = Cm93Scale::from_char(extension.chars().next()?)?;

        let subcell = stem.chars().next()?;
        if !(subcell == '0' || subcell.is_ascii_alphabetic()) {
            return None;
        }

        let mut normalized = stem.as_bytes().to_vec();
        normalized[0] = b'0';
        let numeric = std::str::from_utf8(&normalized).ok()?;
        let lat: i32 = numeric.get(0..4)?.parse().ok()?;
        let lon: i32 = numeric.get(4..8)?.parse().ok()?;

        Some(Self {
            subcell,
            latitude_index: lat,
            longitude_index: lon,
            scale,
            compressed,
        })
    }

    pub fn approximate_origin_deg(&self) -> (f64, f64) {
        let lat = (self.latitude_index as f64 - 270.0) / 3.0;
        let mut lon = self.longitude_index as f64 / 3.0;
        if lon >= 180.0 {
            lon -= 360.0;
        }
        (lat, lon)
    }

    pub fn approximate_bounds_deg(&self) -> (f64, f64, f64, f64) {
        let (lat, lon) = self.approximate_origin_deg();
        let step = self.scale.cell_step_thirds() as f64 / 3.0;
        (lat, lon, lat + step, lon + step)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cm93ObjectClass {
    pub code: String,
    pub id: u32,
    pub geometry: Option<char>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cm93Attribute {
    pub code: String,
    pub id: u32,
    pub value_type: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cm93Dictionary {
    pub object_classes: BTreeMap<u32, Cm93ObjectClass>,
    pub attributes: BTreeMap<u32, Cm93Attribute>,
}

impl Cm93Dictionary {
    pub fn parse_object_dictionary(text: &str) -> Self {
        let mut dictionary = Self::default();
        for raw_line in text.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            let fields: Vec<_> = line.split('|').map(str::trim).collect();
            if fields.len() < 3 {
                continue;
            }
            let Ok(id) = fields[1].parse::<u32>() else {
                continue;
            };
            let geometry = fields[2]
                .chars()
                .next()
                .map(|value| value.to_ascii_uppercase());
            dictionary.object_classes.insert(
                id,
                Cm93ObjectClass {
                    code: fields[0].to_string(),
                    id,
                    geometry,
                },
            );
        }
        dictionary
    }

    pub fn apply_attribute_dictionary(&mut self, text: &str) {
        for raw_line in text.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            let fields: Vec<_> = line.split('|').map(str::trim).collect();
            if fields.len() < 2 {
                continue;
            }
            let Ok(id) = fields[1].parse::<u32>() else {
                continue;
            };
            let value_type = fields
                .iter()
                .find(|field| field.starts_with('a'))
                .map(|value| value.to_string());
            self.attributes.insert(
                id,
                Cm93Attribute {
                    code: fields[0].to_string(),
                    id,
                    value_type,
                },
            );
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Cm93DatasetIndex {
    pub cells_by_scale: BTreeMap<Cm93Scale, usize>,
    pub compressed_cells: usize,
    pub object_dictionary_found: bool,
    pub attribute_dictionary_found: bool,
    pub rejected_files: usize,
}

impl Cm93DatasetIndex {
    pub fn observe_path(&mut self, path: &str) {
        let file_name = Path::new(path)
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or(path);
        let lower = file_name.to_ascii_lowercase();

        if lower == "cm93obj.dic" {
            self.object_dictionary_found = true;
            return;
        }
        if matches!(lower.as_str(), "attrlut.dic" | "cm93attr.dic") {
            self.attribute_dictionary_found = true;
            return;
        }

        if let Some(cell) = Cm93CellName::parse(file_name) {
            *self.cells_by_scale.entry(cell.scale).or_insert(0) += 1;
            if cell.compressed {
                self.compressed_cells += 1;
            }
        } else {
            self.rejected_files += 1;
        }
    }

    pub fn looks_like_dataset(&self) -> bool {
        self.object_dictionary_found && !self.cells_by_scale.is_empty()
    }

    pub fn total_cells(&self) -> usize {
        self.cells_by_scale.values().sum()
    }
}

impl ChartProvider for Cm93Provider {
    fn provider_name(&self) -> &'static str {
        "CM93Provider"
    }

    fn provider_version(&self) -> &'static str {
        "0.3.0"
    }

    fn can_open(&self, _header: &[u8], extension: Option<&str>) -> bool {
        matches!(
            extension,
            Some(ext)
                if Cm93Scale::from_char(ext.chars().next().unwrap_or_default()).is_some()
                    || ext.eq_ignore_ascii_case("cm93")
        )
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
}

pub fn supported_without_protection_bypass() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_scale_chars() {
        assert_eq!(Cm93Scale::from_char('e').unwrap().native_scale(), 50_000);
        assert_eq!(Cm93Scale::from_char('G').unwrap().native_scale(), 7_500);
    }

    #[test]
    fn parses_cell_name() {
        let cell = Cm93CellName::parse("01230360.E").unwrap();
        assert_eq!(cell.subcell, '0');
        assert_eq!(cell.latitude_index, 123);
        assert_eq!(cell.longitude_index, 360);
        assert_eq!(cell.scale, Cm93Scale::E);
    }

    #[test]
    fn parses_subcell_and_compression() {
        let cell = Cm93CellName::parse("A1230360.F.xz").unwrap();
        assert_eq!(cell.subcell, 'A');
        assert!(cell.compressed);
        assert_eq!(cell.scale, Cm93Scale::F);
    }

    #[test]
    fn parses_dictionary() {
        let mut dictionary = Cm93Dictionary::parse_object_dictionary("DEPARE|42|A\nBOYLAT|7|P\n");
        dictionary.apply_attribute_dictionary("COLOUR|1|unused|aBYTE\n");
        assert_eq!(dictionary.object_classes[&42].code, "DEPARE");
        assert_eq!(
            dictionary.attributes[&1].value_type.as_deref(),
            Some("aBYTE")
        );
    }

    #[test]
    fn dataset_index_detects_cm93_layout() {
        let mut index = Cm93DatasetIndex::default();
        index.observe_path("/charts/CM93OBJ.DIC");
        index.observe_path("/charts/00300060/E/01230360.E");
        index.observe_path("/charts/00300060/F/A1230360.F.xz");
        assert!(index.looks_like_dataset());
        assert_eq!(index.total_cells(), 2);
        assert_eq!(index.compressed_cells, 1);
    }
}
