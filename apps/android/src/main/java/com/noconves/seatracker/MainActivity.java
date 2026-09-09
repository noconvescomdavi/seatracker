package com.noconves.seatracker;

import android.Manifest;
import android.app.Activity;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.location.Location;
import android.location.LocationListener;
import android.location.LocationManager;
import android.net.Uri;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.os.SystemClock;
import android.content.SharedPreferences;
import java.util.HashSet;
import java.util.Set;
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

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

public class MainActivity extends AppCompatActivity implements LocationListener {
    private static final int REQ_LOCATION = 40;
    private MapView mapView;
    private MapLibreMap map;
    private LocationManager locationManager;
    private TextView status;
    private Marker ownShip;
    private Marker mobMarker;
    private final List<LatLng> trackPoints = new ArrayList<>();
    private Polyline trackLine;
    private boolean tracking = false;
    private Location lastLocation;
    private SharedPreferences prefs;
    private final Handler gpsHandler = new Handler(Looper.getMainLooper());
    private long lastFixElapsedMs = 0L;
    private static final long GPS_STALE_MS = 10_000L;
    private final Runnable gpsFreshnessWatch = new Runnable() {
        @Override public void run() {
            updateGpsFreshnessUi();
            gpsHandler.postDelayed(this, 2_000L);
        }
    };

