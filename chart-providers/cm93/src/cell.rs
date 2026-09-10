use seatracker_charts::{BoundingBox, ChartObjectKind, Geometry, SeaChartModel, SeaChartObject};
use std::collections::BTreeMap;
use std::fmt;

use crate::{Cm93CellName, Cm93Dictionary};

const CM93_RADIUS_M: f64 = 6_378_388.0;
const MAX_POINTS: usize = 8_000_000;
const MAX_FEATURES: usize = 500_000;
const MAX_ATTR_BYTES: usize = 64 * 1024 * 1024;

// CM93 byte permutation used by legacy CM93/2 cells. Kept isolated here so the
// chart provider remains independent from the rest of the SeaTracker core.
const CM93_PERMUTATION: [u8; 256] = [
    0xCD, 0xEA, 0xDC, 0x48, 0x3E, 0x6D, 0xCA, 0x7B, 0x52, 0xE1, 0xA4, 0x8E, 0xAB, 0x05, 0xA7, 0x97,
    0xB9, 0x60, 0x39, 0x85, 0x7C, 0x56, 0x7A, 0xBA, 0x68, 0x6E, 0xF5, 0x5D, 0x02, 0x4E, 0x0F, 0xA1,
    0x27, 0x24, 0x41, 0x34, 0x00, 0x5A, 0xFE, 0xCB, 0xD0, 0xFA, 0xF8, 0x6C, 0x74, 0x96, 0x9E, 0x0E,
    0xC2, 0x49, 0xE3, 0xE5, 0xC0, 0x3B, 0x59, 0x18, 0xA9, 0x86, 0x8F, 0x30, 0xC3, 0xA8, 0x22, 0x0A,
    0x14, 0x1A, 0xB2, 0xC9, 0xC7, 0xED, 0xAA, 0x29, 0x94, 0x75, 0x0D, 0xAC, 0x0C, 0xF4, 0xBB, 0xC5,
    0x3F, 0xFD, 0xD9, 0x9C, 0x4F, 0xD5, 0x84, 0x1E, 0xB1, 0x81, 0x69, 0xB4, 0x09, 0xB8, 0x3C, 0xAF,
    0xA3, 0x08, 0xBF, 0xE0, 0x9A, 0xD7, 0xF7, 0x8C, 0x67, 0x66, 0xAE, 0xD4, 0x4C, 0xA5, 0xEC, 0xF9,
    0xB6, 0x64, 0x78, 0x06, 0x5B, 0x9B, 0xF2, 0x99, 0xCE, 0xDB, 0x53, 0x55, 0x65, 0x8D, 0x07, 0x33,
    0x04, 0x37, 0x92, 0x26, 0x23, 0xB5, 0x58, 0xDA, 0x2F, 0xB3, 0x40, 0x5E, 0x7F, 0x4B, 0x62, 0x80,
    0xE4, 0x6F, 0x73, 0x1D, 0xDF, 0x17, 0xCC, 0x28, 0x25, 0x2D, 0xEE, 0x3A, 0x98, 0xE2, 0x01, 0xEB,
    0xDD, 0xBC, 0x90, 0xB0, 0xFC, 0x95, 0x76, 0x93, 0x46, 0x57, 0x2C, 0x2B, 0x50, 0x11, 0x0B, 0xC1,
    0xF0, 0xE7, 0xD6, 0x21, 0x31, 0xDE, 0xFF, 0xD8, 0x12, 0xA6, 0x4D, 0x8A, 0x13, 0x43, 0x45, 0x38,
    0xD2, 0x87, 0xA0, 0xEF, 0x82, 0xF1, 0x47, 0x89, 0x6A, 0xC8, 0x54, 0x1B, 0x16, 0x7E, 0x79, 0xBD,
    0x6B, 0x91, 0xA2, 0x71, 0x36, 0xB7, 0x03, 0x3D, 0x72, 0xC6, 0x44, 0x8B, 0xCF, 0x15, 0x9F, 0x32,
    0xC4, 0x77, 0x83, 0x63, 0x20, 0x88, 0xF6, 0xAD, 0xF3, 0xE8, 0x4A, 0xE9, 0x35, 0x1C, 0x5F, 0x19,
    0x1F, 0x7D, 0x70, 0xFB, 0xD1, 0x51, 0x10, 0xD3, 0x2E, 0x61, 0x9D, 0x5C, 0x2A, 0x42, 0xBE, 0xE6,
];

