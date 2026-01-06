# RDRPI

A lightweight web‑based internet radio that runs an async HTTP server with Axum, renders the UI using Askama, spawns FFmpeg to play selected HLS streams, and controls system audio via WirePlumber/PipeWire. Built for headless servers, it lets you choose channels and adjust volume from any device on the local network for simple remote playback management. Channels are currently hard‑coded in the source code for convenience.

# Usage

Before launching the application, run setcap to allow it to bind to port 80.

```
cargo build --release
sudo setcap cap_net_bind_service=ep target/release/RDRPI
./target/release/RDRPI &
xdg-open "http://127.0.0.1"
```
