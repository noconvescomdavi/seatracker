# Additional map-source references

## giantramen/Navionics

Repository: https://github.com/giantramen/Navionics

Observed:
- Public repository.
- README describes packaging an iOS Navionics framework for CocoaPods.
- Repository includes a large `NavionicsMobileSDK.zip`.
- No redistribution or integration rights are inferred from repository visibility alone.

SeaTracker usage:
- Reference only unless license/terms explicitly permit integration.
- Do not bundle the SDK, proprietary charts, credentials, tokens, or protected assets without verified permission.
- Do not use it to bypass Navionics authentication, licensing, DRM, or service controls.

Classification: REFERENCE / LEGAL REVIEW REQUIRED.

## johanberonius/custom-map-source

Repository: https://github.com/johanberonius/custom-map-source

Observed patterns:
- Small XML `.ms` descriptors for custom tiled sources.
- XYZ/TMS/WMTS style URL templates.
- Server-part sharding and inverted-Y patterns.
- Examples include Kartverket nautical raster WMTS.
- Historical Navionics tile/proxy examples exist.

SeaTracker usage:
- Use as a conceptual reference for a future CustomTileSourceProvider.
- Support declarative source metadata: name, URL template, min/max zoom, tile scheme, format, attribution, headers/credentials supplied by the user, and offline/cache policy.
- Network sources must be opt-in and never become a dependency for core navigation.
- Each remote source must retain attribution/licensing metadata and obey provider terms.
- Historical Navionics proxy/token logic must not be copied as an authentication bypass.

## Proposed SeaTracker extension

```
ChartProvider
  ├── FileChartProvider
  │   ├── S57Provider
  │   ├── KAPProvider
  │   ├── MBTilesProvider
  │   └── NV2Provider
  └── TileSourceProvider
      ├── XYZProvider
      ├── TMSProvider
      └── WMTSProvider
```

Remote tiled maps are overlays/base-map sources, not authoritative ENC replacements by default.


## mgalbright/google-earth-to-navionics-route-converter

Repository: https://github.com/mgalbright/google-earth-to-navionics-route-converter

License: MIT.

Observed:
- Converts Google Earth KML routes to a GPX variant accepted by Navionics Boating.
- Uses GPSBabel as an intermediate conversion step to GeoJSON.
- Emits GPX 1.1 route points (rte/rtept) and includes regression tests.
- Experimental standard-GPX input is also supported.

SeaTracker usage:
- Reference for RouteImportProvider / RouteExportProvider interoperability.
- Implement native KML, GPX and GeoJSON parsing in the SeaTracker core instead of requiring GPSBabel on Android.
- Add an optional Navionics-compatible GPX export profile.
- Preserve waypoint order, route name and coordinate precision.
- Add golden/round-trip tests using independent fixtures.


## 50North4West/Navionics-GPX-Functions

Repository: https://github.com/50North4West/Navionics-GPX-Functions

License: GPL-3.0.

Observed:
- Parses Navionics GPX tracks (trk/trkseg/trkpt) and routes (rte/rtept).
- Converts consecutive points into leg-like point pairs.
- Includes a spherical geographic-center helper.
- Repository contains example Navionics route and large track GPX fixtures.

SeaTracker usage:
- Treat primarily as behavioral/reference material because GPL-3.0 copyleft is stronger than the preferred SeaTracker dependency posture.
- Do not copy GPL source into a differently licensed SeaTracker core without an explicit licensing decision.
- Use independently implemented GPX 1.1 parsing based on the public format.
- Support both route and track objects, preserving timestamps/extensions when present.
- Use sample structure only as validation evidence where licensing permits; prefer independently generated test fixtures.

## Consolidated route interchange architecture

```
RouteInterchange
  ├── Import
  │   ├── GPX 1.1 (routes, tracks, waypoints)
  │   ├── KML
  │   └── GeoJSON
  └── Export
      ├── Standard GPX 1.1
      ├── Navionics-compatible GPX profile
      ├── KML
      └── GeoJSON
```

Route-file parsing remains separate from route guidance, voyage planning and autopilot output.
