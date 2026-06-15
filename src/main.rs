mod app;
mod config;
mod nvs;
mod ui;
mod vofa;
mod worker;

use app::{ActiveTab, App, PlotterMode, WidgetType};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, time::Duration};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let config_path = "project_config.json".to_string();
    let mut app = App::new(config_path);

    // Initial port scan
    app.scan_ports();

    // Create channel for worker messages
    let (tx, mut rx) = mpsc::channel(100);

    // Setup input events stream & ticks
    let mut reader = EventStream::new();
    let mut interval = tokio::time::interval(Duration::from_millis(100));

    let mut exit = false;

    while !exit {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        tokio::select! {
            // Background worker messages
            Some(msg) = rx.recv() => {
                app.handle_worker_message(msg);
            }
            // Standard ticks
            _ = interval.tick() => {
                app.tick();

                // Scan ports automatically every 2 seconds if not flashing
                if !app.is_flashing && app.last_port_scan.elapsed() > Duration::from_secs(2) {
                    app.scan_ports();
                    app.last_port_scan = std::time::Instant::now();
                }
            }
            // User keyboard/terminal events
            maybe_event = reader.next() => {
                if let Some(Ok(event)) = maybe_event {
                    match event {
                        Event::Key(key) => {
                            if key.kind == KeyEventKind::Press {
                                if app.show_exit_menu {
                                    match key.code {
                                        KeyCode::Esc => {
                                            app.show_exit_menu = false;
                                        }
                                        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                                            app.exit_menu_selected = if app.exit_menu_selected == 0 { 1 } else { 0 };
                                        }
                                        KeyCode::Enter => {
                                            match app.exit_menu_selected {
                                                0 => {
                                                    app.show_exit_menu = false;
                                                    app.show_tool_settings = true;
                                                    app.tool_settings_selected = if app.tool_config.language == "zh" { 1 } else { 0 };
                                                }
                                                1 => {
                                                    if !app.is_flashing {
                                                        exit = true;
                                                    } else {
                                                        app.show_exit_menu = false;
                                                        app.log("Cannot exit while flashing is active!");
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                        _ => {}
                                    }
                                } else if app.show_tool_settings {
                                    match key.code {
                                        KeyCode::Esc => {
                                            app.show_tool_settings = false;
                                        }
                                        KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right | KeyCode::Tab => {
                                            app.tool_settings_selected = if app.tool_settings_selected == 0 { 1 } else { 0 };
                                        }
                                        KeyCode::Enter => {
                                            let new_lang = if app.tool_settings_selected == 0 { "en" } else { "zh" };
                                            app.tool_config.language = new_lang.to_string();
                                            if let Err(e) = app.tool_config.save() {
                                                app.log(format!("Failed to save tool config: {}", e));
                                            } else {
                                                app.log("Tool configuration saved.");
                                            }
                                            app.show_tool_settings = false;
                                        }
                                        _ => {}
                                    }
                                } else if app.show_port_menu {
                                    match key.code {
                                        KeyCode::Esc => {
                                            app.show_port_menu = false;
                                        }
                                        KeyCode::Up => {
                                            let total_items = app.channels.len() + 1;
                                            if app.port_menu_selected > 0 {
                                                app.port_menu_selected -= 1;
                                            } else {
                                                app.port_menu_selected = total_items - 1;
                                            }
                                        }
                                        KeyCode::Down => {
                                            let total_items = app.channels.len() + 1;
                                            if app.port_menu_selected < total_items - 1 {
                                                app.port_menu_selected += 1;
                                            } else {
                                                app.port_menu_selected = 0;
                                            }
                                        }
                                        KeyCode::Enter => {
                                            let total_items = app.channels.len() + 1;
                                            if app.port_menu_selected < total_items {
                                                app.selected_channel_idx = app.port_menu_selected;
                                                if let Some(port) = app.get_selected_port() {
                                                    app.log(format!("Selected port switched to {}.", port));
                                                }
                                            }
                                            app.show_port_menu = false;
                                        }
                                        _ => {}
                                    }
                                } else if app.is_entering_password {
                                    // Password prompt input handling
                                    match key.code {
                                        KeyCode::Enter => {
                                            app.unlock_admin();
                                        }
                                        KeyCode::Esc => {
                                            app.is_entering_password = false;
                                            app.password_input.clear();
                                            app.password_incorrect = false;
                                        }
                                        KeyCode::Backspace => {
                                            app.password_input.pop();
                                        }
                                        KeyCode::Char(c) => {
                                            app.password_input.push(c);
                                        }
                                        _ => {}
                                    }
                                } else if app.is_editing_config {
                                    // Configuration field editing input handling
                                    match key.code {
                                        KeyCode::Enter => {
                                            app.config.set_field(app.selected_config_field, app.edit_buffer.clone());
                                            let _ = app.config.save_to_file(&app.config_path);
                                            app.is_editing_config = false;
                                            app.log("Config field saved.");
                                        }
                                        KeyCode::Esc => {
                                            app.is_editing_config = false;
                                        }
                                        KeyCode::Backspace => {
                                            app.edit_buffer.pop();
                                        }
                                        KeyCode::Char(c) => {
                                            app.edit_buffer.push(c);
                                        }
                                        _ => {}
                                    }
                                } else if app.active_tab == ActiveTab::Serial && app.serial_is_typing {
                                    // Serial send buffer typing input handling
                                    match key.code {
                                        KeyCode::Enter => {
                                            let cmd = app.serial_send_buffer.trim().to_string();
                                            if !cmd.is_empty() {
                                                let port = app.get_selected_port().unwrap_or_else(|| "NONE".to_string());
                                                app.log(format!("[{}] [TX] {}", port, cmd));
                                                app.serial_send_history.push(cmd.clone());

                                                // Simulated interactive command responses
                                                let response = match cmd.as_str() {
                                                    "AT" => Some("OK".to_string()),
                                                    "AT+GMR" => Some("ESP32-D0WDQ6-V3 (IDF v4.4, PioPulse Mock v0.1.3)".to_string()),
                                                    "help" | "?" => Some("Available commands: AT, AT+GMR, RESET, help".to_string()),
                                                    "RESET" => {
                                                        app.log(format!("[{}] [RX] System restarting...", port));
                                                        Some("System Initialized. Ready.".to_string())
                                                    }
                                                    _ => Some(format!("Error: Unknown command '{}'. Type 'help'.", cmd)),
                                                };
                                                if let Some(resp) = response {
                                                    app.log(format!("[{}] [RX] {}", port, resp));
                                                }

                                                app.serial_send_buffer.clear();
                                            }
                                        }
                                        KeyCode::Esc => {
                                            app.serial_is_typing = false;
                                        }
                                        KeyCode::Backspace => {
                                            app.serial_send_buffer.pop();
                                        }
                                        KeyCode::Char(c) => {
                                            app.serial_send_buffer.push(c);
                                        }
                                        _ => {}
                                    }
                                } else if app.active_tab == ActiveTab::Widgets && app.is_adding_widget {
                                    // Widgets search catalog modal input handling
                                    let filtered_items = crate::ui::widgets::get_filtered_catalog_items(
                                        &app.widget_search_input,
                                        &app.tool_config.language,
                                    );

                                    match key.code {
                                        KeyCode::Enter => {
                                            if !filtered_items.is_empty() {
                                                let selected_idx = app.add_menu_selected.min(filtered_items.len() - 1);
                                                let widget_type = filtered_items[selected_idx].2;
                                                app.add_widget(widget_type);
                                            }
                                            app.is_adding_widget = false;
                                            app.widget_search_input.clear();
                                            app.add_menu_selected = 0;
                                        }
                                        KeyCode::Esc => {
                                            app.is_adding_widget = false;
                                            app.widget_search_input.clear();
                                            app.add_menu_selected = 0;
                                        }
                                        KeyCode::Up => {
                                            if !filtered_items.is_empty() {
                                                if app.add_menu_selected > 0 {
                                                    app.add_menu_selected -= 1;
                                                } else {
                                                    app.add_menu_selected = filtered_items.len() - 1;
                                                }
                                            }
                                        }
                                        KeyCode::Down => {
                                            if !filtered_items.is_empty() {
                                                if app.add_menu_selected < filtered_items.len() - 1 {
                                                    app.add_menu_selected += 1;
                                                } else {
                                                    app.add_menu_selected = 0;
                                                }
                                            }
                                        }
                                        KeyCode::Backspace => {
                                            app.widget_search_input.pop();
                                            app.add_menu_selected = 0;
                                        }
                                        KeyCode::Char(c) => {
                                            app.widget_search_input.push(c);
                                            app.add_menu_selected = 0;
                                        }
                                        _ => {}
                                    }
                                } else {
                                    // General navigation/operation input handling
                                    match key.code {
                                         KeyCode::Esc => {
                                             app.show_exit_menu = true;
                                             app.exit_menu_selected = 0;
                                         }
                                         KeyCode::Char('q') => {
                                             app.show_exit_menu = true;
                                             app.exit_menu_selected = 1;
                                         }
                                        KeyCode::Char('1') => app.active_tab = ActiveTab::Serial,
                                        KeyCode::Char('2') => app.active_tab = ActiveTab::Plotter,
                                        KeyCode::Char('3') => app.active_tab = ActiveTab::Widgets,
                                        KeyCode::Char('4') => app.active_tab = ActiveTab::Flasher,
                                        KeyCode::Char('5') => app.active_tab = ActiveTab::Configuration,

                                        KeyCode::Char('a') | KeyCode::Char('A') => {
                                            if app.active_tab == ActiveTab::Widgets {
                                                app.is_adding_widget = true;
                                                app.widget_search_input.clear();
                                                app.add_menu_selected = 0;
                                            }
                                        }
                                        KeyCode::Char('d') | KeyCode::Char('D') => {
                                            if app.active_tab == ActiveTab::Widgets {
                                                app.delete_selected_widget();
                                            }
                                        }

                                        // Focused-pane controls: Manual IMU override, rotation, and translation (UJIKOL) when viewing IMU Cube
                                        KeyCode::Char('t') | KeyCode::Char('T') => {
                                            if app.active_tab == ActiveTab::Plotter && app.plotter_mode == PlotterMode::IMUCube {
                                                app.manual_imu_override = !app.manual_imu_override;
                                                app.log(format!("Manual IMU Override: {}", if app.manual_imu_override { "ENABLED" } else { "DISABLED" }));
                                            } else if app.active_tab == ActiveTab::Serial {
                                                app.serial_hex_mode_tx = !app.serial_hex_mode_tx;
                                                app.log(format!("Hex TX Mode: {}", if app.serial_hex_mode_tx { "ENABLED" } else { "DISABLED" }));
                                            }
                                        }
                                        KeyCode::Char('u') | KeyCode::Char('U') => {
                                            if app.active_tab == ActiveTab::Widgets && app.dashboard_widgets.get(app.selected_widget_idx) == Some(&WidgetType::Cube) {
                                                app.manual_imu_override = true;
                                                app.manual_pitch += 0.2;
                                                app.log(format!("Manual Pitch set to {:.2} rad ({:.1}°)", app.manual_pitch, app.manual_pitch.to_degrees()));
                                            }
                                        }
                                        KeyCode::Char('j') | KeyCode::Char('J') => {
                                            if app.active_tab == ActiveTab::Widgets && app.dashboard_widgets.get(app.selected_widget_idx) == Some(&WidgetType::Cube) {
                                                app.manual_imu_override = true;
                                                app.manual_pitch -= 0.2;
                                                app.log(format!("Manual Pitch set to {:.2} rad ({:.1}°)", app.manual_pitch, app.manual_pitch.to_degrees()));
                                            }
                                        }
                                        KeyCode::Char('i') | KeyCode::Char('I') => {
                                            if app.active_tab == ActiveTab::Widgets && app.dashboard_widgets.get(app.selected_widget_idx) == Some(&WidgetType::Cube) {
                                                app.manual_imu_override = true;
                                                app.manual_roll += 0.2;
                                                app.log(format!("Manual Roll set to {:.2} rad ({:.1}°)", app.manual_roll, app.manual_roll.to_degrees()));
                                            } else if app.active_tab == ActiveTab::Serial {
                                                app.serial_is_typing = true;
                                            }
                                        }
                                        KeyCode::Char('k') | KeyCode::Char('K') => {
                                            if app.active_tab == ActiveTab::Widgets && app.dashboard_widgets.get(app.selected_widget_idx) == Some(&WidgetType::Cube) {
                                                app.manual_imu_override = true;
                                                app.manual_roll -= 0.2;
                                                app.log(format!("Manual Roll set to {:.2} rad ({:.1}°)", app.manual_roll, app.manual_roll.to_degrees()));
                                            }
                                        }
                                        KeyCode::Char('o') | KeyCode::Char('O') => {
                                            if app.active_tab == ActiveTab::Widgets && app.dashboard_widgets.get(app.selected_widget_idx) == Some(&WidgetType::Cube) {
                                                app.manual_imu_override = true;
                                                app.manual_yaw += 0.2;
                                                app.log(format!("Manual Yaw set to {:.2} rad ({:.1}°)", app.manual_yaw, app.manual_yaw.to_degrees()));
                                            }
                                        }
                                        KeyCode::Char('l') | KeyCode::Char('L') => {
                                            if app.active_tab == ActiveTab::Widgets && app.dashboard_widgets.get(app.selected_widget_idx) == Some(&WidgetType::Cube) {
                                                app.manual_imu_override = true;
                                                app.manual_yaw -= 0.2;
                                                app.log(format!("Manual Yaw set to {:.2} rad ({:.1}°)", app.manual_yaw, app.manual_yaw.to_degrees()));
                                            }
                                        }
                                        KeyCode::Char('r') | KeyCode::Char('R') => {
                                            if app.active_tab == ActiveTab::Widgets && app.dashboard_widgets.get(app.selected_widget_idx) == Some(&WidgetType::Cube) {
                                                app.manual_imu_override = true;
                                                app.manual_pitch = 0.0;
                                                app.manual_roll = 0.0;
                                                app.manual_yaw = 0.0;
                                                app.manual_tx = 0.0;
                                                app.manual_ty = 0.0;
                                                app.manual_tz = 0.0;
                                                app.log("Manual override rotations & translations reset to 0.");
                                            }
                                        }

                                        KeyCode::Char('c') | KeyCode::Char('C') => {
                                            if app.active_tab == ActiveTab::Plotter {
                                                if let Some(port) = app.get_selected_port() {
                                                    app.waveform_history.remove(&port);
                                                    app.log(format!("Cleared telemetry buffer for {}.", port));
                                                }
                                            } else if !app.is_flashing {
                                                app.stats.total_passed = 0;
                                                app.stats.total_failed = 0;
                                                app.stats.total_attempted = 0;
                                                app.log("Production counters cleared.");
                                            }
                                        }
                                        KeyCode::Char('s') | KeyCode::Char('S') => {
                                            if app.active_tab == ActiveTab::Plotter {
                                                app.set_simulation_active(!app.simulation_active);
                                                app.log(format!("Simulated waveform source: {}", if app.simulation_active { "ON" } else { "OFF" }));
                                            } else if app.active_tab == ActiveTab::Serial {
                                                app.serial_auto_scroll = !app.serial_auto_scroll;
                                                app.log(format!("Auto Scroll: {}", if app.serial_auto_scroll { "ENABLED" } else { "DISABLED" }));
                                            }
                                        }
                                        KeyCode::Char('b') | KeyCode::Char('B') => {
                                            if app.active_tab == ActiveTab::Serial {
                                                app.serial_baud_rate = match app.serial_baud_rate {
                                                    9600 => 115200,
                                                    115200 => 921600,
                                                    921600 => 1152000,
                                                    _ => 9600,
                                                };
                                                app.log(format!("Baud rate set to {} bps.", app.serial_baud_rate));
                                            }
                                        }
                                        KeyCode::Char('n') | KeyCode::Char('N') => {
                                            if app.active_tab == ActiveTab::Serial {
                                                app.serial_add_newline = !app.serial_add_newline;
                                                app.log(format!("Send Newline: {}", if app.serial_add_newline { "ENABLED" } else { "DISABLED" }));
                                            }
                                        }
                                        KeyCode::Char('h') | KeyCode::Char('H') => {
                                            if app.active_tab == ActiveTab::Serial {
                                                app.serial_hex_mode_rx = !app.serial_hex_mode_rx;
                                                app.log(format!("Hex RX Mode: {}", if app.serial_hex_mode_rx { "ENABLED" } else { "DISABLED" }));
                                            }
                                        }
                                        KeyCode::Char('v') | KeyCode::Char('V') => {
                                            if app.active_tab == ActiveTab::Plotter {
                                                app.plotter_mode = match app.plotter_mode {
                                                    crate::app::PlotterMode::Waveform => crate::app::PlotterMode::BarChart,
                                                    crate::app::PlotterMode::BarChart => crate::app::PlotterMode::Histogram,
                                                    crate::app::PlotterMode::Histogram => crate::app::PlotterMode::FftSpectrum,
                                                    crate::app::PlotterMode::FftSpectrum
                                                    | crate::app::PlotterMode::IMUCube
                                                    | crate::app::PlotterMode::RoiImage => crate::app::PlotterMode::Waveform,
                                                };
                                                app.log(format!("Plotter View Mode set to {:?}", app.plotter_mode));
                                            }
                                        }
                                        KeyCode::Char('m') | KeyCode::Char('M') => {
                                            if app.active_tab == ActiveTab::Plotter {
                                                app.vofa_mode = match app.vofa_mode {
                                                    crate::vofa::VofaMode::FireWater => crate::vofa::VofaMode::JustFloat,
                                                    crate::vofa::VofaMode::JustFloat => crate::vofa::VofaMode::IndexFloat,
                                                    crate::vofa::VofaMode::IndexFloat => crate::vofa::VofaMode::FireWater,
                                                };
                                                crate::vofa::ACTIVE_VOFA_MODE.store(
                                                    app.vofa_mode.to_u8(),
                                                    std::sync::atomic::Ordering::Relaxed,
                                                );
                                                app.log(format!("VOFA+ Protocol Mode set to {:?}", app.vofa_mode));
                                            }
                                        }
                                        KeyCode::Left => {
                                            if app.active_tab == ActiveTab::Plotter {
                                                let limit = app.channels.len() + 1;
                                                if limit > 0 {
                                                    if app.selected_channel_idx > 0 {
                                                        app.selected_channel_idx -= 1;
                                                    } else {
                                                        app.selected_channel_idx = limit - 1;
                                                    }
                                                }
                                            } else if app.active_tab == ActiveTab::Widgets {
                                                if app.dashboard_widgets.get(app.selected_widget_idx) == Some(&WidgetType::Cube) && app.manual_imu_override {
                                                    app.manual_tx -= 0.15;
                                                    app.log(format!("Manual Trans X set to {:.2}", app.manual_tx));
                                                } else if !app.dashboard_widgets.is_empty() {
                                                    if app.selected_widget_idx > 0 {
                                                        app.selected_widget_idx -= 1;
                                                    } else {
                                                        app.selected_widget_idx = app.dashboard_widgets.len() - 1;
                                                    }
                                                }
                                            }
                                        }
                                        KeyCode::Char('[') => {
                                            if app.active_tab == ActiveTab::Plotter {
                                                let limit = app.channels.len() + 1;
                                                if limit > 0 {
                                                    if app.selected_channel_idx > 0 {
                                                        app.selected_channel_idx -= 1;
                                                    } else {
                                                        app.selected_channel_idx = limit - 1;
                                                    }
                                                }
                                            }
                                        }
                                        KeyCode::Right => {
                                            if app.active_tab == ActiveTab::Plotter {
                                                let limit = app.channels.len() + 1;
                                                if limit > 0 {
                                                    if app.selected_channel_idx < limit - 1 {
                                                        app.selected_channel_idx += 1;
                                                    } else {
                                                        app.selected_channel_idx = 0;
                                                    }
                                                }
                                            } else if app.active_tab == ActiveTab::Widgets {
                                                if app.dashboard_widgets.get(app.selected_widget_idx) == Some(&WidgetType::Cube) && app.manual_imu_override {
                                                    app.manual_tx += 0.15;
                                                    app.log(format!("Manual Trans X set to {:.2}", app.manual_tx));
                                                } else if !app.dashboard_widgets.is_empty() {
                                                    if app.selected_widget_idx < app.dashboard_widgets.len() - 1 {
                                                        app.selected_widget_idx += 1;
                                                    } else {
                                                        app.selected_widget_idx = 0;
                                                    }
                                                }
                                            }
                                        }
                                        KeyCode::Char(']') => {
                                            if app.active_tab == ActiveTab::Plotter {
                                                let limit = app.channels.len() + 1;
                                                if limit > 0 {
                                                    if app.selected_channel_idx < limit - 1 {
                                                        app.selected_channel_idx += 1;
                                                    } else {
                                                        app.selected_channel_idx = 0;
                                                    }
                                                }
                                            }
                                        }
                                        KeyCode::PageUp => {
                                            if app.active_tab == ActiveTab::Widgets && app.dashboard_widgets.get(app.selected_widget_idx) == Some(&WidgetType::Cube) {
                                                app.manual_imu_override = true;
                                                app.manual_tz += 0.15;
                                                app.log(format!("Manual Trans Z set to {:.2}", app.manual_tz));
                                            }
                                        }
                                        KeyCode::PageDown => {
                                            if app.active_tab == ActiveTab::Widgets && app.dashboard_widgets.get(app.selected_widget_idx) == Some(&WidgetType::Cube) {
                                                app.manual_imu_override = true;
                                                app.manual_tz -= 0.15;
                                                app.log(format!("Manual Trans Z set to {:.2}", app.manual_tz));
                                            }
                                        }
                                        KeyCode::F(1) => {
                                            if app.admin_mode {
                                                app.lock_admin();
                                            } else {
                                                app.is_entering_password = true;
                                            }
                                        }
                                        KeyCode::F(2) => {
                                            app.show_sidebar = !app.show_sidebar;
                                            app.log(format!("Sidebar visibility: {}", if app.show_sidebar { "SHOWN" } else { "HIDDEN" }));
                                        }
                                        KeyCode::Char(' ') => {
                                            if app.active_tab == ActiveTab::Plotter {
                                                app.set_simulation_active(!app.simulation_active);
                                                app.log(format!("Simulated waveform source: {}", if app.simulation_active { "ON" } else { "OFF" }));
                                            } else if app.active_tab == ActiveTab::Flasher || app.active_tab == ActiveTab::Configuration {
                                                app.start_flashing(tx.clone());
                                            } else if app.active_tab == ActiveTab::Serial {
                                                app.serial_is_typing = true;
                                            }
                                        }
                                        KeyCode::Up => {
                                            if app.active_tab == ActiveTab::Configuration {
                                                if app.selected_config_field > 0 {
                                                    app.selected_config_field -= 1;
                                                } else {
                                                    app.selected_config_field = 13;
                                                }
                                            }
                                        }
                                        KeyCode::Down => {
                                            if app.active_tab == ActiveTab::Configuration {
                                                if app.selected_config_field < 13 {
                                                    app.selected_config_field += 1;
                                                } else {
                                                    app.selected_config_field = 0;
                                                }
                                            }
                                        }
                                        KeyCode::Enter => {
                                            if app.active_tab == ActiveTab::Configuration && app.admin_mode {
                                                app.is_editing_config = true;
                                                app.edit_buffer = app.config.get_field(app.selected_config_field);
                                            } else if app.active_tab == ActiveTab::Serial {
                                                app.serial_is_typing = true;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        Event::Mouse(mouse) => {
                            match mouse.kind {
                                crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left)
                                | crossterm::event::MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
                                    if !app.handle_mouse_click(mouse.column, mouse.row, tx.clone()) {
                                        exit = true;
                                    }
                                }
                                crossterm::event::MouseEventKind::ScrollUp => {
                                    app.handle_mouse_scroll(true);
                                }
                                crossterm::event::MouseEventKind::ScrollDown => {
                                    app.handle_mouse_scroll(false);
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
