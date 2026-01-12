# wireguard-keepalived

By Gemini 3.0 Pro.

```
Write a "wireguard-keepalived" program using Rust. It's as simple as:

1. Run "wg show" and parse result.
2. For each wg interface that has at least one peer which has "persistent keepalive" (in which case it means the interface is running as client and connecting to a server), if "latest handshake" is older than 180 seconds (wg session key valid time limit), run "wg set wg0 listen-port 0".
3. Sleep for 25 seconds and run again.
```

## Build

1. Install [cross](https://crates.io/crates/cross): `cargo install cross`. Note `cross` uses Docker.
2. Run `./build_amd64.sh`, `./build_arm64.sh` or `./build_mips.sh` to build Linux amd64 / arm64 /  mipsle (softfloat) binary.

## Run

Just put `wireguard-keepalived` binary to PATH and execute it. It has zero config.

## Run as service

Either install it as systemd service (See `wireguard-keepalived.service`), or use `start-stop-daemon` to start / stop it (See `start-wireguard-keepalived.sh` and `stop-wireguard-keepalived.sh`).
