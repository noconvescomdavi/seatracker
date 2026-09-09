# SeaTracker implementation status

## Current milestone
Technical Preview 0.2 — core expansion.

## Implemented and covered by the repository
- Rust workspace with geographic, navigation and chart-abstraction foundations.
- SeaChartObject / ChartProvider contract.
- Android chartplotter shell using MapLibre Native.
- Offline base canvas.
- GPS own-ship position and COG/SOG.
- Immediate MOB position capture.
- Waypoints at own ship and map long-press.
- Basic track display.
- Android document picker for chart import.
- GPX import for waypoints, routes and tracks.
- GPX route export using a Navionics-compatible open GPX profile.
- NMEA 0183 core parser coverage for RMC, GGA, GLL, VTG, HDT, HDG, GSA and GSV.
- AIS target model, stale-data handling foundation and CPA/TCPA calculation.
- EBL/VRM geographic measurement foundation.
- Voyage-planning leg/TTG calculations.
- Alarm thresholds foundation.
- Chart catalog format detection and SHA-256 record creation.
- Platform-neutral persistence boundary.
- Tide/current provider interfaces.
- Plugin manifest, permission and lifecycle foundation.
- Compilable S-57 provider probe with bounded ISO-8211 detection.
- Compilable BSB/KAP provider with bounded metadata parsing.
- Compilable MBTiles provider with SQLite container detection.
- CI for Rust workspace tests and Android APK artifact.

## In progress / not yet complete
- Full S-57 dataset/feature/spatial-object decoder and S-52 portrayal.
- S-63 lawful permit/protection adapter.
- Complete KAP raster decoding and pixel ↔ geographic calibration.
- MBTiles metadata/tile reader and cache.
- CM93 parser.
- NV2 evidence-based parser.
- Chart database, spatial index, quilting and cache.
- Full AIS VDM/VDO decoder and target database.
- Android ↔ Rust FFI integration.
- Persistent route/waypoint/track UI on Android.
- Full voyage-planning UI.
- Tide/current data sources and presentation.
- Windows desktop application.
- Complete sandboxed plugin runtime.
- Brazilian chart corpus validation: uploaded RAR still could not be read by the execution runtime, therefore compatibility is not claimed.

## Safety
SeaTracker remains an Electronic Chart System / chartplotter technical preview and is not represented as a certified ECDIS.
