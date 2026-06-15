use crate::config::ProjectConfig;
use crate::worker::{self, WorkerMessage};
use std::sync::Arc;
use std::time::{Duration, Instant};
use ratatui::layout::Rect;

#[derive(Debug, Clone, Default)]
pub struct LayoutZones {
    pub header: Rect,
    pub tabs: Rect,
    pub config_table: Rect,
    pub help_sidebar: Rect,
    pub password_modal: Rect,
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub port: String,
    pub chip: Option<String>,
    pub mac: Option<String>,
    pub status: String,
    pub progress: u8,
    pub speed: String,
    pub error: Option<String>,
    pub finished: bool,
    pub success: bool,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
    pub usb_product: Option<String>,
    pub usb_manufacturer: Option<String>,
}

impl Channel {
    pub fn new(port: worker::DetectedPort) -> Self {
        Self {
            port: port.name,
            chip: None,
            mac: None,
            status: "Idle".to_string(),
            progress: 0,
            speed: "N/A".to_string(),
            error: None,
            finished: false,
            success: false,
            vid: port.vid,
            pid: port.pid,
            usb_product: port.product,
            usb_manufacturer: port.manufacturer,
        }
    }
}

pub struct Stats {
    pub total_passed: u32,
    pub total_failed: u32,
    pub total_attempted: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Channels,
    Logs,
    Configuration,
}

pub struct App {
    pub channels: Vec<Channel>,
    pub stats: Stats,
    pub logs: Vec<String>,
    pub config: ProjectConfig,
    pub config_path: String,
    
    // UI state
    pub active_tab: ActiveTab,
    pub selected_channel_idx: usize,
    pub selected_config_field: usize,
    pub is_editing_config: bool,
    pub edit_buffer: String,
    pub logs_scroll_offset: usize,
    
    // Operations
    pub admin_mode: bool,
    pub password_input: String,
    pub is_entering_password: bool,
    pub password_incorrect: bool,
    
    pub is_flashing: bool,
    pub start_time: Option<Instant>,
    pub elapsed_time: Duration,
    
    pub last_port_scan: Instant,
    pub layout_zones: LayoutZones,
}

impl App {
    pub fn new(config_path: String) -> Self {
        let config = ProjectConfig::load_from_file(&config_path)
            .unwrap_or_else(|_| {
                let default_cfg = ProjectConfig::default();
                let _ = default_cfg.save_to_file(&config_path);
                default_cfg
            });

        let mut app = Self {
            channels: Vec::new(),
            stats: Stats {
                total_passed: 0,
                total_failed: 0,
                total_attempted: 0,
            },
            logs: Vec::new(),
            config,
            config_path,
            active_tab: ActiveTab::Channels,
            selected_channel_idx: 0,
            selected_config_field: 0,
            is_editing_config: false,
            edit_buffer: String::new(),
            logs_scroll_offset: 0,
            admin_mode: false,
            password_input: String::new(),
            is_entering_password: false,
            password_incorrect: false,
            is_flashing: false,
            start_time: None,
            elapsed_time: Duration::from_secs(0),
            last_port_scan: Instant::now() - Duration::from_secs(10), // force scan on startup
            layout_zones: LayoutZones::default(),
        };

        app.log("System Initialized. Press F1/Tab to unlock Admin Mode. Press SPACE to Flash.");
        app
    }

    pub fn log(&mut self, msg: impl Into<String>) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        self.logs.push(format!("[{}] {}", timestamp, msg.into()));
        // Keep logs size reasonable
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    pub fn channel_log(&mut self, port: &str, msg: impl Into<String>) {
        self.log(format!("[{}] {}", port, msg.into()));
    }

    pub fn scan_ports(&mut self) {
        if self.is_flashing {
            return;
        }

        let ports = worker::get_available_serial_ports();

        // Check if the ports list has changed
        let has_changed = if ports.len() != self.channels.len() {
            true
        } else {
            ports.iter().zip(self.channels.iter()).any(|(p, c)| p.name != c.port)
        };

        if has_changed {
            self.channels = ports.into_iter().map(|p| Channel::new(p)).collect();
            self.log(format!("Ports updated. Found {} active devices.", self.channels.len()));
            if self.selected_channel_idx >= self.channels.len() && !self.channels.is_empty() {
                self.selected_channel_idx = self.channels.len() - 1;
            }
        }
    }

