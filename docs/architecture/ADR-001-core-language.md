# ADR-001 — Core Language

Status: Accepted (Phase 0)

## Context
SeaTracker needs a platform-independent core for geodesy, chart parsing, navigation state, AIS/GPS/NMEA, routes and safety-critical validation. Parsers process untrusted binary/text input and must be robust against memory corruption and malformed offsets.

## Decision
Use Rust as the primary SeaTracker Core language.

## Consequences
- memory-safe default for parsers and navigation engines;
- deterministic native performance;
- C-compatible FFI for Windows/Android shells;
- Cargo workspace supports unit/integration/fuzz tooling;
- unsafe code is permitted only behind narrow reviewed boundaries and must be documented.
