package com.noconves.seatracker;

import org.tukaani.xz.XZInputStream;

import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.FileInputStream;
import java.io.InputStream;
import java.util.Locale;

final class Cm93FileBytes {
    private static final int MAX_UNCOMPRESSED_BYTES = 256 * 1024 * 1024;

    private Cm93FileBytes() {}

    static byte[] read(File file) throws Exception {
        boolean compressed = file.getName().toLowerCase(Locale.ROOT).endsWith(".xz");
        try (InputStream raw = new FileInputStream(file);
             InputStream in = compressed ? new XZInputStream(raw, 32 * 1024 * 1024) : raw;
             ByteArrayOutputStream out = new ByteArrayOutputStream()) {
            byte[] buffer = new byte[128 * 1024];
            int read;
            int total = 0;
            while ((read = in.read(buffer)) != -1) {
                if (read == 0) continue;
                if (total > MAX_UNCOMPRESSED_BYTES - read) {
                    throw new IllegalStateException("CM93 descompactado excede 256 MB");
                }
                out.write(buffer, 0, read);
                total += read;
            }
            return out.toByteArray();
        }
    }
}
