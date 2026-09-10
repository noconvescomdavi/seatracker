package com.noconves.seatracker;

import java.io.BufferedInputStream;
import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.util.Locale;
import java.util.zip.ZipEntry;
import java.util.zip.ZipInputStream;

/** Secure ZIP extractor restricted to CM93 dataset files. */
public final class SafeZipExtractor {
    private static final int MAX_ENTRIES = 100_000;
    private static final long MAX_FILE_BYTES = 512L * 1024L * 1024L;
    private static final long MAX_TOTAL_BYTES = 4L * 1024L * 1024L * 1024L;

    public static final class Result {
        public final int files;
        public final int cells;
        public final int dictionaries;
        public final long bytes;

        Result(int files, int cells, int dictionaries, long bytes) {
            this.files = files;
            this.cells = cells;
            this.dictionaries = dictionaries;
            this.bytes = bytes;
        }
    }

    private SafeZipExtractor() {}

    public static Result extractCm93(File archive, File destination) throws IOException {
        if (archive == null || !archive.isFile()) throw new IOException("Arquivo ZIP inexistente");
        if (destination == null) throw new IOException("Destino CM93 inválido");
        if (!destination.exists() && !destination.mkdirs()) throw new IOException("Falha ao criar pasta CM93");

        String destinationCanonical = destination.getCanonicalPath() + File.separator;
        int entries = 0;
        int files = 0;
        int cells = 0;
        int dictionaries = 0;
        long total = 0L;
        byte[] buffer = new byte[128 * 1024];

        try (ZipInputStream zip = new ZipInputStream(new BufferedInputStream(new FileInputStream(archive)))) {
            ZipEntry entry;
            while ((entry = zip.getNextEntry()) != null) {
                if (++entries > MAX_ENTRIES) throw new IOException("ZIP CM93 possui entradas demais");
                if (entry.isDirectory()) {
                    zip.closeEntry();
                    continue;
                }

                String normalizedName = entry.getName().replace('\\', '/');
                String base = new File(normalizedName).getName();
                String lower = base.toLowerCase(Locale.ROOT);
                boolean dictionary = lower.equals("cm93obj.dic")
                    || lower.equals("attrlut.dic")
                    || lower.equals("cm93attr.dic");
                boolean cell = Cm93ChartDecoder.isCm93CellName(base);
                if (!dictionary && !cell) {
                    zip.closeEntry();
                    continue;
                }

                File output = new File(destination, normalizedName);
                String outputCanonical = output.getCanonicalPath();
                if (!outputCanonical.startsWith(destinationCanonical)) {
                    throw new IOException("Entrada ZIP insegura bloqueada");
                }
                File parent = output.getParentFile();
                if (parent != null && !parent.exists() && !parent.mkdirs()) {
                    throw new IOException("Falha ao criar diretório CM93");
                }

                long fileBytes = 0L;
                try (FileOutputStream out = new FileOutputStream(output, false)) {
                    int read;
                    while ((read = zip.read(buffer)) != -1) {
                        fileBytes += read;
                        total += read;
                        if (fileBytes > MAX_FILE_BYTES) throw new IOException("Arquivo CM93 excede 512 MB");
                        if (total > MAX_TOTAL_BYTES) throw new IOException("Dataset CM93 excede 4 GB descompactados");
                        out.write(buffer, 0, read);
                    }
                    out.flush();
                }

                files++;
                if (dictionary) dictionaries++;
                if (cell) cells++;
                zip.closeEntry();
            }
        }

        if (cells == 0) throw new IOException("ZIP não contém células CM93 reconhecidas");
        return new Result(files, cells, dictionaries, total);
    }
}
