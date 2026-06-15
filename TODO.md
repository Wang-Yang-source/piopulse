# ☕ PioPulse: Planned Upgrades & TODO List

This document tracks the engineering tasks required to implement the next-generation factory flashing features and the VOFA+ enhanced telemetry plotter inside `PioPulse`.

---

## 🟩 Phase 1: Dynamic NVS & SN Provisioning (动态写号)
- [x] **Create NVS Schema Compiler**:
  - [x] Implement a helper function in Rust to construct a standard Espressif NVS key-value structure in memory.
  - [x] Support data types: `u8`, `u32`, and UTF-8 strings.
- [x] **Automate SN Generation**:
  - [x] Extract the detected MAC address inside the flasher thread.
  - [x] Generate SN based on a customizable pattern (e.g., `PP-{CHIP}-{YYMM}-BATCH-{MAC_SHORT}`).
- [x] **Flash Injection**:
  - [x] Compile the NVS struct to an `nvs.bin` byte array on-the-fly.
  - [x] Automatically write the compiled `nvs.bin` to the configured NVS offset (default: `0x9000`) after flashing partitions.

---

## 🟩 Phase 2: Post-Flash Automated Serial QA Self-Test (产测自测)
- [ ] **Serial Handshake Protocol**:
  - [ ] Define a simple JSON communication frame (e.g., `{"cmd": "ping_qa"}`).
  - [ ] Set up the flasher task to switch to serial monitor mode immediately after resetting the board.
- [ ] **Firmware Handshake**:
  - [ ] Send the test query command and wait for a structured JSON response from the board.
  - [ ] Add a timeout (e.g., 5 seconds) to prevent hanging if the board is bricked.
- [ ] **QA UI Card Mapping**:
  - [ ] Extend `Channel` struct in `src/app.rs` with a `qa_result` field.
  - [ ] Render the channel monitor cards with a distinct "QA Passed" (Bright Green) or "QA Failed" (Red with details) status.

---

## 🟩 Phase 3: Local SQLite / CSV Auditing (本地量产审计)
- [ ] **SQLite Integration**:
  - [ ] Add `rusqlite` to `Cargo.toml`.
  - [ ] Initialize a database schema `production_history.db` on app startup.
- [ ] **Automatic Log Entry**:
  - [ ] Save timestamps, MAC, generated SN, flashing speed, and QA self-test results upon completion of every channel task.
- [ ] **Admin Panel Exporter**:
  - [ ] Add a button in the Configuration/Admin tab to export the SQLite history table to a standard CSV report.

---

## 🟦 Phase 4: VOFA+ Protocol Engines & Serial Connection Control (第二界面协议与连接)
Enhance the serial monitoring infrastructure to dynamically support VOFA+ communication protocols and serial parameters on the **Plotter Tab (第二个界面)**.

- [ ] **Make Page 2 a Practical Serial Debugger First (第二界面优先做成好用的串口调试器)**:
  - [ ] Redesign the Plotter tab into a three-zone serial workbench:
    - left: port list and connection profile selector;
    - center: receive console / waveform / parsed table switcher;
    - bottom: command input, send options, and quick-send buttons.
  - [ ] Add explicit Connect/Disconnect controls instead of relying only on flash flow or background monitor state.
  - [ ] Support common serial profiles: baudrate, data bits, stop bits, parity, flow control, DTR, RTS, and line ending (`None`, `\n`, `\r\n`, `\r`).
  - [ ] Provide a receive console with timestamp toggle, autoscroll toggle, pause/resume, clear, copy/export, UTF-8/ASCII/HEX views, and RX byte counter.
  - [ ] Provide a send panel with manual input, send-as-text/send-as-hex toggle, command history, quick-send slots, repeated send timer, and TX byte counter.
  - [ ] Add filter/search/highlight support for incoming logs, including error keyword highlighting and regex/plain-text modes.
  - [ ] Persist per-port debugger settings in `ProjectConfig` so repeated debugging sessions reopen with the same profile.
  - [ ] Make error states actionable: port busy, permission denied, device removed, decode failure, and buffer overflow should show clear recovery hints.

- [ ] **Dynamic Connection Configuration**:
  - [ ] Add fields to `ProjectConfig` and the UI Config tab for telemetry serial settings:
    - `baudrate` (115200, 921600, 1000000, etc.)
    - `data_bits` (8)
    - `stop_bits` (1, 2)
    - `parity` (None, Even, Odd)
    - `flow_control` (None, Hardware)
  - [ ] Implement UI indicators in the Plotter tab for active connection state (Port name, current Baudrate, Buffer overflow status).
- [ ] **RawData Mode Integration**:
  - [ ] Add `VofaMode::RawData` to the telemetry configuration.
  - [ ] Design a TUI scrolling console view that displays raw bytes or UTF-8 characters received.
  - [ ] Implement Pause/Resume scrolling, Hex/ASCII view toggle, and Clear buffer buttons.
- [ ] **FireWater Protocol Engine**:
  - [ ] Upgrade `vofa::VofaParser` to handle robust FireWater CSV string parsing with comma separation and newlines `\n` or `\r\n`.
  - [ ] Support dynamic multi-channel float extraction (e.g., `printf("%f,%f,%f\n", target, current, output)`).
  - [ ] Handle invalid frames gracefully without resetting or panicking the parser.
- [ ] **JustFloat Protocol Engine**:
  - [ ] Implement robust binary float decoding: parse arrays of little-endian `f32` numbers ending with the tail byte-sequence `[0x00, 0x00, 0x80, 0x7F]` (representing a float `NaN`).
  - [ ] Optimize the `parser.feed()` loop for high-frequency binary data, minimizing allocation overhead during memory chunking.

---

