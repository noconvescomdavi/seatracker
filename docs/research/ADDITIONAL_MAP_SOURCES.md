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
