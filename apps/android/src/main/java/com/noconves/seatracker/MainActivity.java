package com.noconves.seatracker;

import android.Manifest;
import android.app.Activity;
import androidx.appcompat.app.AlertDialog;
import android.content.Intent;
import android.content.SharedPreferences;
import android.content.pm.PackageManager;
import android.database.Cursor;
import android.location.GnssStatus;
import android.location.Location;
import android.location.LocationListener;
import android.location.LocationManager;
import android.net.Uri;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.os.SystemClock;
import android.provider.OpenableColumns;
import android.text.InputType;
import android.widget.Button;
import android.widget.EditText;
import android.widget.LinearLayout;
import android.widget.TextView;
import android.widget.Toast;

import androidx.activity.result.ActivityResultLauncher;
import androidx.activity.result.contract.ActivityResultContracts;
import androidx.appcompat.app.AppCompatActivity;
import androidx.core.app.ActivityCompat;
import androidx.documentfile.provider.DocumentFile;

import org.maplibre.android.MapLibre;
import org.maplibre.android.annotations.Marker;
import org.maplibre.android.annotations.MarkerOptions;
import org.maplibre.android.annotations.Polyline;
import org.maplibre.android.annotations.PolylineOptions;
import org.maplibre.android.camera.CameraPosition;
import org.maplibre.android.camera.CameraUpdateFactory;
import org.maplibre.android.geometry.LatLng;
import org.maplibre.android.geometry.LatLngBounds;
import org.maplibre.android.maps.MapLibreMap;
import org.maplibre.android.maps.MapView;
import org.maplibre.android.style.layers.RasterLayer;
import org.maplibre.android.style.sources.ImageSource;
import org.maplibre.android.style.sources.RasterSource;
import org.maplibre.android.style.sources.TileSet;

import java.io.File;
import java.io.BufferedReader;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;

public class MainActivity extends AppCompatActivity implements LocationListener {
    private static final int REQ_LOCATION = 40;
    private static final long GPS_STALE_MS = 10_000L;
    private static final String ACTIVE_MBTILES = "active.mbtiles";
    private static final String ACTIVE_KAP = "active.kap";
    private static final String CHARTS_DIR = "charts";
    private static final int MAX_TRACK_POINTS = 5_000;

    private MapView mapView;
    private MapLibreMap map;
    private LocationManager locationManager;
    private TextView status;
    private TextView chartStatus;
    private TextView cursorStatus;
    private Marker ownShip;
    private Marker mobMarker;
    private Marker measureMarker;
    private Polyline trackLine;
    private Polyline routeLine;
    private Polyline eblLine;
    private Polyline vrmLine;
    private final List<Marker> routeMarkers = new ArrayList<>();
    private final List<Marker> waypointMarkers = new ArrayList<>();
    private final List<LatLng> trackPoints = new ArrayList<>();
    private final List<LatLng> routePoints = new ArrayList<>();
    private final List<LatLng> waypointPoints = new ArrayList<>();
    private boolean tracking = false;
    private boolean routeEditing = false;
    private boolean measureMode = false;
    private boolean cameraCenteredOnGps = false;
    private Location lastLocation;
    private SharedPreferences prefs;
    private MbTilesTileServer tileServer;
    private NmeaUdpReceiver nmeaUdpReceiver;
    private NmeaTcpClient nmeaTcpClient;
    private long lastNmeaElapsedMs = 0L;
    private LatLng lastNmeaPosition;
    private Float lastNmeaSog;
    private Float lastNmeaCog;
    private String lastNmeaSource = "NMEA UDP :10110";
    private final AisDecoder aisDecoder = new AisDecoder();
    private final Map<Integer, Marker> aisMarkers = new HashMap<>();
    private int tileServerPort = -1;
    private int satellitesInView = 0;
    private String activeChartLabel = "Carta: nenhuma carta carregada";

    private final Handler gpsHandler = new Handler(Looper.getMainLooper());
    private long lastFixElapsedMs = 0L;
    private final Runnable gpsFreshnessWatch = new Runnable() {
        @Override
        public void run() {
            updateGpsFreshnessUi();
            gpsHandler.postDelayed(this, 2_000L);
        }
    };

    private final GnssStatus.Callback gnssCallback = new GnssStatus.Callback() {
        @Override
        public void onSatelliteStatusChanged(GnssStatus gnssStatus) {
            satellitesInView = gnssStatus.getSatelliteCount();
        }

        @Override
        public void onStopped() {
            satellitesInView = 0;
        }
    };

    private final ActivityResultLauncher<Intent> chartPicker =
        registerForActivityResult(new ActivityResultContracts.StartActivityForResult(), result -> {
            if (result.getResultCode() != Activity.RESULT_OK || result.getData() == null) return;
            Uri uri = result.getData().getData();
            if (uri == null) return;
            importChart(uri);
        });

    private final ActivityResultLauncher<Uri> chartFolderPicker =
        registerForActivityResult(new ActivityResultContracts.OpenDocumentTree(), uri -> {
            if (uri == null) return;
            try {
                getContentResolver().takePersistableUriPermission(
                    uri,
                    Intent.FLAG_GRANT_READ_URI_PERMISSION
                );
            } catch (SecurityException ignored) {
            }
            importChartFolder(uri);
        });

    private final ActivityResultLauncher<String[]> gpxImportPicker =
        registerForActivityResult(new ActivityResultContracts.OpenDocument(), uri -> {
            if (uri != null) importGpx(uri);
        });

    private final ActivityResultLauncher<String> gpxExportPicker =
        registerForActivityResult(
            new ActivityResultContracts.CreateDocument("application/gpx+xml"),
            uri -> {
                if (uri != null) exportGpx(uri);
            }
        );

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        MapLibre.getInstance(this);
        setContentView(R.layout.activity_main);

        prefs = getSharedPreferences("seatracker", MODE_PRIVATE);
        status = findViewById(R.id.status);
        chartStatus = findViewById(R.id.chartStatus);
        cursorStatus = findViewById(R.id.cursorStatus);
        mapView = findViewById(R.id.mapView);
        mapView.onCreate(savedInstanceState);

