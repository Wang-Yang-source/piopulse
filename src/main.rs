mod app;
mod config;
mod nvs;
mod ui;
mod vofa;
mod worker;

use app::{ActiveTab, App, PlotterMode, WidgetType};
use crossterm::{
    event::{
        DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEventKind,
        KeyModifiers,
    },
    execute,
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, SetTitle, disable_raw_mode, enable_raw_mode,
    },
};
use futures::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, sync::Arc, time::Duration};
use tokio::sync::mpsc;

enum CliMode {
    Tui {
        external_platformio_ini: Option<std::path::PathBuf>,
    },
    Ports,
    Flash {
        external_platformio_ini: Option<std::path::PathBuf>,
        port: Option<String>,
        all: bool,
    },
}

#[cfg(unix)]
async fn shutdown_signal() -> io::Result<()> {
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())?;

    tokio::select! {
        _ = sigterm.recv() => {}
        _ = sigint.recv() => {}
    }

    Ok(())
}

#[cfg(not(unix))]
async fn shutdown_signal() -> io::Result<()> {
    tokio::signal::ctrl_c().await
}

fn restore_terminal() {
    let _ = disable_raw_mode();
    let mut stdout = std::io::stdout();
    let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
}

fn print_help() {
    println!("PioPulse - embedded flashing and serial production tool");
    println!();
    println!("Usage:");
    println!("  piopulse                         Start TUI");
    println!("  piopulse ports                   List detected devices");
    println!("  piopulse flash [--all]           Flash detected device(s) without TUI");
    println!("  piopulse flash --port <name>     Flash one device without TUI");
    println!("  piopulse [platformio.ini]        Start TUI using a PlatformIO project");
    println!();
    println!("Options:");
    println!("  -h, --help                       Show this help message");
    println!("  -v, --version                    Show version information");
    println!("  --platformio-ini, --pio-ini FILE Use an external platformio.ini");
}

fn parse_cli_args(args: &[String]) -> CliMode {
    let mut external_platformio_ini: Option<std::path::PathBuf> = None;
    if args.len() <= 1 {
        return CliMode::Tui {
            external_platformio_ini,
        };
    }

    match args[1].as_str() {
        "--version" | "-v" | "version" => {
            println!("piopulse {}", env!("CARGO_PKG_VERSION"));
            std::process::exit(0);
        }
        "--help" | "-h" | "help" => {
            print_help();
            std::process::exit(0);
        }
        "ports" | "devices" => CliMode::Ports,
        "flash" => {
            let mut idx = 2;
            let mut port = None;
            let mut all = false;
            while idx < args.len() {
                match args[idx].as_str() {
                    "--all" | "-a" => all = true,
                    "--port" | "-p" => {
                        idx += 1;
                        if idx >= args.len() {
                            eprintln!("Error: --port requires a device name.");
                            std::process::exit(2);
                        }
                        port = Some(args[idx].clone());
                    }
                    "--platformio-ini" | "--pio-ini" => {
                        idx += 1;
                        if idx >= args.len() {
                            eprintln!("Error: --platformio-ini requires a file path.");
                            std::process::exit(2);
                        }
                        external_platformio_ini = Some(std::path::PathBuf::from(&args[idx]));
                    }
                    other => {
                        eprintln!("Error: Unknown flash argument '{}'", other);
                        eprintln!("Run 'piopulse --help' for usage details.");
                        std::process::exit(2);
                    }
                }
                idx += 1;
            }
            CliMode::Flash {
                external_platformio_ini,
                port,
                all,
            }
        }
        _ => {
            let mut idx = 1;
            while idx < args.len() {
                match args[idx].as_str() {
                    "--platformio-ini" | "--pio-ini" => {
                        idx += 1;
                        if idx >= args.len() {
                            eprintln!("Error: --platformio-ini requires a file path.");
                            std::process::exit(2);
                        }
                        external_platformio_ini = Some(std::path::PathBuf::from(&args[idx]));
                    }
                    other => {
                        let path = std::path::PathBuf::from(other);
                        if path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .is_some_and(|name| name == "platformio.ini")
                        {
                            external_platformio_ini = Some(path);
                        } else {
                            eprintln!("Error: Unknown argument '{}'", other);
                            eprintln!("Run 'piopulse --help' for usage details.");
                            std::process::exit(2);
                        }
                    }
                }
                idx += 1;
            }
            CliMode::Tui {
                external_platformio_ini,
            }
        }
    }
}

