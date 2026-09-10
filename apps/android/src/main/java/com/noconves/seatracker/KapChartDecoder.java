package com.noconves.seatracker;

import android.graphics.Bitmap;

import org.maplibre.android.geometry.LatLng;
import org.maplibre.android.geometry.LatLngQuad;

import java.io.File;
import java.io.IOException;
import java.io.RandomAccessFile;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;

/**
 * Independent BSB/KAP raster reader used by SeaTracker.
 *
 * Reads the textual BSB header, palette, row index and run-length encoded
 * raster. The decoded bitmap is downsampled to a bounded texture size for
 * safe use as a MapLibre ImageSource on Android.
 */
public final class KapChartDecoder {
    private static final int MAX_HEADER_BYTES = 512 * 1024;
    private static final int MAX_TEXTURE_DIMENSION = 4096;
    private static final long MAX_PIXELS = 250_000_000L;

    public static final class Result {
        public final String name;
        public final Bitmap bitmap;
        public final LatLngQuad quad;
        public final int sourceWidth;
        public final int sourceHeight;
        public final int bitsPerPixel;
        public final Integer scale;
        public final String datum;
        public final String projection;
        public final double centerLat;
        public final double centerLon;
        public final double minLat;
        public final double minLon;
        public final double maxLat;
        public final double maxLon;

        Result(
            String name,
            Bitmap bitmap,
            LatLngQuad quad,
            int sourceWidth,
            int sourceHeight,
            int bitsPerPixel,
            Integer scale,
            String datum,
            String projection,
            double centerLat,
            double centerLon,
            double minLat,
            double minLon,
            double maxLat,
            double maxLon
        ) {
            this.name = name;
            this.bitmap = bitmap;
            this.quad = quad;
            this.sourceWidth = sourceWidth;
            this.sourceHeight = sourceHeight;
            this.bitsPerPixel = bitsPerPixel;
            this.scale = scale;
            this.datum = datum;
            this.projection = projection;
            this.centerLat = centerLat;
            this.centerLon = centerLon;
            this.minLat = minLat;
            this.minLon = minLon;
            this.maxLat = maxLat;
            this.maxLon = maxLon;
        }

        public String summary() {
            return String.format(
                Locale.US,
                "%s • %dx%d • IFM %d%s",
                name,
                sourceWidth,
                sourceHeight,
                bitsPerPixel,
                scale == null ? "" : " • 1:" + scale
            );
        }
    }

    private static final class Header {
        String name = "KAP chart";
        int width;
        int height;
        int bits;
        Integer scale;
        String datum;
        String projection;
        final Map<Integer, Integer> palette = new HashMap<>();
        final List<Reference> refs = new ArrayList<>();
    }

    private static final class Reference {
        final double x;
        final double y;
        final double lat;
        final double lon;

        Reference(double x, double y, double lat, double lon) {
            this.x = x;
            this.y = y;
            this.lat = lat;
            this.lon = lon;
        }
    }

    private KapChartDecoder() {}

    public static Result decode(File file) throws IOException {
        try (RandomAccessFile raf = new RandomAccessFile(file, "r")) {
            if (raf.length() < 32) throw new IOException("Arquivo KAP muito pequeno");

            Header header = readHeader(raf);
            validateHeader(header);
            long[] rowOffsets = readRowOffsets(raf, header.height);

            int sample = Math.max(
                1,
                Math.max(
                    divideCeil(header.width, MAX_TEXTURE_DIMENSION),
                    divideCeil(header.height, MAX_TEXTURE_DIMENSION)
                )
            );
            int outWidth = divideCeil(header.width, sample);
            int outHeight = divideCeil(header.height, sample);

            Bitmap bitmap;
            try {
                bitmap = Bitmap.createBitmap(outWidth, outHeight, Bitmap.Config.ARGB_8888);
            } catch (RuntimeException error) {
                throw new IOException("Sem memória para raster KAP " + outWidth + "x" + outHeight, error);
            }

            byte[] row = new byte[header.width];
            int[] argb = new int[outWidth];
            int outY = 0;

            for (int sourceY = 0; sourceY < header.height; sourceY += sample) {
                decodeRow(raf, rowOffsets[sourceY], header.bits, row);
                for (int outX = 0, sourceX = 0; outX < outWidth; outX++, sourceX += sample) {
                    int paletteIndex = row[Math.min(sourceX, header.width - 1)] & 0xff;
                    argb[outX] = header.palette.getOrDefault(paletteIndex, 0xff000000);
                }
                bitmap.setPixels(argb, 0, outWidth, 0, outY, outWidth, 1);
                outY++;
            }

            LatLngQuad quad = buildQuad(header);
            double[] bounds = bounds(header.refs);
            return new Result(
                header.name,
                bitmap,
                quad,
                header.width,
                header.height,
                header.bits,
                header.scale,
                header.datum,
                header.projection,
                (bounds[0] + bounds[2]) / 2.0,
                (bounds[1] + bounds[3]) / 2.0,
                bounds[0],
                bounds[1],
                bounds[2],
                bounds[3]
            );
        }
    }

