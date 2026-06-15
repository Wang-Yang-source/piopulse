# ☕ PioPulse

**PioPulse** is a terminal user interface (TUI) factory flashing tool designed for high-concurrency ESP32 chip flashing on production lines. 

Unlike general-purpose tools like PlatformIO or Arduino IDE, PioPulse is built specifically for factory environments. It prioritizes **fail-safe operation (anti-fooling / 防呆)**, **extreme parallel execution**, **comprehensive QA testing**, and **production line audit logs**.

## ✨ Core Features

1. **Simultaneous Multi-Device Flashing (Parallelism)**: Uses Rust async workers (`tokio`) to flash multiple connected ESP32 boards concurrently. If one device gets disconnected or hangs, other devices continue flashing unaffected.
2. **Auto-Discovery**: Continuously scans serial ports to auto-detect ESP32 connections. No manual port selection required for the operator.
3. **Simulation Mode**: Includes a fully-simulated factory flashing flow. Operators and developers can play with the interface, trigger mock writes, and witness error-handling rates without any actual hardware connected.
4. **Operator & Admin Mode Separations**:
   - **Operator Mode**: Interface is locked to prevent unauthorized changes. Operators can only press `Space` to start/stop the flashing process and `1/2/3` to view tabs.
   - **Admin Mode**: Password protected (default: `admin`). Administrators can live-edit offsets, flash speeds, flash modes, crystal frequencies, and target firmware paths.
5. **Real-Time Auditing & Statistics**: Displays total attempts, passes, failures, and yield rate. Logs are written to an on-screen scrolling panel for real-time diagnostics.

---

## 🛠️ Architecture

The project is structured into clean Rust modules:
- [`src/main.rs`](file:///home/waya/Projects/PioPulse/src/main.rs): Coordinates the asynchronous event loop, crossterm inputs, and state dispatcher.
- [`src/app.rs`](file:///home/waya/Projects/PioPulse/src/app.rs): Implements the core application states, stats counters, and port scanning.
- [`src/ui.rs`](file:///home/waya/Projects/PioPulse/src/ui.rs): Implements the layout widgets, grid rendering for channels, logs tab, config tab, and authorization modals using `ratatui`.
- [`src/worker.rs`](file:///home/waya/Projects/PioPulse/src/worker.rs): Manages background tasks. Wraps `/home/waya/.local/bin/esptool.py` to parse real flashing stdout logs, and contains the logic for Simulation Mode.
- [`src/config.rs`](file:///home/waya/Projects/PioPulse/src/config.rs): Handles JSON serialization/deserialization for project settings. Saves files to `project_config.json`.

---

## 🚀 How to Run

To build and run PioPulse:

```bash
# Navigate to the project directory
cd PioPulse

# Compile and run the application
cargo run
```

### Keyboard Shortcuts
- **`Space`**: Start batch flashing to all detected/simulated devices.
- **`Tab` or `F1`**: Toggle **Admin Mode** (Enter password: `admin`).
- **`s`**: Toggle **Simulation Mode** (Default: `ON`). Enables mock flashing for demonstration purposes.
- **`c`**: Clear production statistics.
- **`1`**: Switch to **Channels Tab** (Displays grid cards for each channel).
- **`2`**: Switch to **Logs Tab** (Displays scrolling log entries).
- **`3`**: Switch to **Configuration Tab** (Locks/unlocks setting alterations).
- **`Esc`**: Exit the application.