fn print_detected_ports() {
    let ports = worker::get_available_serial_ports();
    if ports.is_empty() {
        println!("No devices detected.");
        return;
    }
    for port in ports {
        println!(
            "{}\tvid={}\tpid={}\tproduct={}\tmanufacturer={}",
            port.name,
            port.vid
                .map(|v| format!("{:04x}", v))
                .unwrap_or_else(|| "-".to_string()),
            port.pid
                .map(|p| format!("{:04x}", p))
                .unwrap_or_else(|| "-".to_string()),
            port.product.unwrap_or_else(|| "-".to_string()),
            port.manufacturer.unwrap_or_else(|| "-".to_string())
        );
    }
}

async fn run_cli_flash(
    external_platformio_ini: Option<std::path::PathBuf>,
    requested_port: Option<String>,
    all: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = "build/piopulse.toml".to_string();
    let mut app = match external_platformio_ini {
        Some(pio_ini) => App::new_with_platformio_ini(config_path, Some(pio_ini)),
        None => App::new(config_path),
    };
    app.scan_ports(None);

    if !app.ensure_flash_manifest_ready() {
        return Err("flash manifest is not ready".into());
    }

    let target_ports = if let Some(port) = requested_port {
        vec![port]
    } else if all {
        app.channels
            .iter()
            .map(|channel| channel.port.clone())
            .collect()
    } else {
        match app.channels.as_slice() {
            [] => return Err("no devices detected".into()),
            [channel] => vec![channel.port.clone()],
            _ => {
                return Err(
                    "multiple devices detected; use `piopulse flash --all` or `--port <name>`"
                        .into(),
                );
            }
        }
    };

    if target_ports.is_empty() {
        return Err("no devices selected for flashing".into());
    }

    println!("PioPulse CLI flash: {} device(s)", target_ports.len());
    println!("Config: {}", app.config_path);
    app.log(format!(
        "--- CLI Flash Started for {} device(s) ---",
        target_ports.len()
    ));

    let (tx, mut rx) = mpsc::channel::<worker::WorkerMessage>(200);
    let config = Arc::new(app.config.clone());
    for port in &target_ports {
        println!("[{}] start", port);
        app.log(format!("[{}] CLI flash queued.", port));
        worker::start_flashing_task(port.clone(), config.clone(), tx.clone());
    }
    drop(tx);

    let mut finished = 0usize;
    let mut failed = 0usize;
    while let Some(message) = rx.recv().await {
        match message {
            worker::WorkerMessage::StatusUpdate {
                port,
                status,
                progress,
                speed,
            } => {
                let line = format!("[{}] STATUS {}% {} ({})", port, progress, status, speed);
                println!("{}", line);
                app.log(line);
            }
            worker::WorkerMessage::MacAddressDetected { port, mac, chip } => {
                let line = format!("[{}] DETECTED chip={} mac={}", port, chip, mac);
                println!("{}", line);
                app.log(line);
            }
            worker::WorkerMessage::ProvisioningGenerated {
                port,
                serial_number,
                device_name,
            } => {
                let line = format!(
                    "[{}] PROVISION serial={} device={}",
                    port, serial_number, device_name
                );
                println!("{}", line);
                app.log(line);
            }
            worker::WorkerMessage::ProductionStep { port, step, detail } => {
                let line = format!("[{}] STEP {}={}", port, step, detail);
                println!("{}", line);
                app.log(line);
            }
            worker::WorkerMessage::Log { port, message } => {
                let line = format!("[{}] {}", port, message);
                println!("{}", line);
                app.log(line);
            }
            worker::WorkerMessage::Finished {
                port,
                success,
                error_msg,
                mac,
            } => {
                finished += 1;
                if !success {
                    failed += 1;
                }
                let line = format!(
                    "[{}] FINISHED result={} mac={} error={}",
                    port,
                    if success { "OK" } else { "FAIL" },
                    mac.unwrap_or_else(|| "-".to_string()),
                    error_msg.clone().unwrap_or_default()
                );
                println!("{}", line);
                app.log(line);
                if let Some(err) = error_msg {
                    eprintln!("[{}] error: {}", port, err);
                }
                if finished >= target_ports.len() {
                    break;
                }
            }
            _ => {}
        }
    }

    let passed = target_ports.len().saturating_sub(failed);
    println!("Summary: passed={}, failed={}", passed, failed);
    app.log(format!(
        "--- CLI Flash Completed. Passed: {}, Failed: {} ---",
        passed, failed
    ));

    if failed == 0 {
        Ok(())
    } else {
        Err(format!("{} device(s) failed", failed).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let cli_mode = parse_cli_args(&args);

    let external_platformio_ini = match cli_mode {
        CliMode::Ports => {
            print_detected_ports();
            return Ok(());
        }
        CliMode::Flash {
            external_platformio_ini,
            port,
            all,
        } => {
            return run_cli_flash(external_platformio_ini, port, all).await;
        }
        CliMode::Tui {
            external_platformio_ini,
        } => external_platformio_ini,
    };

    // Setup panic hook to restore terminal
    std::panic::set_hook(Box::new(|info| {
        restore_terminal();
        eprintln!("Panic occurred: {:?}", info);
    }));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        SetTitle("☕PioPulse"),
        EnterAlternateScreen,
        EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app on a blocking worker so the terminal can show startup progress immediately.
    let config_path = "build/piopulse.toml".to_string();
    let startup_lang = config::ToolConfig::load().language;
    let startup_started = std::time::Instant::now();
    let config_path_for_startup = config_path.clone();
    let mut startup_task = tokio::task::spawn_blocking(move || {
        let mut app = match external_platformio_ini {
            Some(pio_ini) => App::new_with_platformio_ini(config_path_for_startup, Some(pio_ini)),
            None => App::new(config_path_for_startup),
        };
        app.scan_ports(None);
        app
    });

    // Setup input events stream, signal handling & ticks
    let mut reader = EventStream::new();
    let shutdown_signal = shutdown_signal();
    tokio::pin!(shutdown_signal);
    let mut startup_interval = tokio::time::interval(Duration::from_millis(100));

    let mut app = loop {
        terminal.draw(|f| ui::draw_startup_screen(f, &startup_lang, startup_started.elapsed()))?;

        tokio::select! {
            app_result = &mut startup_task => {
                match app_result {
                    Ok(app) => break app,
                    Err(err) => {
                        restore_terminal();
                        return Err(Box::new(err) as Box<dyn std::error::Error>);
                    }
                }
            }
            signal_result = &mut shutdown_signal => {
                signal_result?;
                restore_terminal();
                std::process::exit(0);
            }
            _ = startup_interval.tick() => {}
            maybe_event = reader.next() => {
                if let Some(Ok(Event::Key(key))) = maybe_event {
                    if key.kind == KeyEventKind::Press
                        && key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        restore_terminal();
                        std::process::exit(0);
                    }
                }
            }
        }
    };

    // Create channel for worker messages
    let (tx, mut rx) = mpsc::channel(100);
    app.worker_tx = Some(tx.clone());

    // Auto-build PlatformIO project if detected
    app.trigger_startup_build(tx.clone());

    let mut interval = tokio::time::interval(Duration::from_millis(100));
    let mut exit = false;

    while !exit {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        tokio::select! {
            // Background worker messages
            Some(msg) = rx.recv() => {
                app.handle_worker_message(msg);
                while let Ok(msg) = rx.try_recv() {
                    app.handle_worker_message(msg);
                }
            }
            // Signal handlers
            signal_result = &mut shutdown_signal => {
                signal_result?;
                exit = true;
            }
            // Standard ticks
            _ = interval.tick() => {
                app.tick();

                // Scan ports automatically every 2 seconds
                if app.last_port_scan.elapsed() > Duration::from_secs(2) {
                    app.scan_ports(Some(tx.clone()));
                    app.update_auto_flash_sensing(tx.clone());
                    app.last_port_scan = std::time::Instant::now();
                }
            }
            // User keyboard/terminal events
            maybe_event = reader.next() => {
                if let Some(Ok(event)) = maybe_event {
                    match event {
                        Event::Key(key) => {
                            if key.kind == KeyEventKind::Press {
                                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                                    exit = true;
                                } else if app.splash_ticks_remaining.is_some() {
                                    app.finish_splash();
                                } else if app.show_exit_menu {
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
                                            let total_items = app.channels.len();
                                            if total_items > 0 {
                                                if app.port_menu_selected > 0 {
                                                    app.port_menu_selected -= 1;
                                                } else {
                                                    app.port_menu_selected = total_items - 1;
                                                }
                                            }
                                        }
                                        KeyCode::Down => {
                                            let total_items = app.channels.len();
                                            if total_items > 0 {
                                                if app.port_menu_selected < total_items - 1 {
                                                    app.port_menu_selected += 1;
                                                } else {
                                                    app.port_menu_selected = 0;
                                                }
                                            }
                                        }
                                        KeyCode::Enter => {
                                            let total_items = app.channels.len();
                                            if total_items > 0 && app.port_menu_selected < total_items {
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
                                            app.config.set_field(app.config_real_field_index(app.selected_config_field), app.edit_buffer.clone());
                                            let saved = app.save_config("save config field");
                                            app.is_editing_config = false;
                                            if saved {
                                                app.log("Config field saved.");
                                            }
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
                                } else if app.show_manifest_delete_confirm {
                                    match key.code {
                                        KeyCode::Enter => {
                                            app.confirm_manifest_delete();
                                        }
                                        KeyCode::Esc => {
                                            app.show_manifest_delete_confirm = false;
                                            app.manifest_delete_image_label.clear();
                                        }
                                        _ => {}
                                    }
                                } else if app.show_manifest_edit_modal {
                                    match key.code {
                                        KeyCode::Enter => {
                                            app.save_manifest_edit();
                                        }
                                        KeyCode::Esc => {
                                            app.show_manifest_edit_modal = false;
                                        }
                                        KeyCode::Backspace => {
                                            app.manifest_edit_input.pop();
                                        }
                                        KeyCode::Char(c) => {
                                            app.manifest_edit_input.push(c);
                                        }
                                        _ => {}
                                    }
                                } else if app.show_file_picker {
                                    match key.code {
                                        KeyCode::Enter => {
                                            app.select_file_picker_item();
                                        }
                                        KeyCode::Esc => {
                                            app.show_file_picker = false;
                                        }
                                        KeyCode::Up => {
                                            if !app.file_picker_items.is_empty() {
                                                if app.file_picker_selected_idx > 0 {
                                                    app.file_picker_selected_idx -= 1;
                                                } else {
                                                    app.file_picker_selected_idx = app.file_picker_items.len() - 1;
                                                }
                                            }
                                        }
                                        KeyCode::Down => {
                                            if !app.file_picker_items.is_empty() {
                                                if app.file_picker_selected_idx < app.file_picker_items.len() - 1 {
                                                    app.file_picker_selected_idx += 1;
                                                } else {
                                                    app.file_picker_selected_idx = 0;
                                                }
                                            }
                                        }
                                        KeyCode::Backspace => {
                                            if !app.file_picker_search_input.is_empty() {
                                                app.file_picker_search_input.pop();
                                                app.file_picker_selected_idx = 0;
                                                app.refresh_file_picker_items();
                                            } else {
                                                if let Some(parent) = app.file_picker_current_dir.parent() {
                                                    app.file_picker_current_dir = parent.to_path_buf();
                                                    app.file_picker_selected_idx = 0;
                                                    app.refresh_file_picker_items();
                                                }
                                            }
                                        }
                                        KeyCode::Char(c) => {
                                            app.file_picker_search_input.push(c);
                                            app.file_picker_selected_idx = 0;
                                            app.refresh_file_picker_items();
                                        }
                                        _ => {}
                                    }
                                } else if app.show_custom_baud_modal {
                                    match key.code {
                                        KeyCode::Enter => {
                                            app.apply_custom_baud_rate();
                                        }
                                        KeyCode::Esc => {
                                            app.show_custom_baud_modal = false;
                                        }
                                        KeyCode::Tab => {
                                            app.start_auto_baud_detection();
                                        }
                                        KeyCode::Backspace => {
                                            app.custom_baud_input.pop();
                                        }
                                        KeyCode::Char(c) => {
                                            if c.is_ascii_digit() {
                                                app.custom_baud_input.push(c);
                                            }
                                        }
                                        _ => {}
                                    }
                                } else if app.show_auto_reply_modal {
                                    match key.code {
                                        KeyCode::Enter => {
                                            app.save_auto_reply();
                                        }
                                        KeyCode::Esc => {
                                            app.show_auto_reply_modal = false;
                                        }
                                        KeyCode::Tab => {
                                            app.auto_reply_focused_field = (app.auto_reply_focused_field + 1) % 2;
                                        }
                                        KeyCode::Backspace => {
                                            if app.auto_reply_focused_field == 0 {
                                                app.auto_reply_pattern_input.pop();
                                            } else {
                                                app.auto_reply_response_input.pop();
                                            }
                                        }
                                        KeyCode::Char(c) => {
                                            if app.auto_reply_focused_field == 0 {
                                                app.auto_reply_pattern_input.push(c);
                                            } else {
                                                app.auto_reply_response_input.push(c);
                                            }
                                        }
                                        _ => {}
                                    }
                                } else if app.active_tab == ActiveTab::Serial && app.serial_is_typing {
                                    // Serial send buffer typing input handling
                                    match key.code {
                                        KeyCode::Enter => {
                                            let cmd = app.serial_send_buffer.trim().to_string();
                                            if !cmd.is_empty() {
                                                app.submit_serial_command(&cmd);

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
                                    app.is_adding_widget = false;
                                    app.widget_search_input.clear();
                                    app.add_menu_selected = 0;
                                    app.log("Probe dashboard is fixed.");
                                } else {
                                    // General navigation/operation input handling
                                    if app.active_tab == ActiveTab::Widgets {
                                        app.dashboard_widgets.clear();
                                        app.selected_widget_idx = 0;
                                        app.is_adding_widget = false;
                                    }
                                    match key.code {
                                         KeyCode::Esc => {
                                             app.show_exit_menu = true;
                                             app.exit_menu_selected = 0;
                                         }
                                         KeyCode::Char('q') => {
                                             app.show_exit_menu = true;
                                             app.exit_menu_selected = 1;
                                         }
                                        KeyCode::Char('1') => {
                                            app.active_tab = ActiveTab::Serial;
                                            app.log("Switched to Serial tab.");
                                        }
                                        KeyCode::Char('2') => {
                                            app.active_tab = ActiveTab::Plotter;
                                            app.log("Switched to Plotter tab.");
                                        }
                                        KeyCode::Char('3') => {
                                            app.active_tab = ActiveTab::Widgets;
                                            app.dashboard_widgets.clear();
                                            app.is_adding_widget = false;
                                            app.log("Switched to probe dashboard.");
                                        }
                                        KeyCode::Char('4') => {
                                            app.active_tab = ActiveTab::Flasher;
                                            app.log("Switched to Flasher tab.");
                                        }
                                        KeyCode::Char('5') => {
                                            app.active_tab = ActiveTab::Configuration;
                                            app.log("Switched to Configuration tab.");
                                        }

                                        KeyCode::Char('p') | KeyCode::Char('P') => {
                                            if app.active_tab == ActiveTab::Serial {
                                                app.show_port_menu = !app.show_port_menu;
                                                app.port_menu_selected = app.selected_channel_idx;
                                            }
                                        }

                                         KeyCode::Char('a') | KeyCode::Char('A') => {
                                             if app.active_tab == ActiveTab::Widgets {
                                                 app.dashboard_widgets.clear();
                                                 app.is_adding_widget = false;
                                                 app.widget_search_input.clear();
                                                 app.add_menu_selected = 0;
                                                 app.log("Probe dashboard is fixed.");
                                             } else if app.active_tab == ActiveTab::Flasher {
                                                 app.auto_flash = !app.auto_flash;
                                                 app.log(format!("Auto-Flash mode: {}", if app.auto_flash { "ENABLED" } else { "DISABLED" }));
                                             } else if app.active_tab == ActiveTab::Serial {
                                                 app.serial_auto_reply_enabled = !app.serial_auto_reply_enabled;
                                                 if app.serial_auto_reply_enabled && app.serial_auto_reply_pattern.is_empty() {
                                                     app.show_auto_reply_modal = true;
                                                     app.auto_reply_pattern_input = app.serial_auto_reply_pattern.clone();
                                                     app.auto_reply_response_input = app.serial_auto_reply_response.clone();
                                                     app.auto_reply_focused_field = 0;
                                                 }
                                                 app.log(format!("Auto Reply: {}", if app.serial_auto_reply_enabled { "ENABLED" } else { "DISABLED" }));
                                             } else {
                                                 app.log("Shortcut A is available on Flasher (auto-flash) or Serial (auto-reply).");
                                             }
                                         }
                                         KeyCode::Char('d') | KeyCode::Char('D') => {
                                             if app.active_tab == ActiveTab::Widgets {
                                                 app.delete_selected_widget();
                                             } else if app.active_tab == ActiveTab::Serial {
                                                 app.toggle_dtr();
                                             } else {
                                                 app.log("Shortcut D is available on Serial (DTR).");
                                             }
                                         }
                                         KeyCode::Char('g') | KeyCode::Char('G') => {
                                             if app.active_tab == ActiveTab::Serial {
                                                 app.toggle_rts();
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
                                            if app.active_tab == ActiveTab::Serial {
                                                app.toggle_serial_recording();
                                            } else if app.active_tab == ActiveTab::Widgets && app.dashboard_widgets.get(app.selected_widget_idx) == Some(&WidgetType::Cube) {
                                                app.manual_imu_override = true;
                                                app.manual_pitch = 0.0;
                                                app.manual_roll = 0.0;
                                                app.manual_yaw = 0.0;
                                                app.manual_tx = 0.0;
                                                app.manual_ty = 0.0;
                                                app.manual_tz = 0.0;
                                                app.cube_zoom = 1.0;
                                                app.log("Manual override rotations, translations & zoom reset.");
                                            }
                                        }
                                        KeyCode::Char('x') | KeyCode::Char('X') => {
                                            if app.active_tab == ActiveTab::Widgets && app.dashboard_widgets.get(app.selected_widget_idx) == Some(&WidgetType::Cube) {
                                                app.show_cube_axes = !app.show_cube_axes;
                                                app.log(format!("Cube Axes Visibility: {}", if app.show_cube_axes { "ON" } else { "OFF" }));
                                            }
                                        }
                                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                                            if app.active_tab == ActiveTab::Serial {
                                                if app.serial_playback_active {
                                                    app.stop_serial_timeline_playback();
                                                } else {
                                                    app.start_serial_timeline_playback();
                                                }
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
                                            } else {
                                                app.log("Cannot clear production counters while flashing is active.");
                                            }
                                        }
                                        KeyCode::Char('s') | KeyCode::Char('S') => {
                                            if app.active_tab == ActiveTab::Plotter {
                                                app.set_plotter_active(!app.plotter_active);
                                                app.log(format!("Plotter active: {}", if app.plotter_active { "ON" } else { "OFF" }));
                                            } else if app.active_tab == ActiveTab::Serial {
                                                app.serial_auto_scroll = !app.serial_auto_scroll;
                                                app.log(format!("Auto Scroll: {}", if app.serial_auto_scroll { "ENABLED" } else { "DISABLED" }));
                                            }
                                        }
                                        KeyCode::Char('b') | KeyCode::Char('B') => {
                                            if app.active_tab == ActiveTab::Serial {
                                                app.show_custom_baud_modal = true;
                                                app.custom_baud_input = app.serial_baud_rate.to_string();
                                                app.log("Custom baud dialog opened.");
                                            } else if app.active_tab == ActiveTab::Flasher {
                                                app.flash_batch_mode = !app.flash_batch_mode;
                                                app.log(format!("Flash Mode set to: {}", if app.flash_batch_mode { "BATCH" } else { "SINGLE" }));
                                            } else {
                                                app.log("Shortcut B is available on Flasher (batch/single) or Serial (baud).");
                                            }
                                        }
                                        KeyCode::Char('f') | KeyCode::Char('F') => {
                                            if app.active_tab == ActiveTab::Serial {
                                                app.cycle_serial_frame_format();
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
                                                app.cycle_plotter_mode();
                                            }
                                        }
                                        KeyCode::Char('m') | KeyCode::Char('M') => {
                                            if app.active_tab == ActiveTab::Serial {
                                                app.toggle_serial_monitor();
                                            } else if app.active_tab == ActiveTab::Plotter {
                                                app.cycle_vofa_mode();
                                             } else if app.active_tab == ActiveTab::Flasher {
                                                 app.toggle_merged_flash();
                                            } else {
                                                app.log("Shortcut M is available on Serial (monitor), Plotter (VOFA), or Flasher (merged mode).");
                                            }
                                        }
                                        KeyCode::Char('+') | KeyCode::Char('=') => {
                                            if app.active_tab == ActiveTab::Plotter {
                                                app.zoom_plotter_view(true);
                                            } else if app.active_tab == ActiveTab::Widgets && app.dashboard_widgets.get(app.selected_widget_idx) == Some(&WidgetType::Cube) {
                                                app.cube_zoom *= 1.2;
                                                app.log(format!("Cube Zoom: {:.2}x", app.cube_zoom));
                                            }
                                        }
                                        KeyCode::Char('-') | KeyCode::Char('_') => {
                                            if app.active_tab == ActiveTab::Plotter {
                                                app.zoom_plotter_view(false);
                                            } else if app.active_tab == ActiveTab::Widgets && app.dashboard_widgets.get(app.selected_widget_idx) == Some(&WidgetType::Cube) {
                                                app.cube_zoom /= 1.2;
                                                app.log(format!("Cube Zoom: {:.2}x", app.cube_zoom));
                                            }
                                        }
                                        KeyCode::Char(',') | KeyCode::Char('<') => {
                                            if app.active_tab == ActiveTab::Plotter {
                                                app.pan_plotter_view(true);
                                            }
                                        }
                                        KeyCode::Char('.') | KeyCode::Char('>') => {
                                            if app.active_tab == ActiveTab::Plotter {
                                                app.pan_plotter_view(false);
                                            }
                                        }
                                        KeyCode::Char('0') => {
                                            if app.active_tab == ActiveTab::Plotter {
                                                app.reset_plotter_view();
                                            }
                                        }
                                        KeyCode::Left => {
                                            if app.active_tab == ActiveTab::Plotter {
                                                let limit = app.channels.len();
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
                                                let limit = app.channels.len();
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
                                                let limit = app.channels.len();
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
                                                let limit = app.channels.len();
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
                                                app.log("Admin unlock prompt opened.");
                                            }
                                        }
                                        KeyCode::F(2) => {
                                            app.show_sidebar = !app.show_sidebar;
                                            app.log(format!("Sidebar visibility: {}", if app.show_sidebar { "SHOWN" } else { "HIDDEN" }));
                                        }
                                        KeyCode::Char(' ') => {
                                            if app.active_tab == ActiveTab::Plotter {
                                                app.set_plotter_active(!app.plotter_active);
                                                app.log(format!("Plotter active: {}", if app.plotter_active { "ON" } else { "OFF" }));
                                            } else if app.active_tab == ActiveTab::Flasher {
                                                if app.flash_batch_mode {
                                                    app.start_flashing(tx.clone());
                                                } else {
                                                    app.start_flashing_selected(tx.clone());
                                                }
                                            } else if app.active_tab == ActiveTab::Configuration {
                                                app.start_flashing(tx.clone());
                                            } else if app.active_tab == ActiveTab::Serial {
                                                app.serial_is_typing = true;
                                                app.log("Serial typing mode opened.");
                                            } else if app.active_tab == ActiveTab::Widgets {
                                                app.dashboard_widgets.clear();
                                                app.is_adding_widget = false;
                                                app.log("Probe dashboard is fixed; module focus is disabled.");
                                            }
                                        }
                                        KeyCode::Up => {
                                            if app.active_tab == ActiveTab::Configuration {
                                                if app.selected_config_field > 0 {
                                                    app.selected_config_field -= 1;
                                                } else {
                                                    app.selected_config_field =
                                                        app.config_field_map.len() - 1;
                                                }
                                            } else if app.active_tab == ActiveTab::Flasher {
                                                app.move_flash_selection(-1);
                                            }
                                        }
                                        KeyCode::Down => {
                                            if app.active_tab == ActiveTab::Configuration {
                                                if app.selected_config_field
                                                    < app.config_field_map.len() - 1
                                                {
                                                    app.selected_config_field += 1;
                                                } else {
                                                    app.selected_config_field = 0;
                                                }
                                            } else if app.active_tab == ActiveTab::Flasher {
                                                app.move_flash_selection(1);
                                            }
                                        }
                                        KeyCode::Enter => {
                                            if app.active_tab == ActiveTab::Configuration && app.admin_mode {
                                                app.is_editing_config = true;
                                                app.edit_buffer = app.config.get_field(app.config_real_field_index(app.selected_config_field));
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
                                crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                                    if !app.handle_mouse_click(mouse.column, mouse.row, tx.clone()) {
                                        exit = true;
                                    }
                                }
                                crossterm::event::MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
                                    app.handle_mouse_drag(mouse.column, mouse.row, crossterm::event::MouseButton::Left);
                                    if !app.handle_mouse_click(mouse.column, mouse.row, tx.clone()) {
                                        exit = true;
                                    }
                                }
                                crossterm::event::MouseEventKind::Drag(crossterm::event::MouseButton::Right) | crossterm::event::MouseEventKind::Drag(crossterm::event::MouseButton::Middle) => {
                                    app.handle_mouse_drag(mouse.column, mouse.row, crossterm::event::MouseButton::Right);
                                }
                                crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Right) => {
                                    let _ = app.handle_mouse_right_click(mouse.column, mouse.row, tx.clone());
                                }
                                crossterm::event::MouseEventKind::ScrollUp => {
                                    app.handle_mouse_scroll(true, mouse.column, mouse.row);
                                }
                                crossterm::event::MouseEventKind::ScrollDown => {
                                    app.handle_mouse_scroll(false, mouse.column, mouse.row);
                                }
                                crossterm::event::MouseEventKind::Moved => {
                                    app.handle_mouse_move(mouse.column, mouse.row);
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
    restore_terminal();
    terminal.show_cursor()?;

    Ok(())
}