#[derive(Debug, Clone, PartialEq)]
pub enum Cm93CellError {
    Truncated,
    InvalidProlog,
    InvalidHeader(&'static str),
    InvalidReference,
    LimitExceeded(&'static str),
    InvalidFeature,
}

impl fmt::Display for Cm93CellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => write!(f, "CM93 cell is truncated"),
            Self::InvalidProlog => write!(f, "CM93 prolog/file length validation failed"),
            Self::InvalidHeader(field) => write!(f, "invalid CM93 header field: {field}"),
            Self::InvalidReference => {
                write!(f, "CM93 feature references an invalid geometry index")
            }
            Self::LimitExceeded(name) => write!(f, "CM93 safety limit exceeded: {name}"),
            Self::InvalidFeature => write!(f, "CM93 feature record is malformed"),
        }
    }
}

impl std::error::Error for Cm93CellError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cm93Point {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cm93Point3d {
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cm93Header {
    pub lon_min: f64,
    pub lat_min: f64,
    pub lon_max: f64,
    pub lat_max: f64,
    pub easting_min: f64,
    pub northing_min: f64,
    pub easting_max: f64,
    pub northing_max: f64,
    pub vector_records: usize,
    pub vector_points: usize,
    pub point3d_records: usize,
    pub point3d_points: usize,
    pub point2d_records: usize,
    pub feature_records: usize,
    pub related_object_pointers: usize,
    pub attribute_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cm93SegmentRef {
    pub edge_index: usize,
    pub usage: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cm93FeatureGeometry {
    None,
    Point(usize),
    Soundings(usize),
    Segments(Vec<Cm93SegmentRef>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cm93Feature {
    pub object_type: u8,
    pub geometry_flags: u8,
    pub geometry: Cm93FeatureGeometry,
    pub related_indices: Vec<usize>,
    pub attribute_count: u8,
    pub attributes_raw: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cm93Cell {
    pub header: Cm93Header,
    pub edges: Vec<Vec<Cm93Point>>,
    pub soundings: Vec<Vec<Cm93Point3d>>,
    pub points: Vec<Cm93Point>,
    pub features: Vec<Cm93Feature>,
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
    decode: [u8; 256],
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        let mut decode = [0u8; 256];
        for (plain, mapped) in CM93_PERMUTATION.iter().copied().enumerate() {
            let encoded = mapped ^ 8;
            decode[encoded as usize] = plain as u8;
        }
        Self {
            bytes,
            pos: 0,
            decode,
        }
    }

    fn position(&self) -> usize {
        self.pos
    }

    fn seek(&mut self, position: usize) -> Result<(), Cm93CellError> {
        if position > self.bytes.len() {
            return Err(Cm93CellError::Truncated);
        }
        self.pos = position;
        Ok(())
    }

    fn decoded(&mut self, count: usize) -> Result<Vec<u8>, Cm93CellError> {
        let end = self
            .pos
            .checked_add(count)
            .ok_or(Cm93CellError::Truncated)?;
        let raw = self
            .bytes
            .get(self.pos..end)
            .ok_or(Cm93CellError::Truncated)?;
        self.pos = end;
        Ok(raw
            .iter()
            .map(|value| self.decode[*value as usize])
            .collect())
    }

    fn u8(&mut self) -> Result<u8, Cm93CellError> {
        Ok(self.decoded(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, Cm93CellError> {
        let b = self.decoded(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    fn i32(&mut self) -> Result<i32, Cm93CellError> {
        let b = self.decoded(4)?;
        Ok(i32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn f64(&mut self) -> Result<f64, Cm93CellError> {
        let b = self.decoded(8)?;
        Ok(f64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }
}

pub fn decode_cm93_cell(bytes: &[u8]) -> Result<Cm93Cell, Cm93CellError> {
    if bytes.len() < 138 {
        return Err(Cm93CellError::Truncated);
    }
    let mut r = Reader::new(bytes);

    let prolog_header_len = r.u16()? as usize;
    let table1_len = r.i32()?;
    let table2_len = r.i32()?;
    if table1_len < 0 || table2_len < 0 {
        return Err(Cm93CellError::InvalidProlog);
    }
    let expected = prolog_header_len
        .checked_add(table1_len as usize)
        .and_then(|v| v.checked_add(table2_len as usize))
        .ok_or(Cm93CellError::InvalidProlog)?;
    if expected != bytes.len() || prolog_header_len < 138 {
        return Err(Cm93CellError::InvalidProlog);
    }

    let header = read_header(&mut r)?;
    if r.position() != 138 {
        return Err(Cm93CellError::InvalidHeader("size"));
    }

    let mut edges = Vec::with_capacity(header.vector_records);
    let mut edge_points = 0usize;
    for _ in 0..header.vector_records {
        let count = r.u16()? as usize;
        edge_points = edge_points
            .checked_add(count)
            .ok_or(Cm93CellError::LimitExceeded("vector points"))?;
        if edge_points > MAX_POINTS {
            return Err(Cm93CellError::LimitExceeded("vector points"));
        }
        let mut edge = Vec::with_capacity(count);
        for _ in 0..count {
            edge.push(Cm93Point {
                x: r.u16()?,
                y: r.u16()?,
            });
        }
        edges.push(edge);
    }
    if header.vector_points != 0 && edge_points != header.vector_points {
        return Err(Cm93CellError::InvalidHeader("vector point count"));
    }

    let mut soundings = Vec::with_capacity(header.point3d_records);
    let mut sounding_points = 0usize;
    for _ in 0..header.point3d_records {
        let count = r.u16()? as usize;
        sounding_points = sounding_points
            .checked_add(count)
            .ok_or(Cm93CellError::LimitExceeded("3d points"))?;
        if sounding_points > MAX_POINTS {
            return Err(Cm93CellError::LimitExceeded("3d points"));
        }
        let mut record = Vec::with_capacity(count);
        for _ in 0..count {
            record.push(Cm93Point3d {
                x: r.u16()?,
                y: r.u16()?,
                z: r.u16()?,
            });
        }
        soundings.push(record);
    }
    if header.point3d_points != 0 && sounding_points != header.point3d_points {
        return Err(Cm93CellError::InvalidHeader("3d point count"));
    }

    if header.point2d_records > MAX_POINTS {
        return Err(Cm93CellError::LimitExceeded("2d points"));
    }
    let mut points = Vec::with_capacity(header.point2d_records);
    for _ in 0..header.point2d_records {
        points.push(Cm93Point {
            x: r.u16()?,
            y: r.u16()?,
        });
    }

    if header.feature_records > MAX_FEATURES {
        return Err(Cm93CellError::LimitExceeded("features"));
    }
    let mut features = Vec::with_capacity(header.feature_records);
    let mut total_attribute_bytes = 0usize;
    for _ in 0..header.feature_records {
        let start = r.position();
        let object_type = r.u8()?;
        let geometry_flags = r.u8()?;
        let record_len = r.u16()? as usize;
        if record_len < 4 {
            return Err(Cm93CellError::InvalidFeature);
        }
        let end = start
            .checked_add(record_len)
            .ok_or(Cm93CellError::InvalidFeature)?;
        if end > bytes.len() {
            return Err(Cm93CellError::Truncated);
        }

        let geometry = match geometry_flags & 0x0f {
            1 => {
                let index = r.u16()? as usize;
                if index >= points.len() {
                    return Err(Cm93CellError::InvalidReference);
                }
                Cm93FeatureGeometry::Point(index)
            }
            2 | 4 => {
                let count = r.u16()? as usize;
                let mut refs = Vec::with_capacity(count);
                for _ in 0..count {
                    let packed = r.u16()?;
                    let edge_index = (packed & 0x1fff) as usize;
                    if edge_index >= edges.len() {
                        return Err(Cm93CellError::InvalidReference);
                    }
                    refs.push(Cm93SegmentRef {
                        edge_index,
                        usage: (packed >> 13) as u8,
                    });
                }
                Cm93FeatureGeometry::Segments(refs)
            }
            8 => {
                let index = r.u16()? as usize;
                if index >= soundings.len() {
                    return Err(Cm93CellError::InvalidReference);
                }
                Cm93FeatureGeometry::Soundings(index)
            }
            _ => Cm93FeatureGeometry::None,
        };

        let mut related_indices = Vec::new();
        if geometry_flags & 0x10 != 0 {
            let count = r.u8()? as usize;
            related_indices.reserve(count);
            for _ in 0..count {
                related_indices.push(r.u16()? as usize);
            }
        }
        if geometry_flags & 0x20 != 0 {
            let _ = r.u16()?;
        }

        let mut attribute_count = 0u8;
        let mut attributes_raw = Vec::new();
        if geometry_flags & 0x80 != 0 {
            attribute_count = r.u8()?;
            if r.position() > end {
                return Err(Cm93CellError::InvalidFeature);
            }
            let attr_len = end - r.position();
            total_attribute_bytes = total_attribute_bytes
                .checked_add(attr_len)
                .ok_or(Cm93CellError::LimitExceeded("attribute bytes"))?;
            if total_attribute_bytes > MAX_ATTR_BYTES {
                return Err(Cm93CellError::LimitExceeded("attribute bytes"));
            }
            attributes_raw = r.decoded(attr_len)?;
        }

        if r.position() > end {
            return Err(Cm93CellError::InvalidFeature);
        }
        if r.position() < end {
            r.seek(end)?;
        }
        features.push(Cm93Feature {
            object_type,
            geometry_flags,
            geometry,
            related_indices,
            attribute_count,
            attributes_raw,
        });
    }

    Ok(Cm93Cell {
        header,
        edges,
        soundings,
        points,
        features,
    })
}

fn read_header(r: &mut Reader<'_>) -> Result<Cm93Header, Cm93CellError> {
    let lon_min = r.f64()?;
    let lat_min = r.f64()?;
    let lon_max = r.f64()?;
    let lat_max = r.f64()?;
    let easting_min = r.f64()?;
    let northing_min = r.f64()?;
    let easting_max = r.f64()?;
    let northing_max = r.f64()?;
    for (name, value) in [
        ("lon_min", lon_min),
        ("lat_min", lat_min),
        ("lon_max", lon_max),
        ("lat_max", lat_max),
        ("easting_min", easting_min),
        ("northing_min", northing_min),
        ("easting_max", easting_max),
        ("northing_max", northing_max),
    ] {
        if !value.is_finite() {
            return Err(Cm93CellError::InvalidHeader(name));
        }
    }
    let vector_records = r.u16()? as usize;
    let vector_points = nonnegative(r.i32()?, "vector points")?;
    let _m46 = r.i32()?;
    let _m4a = r.i32()?;
    let point3d_records = r.u16()? as usize;
    let point3d_points = nonnegative(r.i32()?, "3d points")?;
    let _m54 = r.i32()?;
    let point2d_records = r.u16()? as usize;
    let _m5a = r.u16()?;
    let _m5c = r.u16()?;
    let feature_records = r.u16()? as usize;
    let _m60 = r.i32()?;
    let _m64 = r.i32()?;
    let _m68 = r.u16()?;
    let _m6a = r.u16()?;
    let _m6c = r.u16()?;
    let related_object_pointers = nonnegative(r.i32()?, "related pointers")?;
    let _m72 = r.i32()?;
    let _m76 = r.u16()?;
    let attribute_bytes = nonnegative(r.i32()?, "attribute bytes")?;
    let _m7c = r.i32()?;

    if vector_points > MAX_POINTS || point3d_points > MAX_POINTS || point2d_records > MAX_POINTS {
        return Err(Cm93CellError::LimitExceeded("header point counts"));
    }
    if feature_records > MAX_FEATURES {
        return Err(Cm93CellError::LimitExceeded("feature records"));
    }
    if attribute_bytes > MAX_ATTR_BYTES {
        return Err(Cm93CellError::LimitExceeded("attribute bytes"));
    }
    Ok(Cm93Header {
        lon_min,
        lat_min,
        lon_max,
        lat_max,
        easting_min,
        northing_min,
        easting_max,
        northing_max,
        vector_records,
        vector_points,
        point3d_records,
        point3d_points,
        point2d_records,
        feature_records,
        related_object_pointers,
        attribute_bytes,
    })
}

fn nonnegative(value: i32, field: &'static str) -> Result<usize, Cm93CellError> {
    if value < 0 {
        Err(Cm93CellError::InvalidHeader(field))
    } else {
        Ok(value as usize)
    }
}

impl Cm93Cell {
    pub fn bounds(&self) -> BoundingBox {
        BoundingBox {
            min_lat: self.header.lat_min,
            min_lon: normalize_lon(self.header.lon_min),
            max_lat: self.header.lat_max,
            max_lon: normalize_lon(self.header.lon_max),
        }
    }

    pub fn transform(&self, point: Cm93Point) -> [f64; 2] {
        let mut dx = self.header.easting_max - self.header.easting_min;
        if dx < 0.0 {
            dx += CM93_RADIUS_M * 2.0 * std::f64::consts::PI;
        }
        let x_rate = dx / 65535.0;
        let y_rate = (self.header.northing_max - self.header.northing_min) / 65535.0;
        let easting = point.x as f64 * x_rate + self.header.easting_min;
        let northing = point.y as f64 * y_rate + self.header.northing_min;
        let lat = (2.0 * (northing / CM93_RADIUS_M).exp().atan() - std::f64::consts::FRAC_PI_2)
            .to_degrees();
        let lon = normalize_lon((easting / CM93_RADIUS_M).to_degrees());
        [lon, lat]
    }

    pub fn to_sea_chart_model(
        &self,
        source: &str,
        file_name: &str,
        dictionary: &Cm93Dictionary,
    ) -> SeaChartModel {
        let scale = Cm93CellName::parse(file_name).map(|name| name.scale.native_scale());
        let mut objects = Vec::new();
        for (index, feature) in self.features.iter().enumerate() {
            let class = dictionary.object_classes.get(&(feature.object_type as u32));
            let code = class.map(|entry| entry.code.as_str()).unwrap_or("UNKNOWN");
            let kind = chart_kind(code, &feature.geometry);
            let geometry = self.feature_geometry(feature);
            let mut attributes = BTreeMap::new();
            attributes.insert("CM93_CLASS".to_string(), code.to_string());
            attributes.insert("CM93_CLASS_ID".to_string(), feature.object_type.to_string());
            attributes.insert(
                "CM93_GEOTYPE".to_string(),
                format!("0x{:02X}", feature.geometry_flags),
            );
            attributes.insert(
                "CM93_ATTR_COUNT".to_string(),
                feature.attribute_count.to_string(),
            );
            if !feature.attributes_raw.is_empty() {
                attributes.insert(
                    "CM93_ATTR_RAW_HEX".to_string(),
                    hex(&feature.attributes_raw),
                );
            }
            objects.push(SeaChartObject {
                id: format!("cm93:{file_name}:{index}"),
                kind,
                geometry,
                attributes,
                source: source.to_string(),
                edition: None,
                scale,
                zoom_level: None,
                bounds: None,
                quality: None,
                date: None,
                category: Some(code.to_string()),
                render_priority: render_priority(code),
            });
        }
        let mut metadata = BTreeMap::new();
        metadata.insert("format".to_string(), "CM93/2".to_string());
        metadata.insert("feature_count".to_string(), self.features.len().to_string());
        metadata.insert("edge_count".to_string(), self.edges.len().to_string());
        metadata.insert(
            "sounding_record_count".to_string(),
            self.soundings.len().to_string(),
        );
        SeaChartModel {
            chart_id: file_name.to_string(),
            provider: "CM93Provider".to_string(),
            provider_version: "0.3.0".to_string(),
            source: source.to_string(),
            edition: None,
            scale,
            datum: Some("CM93 International 1924 / WGS84-compatible display transform".to_string()),
            projection: Some("Mercator".to_string()),
            bounds: Some(self.bounds()),
            raster_layers: Vec::new(),
            objects,
            metadata,
        }
    }

    fn feature_geometry(&self, feature: &Cm93Feature) -> Option<Geometry> {
        match &feature.geometry {
            Cm93FeatureGeometry::None => None,
            Cm93FeatureGeometry::Point(index) => self
                .points
                .get(*index)
                .copied()
                .map(|p| Geometry::Point(self.transform(p))),
            Cm93FeatureGeometry::Soundings(index) => {
                let record = self.soundings.get(*index)?;
                Some(Geometry::MultiPoint(
                    record
                        .iter()
                        .map(|p| self.transform(Cm93Point { x: p.x, y: p.y }))
                        .collect(),
                ))
            }
            Cm93FeatureGeometry::Segments(refs) => {
                let mut vertices = Vec::new();
                for reference in refs {
                    let edge = self.edges.get(reference.edge_index)?;
                    if reference.usage & 4 == 4 {
                        vertices.extend(edge.iter().rev().copied().map(|p| self.transform(p)));
                    } else {
                        vertices.extend(edge.iter().copied().map(|p| self.transform(p)));
                    }
                    if vertices.len() >= 2 {
                        let last = vertices.len() - 1;
                        if vertices[last] == vertices[last - 1] {
                            vertices.pop();
                        }
                    }
                }
                match feature.geometry_flags & 0x0f {
                    4 => Some(Geometry::Polygon(vec![vertices])),
                    2 => Some(Geometry::LineString(vertices)),
                    _ => None,
                }
            }
        }
    }
}

fn normalize_lon(lon: f64) -> f64 {
    (lon + 180.0).rem_euclid(360.0) - 180.0
}

fn chart_kind(code: &str, geometry: &Cm93FeatureGeometry) -> ChartObjectKind {
    match code {
        "SOUNDG" => ChartObjectKind::Sounding,
        "DEPCNT" => ChartObjectKind::DepthContour,
        "DEPARE" => ChartObjectKind::DepthArea,
        "COALNE" => ChartObjectKind::Coastline,
        "LNDARE" => ChartObjectKind::LandArea,
        "LIGHTS" => ChartObjectKind::Light,
        c if c.starts_with("BOY") => ChartObjectKind::Buoy,
        c if c.starts_with("BCN") => ChartObjectKind::Beacon,
        "WRECKS" => ChartObjectKind::Wreck,
        "OBSTRN" | "UWTROC" => ChartObjectKind::Obstruction,
        "RESARE" => ChartObjectKind::RestrictedArea,
        "ACHARE" => ChartObjectKind::Anchorage,
        "FAIRWY" => ChartObjectKind::Fairway,
        "TSSLPT" | "TSEZNE" | "TSSBND" | "TSSCRS" => ChartObjectKind::TrafficSeparationScheme,
        "CBLSUB" => ChartObjectKind::Cable,
        "PIPSOL" => ChartObjectKind::Pipeline,
        "BRIDGE" => ChartObjectKind::Bridge,
        "HRBFAC" => ChartObjectKind::Port,
        "BERTHS" => ChartObjectKind::Berth,
        _ => match geometry {
            Cm93FeatureGeometry::Point(_) => ChartObjectKind::NavigationAid,
            _ => ChartObjectKind::Metadata,
        },
    }
}

fn render_priority(code: &str) -> i32 {
    match code {
        "LNDARE" => 10,
        "DEPARE" => 20,
        "COALNE" | "DEPCNT" => 40,
        "FAIRWY" | "RESARE" | "ACHARE" => 50,
        "SOUNDG" => 60,
        "WRECKS" | "OBSTRN" | "UWTROC" => 70,
        "LIGHTS" => 80,
        c if c.starts_with("BOY") || c.starts_with("BCN") => 90,
        _ => 30,
    }
}

fn hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 15) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_table_is_permutation() {
        let mut seen = [false; 256];
        for value in CM93_PERMUTATION {
            assert!(!seen[value as usize]);
            seen[value as usize] = true;
        }
        assert!(seen.iter().all(|value| *value));
    }

    #[test]
    fn normalizes_longitude() {
        assert!((normalize_lon(359.0) + 1.0).abs() < 1e-9);
        assert!((normalize_lon(-181.0) - 179.0).abs() < 1e-9);
    }

    #[test]
    fn rejects_short_cell() {
        assert_eq!(
            decode_cm93_cell(&[0; 20]).unwrap_err(),
            Cm93CellError::Truncated
        );
    }
}