    private static Header readHeader(RandomAccessFile raf) throws IOException {
        Header header = new Header();
        byte[] buffer = new byte[MAX_HEADER_BYTES];
        int length = 0;
        int previous = -1;

        raf.seek(0);
        while (length < buffer.length) {
            int value = raf.read();
            if (value < 0) throw new IOException("Cabeçalho KAP sem terminador");
            if (previous == 0x1a && value == 0x00) {
                length--;
                break;
            }
            buffer[length++] = (byte) value;
            previous = value;
        }
        if (length >= buffer.length) throw new IOException("Cabeçalho KAP excede limite seguro");

        String text = new String(buffer, 0, length, StandardCharsets.ISO_8859_1);
        for (String rawLine : text.split("\\r?\\n")) {
            String line = rawLine.trim();
            if (line.startsWith("BSB/")) parseBsb(line.substring(4), header);
            else if (line.startsWith("KNP/")) parseKnp(line.substring(4), header);
            else if (line.startsWith("IFM/")) header.bits = parseInt(line.substring(4), "IFM");
            else if (line.startsWith("RGB/")) parsePalette(line.substring(4), header);
            else if (line.startsWith("DAY/") && header.palette.isEmpty()) parsePalette(line.substring(4), header);
            else if (line.startsWith("REF/")) parseReference(line.substring(4), header);
        }

        // Some producers place the bit depth immediately after CTRL-Z/NUL.
        if (header.bits == 0 && raf.getFilePointer() < raf.length()) {
            int candidate = raf.readUnsignedByte();
            if (candidate >= 1 && candidate <= 7) header.bits = candidate;
        }
        return header;
    }

    private static void parseBsb(String body, Header header) {
        String[] fields = body.split(",");
        for (int i = 0; i < fields.length; i++) {
            String field = fields[i].trim();
            if (field.startsWith("NA=")) {
                header.name = field.substring(3).trim();
            } else if (field.startsWith("RA=")) {
                try {
                    header.width = Integer.parseInt(field.substring(3).trim());
                    if (i + 1 < fields.length) header.height = Integer.parseInt(fields[i + 1].trim());
                } catch (NumberFormatException ignored) {
                }
            }
        }
    }

    private static void parseKnp(String body, Header header) {
        for (String raw : body.split(",")) {
            String field = raw.trim();
            if (field.startsWith("SC=")) {
                try { header.scale = Integer.parseInt(field.substring(3).trim()); }
                catch (NumberFormatException ignored) {}
            } else if (field.startsWith("GD=")) {
                header.datum = field.substring(3).trim();
            } else if (field.startsWith("PR=")) {
                header.projection = field.substring(3).trim();
            }
        }
    }

    private static void parsePalette(String body, Header header) {
        String[] p = body.split(",");
        if (p.length < 4) return;
        try {
            int index = Integer.parseInt(p[0].trim());
            int red = clampByte(Integer.parseInt(p[1].trim()));
            int green = clampByte(Integer.parseInt(p[2].trim()));
            int blue = clampByte(Integer.parseInt(p[3].trim()));
            header.palette.put(index, 0xff000000 | red << 16 | green << 8 | blue);
        } catch (NumberFormatException ignored) {
        }
    }

    private static void parseReference(String body, Header header) {
        String[] p = body.split(",");
        if (p.length < 5) return;
        try {
            header.refs.add(new Reference(
                Double.parseDouble(p[1].trim()),
                Double.parseDouble(p[2].trim()),
                Double.parseDouble(p[3].trim()),
                Double.parseDouble(p[4].trim())
            ));
        } catch (NumberFormatException ignored) {
        }
    }

    private static void validateHeader(Header h) throws IOException {
        if (h.width <= 0 || h.height <= 0) throw new IOException("Dimensões RA ausentes ou inválidas");
        if ((long) h.width * h.height > MAX_PIXELS) throw new IOException("Carta KAP excede limite de pixels");
        if (h.bits < 1 || h.bits > 7) throw new IOException("IFM inválido: " + h.bits);
        if (h.palette.isEmpty()) throw new IOException("Paleta RGB/DAY ausente");
        if (h.refs.size() < 3) throw new IOException("Carta sem referências geográficas suficientes");
    }

