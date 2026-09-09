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
    Wrecks,
    Obstructions,
    Cables,
    Pipelines,
    TrafficSeparation,
    RestrictedAreas,
    Ais,
    Gps,
    Route,
    Waypoints,
    Track,
    Ebl,
    Vrm,
    Tides,
    Currents,
    Poi,
    UserMarks,
}

#[derive(Debug, Clone)]
pub struct LayerState {
    pub kind: LayerKind,
    pub visible: bool,
    pub order: i32,
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
}
