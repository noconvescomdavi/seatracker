# NV2 Research Log

Status legend: `CONFIRMED`, `PROBABLE`, `HYPOTHESIS`, `UNKNOWN`.

## 2026-09-09 — Phase 0 initialization

- `CONFIRMED`: NV2 research is isolated from the SeaTracker core behind `NV2Provider`.
- `CONFIRMED`: original user-provided chart fixtures must remain unmodified.
- `CONFIRMED`: research must not bypass DRM, encryption or access-control mechanisms.
- `UNKNOWN`: file inventory, headers, versions, chart IDs, bounds, scale, datum, projection, object tables and coordinate encodings in the supplied archive.
- `UNKNOWN`: whether every file in the supplied archive is NV2.

### Current blocker

The attached `navionics e-chart.rar` is available to the session, but repeated container commands for byte-level inspection timed out before returning any metadata. No binary-structure conclusion is therefore recorded yet.

### Next evidence collection

1. calculate SHA-256 for archive and every extracted fixture;
2. inventory filenames, extensions and sizes;
3. create read-only working copies;
4. inspect first/last blocks and string tables;
5. compare multiple files for stable headers and offsets;
6. search for plausible coordinate pairs only after bounding-area evidence exists;
7. add findings to the dedicated NV2 notes with evidence offsets.
