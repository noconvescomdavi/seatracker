# SeaTracker 1.0.0 status

SeaTracker is an Electronic Chart System / chartplotter. It is not represented as a certified ECDIS.

## Android operational path
- Offline raster MBTiles chart import, validation and display.
- Read-only SQLite chart access through localhost-only tile serving.
- Native Android GPS/GNSS acquisition with accuracy, satellite count, SOG/COG and stale-fix state.
- Own-ship position on chart and initial GPS centering.
- Persistent waypoints and route plotting.
- Persistent track recording with bounded point history.
- MOB immediate capture, persistence and bearing/range back to MOB.
- Geographic cursor LAT/LON plus bearing/range from own ship.
- EBL/VRM geodesic measurement.
- State recovery after app restart.
- Background chart file copy to avoid blocking the UI.

## Shared core
- WGS-84/geodesic navigation calculations including great-circle, rhumb line, bearings, destination, cross-track, along-track, TTG and required speed.
- Common SeaChartModel and format-independent provider API.
- NMEA 0183 parser for RMC/GGA/GLL/VTG/HDT/HDG/GSA/GSV.
- AIS multipart assembly, position reports, static/voyage data, MMSI target database and CPA/TCPA safety gating.
- SQLite persistence and chart catalog.
- Viewport chart selection/quilting and bounded tile cache.
- Route editor primitives, GPX interoperability and Voyage Planning calculations.
- Tide/current provider boundaries and tide extrema.
- Navigation simulator.
- Renderer layer model and diagnostics categories/counters.
- Chart, S-57, NV2 and NMEA replay internal tools.

## Chart-provider limitations
Raster MBTiles is the currently operational Android chart path. KAP metadata/calibration and S-57 ISO-8211 structural parsing are implemented in the core, but complete KAP raster decoding, S-57 feature/topology decoding plus S-52 portrayal, and NV2 rendering are not claimed. S-63/CM93/NV2 never bypass protection or access controls.

## Validation
Release CI must pass Rust format, Clippy warnings-as-errors, Rust workspace tests, Android lint and Android APK compilation. A physical Android device is still required to validate real GNSS reception, field accuracy and device-specific behavior before relying on the software operationally.
