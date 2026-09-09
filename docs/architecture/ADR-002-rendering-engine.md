# ADR-002 — Rendering Engine

Status: Accepted (Phase 0)

## Decision
SeaTracker will own a rendering abstraction with distinct raster, vector, symbol, text and overlay pipelines. The initial implementation targets GPU acceleration and keeps nautical portrayal separate from low-level drawing mechanics.

MapLibre Native is a reference for cross-platform GPU architecture, batching, resource lifecycle and map-camera mechanics. SeaTracker will not encode nautical chart semantics as MapLibre-specific styles or require MapLibre as its universal chart engine.

## Rationale
Nautical charts require format-independent object semantics, chart quilting, object information, depth/safety presentation, S-52-related portrayal concerns, navigation overlays and offline chart management that exceed a generic vector-tile map renderer.
