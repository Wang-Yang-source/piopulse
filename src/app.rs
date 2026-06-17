use crate::config::{PROJECT_CONFIG_FIELD_COUNT, ProjectConfig, ToolConfig};
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
    pub tool_settings_modal: Rect,
    pub serial_port_info: Rect,
    pub serial_options: Rect,
    pub serial_quick_commands: Rect,
    pub plotter_header: Rect,
    pub plotter_send_panel: Rect,
    pub plotter_port_selector: Rect,
    pub port_menu_modal: Rect,
    pub widget_add_modal: Rect,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Channel {
    pub port: String,
    pub chip: Option<String>,
    pub mac: Option<String>,
    pub serial_number: Option<String>,
    pub device_name: Option<String>,
    pub lot_code: String,
    pub firmware_version: String,
    pub verify_method: String,
    pub qa_result: String,
    pub trace_id: Option<String>,
    pub bytes_written: usize,
    pub security_state: String,
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
            serial_number: None,
            device_name: None,
            lot_code: "-".to_string(),
            firmware_version: "-".to_string(),
            verify_method: "-".to_string(),
            qa_result: "Pending".to_string(),
            trace_id: None,
            bytes_written: 0,
            security_state: "Unlocked".to_string(),
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
pub enum SerialDirection {
    Rx,
    Tx,
}

#[derive(Debug, Clone)]
pub struct SerialTimelineEntry {
    pub port: String,
    pub direction: SerialDirection,
    pub offset_ms: u128,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct SerialParseSummary {
    pub rx_frames: usize,
    pub tx_frames: usize,
    pub rx_bytes: usize,
    pub tx_bytes: usize,
    pub text_lines: usize,
    pub numeric_frames: usize,
    pub last_text: String,
    pub last_hex: String,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardEmptyAction {
    AddCatalog,
    Button,
    Slider,
    Dashboard,
    Image,
    Cube,
}

pub const PARAM_SLIDER_LABEL_WIDTH: u16 = 6;
pub const PARAM_SLIDER_TRACK_WIDTH: u16 = 10;
pub const PARAM_SLIDER_LAST_OFFSET: u16 = PARAM_SLIDER_TRACK_WIDTH - 1;

pub struct App {
    pub channels: Vec<Channel>,
    pub stats: Stats,
    pub logs: Vec<String>,
    pub config: ProjectConfig,
    pub config_path: String,
    pub tool_config: ToolConfig,
    pub show_tool_settings: bool,
    pub tool_settings_selected: usize,
    pub show_port_menu: bool,
    pub port_menu_selected: usize,

    // UI state
    pub active_tab: ActiveTab,
    pub selected_channel_idx: usize,
    pub selected_config_field: usize,
    pub is_editing_config: bool,
    pub edit_buffer: String,
    pub hover_tab: Option<usize>,
    pub hover_serial_port_info: Option<usize>,
    pub hover_serial_option: Option<usize>,
    pub hover_serial_quick_command: Option<usize>,
    pub hover_plotter_header_action: Option<usize>,
    pub hover_plotter_quick_command: Option<usize>,
    pub hover_dashboard_empty_action: Option<DashboardEmptyAction>,
    pub hover_widget_control: Option<usize>,

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
    pub plotter_active: bool,
    pub plotter_view_samples: usize,
    pub plotter_view_offset: usize,
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
    pub sim_pid_out: f64,
    pub sim_battery_voltage: f64,
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
    pub serial_recording: bool,
    pub serial_playback_active: bool,
    pub serial_playback_cursor: usize,
    pub serial_recording_started: Option<Instant>,
    pub serial_timeline: Vec<SerialTimelineEntry>,
    pub serial_parse_summary: SerialParseSummary,
    pub serial_replay_parser: crate::vofa::VofaParser,
    pub latest_image_width: usize,
    pub latest_image_height: usize,
    pub latest_image_data: Vec<u8>,
    pub show_sidebar: bool,
    pub serial_tx_senders:
        std::collections::HashMap<String, tokio::sync::mpsc::UnboundedSender<Vec<u8>>>,
    pub serial_monitor_baud_rates: std::collections::HashMap<String, u32>,
    pub serial_pending_monitors: std::collections::HashSet<String>,
    pub worker_tx: Option<tokio::sync::mpsc::Sender<crate::worker::WorkerMessage>>,
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

        let waveform_history = std::collections::HashMap::new();

        let tool_config = ToolConfig::load();
        let tool_settings_selected = if tool_config.language == "zh" { 1 } else { 0 };

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
            tool_config,
            show_tool_settings: false,
            tool_settings_selected,
            show_port_menu: false,
            port_menu_selected: 0,
            active_tab: ActiveTab::Serial,
            selected_channel_idx: 0,
            selected_config_field: 0,
            is_editing_config: false,
            edit_buffer: String::new(),
            hover_tab: None,
            hover_serial_port_info: None,
            hover_serial_option: None,
            hover_serial_quick_command: None,
            hover_plotter_header_action: None,
            hover_plotter_quick_command: None,
            hover_dashboard_empty_action: None,
            hover_widget_control: None,
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
            plotter_active: true,
            plotter_view_samples: 100,
            plotter_view_offset: 0,
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
            sim_pid_out: 0.0,
            sim_battery_voltage: 24.2,
            widget_focus: 0,
            dashboard_widgets: Vec::new(),
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
            serial_recording: false,
            serial_playback_active: false,
            serial_playback_cursor: 0,
            serial_recording_started: None,
            serial_timeline: Vec::new(),
            serial_parse_summary: SerialParseSummary::default(),
            serial_replay_parser: crate::vofa::VofaParser::new(crate::vofa::VofaMode::FireWater),
            latest_image_width: 0,
            latest_image_height: 0,
            latest_image_data: Vec::new(),
            show_sidebar: true,
            serial_tx_senders: std::collections::HashMap::new(),
            serial_monitor_baud_rates: std::collections::HashMap::new(),
            serial_pending_monitors: std::collections::HashSet::new(),
            worker_tx: None,
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
            self.channels = ports.into_iter().map(|p| Channel::new(p)).collect();
            self.log(format!(
                "Ports updated. Found {} active devices.",
                self.channels.len()
            ));
            if self.selected_channel_idx >= self.channels.len() {
                self.selected_channel_idx = 0;
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
            channel.serial_number = None;
            channel.device_name = None;
            channel.lot_code = self.config.lot_code.clone();
            channel.firmware_version = self.config.firmware_version.clone();
            channel.verify_method = self.config.verify_method.clone();
            channel.qa_result = "Pending".to_string();
            channel.trace_id = Some(make_trace_id(&channel.port, self.stats.total_attempted + 1));
            channel.bytes_written = 0;
            channel.security_state = if self.config.lock_after_flash
                || self.config.secure_boot
                || self.config.flash_encryption
            {
                "Lock Pending".to_string()
            } else {
                "Unlocked".to_string()
            };
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
        self.update_serial_monitoring();
        self.update_serial_playback();
    }

    pub fn update_serial_monitoring(&mut self) {
        if self.is_flashing {
            let keys: Vec<String> = self.serial_tx_senders.keys().cloned().collect();
            for port in keys {
                self.serial_tx_senders.remove(&port);
                self.serial_monitor_baud_rates.remove(&port);
            }
            self.serial_pending_monitors.clear();
            return;
        }

        let selected_port = self
            .get_selected_port()
            .unwrap_or_else(|| "NONE".to_string());

        // Stop monitors for non-selected ports
        let keys: Vec<String> = self.serial_tx_senders.keys().cloned().collect();
        for port in keys {
            let baud_changed = self
                .serial_monitor_baud_rates
                .get(&port)
                .is_some_and(|baud_rate| *baud_rate != self.serial_baud_rate);
            if port != selected_port || baud_changed {
                self.serial_tx_senders.remove(&port);
                self.serial_monitor_baud_rates.remove(&port);
                if baud_changed {
                    self.log(format!(
                        "Restarting Serial Monitor for {} at {} bps.",
                        port, self.serial_baud_rate
                    ));
                } else {
                    self.log(format!("Closed Serial Monitor for {}.", port));
                }
            }
        }

        // Start monitor for selected port
        if selected_port != "NONE" {
            if !self.serial_tx_senders.contains_key(&selected_port)
                && !self.serial_pending_monitors.contains(&selected_port)
            {
                if let Some(ref tx) = self.worker_tx {
                    self.serial_pending_monitors.insert(selected_port.clone());
                    worker::spawn_serial_monitor(
                        selected_port.clone(),
                        self.serial_baud_rate,
                        tx.clone(),
                    );
                }
            }
        }
    }

    pub fn get_selected_port(&self) -> Option<String> {
        if self.channels.is_empty() {
            None
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
            WorkerMessage::ProvisioningGenerated {
                port,
                serial_number,
                device_name,
            } => {
                if let Some(channel) = self.channels.iter_mut().find(|c| c.port == port) {
                    channel.serial_number = Some(serial_number);
                    channel.device_name = Some(device_name);
                }
            }
            WorkerMessage::ProductionStep { port, step, detail } => {
                if let Some(channel) = self.channels.iter_mut().find(|c| c.port == port) {
                    match step.as_str() {
                        "planned_bytes" => {
                            if let Ok(bytes) = detail.parse::<usize>() {
                                channel.bytes_written = bytes;
                            }
                        }
                        "qa" => channel.qa_result = detail,
                        "security" => channel.security_state = detail,
                        _ => {}
                    }
                }
            }
            WorkerMessage::Finished {
                port,
                success,
                error_msg,
                mac,
            } => {
                self.serial_tx_senders.remove(&port);
                self.serial_monitor_baud_rates.remove(&port);
                self.serial_pending_monitors.remove(&port);
                let mut log_msg = None;
                if let Some(channel) = self.channels.iter_mut().find(|c| c.port == port) {
                    channel.finished = true;
                    channel.success = success;
                    if success {
                        channel.status = "SUCCESS".to_string();
                        channel.progress = 100;
                        if channel.qa_result == "Pending" {
                            channel.qa_result = "PASS".to_string();
                        }
                        if channel.security_state == "Lock Pending" {
                            channel.security_state = "Locked".to_string();
                        }
                        self.stats.total_passed += 1;
                        log_msg = Some("Flashing PASSED!".to_string());
                    } else {
                        channel.status = "FAILED".to_string();
                        let err = error_msg.clone().unwrap_or_default();
                        channel.error = Some(err.clone());
                        channel.qa_result = "FAIL".to_string();
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
            WorkerMessage::SerialData { port, data } => {
                self.capture_serial_timeline_frame(SerialDirection::Rx, &port, &data);
                for line in format_serial_rx_messages(&data, self.serial_hex_mode_rx) {
                    self.channel_log(&port, line);
                }
            }
            WorkerMessage::WaveformData { port, values } => {
                if self.plotter_active {
                    let history = self.waveform_history.entry(port).or_insert_with(Vec::new);
                    history.push(values);
                    if history.len() > 100 {
                        history.remove(0);
                    }
                    self.plotter_view_offset =
                        self.plotter_view_offset.min(self.max_plotter_view_offset());
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
            WorkerMessage::MonitorStarted {
                port,
                baud_rate,
                sender,
            } => {
                self.serial_pending_monitors.remove(&port);
                self.serial_monitor_baud_rates
                    .insert(port.clone(), baud_rate);
                self.serial_tx_senders.insert(port, sender);
            }
            WorkerMessage::MonitorStopped { port } => {
                self.serial_pending_monitors.remove(&port);
                self.serial_monitor_baud_rates.remove(&port);
                self.serial_tx_senders.remove(&port);
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

    pub fn set_plotter_active(&mut self, active: bool) {
        self.plotter_active = active;
    }

    pub fn toggle_serial_recording(&mut self) {
        if self.serial_recording {
            self.serial_recording = false;
            self.serial_recording_started = None;
            self.log(format!(
                "Serial timeline recording stopped. Captured {} frames.",
                self.serial_timeline.len()
            ));
        } else {
            self.serial_playback_active = false;
            self.serial_playback_cursor = 0;
            self.serial_timeline.clear();
            self.serial_parse_summary = SerialParseSummary::default();
            self.serial_recording_started = Some(Instant::now());
            self.serial_recording = true;
            self.log("Serial timeline recording started.");
        }
    }

    pub fn start_serial_timeline_playback(&mut self) {
        if self.serial_timeline.is_empty() {
            self.log("Serial timeline playback skipped: no recorded frames.");
            return;
        }

        self.serial_recording = false;
        self.serial_recording_started = None;
        self.serial_playback_active = true;
        self.serial_playback_cursor = 0;
        self.serial_parse_summary = SerialParseSummary::default();
        self.serial_replay_parser = crate::vofa::VofaParser::new(self.vofa_mode);
        if let Some(port) = self.get_selected_port() {
            self.waveform_history.remove(&port);
        }
        self.log(format!(
            "Serial timeline playback started ({} frames).",
            self.serial_timeline.len()
        ));
    }

    pub fn stop_serial_timeline_playback(&mut self) {
        if self.serial_playback_active {
            self.serial_playback_active = false;
            self.log("Serial timeline playback stopped.");
        }
    }

    fn capture_serial_timeline_frame(
        &mut self,
        direction: SerialDirection,
        port: &str,
        data: &[u8],
    ) {
        update_serial_parse_summary(&mut self.serial_parse_summary, direction, data);

        if !self.serial_recording || data.is_empty() {
            return;
        }

        let offset_ms = self
            .serial_recording_started
            .map(|started| started.elapsed().as_millis())
            .unwrap_or_default();

        self.serial_timeline.push(SerialTimelineEntry {
            port: port.to_string(),
            direction,
            offset_ms,
            data: data.to_vec(),
        });

        if self.serial_timeline.len() > 2000 {
            self.serial_timeline.remove(0);
        }
    }

    fn update_serial_playback(&mut self) {
        if !self.serial_playback_active {
            return;
        }

        let Some(entry) = self
            .serial_timeline
            .get(self.serial_playback_cursor)
            .cloned()
        else {
            self.serial_playback_active = false;
            self.log("Serial timeline playback completed.");
            return;
        };

        self.serial_playback_cursor += 1;
        update_serial_parse_summary(&mut self.serial_parse_summary, entry.direction, &entry.data);

        match entry.direction {
            SerialDirection::Rx => {
                for line in format_serial_rx_messages(&entry.data, self.serial_hex_mode_rx) {
                    self.channel_log(&entry.port, format!("[REPLAY] {}", line));
                }

                self.serial_replay_parser.set_mode(self.vofa_mode);
                for frame in self.serial_replay_parser.feed(&entry.data) {
                    let history = self
                        .waveform_history
                        .entry(entry.port.clone())
                        .or_insert_with(Vec::new);
                    history.push(frame);
                    if history.len() > 100 {
                        history.remove(0);
                    }
                }
            }
            SerialDirection::Tx => {
                self.channel_log(
                    &entry.port,
                    format!("[REPLAY TX] {}", format_hex_bytes(&entry.data)),
                );
            }
        }
    }

    fn selected_waveform_len(&self) -> usize {
        self.get_selected_port()
            .as_ref()
            .and_then(|port| self.waveform_history.get(port))
            .map(Vec::len)
            .unwrap_or_default()
    }

    fn max_plotter_view_offset(&self) -> usize {
        let len = self.selected_waveform_len();
        let visible = self.plotter_view_samples.min(len);
        len.saturating_sub(visible)
    }

    pub fn zoom_plotter_view(&mut self, zoom_in: bool) {
        const MIN_SAMPLES: usize = 8;
        const MAX_SAMPLES: usize = 100;

        let current = self.plotter_view_samples.clamp(MIN_SAMPLES, MAX_SAMPLES);
        let next = if zoom_in {
            (current / 2).max(MIN_SAMPLES)
        } else {
            current.saturating_mul(2).min(MAX_SAMPLES)
        };

        if next != self.plotter_view_samples {
            self.plotter_view_samples = next;
            self.plotter_view_offset = self.plotter_view_offset.min(self.max_plotter_view_offset());
            self.log(format!("Plotter view window set to {} samples.", next));
        }
    }

    pub fn pan_plotter_view(&mut self, older: bool) {
        let step = (self.plotter_view_samples / 4).max(1);
        let max_offset = self.max_plotter_view_offset();
        let next = if older {
            self.plotter_view_offset
                .saturating_add(step)
                .min(max_offset)
        } else {
            self.plotter_view_offset.saturating_sub(step)
        };

        if next != self.plotter_view_offset {
            self.plotter_view_offset = next;
            if next == 0 {
                self.log("Plotter view returned to latest samples.");
            } else {
                self.log(format!("Plotter view offset set to {} samples.", next));
            }
        }
    }

    pub fn reset_plotter_view(&mut self) {
        self.plotter_view_samples = 100;
        self.plotter_view_offset = 0;
        self.log("Plotter view reset to auto-follow latest.");
    }

    pub fn cycle_serial_baud_rate(&mut self) {
        self.serial_baud_rate = next_serial_baud_rate(self.serial_baud_rate);
        if let Some(port) = self.get_selected_port() {
            self.serial_tx_senders.remove(&port);
            self.serial_monitor_baud_rates.remove(&port);
            self.serial_pending_monitors.remove(&port);
        }
        self.log(format!("Baud rate set to {} bps.", self.serial_baud_rate));
    }

    pub fn submit_serial_command(&mut self, cmd: &str) {
        let trimmed = cmd.trim();
        if trimmed.is_empty() {
            return;
        }

        let port = self
            .get_selected_port()
            .unwrap_or_else(|| "NONE".to_string());
        match encode_serial_tx(trimmed, self.serial_hex_mode_tx, self.serial_add_newline) {
            Ok(bytes) => {
                let tx_log = if self.serial_hex_mode_tx {
                    format!("[HEX] {}", format_hex_bytes(&bytes))
                } else {
                    trimmed.to_string()
                };
                self.log(format!("[{}] [TX] {}", port, tx_log));
                self.serial_send_history.push(trimmed.to_string());
                self.capture_serial_timeline_frame(SerialDirection::Tx, &port, &bytes);

                if let Some(sender) = self.serial_tx_senders.get(&port).cloned() {
                    if let Err(e) = sender.send(bytes) {
                        self.log(format!("Failed to send to serial port: {}", e));
                    }
                }
            }
            Err(e) => {
                self.log(format!("Invalid Hex TX input: {}", e));
            }
        }
    }

    pub fn cycle_plotter_mode(&mut self) {
        self.plotter_mode = match self.plotter_mode {
            PlotterMode::Waveform => PlotterMode::BarChart,
            PlotterMode::BarChart => PlotterMode::Histogram,
            PlotterMode::Histogram => PlotterMode::FftSpectrum,
            PlotterMode::FftSpectrum | PlotterMode::IMUCube | PlotterMode::RoiImage => {
                PlotterMode::Waveform
            }
        };
        self.log(format!("Plotter View Mode set to {:?}", self.plotter_mode));
    }

    pub fn cycle_vofa_mode(&mut self) {
        self.vofa_mode = match self.vofa_mode {
            crate::vofa::VofaMode::FireWater => crate::vofa::VofaMode::JustFloat,
            crate::vofa::VofaMode::JustFloat => crate::vofa::VofaMode::IndexFloat,
            crate::vofa::VofaMode::IndexFloat => crate::vofa::VofaMode::FireWater,
        };
        crate::vofa::ACTIVE_VOFA_MODE
            .store(self.vofa_mode.to_u8(), std::sync::atomic::Ordering::Relaxed);
        self.log(format!("VOFA+ Protocol Mode set to {:?}", self.vofa_mode));
    }

    pub fn plotter_header_action_at(&self, col: u16, row: u16) -> Option<usize> {
        let area = self.layout_zones.plotter_header;
        if row != area.y || !self.is_inside_rect(col, row, area) {
            return None;
        }

        use unicode_width::UnicodeWidthStr;
        let lang = &self.tool_config.language;
        let selected_port = self
            .get_selected_port()
            .unwrap_or_else(|| "NONE".to_string());
        let protocol = format!("{:?}", self.vofa_mode);
        let view = format!("{:?}", self.plotter_mode);
        let state = if self.plotter_active {
            if lang == "zh" { "运行中" } else { "RUNNING" }
        } else if lang == "zh" {
            "已暂停"
        } else {
            "PAUSED"
        };

        let relative_col = col.saturating_sub(area.x) as usize;
        let mut cursor = UnicodeWidthStr::width(crate::ui::tr("plot_title", lang)) + 2;
        let items = [
            (if lang == "zh" { "端口" } else { "Port" }, selected_port),
            (if lang == "zh" { "协议" } else { "Protocol" }, protocol),
            (if lang == "zh" { "视图" } else { "View" }, view),
            (
                if lang == "zh" { "状态" } else { "State" },
                state.to_string(),
            ),
        ];

        for (idx, (label, value)) in items.iter().enumerate() {
            let width = UnicodeWidthStr::width(format!(" {}: {} ", label, value).as_str());
            if (cursor..cursor + width).contains(&relative_col) {
                return Some(idx);
            }
            cursor += width + 2;
        }

        None
    }

    pub fn plotter_quick_command_at(&self, col: u16, row: u16) -> Option<usize> {
        let area = self.layout_zones.plotter_send_panel;
        if !self.is_inside_rect(col, row, area) || row != area.y + 2 {
            return None;
        }

        use unicode_width::UnicodeWidthStr;
        let lang = &self.tool_config.language;
        let relative_col = col.saturating_sub(area.x + 1) as usize;
        let mut cursor = UnicodeWidthStr::width(crate::ui::tr("plot_tx_quick", lang));

        for (idx, command) in plotter_quick_commands().iter().enumerate() {
            let width = UnicodeWidthStr::width(format!("[{}]", command).as_str());
            if (cursor..cursor + width).contains(&relative_col) {
                return Some(idx);
            }
            cursor += width + 1;
        }

        None
    }

    pub fn header_mode_action_at(&self, col: u16, row: u16) -> bool {
        let area = self.layout_zones.header;
        if row != area.y || !self.is_inside_rect(col, row, area) {
            return false;
        }

        use unicode_width::UnicodeWidthStr;
        let title = " ☕ PIOPULSE v0.1.3 ";
        let mode = if self.admin_mode {
            crate::ui::tr("admin_mode_header", &self.tool_config.language)
        } else {
            crate::ui::tr("operator_mode_header", &self.tool_config.language)
        };
        let relative_col = col.saturating_sub(area.x) as usize;
        let mode_start = UnicodeWidthStr::width(title) + UnicodeWidthStr::width(" | ");
        let mode_end = mode_start + UnicodeWidthStr::width(mode);

        (mode_start..mode_end).contains(&relative_col)
    }

    pub fn handle_mouse_click(
        &mut self,
        col: u16,
        row: u16,
        _tx: tokio::sync::mpsc::Sender<WorkerMessage>,
    ) -> bool {
        if self.show_port_menu {
            if !self.is_inside_rect(col, row, self.layout_zones.port_menu_modal) {
                self.show_port_menu = false;
            } else {
                let relative_row =
                    row.saturating_sub(self.layout_zones.port_menu_modal.y + 1) as usize;
                let total_items = self.channels.len();
                if relative_row < total_items {
                    self.selected_channel_idx = relative_row;
                    if let Some(port) = self.get_selected_port() {
                        self.log(format!("Selected port switched to {}.", port));
                    }
                    self.show_port_menu = false;
                }
            }
            return true;
        }

        if self.show_exit_menu {
            let area = self.layout_zones.exit_menu_modal;
            if !self.is_inside_rect(col, row, area) {
                self.show_exit_menu = false;
                return true;
            }

            let inner_x = area.x + 1;
            let inner_y = area.y + 1;
            let inner_w = area.width.saturating_sub(2);
            let cards_y = inner_y + 3;
            if row >= cards_y && row < cards_y + 3 {
                self.exit_menu_selected = if col < inner_x + inner_w / 2 { 0 } else { 1 };
            }

            if self.exit_menu_selected == 0 && row >= cards_y && row < cards_y + 3 {
                self.show_exit_menu = false;
                self.show_tool_settings = true;
                self.tool_settings_selected = if self.tool_config.language == "zh" {
                    1
                } else {
                    0
                };
            } else if self.exit_menu_selected == 1 && row >= cards_y && row < cards_y + 3 {
                if self.is_flashing {
                    self.show_exit_menu = false;
                    self.log("Cannot exit while flashing is active!");
                } else {
                    return false;
                }
            }
            return true;
        }

        if self.show_tool_settings {
            let area = self.layout_zones.tool_settings_modal;
            if !self.is_inside_rect(col, row, area) {
                self.show_tool_settings = false;
                return true;
            }

            let inner_x = area.x + 1;
            let inner_y = area.y + 1;
            let inner_w = area.width.saturating_sub(2);
            let cards_y = inner_y + 3;
            if row >= cards_y && row < cards_y + 3 {
                let selected = if col < inner_x + inner_w / 2 { 0 } else { 1 };
                self.tool_settings_selected = selected;
                let new_lang = if selected == 0 { "en" } else { "zh" };
                self.tool_config.language = new_lang.to_string();
                if let Err(e) = self.tool_config.save() {
                    self.log(format!("Failed to save tool config: {}", e));
                } else {
                    self.log("Tool configuration saved.");
                }
                self.show_tool_settings = false;
            }
            return true;
        }

        if self.active_tab == ActiveTab::Widgets && self.is_adding_widget {
            let area = self.layout_zones.widget_add_modal;
            if !self.is_inside_rect(col, row, area) {
                self.is_adding_widget = false;
                self.widget_search_input.clear();
                self.add_menu_selected = 0;
                return true;
            }

            let filtered_items = crate::ui::widgets::get_filtered_catalog_items(
                &self.widget_search_input,
                &self.tool_config.language,
            );
            let item_row = row.saturating_sub(area.y + 7) as usize;
            let visible_items = area.height.saturating_sub(8) as usize;
            if item_row < filtered_items.len().min(visible_items) {
                self.add_menu_selected = item_row;
                self.add_widget(filtered_items[item_row].2);
                self.is_adding_widget = false;
                self.widget_search_input.clear();
                self.add_menu_selected = 0;
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

        if self.is_inside_rect(col, row, self.layout_zones.header) {
            if self.header_mode_action_at(col, row) {
                if self.admin_mode {
                    self.lock_admin();
                } else {
                    self.is_entering_password = true;
                }
            }
            return true;
        }

        // Clicks inside the tabs bar
        if self.is_inside_rect(col, row, self.layout_zones.tabs) {
            if let Some(tab_idx) =
                tab_index_at(self.layout_zones.tabs, col, row, &self.tool_config.language)
            {
                self.active_tab = match tab_idx {
                    0 => ActiveTab::Serial,
                    1 => ActiveTab::Plotter,
                    2 => ActiveTab::Widgets,
                    3 => ActiveTab::Flasher,
                    _ => ActiveTab::Configuration,
                };
            }
            return true;
        }

        // Clicks inside the config table to select/edit field
        if clicked_config_table {
            let rect = self.layout_zones.config_table;
            let relative_row = row.saturating_sub(rect.y + 1) as usize;
            if relative_row < PROJECT_CONFIG_FIELD_COUNT {
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
            let port_count = self.channels.len();
            if relative_row < port_count as u16 {
                let idx = relative_row as usize;
                if idx < self.channels.len() {
                    self.selected_channel_idx = idx;
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
                    4 => self.toggle_serial_recording(),
                    5 => {
                        if self.serial_playback_active {
                            self.stop_serial_timeline_playback();
                        } else {
                            self.start_serial_timeline_playback();
                        }
                    }
                    _ => {}
                }
                return true;
            }

            if self.is_inside_rect(col, row, self.layout_zones.serial_port_info) {
                let click_row = row.saturating_sub(self.layout_zones.serial_port_info.y + 1);
                if click_row == 0 {
                    self.show_port_menu = true;
                    self.port_menu_selected = self.selected_channel_idx;
                } else if click_row == 1 {
                    self.cycle_serial_baud_rate();
                }
                return true;
            }

            if self.is_inside_rect(col, row, self.layout_zones.serial_quick_commands) {
                let command_row =
                    row.saturating_sub(self.layout_zones.serial_quick_commands.y + 2) as usize;
                if let Some(command) = serial_quick_commands().get(command_row) {
                    self.submit_serial_command(command);
                }
                return true;
            }
        }

        if self.active_tab == ActiveTab::Plotter {
            if self.is_inside_rect(col, row, self.layout_zones.plotter_header) {
                if let Some(action) = self.plotter_header_action_at(col, row) {
                    match action {
                        0 => {
                            self.show_port_menu = true;
                            self.port_menu_selected = self.selected_channel_idx;
                        }
                        1 => self.cycle_vofa_mode(),
                        2 => self.cycle_plotter_mode(),
                        _ => {
                            self.set_plotter_active(!self.plotter_active);
                            self.log(format!(
                                "Plotter active: {}",
                                if self.plotter_active { "ON" } else { "OFF" }
                            ));
                        }
                    }
                }
                return true;
            }

            if self.is_inside_rect(col, row, self.layout_zones.plotter_send_panel) {
                if let Some(idx) = self.plotter_quick_command_at(col, row) {
                    if let Some(command) = plotter_quick_commands().get(idx) {
                        self.submit_serial_command(command);
                    }
                }
                return true;
            }
        }

        if self.active_tab == ActiveTab::Widgets {
            if self.is_inside_rect(col, row, self.layout_zones.monitor_panel) {
                if self.dashboard_widgets.is_empty() {
                    if let Some(action) =
                        empty_dashboard_action_at(self.layout_zones.monitor_panel, col, row)
                    {
                        if let Some(widget) = widget_for_empty_action(action) {
                            self.add_widget(widget);
                        } else {
                            self.is_adding_widget = true;
                            self.widget_search_input.clear();
                            self.add_menu_selected = 0;
                        }
                    }
                    return true;
                }

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
                            WidgetType::Button => {
                                if row == inner_y + 1 {
                                    if col >= inner_x + 3 && col <= inner_x + 13 {
                                        self.submit_serial_command("START");
                                    } else if col >= inner_x + 16 && col <= inner_x + 25 {
                                        self.submit_serial_command("STOP");
                                    }
                                } else if row == inner_y + 3 {
                                    if col >= inner_x + 3 && col <= inner_x + 13 {
                                        self.submit_serial_command("RESET");
                                    } else if col >= inner_x + 16 && col <= inner_x + 25 {
                                        self.submit_serial_command("PING");
                                    }
                                }
                            }
                            WidgetType::Toggle => {
                                if row == inner_y + 1 {
                                    self.motor_enabled = !self.motor_enabled;
                                    self.log(format!(
                                        "Motor output: {}",
                                        if self.motor_enabled {
                                            "ENABLED"
                                        } else {
                                            "DISABLED"
                                        }
                                    ));
                                }
                            }
                            WidgetType::Delay => {
                                if row == inner_y + 1 {
                                    self.log("Delayed trigger activated.");
                                }
                            }
                            WidgetType::Dial => {
                                if row == inner_y + 3 || row == inner_y + 4 {
                                    let start_x = inner_x + 6;
                                    let end_x = inner_x + 27;
                                    let c_clamped = col.clamp(start_x, end_x);
                                    let pct = (c_clamped - start_x) as f64
                                        / (end_x - start_x).max(1) as f64;
                                    self.param_target_speed = (pct * 5000.0).clamp(0.0, 5000.0);
                                }
                            }
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
                                if row == inner_y + 1 {
                                    self.set_slider_param_from_col(0, inner_x, col);
                                } else if row == inner_y + 3 {
                                    self.set_slider_param_from_col(1, inner_x, col);
                                } else if row == inner_y + 5 {
                                    self.set_slider_param_from_col(2, inner_x, col);
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

    fn set_slider_param_from_col(&mut self, slider_idx: usize, inner_x: u16, col: u16) {
        let start_x = inner_x + PARAM_SLIDER_LABEL_WIDTH;
        let end_x = start_x + PARAM_SLIDER_LAST_OFFSET;
        let c_clamped = col.clamp(start_x, end_x);
        let pct = (c_clamped - start_x) as f64 / PARAM_SLIDER_LAST_OFFSET.max(1) as f64;

        match slider_idx {
            0 => self.param_kp = (pct * 3.0).clamp(0.0, 3.0),
            1 => self.param_ki = pct.clamp(0.0, 1.0),
            2 => self.param_kd = pct.clamp(0.0, 1.0),
            _ => {}
        }
    }

    pub fn handle_mouse_move(&mut self, col: u16, row: u16) {
        self.clear_hover_state();

        if self.show_port_menu {
            let area = self.layout_zones.port_menu_modal;
            if self.is_inside_rect(col, row, area) {
                let relative_row = row.saturating_sub(area.y + 1) as usize;
                if relative_row < self.channels.len() {
                    self.port_menu_selected = relative_row;
                }
            }
            return;
        }

        if self.show_exit_menu {
            let area = self.layout_zones.exit_menu_modal;
            if !self.is_inside_rect(col, row, area) {
                return;
            }

            let inner_x = area.x + 1;
            let inner_y = area.y + 1;
            let inner_w = area.width.saturating_sub(2);
            let cards_y = inner_y + 3;
            if row >= cards_y && row < cards_y + 3 {
                self.exit_menu_selected = if col < inner_x + inner_w / 2 { 0 } else { 1 };
            }
            return;
        }

        if self.show_tool_settings {
            let area = self.layout_zones.tool_settings_modal;
            if self.is_inside_rect(col, row, area) {
                let inner_x = area.x + 1;
                let inner_y = area.y + 1;
                let inner_w = area.width.saturating_sub(2);
                let cards_y = inner_y + 3;
                if row >= cards_y && row < cards_y + 3 {
                    self.tool_settings_selected = if col < inner_x + inner_w / 2 { 0 } else { 1 };
                }
            }
            return;
        }

        if self.active_tab == ActiveTab::Widgets && self.is_adding_widget {
            let area = self.layout_zones.widget_add_modal;
            if self.is_inside_rect(col, row, area) {
                let filtered_items = crate::ui::widgets::get_filtered_catalog_items(
                    &self.widget_search_input,
                    &self.tool_config.language,
                );
                let item_row = row.saturating_sub(area.y + 7) as usize;
                let visible_items = area.height.saturating_sub(8) as usize;
                if item_row < filtered_items.len().min(visible_items) {
                    self.add_menu_selected = item_row;
                }
            }
            return;
        }

        if let Some(tab_idx) =
            tab_index_at(self.layout_zones.tabs, col, row, &self.tool_config.language)
        {
            self.hover_tab = Some(tab_idx);
            return;
        }

        match self.active_tab {
            ActiveTab::Serial => {
                if self.is_inside_rect(col, row, self.layout_zones.serial_port_info) {
                    let idx = row.saturating_sub(self.layout_zones.serial_port_info.y + 1) as usize;
                    if idx <= 1 {
                        self.hover_serial_port_info = Some(idx);
                    }
                    return;
                }

                if self.is_inside_rect(col, row, self.layout_zones.serial_options) {
                    let idx = row.saturating_sub(self.layout_zones.serial_options.y + 1) as usize;
                    if idx < 6 {
                        self.hover_serial_option = Some(idx);
                    }
                    return;
                }

                if self.is_inside_rect(col, row, self.layout_zones.serial_quick_commands) {
                    let idx =
                        row.saturating_sub(self.layout_zones.serial_quick_commands.y + 2) as usize;
                    if idx < serial_quick_commands().len() {
                        self.hover_serial_quick_command = Some(idx);
                    }
                }
            }
            ActiveTab::Plotter => {
                if self.is_inside_rect(col, row, self.layout_zones.plotter_header) {
                    self.hover_plotter_header_action = self.plotter_header_action_at(col, row);
                    return;
                }

                if self.is_inside_rect(col, row, self.layout_zones.plotter_send_panel) {
                    self.hover_plotter_quick_command = self.plotter_quick_command_at(col, row);
                }
            }
            ActiveTab::Widgets => {
                if !self.is_inside_rect(col, row, self.layout_zones.monitor_panel) {
                    return;
                }

                if self.dashboard_widgets.is_empty() {
                    self.hover_dashboard_empty_action = empty_dashboard_action_at(
                        self.layout_zones.monitor_panel,
                        col,
                        row,
                    );
                    return;
                }

                let pane_layouts = crate::ui::widgets::get_pane_layouts(
                    self.layout_zones.monitor_panel,
                    self.dashboard_widgets.len(),
                );
                for (idx, &pane) in pane_layouts.iter().enumerate() {
                    if self.is_inside_rect(col, row, pane) {
                        self.selected_widget_idx = idx;
                        self.hover_widget_control =
                            widget_control_at(self.dashboard_widgets[idx], pane, col, row);
                        return;
                    }
                }
            }
            _ => {}
        }
    }

    fn clear_hover_state(&mut self) {
        self.hover_tab = None;
        self.hover_serial_port_info = None;
        self.hover_serial_option = None;
        self.hover_serial_quick_command = None;
        self.hover_plotter_header_action = None;
        self.hover_plotter_quick_command = None;
        self.hover_dashboard_empty_action = None;
        self.hover_widget_control = None;
    }

    pub fn handle_mouse_scroll(&mut self, up: bool) {
        match self.active_tab {
            ActiveTab::Plotter => {
                self.zoom_plotter_view(up);
            }
            ActiveTab::Configuration => {
                if self.admin_mode && !self.is_editing_config {
                    if up {
                        if self.selected_config_field > 0 {
                            self.selected_config_field -= 1;
                        } else {
                            self.selected_config_field = PROJECT_CONFIG_FIELD_COUNT - 1;
                        }
                    } else {
                        if self.selected_config_field < PROJECT_CONFIG_FIELD_COUNT - 1 {
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

fn make_trace_id(port: &str, attempt: u32) -> String {
    let sanitized_port: String = port
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect();
    format!("TRACE-{}-{:06}", sanitized_port, attempt)
}

pub fn next_serial_baud_rate(current: u32) -> u32 {
    match current {
        9600 => 115200,
        115200 => 921600,
        921600 => 1152000,
        _ => 9600,
    }
}

pub fn serial_quick_commands() -> &'static [&'static str] {
    &[
        "AT",
        "AT+GMR",
        "ATI",
        "ATE1",
        "ATE0",
        "AT+CSQ",
        "AT+CIFSR",
        "AT+CWMODE?",
        "AT+UART_CUR?",
        "ATZ",
        "RESET",
        "help",
    ]
}

pub fn plotter_quick_commands() -> &'static [&'static str] {
    &["RESET", "VERSION?", "START", "STOP", "QA PING"]
}

fn empty_dashboard_action_at(
    monitor_panel: Rect,
    col: u16,
    row: u16,
) -> Option<DashboardEmptyAction> {
    let relative_row = row.saturating_sub(monitor_panel.y + 1);
    let compact = monitor_panel.height < 16 || monitor_panel.width < 82;
    match relative_row {
        3 => {
            let rel_x = col.saturating_sub(monitor_panel.x + 3);
            match rel_x {
                0..=15 => Some(DashboardEmptyAction::AddCatalog),
                16..=27 => Some(DashboardEmptyAction::Button),
                28..=39 => Some(DashboardEmptyAction::Slider),
                40..=57 => Some(DashboardEmptyAction::Dashboard),
                58..=70 => Some(DashboardEmptyAction::Image),
                _ => None,
            }
        }
        _ if compact => None,
        7 => Some(DashboardEmptyAction::Button),
        8 => Some(DashboardEmptyAction::Slider),
        9 => Some(DashboardEmptyAction::Dashboard),
        10 => Some(DashboardEmptyAction::Image),
        11 => Some(DashboardEmptyAction::Cube),
        _ => None,
    }
}

fn widget_for_empty_action(action: DashboardEmptyAction) -> Option<WidgetType> {
    match action {
        DashboardEmptyAction::AddCatalog => None,
        DashboardEmptyAction::Button => Some(WidgetType::Button),
        DashboardEmptyAction::Slider => Some(WidgetType::Slider),
        DashboardEmptyAction::Dashboard => Some(WidgetType::Dashboard),
        DashboardEmptyAction::Image => Some(WidgetType::Image),
        DashboardEmptyAction::Cube => Some(WidgetType::Cube),
    }
}

fn widget_control_at(widget: WidgetType, pane: Rect, col: u16, row: u16) -> Option<usize> {
    let inner_y = pane.y + 1;
    let inner_x = pane.x + 1;

    match widget {
        WidgetType::Button => {
            if row == inner_y + 1 {
                if col >= inner_x + 3 && col <= inner_x + 13 {
                    Some(0)
                } else if col >= inner_x + 16 && col <= inner_x + 25 {
                    Some(1)
                } else {
                    None
                }
            } else if row == inner_y + 3 {
                if col >= inner_x + 3 && col <= inner_x + 13 {
                    Some(2)
                } else if col >= inner_x + 16 && col <= inner_x + 25 {
                    Some(3)
                } else {
                    None
                }
            } else {
                None
            }
        }
        WidgetType::Toggle | WidgetType::Delay => {
            if row == inner_y + 1 {
                Some(0)
            } else {
                None
            }
        }
        WidgetType::Dial => {
            if row == inner_y + 3 || row == inner_y + 4 {
                Some(0)
            } else {
                None
            }
        }
        WidgetType::Knob => {
            if row == inner_y + 4 {
                Some(0)
            } else {
                None
            }
        }
        WidgetType::Slider => match row {
            r if r == inner_y + 1 => Some(0),
            r if r == inner_y + 3 => Some(1),
            r if r == inner_y + 5 => Some(2),
            _ => None,
        },
        _ => None,
    }
}

pub fn tab_index_at(tabs_area: Rect, col: u16, row: u16, lang: &str) -> Option<usize> {
    if row < tabs_area.y || row >= tabs_area.y + tabs_area.height {
        return None;
    }

    let relative_col = col.saturating_sub(tabs_area.x) as usize;
    use unicode_width::UnicodeWidthStr;
    let titles = [
        crate::ui::tr("tab_serial", lang),
        crate::ui::tr("tab_plot", lang),
        crate::ui::tr("tab_dash", lang),
        crate::ui::tr("tab_flash", lang),
        crate::ui::tr("tab_settings", lang),
    ];

    let mut current_x = 0;
    for (idx, title) in titles.iter().enumerate() {
        let width = UnicodeWidthStr::width(*title);
        if (current_x..=current_x + width).contains(&relative_col) {
            return Some(idx);
        }
        current_x += width + 3;
    }

    None
}

pub fn encode_serial_tx(input: &str, hex_mode: bool, add_newline: bool) -> Result<Vec<u8>, String> {
    if hex_mode {
        return parse_hex_bytes(input);
    }

    let mut bytes = input.as_bytes().to_vec();
    if add_newline {
        bytes.push(b'\n');
    }
    Ok(bytes)
}

pub fn format_hex_bytes(data: &[u8]) -> String {
    data.iter()
        .map(|byte| format!("{:02X}", byte))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn format_serial_rx_messages(data: &[u8], hex_mode: bool) -> Vec<String> {
    if data.is_empty() {
        return Vec::new();
    }

    if hex_mode {
        return vec![format!("[HEX] {}", format_hex_bytes(data))];
    }

    let Ok(text) = std::str::from_utf8(data) else {
        return Vec::new();
    };

    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn update_serial_parse_summary(
    summary: &mut SerialParseSummary,
    direction: SerialDirection,
    data: &[u8],
) {
    if data.is_empty() {
        return;
    }

    match direction {
        SerialDirection::Rx => {
            summary.rx_frames += 1;
            summary.rx_bytes += data.len();
        }
        SerialDirection::Tx => {
            summary.tx_frames += 1;
            summary.tx_bytes += data.len();
        }
    }

    summary.last_hex = format_hex_bytes(data);

    if let Ok(text) = std::str::from_utf8(data) {
        let mut last_line = None;
        for line in text.lines().map(str::trim).filter(|line| !line.is_empty()) {
            summary.text_lines += 1;
            last_line = Some(line.to_string());

            let numeric_count = line
                .split(|ch: char| ch == ',' || ch == ';' || ch.is_ascii_whitespace())
                .filter(|part| !part.is_empty())
                .filter(|part| part.parse::<f32>().is_ok())
                .count();
            if numeric_count > 0 {
                summary.numeric_frames += 1;
            }
        }
        if let Some(line) = last_line {
            summary.last_text = line;
        }
    }
}

pub fn parse_hex_bytes(input: &str) -> Result<Vec<u8>, String> {
    let mut hex = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch.is_ascii_hexdigit() {
            hex.push(ch);
        } else if ch == 'x' || ch == 'X' {
            if !hex.ends_with('0') {
                return Err(format!("unexpected '{}' in hex input", ch));
            }
            hex.pop();
        } else if ch.is_ascii_whitespace() || ch == ',' || ch == '_' || ch == '-' {
            continue;
        } else {
            return Err(format!("invalid hex character '{}'", ch));
        }

        if chars.peek().is_none() && hex.len() % 2 != 0 {
            return Err("hex input must contain an even number of digits".to_string());
        }
    }

    if hex.is_empty() {
        return Err("hex input is empty".to_string());
    }

    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let s = std::str::from_utf8(pair).map_err(|e| e.to_string())?;
            u8::from_str_radix(s, 16).map_err(|e| e.to_string())
        })
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_menu_toggle_and_select() {
        // Use a temporary path or mock path
        let mut app = App::new("test_project_config.json".to_string());

        // Initially false
        assert!(!app.show_port_menu);
        assert_eq!(app.port_menu_selected, 0);

        // Configure layout zones for test
        app.layout_zones.serial_port_info = ratatui::layout::Rect::new(10, 5, 20, 3);
        app.layout_zones.port_menu_modal = ratatui::layout::Rect::new(30, 10, 40, 6);

        // Create a mock channel
        app.channels = vec![Channel::new(crate::worker::DetectedPort {
            name: "COM3".to_string(),
            vid: None,
            pid: None,
            product: None,
            manufacturer: None,
        })];

        let (tx, _rx) = tokio::sync::mpsc::channel(1);

        // Click on row 0 of serial_port_info (y + 1 = 6)
        let handled = app.handle_mouse_click(12, 6, tx.clone());
        assert!(handled);
        assert!(app.show_port_menu);
        assert_eq!(app.port_menu_selected, app.selected_channel_idx);

        // Click outside port_menu_modal should close it
        let handled_outside = app.handle_mouse_click(0, 0, tx.clone());
        assert!(handled_outside);
        assert!(!app.show_port_menu);

        // Open it again
        let handled = app.handle_mouse_click(12, 6, tx.clone());
        assert!(handled);
        assert!(app.show_port_menu);

        // Click inside port_menu_modal on the first item ("COM3", which is index 0, y + 1 = 11)
        let handled_inside = app.handle_mouse_click(32, 11, tx.clone());
        assert!(handled_inside);
        assert!(!app.show_port_menu);
        assert_eq!(app.selected_channel_idx, 0);
        assert_eq!(app.get_selected_port().unwrap(), "COM3");

        // Clean up mock file if created
        let _ = std::fs::remove_file("test_project_config.json");
    }

    #[test]
    fn test_exit_menu_buttons_are_mouse_clickable() {
        let mut app = App::new("test_project_config.json".to_string());
        let (tx, _rx) = tokio::sync::mpsc::channel(1);

        app.show_exit_menu = true;
        app.layout_zones.exit_menu_modal = ratatui::layout::Rect::new(10, 5, 48, 11);

        let settings_click = app.handle_mouse_click(16, 9, tx.clone());
        assert!(settings_click);
        assert!(!app.show_exit_menu);
        assert!(app.show_tool_settings);

        app.show_tool_settings = false;
        app.show_exit_menu = true;
        let quit_click = app.handle_mouse_click(44, 9, tx.clone());
        assert!(!quit_click);

        let _ = std::fs::remove_file("test_project_config.json");
    }

    #[test]
    fn test_exit_menu_buttons_select_on_mouse_hover() {
        let mut app = App::new("test_project_config.json".to_string());
        app.show_exit_menu = true;
        app.layout_zones.exit_menu_modal = ratatui::layout::Rect::new(10, 5, 48, 11);

        app.exit_menu_selected = 1;
        app.handle_mouse_move(16, 9);
        assert_eq!(app.exit_menu_selected, 0);

        app.handle_mouse_move(44, 9);
        assert_eq!(app.exit_menu_selected, 1);

        let _ = std::fs::remove_file("test_project_config.json");
    }

    #[test]
    fn test_serial_buttons_select_on_mouse_hover() {
        let mut app = App::new("test_project_config.json".to_string());
        app.active_tab = ActiveTab::Serial;
        app.layout_zones.serial_port_info = ratatui::layout::Rect::new(10, 5, 20, 7);
        app.layout_zones.serial_options = ratatui::layout::Rect::new(30, 5, 20, 8);
        app.layout_zones.serial_quick_commands = ratatui::layout::Rect::new(50, 5, 30, 16);

        app.handle_mouse_move(12, 6);
        assert_eq!(app.hover_serial_port_info, Some(0));

        app.handle_mouse_move(32, 8);
        assert_eq!(app.hover_serial_option, Some(2));

        app.handle_mouse_move(52, 10);
        assert_eq!(app.hover_serial_quick_command, Some(3));

        let _ = std::fs::remove_file("test_project_config.json");
    }

    #[test]
    fn test_dashboard_empty_buttons_select_on_mouse_hover() {
        let mut app = App::new("test_project_config.json".to_string());
        app.active_tab = ActiveTab::Widgets;
        app.layout_zones.monitor_panel = ratatui::layout::Rect::new(10, 5, 80, 20);

        app.handle_mouse_move(30, 9);
        assert_eq!(
            app.hover_dashboard_empty_action,
            Some(DashboardEmptyAction::Button)
        );

        app.handle_mouse_move(12, 14);
        assert_eq!(
            app.hover_dashboard_empty_action,
            Some(DashboardEmptyAction::Slider)
        );

        let _ = std::fs::remove_file("test_project_config.json");
    }

    #[test]
    fn test_slider_click_uses_visual_track_columns() {
        let mut app = App::new("test_project_config.json".to_string());
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        app.active_tab = ActiveTab::Widgets;
        app.layout_zones.monitor_panel = ratatui::layout::Rect::new(10, 5, 80, 20);
        app.dashboard_widgets = vec![WidgetType::Slider];

        let pane = crate::ui::widgets::get_pane_layouts(app.layout_zones.monitor_panel, 1)[0];
        let inner_x = pane.x + 1;
        let inner_y = pane.y + 1;
        let track_start = inner_x + PARAM_SLIDER_LABEL_WIDTH;

        assert!(app.handle_mouse_click(track_start, inner_y + 1, tx.clone()));
        assert_eq!(app.param_kp, 0.0);

        assert!(app.handle_mouse_click(track_start + 5, inner_y + 1, tx.clone()));
        assert!((app.param_kp - (5.0 / 9.0 * 3.0)).abs() < f64::EPSILON);

        assert!(app.handle_mouse_click(track_start + PARAM_SLIDER_LAST_OFFSET, inner_y + 1, tx));
        assert_eq!(app.param_kp, 3.0);

        let _ = std::fs::remove_file("test_project_config.json");
    }

    #[test]
    fn test_top_tabs_select_on_mouse_hover() {
        let mut app = App::new("test_project_config.json".to_string());
        app.layout_zones.tabs = ratatui::layout::Rect::new(0, 3, 80, 3);

        app.handle_mouse_move(0, 3);
        assert_eq!(app.hover_tab, Some(0));

        let tab_plot_x = crate::ui::tr("tab_serial", &app.tool_config.language).len() + 3;
        app.handle_mouse_move(tab_plot_x as u16, 3);
        assert_eq!(app.hover_tab, Some(1));

        let _ = std::fs::remove_file("test_project_config.json");
    }

    #[test]
    fn test_plotter_header_clicks_use_real_pill_bounds() {
        use unicode_width::UnicodeWidthStr;

        let mut app = App::new("test_project_config.json".to_string());
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        app.active_tab = ActiveTab::Plotter;
        app.layout_zones.plotter_header = ratatui::layout::Rect::new(0, 5, 120, 4);

        let lang = &app.tool_config.language;
        let mut cursor = UnicodeWidthStr::width(crate::ui::tr("plot_title", lang)) + 2;
        cursor += UnicodeWidthStr::width(" Port: NONE ") + 2;
        cursor += UnicodeWidthStr::width(" Protocol: FireWater ") + 2;

        let protocol_before = app.vofa_mode;
        let view_before = app.plotter_mode;
        let view_x = cursor + 1;
        assert!(app.handle_mouse_click(view_x as u16, 5, tx.clone()));
        assert_eq!(app.vofa_mode, protocol_before);
        assert_ne!(app.plotter_mode, view_before);

        let view_width = UnicodeWidthStr::width(" View: BarChart ");
        let state_x = cursor + view_width + 2 + 1;
        let view_after = app.plotter_mode;
        assert!(app.handle_mouse_click(state_x as u16, 5, tx.clone()));
        assert_eq!(app.plotter_mode, view_after);
        assert!(!app.plotter_active);

        let _ = std::fs::remove_file("test_project_config.json");
    }

    #[test]
    fn test_plotter_quick_commands_use_real_button_bounds() {
        use unicode_width::UnicodeWidthStr;

        let mut app = App::new("test_project_config.json".to_string());
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        app.active_tab = ActiveTab::Plotter;
        app.layout_zones.plotter_send_panel = ratatui::layout::Rect::new(0, 10, 100, 5);

        let lang = &app.tool_config.language;
        let mut cursor = UnicodeWidthStr::width(crate::ui::tr("plot_tx_quick", lang));
        cursor += UnicodeWidthStr::width("[RESET]") + 1;
        let version_width = UnicodeWidthStr::width("[VERSION?]");

        let gap_after_version_x = cursor + version_width;
        assert!(app.handle_mouse_click(gap_after_version_x as u16 + 1, 12, tx.clone()));
        assert!(app.serial_send_history.is_empty());

        let start_x = gap_after_version_x + 1;
        assert!(app.handle_mouse_click(start_x as u16 + 1, 12, tx.clone()));
        assert_eq!(
            app.serial_send_history.last().map(String::as_str),
            Some("START")
        );

        let _ = std::fs::remove_file("test_project_config.json");
    }

    #[test]
    fn test_header_mode_click_opens_sudo_prompt() {
        use unicode_width::UnicodeWidthStr;

        let mut app = App::new("test_project_config.json".to_string());
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        app.layout_zones.header = ratatui::layout::Rect::new(0, 0, 80, 2);

        let mode_x =
            UnicodeWidthStr::width(" ☕ PIOPULSE v0.1.3 ") + UnicodeWidthStr::width(" | ") + 1;
        assert!(app.handle_mouse_click(mode_x as u16, 0, tx));
        assert!(app.is_entering_password);

        let _ = std::fs::remove_file("test_project_config.json");
    }

    #[test]
    fn test_plotter_view_zoom_pan_and_reset() {
        let mut app = App::new("test_project_config.json".to_string());
        app.channels.push(Channel::new(crate::worker::DetectedPort {
            name: "TEST".to_string(),
            vid: None,
            pid: None,
            product: None,
            manufacturer: None,
        }));
        app.waveform_history.insert(
            "TEST".to_string(),
            (0..100).map(|idx| vec![idx as f32]).collect(),
        );

        app.zoom_plotter_view(true);
        assert_eq!(app.plotter_view_samples, 50);

        app.pan_plotter_view(true);
        assert!(app.plotter_view_offset > 0);

        app.pan_plotter_view(false);
        assert_eq!(app.plotter_view_offset, 0);

        app.reset_plotter_view();
        assert_eq!(app.plotter_view_samples, 100);
        assert_eq!(app.plotter_view_offset, 0);

        let _ = std::fs::remove_file("test_project_config.json");
    }

    #[test]
    fn test_serial_timeline_recording_and_playback_rebuilds_waveform() {
        let mut app = App::new("test_project_config.json".to_string());
        app.channels.push(Channel::new(crate::worker::DetectedPort {
            name: "TEST".to_string(),
            vid: None,
            pid: None,
            product: None,
            manufacturer: None,
        }));

        app.toggle_serial_recording();
        app.handle_worker_message(WorkerMessage::SerialData {
            port: "TEST".to_string(),
            data: b"1.0,2.0\n".to_vec(),
        });
        app.submit_serial_command("AT");
        app.toggle_serial_recording();

        assert_eq!(app.serial_timeline.len(), 2);
        assert!(app.serial_parse_summary.numeric_frames > 0);

        app.waveform_history.remove("TEST");
        app.start_serial_timeline_playback();
        while app.serial_playback_active {
            app.update_serial_playback();
        }

        assert_eq!(
            app.waveform_history
                .get("TEST")
                .and_then(|frames| frames.last())
                .cloned(),
            Some(vec![1.0, 2.0])
        );

        let _ = std::fs::remove_file("test_project_config.json");
    }

    #[test]
    fn test_serial_tx_encoding_modes() {
        assert_eq!(
            encode_serial_tx("AT", false, true).unwrap(),
            b"AT\n".to_vec()
        );
        assert_eq!(
            encode_serial_tx("0x41 42,43", true, true).unwrap(),
            b"ABC".to_vec()
        );
        assert!(encode_serial_tx("0x4", true, false).is_err());
        assert!(encode_serial_tx("GG", true, false).is_err());
    }

    #[test]
    fn test_serial_rx_format_modes() {
        assert_eq!(
            format_serial_rx_messages(b"hello\n\nworld\r\n", false),
            vec!["hello".to_string(), "world".to_string()]
        );
        assert!(format_serial_rx_messages(&[0xFF, 0x00], false).is_empty());
        assert_eq!(
            format_serial_rx_messages(&[0x0A, 0xFF], true),
            vec!["[HEX] 0A FF".to_string()]
        );
    }
}