    private static long[] readRowOffsets(RandomAccessFile raf, int height) throws IOException {
        long fileLength = raf.length();
        if (fileLength < 4) throw new IOException("Índice KAP ausente");

        raf.seek(fileLength - 4);
        long indexStart = readU32(raf);
        long indexBytes = (long) height * 4L;

        if (indexStart <= 0 || indexStart + indexBytes > fileLength - 4) {
            throw new IOException("Tabela de índice KAP inválida");
        }

        long[] offsets = new long[height];
        raf.seek(indexStart);
        long previous = -1;
        for (int i = 0; i < height; i++) {
            long offset = readU32(raf);
            if (offset <= 0 || offset >= indexStart || (previous >= 0 && offset < previous)) {
                throw new IOException("Offset de linha KAP inválido");
            }
            offsets[i] = offset;
            previous = offset;
        }
        return offsets;
    }

    private static void decodeRow(RandomAccessFile raf, long offset, int bits, byte[] output)
        throws IOException {
        raf.seek(offset);
        readVariableNumber(raf, 0, 0x7f); // encoded line number

        int shift = 7 - bits;
        int countMask = (1 << shift) - 1;
        int x = 0;

        while (x < output.length) {
            NumberRun run = readVariableNumber(raf, shift, countMask);
            if (run.count <= 0) throw new IOException("Run KAP inválido");
            int end = Math.min(output.length, x + run.count);
            while (x < end) output[x++] = (byte) run.pixel;
        }

        int terminator = raf.read();
        if (terminator < 0) throw new IOException("Linha KAP truncada");
    }

    private static final class NumberRun {
        final int pixel;
        final int count;

        NumberRun(int pixel, int count) {
            this.pixel = pixel;
            this.count = count;
        }
    }

    private static NumberRun readVariableNumber(RandomAccessFile raf, int shift, int mask)
        throws IOException {
        int c = raf.read();
        if (c < 0) throw new IOException("Raster KAP truncado");

        long value = c & 0x7f;
        int pixel = (int) (value >> shift);
        long count = value & mask;

        int continuation = 0;
        while ((c & 0x80) != 0) {
            if (++continuation > 5) throw new IOException("Run KAP excede limite");
            c = raf.read();
            if (c < 0) throw new IOException("Raster KAP truncado");
            count = (count << 7) + (c & 0x7f);
            if (count > Integer.MAX_VALUE - 1L) throw new IOException("Run KAP excessivo");
        }
        return new NumberRun(pixel, (int) count + 1);
    }

    private static LatLngQuad buildQuad(Header h) throws IOException {
        Reference nw = nearest(h.refs, 0.0, 0.0);
        Reference ne = nearest(h.refs, h.width - 1.0, 0.0);
        Reference se = nearest(h.refs, h.width - 1.0, h.height - 1.0);
        Reference sw = nearest(h.refs, 0.0, h.height - 1.0);

        if (nw == null || ne == null || se == null || sw == null) {
            throw new IOException("Não foi possível calcular os cantos geográficos");
        }

        return new LatLngQuad(
            new LatLng(nw.lat, nw.lon),
            new LatLng(ne.lat, ne.lon),
            new LatLng(se.lat, se.lon),
            new LatLng(sw.lat, sw.lon)
        );
    }

    private static double[] bounds(List<Reference> refs) {
        double minLat = Double.POSITIVE_INFINITY;
        double maxLat = Double.NEGATIVE_INFINITY;
        double minLon = Double.POSITIVE_INFINITY;
        double maxLon = Double.NEGATIVE_INFINITY;
        for (Reference ref : refs) {
            minLat = Math.min(minLat, ref.lat);
            maxLat = Math.max(maxLat, ref.lat);
            minLon = Math.min(minLon, ref.lon);
            maxLon = Math.max(maxLon, ref.lon);
        }
        return new double[]{minLat, minLon, maxLat, maxLon};
    }

    private static Reference nearest(List<Reference> refs, double x, double y) {
        Reference best = null;
        double bestDistance = Double.POSITIVE_INFINITY;
        for (Reference ref : refs) {
            double dx = ref.x - x;
            double dy = ref.y - y;
            double distance = dx * dx + dy * dy;
            if (distance < bestDistance) {
                bestDistance = distance;
                best = ref;
            }
        }
        return best;
    }

    private static long readU32(RandomAccessFile raf) throws IOException {
        return ((long) raf.readUnsignedByte() << 24)
            | ((long) raf.readUnsignedByte() << 16)
            | ((long) raf.readUnsignedByte() << 8)
            | raf.readUnsignedByte();
    }

    private static int parseInt(String value, String field) throws IOException {
        try {
            return Integer.parseInt(value.trim());
        } catch (NumberFormatException error) {
            throw new IOException(field + " inválido", error);
        }
    }

    private static int clampByte(int value) {
        return Math.max(0, Math.min(255, value));
    }

    private static int divideCeil(int value, int divisor) {
        return (value + divisor - 1) / divisor;
    }
}