    pub fn start_flashing(&mut self, tx: tokio::sync::mpsc::Sender<WorkerMessage>) {
        if self.is_flashing || self.channels.is_empty() {
            return;
        }

        self.is_flashing = true;
        self.start_time = Some(Instant::now());
        self.elapsed_time = Duration::from_secs(0);
        self.log(format!(
            "--- Start Batch Flashing to {} devices ---",
            self.channels.len()
        ));

        let config_arc = Arc::new(self.config.clone());

        for channel in &mut self.channels {
            channel.status = "Queued...".to_string();
            channel.progress = 0;
            channel.chip = None;
            channel.mac = None;
            channel.error = None;
            channel.finished = false;
            channel.success = false;

            self.stats.total_attempted += 1;

            // Spawn the task
            worker::start_flashing_task(
                channel.port.clone(),
                config_arc.clone(),
                tx.clone(),
            );
        }
    }

    pub fn update_elapsed_time(&mut self) {
        if self.is_flashing {
            if let Some(start) = self.start_time {
                self.elapsed_time = start.elapsed();
            }
        }
    }

    pub fn handle_worker_message(&mut self, msg: WorkerMessage) {
        match msg {
            WorkerMessage::StatusUpdate { port, status, progress, speed } => {
                if let Some(channel) = self.channels.iter_mut().find(|c| c.port == port) {
                    channel.status = status;
                    channel.progress = progress;
                    channel.speed = speed;
                }
            }
            WorkerMessage::MacAddressDetected { port, mac, chip } => {
                if let Some(channel) = self.channels.iter_mut().find(|c| c.port == port) {
                    channel.mac = Some(mac);
                    channel.chip = Some(chip);
                }
            }
            WorkerMessage::Finished { port, success, error_msg, mac } => {
                let mut log_msg = None;
                if let Some(channel) = self.channels.iter_mut().find(|c| c.port == port) {
                    channel.finished = true;
                    channel.success = success;
                    if success {
                        channel.status = "SUCCESS".to_string();
                        channel.progress = 100;
                        self.stats.total_passed += 1;
                        log_msg = Some("Flashing PASSED!".to_string());
                    } else {
                        channel.status = "FAILED".to_string();
                        let err = error_msg.clone().unwrap_or_default();
                        channel.error = Some(err.clone());
                        self.stats.total_failed += 1;
                        log_msg = Some(format!("Flashing FAILED: {}", err));
                    }
                    if let Some(m) = mac {
                        channel.mac = Some(m);
                    }
                }

                if let Some(msg) = log_msg {
                    self.channel_log(&port, msg);
                }

                // Check if all channels have finished
                let all_finished = self.channels.iter().all(|c| c.finished);
                if all_finished {
                    self.is_flashing = false;
                    let total_secs = self.elapsed_time.as_secs_f32();
                    self.log(format!(
                        "--- Batch Flashing Completed in {:.2}s. Passed: {}, Failed: {} ---",
                        total_secs, self.stats.total_passed, self.stats.total_failed
                    ));
                }
            }
            WorkerMessage::Log { port, message } => {
                self.channel_log(&port, message);
            }
        }
    }

    pub fn unlock_admin(&mut self) {
        if self.password_input == "admin" {
            self.admin_mode = true;
            self.is_entering_password = false;
            self.password_incorrect = false;
            self.password_input.clear();
            self.log("Admin Mode unlocked.");
        } else {
            self.password_incorrect = true;
            self.password_input.clear();
        }
    }

    pub fn lock_admin(&mut self) {
        self.admin_mode = false;
        self.is_editing_config = false;
        self.log("Admin Mode locked. Switched to Operator Mode.");
    }

