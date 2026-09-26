# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

Rayhunter (EFF) detects IMSI catchers / cell-site simulators. A Rust daemon runs on cheap Qualcomm-based mobile hotspots (Orbic RC400L, TP-Link M7350/M7310/M7200, T-Mobile TMOHS1, Wingtech CT2MHS01, UZ801, Moxee, PinePhone), reads baseband traffic from `/dev/diag`, runs heuristic analyzers over it, and serves a web UI. User/dev docs are an mdBook in `doc/` (published at efforg.github.io/rayhunter).

## Commands

The Rust toolchain is pinned in `rust-toolchain.toml`. CI builds with `RUSTFLAGS=-Dwarnings`, so warnings fail.

```sh
# The daemon embeds the built frontend via include_bytes!, so build the web UI
# before `cargo check`/`test` on the daemon (from repo root, npm workspaces):
npm install && npm run build -w daemon/web

cargo fmt --all --check
cargo check
cargo test                               # default workspace members (excludes installer-gui)
cargo test -p rayhunter <test_name>      # single test in lib; crates: rayhunter, rayhunter-daemon, installer, rayhunter-check, telcom-parser, rootshell
cargo clippy

# installer-gui (Tauri) is not a default member; check it explicitly
cargo check -p installer-gui && cargo clippy -p installer-gui

# Frontend (daemon/web, SvelteKit + Tailwind)
npm run lint -w daemon/web     # prettier --check + eslint
npm run check -w daemon/web    # svelte-check
npm run test -w daemon/web     # vitest; single file: npm run test -w daemon/web -- src/path/file.test.ts
cd daemon/web && API_TARGET=http://192.168.1.1:8080 npm run dev   # hot-reload UI against a device (default localhost:8080)

# Docs
mdbook test && mdbook build
```

Device builds (target `armv7-unknown-linux-musleabihf`; aliases in `.cargo/config.toml`):

```sh
./scripts/build-dev.sh                     # frontend + daemon + rootshell (firmware-devel profile), optional wifi tools
./scripts/install-dev.sh orbic             # = cargo run -p installer -- orbic ...; see `install-dev.sh util --help`
cargo build-daemon-firmware-devel          # just the daemon (rustcrypto TLS, pure Rust)
cargo build-daemon-firmware                # release profile + pq-tls; needs arm-linux-musleabihf C cross-compiler
```

Run the daemon on a PC without hardware: set `debug_mode = true` in a config (skips DIAG, display, keys, battery, WiFi) — see `doc/installing-from-source.md`, then `cargo run -p rayhunter-daemon --bin rayhunter-daemon -- ./config.toml`.

OpenAPI spec: `cargo run --bin gen_api --features apidocs -- out.json` (API types carry `#[cfg_attr(feature = "apidocs", derive(utoipa::ToSchema))]`; keep that on new API-facing types).

## Architecture

Cargo workspace crates:

- **`lib/` (crate `rayhunter`)** — hardware-independent core, also used off-device.
  - `diag_device.rs`, `diag/`, `hdlc.rs`: talk to Qualcomm DIAG, framing, and parse diag log messages (deku-based binary parsing).
  - `qmdl.rs`: QMDL is the on-disk raw capture format (raw DIAG messages); `pcap.rs` / `gsmtap/` convert them to GSMTAP pcapng for Wireshark.
  - `analysis/`: heuristics. `analyzer.rs` defines the `Analyzer` trait (`metadata()`, `analyze_information_element()`, `report_skipped_packet()`), the `Harness`, and the `all_analyzers()` registry. `information_element.rs` decodes GSMTAP payloads into LTE RRC (via `telcom-parser`) and NAS (via `pycrate-rs`). Each heuristic is one file; its `AnalyzerMetadata.key` is a stable config key and `version` must be bumped on substantive behavior changes. Heuristics are documented for users in `doc/heuristics.md`.
  - `Device` enum in `lib.rs` lists supported hardware.
- **`telcom-parser/`** — LTE RRC decoder generated from the ASN.1 specs in `telcom-parser/specs/` (see its README before regenerating `lte_rrc.rs`). `tools/asn1grep.py` helps find types in those specs.
- **`daemon/` (`rayhunter-daemon`)** — the on-device binary. `main.rs` wires an axum HTTP API (`server.rs`, `/api/*`) plus long-lived tokio tasks coordinated with `TaskTracker`/`CancellationToken` and mpsc control channels: diag read thread (`diag.rs`, writes QMDL via `qmdl_store.rs` and runs analyzers live), analysis thread for re-analyzing stored recordings (`analysis.rs`), display (`display/`, per-device backends), key input, battery (`battery/`, per-device), notifications, GPS, WebDAV upload, and update checks. Per-device behavior is selected at runtime from the `Device` in config. The web UI (`daemon/web`, SvelteKit static build, gzipped) is compiled into the binary.
- **`installer/`** — host-side CLI that roots/installs onto each device (one module per device family in `installer/src/`; ADB over nusb/libusb, HTTP/telnet exploits, serial). `installer/build.rs` embeds the prebuilt ARM `rayhunter-daemon` and `rootshell` from `target/armv7-unknown-linux-musleabihf/<profile>/` (and optional wpa_supplicant/iw from `tools/build-wpa-supplicant/out`), overridable via `FILE_*` env vars — build firmware binaries before building the installer for a real install. Must work on Linux, macOS, and Windows.
- **`installer-gui/`** — experimental Tauri + Svelte wrapper around the installer; not a default workspace member.
- **`check/` (`rayhunter-check`)** — offline CLI running the analyzers over QMDL/pcap files or directories (`-p path`, `-P` to also pcapify).
- **`rootshell/`** — tiny setuid helper installed on devices to get root.

## Conventions

- Run `cargo fmt` and `cargo clippy` before submitting. Keep PRs to roughly ≤400 lines of non-test code; larger features should be discussed with maintainers first (`CONTRIBUTING.md`).
- AI-generated contributions must be disclosed and thoroughly understood/tested by the submitter; PR descriptions and comments should be written by the human contributor.
- Frontend: the `??=` operator is banned by eslint — use an explicit `=== undefined` check.
- Version bumps across crates/files: `scripts/set-versions.sh VERSION`.

## Hermes / Claude coordination

This repository is **Rayhunter / EFForg rayhunter**, not Velociraptor Claw Edition.
Use the project-local `/hermes-bridge` command and `rayhunter-hermes-bridge` skill
for Hermes coordination. The explicit Hermes Kanban board is `rayhunter-orbic`;
always run board commands with `HERMES_KANBAN_BOARD=rayhunter-orbic`.

- Preserve existing dirty and untracked paths; do not reset, clean, stash, overwrite,
  commit, push, or merge without explicit authorization.
- Read complete bridge directives from `~/.hermes/claude-bridge/inbox/`.
- For directives with an `ack_token`, append one exact ACK line to `channel.jsonl`
  and write the required outbox receipt; do not treat a heartbeat as completion proof.
- Keep Orbic-specific and shared/general Rayhunter work distinct.
- Do not infer Orbic hardware from Qualcomm USB vendor `0x05c6` alone.
- Distinguish local tests, provider CI, and physical-hardware validation.
- If no valid Rayhunter directive exists, report live board/repository state and wait;
  do not invent work or select a follow-on card.
