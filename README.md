# RDRPI

A lightweight web‑based internet radio that runs an async HTTP server with Axum, renders the UI using Askama, spawns FFmpeg to play selected HLS streams, and controls system audio via WirePlumber/PipeWire. Built for headless servers, it lets you choose channels and adjust volume from any device on the local network for simple remote playback management. Channels are currently hard‑coded in the source code for convenience.

# Usage

You can set a custom ip-address/port using the --ip and --port arguments along with their shorter counterparts -i and -p. The stream file location can be changed via the -f and --file arguments.

```
$ cargo run --release
Running RDRPI @ http://0.0.0.0:8080
```

# Couldn't bind to port 80

Before launching the application, run setcap to allow it to bind to port 80. You'll need to do this each time after compiling the binary.

```
$ cargo run --release -- --port 80
Error: "Couldn't bind to port 80"
$ sudo setcap cap_net_bind_service=ep target/release/RDRPI
$ cargo run --release -- --port 80
Running RDRPI @ http://0.0.0.0:80
```