    private final ActivityResultLauncher<Intent> chartPicker =
        registerForActivityResult(new ActivityResultContracts.StartActivityForResult(), result -> {
            if (result.getResultCode() != Activity.RESULT_OK || result.getData() == null) return;
            Uri uri = result.getData().getData();
            if (uri == null) return;
            try {
                getContentResolver().takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION);
            } catch (SecurityException ignored) { }
            Set<String> charts = new HashSet<>(prefs.getStringSet("chart_uris", new HashSet<>()));
            charts.add(uri.toString());
            prefs.edit().putStringSet("chart_uris", charts).apply();
            Toast.makeText(this,
                "Carta adicionada ao catálogo local (" + charts.size() + "). O ChartProvider validará o formato antes da indexação.",
                Toast.LENGTH_LONG).show();
        });

    @Override protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        MapLibre.getInstance(this);
        setContentView(R.layout.activity_main);
        status = findViewById(R.id.status);
        prefs = getSharedPreferences("seatracker", MODE_PRIVATE);
        mapView = findViewById(R.id.mapView);
        mapView.onCreate(savedInstanceState);
        mapView.getMapAsync(m -> {
            map = m;
            map.setStyle("asset://offline-style.json", style -> {
                map.setCameraPosition(new CameraPosition.Builder()
                    .target(new LatLng(-22.9, -43.2)).zoom(5.0).build());
                map.addOnMapLongClickListener(point -> {
                    map.addMarker(new MarkerOptions().position(point).title("Waypoint"));
                    return true;
                });
            });
        });
        findViewById(R.id.btnMob).setOnClickListener(v -> activateMob());
        findViewById(R.id.btnWp).setOnClickListener(v -> addWaypointAtShip());
        findViewById(R.id.btnTrack).setOnClickListener(v -> toggleTrack((Button) v));
        findViewById(R.id.btnImport).setOnClickListener(v -> importChart());
        locationManager = (LocationManager) getSystemService(LOCATION_SERVICE);
        requestLocation();
        gpsHandler.post(gpsFreshnessWatch);
    }

    private void requestLocation() {
        if (ActivityCompat.checkSelfPermission(this, Manifest.permission.ACCESS_FINE_LOCATION) != PackageManager.PERMISSION_GRANTED) {
            ActivityCompat.requestPermissions(this,
                new String[]{Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.ACCESS_COARSE_LOCATION}, REQ_LOCATION);
            return;
        }
        if (!locationManager.isProviderEnabled(LocationManager.GPS_PROVIDER)) {
            status.setText("SeaTracker • GPS desativado");
            return;
        }
        status.setText("SeaTracker • procurando sinal GPS…");
        locationManager.requestLocationUpdates(LocationManager.GPS_PROVIDER, 1000, 0f, this);
    }

    @Override public void onRequestPermissionsResult(int requestCode, String[] permissions, int[] grantResults) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults);
        if (requestCode == REQ_LOCATION && grantResults.length > 0 && grantResults[0] == PackageManager.PERMISSION_GRANTED) requestLocation();
        else status.setText("SeaTracker • GPS sem permissão");
    }

    @Override public void onLocationChanged(Location location) {
        lastLocation = location;
        lastFixElapsedMs = SystemClock.elapsedRealtime();
        LatLng p = new LatLng(location.getLatitude(), location.getLongitude());
        if (map != null) {
            if (ownShip == null) ownShip = map.addMarker(new MarkerOptions().position(p).title("Own Ship"));
            else ownShip.setPosition(p);
            if (tracking) {
                trackPoints.add(p);
                if (trackLine != null) map.removePolyline(trackLine);
                trackLine = map.addPolyline(new PolylineOptions().addAll(trackPoints).width(4f));
            }
        }
        double accuracy = location.hasAccuracy() ? location.getAccuracy() : Double.NaN;
        status.setText(String.format(Locale.US,
            "GPS OK • LAT %.6f  LON %.6f  ACC %.0fm  SOG %.1f kn  COG %.0f°",
            location.getLatitude(), location.getLongitude(), accuracy,
            location.hasSpeed() ? location.getSpeed() * 1.943844f : 0f,
            location.hasBearing() ? location.getBearing() : 0f));
    }

    private void updateGpsFreshnessUi() {
        if (ActivityCompat.checkSelfPermission(this, Manifest.permission.ACCESS_FINE_LOCATION) != PackageManager.PERMISSION_GRANTED) {
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
            status.setText(String.format(Locale.US, "SeaTracker • posição GPS desatualizada (%ds)", age / 1000));
        }
    }

    @Override public void onProviderDisabled(String provider) {
        if (LocationManager.GPS_PROVIDER.equals(provider)) {
            status.setText("SeaTracker • GPS desativado");
        }
    }

    @Override public void onProviderEnabled(String provider) {
        if (LocationManager.GPS_PROVIDER.equals(provider)) {
            status.setText("SeaTracker • procurando sinal GPS…");
        }
    }

    private boolean hasFreshGpsFix() {
        return lastLocation != null
            && lastFixElapsedMs > 0L
            && (SystemClock.elapsedRealtime() - lastFixElapsedMs) <= GPS_STALE_MS;
    }

    private void activateMob() {
        if (!hasFreshGpsFix() || map == null) {
            Toast.makeText(this, "MOB indisponível: posição GPS ainda não válida.", Toast.LENGTH_SHORT).show();
            return;
        }
        LatLng p = new LatLng(lastLocation.getLatitude(), lastLocation.getLongitude());
        if (mobMarker != null) map.removeMarker(mobMarker);
        mobMarker = map.addMarker(new MarkerOptions().position(p).title("MOB"));
        map.animateCamera(CameraUpdateFactory.newLatLngZoom(p, 14));
        prefs.edit()
            .putFloat("mob_lat", (float) p.getLatitude())
            .putFloat("mob_lon", (float) p.getLongitude())
            .putLong("mob_time", System.currentTimeMillis())
            .apply();
        Toast.makeText(this, "MOB registrado e persistido.", Toast.LENGTH_LONG).show();
    }

    private void addWaypointAtShip() {
        if (!hasFreshGpsFix() || map == null) {
            Toast.makeText(this, "Sem posição válida para criar waypoint.", Toast.LENGTH_SHORT).show();
            return;
        }
        map.addMarker(new MarkerOptions()
            .position(new LatLng(lastLocation.getLatitude(), lastLocation.getLongitude()))
            .title("Waypoint"));
    }

    private void toggleTrack(Button button) {
        tracking = !tracking;
        button.setText(tracking ? "STOP" : "TRACK");
        if (tracking && hasFreshGpsFix()) {
            trackPoints.clear();
            trackPoints.add(new LatLng(lastLocation.getLatitude(), lastLocation.getLongitude()));
        }
    }

    private void importChart() {
        Intent intent = new Intent(Intent.ACTION_OPEN_DOCUMENT);
        intent.addCategory(Intent.CATEGORY_OPENABLE);
        intent.setType("*/*");
        intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION | Intent.FLAG_GRANT_PERSISTABLE_URI_PERMISSION);
        chartPicker.launch(intent);
    }

    @Override protected void onStart() { super.onStart(); mapView.onStart(); }
    @Override protected void onResume() { super.onResume(); mapView.onResume(); }
    @Override protected void onPause() { mapView.onPause(); super.onPause(); }
    @Override protected void onStop() { mapView.onStop(); super.onStop(); }
    @Override public void onLowMemory() { super.onLowMemory(); mapView.onLowMemory(); }
    @Override protected void onDestroy() {
        gpsHandler.removeCallbacks(gpsFreshnessWatch);
        if (ActivityCompat.checkSelfPermission(this, Manifest.permission.ACCESS_FINE_LOCATION) == PackageManager.PERMISSION_GRANTED) {
            locationManager.removeUpdates(this);
        }
        mapView.onDestroy();
        super.onDestroy();
    }
    @Override protected void onSaveInstanceState(Bundle outState) {
        super.onSaveInstanceState(outState);
        mapView.onSaveInstanceState(outState);
    }
}
