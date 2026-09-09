# SeaTracker implementation status

## Current milestone
Technical Preview 0.1.

Implemented:
- Rust workspace with geographic, navigation and chart-abstraction foundations.
- SeaChartObject / ChartProvider contract.
- Android chartplotter shell using MapLibre Native.
- Offline base canvas.
- GPS own-ship position and COG/SOG.
- Immediate MOB position capture.
- Waypoints at own ship and map long-press.
- Basic track display.
- Android document picker for chart import.
- CI for Rust tests and Android APK artifact.

Not yet validated/complete:
- Real S-57/S-63/KAP/MBTiles/CM93/NV2 parsers.
- S-52 presentation implementation.
- Chart quilting/catalog/database.
- NMEA/AIS and CPA/TCPA.
- Route/voyage-planning persistence.
- Tide/current providers.
- Android Rust FFI integration.
- Windows desktop application.
- Complete plugin SDK.
- Brazilian chart corpus: the uploaded RAR could not be read by the execution runtime, so no compatibility claim is made.

Safety classification: Electronic Chart System / chartplotter technical preview only.
