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

    private final ActivityResultLauncher<Intent> chartPicker =
        registerForActivityResult(new ActivityResultContracts.StartActivityForResult(), result -> {
            if (result.getResultCode() != Activity.RESULT_OK || result.getData() == null) return;
            Uri uri = result.getData().getData();
            if (uri == null) return;
            try {
                getContentResolver().takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION);
            } catch (SecurityException ignored) { }
            Toast.makeText(this,
                "Carta selecionada. O ChartProvider validará o formato antes da indexação.",
                Toast.LENGTH_LONG).show();
        });

    @Override protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        MapLibre.getInstance(this);
        setContentView(R.layout.activity_main);
        status = findViewById(R.id.status);
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
    }

    private void requestLocation() {
        if (ActivityCompat.checkSelfPermission(this, Manifest.permission.ACCESS_FINE_LOCATION) != PackageManager.PERMISSION_GRANTED) {
            ActivityCompat.requestPermissions(this,
                new String[]{Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.ACCESS_COARSE_LOCATION}, REQ_LOCATION);
            return;
        }
        locationManager.requestLocationUpdates(LocationManager.GPS_PROVIDER, 1000, 1f, this);
    }

    @Override public void onRequestPermissionsResult(int requestCode, String[] permissions, int[] grantResults) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults);
        if (requestCode == REQ_LOCATION && grantResults.length > 0 && grantResults[0] == PackageManager.PERMISSION_GRANTED) requestLocation();
        else status.setText("SeaTracker • GPS sem permissão");
    }

    @Override public void onLocationChanged(Location location) {
        lastLocation = location;
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
        status.setText(String.format(Locale.US, "LAT %.6f  LON %.6f  SOG %.1f kn  COG %.0f°",
            location.getLatitude(), location.getLongitude(),
            location.hasSpeed() ? location.getSpeed() * 1.943844f : 0f,
            location.hasBearing() ? location.getBearing() : 0f));
    }

    private void activateMob() {
        if (lastLocation == null || map == null) {
            Toast.makeText(this, "MOB indisponível: posição GPS ainda não válida.", Toast.LENGTH_SHORT).show();
            return;
        }
        LatLng p = new LatLng(lastLocation.getLatitude(), lastLocation.getLongitude());
        if (mobMarker != null) map.removeMarker(mobMarker);
        mobMarker = map.addMarker(new MarkerOptions().position(p).title("MOB"));
        map.animateCamera(CameraUpdateFactory.newLatLngZoom(p, 14));
        Toast.makeText(this, "MOB registrado imediatamente.", Toast.LENGTH_LONG).show();
    }

    private void addWaypointAtShip() {
        if (lastLocation == null || map == null) {
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
        if (tracking && lastLocation != null) {
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
