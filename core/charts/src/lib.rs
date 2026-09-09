use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BoundingBox {
    pub min_lat: f64,
    pub min_lon: f64,
    pub max_lat: f64,
    pub max_lon: f64,
}

impl BoundingBox {
    pub fn contains(&self, lat: f64, lon: f64) -> bool {
        lat >= self.min_lat && lat <= self.max_lat && lon >= self.min_lon && lon <= self.max_lon
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min_lat <= other.max_lat
            && self.max_lat >= other.min_lat
            && self.min_lon <= other.max_lon
            && self.max_lon >= other.min_lon
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Geometry {
    Point([f64; 2]),
    MultiPoint(Vec<[f64; 2]>),
    LineString(Vec<[f64; 2]>),
    MultiLineString(Vec<Vec<[f64; 2]>>),
    Polygon(Vec<Vec<[f64; 2]>>),
    MultiPolygon(Vec<Vec<Vec<[f64; 2]>>>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RasterLayer {
    pub id: String,
    pub source: String,
    pub bounds: Option<BoundingBox>,
    pub width_px: Option<u32>,
    pub height_px: Option<u32>,
    pub min_zoom: Option<u8>,
    pub max_zoom: Option<u8>,
    pub tile_size: Option<u16>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChartObjectKind {
    Sounding,
    DepthContour,
    DepthArea,
    Coastline,
    LandArea,
    Light,
    Buoy,
    Beacon,
    Wreck,
    Obstruction,
    RestrictedArea,
    Anchorage,
    Fairway,
    TrafficSeparationScheme,
    Cable,
    Pipeline,
    Bridge,
    Port,
    Berth,
    NavigationAid,
    TextLabel,
    Elevation,
    TriangulationPoint,
    Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SeaChartObject {
    pub id: String,
    pub kind: ChartObjectKind,
    pub geometry: Option<Geometry>,
    pub attributes: BTreeMap<String, String>,
    pub source: String,
    pub edition: Option<String>,
    pub scale: Option<u32>,
    pub zoom_level: Option<u8>,
    pub bounds: Option<BoundingBox>,
    pub quality: Option<String>,
    pub date: Option<String>,
    pub category: Option<String>,
    pub render_priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SeaChartModel {
    pub chart_id: String,
    pub provider: String,
    pub provider_version: String,
    pub source: String,
    pub edition: Option<String>,
    pub scale: Option<u32>,
    pub datum: Option<String>,
    pub projection: Option<String>,
    pub bounds: Option<BoundingBox>,
    pub raster_layers: Vec<RasterLayer>,
    pub objects: Vec<SeaChartObject>,
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProviderCapabilities {
    pub raster: bool,
    pub vector: bool,
    pub updates: bool,
    pub protected_content: bool,
    pub object_info: bool,
}

pub trait ChartProvider {
    fn provider_name(&self) -> &'static str;
    fn provider_version(&self) -> &'static str {
        "0.1.0"
    }
    fn can_open(&self, header: &[u8], extension: Option<&str>) -> bool;
    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            raster: false,
            vector: false,
            updates: false,
            protected_content: false,
            object_info: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_intersection() {
        let a = BoundingBox {
            min_lat: -10.0,
            min_lon: -20.0,
            max_lat: 0.0,
            max_lon: -10.0,
        };
        let b = BoundingBox {
            min_lat: -5.0,
            min_lon: -15.0,
            max_lat: 5.0,
            max_lon: -5.0,
        };
        assert!(a.intersects(&b));
    }
}
