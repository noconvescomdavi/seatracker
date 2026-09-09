package com.noconves.seatracker;

import android.Manifest;
import android.app.Activity;
import android.content.Intent;
import android.content.SharedPreferences;
import android.content.pm.PackageManager;
import android.database.Cursor;
import android.location.Location;
import android.location.LocationListener;
import android.location.LocationManager;
import android.net.Uri;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.os.SystemClock;
import android.provider.OpenableColumns;
import android.widget.Button;
import android.widget.TextView;
import android.widget.Toast;

import androidx.activity.result.ActivityResultLauncher;
import androidx.activity.result.contract.ActivityResultContracts;
import androidx.appcompat.app.AppCompatActivity;
import androidx.core.app.ActivityCompat;

import org.maplibre.android.MapLibre;
import org.maplibre.android.annotations.Marker;
import org.maplibre.android.annotations.MarkerOptions;
import org.maplibre.android.annotations.Polyline;
import org.maplibre.android.annotations.PolylineOptions;
import org.maplibre.android.camera.CameraPosition;
import org.maplibre.android.camera.CameraUpdateFactory;
import org.maplibre.android.geometry.LatLng;
import org.maplibre.android.maps.MapLibreMap;
import org.maplibre.android.maps.MapView;
import org.maplibre.android.style.layers.RasterLayer;
import org.maplibre.android.style.sources.RasterSource;
import org.maplibre.android.style.sources.TileSet;

import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

public class MainActivity extends AppCompatActivity implements LocationListener {
    private static final int REQ_LOCATION = 40;
    private static final long GPS_STALE_MS = 10_000L;
    private static final String ACTIVE_MBTILES = "active.mbtiles";

    private MapView mapView;
    private MapLibreMap map;
    private LocationManager locationManager;
    private TextView status;
    private TextView chartStatus;
    private Marker ownShip;
    private Marker mobMarker;
    private Polyline trackLine;
    private Polyline routeLine;
    private final List<LatLng> trackPoints = new ArrayList<>();
    private final List<LatLng> routePoints = new ArrayList<>();
    private boolean tracking = false;
    private boolean routeEditing = false;
    private boolean cameraCenteredOnGps = false;
    private Location lastLocation;
    private SharedPreferences prefs;
    private MbTilesTileServer tileServer;
    private int tileServerPort = -1;

    private final Handler gpsHandler = new Handler(Looper.getMainLooper());
    private long lastFixElapsedMs = 0L;
    private final Runnable gpsFreshnessWatch = new Runnable() {
        @Override
        public void run() {
            updateGpsFreshnessUi();
            gpsHandler.postDelayed(this, 2_000L);
        }
    };

    private final ActivityResultLauncher<Intent> chartPicker =
        registerForActivityResult(new ActivityResultContracts.StartActivityForResult(), result -> {
            if (result.getResultCode() != Activity.RESULT_OK || result.getData() == null) return;
            Uri uri = result.getData().getData();
            if (uri == null) return;
            importChart(uri);
        });

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        MapLibre.getInstance(this);
        setContentView(R.layout.activity_main);

        prefs = getSharedPreferences("seatracker", MODE_PRIVATE);
        status = findViewById(R.id.status);
        chartStatus = findViewById(R.id.chartStatus);
        mapView = findViewById(R.id.mapView);
        mapView.onCreate(savedInstanceState);

