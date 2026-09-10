package com.noconves.seatracker;

import org.maplibre.android.geometry.LatLng;

import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.FileInputStream;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Locale;

public final class KapChartCatalog {
    public static final class Entry {
        public final File file;
        public final String name;
        public final int scale;
        public final int width;
        public final int height;
        public final double minLat;
        public final double minLon;
        public final double maxLat;
        public final double maxLon;

        Entry(File file, String name, int scale, int width, int height,
              double minLat, double minLon, double maxLat, double maxLon) {
            this.file = file;
            this.name = name;
            this.scale = scale;
            this.width = width;
            this.height = height;
            this.minLat = minLat;
            this.minLon = minLon;
            this.maxLat = maxLat;
            this.maxLon = maxLon;
        }

        public boolean contains(LatLng point) {
            if (point == null) return false;
            double lat = point.getLatitude();
            double lon = point.getLongitude();
            return lat >= minLat && lat <= maxLat && lon >= minLon && lon <= maxLon;
        }
    }

    private KapChartCatalog() {}

    public static List<Entry> scan(File directory) {
        List<Entry> out = new ArrayList<>();
        if (directory == null || !directory.isDirectory()) return out;
        File[] files = directory.listFiles();
        if (files == null) return out;
        for (File file : files) {
            String lower = file.getName().toLowerCase(Locale.ROOT);
            if (!file.isFile() || (!lower.endsWith(".kap") && !lower.endsWith(".bsb"))) continue;
            try {
                Entry entry = inspect(file);
                if (entry != null) out.add(entry);
            } catch (Exception ignored) {
            }
        }
        out.sort(Comparator.comparingInt(entry -> entry.scale <= 0 ? Integer.MAX_VALUE : entry.scale));
        return out;
    }

    public static Entry inspect(File file) throws Exception {
        byte[] header = readHeader(file, 512 * 1024);
        String text = new String(header, StandardCharsets.ISO_8859_1).replace("\r", "");
        String name = file.getName();
        int scale = 0;
        int width = 0;
        int height = 0;
        double minLat = Double.POSITIVE_INFINITY;
        double minLon = Double.POSITIVE_INFINITY;
        double maxLat = Double.NEGATIVE_INFINITY;
        double maxLon = Double.NEGATIVE_INFINITY;
        int refs = 0;

        for (String line : text.split("\n")) {
            String trimmed = line.trim();
            if (trimmed.startsWith("BSB/")) {
                for (String token : trimmed.substring(4).split(",")) {
                    String[] pair = token.split("=", 2);
                    if (pair.length != 2) continue;
                    if (pair[0].trim().equalsIgnoreCase("NA")) name = pair[1].trim();
                    if (pair[0].trim().equalsIgnoreCase("RA")) {
                        String[] dims = pair[1].split(",");
                        if (dims.length >= 2) {
                            width = parseInt(dims[0], width);
                            height = parseInt(dims[1], height);
                        }
                    }
                }
            } else if (trimmed.startsWith("KNP/")) {
                for (String token : trimmed.substring(4).split(",")) {
                    String[] pair = token.split("=", 2);
                    if (pair.length == 2 && pair[0].trim().equalsIgnoreCase("SC")) {
                        scale = parseInt(pair[1], scale);
                    }
                }
            } else if (trimmed.startsWith("REF/")) {
                String[] f = trimmed.substring(4).split(",");
                if (f.length >= 5) {
                    try {
                        double lat = Double.parseDouble(f[3].trim());
                        double lon = Double.parseDouble(f[4].trim());
                        if (lat >= -90 && lat <= 90 && lon >= -180 && lon <= 180) {
                            minLat = Math.min(minLat, lat);
                            maxLat = Math.max(maxLat, lat);
                            minLon = Math.min(minLon, lon);
                            maxLon = Math.max(maxLon, lon);
                            refs++;
                        }
                    } catch (NumberFormatException ignored) {
                    }
                }
            }
        }

        if (refs < 2 || !Double.isFinite(minLat) || !Double.isFinite(minLon)) return null;
        return new Entry(file, name, scale, width, height, minLat, minLon, maxLat, maxLon);
    }

    public static Entry bestFor(List<Entry> entries, LatLng position, Entry current) {
        if (position == null || entries == null || entries.isEmpty()) return current;
        if (current != null && current.contains(position)) {
            Entry better = null;
            for (Entry entry : entries) {
                if (!entry.contains(position)) continue;
                if (entry.scale > 0 && current.scale > 0 && entry.scale < current.scale / 2) {
                    if (better == null || entry.scale < better.scale) better = entry;
                }
            }
            return better == null ? current : better;
        }
        Entry best = null;
        for (Entry entry : entries) {
            if (!entry.contains(position)) continue;
            if (best == null) best = entry;
            else if (entry.scale > 0 && (best.scale <= 0 || entry.scale < best.scale)) best = entry;
        }
        return best;
    }

    private static byte[] readHeader(File file, int maxBytes) throws Exception {
        try (FileInputStream in = new FileInputStream(file);
             ByteArrayOutputStream out = new ByteArrayOutputStream()) {
            int b;
            while (out.size() < maxBytes && (b = in.read()) != -1) {
                if (b == 0x1A || b == 0x00) break;
                out.write(b);
            }
            return out.toByteArray();
        }
    }

    private static int parseInt(String value, int fallback) {
        try { return Integer.parseInt(value.trim()); }
        catch (NumberFormatException ignored) { return fallback; }
    }
}
