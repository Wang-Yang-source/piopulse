# PioPulse

[![Crates.io](https://img.shields.io/crates/v/piopulse.svg)](https://crates.io/crates/piopulse)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Docs.rs](https://docs.rs/piopulse/badge.svg)](https://docs.rs/piopulse)

English documentation. The default README is now Chinese: [README.md](README.md).

**PioPulse** is a terminal user interface (TUI) production flashing and serial workbench for ESP32-oriented factory workflows. It focuses on multi-device flashing, serial monitoring, VOFA+ telemetry visualization, production traceability, and operator/admin separation.

## Core Features

- Multi-device ESP32 flashing through async Rust workers.
- Automatic USB serial port discovery.
- Production flashing dashboard with per-channel status, progress, MAC, serial number, firmware version, QA result, security state, byte count, and trace ID.
- Configurable production policies: verify method, blank-check policy, erase mode, NVS offset, SN prefix, lot code, firmware version, MES endpoint, label template, QA script, secure boot, flash encryption, and lock-after-flash intent.
- Dynamic NVS provisioning for serial number and device name.
- Serial terminal with RX/TX logging, hex display/send modes, newline control, baud-rate switching, quick command templates, timeline recording, and replay parsing.
- VOFA+ plotter support for FireWater, JustFloat, and IndexFloat style streams.
- Operator/Admin mode separation to reduce accidental production changes.

## Preview

Screenshots are stored in [`sources/`](sources/) and show the main TUI workflows.

| Serial terminal | Serial monitoring |
| --- | --- |
| ![Serial terminal](sources/微信图片_20260618032546_6773_1.png) | ![Serial monitoring](sources/微信图片_20260618032701_6776_1.png) |

| Batch flashing | Telemetry plotter |
| --- | --- |
| ![Batch flashing](sources/微信图片_20260618032620_6774_1.png) | ![Telemetry plotter](sources/微信图片_20260618032634_6775_1.png) |

Demo video: [PioPulse TUI recording](sources/3367a56efd718d1d42ed957b38ccc8f8_raw.mp4)

## Run

```bash
cargo run
```

## Main Shortcuts

- `1`: Serial terminal
- `2`: Plotter
- `3`: Widgets/dashboard
- `4`: Flasher
- `5`: Configuration
- `Space`: Flash the selected device on the flasher tab, or start typing on the serial tab
- `B`: Batch flash all devices on the flasher tab
- `F1`: Unlock/lock admin mode
- `F2`: Toggle sidebar
- `Esc`: Exit or cancel the current modal/editing state

## Project Layout

- `src/main.rs`: terminal setup, event loop, keyboard/mouse dispatch.
- `src/app.rs`: application state, production counters, channel state, port scanning, UI event handling.
- `src/ui.rs`: top-level layout and tab routing.
- `src/ui/serial.rs`: serial terminal and quick command panel.
- `src/ui/channels.rs`: production flashing dashboard.
- `src/ui/config.rs`: editable project and production settings.
- `src/worker.rs`: background flashing and serial monitor workers.
- `src/nvs.rs`: ESP32 NVS page generation for provisioning data.

## Notes

PioPulse currently implements the ESP32 serial flashing path and models several production-line policies in the UI and trace flow. Some advanced factory features such as MES upload, label printer integration, secure boot eFuse locking, and full hardware QA scripts are represented as configurable production policies and should be connected to site-specific infrastructure before use in a real production line.
