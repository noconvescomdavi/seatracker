package com.noconves.seatracker;

import org.maplibre.android.geometry.LatLng;
import java.util.List;

public final class RouteNavigator {
    public static final class Guidance {
        public final int legIndex;
        public final LatLng destination;
        public final double distanceNm;
        public final double bearingDeg;
        public final double xteNm;
        public final long etaMillis;
        public final boolean arrival;

        Guidance(int legIndex, LatLng destination, double distanceNm, double bearingDeg,
                 double xteNm, long etaMillis, boolean arrival) {
            this.legIndex = legIndex;
            this.destination = destination;
            this.distanceNm = distanceNm;
            this.bearingDeg = bearingDeg;
            this.xteNm = xteNm;
            this.etaMillis = etaMillis;
            this.arrival = arrival;
        }
    }

    private int activeLeg = 1;
    private double arrivalRadiusNm = 0.05;

    public void reset() { activeLeg = 1; }
    public int getActiveLeg() { return activeLeg; }
    public void setArrivalRadiusNm(double value) {
        if (Double.isFinite(value) && value >= 0.01 && value <= 2.0) arrivalRadiusNm = value;
    }

    public Guidance update(List<LatLng> route, LatLng own, double sogKnots, long nowMillis) {
        if (route == null || own == null || route.size() < 2) return null;
        if (activeLeg < 1) activeLeg = 1;
        if (activeLeg >= route.size()) activeLeg = route.size() - 1;

        LatLng from = route.get(activeLeg - 1);
        LatLng to = route.get(activeLeg);
        double distance = NavigationMath.distanceNm(own, to);
        boolean arrival = distance <= arrivalRadiusNm;
        if (arrival && activeLeg < route.size() - 1) {
            activeLeg++;
            from = route.get(activeLeg - 1);
            to = route.get(activeLeg);
            distance = NavigationMath.distanceNm(own, to);
            arrival = distance <= arrivalRadiusNm;
        }

        double bearing = NavigationMath.bearingDeg(own, to);
        double xte = crossTrackNm(from, to, own);
        long eta = 0L;
        if (Double.isFinite(sogKnots) && sogKnots > 0.2) {
            double millis = distance / sogKnots * 3_600_000.0;
            if (Double.isFinite(millis) && millis >= 0 && millis <= Long.MAX_VALUE - nowMillis) {
                eta = nowMillis + Math.round(millis);
            }
        }
        return new Guidance(activeLeg, to, distance, bearing, xte, eta, arrival);
    }

    private static double crossTrackNm(LatLng start, LatLng end, LatLng point) {
        final double earthRadiusNm = 3440.065;
        double d13 = NavigationMath.distanceNm(start, point) / earthRadiusNm;
        double brg13 = Math.toRadians(NavigationMath.bearingDeg(start, point));
        double brg12 = Math.toRadians(NavigationMath.bearingDeg(start, end));
        double value = Math.sin(d13) * Math.sin(brg13 - brg12);
        value = Math.max(-1.0, Math.min(1.0, value));
        return Math.asin(value) * earthRadiusNm;
    }
}
