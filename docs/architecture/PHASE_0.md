# Phase 0 — Research and Architecture

## Scope

This phase establishes SeaTracker as an independent offline-first ECS/chartplotter with a platform-neutral navigation core, format-isolated chart providers, a common `SeaChartModel`, and a GPU-oriented renderer. Windows and Android are the initial targets.

## Mandatory invariants

1. UI never depends directly on a source chart format.
2. Every chart provider emits the same internal chart model.
3. Parsers are treated as untrusted-input boundaries.
4. No DRM, encryption, license or access-control bypass is permitted.
5. Navigation calculations preserve validity/state information: `UNKNOWN`, `INVALID`, `STALE`, `UNAVAILABLE`.
6. Essential chart/navigation functions remain offline-capable.
7. SeaTracker is an ECS/chartplotter, not a certified ECDIS.

## Initial stack decision

- Core: Rust workspace.
- Desktop shell: Tauri when compatible with required filesystem/serial/GPU integration.
- Android: native Kotlin shell with Rust core through a stable FFI boundary.
- Rendering: SeaTracker-owned rendering abstraction; renderer implementation may use proven GPU concepts from MapLibre Native, but SeaTracker must not make chart semantics depend on MapLibre styles or vector-tile assumptions.
- Persistence: SQLite with spatial indexing strategy appropriate to each subsystem.
- Interchange/UI schemas: versioned Rust data structures serialized through explicit DTOs.

## Rendering architecture

`ChartProvider -> SeaChartModel -> RenderScene -> RendererBackend`

The render pipeline is split into chart semantics and GPU mechanics. Chart semantics include object classification, portrayal priority, safety/depth rules, labels and nautical attributes. GPU mechanics include batching, buffers, atlases, culling, LOD, tile/cache management and text placement.

MapLibre Native is a useful reference for cross-platform GPU map rendering, lifecycle and C++/platform separation. It is not sufficient by itself as the SeaTracker chart engine because SeaTracker requires S-57/S-63/KAP/CM93/NV2 semantics, S-52-like portrayal, object inspection, chart quilting, route/navigation overlays and offline chart-library management.

## S-57 research reference

The NauticalChartsViewerSample-ForWpf project demonstrates a desktop flow for loading unencrypted S-57 ENC (`*.000`) and drawing it with a presentation library. Its architecture and presentation workflow are research references only; SeaTracker will not depend on the sample or on Map Suite.

## NV2 research boundary

The supplied Navionics corpus is a research fixture set. Originals must remain immutable. The research pipeline must:

1. record SHA-256;
2. enumerate files and sizes;
3. classify magic/header candidates;
4. identify repeated structures, offsets, strings and numerical patterns;
5. test coordinate hypotheses only against evidence;
6. maintain `CONFIRMED`, `PROBABLE`, `HYPOTHESIS`, `UNKNOWN` classifications;
7. reject any requirement to bypass encryption or access controls.

Container inspection of the uploaded RAR was not completed in this execution because the runtime file-inspection command repeatedly timed out before producing file metadata. Therefore no claim is made yet about the archive contents or NV2 byte structure.

## Planned repository layout

```text
apps/desktop/
apps/android/
core/navigation/
core/geographic/
core/charts/
core/ais/
core/gps/
core/nmea/
core/routes/
core/tides/
core/alarms/
chart-providers/s57/
chart-providers/s63/
chart-providers/kap/
chart-providers/mbtiles/
chart-providers/cm93/
chart-providers/nv2/
renderer/core/
renderer/raster/
renderer/vector/
renderer/symbols/
renderer/text/
renderer/overlays/
ui/components/
ui/chart/
ui/navigation/
ui/settings/
database/
simulator/
tests/
tools/chart-inspector/
tools/nv2-inspector/
tools/s57-inspector/
tools/nmea-replay/
docs/architecture/
docs/formats/nv2/
docs/navigation/
docs/testing/
docs/research/
docs/licenses/
samples/
```

## Phase 0 exit criteria

Phase 0 is complete only after:

- ADRs are accepted and committed;
- repository skeleton and build system exist;
- reference/provenance matrix exists;
- chart fixture inventory and hashes exist;
- the initial Rust workspace compiles;
- baseline unit-test and CI strategy is documented;
- no structural blocker remains known.

Current state: architecture documentation initialized; fixture inventory is pending due to runtime archive-inspection failure.
