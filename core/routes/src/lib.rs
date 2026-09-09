use quick_xml::{de::from_str, se::to_string};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Waypoint {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Route {
    pub name: String,
    pub waypoints: Vec<Waypoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Track {
    pub name: String,
    pub points: Vec<Waypoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct GpxData {
    pub waypoints: Vec<Waypoint>,
    pub routes: Vec<Route>,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Deserialize)]
struct Gpx {
    #[serde(rename = "wpt", default)]
    waypoints: Vec<GpxPoint>,
    #[serde(rename = "rte", default)]
    routes: Vec<GpxRoute>,
    #[serde(rename = "trk", default)]
    tracks: Vec<GpxTrack>,
}

#[derive(Debug, Deserialize)]
struct GpxRoute {
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "rtept", default)]
    points: Vec<GpxPoint>,
}

#[derive(Debug, Deserialize)]
struct GpxTrack {
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "trkseg", default)]
    segments: Vec<GpxTrackSegment>,
}

#[derive(Debug, Deserialize)]
struct GpxTrackSegment {
    #[serde(rename = "trkpt", default)]
    points: Vec<GpxPoint>,
}

#[derive(Debug, Deserialize)]
struct GpxPoint {
    #[serde(rename = "@lat")]
    lat: f64,
    #[serde(rename = "@lon")]
    lon: f64,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    desc: Option<String>,
}

fn to_waypoint(p: GpxPoint, fallback: String) -> Waypoint {
    Waypoint {
        name: p.name.unwrap_or(fallback),
        latitude: p.lat,
        longitude: p.lon,
        notes: p.desc,
    }
}

pub fn import_gpx(xml: &str) -> Result<GpxData, String> {
    let gpx: Gpx = from_str(xml).map_err(|e| e.to_string())?;

    let waypoints = gpx.waypoints.into_iter().enumerate()
        .map(|(i,p)| to_waypoint(p, format!("WP{}", i + 1)))
        .collect();

    let routes = gpx.routes.into_iter().map(|r| Route {
        name: r.name.unwrap_or_else(|| "Route".into()),
        waypoints: r.points.into_iter().enumerate()
            .map(|(i,p)| to_waypoint(p, format!("WP{}", i + 1)))
            .collect(),
    }).collect();

    let tracks = gpx.tracks.into_iter().map(|t| {
        let mut n = 0usize;
        let mut points = Vec::new();
        for seg in t.segments {
            for p in seg.points {
                n += 1;
                points.push(to_waypoint(p, format!("TP{}", n)));
            }
        }
        Track { name: t.name.unwrap_or_else(|| "Track".into()), points }
    }).collect();

    Ok(GpxData { waypoints, routes, tracks })
}

pub fn import_gpx_routes(xml: &str) -> Result<Vec<Route>, String> {
    Ok(import_gpx(xml)?.routes)
}

#[derive(Serialize)]
struct OutGpx<'a> {
    #[serde(rename = "@version")]
    version: &'static str,
    #[serde(rename = "@creator")]
    creator: &'static str,
    #[serde(rename = "rte")]
    routes: Vec<OutRoute<'a>>,
}

#[derive(Serialize)]
struct OutRoute<'a> {
    name: &'a str,
    #[serde(rename = "rtept")]
    points: Vec<OutPoint<'a>>,
}

#[derive(Serialize)]
struct OutPoint<'a> {
    #[serde(rename = "@lat")]
    lat: f64,
    #[serde(rename = "@lon")]
    lon: f64,
    name: &'a str,
}

pub fn export_gpx_routes(routes: &[Route]) -> Result<String, String> {
    let out = OutGpx {
        version: "1.1",
        creator: "SeaTracker",
        routes: routes.iter().map(|r| OutRoute {
            name: &r.name,
            points: r.waypoints.iter().map(|p| OutPoint {
                lat: p.latitude, lon: p.longitude, name: &p.name
            }).collect(),
        }).collect(),
    };
    let body = to_string(&out).map_err(|e| e.to_string())?;
    Ok(format!(r#"<?xml version="1.0" encoding="UTF-8"?>{}"#, body))
}

pub fn export_navionics_compatible_routes(routes: &[Route]) -> Result<String, String> {
    export_gpx_routes(routes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_route_track_and_waypoint() {
        let xml = r#"<gpx version="1.1" creator="Navionics">
          <wpt lat="-22.8" lon="-43.3"><name>Mark</name></wpt>
          <rte><name>Test</name><rtept lat="-22.9" lon="-43.2"><name>A</name></rtept><rtept lat="-23.0" lon="-43.1"><name>B</name></rtept></rte>
          <trk><name>Track 1</name><trkseg><trkpt lat="-22.7" lon="-43.0"/><trkpt lat="-22.6" lon="-42.9"/></trkseg></trk>
        </gpx>"#;
        let data = import_gpx(xml).unwrap();
        assert_eq!(data.waypoints.len(), 1);
        assert_eq!(data.routes[0].name, "Test");
        assert_eq!(data.routes[0].waypoints.len(), 2);
        assert_eq!(data.tracks[0].points.len(), 2);
    }

    #[test]
    fn exports_route() {
        let routes = vec![Route {
            name: "R1".into(),
            waypoints: vec![Waypoint {
                name: "A".into(),
                latitude: -22.9,
                longitude: -43.2,
                notes: None,
            }],
        }];
        let xml = export_navionics_compatible_routes(&routes).unwrap();
        assert!(xml.contains("rtept"));
        assert!(xml.contains("SeaTracker"));
    }
}
