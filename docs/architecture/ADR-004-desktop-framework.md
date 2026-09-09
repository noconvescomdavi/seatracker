# ADR-004 — Desktop Framework

Status: Proposed/Preferred (Phase 0)

## Decision
Prefer Tauri for the Windows desktop shell, subject to validation of filesystem throughput, serial/COM access, TCP/UDP, native dialogs, fullscreen behavior and GPU integration.

The Rust SeaTracker Core remains independent of Tauri. If Tauri becomes a blocker, the shell may change without rewriting chart/navigation logic.

## Validation gate
Before Phase 1 desktop integration, prove: large-directory scanning, native chart file access, serial device enumeration/read, UDP/TCP ingest, renderer surface integration and persistence recovery.
