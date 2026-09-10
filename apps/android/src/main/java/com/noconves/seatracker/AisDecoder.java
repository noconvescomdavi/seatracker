package com.noconves.seatracker;

import java.util.HashMap;
import java.util.Map;

public final class AisDecoder {
    public static final class Target {
        public final int mmsi;
        public final Double latitude;
        public final Double longitude;
        public final Float sogKnots;
        public final Float cogDeg;
        public final Integer headingDeg;
        public final long receivedAtMs;

        Target(
            int mmsi,
            Double latitude,
            Double longitude,
            Float sogKnots,
            Float cogDeg,
            Integer headingDeg,
            long receivedAtMs
        ) {
            this.mmsi = mmsi;
            this.latitude = latitude;
            this.longitude = longitude;
            this.sogKnots = sogKnots;
            this.cogDeg = cogDeg;
            this.headingDeg = headingDeg;
            this.receivedAtMs = receivedAtMs;
        }

        public boolean hasPosition() {
            return latitude != null && longitude != null;
        }
    }

    private static final class FragmentGroup {
        final int total;
        final String[] payloads;
        int fillBits;
        long updatedAtMs;

        FragmentGroup(int total, long updatedAtMs) {
            this.total = total;
            this.payloads = new String[total];
            this.updatedAtMs = updatedAtMs;
        }

        boolean complete() {
            for (String payload : payloads) if (payload == null) return false;
            return true;
        }

        String payload() {
            StringBuilder out = new StringBuilder();
            for (String payload : payloads) out.append(payload);
            return out.toString();
        }
    }

    private final Map<String, FragmentGroup> fragments = new HashMap<>();

    public Target push(String sentence, long receivedAtMs) {
        if (sentence == null) return null;
        String raw = sentence.trim();
        if (!raw.startsWith("!")) return null;

        int star = raw.indexOf('*');
        String body = star >= 0 ? raw.substring(1, star) : raw.substring(1);
        String[] f = body.split(",", -1);
        if (f.length < 7 || (!f[0].endsWith("VDM") && !f[0].endsWith("VDO"))) return null;

        try {
            int total = Integer.parseInt(f[1]);
            int number = Integer.parseInt(f[2]);
            String sequence = f[3];
            String channel = f[4];
            String payload = f[5];
            int fillBits = Integer.parseInt(f[6]);

            if (total <= 1) return decode(payload, fillBits, receivedAtMs);
            if (number < 1 || number > total || sequence.isEmpty()) return null;

            String key = sequence + ":" + channel;
            FragmentGroup group = fragments.get(key);
            if (group == null || group.total != total) {
                group = new FragmentGroup(total, receivedAtMs);
                fragments.put(key, group);
            }
            group.payloads[number - 1] = payload;
            group.updatedAtMs = receivedAtMs;
            if (number == total) group.fillBits = fillBits;
            if (!group.complete()) return null;

            fragments.remove(key);
            return decode(group.payload(), group.fillBits, receivedAtMs);
        } catch (RuntimeException ignored) {
            return null;
        }
    }

    public void discardStale(long nowMs, long staleAfterMs) {
        fragments.entrySet().removeIf(
            entry -> nowMs - entry.getValue().updatedAtMs > staleAfterMs
        );
    }

    private static Target decode(String payload, int fillBits, long receivedAtMs) {
        boolean[] bits = payloadBits(payload, fillBits);
        if (bits == null || bits.length < 38) return null;

        int type = (int) unsigned(bits, 0, 6);
        int mmsi = (int) unsigned(bits, 8, 30);

        if (type >= 1 && type <= 3) {
            long sogRaw = unsigned(bits, 50, 10);
            long lonRaw = signed(bits, 61, 28);
            long latRaw = signed(bits, 89, 27);
            long cogRaw = unsigned(bits, 116, 12);
            long headingRaw = unsigned(bits, 128, 9);
            return new Target(
                mmsi,
                validLatitude(latRaw),
                validLongitude(lonRaw),
                sogRaw < 1023 ? sogRaw / 10.0f : null,
                cogRaw < 3600 ? cogRaw / 10.0f : null,
                headingRaw < 511 ? (int) headingRaw : null,
                receivedAtMs
            );
        }

        if (type == 18 || type == 19) {
            long sogRaw = unsigned(bits, 46, 10);
            long lonRaw = signed(bits, 57, 28);
            long latRaw = signed(bits, 85, 27);
            long cogRaw = unsigned(bits, 112, 12);
            long headingRaw = unsigned(bits, 124, 9);
            return new Target(
                mmsi,
                validLatitude(latRaw),
                validLongitude(lonRaw),
                sogRaw < 1023 ? sogRaw / 10.0f : null,
                cogRaw < 3600 ? cogRaw / 10.0f : null,
                headingRaw < 511 ? (int) headingRaw : null,
                receivedAtMs
            );
        }

        return null;
    }

    private static Double validLongitude(long raw) {
        return Math.abs(raw) <= 108_000_000L ? raw / 600_000.0 : null;
    }

    private static Double validLatitude(long raw) {
        return Math.abs(raw) <= 54_000_000L ? raw / 600_000.0 : null;
    }

    private static boolean[] payloadBits(String payload, int fillBits) {
        if (fillBits < 0 || fillBits > 5) return null;
        boolean[] all = new boolean[payload.length() * 6 - fillBits];
        int index = 0;
        for (int i = 0; i < payload.length(); i++) {
            int value = payload.charAt(i) - 48;
            if (value > 40) value -= 8;
            if (value < 0 || value > 63) return null;
            for (int bit = 5; bit >= 0; bit--) {
                if (index < all.length) all[index++] = ((value >> bit) & 1) != 0;
            }
        }
        return all;
    }

    private static long unsigned(boolean[] bits, int start, int length) {
        if (start < 0 || length <= 0 || start + length > bits.length) return 0;
        long value = 0;
        for (int i = start; i < start + length; i++) {
            value = (value << 1) | (bits[i] ? 1L : 0L);
        }
        return value;
    }

    private static long signed(boolean[] bits, int start, int length) {
        long value = unsigned(bits, start, length);
        long sign = 1L << (length - 1);
        if ((value & sign) != 0) value -= 1L << length;
        return value;
    }
}