        restoreRoute();

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
                        map.addMarker(new MarkerOptions().position(point).title("Waypoint"));
                    }
                    return true;
                });

                redrawRoute();
                restoreActiveMbTiles();
            });
        });

        Button routeButton = findViewById(R.id.btnRoute);
        routeButton.setOnClickListener(v -> {
            routeEditing = !routeEditing;
            routeButton.setText(routeEditing ? "ROTA*" : "ROTA");
            Toast.makeText(
                this,
                routeEditing
                    ? "Modo rota: toque e segure na carta para adicionar pernas."
                    : "Rota salva.",
                Toast.LENGTH_SHORT
            ).show();
            persistRoute();
        });
        routeButton.setOnLongClickListener(v -> {
            clearRoute();
            Toast.makeText(this, "Rota apagada.", Toast.LENGTH_SHORT).show();
            return true;
        });

        findViewById(R.id.btnMob).setOnClickListener(v -> activateMob());
        findViewById(R.id.btnTrack).setOnClickListener(v -> toggleTrack((Button) v));
        findViewById(R.id.btnImport).setOnClickListener(v -> chooseChart());

        locationManager = (LocationManager) getSystemService(LOCATION_SERVICE);
        requestLocation();
        gpsHandler.post(gpsFreshnessWatch);
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
                trackPoints.add(position);
                redrawTrack();
            }
        }

        double accuracy = location.hasAccuracy() ? location.getAccuracy() : Double.NaN;
        status.setText(String.format(
            Locale.US,
            "GPS OK • LAT %.6f  LON %.6f  ACC %.0fm  SOG %.1f kn  COG %.0f°",
            location.getLatitude(),
            location.getLongitude(),
            accuracy,
            location.hasSpeed() ? location.getSpeed() * 1.943844f : 0f,
            location.hasBearing() ? location.getBearing() : 0f
        ));
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
            status.setText("SeaTracker • procurando sinal GPS…");
            return;
        }
        long age = SystemClock.elapsedRealtime() - lastFixElapsedMs;
        if (age > GPS_STALE_MS) {
            status.setText(String.format(
                Locale.US,
                "SeaTracker • posição GPS desatualizada (%ds)",
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

    private void activateMob() {
        if (!hasFreshGpsFix() || map == null) {
            Toast.makeText(
                this,
                "MOB indisponível: posição GPS ainda não é válida.",
                Toast.LENGTH_SHORT
            ).show();
            return;
        }

        LatLng position = new LatLng(lastLocation.getLatitude(), lastLocation.getLongitude());
        if (mobMarker != null) {
            map.removeMarker(mobMarker);
        }
        mobMarker = map.addMarker(new MarkerOptions().position(position).title("MOB"));
        map.animateCamera(CameraUpdateFactory.newLatLngZoom(position, 14.0));
        prefs.edit()
            .putFloat("mob_lat", (float) position.getLatitude())
            .putFloat("mob_lon", (float) position.getLongitude())
            .putLong("mob_time", System.currentTimeMillis())
            .apply();
        Toast.makeText(this, "MOB registrado e persistido.", Toast.LENGTH_LONG).show();
    }

    private void toggleTrack(Button button) {
        tracking = !tracking;
        button.setText(tracking ? "STOP" : "TRACK");
        if (tracking) {
            trackPoints.clear();
            if (hasFreshGpsFix()) {
                trackPoints.add(new LatLng(lastLocation.getLatitude(), lastLocation.getLongitude()));
            }
            redrawTrack();
        }
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

    private void addRoutePoint(LatLng point) {
        routePoints.add(point);
        if (map != null) {
            map.addMarker(new MarkerOptions()
                .position(point)
                .title("Rota WP " + routePoints.size()));
        }
        redrawRoute();
        persistRoute();
    }

    private void redrawRoute() {
        if (map == null) return;
        if (routeLine != null) {
            map.removePolyline(routeLine);
            routeLine = null;
        }
        if (routePoints.size() >= 2) {
            routeLine = map.addPolyline(new PolylineOptions().addAll(routePoints).width(6f));
        }
    }

    private void persistRoute() {
        StringBuilder value = new StringBuilder();
        for (LatLng point : routePoints) {
            if (value.length() > 0) value.append(';');
            value.append(point.getLatitude()).append(',').append(point.getLongitude());
        }
        prefs.edit().putString("active_route", value.toString()).apply();
    }

    private void restoreRoute() {
        routePoints.clear();
        String saved = prefs.getString("active_route", "");
        if (saved == null || saved.isEmpty()) return;
        for (String item : saved.split(";")) {
            String[] parts = item.split(",");
            if (parts.length != 2) continue;
            try {
                routePoints.add(new LatLng(
                    Double.parseDouble(parts[0]),
                    Double.parseDouble(parts[1])
                ));
            } catch (NumberFormatException ignored) {
            }
        }
    }

    private void clearRoute() {
        routePoints.clear();
        persistRoute();
        redrawRoute();
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
        if (name == null || !name.toLowerCase(Locale.ROOT).endsWith(".mbtiles")) {
            chartStatus.setText("Carta não carregada: selecione um arquivo .mbtiles");
            Toast.makeText(
                this,
                "Nesta versão operacional, a carta local renderizável deve estar em MBTiles.",
                Toast.LENGTH_LONG
            ).show();
            return;
        }

        File chartsDir = new File(getFilesDir(), "charts");
        if (!chartsDir.exists() && !chartsDir.mkdirs()) {
            chartStatus.setText("Falha ao criar armazenamento de cartas");
            return;
        }

        File target = new File(chartsDir, ACTIVE_MBTILES);
        try (InputStream in = getContentResolver().openInputStream(uri);
             FileOutputStream out = new FileOutputStream(target, false)) {
            if (in == null) throw new IllegalStateException("Não foi possível abrir a carta");
            byte[] buffer = new byte[64 * 1024];
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
            prefs.edit().putString("active_chart_name", name).apply();
            loadMbTiles(target, name);
        } catch (Exception e) {
            chartStatus.setText("Falha ao importar carta");
            Toast.makeText(this, "Erro na carta: " + e.getMessage(), Toast.LENGTH_LONG).show();
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

    private void restoreActiveMbTiles() {
        File file = new File(new File(getFilesDir(), "charts"), ACTIVE_MBTILES);
        if (!file.isFile()) return;
        String name = prefs.getString("active_chart_name", ACTIVE_MBTILES);
        loadMbTiles(file, name);
    }

    private void loadMbTiles(File file, String displayName) {
        try {
            if (tileServer != null) {
                tileServer.stop();
            }
            tileServer = new MbTilesTileServer(file);
            tileServerPort = tileServer.start();

            if (map == null) return;
            map.getStyle(style -> {
                if (style.getLayer("chart-raster") != null) {
                    style.removeLayer("chart-raster");
                }
                if (style.getSource("chart-source") != null) {
                    style.removeSource("chart-source");
                }

                String template = "http://127.0.0.1:" + tileServerPort
                    + "/tiles/{z}/{x}/{y}.png";
                TileSet tileSet = new TileSet("2.2.0", template);
                RasterSource source = new RasterSource("chart-source", tileSet, 256);
                RasterLayer layer = new RasterLayer("chart-raster", "chart-source");
                style.addSource(source);
                style.addLayer(layer);
                chartStatus.setText("Carta MBTiles ativa: " + displayName);
            });
        } catch (Exception e) {
            chartStatus.setText("Carta inválida ou incompatível");
            Toast.makeText(this, "Falha MBTiles: " + e.getMessage(), Toast.LENGTH_LONG).show();
        }
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
        if (ActivityCompat.checkSelfPermission(this, Manifest.permission.ACCESS_FINE_LOCATION)
            == PackageManager.PERMISSION_GRANTED) {
            locationManager.removeUpdates(this);
        }
        if (tileServer != null) {
            tileServer.stop();
        }
        mapView.onDestroy();
        super.onDestroy();
    }

    @Override
    protected void onSaveInstanceState(Bundle outState) {
        super.onSaveInstanceState(outState);
        mapView.onSaveInstanceState(outState);
    }
}
