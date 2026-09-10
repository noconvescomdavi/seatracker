package com.noconves.seatracker;

import java.io.Closeable;
import java.net.DatagramPacket;
import java.net.DatagramSocket;
import java.net.InetSocketAddress;
import java.nio.charset.StandardCharsets;
import java.util.concurrent.atomic.AtomicBoolean;

public final class NmeaUdpReceiver implements Closeable {
    public interface Listener {
        void onSentence(String sentence, long receivedAtMs);
        void onStatus(String message);
    }

    private final int port;
    private final Listener listener;
    private final AtomicBoolean running = new AtomicBoolean(false);
    private DatagramSocket socket;
    private Thread thread;

    public NmeaUdpReceiver(int port, Listener listener) {
        this.port = port;
        this.listener = listener;
    }

    public synchronized void start() {
        if (running.get()) return;
        running.set(true);
        thread = new Thread(this::loop, "SeaTracker-NMEA-UDP-" + port);
        thread.start();
    }

    private void loop() {
        try {
            socket = new DatagramSocket(null);
            socket.setReuseAddress(true);
            socket.bind(new InetSocketAddress(port));
            listener.onStatus("NMEA UDP ativo na porta " + port);

            byte[] buffer = new byte[8192];
            while (running.get()) {
                DatagramPacket packet = new DatagramPacket(buffer, buffer.length);
                socket.receive(packet);
                String payload = new String(
                    packet.getData(),
                    packet.getOffset(),
                    packet.getLength(),
                    StandardCharsets.US_ASCII
                );
                long now = System.currentTimeMillis();
                for (String line : payload.split("[\\r\\n]+")) {
                    String sentence = line.trim();
                    if (!sentence.isEmpty()) listener.onSentence(sentence, now);
                }
            }
        } catch (Exception error) {
            if (running.get()) listener.onStatus("NMEA UDP indisponível: " + error.getMessage());
        } finally {
            running.set(false);
            if (socket != null) socket.close();
            socket = null;
        }
    }

    @Override
    public synchronized void close() {
        running.set(false);
        if (socket != null) socket.close();
        socket = null;
        thread = null;
    }
}
