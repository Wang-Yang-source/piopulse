use crate::config::ProjectConfig;
use crate::worker::{self, WorkerMessage};
use ratatui::layout::Rect;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default)]
pub struct LayoutZones {
    pub header: Rect,
    pub tabs: Rect,
    pub config_table: Rect,
    pub monitor_panel: Rect,
    pub password_modal: Rect,
    pub exit_menu_modal: Rect,
    pub serial_port_info: Rect,
    pub serial_options: Rect,
    pub plotter_port_selector: Rect,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
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
    Serial,
    Plotter,
    Widgets,
    Flasher,
    Configuration,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlotterMode {
    Waveform,
    BarChart,
    Histogram,
    FftSpectrum,
    IMUCube,
    RoiImage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetType {
    Cube,
    Image,
    Button,
    Slider,
    Dial,
    Joystick,
    Light,
    Gauge,
    Dashboard,
    Example,
    Delay,
    Toggle,
    Knob,
    Ring,
    Pad,
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

    // Operations
    pub admin_mode: bool,
    pub password_input: String,
    pub is_entering_password: bool,
    pub password_incorrect: bool,
    pub show_exit_menu: bool,
    pub exit_menu_selected: usize,

    pub is_flashing: bool,
    pub start_time: Option<Instant>,
    pub elapsed_time: Duration,

    pub last_port_scan: Instant,
    pub layout_zones: LayoutZones,
    pub waveform_history: std::collections::HashMap<String, Vec<Vec<f32>>>,
    pub simulation_active: bool,
    pub vofa_mode: crate::vofa::VofaMode,
    pub plotter_mode: PlotterMode,
    pub manual_imu_override: bool,
    pub manual_pitch: f64,
    pub manual_roll: f64,
    pub manual_yaw: f64,
    pub manual_tx: f64,
    pub manual_ty: f64,
    pub manual_tz: f64,

    // Robot Dashboard State (PID simulation and Widget focuses)
    pub param_kp: f64,
    pub param_ki: f64,
    pub param_kd: f64,
    pub param_knob: f64,
    pub param_target_speed: f64,
    pub motor_enabled: bool,
    pub sim_motor_speed: f64,
    pub sim_pid_integral: f64,
    pub sim_pid_prev_error: f64,
    pub sim_pid_out: f64,
    pub sim_battery_voltage: f64,
    pub pid_history: Vec<(f32, f32, f32)>,
    pub widget_focus: usize,

    // Zellij-style Dashboard State
    pub dashboard_widgets: Vec<WidgetType>,
    pub selected_widget_idx: usize,
    pub is_adding_widget: bool,
    pub add_menu_selected: usize,
    pub widget_search_input: String,

    // Serial Terminal Debugger State (Tab 1)
    pub serial_send_buffer: String,
    pub serial_is_typing: bool,
    pub serial_hex_mode_rx: bool,
    pub serial_hex_mode_tx: bool,
    pub serial_auto_scroll: bool,
    pub serial_add_newline: bool,
    pub serial_baud_rate: u32,
    pub serial_send_history: Vec<String>,
    pub latest_image_width: usize,
    pub latest_image_height: usize,
    pub latest_image_data: Vec<u8>,
    pub show_sidebar: bool,
}

impl App {
    pub fn new(config_path: String) -> Self {
        let mut pio_detected = false;
        let config = if let Some(pio_cfg) = ProjectConfig::detect_platformio_config() {
            pio_detected = true;
            pio_cfg
        } else {
            ProjectConfig::load_from_file(&config_path).unwrap_or_else(|_| {
                let default_cfg = ProjectConfig::default();
                let _ = default_cfg.save_to_file(&config_path);
                default_cfg
            })
        };

        let mut waveform_history = std::collections::HashMap::new();
        waveform_history.insert(
            "SIMULATED".to_string(),
            vec![vec![0.0, 2.5, 0.0, 0.0, 0.35, 0.0]],
        );

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
            active_tab: ActiveTab::Serial,
            selected_channel_idx: 0,
            selected_config_field: 0,
            is_editing_config: false,
            edit_buffer: String::new(),
            admin_mode: false,
            password_input: String::new(),
            is_entering_password: false,
            password_incorrect: false,
            show_exit_menu: false,
            exit_menu_selected: 0,
            is_flashing: false,
            start_time: None,
            elapsed_time: Duration::from_secs(0),
            last_port_scan: Instant::now() - Duration::from_secs(10), // force scan on startup
            layout_zones: LayoutZones::default(),
            waveform_history,
            simulation_active: true,
            vofa_mode: crate::vofa::VofaMode::FireWater,
            plotter_mode: PlotterMode::Waveform,
            manual_imu_override: false,
            manual_pitch: 0.0,
            manual_roll: 0.0,
            manual_yaw: 0.0,
            manual_tx: 0.0,
            manual_ty: 0.0,
            manual_tz: 0.0,
            param_kp: 1.5,
            param_ki: 0.1,
            param_kd: 0.15,
            param_knob: 0.75,
            param_target_speed: 1500.0,
            motor_enabled: false,
            sim_motor_speed: 0.0,
            sim_pid_integral: 0.0,
            sim_pid_prev_error: 0.0,
            sim_pid_out: 0.0,
            sim_battery_voltage: 24.2,
            pid_history: Vec::new(),
            widget_focus: 0,
            dashboard_widgets: vec![WidgetType::Cube],
            selected_widget_idx: 0,
            is_adding_widget: false,
            add_menu_selected: 0,
            widget_search_input: String::new(),

            // Serial Terminal Debugger state
            serial_send_buffer: String::new(),
            serial_is_typing: false,
            serial_hex_mode_rx: false,
            serial_hex_mode_tx: false,
            serial_auto_scroll: true,
            serial_add_newline: true,
            serial_baud_rate: 115200,
            serial_send_history: Vec::new(),
            latest_image_width: 0,
            latest_image_height: 0,
            latest_image_data: Vec::new(),
            show_sidebar: true,
        };

        crate::vofa::ACTIVE_VOFA_MODE
            .store(app.vofa_mode.to_u8(), std::sync::atomic::Ordering::Relaxed);

        app.log("System Initialized. Press F1 to unlock Admin Mode. Press SPACE to Flash.");
        if pio_detected {
            app.log(format!(
                "PlatformIO project detected! Auto-configured environments: {}",
                app.config.name
            ));
        }
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
            ports
                .iter()
                .zip(self.channels.iter())
                .any(|(p, c)| p.name != c.port)
        };

        if has_changed {
            let was_simulated =
                self.selected_channel_idx >= self.channels.len() || self.channels.is_empty();
            self.channels = ports.into_iter().map(|p| Channel::new(p)).collect();
            self.log(format!(
                "Ports updated. Found {} active devices.",
                self.channels.len()
            ));
            if was_simulated {
                self.selected_channel_idx = self.channels.len();
            } else if self.selected_channel_idx >= self.channels.len() {
                self.selected_channel_idx = self.channels.len();
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
            worker::start_flashing_task(channel.port.clone(), config_arc.clone(), tx.clone());
        }
    }

    pub fn update_elapsed_time(&mut self) {
        if self.is_flashing {
            if let Some(start) = self.start_time {
                self.elapsed_time = start.elapsed();
            }
        }
    }

    pub fn tick(&mut self) {
        self.update_elapsed_time();

        if self.simulation_active {
            let elapsed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f32();

            let frame = vec![
                (elapsed * 1.2).sin() * 3.0,
                (elapsed * 1.8).cos() * 2.5,
                (elapsed * 0.8).sin() * 4.0,
                (elapsed * 2.2).sin() * 0.45,
                (elapsed * 1.6).cos() * 0.35,
                (elapsed * 1.1).sin() * 0.25,
            ];
            let history = self
                .waveform_history
                .entry("SIMULATED".to_string())
                .or_insert_with(Vec::new);
            history.push(frame);
            if history.len() > 100 {
                history.remove(0);
            }
        }

        // PID Simulation Tick
        if self.motor_enabled {
            let error = self.param_target_speed - self.sim_motor_speed;
            let p_out = self.param_kp * error;
            self.sim_pid_integral = (self.sim_pid_integral + error * 0.1).clamp(-2000.0, 2000.0);
            let i_out = self.param_ki * self.sim_pid_integral;
            let d_out = self.param_kd * (error - self.sim_pid_prev_error) / 0.1;
            self.sim_pid_prev_error = error;

            let out = p_out + i_out + d_out;
            self.sim_pid_out = out;
            self.sim_motor_speed = (self.sim_motor_speed + out * 0.05).clamp(-5000.0, 5000.0);
        } else {
            self.sim_motor_speed *= 0.85;
            if self.sim_motor_speed.abs() < 1.0 {
                self.sim_motor_speed = 0.0;
            }
            self.sim_pid_integral = 0.0;
            self.sim_pid_prev_error = 0.0;
            self.sim_pid_out = 0.0;
        }

        let battery_load = if self.motor_enabled {
            (self.sim_motor_speed.abs() / 5000.0) * 1.5
        } else {
            0.0
        };
        let elapsed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as f64;
        let noise = (elapsed * 0.05).sin() * 0.05;
        self.sim_battery_voltage = 24.2 - battery_load + noise;

        self.pid_history.push((
            self.param_target_speed as f32,
            self.sim_motor_speed as f32,
            self.sim_pid_out as f32,
        ));
        if self.pid_history.len() > 100 {
            self.pid_history.remove(0);
        }
    }

    pub fn get_selected_port(&self) -> Option<String> {
        let sim_idx = self.channels.len();
        if self.selected_channel_idx == sim_idx || self.channels.is_empty() {
            Some("SIMULATED".to_string())
        } else {
            let idx = self.selected_channel_idx.min(self.channels.len() - 1);
            Some(self.channels[idx].port.clone())
        }
    }

    pub fn handle_worker_message(&mut self, msg: WorkerMessage) {
        match msg {
            WorkerMessage::StatusUpdate {
                port,
                status,
                progress,
                speed,
            } => {
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
            WorkerMessage::Finished {
                port,
                success,
                error_msg,
                mac,
            } => {
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
            WorkerMessage::WaveformData { port, values } => {
                if self.simulation_active {
                    let history = self.waveform_history.entry(port).or_insert_with(Vec::new);
                    history.push(values);
                    if history.len() > 100 {
                        history.remove(0);
                    }
                }
            }
            WorkerMessage::ImageData {
                port: _,
                id: _,
                width,
                height,
                format: _,
                data,
            } => {
                self.latest_image_width = width;
                self.latest_image_height = height;
                self.latest_image_data = data;
            }
        }
    }

    #[allow(dead_code)]
    pub fn add_widget(&mut self, widget: WidgetType) {
        if self.dashboard_widgets.len() < 6 {
            self.dashboard_widgets.push(widget);
            self.selected_widget_idx = self.dashboard_widgets.len() - 1;
            self.log(format!("Added {:?} widget to Page 4.", widget));
        } else {
            self.log("Maximum of 6 widgets allowed in workspace.");
        }
    }

    #[allow(dead_code)]
    pub fn delete_selected_widget(&mut self) {
        if !self.dashboard_widgets.is_empty() {
            let removed = self.dashboard_widgets.remove(self.selected_widget_idx);
            self.log(format!("Removed {:?} widget from Page 4.", removed));
            if self.selected_widget_idx >= self.dashboard_widgets.len()
                && !self.dashboard_widgets.is_empty()
            {
                self.selected_widget_idx = self.dashboard_widgets.len() - 1;
            }
        }
    }

    pub fn unlock_admin(&mut self) {
        if verify_sudo_password(&self.password_input) {
            self.admin_mode = true;
            self.is_entering_password = false;
            self.password_incorrect = false;
            self.password_input.clear();
            self.log("Admin Mode unlocked via sudo verification.");
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

    pub fn set_simulation_active(&mut self, active: bool) {
        self.simulation_active = active;
        if !active {
            self.waveform_history.remove("SIMULATED");
        }
    }

    pub fn handle_mouse_click(
        &mut self,
        col: u16,
        row: u16,
        _tx: tokio::sync::mpsc::Sender<WorkerMessage>,
    ) -> bool {
        if self.show_exit_menu {
            if !self.is_inside_rect(col, row, self.layout_zones.exit_menu_modal) {
                self.show_exit_menu = false;
                return true;
            }

            let relative_row = row.saturating_sub(self.layout_zones.exit_menu_modal.y + 1);
            if (3..6).contains(&relative_row) {
                self.active_tab = ActiveTab::Configuration;
                self.show_exit_menu = false;
            } else if (6..9).contains(&relative_row) {
                if self.is_flashing {
                    self.show_exit_menu = false;
                    self.log("Cannot exit while flashing is active!");
                } else {
                    return false;
                }
            }
            return true;
        }

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
            && self.is_inside_rect(col, row, self.layout_zones.config_table);

        if self.is_editing_config && !clicked_config_table {
            // Save current field
            if self.admin_mode {
                self.config
                    .set_field(self.selected_config_field, self.edit_buffer.clone());
                let _ = self.config.save_to_file(&self.config_path);
                self.log("Saved configuration.");
            }
            self.is_editing_config = false;
        }

        // Clicks inside the tabs bar
        if self.is_inside_rect(col, row, self.layout_zones.tabs) {
            let relative_col = col.saturating_sub(self.layout_zones.tabs.x);
            if relative_col < 22 {
                self.active_tab = ActiveTab::Serial;
            } else if relative_col < 45 {
                self.active_tab = ActiveTab::Plotter;
            } else if relative_col < 67 {
                self.active_tab = ActiveTab::Widgets;
            } else if relative_col < 87 {
                self.active_tab = ActiveTab::Flasher;
            } else {
                self.active_tab = ActiveTab::Configuration;
            }
            return true;
        }

        // Clicks inside the config table to select/edit field
        if clicked_config_table {
            let rect = self.layout_zones.config_table;
            let relative_row = row.saturating_sub(rect.y + 1) as usize;
            if relative_row < 14 {
                // We have 14 config fields total
                if self.admin_mode {
                    if self.is_editing_config {
                        if self.selected_config_field != relative_row {
                            // Save current field
                            self.config
                                .set_field(self.selected_config_field, self.edit_buffer.clone());
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
                } else {
                    self.selected_config_field = relative_row;
                }
                return true;
            }
        }

        if self.active_tab == ActiveTab::Plotter
            && self.is_inside_rect(col, row, self.layout_zones.plotter_port_selector)
        {
            let relative_row = row.saturating_sub(self.layout_zones.plotter_port_selector.y + 1);
            let port_count = self.channels.len() + 1;
            if relative_row < port_count as u16 {
                let idx = relative_row as usize;
                if idx < self.channels.len() {
                    self.selected_channel_idx = idx;
                } else {
                    self.selected_channel_idx = self.channels.len();
                    self.set_simulation_active(!self.simulation_active);
                    self.log(format!(
                        "Simulated waveform source: {}",
                        if self.simulation_active { "ON" } else { "OFF" }
                    ));
                }
            }
            return true;
        }

        // 3. Serial Settings & Toggles Check
        if self.active_tab == ActiveTab::Serial {
            if self.is_inside_rect(col, row, self.layout_zones.serial_options) {
                let click_row = row.saturating_sub(self.layout_zones.serial_options.y + 1);
                match click_row {
                    0 => {
                        self.serial_auto_scroll = !self.serial_auto_scroll;
                        self.log(format!(
                            "Auto Scroll: {}",
                            if self.serial_auto_scroll {
                                "ENABLED"
                            } else {
                                "DISABLED"
                            }
                        ));
                    }
                    1 => {
                        self.serial_add_newline = !self.serial_add_newline;
                        self.log(format!(
                            "Send Newline: {}",
                            if self.serial_add_newline {
                                "ENABLED"
                            } else {
                                "DISABLED"
                            }
                        ));
                    }
                    2 => {
                        self.serial_hex_mode_rx = !self.serial_hex_mode_rx;
                        self.log(format!(
                            "Hex RX Mode: {}",
                            if self.serial_hex_mode_rx {
                                "ENABLED"
                            } else {
                                "DISABLED"
                            }
                        ));
                    }
                    3 => {
                        self.serial_hex_mode_tx = !self.serial_hex_mode_tx;
                        self.log(format!(
                            "Hex TX Mode: {}",
                            if self.serial_hex_mode_tx {
                                "ENABLED"
                            } else {
                                "DISABLED"
                            }
                        ));
                    }
                    _ => {}
                }
                return true;
            }

            if self.is_inside_rect(col, row, self.layout_zones.serial_port_info) {
                let click_row = row.saturating_sub(self.layout_zones.serial_port_info.y + 1);
                if click_row == 1 {
                    self.serial_baud_rate = match self.serial_baud_rate {
                        9600 => 115200,
                        115200 => 921600,
                        921600 => 1152000,
                        _ => 9600,
                    };
                    self.log(format!("Baud rate set to {} bps.", self.serial_baud_rate));
                }
                return true;
            }
        }

        if self.active_tab == ActiveTab::Widgets {
            if self.is_inside_rect(col, row, self.layout_zones.monitor_panel) {
                let pane_layouts = crate::ui::widgets::get_pane_layouts(
                    self.layout_zones.monitor_panel,
                    self.dashboard_widgets.len(),
                );
                for (idx, &pane) in pane_layouts.iter().enumerate() {
                    if self.is_inside_rect(col, row, pane) {
                        self.selected_widget_idx = idx;

                        let inner_y = pane.y + 1;
                        let inner_x = pane.x + 1;

                        match self.dashboard_widgets[idx] {
                            WidgetType::Knob => {
                                if row == inner_y + 4 {
                                    let start_x = inner_x + 8;
                                    let end_x = inner_x + 18;
                                    let c_clamped = col.clamp(start_x, end_x);
                                    let pct = (c_clamped - start_x) as f64 / 10.0;
                                    self.param_knob = pct.clamp(0.0, 1.0);
                                }
                            }
                            WidgetType::Slider => {
                                let start_x = inner_x + 6;
                                let end_x = inner_x + 15;
                                if row == inner_y + 1 {
                                    let c_clamped = col.clamp(start_x, end_x);
                                    let pct = (c_clamped - start_x) as f64 / 9.0;
                                    self.param_kp = (pct * 3.0).clamp(0.0, 3.0);
                                } else if row == inner_y + 3 {
                                    let c_clamped = col.clamp(start_x, end_x);
                                    let pct = (c_clamped - start_x) as f64 / 9.0;
                                    self.param_ki = pct.clamp(0.0, 1.0);
                                } else if row == inner_y + 5 {
                                    let c_clamped = col.clamp(start_x, end_x);
                                    let pct = (c_clamped - start_x) as f64 / 9.0;
                                    self.param_kd = pct.clamp(0.0, 1.0);
                                }
                            }
                            _ => {}
                        }
                        return true;
                    }
                }
            }
        }

        true // Continue running
    }

    pub fn handle_mouse_scroll(&mut self, up: bool) {
        match self.active_tab {
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
            ActiveTab::Widgets => {
                if self.dashboard_widgets.get(self.selected_widget_idx) == Some(&WidgetType::Cube) {
                    if up {
                        self.widget_focus = if self.widget_focus > 0 {
                            self.widget_focus - 1
                        } else {
                            7
                        };
                    } else {
                        self.widget_focus = (self.widget_focus + 1) % 8;
                    }
                }
            }
            _ => {}
        }
    }

    fn is_inside_rect(&self, col: u16, row: u16, rect: Rect) -> bool {
        col >= rect.x
            && col < (rect.x + rect.width)
            && row >= rect.y
            && row < (rect.y + rect.height)
    }
}

fn verify_sudo_password(password: &str) -> bool {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = match Command::new("sudo")
        .args(&["-S", "-v"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    if let Some(mut stdin) = child.stdin.take() {
        if stdin
            .write_all(format!("{}\n", password).as_bytes())
            .is_err()
        {
            return false;
        }
    }

    match child.wait() {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}
