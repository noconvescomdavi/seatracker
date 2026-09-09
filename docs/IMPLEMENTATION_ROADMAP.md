# SeaTracker implementation roadmap

Status is evidence-based. "Implemented" means code exists and tests/builds are expected to exercise it; it does not mean certified for navigation.

## Phase 1-3 foundation
- Geographic/navigation/chart-model crates: implemented foundation.
- Android map shell, GPS, own ship, MOB, waypoint and basic track: implemented technical preview.
- GPX route import/export: implemented core foundation.
- NMEA 0183 RMC/GGA/VTG/HDT: implemented parser foundation.

## Phase 4
- AIS target model, staleness semantics and CPA/TCPA math: implemented foundation.
- AIS VDM/VDO six-bit decoder and target database: pending.
- Alarm engine: pending.

## Phase 5
- Chart catalog/scanner/index/database/quilting/cache: next active work.

## Phase 6
- S-57/KAP/MBTiles real providers: pending.
- S-63/CM93/NV2: research/permission constrained; never bypass protections.

## Phase 7-10
- Voyage planning, environmental providers, Android hardening, Windows, plugin SDK and fuzz/regression hardening remain active backlog.