        status.setOnClickListener(v -> showDashboard());
        status.setOnLongClickListener(v -> {
            showConnectionManager();
            return true;
        });
        chartStatus.setOnClickListener(v -> showChartLibrary());

        restorePoints("active_route", routePoints);
        restorePoints("waypoints", waypointPoints);
        restorePoints("active_track", trackPoints);

        mapView.getMapAsync(m -> {
            map = m;
            map.setStyle("asset://offline-style.json", style -> {
                map.setCameraPosition(new CameraPosition.Builder()
                    .target(new LatLng(-22.9, -43.2))
                    .zoom(5.0)
                    .build());

                map.addOnMapLongClickListener(point -> {
                    if (routeEditing) {
                        addRoutePoint(point);
                    } else {
                        addWaypoint(point);
                    }
                    return true;
                });

                map.addOnMapClickListener(point -> {
                    if (measureMode) {
                        drawMeasurement(point);
                    } else {
                        showCursor(point);
                    }
                    return true;
                });

                redrawRoute();
                redrawWaypoints();
                redrawTrack();
                restoreMob();
                restoreActiveChart();
            });
        });

        configureButtons();

        locationManager = (LocationManager) getSystemService(LOCATION_SERVICE);
        requestLocation();
        startNmeaUdp();
        startConfiguredNmeaTcp();
        gpsHandler.post(gpsFreshnessWatch);
    }

    private void showDashboard() {
        LatLng own = ownShipPosition();
        String source = hasFreshGpsFix() ? "Android GPS/GNSS"
            : hasFreshNmeaFix() ? lastNmeaSource
            : "Sem posição válida";
        float sog = hasFreshGpsFix() && lastLocation != null && lastLocation.hasSpeed()
            ? lastLocation.getSpeed() * 1.943844f
            : lastNmeaSog == null ? 0f : lastNmeaSog;
        float cog = hasFreshGpsFix() && lastLocation != null && lastLocation.hasBearing()
            ? lastLocation.getBearing()
            : lastNmeaCog == null ? 0f : lastNmeaCog;

        String position = own == null
            ? "--"
            : String.format(Locale.US, "%.6f, %.6f", own.getLatitude(), own.getLongitude());

        String message = String.format(
            Locale.US,
            "Fonte: %s\nPosição: %s\nSOG: %.1f kn\nCOG: %.0f°\nSatélites: %d\nAIS targets: %d\nRota: %d WP / %.2f NM\nTrack: %d pontos\n%s",
            source,
            position,
            sog,
            cog,
            satellitesInView,
            aisMarkers.size(),
            routePoints.size(),
            routeDistanceNm(),
            trackPoints.size(),
            activeChartLabel
        );

        new AlertDialog.Builder(this)
            .setTitle("SeaTracker Dashboard")
            .setMessage(message)
            .setPositiveButton("Fechar", null)
            .show();
    }

    private double routeDistanceNm() {
        double total = 0.0;
        for (int i = 1; i < routePoints.size(); i++) {
            total += NavigationMath.distanceNm(routePoints.get(i - 1), routePoints.get(i));
        }
        return total;
    }

    private void configureButtons() {
        Button routeButton = findViewById(R.id.btnRoute);
        routeButton.setOnClickListener(v -> {
            routeEditing = !routeEditing;
            measureMode = false;
            routeButton.setText(routeEditing ? "ROTA*" : "ROTA");
            findViewById(R.id.btnMeasure).setSelected(false);
            Toast.makeText(
                this,
                routeEditing
                    ? "Modo rota: toque e segure para adicionar waypoints."
                    : routeSummary(),
                Toast.LENGTH_SHORT
            ).show();
            persistPoints("active_route", routePoints);
        });
        routeButton.setOnLongClickListener(v -> {
            showRouteManager();
            return true;
        });

        Button trackButton = findViewById(R.id.btnTrack);
        trackButton.setOnClickListener(v -> toggleTrack(trackButton));
        trackButton.setOnLongClickListener(v -> {
            trackPoints.clear();
            persistPoints("active_track", trackPoints);
            redrawTrack();
            Toast.makeText(this, "Track apagado.", Toast.LENGTH_SHORT).show();
            return true;
        });

        Button measureButton = findViewById(R.id.btnMeasure);
        measureButton.setOnClickListener(v -> {
            measureMode = !measureMode;
            routeEditing = false;
            findViewById(R.id.btnRoute).setSelected(false);
            measureButton.setText(measureMode ? "EBL*" : "EBL");
            Toast.makeText(
                this,
                measureMode
                    ? "EBL/VRM: toque na carta para medir a partir do own ship."
                    : "EBL/VRM encerrado.",
                Toast.LENGTH_SHORT
            ).show();
        });
        measureButton.setOnLongClickListener(v -> {
            clearMeasurement();
            return true;
        });

        findViewById(R.id.btnMob).setOnClickListener(v -> activateMob());
        findViewById(R.id.btnImport).setOnClickListener(v -> showChartLibrary());
        findViewById(R.id.btnImport).setOnLongClickListener(v -> {
            chooseChartFolder();
            return true;
        });
    }

    private void startNmeaUdp() {
        nmeaUdpReceiver = new NmeaUdpReceiver(10110, new NmeaUdpReceiver.Listener() {
            @Override
            public void onSentence(String sentence, long receivedAtMs) {
                handleMarineSentence(sentence, receivedAtMs, "NMEA UDP :10110");
            }

            @Override
            public void onStatus(String message) {
                runOnUiThread(() -> {
                    if (!hasFreshGpsFix() && lastNmeaPosition == null) {
                        cursorStatus.setText(message);
                    }
                });
            }
        });
        nmeaUdpReceiver.start();
    }

    private void handleMarineSentence(String sentence, long receivedAtMs, String source) {
        if (sentence.startsWith("$")) {
            Nmea0183Parser.Update update = Nmea0183Parser.parse(sentence);
            if (update != null) {
                runOnUiThread(() -> {
                    lastNmeaSource = source;
                    applyNmeaUpdate(update);
                });
            }
        } else if (sentence.startsWith("!")) {
            AisDecoder.Target target = aisDecoder.push(sentence, receivedAtMs);
            if (target != null && target.hasPosition()) {
                runOnUiThread(() -> applyAisTarget(target));
            }
        }
    }

    private void startConfiguredNmeaTcp() {
        if (nmeaTcpClient != null) {
            nmeaTcpClient.close();
            nmeaTcpClient = null;
        }
        if (!prefs.getBoolean("nmea_tcp_enabled", false)) return;
        String host = prefs.getString("nmea_tcp_host", "").trim();
        int port = prefs.getInt("nmea_tcp_port", 10110);
        if (host.isEmpty() || port < 1 || port > 65535) return;

        nmeaTcpClient = new NmeaTcpClient(host, port, new NmeaTcpClient.Listener() {
            @Override
            public void onSentence(String sentence, long receivedAtMs) {
                handleMarineSentence(sentence, receivedAtMs, "NMEA TCP " + host + ":" + port);
            }

            @Override
            public void onStatus(String message) {
                runOnUiThread(() -> cursorStatus.setText(message));
            }
        });
        nmeaTcpClient.start();
    }

    private void showConnectionManager() {
        LinearLayout form = new LinearLayout(this);
        form.setOrientation(LinearLayout.VERTICAL);
        int pad = (int) (16 * getResources().getDisplayMetrics().density);
        form.setPadding(pad, pad, pad, 0);

        EditText hostInput = new EditText(this);
        hostInput.setHint("Host/IP NMEA TCP");
        hostInput.setSingleLine(true);
        hostInput.setText(prefs.getString("nmea_tcp_host", ""));
        form.addView(hostInput);

        EditText portInput = new EditText(this);
        portInput.setHint("Porta (ex.: 10110)");
        portInput.setInputType(InputType.TYPE_CLASS_NUMBER);
        portInput.setSingleLine(true);
        portInput.setText(Integer.toString(prefs.getInt("nmea_tcp_port", 10110)));
        form.addView(portInput);

        new AlertDialog.Builder(this)
            .setTitle("Conexões de Navegação")
            .setMessage("UDP 10110 permanece ativo. Configure aqui uma fonte NMEA/AIS TCP adicional.")
            .setView(form)
            .setPositiveButton("Salvar e conectar", (dialog, which) -> {
                String host = hostInput.getText().toString().trim();
                int port;
                try {
                    port = Integer.parseInt(portInput.getText().toString().trim());
                } catch (NumberFormatException error) {
                    Toast.makeText(this, "Porta TCP inválida.", Toast.LENGTH_LONG).show();
                    return;
                }
                if (host.isEmpty() || port < 1 || port > 65535) {
                    Toast.makeText(this, "Host/porta TCP inválidos.", Toast.LENGTH_LONG).show();
                    return;
                }
                prefs.edit()
                    .putBoolean("nmea_tcp_enabled", true)
                    .putString("nmea_tcp_host", host)
                    .putInt("nmea_tcp_port", port)
                    .apply();
                startConfiguredNmeaTcp();
            })
            .setNeutralButton("Desativar TCP", (dialog, which) -> {
                prefs.edit().putBoolean("nmea_tcp_enabled", false).apply();
                if (nmeaTcpClient != null) {
                    nmeaTcpClient.close();
                    nmeaTcpClient = null;
                }
                cursorStatus.setText("NMEA TCP desativado • UDP 10110 ativo");
            })
            .setNegativeButton("Cancelar", null)
            .show();
    }

    private void applyAisTarget(AisDecoder.Target target) {
        if (map == null || target.latitude == null || target.longitude == null) return;
        LatLng position = new LatLng(target.latitude, target.longitude);
        Marker marker = aisMarkers.get(target.mmsi);
        String title = String.format(
            Locale.US,
            "AIS %09d • SOG %.1f kn • COG %.0f°",
            target.mmsi,
            target.sogKnots == null ? 0f : target.sogKnots,
            target.cogDeg == null ? 0f : target.cogDeg
        );
        if (marker == null) {
            marker = map.addMarker(new MarkerOptions().position(position).title(title));
            aisMarkers.put(target.mmsi, marker);
        } else {
            marker.setPosition(position);
            marker.setTitle(title);
        }
        aisDecoder.discardStale(System.currentTimeMillis(), 180_000L);
    }

    private void applyNmeaUpdate(Nmea0183Parser.Update update) {
        if (update.latitude != null && update.longitude != null && update.positionValid) {
            lastNmeaPosition = new LatLng(update.latitude, update.longitude);
            lastNmeaElapsedMs = SystemClock.elapsedRealtime();
        }
        if (update.sogKnots != null) lastNmeaSog = update.sogKnots;
        if (update.cogDeg != null) lastNmeaCog = update.cogDeg;

        if (!hasFreshGpsFix() && lastNmeaPosition != null && map != null) {
            if (ownShip == null) {
                ownShip = map.addMarker(
                    new MarkerOptions().position(lastNmeaPosition).title("Own Ship • NMEA")
                );
            } else {
                ownShip.setPosition(lastNmeaPosition);
            }

            if (!cameraCenteredOnGps) {
                cameraCenteredOnGps = true;
                map.animateCamera(CameraUpdateFactory.newLatLngZoom(lastNmeaPosition, 12.0));
            }

            status.setText(String.format(
                Locale.US,
                "NMEA UDP • %.6f %.6f • SOG %.1f kn • COG %.0f°",
                lastNmeaPosition.getLatitude(),
                lastNmeaPosition.getLongitude(),
                lastNmeaSog == null ? 0f : lastNmeaSog,
                lastNmeaCog == null ? 0f : lastNmeaCog
            ));

            if (tracking) addTrackPoint(lastNmeaPosition);
            updateMobStatus(lastNmeaPosition);
        }
    }

    private boolean hasFreshNmeaFix() {
        return lastNmeaPosition != null
            && lastNmeaElapsedMs > 0L
            && SystemClock.elapsedRealtime() - lastNmeaElapsedMs <= GPS_STALE_MS;
    }

    private void requestLocation() {
        if (ActivityCompat.checkSelfPermission(this, Manifest.permission.ACCESS_FINE_LOCATION)
            != PackageManager.PERMISSION_GRANTED) {
            ActivityCompat.requestPermissions(
                this,
                new String[]{
                    Manifest.permission.ACCESS_FINE_LOCATION,
                    Manifest.permission.ACCESS_COARSE_LOCATION
                },
                REQ_LOCATION
            );
            return;
        }

        if (!locationManager.isProviderEnabled(LocationManager.GPS_PROVIDER)) {
            status.setText("SeaTracker • GPS desativado");
            return;
        }

        status.setText("SeaTracker • procurando sinal GPS…");
        locationManager.requestLocationUpdates(LocationManager.GPS_PROVIDER, 1000, 0f, this);
        try {
            locationManager.registerGnssStatusCallback(gnssCallback, gpsHandler);
        } catch (RuntimeException ignored) {
        }
    }

    @Override
    public void onRequestPermissionsResult(int requestCode, String[] permissions, int[] grantResults) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults);
        if (requestCode == REQ_LOCATION
            && grantResults.length > 0
            && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
            requestLocation();
        } else {
            status.setText("SeaTracker • GPS sem permissão");
        }
    }

    @Override
    public void onLocationChanged(Location location) {
        lastLocation = location;
        lastFixElapsedMs = SystemClock.elapsedRealtime();
        LatLng position = new LatLng(location.getLatitude(), location.getLongitude());

        if (map != null) {
            if (ownShip == null) {
                ownShip = map.addMarker(new MarkerOptions().position(position).title("Own Ship"));
            } else {
                ownShip.setPosition(position);
            }

            if (!cameraCenteredOnGps) {
                cameraCenteredOnGps = true;
                map.animateCamera(CameraUpdateFactory.newLatLngZoom(position, 12.0));
            }

            if (tracking) {
                addTrackPoint(position);
            }
        }

        double accuracy = location.hasAccuracy() ? location.getAccuracy() : Double.NaN;
        status.setText(String.format(
            Locale.US,
            "GPS OK • %.6f %.6f • ACC %.0fm • SAT %d • SOG %.1f kn • COG %.0f°",
            location.getLatitude(),
            location.getLongitude(),
            accuracy,
            satellitesInView,
            location.hasSpeed() ? location.getSpeed() * 1.943844f : 0f,
            location.hasBearing() ? location.getBearing() : 0f
        ));

        updateMobStatus(position);
    }

    private void updateGpsFreshnessUi() {
        if (ActivityCompat.checkSelfPermission(this, Manifest.permission.ACCESS_FINE_LOCATION)
            != PackageManager.PERMISSION_GRANTED) {
            return;
        }
        if (!locationManager.isProviderEnabled(LocationManager.GPS_PROVIDER)) {
            status.setText("SeaTracker • GPS desativado");
            return;
        }
        if (lastFixElapsedMs == 0L) {
            if (!hasFreshNmeaFix()) {
                status.setText("SeaTracker • procurando GPS/NMEA…");
            }
            return;
        }
        long age = SystemClock.elapsedRealtime() - lastFixElapsedMs;
        if (age > GPS_STALE_MS) {
            status.setText(String.format(
                Locale.US,
                "SeaTracker • posição GPS STALE (%ds) • não usar para navegação",
                age / 1000
            ));
        }
    }

    @Override
    public void onProviderDisabled(String provider) {
        if (LocationManager.GPS_PROVIDER.equals(provider)) {
            status.setText("SeaTracker • GPS desativado");
        }
    }

    @Override
    public void onProviderEnabled(String provider) {
        if (LocationManager.GPS_PROVIDER.equals(provider)) {
            status.setText("SeaTracker • procurando sinal GPS…");
            requestLocation();
        }
    }

    private boolean hasFreshGpsFix() {
        return lastLocation != null
            && lastFixElapsedMs > 0L
            && (SystemClock.elapsedRealtime() - lastFixElapsedMs) <= GPS_STALE_MS;
    }

    private LatLng ownShipPosition() {
        if (hasFreshGpsFix()) {
            return new LatLng(lastLocation.getLatitude(), lastLocation.getLongitude());
        }
        if (hasFreshNmeaFix()) {
            return lastNmeaPosition;
        }
        return null;
    }

    private void activateMob() {
        LatLng position = ownShipPosition();
        if (position == null || map == null) {
            Toast.makeText(
                this,
                "MOB indisponível: posição GPS ainda não é válida.",
                Toast.LENGTH_SHORT
            ).show();
            return;
        }

        if (mobMarker != null) map.removeMarker(mobMarker);
        mobMarker = map.addMarker(new MarkerOptions().position(position).title("MOB"));
        map.animateCamera(CameraUpdateFactory.newLatLngZoom(position, 14.0));
        prefs.edit()
            .putString("mob_position", encodePoint(position))
            .putLong("mob_time", System.currentTimeMillis())
            .apply();
        Toast.makeText(this, "MOB registrado imediatamente.", Toast.LENGTH_LONG).show();
    }

    private void restoreMob() {
        if (map == null) return;
        String value = prefs.getString("mob_position", null);
        LatLng point = decodePoint(value);
        if (point != null) {
            mobMarker = map.addMarker(new MarkerOptions().position(point).title("MOB"));
        }
    }

    private void updateMobStatus(LatLng own) {
        if (mobMarker == null) return;
        LatLng mob = mobMarker.getPosition();
        double bearing = NavigationMath.bearingDeg(own, mob);
        double range = NavigationMath.distanceNm(own, mob);
        cursorStatus.setText(String.format(
            Locale.US,
            "MOB • BRG %.0f° • RNG %.2f NM",
            bearing,
            range
        ));
    }

    private void showRouteManager() {
        String[] actions = {
            "Importar GPX",
            "Exportar rota GPX",
            "Inverter rota",
            "Apagar rota"
        };
        new AlertDialog.Builder(this)
            .setTitle("Route Manager")
            .setItems(actions, (dialog, which) -> {
                switch (which) {
                    case 0:
                        gpxImportPicker.launch(new String[]{
                            "application/gpx+xml",
                            "application/xml",
                            "text/xml",
                            "text/plain",
                            "*/*"
                        });
                        break;
                    case 1:
                        if (routePoints.isEmpty()) {
                            Toast.makeText(this, "Não há rota para exportar.", Toast.LENGTH_SHORT).show();
                        } else {
                            gpxExportPicker.launch("SeaTracker-route.gpx");
                        }
                        break;
                    case 2:
                        java.util.Collections.reverse(routePoints);
                        persistPoints("active_route", routePoints);
                        redrawRoute();
                        cursorStatus.setText(routeSummary());
                        break;
                    case 3:
                        clearRoute();
                        Toast.makeText(this, "Rota apagada.", Toast.LENGTH_SHORT).show();
                        break;
                    default:
                        break;
                }
            })
            .setNegativeButton("Fechar", null)
            .show();
    }

    private void importGpx(Uri uri) {
        new Thread(() -> {
            try (InputStream in = getContentResolver().openInputStream(uri);
                 BufferedReader reader = new BufferedReader(
                     new InputStreamReader(in, StandardCharsets.UTF_8)
                 )) {
                if (in == null) throw new IllegalStateException("Não foi possível abrir o GPX");
                StringBuilder xml = new StringBuilder();
                String line;
                while ((line = reader.readLine()) != null) {
                    if (xml.length() > 8_000_000) {
                        throw new IllegalStateException("GPX excede o limite de 8 MB");
                    }
                    xml.append(line).append('\n');
                }

                GpxRouteCodec.Data data = GpxRouteCodec.parse(xml.toString());
                runOnUiThread(() -> {
                    if (!data.route.isEmpty()) {
                        routePoints.clear();
                        routePoints.addAll(data.route);
                        persistPoints("active_route", routePoints);
                        redrawRoute();
                    }
                    if (!data.waypoints.isEmpty()) {
                        waypointPoints.addAll(data.waypoints);
                        persistPoints("waypoints", waypointPoints);
                        redrawWaypoints();
                    }
                    Toast.makeText(
                        this,
                        "GPX importado: " + data.route.size() + " ponto(s) de rota e "
                            + data.waypoints.size() + " waypoint(s).",
                        Toast.LENGTH_LONG
                    ).show();
                    cursorStatus.setText(routeSummary());
                });
            } catch (Exception error) {
                runOnUiThread(() -> Toast.makeText(
                    this,
                    "Falha GPX: " + error.getMessage(),
                    Toast.LENGTH_LONG
                ).show());
            }
        }, "SeaTracker-GPX-Import").start();
    }

    private void exportGpx(Uri uri) {
        try (OutputStream out = getContentResolver().openOutputStream(uri, "wt")) {
            if (out == null) throw new IllegalStateException("Não foi possível criar o GPX");
            String xml = GpxRouteCodec.exportRoute("SeaTracker Route", routePoints);
            out.write(xml.getBytes(StandardCharsets.UTF_8));
            out.flush();
            Toast.makeText(this, "Rota GPX exportada.", Toast.LENGTH_LONG).show();
        } catch (Exception error) {
            Toast.makeText(
                this,
                "Falha ao exportar GPX: " + error.getMessage(),
                Toast.LENGTH_LONG
            ).show();
        }
    }

    private void toggleTrack(Button button) {
        tracking = !tracking;
        button.setText(tracking ? "STOP" : "TRACK");
        if (tracking && trackPoints.isEmpty()) {
            LatLng own = ownShipPosition();
            if (own != null) {
                trackPoints.add(own);
                persistPoints("active_track", trackPoints);
            }
        }
        redrawTrack();
    }

    private void addTrackPoint(LatLng position) {
        if (!trackPoints.isEmpty()) {
            LatLng last = trackPoints.get(trackPoints.size() - 1);
            if (NavigationMath.distanceNm(last, position) < 0.003) {
                return;
            }
        }
        if (trackPoints.size() == MAX_TRACK_POINTS) {
            trackPoints.remove(0);
        }
        trackPoints.add(position);
        persistPoints("active_track", trackPoints);
        redrawTrack();
    }

    private void redrawTrack() {
        if (map == null) return;
        if (trackLine != null) {
            map.removePolyline(trackLine);
            trackLine = null;
        }
        if (trackPoints.size() >= 2) {
            trackLine = map.addPolyline(new PolylineOptions().addAll(trackPoints).width(4f));
        }
    }

    private void addWaypoint(LatLng point) {
        waypointPoints.add(point);
        persistPoints("waypoints", waypointPoints);
        redrawWaypoints();
        showCursor(point);
    }

    private void redrawWaypoints() {
        if (map == null) return;
        removeMarkers(waypointMarkers);
        for (int i = 0; i < waypointPoints.size(); i++) {
            waypointMarkers.add(map.addMarker(
                new MarkerOptions()
                    .position(waypointPoints.get(i))
                    .title("WP " + (i + 1))
            ));
        }
    }

    private void addRoutePoint(LatLng point) {
        routePoints.add(point);
        persistPoints("active_route", routePoints);
        redrawRoute();
        cursorStatus.setText(routeSummary());
    }

    private void redrawRoute() {
        if (map == null) return;
        if (routeLine != null) {
            map.removePolyline(routeLine);
            routeLine = null;
        }
        removeMarkers(routeMarkers);
        for (int i = 0; i < routePoints.size(); i++) {
            routeMarkers.add(map.addMarker(
                new MarkerOptions()
                    .position(routePoints.get(i))
                    .title("Rota WP " + (i + 1))
            ));
        }
        if (routePoints.size() >= 2) {
            routeLine = map.addPolyline(new PolylineOptions().addAll(routePoints).width(6f));
        }
    }

    private String routeSummary() {
        return String.format(
            Locale.US,
            "Rota • %d WP • %.2f NM",
            routePoints.size(),
            routeDistanceNm()
        );
    }

    private void clearRoute() {
        routePoints.clear();
        persistPoints("active_route", routePoints);
        redrawRoute();
        cursorStatus.setText("Rota apagada");
    }

    private void showCursor(LatLng point) {
        LatLng own = ownShipPosition();
        if (own == null) {
            cursorStatus.setText(String.format(
                Locale.US,
                "Cursor • LAT %.6f • LON %.6f",
                point.getLatitude(),
                point.getLongitude()
            ));
            return;
        }
        cursorStatus.setText(String.format(
            Locale.US,
            "Cursor %.6f %.6f • BRG %.0f° • RNG %.2f NM",
            point.getLatitude(),
            point.getLongitude(),
            NavigationMath.bearingDeg(own, point),
            NavigationMath.distanceNm(own, point)
        ));
    }

    private void drawMeasurement(LatLng point) {
        LatLng own = ownShipPosition();
        if (own == null || map == null) {
            Toast.makeText(this, "EBL/VRM requer GPS válido.", Toast.LENGTH_SHORT).show();
            return;
        }
        clearMeasurement();
        double range = NavigationMath.distanceNm(own, point);
        double bearing = NavigationMath.bearingDeg(own, point);
        eblLine = map.addPolyline(
            new PolylineOptions().add(own, point).width(4f)
        );
        vrmLine = map.addPolyline(
            new PolylineOptions().addAll(NavigationMath.circle(own, range, 72)).width(2f)
        );
        measureMarker = map.addMarker(
            new MarkerOptions().position(point).title(String.format(
                Locale.US,
                "EBL %.0f° / VRM %.2f NM",
                bearing,
                range
            ))
        );
        cursorStatus.setText(String.format(
            Locale.US,
            "EBL %.0f° • VRM %.2f NM",
            bearing,
            range
        ));
    }

    private void clearMeasurement() {
        if (map == null) return;
        if (eblLine != null) {
            map.removePolyline(eblLine);
            eblLine = null;
        }
        if (vrmLine != null) {
            map.removePolyline(vrmLine);
            vrmLine = null;
        }
        if (measureMarker != null) {
            map.removeMarker(measureMarker);
            measureMarker = null;
        }
        cursorStatus.setText("EBL/VRM limpo");
    }

    private void removeMarkers(List<Marker> markers) {
        if (map == null) return;
        for (Marker marker : markers) {
            map.removeMarker(marker);
        }
        markers.clear();
    }

    private void persistPoints(String key, List<LatLng> points) {
        StringBuilder value = new StringBuilder();
        for (LatLng point : points) {
            if (value.length() > 0) value.append(';');
            value.append(encodePoint(point));
        }
        prefs.edit().putString(key, value.toString()).apply();
    }

    private void restorePoints(String key, List<LatLng> output) {
        output.clear();
        String saved = prefs.getString(key, "");
        if (saved == null || saved.isEmpty()) return;
        for (String item : saved.split(";")) {
            LatLng point = decodePoint(item);
            if (point != null) output.add(point);
        }
    }

    private String encodePoint(LatLng point) {
        return point.getLatitude() + "," + point.getLongitude();
    }

    private LatLng decodePoint(String value) {
        if (value == null || value.isEmpty()) return null;
        String[] parts = value.split(",");
        if (parts.length != 2) return null;
        try {
            double lat = Double.parseDouble(parts[0]);
            double lon = Double.parseDouble(parts[1]);
            if (lat < -90.0 || lat > 90.0 || lon < -180.0 || lon > 180.0) return null;
            return new LatLng(lat, lon);
        } catch (NumberFormatException ignored) {
            return null;
        }
    }

    private void chooseChart() {
        Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
        intent.addCategory(Intent.CATEGORY_OPENABLE);
        intent.setType("*/*");
        intent.addFlags(
            Intent.FLAG_GRANT_READ_URI_PERMISSION
                | Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION
        );
        chartPicker.launch(intent);
    }

    private void importChart(Uri uri) {
        String name = displayName(uri);
        if (name == null) {
            chartStatus.setText("Carta não carregada: nome do arquivo indisponível");
            return;
        }

        String lower = name.toLowerCase(Locale.ROOT);
        final String kind;
        if (lower.endsWith(".kap") || lower.endsWith(".bsb")) {
            kind = "kap";
        } else if (lower.endsWith(".mbtiles")) {
            kind = "mbtiles";
        } else if (lower.endsWith(".nv2")) {
            chartStatus.setText("NV2 detectado • provider em validação");
            Toast.makeText(this, "NV2 ainda não possui renderização validada.", Toast.LENGTH_LONG).show();
            return;
        } else {
            chartStatus.setText("Formato ainda não suportado: " + name);
            Toast.makeText(this, "Selecione KAP/BSB ou MBTiles.", Toast.LENGTH_LONG).show();
            return;
        }

        chartStatus.setText("Importando carta " + name + "…");
        new Thread(() -> copyAndOpenChart(uri, name, kind), "SeaTracker-ChartImport").start();
    }

    private void copyAndOpenChart(Uri uri, String name, String kind) {
        File chartsDir = new File(getFilesDir(), CHARTS_DIR);
        if (!chartsDir.exists() && !chartsDir.mkdirs()) {
            runOnUiThread(() -> chartStatus.setText("Falha ao criar armazenamento de cartas"));
            return;
        }

        String safeName = sanitizeChartFileName(name);
        String activeName = kind.equals("kap") ? safeName : ACTIVE_MBTILES;
        File temp = new File(chartsDir, activeName + ".partial");
        File target = new File(chartsDir, activeName);
        try (InputStream in = getContentResolver().openInputStream(uri);
             FileOutputStream out = new FileOutputStream(temp, false)) {
            if (in == null) throw new IllegalStateException("Não foi possível abrir a carta");
            byte[] buffer = new byte[128 * 1024];
            int read;
            long total = 0L;
            while ((read = in.read(buffer)) != -1) {
                total += read;
                if (total > 4L * 1024L * 1024L * 1024L) {
                    throw new IllegalStateException("Carta excede o limite de 4 GB");
                }
                out.write(buffer, 0, read);
            }
            out.flush();

            if (target.exists() && !target.delete()) {
                throw new IllegalStateException("Não foi possível substituir a carta anterior");
            }
            if (!temp.renameTo(target)) {
                throw new IllegalStateException("Não foi possível finalizar a importação");
            }

            prefs.edit()
                .putString("active_chart_name", name)
                .putString("active_chart_kind", kind)
                .putString("active_chart_file", target.getName())
                .apply();

            runOnUiThread(() -> {
                if (kind.equals("kap")) {
                    loadKap(target, name);
                } else {
                    loadMbTiles(target, name);
                }
            });
        } catch (Exception e) {
            temp.delete();
            runOnUiThread(() -> {
                chartStatus.setText("Falha ao importar carta");
                Toast.makeText(this, "Erro na carta: " + e.getMessage(), Toast.LENGTH_LONG).show();
            });
        }
    }

    private String displayName(Uri uri) {
        try (Cursor cursor = getContentResolver().query(
            uri,
            new String[]{OpenableColumns.DISPLAY_NAME},
            null,
            null,
            null
        )) {
            if (cursor != null && cursor.moveToFirst()) {
                int index = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME);
                if (index >= 0) return cursor.getString(index);
            }
        }
        return uri.getLastPathSegment();
    }

    private void restoreActiveChart() {
        File chartsDir = new File(getFilesDir(), CHARTS_DIR);
        String kind = prefs.getString("active_chart_kind", "");
        String name = prefs.getString("active_chart_name", "Carta");
        String fileName = prefs.getString("active_chart_file", "");

        if ("kap".equals(kind)) {
            File file = new File(chartsDir, fileName.isEmpty() ? ACTIVE_KAP : fileName);
            if (file.isFile()) loadKap(file, name);
            return;
        }

        if ("mbtiles".equals(kind)) {
            File file = new File(chartsDir, fileName.isEmpty() ? ACTIVE_MBTILES : fileName);
            if (file.isFile()) loadMbTiles(file, name);
        }
    }

    private void loadKap(File file, String displayName) {
        chartStatus.setText("Decodificando KAP/BSB " + displayName + "…");
        new Thread(() -> {
            try {
                KapChartDecoder.Result result = KapChartDecoder.decode(file);
                runOnUiThread(() -> {
                    if (map == null) {
                        result.bitmap.recycle();
                        return;
                    }
                    map.getStyle(style -> {
                        if (style.getLayer("chart-raster") != null) style.removeLayer("chart-raster");
                        if (style.getSource("chart-source") != null) style.removeSource("chart-source");

                        if (tileServer != null) {
                            tileServer.stop();
                            tileServer = null;
                        }

                        ImageSource source = new ImageSource(
                            "chart-source",
                            result.quad,
                            result.bitmap
                        );
                        RasterLayer layer = new RasterLayer("chart-raster", "chart-source");
                        style.addSource(source);
                        style.addLayer(layer);

                        activeChartLabel = "Carta KAP ativa: " + result.summary();
                        chartStatus.setText(activeChartLabel);

                        if (!cameraCenteredOnGps) {
                            LatLngBounds bounds = new LatLngBounds.Builder()
                                .include(new LatLng(result.minLat, result.minLon))
                                .include(new LatLng(result.maxLat, result.maxLon))
                                .build();
                            map.animateCamera(CameraUpdateFactory.newLatLngBounds(bounds, 48));
                        }
                    });
                });
            } catch (Exception e) {
                runOnUiThread(() -> {
                    chartStatus.setText("Falha KAP/BSB: " + e.getMessage());
                    Toast.makeText(
                        this,
                        "Não foi possível decodificar a carta: " + e.getMessage(),
                        Toast.LENGTH_LONG
                    ).show();
                });
            }
        }, "SeaTracker-KAP-Decode").start();
    }

    private void loadMbTiles(File file, String displayName) {
        try {
            if (tileServer != null) tileServer.stop();
            tileServer = new MbTilesTileServer(file);
            tileServerPort = tileServer.start();
            MbTilesTileServer.Info info = tileServer.getInfo();

            if (map == null) return;
            map.getStyle(style -> {
                if (style.getLayer("chart-raster") != null) style.removeLayer("chart-raster");
                if (style.getSource("chart-source") != null) style.removeSource("chart-source");

                String template = "http://127.0.0.1:" + tileServerPort
                    + "/tiles/{z}/{x}/{y}.png";
                TileSet tileSet = new TileSet("2.2.0", template);
                RasterSource source = new RasterSource("chart-source", tileSet, 256);
                RasterLayer layer = new RasterLayer("chart-raster", "chart-source");
                style.addSource(source);
                style.addLayer(layer);

                String title = info != null && info.name != null ? info.name : displayName;
                String zoom = info != null && info.minZoom != null && info.maxZoom != null
                    ? " • Z" + info.minZoom + "-" + info.maxZoom
                    : "";
                activeChartLabel = "Carta MBTiles ativa: " + title + zoom;
                chartStatus.setText(activeChartLabel);

                if (!cameraCenteredOnGps && info != null && info.bounds != null) {
                    double centerLon = (info.bounds[0] + info.bounds[2]) / 2.0;
                    double centerLat = (info.bounds[1] + info.bounds[3]) / 2.0;
                    double chartZoom = info.minZoom != null ? Math.max(1.0, info.minZoom) : 6.0;
                    map.animateCamera(CameraUpdateFactory.newLatLngZoom(
                        new LatLng(centerLat, centerLon),
                        chartZoom
                    ));
                }
            });
        } catch (Exception e) {
            chartStatus.setText("Carta inválida ou incompatível");
            Toast.makeText(this, "Falha MBTiles: " + e.getMessage(), Toast.LENGTH_LONG).show();
        }
    }

    private void chooseChartFolder() {
        chartFolderPicker.launch(null);
    }

    private void importChartFolder(Uri treeUri) {
        chartStatus.setText("Indexando pasta de cartas…");
        new Thread(() -> {
            DocumentFile root = DocumentFile.fromTreeUri(this, treeUri);
            if (root == null || !root.isDirectory()) {
                runOnUiThread(() -> chartStatus.setText("Pasta de cartas inválida"));
                return;
            }

            File chartsDir = new File(getFilesDir(), CHARTS_DIR);
            if (!chartsDir.exists() && !chartsDir.mkdirs()) {
                runOnUiThread(() -> chartStatus.setText("Falha ao criar biblioteca local"));
                return;
            }

            int imported = importKapDocumentsRecursive(root, chartsDir, 0, 500);
            final int importedCount = imported;
            runOnUiThread(() -> {
                chartStatus.setText("Biblioteca: " + importedCount + " carta(s) KAP/BSB importada(s)");
                Toast.makeText(
                    this,
                    "Importação de pasta concluída: " + importedCount + " carta(s).",
                    Toast.LENGTH_LONG
                ).show();
                showChartLibrary();
            });
        }, "SeaTracker-ChartFolderImport").start();
    }

    private int importKapDocumentsRecursive(
        DocumentFile directory,
        File chartsDir,
        int depth,
        int remaining
    ) {
        if (depth > 8 || remaining <= 0) return 0;
        int imported = 0;

        for (DocumentFile child : directory.listFiles()) {
            if (imported >= remaining) break;
            if (child.isDirectory()) {
                imported += importKapDocumentsRecursive(
                    child,
                    chartsDir,
                    depth + 1,
                    remaining - imported
                );
                continue;
            }

            String name = child.getName();
            if (name == null) continue;
            String lower = name.toLowerCase(Locale.ROOT);
            if (!lower.endsWith(".kap") && !lower.endsWith(".bsb")) continue;

            File target = new File(chartsDir, sanitizeChartFileName(name));
            File temp = new File(chartsDir, target.getName() + ".partial");

            try (InputStream in = getContentResolver().openInputStream(child.getUri());
                 FileOutputStream out = new FileOutputStream(temp, false)) {
                if (in == null) continue;
                byte[] buffer = new byte[128 * 1024];
                int read;
                long total = 0L;
                while ((read = in.read(buffer)) != -1) {
                    total += read;
                    if (total > 2L * 1024L * 1024L * 1024L) {
                        throw new IllegalStateException("Carta individual excede 2 GB");
                    }
                    out.write(buffer, 0, read);
                }
                out.flush();

                if (target.exists() && !target.delete()) {
                    temp.delete();
                    continue;
                }
                if (!temp.renameTo(target)) {
                    temp.delete();
                    continue;
                }
                imported++;
            } catch (Exception ignored) {
                temp.delete();
            }
        }
        return imported;
    }

    private String sanitizeChartFileName(String name) {
        String clean = name.replaceAll("[^A-Za-z0-9._-]", "_");
        if (clean.isEmpty()) clean = "chart.kap";
        return clean;
    }

    private void showChartLibrary() {
        File chartsDir = new File(getFilesDir(), CHARTS_DIR);
        if (!chartsDir.exists() && !chartsDir.mkdirs()) {
            Toast.makeText(this, "Não foi possível abrir a biblioteca de cartas.", Toast.LENGTH_LONG).show();
            return;
        }

        File[] files = chartsDir.listFiles(file -> {
            String lower = file.getName().toLowerCase(Locale.ROOT);
            return file.isFile() && (lower.endsWith(".kap") || lower.endsWith(".bsb") || lower.endsWith(".mbtiles"));
        });

        List<File> charts = new ArrayList<>();
        if (files != null) {
            for (File file : files) charts.add(file);
            charts.sort((a, b) -> a.getName().compareToIgnoreCase(b.getName()));
        }

        List<String> labels = new ArrayList<>();
        labels.add("＋ Importar nova carta");
        labels.add("📁 Importar pasta de cartas");
        for (File file : charts) labels.add(file.getName());

        new AlertDialog.Builder(this)
            .setTitle("Biblioteca de Cartas")
            .setItems(labels.toArray(new String[0]), (dialog, which) -> {
                if (which == 0) {
                    chooseChart();
                    return;
                }
                if (which == 1) {
                    chooseChartFolder();
                    return;
                }

                File selected = charts.get(which - 2);
                String lower = selected.getName().toLowerCase(Locale.ROOT);
                String kind = lower.endsWith(".mbtiles") ? "mbtiles" : "kap";
                prefs.edit()
                    .putString("active_chart_name", selected.getName())
                    .putString("active_chart_kind", kind)
                    .putString("active_chart_file", selected.getName())
                    .apply();

                if ("kap".equals(kind)) loadKap(selected, selected.getName());
                else loadMbTiles(selected, selected.getName());
            })
            .setNegativeButton("Fechar", null)
            .show();
    }

    @Override
    protected void onStart() {
        super.onStart();
        mapView.onStart();
    }

    @Override
    protected void onResume() {
        super.onResume();
        mapView.onResume();
    }

    @Override
    protected void onPause() {
        mapView.onPause();
        super.onPause();
    }

    @Override
    protected void onStop() {
        mapView.onStop();
        super.onStop();
    }

    @Override
    public void onLowMemory() {
        super.onLowMemory();
        mapView.onLowMemory();
    }

    @Override
    protected void onDestroy() {
        gpsHandler.removeCallbacks(gpsFreshnessWatch);
        if (locationManager != null
            && ActivityCompat.checkSelfPermission(this, Manifest.permission.ACCESS_FINE_LOCATION)
            == PackageManager.PERMISSION_GRANTED) {
            locationManager.removeUpdates(this);
            try {
                locationManager.unregisterGnssStatusCallback(gnssCallback);
            } catch (RuntimeException ignored) {
            }
        }
        if (tileServer != null) tileServer.stop();
        if (nmeaUdpReceiver != null) nmeaUdpReceiver.close();
        if (nmeaTcpClient != null) nmeaTcpClient.close();
        mapView.onDestroy();
        super.onDestroy();
    }

    @Override
    protected void onSaveInstanceState(Bundle outState) {
        super.onSaveInstanceState(outState);
        mapView.onSaveInstanceState(outState);
    }
}
