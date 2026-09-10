#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerKind {
    BaseChart,
    Raster,
    Enc,
    DepthAreas,
    Soundings,
    DepthContours,
    NavigationAids,
    Lights,
    LightSectors,
    Wrecks,
    Obstructions,
    Cables,
    Pipelines,
    TrafficSeparation,
    RestrictedAreas,
    AnchorageAreas,
    DredgedAreas,
    Fairways,
    Coastline,
    LandAreas,
    Text,
    Ais,
    AisTracks,
    AisCpa,
    Gps,
    Route,
    Waypoints,
    Track,
    Ebl,
    Vrm,
    Tides,
    Currents,
    Weather,
    Grib,
    Poi,
    UserMarks,
}

#[derive(Debug, Clone)]
pub struct LayerState {
    pub kind: LayerKind,
    pub visible: bool,
    pub order: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncDisplayCategory {
    Base,
    Standard,
    All,
    MarinerStandard,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Palette {
    Day,
    Dusk,
    Night,
}
#[derive(Debug, Clone, PartialEq)]
pub struct EncPortrayalSettings {
    pub display_category: EncDisplayCategory,
    pub palette: Palette,
    pub safety_depth_m: f32,
    pub safety_contour_m: f32,
    pub shallow_contour_m: f32,
    pub deep_contour_m: f32,
    pub show_soundings: bool,
    pub show_text: bool,
    pub show_light_descriptions: bool,
    pub show_national_text: bool,
    pub show_chart_boundaries: bool,
    pub show_quality_of_data: bool,
}
impl Default for EncPortrayalSettings {
    fn default() -> Self {
        Self {
            display_category: EncDisplayCategory::Standard,
            palette: Palette::Day,
            safety_depth_m: 5.0,
            safety_contour_m: 10.0,
            shallow_contour_m: 2.0,
            deep_contour_m: 20.0,
            show_soundings: true,
            show_text: true,
            show_light_descriptions: false,
            show_national_text: false,
            show_chart_boundaries: false,
            show_quality_of_data: false,
        }
    }
}
impl EncPortrayalSettings {
    pub fn validate(&self) -> Result<(), String> {
        for value in [
            self.safety_depth_m,
            self.safety_contour_m,
            self.shallow_contour_m,
            self.deep_contour_m,
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err("ENC depth settings must be finite and non-negative".into());
            }
        }
        if self.shallow_contour_m > self.deep_contour_m {
            return Err("shallow contour cannot exceed deep contour".into());
        }
        Ok(())
    }
    pub fn depth_band(&self, depth_m: f32) -> DepthBand {
        if depth_m < self.shallow_contour_m {
            DepthBand::VeryShallow
        } else if depth_m < self.safety_contour_m {
            DepthBand::Shallow
        } else if depth_m < self.deep_contour_m {
            DepthBand::Safe
        } else {
            DepthBand::Deep
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthBand {
    VeryShallow,
    Shallow,
    Safe,
    Deep,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum S57GeometryKind {
    Point,
    Line,
    Area,
    Meta,
    Unknown,
}

pub fn classify_s57_object(acronym: &str) -> (LayerKind, S57GeometryKind) {
    match acronym {
        "DEPARE" => (LayerKind::DepthAreas, S57GeometryKind::Area),
        "DEPCNT" => (LayerKind::DepthContours, S57GeometryKind::Line),
        "SOUNDG" => (LayerKind::Soundings, S57GeometryKind::Point),
        "LIGHTS" => (LayerKind::Lights, S57GeometryKind::Point),
        "BOYLAT" | "BOYCAR" | "BOYSAW" | "BCNLAT" | "BCNCAR" | "BCNSAW" => {
            (LayerKind::NavigationAids, S57GeometryKind::Point)
        }
        "WRECKS" => (LayerKind::Wrecks, S57GeometryKind::Point),
        "OBSTRN" | "UWTROC" => (LayerKind::Obstructions, S57GeometryKind::Point),
        "CBLSUB" => (LayerKind::Cables, S57GeometryKind::Line),
        "PIPSOL" => (LayerKind::Pipelines, S57GeometryKind::Line),
        "TSSLPT" | "TSEZNE" | "TSSBND" | "TSSCRS" => {
            (LayerKind::TrafficSeparation, S57GeometryKind::Line)
        }
        "RESARE" => (LayerKind::RestrictedAreas, S57GeometryKind::Area),
        "ACHARE" => (LayerKind::AnchorageAreas, S57GeometryKind::Area),
        "DRGARE" => (LayerKind::DredgedAreas, S57GeometryKind::Area),
        "FAIRWY" => (LayerKind::Fairways, S57GeometryKind::Area),
        "COALNE" => (LayerKind::Coastline, S57GeometryKind::Line),
        "LNDARE" => (LayerKind::LandAreas, S57GeometryKind::Area),
        "M_COVR" | "M_QUAL" | "M_NSYS" | "M_NPUB" => (LayerKind::Enc, S57GeometryKind::Meta),
        _ => (LayerKind::Enc, S57GeometryKind::Unknown),
    }
}

pub fn stable_layer_order(layers: &mut [LayerState]) {
    layers.sort_by_key(|layer| layer.order);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn layers_sort_by_order() {
        let mut layers = vec![
            LayerState {
                kind: LayerKind::Gps,
                visible: true,
                order: 10,
            },
            LayerState {
                kind: LayerKind::Raster,
                visible: true,
                order: 0,
            },
        ];
        stable_layer_order(&mut layers);
        assert_eq!(layers[0].kind, LayerKind::Raster);
    }
    #[test]
    fn enc_depth_bands_follow_mariner_settings() {
        let settings = EncPortrayalSettings::default();
        assert_eq!(settings.depth_band(1.0), DepthBand::VeryShallow);
        assert_eq!(settings.depth_band(7.0), DepthBand::Shallow);
        assert_eq!(settings.depth_band(15.0), DepthBand::Safe);
        assert_eq!(settings.depth_band(25.0), DepthBand::Deep);
        assert!(settings.validate().is_ok());
    }
    #[test]
    fn classifies_common_s57_objects() {
        assert_eq!(
            classify_s57_object("LIGHTS"),
            (LayerKind::Lights, S57GeometryKind::Point)
        );
        assert_eq!(
            classify_s57_object("DEPARE"),
            (LayerKind::DepthAreas, S57GeometryKind::Area)
        );
    }
}
