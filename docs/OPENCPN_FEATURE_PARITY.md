# SeaTracker — OpenCPN feature/plugin parity roadmap

SeaTracker uses OpenCPN and its plugin ecosystem as engineering references. It does **not** load OpenCPN desktop plugin binaries directly on Android; wxWidgets/C++ desktop plugins require Android-native adapters or reimplementation behind stable SeaTracker interfaces.

## Reference repositories supplied for the project

### Core
- https://github.com/OpenCPN/OpenCPN
- https://github.com/OpenCPN/plugins
- https://github.com/OpenCPN/opencpn-libs
- https://github.com/OpenCPN/OCPNWindowsCoreBuildSupport

### Dashboard / instruments
- https://github.com/nohal/dashboardsk_pi/
- https://github.com/twoCanPlugin/EngineDashboard
- https://github.com/rgleason/statusbar_pi

### NMEA / NMEA 2000
- https://github.com/nohal/nsk_pi/
- https://github.com/twocanplugin/twocanplugindrivers

### AIS / GPS
- https://github.com/Hakansv/ais-vd_pi
- https://github.com/rgleason/AISradar_pi
- https://github.com/rgleason/GPS-Odometer_pi/tree/master

### Autopilot / route control
- https://github.com/rgleason/pypilot_pi
- https://github.com/douwefokkema/AutoTrackRaymarine_pi
- https://github.com/BerndCirotzki/raymarine_autopilot_pi
- https://github.com/seandepagnier/autopilot_route_pi
- https://github.com/rgleason/autopilot_route_pi

### Additional references
- https://github.com/rgleason/rtlsdr_pi/releases/tag/v1.3.1-ov50beta
- https://github.com/Rasbats/e_timer_pi
- https://github.com/Rasbats/calculator_pi/releases/tag/v2.1
- https://github.com/antipole2/JavaScript_pi/releases/tag/v0.2
- https://github.com/Rasbats/EarthExplorer_pi/
- https://github.com/Rasbats/shipdriver_pi/releases/tag/v2.4
- https://github.com/nohal/ocpndebugger_pi
- https://github.com/rgleason/Deviation_pi/releases/tag/v0.1.0-ov50

## Capability tracks

### P0 — chartplotter/navigation baseline
- [x] Android MapLibre viewport
- [x] GPS/GNSS acquisition and stale-fix guard
- [x] KAP/BSB native raster decode/render
- [x] MBTiles raster
- [x] Waypoints
- [x] Routes
- [x] Tracks
- [x] MOB
- [x] EBL/VRM
- [ ] KAP fit-to-bounds and chart quilting
- [ ] multi-chart database/index
- [ ] S-57 ENC + S-52 portrayal
- [ ] CM93 provider
- [ ] NV2 provider where technically/documentarily supported
- [ ] GPX import/export
- [ ] route activation, XTE, BTW/DTW, ETA/TTG

### P1 — vessel data bus
- [ ] NMEA 0183 parser/router
- [ ] TCP/UDP input
- [ ] Bluetooth/USB serial input
- [ ] NMEA 2000 / Signal K adapter
- [ ] data monitor/debug console
- [ ] configurable connections and priorities

### P1 — AIS
- [ ] AIS VDM/VDO decoding
- [ ] targets on chart
- [ ] CPA/TCPA
- [ ] safety zones and alarms
- [ ] target list/query
- [ ] AIS radar view

### P1 — dashboard
- [ ] SOG/COG/HDG
- [ ] position/GNSS quality
- [ ] depth/wind
- [ ] engine/N2K instruments
- [ ] trip/odometer
- [ ] configurable dashboard pages

### P2 — weather/ocean
- [ ] GRIB1/GRIB2
- [ ] wind/pressure/waves/current overlays
- [ ] timeline/interpolation
- [ ] tides/currents
- [ ] weather routing

### P2 — steering/autopilot
- [ ] route-following output abstraction
- [ ] APB/RMB/XTE sentences
- [ ] pypilot adapter
- [ ] Raymarine adapters where protocol support is available
- [ ] explicit arming/interlocks and safe disconnect

### P2 — utilities/plugin parity
- [ ] WMM/deviation
- [ ] calculator
- [ ] timer
- [ ] status bar
- [ ] JavaScript automation sandbox
- [ ] ship driver/simulator
- [ ] VDR/log/replay
- [ ] chart downloader/catalog
- [ ] plugin diagnostics

## Architecture rule

Features are implemented behind SeaTracker interfaces instead of copying desktop plugin ABI:
- ChartProvider
- DataConnection
- NavigationDataBus
- InstrumentProvider
- OverlayProvider
- RouteService
- AlarmProvider
- AutopilotAdapter
- ImportExportProvider

This keeps Android stable while allowing later Windows/Desktop frontends to reuse the same navigation semantics.
