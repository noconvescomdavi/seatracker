package com.noconves.seatracker;

import org.maplibre.android.geometry.LatLng;

import java.io.File;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Locale;

public final class Cm93ChartCatalog {
    public static final class Entry {
        public final File file;
        public final String name;
        public final int scale;
        public final double minLat, minLon, maxLat, maxLon;

        Entry(File file, int scale, double minLat, double minLon, double maxLat, double maxLon) {
            this.file = file;
            this.name = file.getName();
            this.scale = scale;
            this.minLat = minLat;
            this.minLon = minLon;
            this.maxLat = maxLat;
            this.maxLon = maxLon;
        }

        public boolean contains(LatLng point) {
            if (point == null) return false;
            double lat = point.getLatitude();
            double lon = point.getLongitude();
            boolean latOk = lat >= minLat && lat <= maxLat;
            if (!latOk) return false;
            if (minLon <= maxLon) return lon >= minLon && lon <= maxLon;
            return lon >= minLon || lon <= maxLon;
        }
    }

    private Cm93ChartCatalog() {}

    public static List<Entry> scan(File root) {
        List<Entry> entries = new ArrayList<>();
        scanRecursive(root, entries, 0, 100_000);
        entries.sort(Comparator.comparingInt((Entry e) -> e.scale).thenComparing(e -> e.name));
        return entries;
    }

    private static void scanRecursive(File dir, List<Entry> out, int depth, int limit) {
        if (dir == null || !dir.isDirectory() || depth > 12 || out.size() >= limit) return;
        File[] files = dir.listFiles();
        if (files == null) return;
        for (File file : files) {
            if (out.size() >= limit) return;
            if (file.isDirectory()) {
                scanRecursive(file, out, depth + 1, limit);
                continue;
            }
            Entry entry = fromFile(file);
            if (entry != null) out.add(entry);
        }
    }

    public static Entry fromFile(File file) {
        if (file == null || !file.isFile() || !Cm93ChartDecoder.isCm93CellName(file.getName())) return null;
        String n = file.getName().toUpperCase(Locale.ROOT);
        if (n.endsWith(".XZ")) n = n.substring(0, n.length() - 3);
        int dot = n.lastIndexOf('.');
        if (dot < 0 || dot != n.length() - 2) return null;
        String stem = n.substring(0, dot);
        if (stem.length() != 8) return null;

        String normalized = "0" + stem.substring(1);
        final int latIndex;
        final int lonIndex;
        try {
            latIndex = Integer.parseInt(normalized.substring(0, 4));
            lonIndex = Integer.parseInt(normalized.substring(4, 8));
        } catch (NumberFormatException error) {
            return null;
        }

        char scaleChar = n.charAt(n.length() - 1);
        int scale = Cm93ChartDecoder.scaleForFileName(n);
        int thirds;
        switch (scaleChar) {
            case 'Z': thirds = 120; break;
            case 'A': thirds = 60; break;
            case 'B': thirds = 30; break;
            case 'C': thirds = 12; break;
            case 'D': thirds = 3; break;
            default: thirds = 1; break;
        }

        double minLat = (latIndex - 270.0) / 3.0;
        double minLon = normalizeLon(lonIndex / 3.0);
        double maxLat = minLat + thirds / 3.0;
        double maxLon = normalizeLon(minLon + thirds / 3.0);
        return new Entry(file, scale, minLat, minLon, maxLat, maxLon);
    }

    public static Entry bestFor(List<Entry> entries, LatLng position, Entry current) {
        if (position == null || entries == null || entries.isEmpty()) return null;
        if (current != null && current.contains(position)) {
            Entry moreDetailed = null;
            for (Entry entry : entries) {
                if (!entry.contains(position) || entry.scale >= current.scale) continue;
                if (moreDetailed == null || entry.scale < moreDetailed.scale) moreDetailed = entry;
            }
            return moreDetailed != null ? moreDetailed : current;
        }
        Entry best = null;
        for (Entry entry : entries) {
            if (!entry.contains(position)) continue;
            if (best == null || entry.scale < best.scale) best = entry;
        }
        return best;
    }

    private static double normalizeLon(double lon) {
        double value = (lon + 180.0) % 360.0;
        if (value < 0) value += 360.0;
        return value - 180.0;
    }
}
