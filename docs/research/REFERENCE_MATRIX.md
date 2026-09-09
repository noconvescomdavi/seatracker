# SeaTracker Reference Matrix

| SeaTracker subsystem | Primary reference | Use | Dependency? | License/provenance status |
|---|---|---|---|---|
| GPU renderer / camera / cross-platform map lifecycle | `maplibre/maplibre-native` | Architecture and implementation concepts: GPU rendering, batching, resource lifecycle, platform separation | No hard dependency decided | BSD-2-Clause confirmed; record attribution if code is reused |
| S-57 desktop viewer / portrayal workflow | `CaptOz/NauticalChartsViewerSample-ForWpf` | Research flow for opening unencrypted S-57 ENC and presentation-library-driven rendering | No | Sample depends on ThinkGeo Map Suite; use as concept/reference only pending any code-specific license review |
| SeaTracker chart architecture | SeaTracker Prompt Mestre | Requirements for provider isolation, `SeaChartModel`, offline behavior, safety states, tests | Specification | Project-owned specification |
| NV2 | User-provided Navionics fixture archive | Evidence-based binary-format research | Fixture only | Do not redistribute originals; hash and inventory before research; no DRM/access-control bypass |

## Research rules

- Prefer conceptual/architectural reference over copied code.
- Before copying or adapting code, record exact repository, commit/version, file, license and required attribution.
- Chart-format semantics stay isolated from UI and renderer backends.
- Third-party renderer technology must not become a substitute for the SeaTracker Universal Chart Engine.
