use crate::config::ProjectConfig;
use crate::worker::{self, WorkerMessage};
use std::sync::Arc;
use std::time::{Duration, Instant};

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
}

impl Channel {
    pub fn new(port: String) -> Self {
        Self {
            port,
            chip: None,
            mac: None,
            status: "Idle".to_string(),
            progress: 0,
            speed: "N/A".to_string(),
            error: None,
            finished: false,
            success: false,
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
    
    // Operations
    pub admin_mode: bool,
    pub password_input: String,
    pub is_entering_password: bool,
    pub password_incorrect: bool,
    
    pub is_flashing: bool,
    pub simulation_mode: bool,
    pub start_time: Option<Instant>,
    pub elapsed_time: Duration,
    
    pub last_port_scan: Instant,
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
            admin_mode: false,
            password_input: String::new(),
            is_entering_password: false,
            password_incorrect: false,
            is_flashing: false,
            simulation_mode: true, // Default to simulation mode for easy testing
            start_time: None,
            elapsed_time: Duration::from_secs(0),
            last_port_scan: Instant::now() - Duration::from_secs(10), // force scan on startup
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

        let available = worker::get_available_serial_ports();
        
        // If simulation mode is active and we have no physical ports, create mock ones
        let ports = if self.simulation_mode && available.is_empty() {
            vec!["COM3".to_string(), "COM4".to_string(), "COM5".to_string(), "COM6".to_string()]
        } else {
            available
        };

        // Re-synchronize channels vector
        let old_ports: Vec<String> = self.channels.iter().map(|c| c.port.clone()).collect();
        if ports != old_ports {
            self.channels = ports.iter().map(|p| Channel::new(p.clone())).collect();
            self.log(format!("Ports updated. Found {} active devices.", ports.len()));
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
            "--- Start Batch Flashing to {} devices (Simulation={}) ---",
            self.channels.len(),
            self.simulation_mode
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
                self.simulation_mode,
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
}
