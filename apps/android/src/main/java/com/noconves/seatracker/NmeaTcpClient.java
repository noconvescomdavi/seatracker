package com.noconves.seatracker;

import java.io.BufferedReader;
import java.io.Closeable;
import java.io.InputStreamReader;
import java.net.InetSocketAddress;
import java.net.Socket;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.atomic.AtomicBoolean;

public final class NmeaTcpClient implements Closeable {
    public interface Listener {
        void onSentence(String sentence, long receivedAtMs);
        void onStatus(String message);
    }

    private final String host;
    private final int port;
    private final Listener listener;
    private final AtomicBoolean running = new AtomicBoolean(false);
    private Thread thread;
    private Socket socket;

    public NmeaTcpClient(String host, int port, Listener listener) {
        this.host = host;
        this.port = port;
        this.listener = listener;
    }

    public synchronized void start() {
        if (running.get()) return;
        running.set(true);
        thread = new Thread(this::loop, "SeaTracker-NMEA-TCP");
        thread.start();
    }

    private void loop() {
        long retryDelayMs = 1_000L;
        while (running.get()) {
            try (Socket client = new Socket()) {
                socket = client;
                client.connect(new InetSocketAddress(host, port), 5_000);
                client.setKeepAlive(true);
                client.setSoTimeout(15_000);
                listener.onStatus("NMEA TCP conectado a " + host + ":" + port);
                retryDelayMs = 1_000L;

                try (BufferedReader reader = new BufferedReader(
                    new InputStreamReader(client.getInputStream(), StandardCharsets.US_ASCII)
                )) {
                    String line;
                    while (running.get() && (line = reader.readLine()) != null) {
                        String sentence = line.trim();
                        if (!sentence.isEmpty()) {
                            listener.onSentence(sentence, System.currentTimeMillis());
                        }
                    }
                }
            } catch (Exception error) {
                if (running.get()) {
                    listener.onStatus("NMEA TCP reconectando: " + error.getMessage());
                    try {
                        Thread.sleep(retryDelayMs);
                    } catch (InterruptedException interrupted) {
                        Thread.currentThread().interrupt();
                        break;
                    }
                    retryDelayMs = Math.min(30_000L, retryDelayMs * 2L);
                }
            } finally {
                socket = null;
            }
        }
        running.set(false);
    }

    @Override
    public synchronized void close() {
        running.set(false);
        if (socket != null) {
            try {
                socket.close();
            } catch (Exception ignored) {
            }
        }
        if (thread != null) thread.interrupt();
        thread = null;
        socket = null;
    }
}
