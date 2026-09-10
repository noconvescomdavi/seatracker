package com.noconves.seatracker;

import org.maplibre.android.geometry.LatLng;

public final class AisCollisionMonitor {
    public static final class Risk {
        public final double cpaNm;
        public final double tcpaMinutes;
        public final boolean dangerous;

        Risk(double cpaNm, double tcpaMinutes, boolean dangerous) {
            this.cpaNm = cpaNm;
            this.tcpaMinutes = tcpaMinutes;
            this.dangerous = dangerous;
        }
    }

    private AisCollisionMonitor() {}

    public static Risk calculate(
        LatLng own,
        double ownSogKnots,
        double ownCogDeg,
        LatLng target,
        double targetSogKnots,
        double targetCogDeg,
        double cpaLimitNm,
        double tcpaLimitMinutes
    ) {
        if (own == null || target == null) return null;
        if (!finite(ownSogKnots, ownCogDeg, targetSogKnots, targetCogDeg)) return null;

        double meanLat = Math.toRadians((own.getLatitude() + target.getLatitude()) / 2.0);
        double northNm = (target.getLatitude() - own.getLatitude()) * 60.0;
        double eastNm = (target.getLongitude() - own.getLongitude()) * 60.0 * Math.cos(meanLat);

        double[] ownV = velocity(ownSogKnots, ownCogDeg);
        double[] targetV = velocity(targetSogKnots, targetCogDeg);
        double rvEast = targetV[0] - ownV[0];
        double rvNorth = targetV[1] - ownV[1];
        double rv2 = rvEast * rvEast + rvNorth * rvNorth;

        double tcpaHours = 0.0;
        if (rv2 > 1e-9) {
            tcpaHours = -(eastNm * rvEast + northNm * rvNorth) / rv2;
        }
        if (tcpaHours < 0.0) tcpaHours = 0.0;

        double cpaEast = eastNm + rvEast * tcpaHours;
        double cpaNorth = northNm + rvNorth * tcpaHours;
        double cpaNm = Math.hypot(cpaEast, cpaNorth);
        double tcpaMinutes = tcpaHours * 60.0;
        boolean dangerous = cpaNm <= cpaLimitNm && tcpaMinutes <= tcpaLimitMinutes;
        return new Risk(cpaNm, tcpaMinutes, dangerous);
    }

    private static double[] velocity(double knots, double courseDeg) {
        double rad = Math.toRadians(courseDeg);
        return new double[]{knots * Math.sin(rad), knots * Math.cos(rad)};
    }

    private static boolean finite(double... values) {
        for (double value : values) if (!Double.isFinite(value)) return false;
        return true;
    }
}
