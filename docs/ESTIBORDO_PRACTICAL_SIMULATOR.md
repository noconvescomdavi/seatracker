# Estibordo Practical Navigation Simulator

This module turns SeaTracker's chart/navigation stack into a deterministic practical-training environment.

## Runtime contract
- Chart sources: native BSB/KAP and CM93 v2.
- Position source is switchable: GNSS/NMEA for chartplotter mode; simulator engine for training mode.
- A scenario defines initial vessel state, vessel model, wind/current, route/corridor, hazards, limits and completion conditions.
- Simulation uses a fixed time step so replay and assessment are reproducible.
- Every control/state sample is journaled for replay and server-side verification.
- Assessment reports measured facts (XTE, clearance, speed, elapsed time, rule/zone violations); course authors decide scoring policy.
- Training dynamics are educational and must not be represented as a certified ship-handling model.

## Delivery gates
1. Deterministic vessel dynamics + unit tests.
2. Scenario JSON schema and validation.
3. Android SIM mode with engine/rudder controls and simulated own-ship position.
4. Scenario overlay: route, corridor, hazards, start/finish zones.
5. Assessment/replay journal.
6. KAP/BSB hardening: BSB descriptors, REF/PLY/DTM/CPH and high-resolution tiled rendering.
7. CM93 v2 dataset validation and multi-cell selection.
8. Offline persistence and resume.
9. Automated Android/Rust regression tests.
10. Release APK after all CI gates are green and representative real-chart tests pass.
