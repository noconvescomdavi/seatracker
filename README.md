# SeaTracker

SeaTracker is an offline-first Electronic Chart System (ECS) / chartplotter research and development project for Windows and Android.

> **Safety:** SeaTracker is not a certified ECDIS and must not be treated as a substitute for mandatory navigation equipment, official publications, or prudent seamanship.

## Current status

**Phase 0 — Research and architecture.**

The project follows a platform-independent core, isolated chart providers, a common internal chart model, and a GPU-oriented renderer. UI code must not depend directly on S-57, KAP, NV2, or any other source chart format.

## Initial architecture

- `core/` — navigation, geographic, chart model, AIS, GPS, NMEA, routes, tides and alarms
- `chart-providers/` — isolated format adapters
- `renderer/` — rendering abstractions and platform adapters
- `apps/desktop/` — Windows desktop shell
- `apps/android/` — Android shell
- `database/` — catalog and persistence
- `simulator/` — navigation/AIS simulation and replay
- `tools/` — chart and protocol inspection tools
- `tests/` — unit, integration, parser, rendering, fuzz and regression tests
- `docs/` — ADRs, format research, licensing and provenance

See `docs/architecture/PHASE_0.md` and the ADRs for the initial technical decisions.
