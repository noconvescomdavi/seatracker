from pathlib import Path

p = Path('apps/android/src/main/java/com/noconves/seatracker/MainActivity.java')
s = p.read_text()
s = s.replace('import android.provider.OpenableColumns;\nimport android.widget.Button;', 'import android.provider.OpenableColumns;\nimport android.text.InputType;\nimport android.widget.Button;\nimport android.widget.EditText;\nimport android.widget.LinearLayout;')
s = s.replace('    private NmeaUdpReceiver nmeaUdpReceiver;\n', '    private NmeaUdpReceiver nmeaUdpReceiver;\n    private NmeaTcpClient nmeaTcpClient;\n')
s = s.replace('    private Float lastNmeaCog;\n', '    private Float lastNmeaCog;\n    private String lastNmeaSource = "NMEA UDP :10110";\n')
s = s.replace('        status.setOnClickListener(v -> showDashboard());\n', '        status.setOnClickListener(v -> showDashboard());\n        status.setOnLongClickListener(v -> {\n            showConnectionManager();\n            return true;\n        });\n')
s = s.replace('        startNmeaUdp();\n        gpsHandler.post(gpsFreshnessWatch);', '        startNmeaUdp();\n        startConfiguredNmeaTcp();\n        gpsHandler.post(gpsFreshnessWatch);')
s = s.replace(': hasFreshNmeaFix() ? "NMEA UDP :10110"', ': hasFreshNmeaFix() ? lastNmeaSource')
old = '''            @Override
            public void onSentence(String sentence, long receivedAtMs) {
                if (sentence.startsWith("$")) {
                    Nmea0183Parser.Update update = Nmea0183Parser.parse(sentence);
                    if (update != null) {
                        runOnUiThread(() -> applyNmeaUpdate(update));
                    }
                } else if (sentence.startsWith("!")) {
                    AisDecoder.Target target = aisDecoder.push(sentence, receivedAtMs);
                    if (target != null && target.hasPosition()) {
                        runOnUiThread(() -> applyAisTarget(target));
                    }
                }
            }
'''
new = '''            @Override
            public void onSentence(String sentence, long receivedAtMs) {
                handleMarineSentence(sentence, receivedAtMs, "NMEA UDP :10110");
            }
'''
if old not in s:
    raise SystemExit('UDP listener block not found')
s = s.replace(old, new)
anchor = '    private void applyAisTarget(AisDecoder.Target target) {'
methods = '''    private void handleMarineSentence(String sentence, long receivedAtMs, String source) {
        if (sentence.startsWith("$")) {
            Nmea0183Parser.Update update = Nmea0183Parser.parse(sentence);
            if (update != null) {
                runOnUiThread(() -> {
                    lastNmeaSource = source;
                    applyNmeaUpdate(update);
                });
            }
        } else if (sentence.startsWith("!")) {
            AisDecoder.Target target = aisDecoder.push(sentence, receivedAtMs);
            if (target != null && target.hasPosition()) {
                runOnUiThread(() -> applyAisTarget(target));
            }
        }
    }

    private void startConfiguredNmeaTcp() {
        if (nmeaTcpClient != null) {
            nmeaTcpClient.close();
            nmeaTcpClient = null;
        }
        if (!prefs.getBoolean("nmea_tcp_enabled", false)) return;
        String host = prefs.getString("nmea_tcp_host", "").trim();
        int port = prefs.getInt("nmea_tcp_port", 10110);
        if (host.isEmpty() || port < 1 || port > 65535) return;

        nmeaTcpClient = new NmeaTcpClient(host, port, new NmeaTcpClient.Listener() {
            @Override
            public void onSentence(String sentence, long receivedAtMs) {
                handleMarineSentence(sentence, receivedAtMs, "NMEA TCP " + host + ":" + port);
            }

            @Override
            public void onStatus(String message) {
                runOnUiThread(() -> cursorStatus.setText(message));
            }
        });
        nmeaTcpClient.start();
    }

    private void showConnectionManager() {
        LinearLayout form = new LinearLayout(this);
        form.setOrientation(LinearLayout.VERTICAL);
        int pad = (int) (16 * getResources().getDisplayMetrics().density);
        form.setPadding(pad, pad, pad, 0);

        EditText hostInput = new EditText(this);
        hostInput.setHint("Host/IP NMEA TCP");
        hostInput.setSingleLine(true);
        hostInput.setText(prefs.getString("nmea_tcp_host", ""));
        form.addView(hostInput);

        EditText portInput = new EditText(this);
        portInput.setHint("Porta (ex.: 10110)");
        portInput.setInputType(InputType.TYPE_CLASS_NUMBER);
        portInput.setSingleLine(true);
        portInput.setText(Integer.toString(prefs.getInt("nmea_tcp_port", 10110)));
        form.addView(portInput);

        new AlertDialog.Builder(this)
            .setTitle("Conexões de Navegação")
            .setMessage("UDP 10110 permanece ativo. Configure aqui uma fonte NMEA/AIS TCP adicional.")
            .setView(form)
            .setPositiveButton("Salvar e conectar", (dialog, which) -> {
                String host = hostInput.getText().toString().trim();
                int port;
                try {
                    port = Integer.parseInt(portInput.getText().toString().trim());
                } catch (NumberFormatException error) {
                    Toast.makeText(this, "Porta TCP inválida.", Toast.LENGTH_LONG).show();
                    return;
                }
                if (host.isEmpty() || port < 1 || port > 65535) {
                    Toast.makeText(this, "Host/porta TCP inválidos.", Toast.LENGTH_LONG).show();
                    return;
                }
                prefs.edit()
                    .putBoolean("nmea_tcp_enabled", true)
                    .putString("nmea_tcp_host", host)
                    .putInt("nmea_tcp_port", port)
                    .apply();
                startConfiguredNmeaTcp();
            })
            .setNeutralButton("Desativar TCP", (dialog, which) -> {
                prefs.edit().putBoolean("nmea_tcp_enabled", false).apply();
                if (nmeaTcpClient != null) {
                    nmeaTcpClient.close();
                    nmeaTcpClient = null;
                }
                cursorStatus.setText("NMEA TCP desativado • UDP 10110 ativo");
            })
            .setNegativeButton("Cancelar", null)
            .show();
    }

'''
if anchor not in s:
    raise SystemExit('AIS anchor not found')
s = s.replace(anchor, methods + anchor)
s = s.replace('        if (nmeaUdpReceiver != null) nmeaUdpReceiver.close();\n        mapView.onDestroy();', '        if (nmeaUdpReceiver != null) nmeaUdpReceiver.close();\n        if (nmeaTcpClient != null) nmeaTcpClient.close();\n        mapView.onDestroy();')
p.write_text(s)
