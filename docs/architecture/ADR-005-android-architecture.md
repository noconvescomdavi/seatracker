# ADR-005 — Android Architecture

Status: Accepted (Phase 0)

## Decision
Use a native Kotlin Android shell and reuse the Rust SeaTracker Core through a stable FFI/API boundary.

Android-specific responsibilities include lifecycle, permissions, storage access, GPS/location APIs, Bluetooth/USB integration, touch interaction and rendering-surface ownership. Navigation calculations, chart parsing/model, route logic and shared state stay in the Rust core.

## Constraints
- prioritize tablets while supporting phones;
- portrait and landscape;
- offline operation;
- no Android-only chart semantics;
- version the FFI/API explicitly to keep platform shells replaceable.