    pub fn handle_mouse_click(&mut self, col: u16, row: u16, tx: tokio::sync::mpsc::Sender<WorkerMessage>) -> bool {
        // 1. Password Modal Check
        if self.is_entering_password {
            if !self.is_inside_rect(col, row, self.layout_zones.password_modal) {
                // Clicked outside modal -> cancel password entry
                self.is_entering_password = false;
                self.password_input.clear();
                self.password_incorrect = false;
                self.log("Admin login cancelled.");
            }
            return true; // Consume click
        }

        // 2. Config Editing Check
        let clicked_config_table = self.active_tab == ActiveTab::Configuration 
            && self.admin_mode 
            && self.is_inside_rect(col, row, self.layout_zones.config_table);
            
        if self.is_editing_config && !clicked_config_table {
            // Save current field
            self.config.set_field(self.selected_config_field, self.edit_buffer.clone());
            let _ = self.config.save_to_file(&self.config_path);
            self.is_editing_config = false;
            self.log("Saved configuration.");
        }

        // Clicks inside the tabs bar
        if self.is_inside_rect(col, row, self.layout_zones.tabs) {
            let rect = self.layout_zones.tabs;
            if rect.width > 0 {
                let tab_width = rect.width / 3;
                if tab_width > 0 {
                    let clicked_tab = (col - rect.x) / tab_width;
                    match clicked_tab {
                        0 => self.active_tab = ActiveTab::Channels,
                        1 => self.active_tab = ActiveTab::Logs,
                        _ => self.active_tab = ActiveTab::Configuration,
                    }
                    return true;
                }
            }
        }

        // Clicks inside the help sidebar buttons
        if self.is_inside_rect(col, row, self.layout_zones.help_sidebar) {
            let rect = self.layout_zones.help_sidebar;
            // Clicks start inside borders (y + 1)
            let relative_row = row.saturating_sub(rect.y + 1);
            match relative_row {
                0 => {
                    // Space - Trigger Flashing
                    self.start_flashing(tx);
                }
                1 => {
                    // Tab - Toggle Admin
                    if self.admin_mode {
                        self.lock_admin();
                    } else {
                        self.is_entering_password = true;
                    }
                }
                2 => {
                    // c - Clear Statistics
                    if !self.is_flashing {
                        self.stats.total_passed = 0;
                        self.stats.total_failed = 0;
                        self.stats.total_attempted = 0;
                        self.log("Production counters cleared.");
                    }
                }
                3 => {
                    // 1/2/3 - Switch active tab
                    self.active_tab = match self.active_tab {
                        ActiveTab::Channels => ActiveTab::Logs,
                        ActiveTab::Logs => ActiveTab::Configuration,
                        ActiveTab::Configuration => ActiveTab::Channels,
                    };
                }
                4 => {
                    // Esc - Exit Application
                    if !self.is_flashing {
                        return false; // Exit signal
                    } else {
                        self.log("Cannot exit while flashing is active!");
                    }
                }
                _ => {}
            }
            return true;
        }

        // Clicks inside the config table to select/edit field
        if clicked_config_table {
            let rect = self.layout_zones.config_table;
            let relative_row = row.saturating_sub(rect.y + 1) as usize;
            if relative_row < 14 { // We have 14 config fields total
                if self.is_editing_config {
                    if self.selected_config_field != relative_row {
                        // Save current field
                        self.config.set_field(self.selected_config_field, self.edit_buffer.clone());
                        let _ = self.config.save_to_file(&self.config_path);
                        self.log("Saved configuration.");
                        // Start editing new field
                        self.selected_config_field = relative_row;
                        self.edit_buffer = self.config.get_field(relative_row);
                    }
                } else {
                    if self.selected_config_field == relative_row {
                        // Clicked already selected field -> Start editing!
                        self.is_editing_config = true;
                        self.edit_buffer = self.config.get_field(relative_row);
                    } else {
                        self.selected_config_field = relative_row;
                    }
                }
                return true;
            }
        }

        true // Continue running
    }

    pub fn handle_mouse_scroll(&mut self, up: bool) {
        match self.active_tab {
            ActiveTab::Logs => {
                if up {
                    if self.logs_scroll_offset < self.logs.len().saturating_sub(5) {
                        self.logs_scroll_offset += 2;
                    }
                    if self.logs_scroll_offset >= self.logs.len() {
                        self.logs_scroll_offset = self.logs.len().saturating_sub(1);
                    }
                } else {
                    self.logs_scroll_offset = self.logs_scroll_offset.saturating_sub(2);
                }
            }
            ActiveTab::Configuration => {
                if self.admin_mode && !self.is_editing_config {
                    if up {
                        if self.selected_config_field > 0 {
                            self.selected_config_field -= 1;
                        } else {
                            self.selected_config_field = 13;
                        }
                    } else {
                        if self.selected_config_field < 13 {
                            self.selected_config_field += 1;
                        } else {
                            self.selected_config_field = 0;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn is_inside_rect(&self, col: u16, row: u16, rect: Rect) -> bool {
        col >= rect.x && col < (rect.x + rect.width) && row >= rect.y && row < (rect.y + rect.height)
    }
}