## 🟦 Phase 5: Interactive TUI Controls & Dashboards (第二界面交互与控件)
Implement an interactive control and display dashboard inside the TUI Plotter layout, allowing users to send parameters/commands and visualize statuses.

```
+-------------------------------------------------------------+
| Plotter Tab: VOFA+ Telemetry Dashboard                      |
+------------------------------+------------------------------+
| [ Telemetry Plotter / FFT ]  | [ Control & Widget Panel ]   |
|                              |                              |
| (Real-time curves or FFT)    |  - LED Status: [ OK ] [ERR]  |
|                              |  - Target Speed: <===|===>   |
|                              |  - Command Input: [        ] |
|                              |  - Buttons: [START] [STOP]   |
+------------------------------+------------------------------+
```

- [ ] **TUI Widget Rendering Engine**:
  - [ ] **Status Lights (指示灯)**: Render colored indicators (e.g., green block `[■]` for OK, red block `[■]` for Fault) representing board-reported status bits.
  - [ ] **Interactive Sliders (滑动条)**: Add vertical or horizontal sliders for parameter tuning (e.g., adjusting PID `Kp` or `target_speed`), adjusting values via arrow keys/mouse dragging.
  - [ ] **Command Buttons (按钮)**: Add clickable/selectable action buttons (e.g., "Start Motor", "Emergency Stop") to trigger predefined serial packet writes.
  - [ ] **Dials & Gauges (仪表盘)**: Render ascii-art semi-circular dials or bar gauges showing current metrics (e.g., voltage, temp, motor speed) relative to threshold limits.
- [ ] **Command Sender Terminal (指令发送)**:
  - [ ] Implement an active command text input line in the Plotter tab to send manual commands down to the micro-controller (e.g., `SET_KP 2.5\n`).
  - [ ] Add command history tracking with Up/Down arrow selection.
  - [ ] Implement variable rate throttling to avoid flooding the MCU's receive buffers.
  - [ ] Add configurable quick-send buttons for frequently used commands such as reset, start, stop, read version, enter bootloader, and factory-test trigger.
  - [ ] Support command templates with simple variables, e.g. `{sn}`, `{mac}`, `{timestamp}`, and selected channel values.

---

## 🟦 Phase 6: Advanced Telemetry Analysis & Session Logger (第二界面高级分析与日志)
Integrate analytical tools and logging capability directly into the TUI to aid debugging without external desktop software.

- [ ] **FFT Spectrum Analyzer (频谱分析)**:
  - [ ] Integrate a simple FFT module (e.g., a lightweight FFT function or standard crate) to compute frequencies from the last 128/256 samples of a selected channel.
  - [ ] Add an "FFT Mode" toggle to the plotter tab to replace the time-domain curve with a dynamic frequency magnitude bar-graph (using Ratatui's `BarChart`).
  - [ ] Display peak frequency and harmonic indicators to identify mechanical resonance or sensor noise.
- [ ] **Data Distribution Histogram (直方图)**:
  - [ ] Accumulate data points and compute bin frequencies to show statistical distributions.
  - [ ] Draw a histogram overlay or panel to analyze noise characteristics, offset offsets, and signal variance.
- [ ] **Session Logger & CSV Exporter**:
  - [ ] Add a "Record Session" button to log parsed data streams directly to local CSV files (e.g., `logs/session_YYYYMMDD_HHMMSS.csv`) in background tasks.
  - [ ] Format fields: `Timestamp, Channel_0, Channel_1, ..., Channel_N`.
- [ ] **Offline Telemetry Playback**:
  - [ ] Implement a playback reader that reads recorded CSV files.
  - [ ] Provide play, pause, fast-forward, and rewind controls in the TUI to replay telemetry waveforms.

---

## 🟨 Cross-Cutting Improvements Worth Doing (值得改进的通用能力)
- [ ] **Input & Shortcut Consistency**:
  - [ ] Add a visible shortcut/help overlay (`?`) for every tab.
  - [ ] Keep keyboard and mouse behavior consistent across Channels, Plotter, Configuration, and Widgets.
  - [ ] Add focus indicators so users can tell whether arrows/typing affect the port list, receive console, send box, or config table.
  - [ ] Keep the Settings/Configuration page as the final tab forever; new feature pages must be inserted before it.
- [ ] **Configuration Safety**:
  - [ ] Validate firmware paths, offsets, baudrates, and serial settings before starting a flash or monitor session.
  - [ ] Add config import/export and reset-to-default actions.
  - [ ] Mask or protect sensitive admin settings while in operator mode.
- [ ] **Reliability & Backpressure**:
  - [ ] Add bounded buffers for serial receive, parser output, UI history, and logs to prevent memory growth during long sessions.
  - [ ] Display dropped-frame/dropped-byte counters when the UI cannot keep up.
  - [ ] Separate flashing serial access from debugger serial access to avoid competing opens on the same port.
- [ ] **Testing & Simulation**:
  - [ ] Add parser unit tests for RawData, FireWater, JustFloat, malformed frames, partial frames, and high-rate streams.
  - [ ] Extend simulation mode to generate realistic serial logs, binary VOFA frames, disconnect/reconnect events, and corrupted packets.
  - [ ] Add snapshot-style tests for important TUI layouts at small and wide terminal sizes.
- [ ] **Production Workflow Polish**:
  - [ ] Add batch summary export with pass/fail, MAC, SN, firmware version, QA result, and operator/admin session metadata.
  - [ ] Add per-device notes or failure reason tagging for rework tracking.
  - [ ] Add optional sound/visual completion cues for production-line operators.
- [ ] **Documentation**:
  - [ ] Update `README.md` shortcuts after the Page 2 debugger workflow is implemented.
  - [ ] Add a short hardware troubleshooting guide for Linux serial permissions, busy ports, boot mode, and cable quality.
