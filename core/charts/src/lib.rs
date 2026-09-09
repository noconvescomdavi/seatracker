use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Geometry {
    Point([f64; 2]),
    MultiPoint(Vec<[f64; 2]>),
    LineString(Vec<[f64; 2]>),
    Polygon(Vec<Vec<[f64; 2]>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeaChartObject {
    pub id: String,
    pub kind: ChartObjectKind,
    pub geometry: Option<Geometry>,
    pub source: String,
}

pub trait ChartProvider {
    fn provider_name(&self) -> &'static str;
    fn can_open(&self, header: &[u8], extension: Option<&str>) -> bool;
}
