# SeaTracker Release Gates

A build may only be called a release candidate when all applicable gates below pass.

## Navigation core
- Rust workspace compiles on the supported toolchain.
- `cargo fmt --check`, `cargo clippy -D warnings` and `cargo test --workspace` are green.
- GPS position validity is explicit: valid, stale, invalid and unavailable are distinguishable.
- SOG, COG, heading and position must never silently reuse stale values.
- MOB stores position and timestamp immediately and remains available after app restart.
- Route, track and waypoint persistence must survive process termination.

## NMEA and AIS
- Checksum validation enabled for NMEA 0183 sentences.
- Unit tests cover RMC, GGA, GLL, VTG, HDT, HDG, GSA and GSV.
- AIS VDM/VDO decoder is fuzz-safe against malformed payloads.
- Multi-fragment AIS must be assembled before decode or rejected explicitly.
- CPA/TCPA never generates an alarm from stale/invalid target or own-ship data.
- AIS target age is visible to the alarm layer.

## Charts
- Every imported file is fingerprinted before parsing.
- Parser selection must be by positive format evidence, not filename alone where possible.
- Parsers have bounded reads and reject malformed/oversized structures safely.
- S-57: ISO-8211 records, feature/spatial records and coordinates validated against known ENC fixtures.
- KAP: header, raster dimensions, REF calibration and pixel/geographic round-trips validated against known fixtures.
- MBTiles: SQLite opened read-only, metadata validated and XYZ/TMS conversion tested.
- S-63: only authorized permit/decryption flows; no bypass of access controls.
- NV2: evidence-based parsing only; unknown structures stay UNKNOWN.
- Quilting must not display overlapping incompatible chart scales without deterministic selection.
- Chart cache must have a configured memory/storage ceiling.

## Android
- Android lint passes.
- APK compiles in CI.
- Cold start, permission denial, GPS acquisition, GPS loss, background/foreground, MOB, waypoint, track and chart import are manually smoke-tested on a physical Android device before final release.
- Offline launch must work without network connectivity.
- App must not require an online basemap for core navigation.
- Crash-free test session is required before release tag.

## Desktop / Windows
- Desktop target compiles against the same shared Rust core.
- Serial/TCP/UDP NMEA input tests pass before Windows is marked production-ready.

## Safety / release labeling
- Until all chart and navigation gates pass, builds remain Technical Preview or Release Candidate.
- SeaTracker is not represented as a certified ECDIS.
- Unsupported or unvalidated chart formats must show an explicit error instead of rendering guessed data.
