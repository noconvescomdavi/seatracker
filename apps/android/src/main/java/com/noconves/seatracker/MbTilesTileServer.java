package com.noconves.seatracker;

import android.database.Cursor;
import android.database.sqlite.SQLiteDatabase;

import java.io.BufferedInputStream;
import java.io.BufferedOutputStream;
import java.io.File;
import java.io.IOException;
import java.net.InetAddress;
import java.net.ServerSocket;
import java.net.Socket;
import java.nio.charset.StandardCharsets;
import java.util.Locale;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

public final class MbTilesTileServer {
    private final File dbFile;
    private final ExecutorService pool = Executors.newFixedThreadPool(2);
    private ServerSocket serverSocket;
    private Thread acceptThread;
    private SQLiteDatabase db;

    public MbTilesTileServer(File dbFile) {
        this.dbFile = dbFile;
    }

    public synchronized int start() throws IOException {
        if (serverSocket != null) {
            return serverSocket.getLocalPort();
        }
        db = SQLiteDatabase.openDatabase(dbFile.getAbsolutePath(), null, SQLiteDatabase.OPEN_READONLY);
        serverSocket = new ServerSocket(0, 50, InetAddress.getByName("127.0.0.1"));
        acceptThread = new Thread(this::acceptLoop, "SeaTracker-MBTiles");
        acceptThread.start();
        return serverSocket.getLocalPort();
    }

    private void acceptLoop() {
        while (serverSocket != null && !serverSocket.isClosed()) {
            try {
                Socket socket = serverSocket.accept();
                pool.execute(() -> handle(socket));
            } catch (IOException ignored) {
                break;
            }
        }
    }

    private void handle(Socket socket) {
        try (Socket s = socket;
             BufferedInputStream in = new BufferedInputStream(s.getInputStream());
             BufferedOutputStream out = new BufferedOutputStream(s.getOutputStream())) {
            String requestLine = readLine(in);
            if (requestLine == null) {
                return;
            }
            String[] parts = requestLine.split(" ");
            if (parts.length < 2) {
                writeStatus(out, 400, "Bad Request");
                return;
            }

            String path = parts[1];
            while (true) {
                String line = readLine(in);
                if (line == null || line.isEmpty()) break;
            }

            if (path.equals("/health")) {
                byte[] ok = "OK".getBytes(StandardCharsets.UTF_8);
                writeBytes(out, 200, "text/plain", ok);
                return;
            }

            if (!path.startsWith("/tiles/")) {
                writeStatus(out, 404, "Not Found");
                return;
            }

            String[] coords = path.substring("/tiles/".length()).split("/");
            if (coords.length != 3) {
                writeStatus(out, 400, "Bad Request");
                return;
            }

            int z = Integer.parseInt(coords[0]);
            int x = Integer.parseInt(coords[1]);
            String yRaw = coords[2];
            int dot = yRaw.indexOf('.');
            int yXyz = Integer.parseInt(dot >= 0 ? yRaw.substring(0, dot) : yRaw);

            if (z < 0 || z > 30 || x < 0 || yXyz < 0) {
                writeStatus(out, 400, "Bad Request");
                return;
            }

            long max = 1L << z;
            long yTmsLong = max - 1L - yXyz;
            if (yTmsLong < 0 || yTmsLong > Integer.MAX_VALUE) {
                writeStatus(out, 404, "Not Found");
                return;
            }

            byte[] tile = readTile(z, x, (int) yTmsLong);
            if (tile == null) {
                writeStatus(out, 404, "Not Found");
                return;
            }

            writeBytes(out, 200, detectMime(tile), tile);
        } catch (Exception ignored) {
        }
    }

    private synchronized byte[] readTile(int z, int x, int yTms) {
        if (db == null) return null;
        try (Cursor cursor = db.rawQuery(
            "SELECT tile_data FROM tiles WHERE zoom_level=? AND tile_column=? AND tile_row=? LIMIT 1",
            new String[]{String.valueOf(z), String.valueOf(x), String.valueOf(yTms)})) {
            if (!cursor.moveToFirst()) return null;
            return cursor.getBlob(0);
        }
    }

    private static String detectMime(byte[] data) {
        if (data.length >= 8
            && data[0] == (byte) 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47) {
            return "image/png";
        }
        if (data.length >= 3
            && data[0] == (byte) 0xFF && data[1] == (byte) 0xD8 && data[2] == (byte) 0xFF) {
            return "image/jpeg";
        }
        if (data.length >= 12
            && data[0] == 'R' && data[1] == 'I' && data[2] == 'F' && data[3] == 'F') {
            return "image/webp";
        }
        return "application/octet-stream";
    }

    private static String readLine(BufferedInputStream in) throws IOException {
        StringBuilder sb = new StringBuilder();
        int prev = -1;
        int current;
        while ((current = in.read()) != -1) {
            if (prev == '\r' && current == '\n') {
                sb.setLength(Math.max(0, sb.length() - 1));
                return sb.toString();
            }
            sb.append((char) current);
            prev = current;
            if (sb.length() > 8192) {
                throw new IOException("HTTP header line too long");
            }
        }
        return sb.length() == 0 ? null : sb.toString();
    }

    private static void writeStatus(BufferedOutputStream out, int code, String message) throws IOException {
        writeBytes(out, code, "text/plain", message.getBytes(StandardCharsets.UTF_8));
    }

    private static void writeBytes(BufferedOutputStream out, int code, String contentType, byte[] body) throws IOException {
        String header = String.format(Locale.US,
            "HTTP/1.1 %d OK\r\nContent-Type: %s\r\nContent-Length: %d\r\nConnection: close\r\nCache-Control: max-age=3600\r\n\r\n",
            code, contentType, body.length);
        out.write(header.getBytes(StandardCharsets.US_ASCII));
        out.write(body);
        out.flush();
    }

    public synchronized void stop() {
        try {
            if (serverSocket != null) serverSocket.close();
        } catch (IOException ignored) {
        }
        serverSocket = null;
        if (db != null) {
            db.close();
            db = null;
        }
        pool.shutdownNow();
    }
}
