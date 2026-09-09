# Phase status

Status is evidence-based. A phase can contain completed core capabilities while a specific protected or unvalidated format remains explicitly unsupported.

- **Phase 0 — Research and architecture:** complete foundation. ADRs, monorepo, CI, reference/license matrix and independent architecture are present. Research remains continuous.
- **Phase 1 — Core:** complete foundation. Shared Rust core includes geographic/geodesic calculations, common chart model, navigation state, position-provider boundary, persistence, route/track and renderer-layer abstractions.
- **Phase 2 — S-57/KAP/MBTiles:** implemented to validated capability boundaries. S-57 has safe ISO-8211 structural parsing/inventory but not full ENC feature topology/S-52 portrayal. KAP has header, reference calibration, bounds and common raster model but not full BSB raster decode in Android. Raster MBTiles is operational on Android with read-only SQLite, XYZ/TMS conversion, metadata, bounds/zoom and bounded cache.
- **Phase 3 — Navigation:** operational Android path for native GPS/GNSS, own ship, stale-fix protection, waypoints, persistent routes, persistent tracks, EBL/VRM, cursor BRG/RNG and MOB. Shared core includes own-ship configuration and track engine.
- **Phase 4 — AIS/CPA/TCPA/alarms:** core implemented for AIS VDM/VDO position reports, multipart assembly, static/voyage type 5 data, MMSI target database, stale handling and CPA/TCPA collision alarm gating. Android AIS transport/UI is not part of the current APK path.
- **Phase 5 — Catalog/index/quilting/cache:** core implemented with SHA-256 records, SQLite catalog, bounds index, viewport selection, deterministic scale/quality quilting and bounded MBTiles tile cache.
- **Phase 6 — S-63/CM93/NV2:** isolated providers and research boundaries implemented. S-63 requires legitimate permits; CM93 protection is not bypassed; NV2 remains evidence-only because supplied RAR bytes could not be reliably processed by the execution runtime. No NV2 rendering claim is made.
- **Phase 7 — Voyage Planning:** core implemented for geodesic legs, course, distance, cumulative distance, TTG and ETA.
- **Phase 8 — Tides/Currents:** provider interfaces, validity semantics and tide-extrema engine implemented. No bundled authoritative tide/current dataset is claimed.
- **Phase 9 — Android:** operational offline MBTiles navigation build with native GPS, chart, route, waypoints, tracks, MOB, cursor and EBL/VRM. Portrait/landscape and touch pan/zoom are handled by the Android/MapLibre UI.
- **Phase 10 — Hardening:** CI enforces rustfmt, Clippy with warnings-as-errors, Rust workspace tests, Android lint and APK build. Parsers use bounded reads and explicit rejection paths. Physical-device GPS/field validation remains a user/device acceptance test and SeaTracker remains an ECS/chartplotter, not certified ECDIS.

## Operational Android chart support

| Format | Android rendering status |
| --- | --- |
| Raster MBTiles | Operational |
| Vector MBTiles/PBF | Detected/modelled; not rendered by current raster Android layer |
| BSB/KAP | Metadata/calibration/model implemented; raster decoding not operational |
| S-57 | ISO-8211 structural parser implemented; complete ENC/S-52 rendering not operational |
| S-63 | Authorized-access boundary only |
| CM93 | Isolated, no protection bypass |
| NV2 | Evidence/research provider only; not operational |
