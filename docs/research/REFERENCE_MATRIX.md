# SeaTracker Reference Matrix

| SeaTracker subsystem | Primary reference | Use | Dependency? | License/provenance status |
|---|---|---|---|---|
| GPU renderer / camera / cross-platform map lifecycle | `maplibre/maplibre-native` | Architecture and implementation concepts: GPU rendering, batching, resource lifecycle, platform separation | Android SDK currently used in technical preview; native core remains isolated | BSD-2-Clause |
| S-57 desktop viewer / portrayal workflow | `CaptOz/NauticalChartsViewerSample-ForWpf` | Research flow for opening unencrypted S-57 ENC and presentation-library-driven rendering | No | ThinkGeo dependency in sample; concept/reference only |
| Navionics iOS SDK packaging | `giantramen/Navionics` | Historical reference for packaging the NavionicsMobileSDK iOS framework | No | No repository license declared; do not copy or redistribute SDK/framework |
| Custom raster/tile map source definitions | `johanberonius/custom-map-source` | Study external custom map-source descriptors and URL templates | No | No repository license declared; concept/reference only |
| KML/GeoJSON/GPX to Navionics route interoperability | `mgalbright/google-earth-to-navionics-route-converter` | Study route conversion pipeline and Navionics-compatible GPX behavior | No runtime dependency | MIT |
| Navionics GPX routes/tracks | `50North4West/Navionics-GPX-Functions` | Behavioral reference for distinguishing route points and track points and for bearing/distance workflows | No | GPL-3.0; no code copied/adapted into SeaTracker |
| SeaTracker chart architecture | SeaTracker Prompt Mestre | Provider isolation, `SeaChartModel`, offline behavior, safety states, tests | Specification | Project-owned specification |
| NV2 | User-provided Navionics fixture archives | Evidence-based binary-format research | Fixture only | Do not redistribute originals; hash and inventory before research; no DRM/access-control bypass |

## Interoperability policy

SeaTracker owns its route/track/waypoint data model. GPX, KML and GeoJSON are import/export adapters, never canonical storage.

Navionics interoperability is implemented from open GPX semantics and independently written SeaTracker code. No Navionics proprietary SDK, chart service, authentication mechanism, DRM component, or protected chart format is embedded without an explicit lawful integration decision.

## Research rules

- Prefer conceptual/architectural reference over copied code.
- Before copying or adapting code, record exact repository, commit/version, file, license and required attribution.
- Chart-format semantics stay isolated from UI and renderer backends.
- Third-party renderer technology must not become a substitute for the SeaTracker Universal Chart Engine.
