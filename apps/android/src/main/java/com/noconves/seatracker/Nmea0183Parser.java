package com.noconves.seatracker;

import java.util.Locale;

public final class Nmea0183Parser {
    public static final class Update {
        public Double latitude;
        public Double longitude;
        public Float sogKnots;
        public Float cogDeg;
        public Float headingTrueDeg;
        public Float hdop;
        public Integer satellites;
        public boolean positionValid;
        public String type;
    }

    private Nmea0183Parser() {}

    public static Update parse(String sentence) {
        if (sentence == null) return null;
        String value = sentence.trim();
        if (!value.startsWith("$") || !validChecksum(value)) return null;

        int star = value.lastIndexOf('*');
        if (star < 0) return null;
        String[] f = value.substring(1, star).split(",", -1);
        if (f.length == 0 || f[0].length() < 3) return null;
        String type = f[0].substring(f[0].length() - 3).toUpperCase(Locale.ROOT);

        Update update = new Update();
        update.type = type;

        try {
            switch (type) {
                case "RMC":
                    if (f.length < 9) return null;
                    update.positionValid = "A".equals(f[2]);
                    update.latitude = coordinate(f[3], f[4]);
                    update.longitude = coordinate(f[5], f[6]);
                    update.sogKnots = number(f[7]);
                    update.cogDeg = number(f[8]);
                    return update;
                case "GGA":
                    if (f.length < 9) return null;
                    update.latitude = coordinate(f[2], f[3]);
                    update.longitude = coordinate(f[4], f[5]);
                    update.positionValid = integer(f[6]) != null && integer(f[6]) > 0;
                    update.satellites = integer(f[7]);
                    update.hdop = number(f[8]);
                    return update;
                case "VTG":
                    if (f.length < 6) return null;
                    update.cogDeg = number(f[1]);
                    update.sogKnots = number(f[5]);
                    return update;
                case "HDT":
                    if (f.length < 2) return null;
                    update.headingTrueDeg = number(f[1]);
                    return update;
                default:
                    return null;
            }
        } catch (RuntimeException ignored) {
            return null;
        }
    }

    public static boolean validChecksum(String sentence) {
        int star = sentence.lastIndexOf('*');
        if (star < 2 || star + 2 >= sentence.length()) return false;
        int checksum = 0;
        for (int i = 1; i < star; i++) checksum ^= sentence.charAt(i);
        try {
            int expected = Integer.parseInt(sentence.substring(star + 1, star + 3), 16);
            return checksum == expected;
        } catch (NumberFormatException ignored) {
            return false;
        }
    }

    private static Double coordinate(String raw, String hemi) {
        if (raw == null || raw.isEmpty()) return null;
        double numeric = Double.parseDouble(raw);
        double degrees = Math.floor(numeric / 100.0);
        double minutes = numeric - degrees * 100.0;
        double result = degrees + minutes / 60.0;
        if ("S".equalsIgnoreCase(hemi) || "W".equalsIgnoreCase(hemi)) result = -result;
        return result;
    }

    private static Float number(String value) {
        if (value == null || value.isEmpty()) return null;
        return Float.parseFloat(value);
    }

    private static Integer integer(String value) {
        if (value == null || value.isEmpty()) return null;
        return Integer.parseInt(value);
    }
}
