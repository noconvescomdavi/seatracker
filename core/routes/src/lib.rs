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

#[derive(Debug, Deserialize)]
struct Gpx {
    #[serde(rename = "rte", default)]
    routes: Vec<GpxRoute>,
}
#[derive(Debug, Deserialize)]
struct GpxRoute {
    #[serde(default)]
    name: Option<String>,
    #[serde(rename = "rtept", default)]
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

pub fn import_gpx_routes(xml: &str) -> Result<Vec<Route>, String> {
    let gpx: Gpx = from_str(xml).map_err(|e| e.to_string())?;
    Ok(gpx.routes.into_iter().map(|r| Route {
        name: r.name.unwrap_or_else(|| "Route".into()),
        waypoints: r.points.into_iter().enumerate().map(|(i,p)| Waypoint {
            name: p.name.unwrap_or_else(|| format!("WP{}", i + 1)),
            latitude: p.lat,
            longitude: p.lon,
            notes: p.desc,
        }).collect(),
    }).collect())
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_navionics_style_route() {
        let xml = r#"<gpx version="1.1" creator="Navionics"><rte><name>Test</name><rtept lat="-22.9" lon="-43.2"><name>A</name></rtept><rtept lat="-23.0" lon="-43.1"><name>B</name></rtept></rte></gpx>"#;
        let routes = import_gpx_routes(xml).unwrap();
        assert_eq!(routes[0].name, "Test");
        assert_eq!(routes[0].waypoints.len(), 2);
    }
}
