# ADR-003 — Chart Abstraction

Status: Accepted (Phase 0)

## Decision
Every source chart format is implemented behind an isolated `ChartProvider` and converted into a common `SeaChartModel` before UI/rendering consumption.

Initial providers: S-57, S-63, BSB/KAP, MBTiles, CM93 and NV2.

## Required model behavior
The model preserves geometry, source attributes, chart/source identity, edition/update metadata, scale, bounding box, data quality, dates, categories and portrayal priority when available. Attributes must not be silently discarded.

## Security boundary
Providers validate offsets, lengths, counts, allocations and coordinate ranges. A malformed chart may fail the provider but must not crash the SeaTracker process.
