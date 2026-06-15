mod app;
mod config;
mod ui;
mod worker;

use app::{ActiveTab, App};
use config::ProjectConfig;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, EventStream, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};
use tokio::sync::mpsc;

fn get_config_field(config: &ProjectConfig, index: usize) -> String {
    match index {
        0 => config.name.clone(),
        1 => config.chip_type.clone(),
        2 => config.baud_rate.to_string(),
        3 => config.flash_mode.clone(),
        4 => config.flash_freq.clone(),
        5 => config.flash_size.clone(),
        6 => config.bootloader_offset.clone(),
        7 => config.bootloader_path.clone(),
        8 => config.partitions_offset.clone(),
        9 => config.partitions_path.clone(),
        10 => config.otadata_offset.clone(),
        11 => config.otadata_path.clone(),
        12 => config.app_offset.clone(),
        13 => config.app_path.clone(),
        _ => String::new(),
    }
}

fn set_config_field(config: &mut ProjectConfig, index: usize, value: String) {
    match index {
        0 => config.name = value,
        1 => config.chip_type = value,
        2 => {
            if let Ok(b) = value.parse::<u32>() {
                config.baud_rate = b;
            }
        }
        3 => config.flash_mode = value,
        4 => config.flash_freq = value,
        5 => config.flash_size = value,
        6 => config.bootloader_offset = value,
        7 => config.bootloader_path = value,
        8 => config.partitions_offset = value,
        9 => config.partitions_path = value,
        10 => config.otadata_offset = value,
        11 => config.otadata_path = value,
        12 => config.app_offset = value,
        13 => config.app_path = value,
        _ => {}
    }
}

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
    let mut edit_buffer = String::new();

    while !exit {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        tokio::select! {
            // Background worker messages
            Some(msg) = rx.recv() => {
                app.handle_worker_message(msg);
            }
            // Standard ticks
            _ = interval.tick() => {
                app.update_elapsed_time();
                
                // Scan ports automatically every 2 seconds if not flashing
                if !app.is_flashing && app.last_port_scan.elapsed() > Duration::from_secs(2) {
                    app.scan_ports();
                    app.last_port_scan = std::time::Instant::now();
                }
            }
            // User keyboard/terminal events
            maybe_event = reader.next() => {
                if let Some(Ok(Event::Key(key))) = maybe_event {
                    if key.kind == KeyEventKind::Press {
                        if app.is_entering_password {
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
                                    set_config_field(&mut app.config, app.selected_config_field, edit_buffer.clone());
                                    let _ = app.config.save_to_file(&app.config_path);
                                    app.is_editing_config = false;
                                    app.log(format!("Config field saved."));
                                }
                                KeyCode::Esc => {
                                    app.is_editing_config = false;
                                }
                                KeyCode::Backspace => {
                                    edit_buffer.pop();
                                    set_config_field(&mut app.config, app.selected_config_field, edit_buffer.clone());
                                }
                                KeyCode::Char(c) => {
                                    edit_buffer.push(c);
                                    set_config_field(&mut app.config, app.selected_config_field, edit_buffer.clone());
                                }
                                _ => {}
                            }
                        } else {
                            // General navigation/operation input handling
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => {
                                    if !app.is_flashing {
                                        exit = true;
                                    } else {
                                        app.log("Cannot exit while flashing is active!");
                                    }
                                }
                                KeyCode::Char('1') => app.active_tab = ActiveTab::Channels,
                                KeyCode::Char('2') => app.active_tab = ActiveTab::Logs,
                                KeyCode::Char('3') => app.active_tab = ActiveTab::Configuration,
                                KeyCode::Char('s') => {
                                    if !app.is_flashing {
                                        app.simulation_mode = !app.simulation_mode;
                                        app.log(format!("Simulation mode set to: {}", app.simulation_mode));
                                        app.scan_ports();
                                    }
                                }
                                KeyCode::Char('c') => {
                                    if !app.is_flashing {
                                        app.stats.total_passed = 0;
                                        app.stats.total_failed = 0;
                                        app.stats.total_attempted = 0;
                                        app.log("Production counters cleared.");
                                    }
                                }
                                KeyCode::Tab | KeyCode::F(1) => {
                                    if app.admin_mode {
                                        app.lock_admin();
                                    } else {
                                        app.is_entering_password = true;
                                    }
                                }
                                KeyCode::Char(' ') => {
                                    app.start_flashing(tx.clone());
                                }
                                KeyCode::Up => {
                                    if app.active_tab == ActiveTab::Configuration && app.admin_mode {
                                        if app.selected_config_field > 0 {
                                            app.selected_config_field -= 1;
                                        } else {
                                            app.selected_config_field = 13;
                                        }
                                    }
                                }
                                KeyCode::Down => {
                                    if app.active_tab == ActiveTab::Configuration && app.admin_mode {
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
                                        edit_buffer = get_config_field(&app.config, app.selected_config_field);
                                    }
                                }
                                _ => {}
                            }
                        }
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
