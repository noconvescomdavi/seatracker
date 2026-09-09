package com.noconves.seatracker;

import org.maplibre.android.geometry.LatLng;

import java.util.ArrayList;
import java.util.List;

public final class NavigationMath {
    private static final double EARTH_RADIUS_M = 6_371_008.8;
    private static final double METERS_PER_NM = 1852.0;

    private NavigationMath() {}

    public static double distanceNm(LatLng a, LatLng b) {
        double lat1 = Math.toRadians(a.getLatitude());
        double lat2 = Math.toRadians(b.getLatitude());
        double dLat = lat2 - lat1;
        double dLon = Math.toRadians(normalizeLongitude(b.getLongitude() - a.getLongitude()));
        double h = Math.pow(Math.sin(dLat / 2.0), 2.0)
            + Math.cos(lat1) * Math.cos(lat2) * Math.pow(Math.sin(dLon / 2.0), 2.0);
        double c = 2.0 * Math.atan2(Math.sqrt(h), Math.sqrt(1.0 - h));
        return EARTH_RADIUS_M * c / METERS_PER_NM;
    }

    public static double bearingDeg(LatLng a, LatLng b) {
        double lat1 = Math.toRadians(a.getLatitude());
        double lat2 = Math.toRadians(b.getLatitude());
        double dLon = Math.toRadians(normalizeLongitude(b.getLongitude() - a.getLongitude()));
        double y = Math.sin(dLon) * Math.cos(lat2);
        double x = Math.cos(lat1) * Math.sin(lat2)
            - Math.sin(lat1) * Math.cos(lat2) * Math.cos(dLon);
        double result = Math.toDegrees(Math.atan2(y, x));
        return (result % 360.0 + 360.0) % 360.0;
    }

    public static LatLng destination(LatLng origin, double bearingDeg, double rangeNm) {
        double phi1 = Math.toRadians(origin.getLatitude());
        double lambda1 = Math.toRadians(origin.getLongitude());
        double theta = Math.toRadians(bearingDeg);
        double delta = rangeNm * METERS_PER_NM / EARTH_RADIUS_M;

        double phi2 = Math.asin(
            Math.sin(phi1) * Math.cos(delta)
                + Math.cos(phi1) * Math.sin(delta) * Math.cos(theta)
        );
        double lambda2 = lambda1 + Math.atan2(
            Math.sin(theta) * Math.sin(delta) * Math.cos(phi1),
            Math.cos(delta) - Math.sin(phi1) * Math.sin(phi2)
        );
        return new LatLng(Math.toDegrees(phi2), normalizeLongitude(Math.toDegrees(lambda2)));
    }

    public static List<LatLng> circle(LatLng center, double rangeNm, int segments) {
        List<LatLng> points = new ArrayList<>();
        int count = Math.max(24, segments);
        for (int i = 0; i <= count; i++) {
            points.add(destination(center, 360.0 * i / count, rangeNm));
        }
        return points;
    }

    private static double normalizeLongitude(double lon) {
        while (lon > 180.0) lon -= 360.0;
        while (lon < -180.0) lon += 360.0;
        return lon;
    }
}
