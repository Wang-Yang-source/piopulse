use crate::config::{
    FirmwareImage, PROJECT_CONFIG_FIELD_COUNT, ProjectConfig, ToolConfig, create_merged_flash_image,
};
use crate::worker::{self, WorkerMessage};
use ratatui::layout::Rect;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub const SPLASH_TICKS: usize = 4;

#[derive(Debug, Clone, Copy)]
pub enum SoundEffect {
    #[allow(dead_code)]
    Boot,
    Success,
    Failure,
    Connect,
    Disconnect,
}

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
    pub flash_summary: Rect,
    pub flash_device_table: Rect,
    pub flash_empty_state: Rect,
    pub flash_mode_toggle: Rect,
    pub flash_donotchg_toggle: Rect,
    pub flash_manifest_status: Rect,
    pub custom_baud_modal: Rect,
    pub auto_reply_modal: Rect,
    pub flash_manifest_table: Rect,
    pub manifest_delete_modal: Rect,
    pub manifest_edit_modal: Rect,
    pub file_picker_modal: Rect,
    pub file_picker_table: Rect,
}

#[derive(Clone, Debug)]
pub struct FilePickerItem {
    pub name: String,
    pub path: std::path::PathBuf,
    pub is_dir: bool,
    pub size_str: String,
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
    pub auto_flash_armed: bool,
    pub auto_probe_pending: bool,
    pub auto_last_probe_at: Instant,
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
            auto_flash_armed: true,
            auto_probe_pending: false,
            auto_last_probe_at: Instant::now() - Duration::from_secs(10),
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
pub enum SerialNoticeKind {
    Info,
    Success,
    Warning,
}

#[derive(Debug, Clone)]
pub struct SerialNotice {
    pub message: String,
    pub kind: SerialNoticeKind,
    pub started_at: Instant,
    pub expires_at: Instant,
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
    pub log_file: Option<File>,
    pub config: ProjectConfig,
    pub config_path: String,
    pub tool_config: ToolConfig,
    pub show_tool_settings: bool,
    pub tool_settings_selected: usize,
    pub show_port_menu: bool,
    pub port_menu_selected: usize,
    pub auto_flash: bool,
    pub use_merged_flash: bool,
    pub flash_batch_mode: bool,
    pub manifest_locked: bool,
    pub show_custom_baud_modal: bool,
    pub custom_baud_input: String,
    pub show_auto_reply_modal: bool,
    pub auto_reply_pattern_input: String,
    pub auto_reply_response_input: String,
    pub auto_reply_focused_field: usize,
    pub show_manifest_edit_modal: bool,
    pub show_manifest_delete_confirm: bool,
    pub manifest_edit_image_label: String,
    pub manifest_delete_image_label: String,
    pub manifest_edit_is_offset: bool,
    pub manifest_edit_input: String,
    pub show_file_picker: bool,
    pub file_picker_current_dir: std::path::PathBuf,
    pub file_picker_items: Vec<FilePickerItem>,
    pub file_picker_selected_idx: usize,
    pub file_picker_search_input: String,
    pub file_picker_image_label: String,

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
    pub serial_quick_scroll: usize,
    pub hover_flash_row: Option<usize>,
    pub hover_flash_action: Option<usize>,
    pub flash_table_scroll: usize,
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
    pub serial_monitor_enabled: bool,
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
    pub serial_tx_senders: std::collections::HashMap<
        String,
        tokio::sync::mpsc::UnboundedSender<crate::worker::MonitorCommand>,
    >,
    pub serial_monitor_baud_rates: std::collections::HashMap<String, u32>,
    pub serial_pending_monitors:
        std::collections::HashMap<String, tokio::sync::oneshot::Sender<()>>,
    pub serial_notice: Option<SerialNotice>,
    pub worker_tx: Option<tokio::sync::mpsc::Sender<crate::worker::WorkerMessage>>,
    pub anim_tick: f32,
    pub splash_ticks_remaining: Option<usize>,
    pub flash_success_ticks_remaining: Option<usize>,
    pub serial_frame_format: String,
    pub serial_auto_reply_enabled: bool,
    pub serial_auto_reply_pattern: String,
    pub serial_auto_reply_response: String,
    pub serial_dtr_active: bool,
    pub serial_rts_active: bool,
    pub serial_auto_baud_scanning: bool,
    #[allow(dead_code)]
    pub serial_auto_baud_tick: u64,
}

impl App {
    pub fn new(config_path: String) -> Self {
        Self::new_with_platformio_ini(config_path, None)
    }

    pub fn new_with_platformio_ini(
        config_path: String,
        external_platformio_ini: Option<std::path::PathBuf>,
    ) -> Self {
        let mut pio_detected = false;
        let mut pio_package_notice = None;
        let requested_config_path = std::path::PathBuf::from(&config_path);
        let factory_dir = std::path::Path::new("build");
        let factory_flash_config = factory_dir.join("piopulse.toml");
        let user_config_path = if requested_config_path == factory_flash_config {
            std::path::PathBuf::from("piopulse.toml")
        } else {
            requested_config_path
        };
        let mut active_config_path = config_path.clone();
        let load_user_or_default = || {
            let config = ProjectConfig::load_from_file(&user_config_path)
                .unwrap_or_else(|_| ProjectConfig::default());
            (config, user_config_path.to_string_lossy().to_string())
        };
        let mut config = if let Some(pio_ini) = external_platformio_ini {
            pio_detected = true;
            let current_dir =
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            match ProjectConfig::prepare_external_platformio_project(&current_dir, &pio_ini) {
                Ok(pio_cfg) => {
                    if let Some(factory_cfg) = create_factory_manifest_from_existing_artifacts(
                        Some(&pio_cfg),
                        factory_dir,
                        &factory_flash_config,
                    ) {
                        pio_package_notice = Some(format!(
                            "External PlatformIO config loaded from {}; build/piopulse.toml generated from existing build binaries.",
                            pio_ini.display()
                        ));
                        active_config_path = factory_flash_config.to_string_lossy().to_string();
                        factory_cfg
                    } else {
                        match pio_cfg.materialize_platformio_factory_package() {
                            Ok(factory_cfg) => {
                                pio_package_notice = Some(format!(
                                    "External PlatformIO config loaded from {}; build package generated at build/piopulse.toml.",
                                    pio_ini.display()
                                ));
                                active_config_path =
                                    factory_flash_config.to_string_lossy().to_string();
                                factory_cfg
                            }
                            Err(err) => {
                                pio_package_notice = Some(format!(
                                    "External PlatformIO config loaded from {}, but startup self-check could not generate build package: {}",
                                    pio_ini.display(),
                                    format_platformio_build_error(&err)
                                ));
                                pio_cfg
                            }
                        }
                    }
                }
                Err(err) => {
                    pio_package_notice = Some(format!(
                        "External PlatformIO config could not be used: {}",
                        format_platformio_build_error(&err)
                    ));
                    let (user_cfg, user_cfg_path) = load_user_or_default();
                    active_config_path = user_cfg_path;
                    user_cfg
                }
            }
        } else if factory_flash_config.exists() {
            active_config_path = factory_flash_config.to_string_lossy().to_string();
            let loaded =
                ProjectConfig::load_from_file(&factory_flash_config).unwrap_or_else(|_| {
                    let (user_cfg, user_cfg_path) = load_user_or_default();
                    active_config_path = user_cfg_path;
                    user_cfg
                });
            if manifest_has_errors(&loaded) || manifest_needs_platformio_refresh(&loaded) {
                if let Some(factory_cfg) = create_factory_manifest_from_existing_artifacts(
                    Some(&loaded),
                    factory_dir,
                    &factory_flash_config,
                ) {
                    pio_package_notice = Some(
                        "Startup self-check repaired build/piopulse.toml from existing build binaries."
                            .to_string(),
                    );
                    active_config_path = factory_flash_config.to_string_lossy().to_string();
                    factory_cfg
                } else if let Some(pio_cfg) = ProjectConfig::detect_platformio_config() {
                    pio_detected = true;
                    match pio_cfg.materialize_platformio_factory_package() {
                        Ok(factory_cfg) => {
                            pio_package_notice = Some(
                                "Startup self-check rebuilt build/piopulse.toml from PlatformIO because the build manifest was invalid."
                                    .to_string(),
                            );
                            active_config_path = factory_flash_config.to_string_lossy().to_string();
                            factory_cfg
                        }
                        Err(err) => {
                            pio_package_notice = Some(format!(
                                "Startup self-check found an invalid build manifest but could not rebuild build package: {}",
                                format_platformio_build_error(&err)
                            ));
                            loaded
                        }
                    }
                } else {
                    loaded
                }
            } else {
                loaded
            }
        } else if let Some(pio_cfg) = ProjectConfig::detect_platformio_config() {
            pio_detected = true;
            if let Some(factory_cfg) = create_factory_manifest_from_existing_artifacts(
                Some(&pio_cfg),
                factory_dir,
                &factory_flash_config,
            ) {
                pio_package_notice = Some("PlatformIO project detected; build/piopulse.toml generated from existing build binaries.".to_string());
                active_config_path = factory_flash_config.to_string_lossy().to_string();
                factory_cfg
            } else {
                match pio_cfg.materialize_platformio_factory_package() {
                    Ok(factory_cfg) => {
                        pio_package_notice = Some(
                            "PlatformIO project detected; build package generated at build/piopulse.toml."
                                .to_string(),
                        );
                        active_config_path = factory_flash_config.to_string_lossy().to_string();
                        factory_cfg
                    }
                    Err(err) => {
                        pio_package_notice = Some(format!(
                            "PlatformIO project detected, but startup self-check could not generate build package: {}",
                            format_platformio_build_error(&err)
                        ));
                        pio_cfg
                    }
                }
            }
        } else if let Some(factory_cfg) = create_factory_manifest_from_existing_artifacts(
            None,
            factory_dir,
            &factory_flash_config,
        ) {
            pio_package_notice = Some(
                "Build manifest generated at build/piopulse.toml from existing build binaries."
                    .to_string(),
            );
            active_config_path = factory_flash_config.to_string_lossy().to_string();
            factory_cfg
        } else if factory_dir.is_dir() {
            let mut default_cfg = ProjectConfig::default();
            default_cfg.populate_default_images_if_empty(factory_dir);
            default_cfg
        } else {
            let (user_cfg, user_cfg_path) = load_user_or_default();
            active_config_path = user_cfg_path;
            user_cfg
        };

        let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let default_image_dir = if factory_dir.is_dir() {
            factory_dir
        } else {
            current_dir.as_path()
        };
        config.populate_default_images_if_empty(default_image_dir);

        let has_segmented = config.images.iter().any(|img| {
            img.label != "merged"
                && img.label != "factory_merged"
                && !img.path.ends_with("merged.bin")
        });
        let has_merged = config.images.iter().any(|img| {
            img.label == "merged"
                || img.label == "factory_merged"
                || img.path.ends_with("merged.bin")
        });
        let use_merged_flash = has_merged && (config.use_merged_flash || !has_segmented);
        let manifest_locked = config.manifest_locked;

        let waveform_history = std::collections::HashMap::new();

        let tool_config = ToolConfig::load();
        let tool_settings_selected = if tool_config.language == "zh" { 1 } else { 0 };
        let (log_file, log_file_path, log_init_error) = open_session_log_file();

        let mut app = Self {
            channels: Vec::new(),
            stats: Stats {
                total_passed: 0,
                total_failed: 0,
                total_attempted: 0,
            },
            logs: Vec::new(),
            log_file,
            config,
            config_path: active_config_path,
            tool_config,
            show_tool_settings: false,
            tool_settings_selected,
            show_port_menu: false,
            port_menu_selected: 0,
            show_custom_baud_modal: false,
            custom_baud_input: String::new(),
            show_auto_reply_modal: false,
            auto_reply_pattern_input: String::new(),
            auto_reply_response_input: String::new(),
            auto_reply_focused_field: 0,
            show_manifest_edit_modal: false,
            show_manifest_delete_confirm: false,
            manifest_edit_image_label: String::new(),
            manifest_delete_image_label: String::new(),
            manifest_edit_is_offset: false,
            manifest_edit_input: String::new(),
            show_file_picker: false,
            file_picker_current_dir: std::path::PathBuf::new(),
            file_picker_items: Vec::new(),
            file_picker_selected_idx: 0,
            file_picker_search_input: String::new(),
            file_picker_image_label: String::new(),
            active_tab: ActiveTab::Serial,
            selected_channel_idx: 0,
            selected_config_field: 0,
            is_editing_config: false,
            edit_buffer: String::new(),
            hover_tab: None,
            hover_serial_port_info: None,
            hover_serial_option: None,
            hover_serial_quick_command: None,
            serial_quick_scroll: 0,
            hover_flash_row: None,
            hover_flash_action: None,
            flash_table_scroll: 0,
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
            serial_monitor_enabled: false,
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
            serial_pending_monitors: std::collections::HashMap::new(),
            serial_notice: None,
            worker_tx: None,
            anim_tick: 0.0,
            splash_ticks_remaining: Some(SPLASH_TICKS),
            flash_success_ticks_remaining: None,
            serial_frame_format: "8-N-1".to_string(),
            serial_auto_reply_enabled: false,
            serial_auto_reply_pattern: String::new(),
            serial_auto_reply_response: String::new(),
            serial_dtr_active: true,
            serial_rts_active: true,
            serial_auto_baud_scanning: false,
            serial_auto_baud_tick: 0,
            auto_flash: false,
            use_merged_flash,
            flash_batch_mode: true,
            manifest_locked,
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
        if let Some(notice) = pio_package_notice {
            app.log(notice);
        }
        if let Some(path) = log_file_path {
            app.log(format!("Session log file: {}", path.display()));
        }
        if let Some(err) = log_init_error {
            app.log(format!("Session log file unavailable: {}", err));
        }
        app
    }

    pub fn log(&mut self, msg: impl Into<String>) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        let line = format!("[{}] {}", timestamp, msg.into());
        self.write_session_log_line(&line);
        self.logs.push(line);
        // Keep logs size reasonable
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    fn log_file_event(&mut self, msg: impl Into<String>) {
        let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
        let line = format!("[{}] {}", timestamp, msg.into());
        self.write_session_log_line(&line);
    }

    fn write_session_log_line(&mut self, line: &str) {
        if let Some(file) = self.log_file.as_mut() {
            let _ = writeln!(file, "{}", line);
            let _ = file.flush();
        }
    }

    pub fn channel_log(&mut self, port: &str, msg: impl Into<String>) {
        self.log(format!("[{}] {}", port, msg.into()));
    }

    pub fn scan_ports(&mut self, tx: Option<tokio::sync::mpsc::Sender<WorkerMessage>>) {
        let ports = worker::get_available_serial_ports();

        let mut new_channels = Vec::new();
        let mut updated = false;

        // 1. Reconcile currently connected ports with existing channels
        for p in ports {
            if let Some(existing_idx) = self.channels.iter().position(|c| c.port == p.name) {
                // Keep existing channel to preserve state (flashing progress/results)
                new_channels.push(self.channels.remove(existing_idx));
            } else {
                // This is a newly plugged in device!
                let chan = Channel::new(p.clone());
                new_channels.push(chan);
                updated = true;
            }
        }

        // Any channels left in self.channels were unplugged
        if !self.channels.is_empty() {
            let removed_ports: Vec<String> = self
                .channels
                .iter()
                .map(|channel| channel.port.clone())
                .collect();
            for port in removed_ports {
                self.log(format!("Device unplugged: {}", port));
            }
            updated = true;
        }

        self.channels = new_channels;

        if updated {
            self.log(format!(
                "Ports updated. Found {} active devices.",
                self.channels.len()
            ));
            if self.selected_channel_idx >= self.channels.len() {
                self.selected_channel_idx = 0;
            }
            self.flash_table_scroll = self.flash_table_scroll.min(
                self.channels
                    .len()
                    .saturating_sub(self.layout_zones.flash_device_table.height as usize),
            );
            let _ = tx;
        }
    }

    pub fn toggle_merged_flash(&mut self) {
        if self.manifest_locked {
            self.log(
                "Firmware manifest is locked; click the status bar to unlock before changing flash mode.",
            );
            return;
        }
        self.use_merged_flash = !self.use_merged_flash;
        self.config.use_merged_flash = self.use_merged_flash;
        let _ = self.config.save_to_file(&self.config_path);
        self.log(format!(
            "Flash mode toggled to: {}",
            if self.use_merged_flash {
                "Merged Bin"
            } else {
                "Segmented"
            }
        ));
    }

    pub fn toggle_do_not_chg_bin(&mut self) {
        if self.manifest_locked {
            self.log(
                "Firmware manifest is locked; click the status bar to unlock before changing DoNotChgBin.",
            );
            return;
        }
        self.config.do_not_chg_bin = !self.config.do_not_chg_bin;
        let _ = self.config.save_to_file(&self.config_path);
        self.log(format!(
            "DoNotChgBin set to {}.",
            if self.config.do_not_chg_bin {
                "TRUE"
            } else {
                "FALSE"
            }
        ));
    }

    pub fn start_flashing_selected(&mut self, tx: tokio::sync::mpsc::Sender<WorkerMessage>) {
        if self.channels.is_empty() {
            self.log(
                "No serial device available. Connect a board or rescan ports before flashing.",
            );
            return;
        }

        let idx = self.selected_channel_idx.min(self.channels.len() - 1);
        self.start_flashing_indices(vec![idx], tx, "selected device");
    }

    pub fn start_flashing(&mut self, tx: tokio::sync::mpsc::Sender<WorkerMessage>) {
        if self.is_flashing {
            self.log("Flash request ignored because flashing is already active.");
            return;
        }
        if self.channels.is_empty() {
            self.log(
                "No serial device available. Connect a board or rescan ports before flashing.",
            );
            return;
        }

        self.start_flashing_indices((0..self.channels.len()).collect(), tx, "all devices");
    }

    fn current_mode_manifest_errors(&self) -> Vec<String> {
        let (_, errors) = self.config.validate_manifest();
        errors
            .into_iter()
            .filter(|err| {
                if self.use_merged_flash {
                    !err.contains("bootloader")
                        && !err.contains("partitions")
                        && !err.contains("boot_app0")
                        && !err.contains("firmware")
                        && !err.contains("Required image")
                } else {
                    !err.contains("merged")
                }
            })
            .collect()
    }

    fn refresh_flash_mode_from_images(&mut self) {
        let has_segmented = self.config.images.iter().any(|img| {
            img.label != "merged"
                && img.label != "factory_merged"
                && !img.path.ends_with("merged.bin")
        });
        let has_merged = self.config.images.iter().any(|img| {
            img.label == "merged"
                || img.label == "factory_merged"
                || img.path.ends_with("merged.bin")
        });
        self.use_merged_flash = has_merged && (self.config.use_merged_flash || !has_segmented);
        self.config.use_merged_flash = self.use_merged_flash;
    }

    pub fn ensure_flash_manifest_ready(&mut self) -> bool {
        let errors = self.current_mode_manifest_errors();
        if errors.is_empty() {
            return true;
        }

        self.log("Firmware manifest is incomplete; trying to build PlatformIO project and generate build package.");

        match self.config.materialize_platformio_factory_package() {
            Ok(config) => {
                self.config = config;
                self.refresh_flash_mode_from_images();
                let errors = self.current_mode_manifest_errors();
                if errors.is_empty() {
                    self.log("Build package generated at build/piopulse.toml; manifest is ready.");
                    true
                } else {
                    self.log(format!(
                        "Factory package generated but manifest is still invalid: {}",
                        errors.join(" | ")
                    ));
                    false
                }
            }
            Err(err) => {
                self.log(format!(
                    "Cannot start flashing because firmware files are missing and build package generation failed: {}",
                    format_platformio_build_error(&err)
                ));
                false
            }
        }
    }

    pub fn update_auto_flash_sensing(&mut self, tx: tokio::sync::mpsc::Sender<WorkerMessage>) {
        if !self.auto_flash || self.is_flashing {
            return;
        }

        let probe_interval = Duration::from_secs(2);
        let config = Arc::new(self.config.clone());

        for channel in &mut self.channels {
            if channel.auto_probe_pending || channel.auto_last_probe_at.elapsed() < probe_interval {
                continue;
            }

            let should_probe =
                channel.auto_flash_armed || (channel.finished && !channel.auto_flash_armed);
            if !should_probe {
                continue;
            }

            channel.auto_probe_pending = true;
            channel.auto_last_probe_at = Instant::now();
            if channel.auto_flash_armed {
                channel.status = "Auto sensing".to_string();
                channel.progress = 0;
            } else {
                channel.status = "Remove flashed board".to_string();
                channel.progress = 100;
            }

            worker::start_auto_probe_task(channel.port.clone(), config.clone(), tx.clone());
        }
    }

    pub fn move_flash_selection(&mut self, delta: isize) {
        if self.channels.is_empty() {
            return;
        }

        let last_idx = self.channels.len() - 1;
        self.selected_channel_idx = if delta.is_negative() {
            self.selected_channel_idx
                .saturating_sub(delta.unsigned_abs())
        } else {
            self.selected_channel_idx
                .saturating_add(delta as usize)
                .min(last_idx)
        };

        let visible_rows = self
            .layout_zones
            .flash_device_table
            .height
            .saturating_sub(3) as usize;
        if visible_rows == 0 {
            return;
        }

        if self.selected_channel_idx < self.flash_table_scroll {
            self.flash_table_scroll = self.selected_channel_idx;
        } else if self.selected_channel_idx >= self.flash_table_scroll + visible_rows {
            self.flash_table_scroll = self.selected_channel_idx + 1 - visible_rows;
        }
    }

    fn start_flashing_indices(
        &mut self,
        indices: Vec<usize>,
        tx: tokio::sync::mpsc::Sender<WorkerMessage>,
        scope: &str,
    ) {
        if self.is_flashing {
            self.log("Flash request ignored because flashing is already active.");
            return;
        }
        if indices.is_empty() {
            self.log("Flash request ignored because no device rows are selected.");
            return;
        }

        if !self.ensure_flash_manifest_ready() {
            for idx in indices {
                if let Some(channel) = self.channels.get_mut(idx) {
                    channel.status = "Manifest missing".to_string();
                    channel.progress = 0;
                    channel.error = Some("Firmware manifest is incomplete".to_string());
                    channel.finished = false;
                    channel.success = false;
                }
            }
            return;
        }

        if self.serial_monitor_enabled || !self.serial_tx_senders.is_empty() {
            self.serial_monitor_enabled = false;
            self.stop_serial_monitors("Stopped serial monitor before flashing.");
            std::thread::sleep(Duration::from_millis(150));
        }

        self.is_flashing = true;
        self.start_time = Some(Instant::now());
        self.elapsed_time = Duration::from_secs(0);
        self.log(format!(
            "--- Start Flashing {} ({}) ---",
            scope,
            indices.len()
        ));

        let config_arc = Arc::new(self.config.clone());

        for idx in indices {
            let Some(channel) = self.channels.get_mut(idx) else {
                continue;
            };
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
            channel.auto_flash_armed = false;
            channel.auto_probe_pending = false;

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
        self.anim_tick += 0.1;
        if let Some(ticks) = self.splash_ticks_remaining {
            if ticks > 1 {
                self.splash_ticks_remaining = Some(ticks - 1);
            } else {
                self.finish_splash();
            }
        }
        if let Some(ticks) = self.flash_success_ticks_remaining {
            if ticks > 1 {
                self.flash_success_ticks_remaining = Some(ticks - 1);
            } else {
                self.flash_success_ticks_remaining = None;
            }
        }
        self.update_elapsed_time();
        self.update_serial_monitoring();
        self.update_serial_playback();
        self.update_serial_notice();
    }

    pub fn finish_splash(&mut self) {
        if self.splash_ticks_remaining.is_some() {
            self.splash_ticks_remaining = None;
        }
    }

    pub fn play_sound(&self, effect: SoundEffect) {
        std::thread::spawn(move || {
            let sample_rate = 8000.0;
            let bytes = match effect {
                SoundEffect::Boot => {
                    let duration = 0.55;
                    let num_samples = (sample_rate * duration) as usize;
                    let mut data = Vec::with_capacity(num_samples);
                    for i in 0..num_samples {
                        let t = i as f64 / sample_rate;
                        let env = (-4.0 * t).exp();
                        let fc = 350.0 + 550.0 * (t / duration);
                        let fm = 110.0;
                        let index = 8.0 * (1.0 - t / duration);

                        let phase_m = 2.0 * std::f64::consts::PI * fm * t;
                        let phase_c = 2.0 * std::f64::consts::PI * fc * t;
                        let sample = (phase_c + index * phase_m.sin()).sin();
                        let byte_val = (127.5 + 127.0 * sample * env) as u8;
                        data.push(byte_val);
                    }
                    data
                }
                SoundEffect::Success => {
                    let duration = 0.55;
                    let num_samples = (sample_rate * duration) as usize;
                    let mut data = Vec::with_capacity(num_samples);
                    for i in 0..num_samples {
                        let t = i as f64 / sample_rate;
                        let env = (-3.0 * t).exp();

                        let sample = if t < 0.15 {
                            let fc = 523.25; // C5
                            let fm = 261.6;
                            let index = 3.0;
                            (2.0 * std::f64::consts::PI * fc * t
                                + index * (2.0 * std::f64::consts::PI * fm * t).sin())
                            .sin()
                        } else {
                            let t2 = t - 0.15;
                            let fc = 783.99; // G5
                            let fm = 392.0;
                            let index = 2.0;
                            (2.0 * std::f64::consts::PI * fc * t2
                                + index * (2.0 * std::f64::consts::PI * fm * t2).sin())
                            .sin()
                        };

                        let byte_val = (127.5 + 127.0 * sample * env) as u8;
                        data.push(byte_val);
                    }
                    data
                }
                SoundEffect::Failure => {
                    let duration = 0.7;
                    let num_samples = (sample_rate * duration) as usize;
                    let mut data = Vec::with_capacity(num_samples);
                    for i in 0..num_samples {
                        let t = i as f64 / sample_rate;
                        let env = (-2.0 * t).exp();
                        let fc = 200.0 - 120.0 * (t / duration);
                        let fm = 55.0;
                        let index = 12.0 * (1.0 - t / duration);

                        let sample = (2.0 * std::f64::consts::PI * fc * t
                            + index * (2.0 * std::f64::consts::PI * fm * t).sin())
                        .sin();
                        let byte_val = (127.5 + 127.0 * sample * env) as u8;
                        data.push(byte_val);
                    }
                    data
                }
                SoundEffect::Connect => {
                    let duration = 0.18;
                    let num_samples = (sample_rate * duration) as usize;
                    let mut data = Vec::with_capacity(num_samples);
                    for i in 0..num_samples {
                        let t = i as f64 / sample_rate;
                        let env = (-4.0 * t).exp();
                        let fc = 600.0 + 800.0 * (t / duration);
                        let fm = 150.0;
                        let index = 2.0;
                        let sample = (2.0 * std::f64::consts::PI * fc * t
                            + index * (2.0 * std::f64::consts::PI * fm * t).sin())
                        .sin();
                        let byte_val = (127.5 + 127.0 * sample * env) as u8;
                        data.push(byte_val);
                    }
                    data
                }
                SoundEffect::Disconnect => {
                    let duration = 0.22;
                    let num_samples = (sample_rate * duration) as usize;
                    let mut data = Vec::with_capacity(num_samples);
                    for i in 0..num_samples {
                        let t = i as f64 / sample_rate;
                        let env = (-3.0 * t).exp();
                        let fc = 1200.0 - 700.0 * (t / duration);
                        let fm = 100.0;
                        let index = 4.0;
                        let sample = (2.0 * std::f64::consts::PI * fc * t
                            + index * (2.0 * std::f64::consts::PI * fm * t).sin())
                        .sin();
                        let byte_val = (127.5 + 127.0 * sample * env) as u8;
                        data.push(byte_val);
                    }
                    data
                }
            };

            use std::io::Write;
            if let Ok(mut child) = std::process::Command::new("aplay")
                .args(&["-t", "raw", "-r", "8000", "-f", "U8"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(&bytes);
                }
                let _ = child.wait();
            }
        });
    }

    fn update_serial_notice(&mut self) {
        if self
            .serial_notice
            .as_ref()
            .is_some_and(|notice| Instant::now() >= notice.expires_at)
        {
            self.serial_notice = None;
        }
    }

    pub fn show_serial_notice(&mut self, message: impl Into<String>, kind: SerialNoticeKind) {
        let now = Instant::now();
        self.serial_notice = Some(SerialNotice {
            message: message.into(),
            kind,
            started_at: now,
            expires_at: now + Duration::from_millis(2600),
        });
    }

    pub fn update_serial_monitoring(&mut self) {
        if self.is_flashing {
            self.stop_serial_monitors("Serial monitor closed while flashing is active.");
            return;
        }

        if !self.serial_monitor_enabled {
            self.stop_serial_monitors("Serial monitor paused.");
            return;
        }

        let selected_port = self
            .get_selected_serial_port()
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
                    self.show_serial_notice(
                        format!("Restarting {} at {} bps", port, self.serial_baud_rate),
                        SerialNoticeKind::Info,
                    );
                } else {
                    self.log(format!("Closed Serial Monitor for {}.", port));
                    self.show_serial_notice(
                        format!("Released serial port {}", port),
                        SerialNoticeKind::Success,
                    );
                }
            }
        }

        // Start monitor for selected port
        if selected_port != "NONE" {
            if !self.serial_tx_senders.contains_key(&selected_port)
                && !self.serial_pending_monitors.contains_key(&selected_port)
            {
                if let Some(ref tx) = self.worker_tx {
                    let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();
                    self.serial_pending_monitors
                        .insert(selected_port.clone(), cancel_tx);
                    worker::spawn_serial_monitor(
                        selected_port.clone(),
                        self.serial_baud_rate,
                        self.serial_frame_format.clone(),
                        tx.clone(),
                        cancel_rx,
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

    pub fn visible_port_indices_for_active_tab(&self) -> Vec<usize> {
        match self.active_tab {
            ActiveTab::Serial | ActiveTab::Plotter => self
                .channels
                .iter()
                .enumerate()
                .filter_map(|(idx, channel)| (!channel.port.starts_with("probe:")).then_some(idx))
                .collect(),
            _ => (0..self.channels.len()).collect(),
        }
    }

    pub fn get_selected_serial_port(&self) -> Option<String> {
        self.channels
            .get(self.selected_channel_idx)
            .filter(|channel| !channel.port.starts_with("probe:"))
            .or_else(|| {
                self.channels
                    .iter()
                    .find(|channel| !channel.port.starts_with("probe:"))
            })
            .map(|channel| channel.port.clone())
    }

    pub fn toggle_serial_monitor(&mut self) {
        if !self.serial_monitor_enabled && self.get_selected_serial_port().is_none() {
            self.log("Serial monitor not started: no serial port is available.");
            self.show_serial_notice("No serial port available", SerialNoticeKind::Warning);
            return;
        }
        self.serial_monitor_enabled = !self.serial_monitor_enabled;
        if self.serial_monitor_enabled {
            self.log("Serial monitor resumed.");
            self.show_serial_notice("Serial monitor resumed", SerialNoticeKind::Info);
        } else {
            let had_monitor =
                !self.serial_tx_senders.is_empty() || !self.serial_pending_monitors.is_empty();
            self.stop_serial_monitors("Serial monitor paused and port released.");
            if !had_monitor {
                self.show_serial_notice("Serial monitor paused", SerialNoticeKind::Warning);
            }
        }
    }

    fn stop_serial_monitors(&mut self, reason: &str) {
        let had_monitor =
            !self.serial_tx_senders.is_empty() || !self.serial_pending_monitors.is_empty();
        self.serial_tx_senders.clear();
        self.serial_monitor_baud_rates.clear();
        self.serial_pending_monitors.clear();
        if had_monitor {
            self.log(reason);
            self.show_serial_notice(reason, SerialNoticeKind::Success);
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
                let status_for_log = status.clone();
                let speed_for_log = speed.clone();
                if let Some(channel) = self.channels.iter_mut().find(|c| c.port == port) {
                    channel.status = status;
                    channel.progress = progress;
                    channel.speed = speed;
                }
                self.log_file_event(format!(
                    "[{}] STATUS status='{}' progress={} speed='{}'",
                    port, status_for_log, progress, speed_for_log
                ));
            }
            WorkerMessage::MacAddressDetected { port, mac, chip } => {
                self.log_file_event(format!("[{}] DETECTED chip='{}' mac='{}'", port, chip, mac));
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
                self.log_file_event(format!(
                    "[{}] PROVISION serial='{}' device='{}'",
                    port, serial_number, device_name
                ));
                if let Some(channel) = self.channels.iter_mut().find(|c| c.port == port) {
                    channel.serial_number = Some(serial_number);
                    channel.device_name = Some(device_name);
                }
            }
            WorkerMessage::ProductionStep { port, step, detail } => {
                self.log_file_event(format!("[{}] STEP {}='{}'", port, step, detail));
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
            WorkerMessage::AutoProbeResult {
                port,
                present,
                chip,
                mac,
                error_msg,
            } => {
                let mut start_idx = None;
                let mut log_msg = None;

                if let Some(idx) = self.channels.iter().position(|c| c.port == port) {
                    let channel = &mut self.channels[idx];
                    channel.auto_probe_pending = false;
                    channel.auto_last_probe_at = Instant::now();

                    if present {
                        channel.chip = chip;
                        channel.mac = mac;
                        if channel.auto_flash_armed {
                            channel.status = "Auto detected".to_string();
                            channel.progress = 1;
                            start_idx = Some(idx);
                            log_msg =
                                Some(format!("[{}] Auto-sensed product, starting flash.", port));
                        } else {
                            channel.status = "Remove flashed board".to_string();
                            channel.progress = 100;
                        }
                    } else if channel.auto_flash_armed {
                        channel.status = "Waiting for product".to_string();
                        channel.progress = 0;
                        channel.chip = None;
                        channel.mac = None;
                        if let Some(err) = error_msg {
                            channel.error = Some(err);
                        }
                    } else {
                        channel.auto_flash_armed = true;
                        channel.finished = false;
                        channel.success = false;
                        channel.status = "Waiting for product".to_string();
                        channel.progress = 0;
                        channel.chip = None;
                        channel.mac = None;
                        channel.serial_number = None;
                        channel.device_name = None;
                        channel.trace_id = None;
                        channel.error = None;
                        channel.qa_result = "Pending".to_string();
                        log_msg = Some(format!(
                            "[{}] Flashed board removed; station is ready for the next product.",
                            port
                        ));
                    }
                }

                if let Some(msg) = log_msg {
                    self.log(msg);
                }

                if let (Some(idx), Some(tx)) = (start_idx, self.worker_tx.clone()) {
                    if self.auto_flash && !self.is_flashing {
                        self.start_flashing_indices(vec![idx], tx, "auto-sensed product");
                    }
                }
            }
            WorkerMessage::Finished {
                port,
                success,
                error_msg,
                mac,
            } => {
                self.log_file_event(format!(
                    "[{}] FINISHED result={} mac='{}' error='{}'",
                    port,
                    if success { "OK" } else { "FAIL" },
                    mac.as_deref().unwrap_or("-"),
                    error_msg.as_deref().unwrap_or("")
                ));
                self.serial_tx_senders.remove(&port);
                self.serial_monitor_baud_rates.remove(&port);
                self.serial_pending_monitors.remove(&port);
                let mut log_msg = None;
                let mut play_effect = None;
                if let Some(channel) = self.channels.iter_mut().find(|c| c.port == port) {
                    channel.finished = true;
                    channel.success = success;
                    channel.auto_probe_pending = false;
                    if self.auto_flash {
                        channel.auto_flash_armed = false;
                        channel.auto_last_probe_at = Instant::now();
                    }
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
                        play_effect = Some(SoundEffect::Success);
                        self.flash_success_ticks_remaining = Some(30);
                    } else {
                        channel.status = "FAILED".to_string();
                        let err = error_msg.clone().unwrap_or_default();
                        channel.error = Some(err.clone());
                        channel.qa_result = "FAIL".to_string();
                        self.stats.total_failed += 1;
                        log_msg = Some(format!("Flashing FAILED: {}", err));
                        play_effect = Some(SoundEffect::Failure);
                    }
                    if let Some(m) = mac {
                        channel.mac = Some(m);
                    }
                }

                if let Some(effect) = play_effect {
                    self.play_sound(effect);
                }

                if let Some(msg) = log_msg {
                    self.channel_log(&port, msg);
                }

                // Check if all channels have finished
                let all_finished = self
                    .channels
                    .iter()
                    .all(|c| c.finished || c.status == "Idle");
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

                // Auto-Reply logic
                if self.serial_auto_reply_enabled && !self.serial_auto_reply_pattern.is_empty() {
                    let rx_text = String::from_utf8_lossy(&data);
                    let matched = if let Ok(re) = regex::Regex::new(&self.serial_auto_reply_pattern)
                    {
                        re.is_match(&rx_text)
                    } else {
                        rx_text.contains(&self.serial_auto_reply_pattern)
                    };
                    if matched {
                        let unescaped_resp = unescape_string(&self.serial_auto_reply_response);
                        let bytes = unescaped_resp.as_bytes().to_vec();

                        if let Some(sender) = self.serial_tx_senders.get(&port).cloned() {
                            let tx_log =
                                format!("[Auto-Reply] {}", self.serial_auto_reply_response);
                            self.log(format!("[{}] [TX] {}", port, tx_log));
                            self.capture_serial_timeline_frame(SerialDirection::Tx, &port, &bytes);
                            let _ = sender.send(crate::worker::MonitorCommand::WriteData(bytes));
                        }
                    }
                }

                for line in format_serial_rx_messages(&data, self.serial_hex_mode_rx) {
                    self.channel_log(&port, line);
                }
            }
            WorkerMessage::BaudDetected { port, baud_rate } => {
                self.serial_auto_baud_scanning = false;
                self.serial_baud_rate = baud_rate;
                self.show_serial_notice(
                    format!("Baud: {} bps", baud_rate),
                    SerialNoticeKind::Success,
                );
                self.log(format!(
                    "[{}] Auto-detected baud rate: {} bps.",
                    port, baud_rate
                ));

                // If it was monitoring, restart monitoring at the new baud rate
                if let Some(active_port) = self.get_selected_serial_port() {
                    if active_port == port {
                        let had_monitor = self.serial_tx_senders.remove(&port).is_some()
                            || self.serial_monitor_baud_rates.remove(&port).is_some()
                            || self.serial_pending_monitors.remove(&port).is_some();
                        if had_monitor {
                            self.toggle_serial_monitor(); // Re-open at new baud
                        }
                    }
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
                self.serial_tx_senders.insert(port.clone(), sender.clone());

                // Set initial DTR and RTS pin states
                let _ = sender.send(crate::worker::MonitorCommand::SetDtr(
                    self.serial_dtr_active,
                ));
                let _ = sender.send(crate::worker::MonitorCommand::SetRts(
                    self.serial_rts_active,
                ));

                self.show_serial_notice(
                    format!("Monitoring {} at {} bps", port, baud_rate),
                    SerialNoticeKind::Info,
                );
                self.play_sound(SoundEffect::Connect);
            }
            WorkerMessage::MonitorStopped { port } => {
                self.serial_pending_monitors.remove(&port);
                self.serial_monitor_baud_rates.remove(&port);
                self.serial_tx_senders.remove(&port);
                self.show_serial_notice(
                    format!("Serial port {} released", port),
                    SerialNoticeKind::Success,
                );
                self.play_sound(SoundEffect::Disconnect);
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
        } else {
            self.log("No widget selected. Press A to add a widget first.");
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
        if let Some(port) = self.get_selected_serial_port() {
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
        self.get_selected_serial_port()
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

    #[allow(dead_code)]
    pub fn cycle_serial_baud_rate(&mut self) {
        self.serial_baud_rate = next_serial_baud_rate(self.serial_baud_rate);
        if let Some(port) = self.get_selected_serial_port() {
            let had_monitor = self.serial_tx_senders.remove(&port).is_some()
                || self.serial_monitor_baud_rates.remove(&port).is_some()
                || self.serial_pending_monitors.remove(&port).is_some();
            if had_monitor {
                self.show_serial_notice(
                    format!("Restarting {} at {} bps", port, self.serial_baud_rate),
                    SerialNoticeKind::Info,
                );
            }
        }
        self.log(format!("Baud rate set to {} bps.", self.serial_baud_rate));
    }

    pub fn cycle_serial_frame_format(&mut self) {
        let next = match self.serial_frame_format.as_str() {
            "8-N-1" => "8-E-1",
            "8-E-1" => "8-O-1",
            "8-O-1" => "8-N-2",
            "8-N-2" => "7-N-1",
            "7-N-1" => "7-E-1",
            "7-E-1" => "7-O-1",
            _ => "8-N-1",
        };
        self.serial_frame_format = next.to_string();
        if let Some(port) = self.get_selected_serial_port() {
            let had_monitor = self.serial_tx_senders.remove(&port).is_some()
                || self.serial_monitor_baud_rates.remove(&port).is_some()
                || self.serial_pending_monitors.remove(&port).is_some();
            if had_monitor {
                self.show_serial_notice(
                    format!("Restarting {} with {}", port, self.serial_frame_format),
                    SerialNoticeKind::Info,
                );
            }
        }
        self.log(format!(
            "Serial frame format set to {}.",
            self.serial_frame_format
        ));
    }

    pub fn submit_serial_command(&mut self, cmd: &str) {
        let trimmed = cmd.trim();
        if trimmed.is_empty() {
            return;
        }

        let port = self
            .get_selected_serial_port()
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
                    if let Err(e) = sender.send(crate::worker::MonitorCommand::WriteData(bytes)) {
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
            crate::vofa::VofaMode::IndexFloat => crate::vofa::VofaMode::RawData,
            crate::vofa::VofaMode::RawData => crate::vofa::VofaMode::FireWater,
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
            .get_selected_serial_port()
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
        let items =
            plotter_header_items(lang, true, selected_port, protocol, view, state.to_string());

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
        let title = format!(" ☕ PIOPULSE v{} ", env!("CARGO_PKG_VERSION"));
        let mode = if self.admin_mode {
            crate::ui::tr("admin_mode_header", &self.tool_config.language)
        } else {
            crate::ui::tr("operator_mode_header", &self.tool_config.language)
        };
        let relative_col = col.saturating_sub(area.x) as usize;
        let mode_start = UnicodeWidthStr::width(title.as_str()) + UnicodeWidthStr::width(" | ");
        let mode_end = mode_start + UnicodeWidthStr::width(mode);

        (mode_start..mode_end).contains(&relative_col)
    }

    pub fn get_flash_summary_button_rects(&self) -> Vec<Rect> {
        let area = self.layout_zones.flash_summary;
        if area.width < 4 || area.height < 8 {
            return vec![Rect::default(); 4];
        }

        let stats_block = ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .border_type(ratatui::widgets::BorderType::Rounded);
        let inner_area = stats_block.inner(area);

        let vertical_chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(4), // Stats text + separator line
                ratatui::layout::Constraint::Length(2), // Buttons rows
            ])
            .split(inner_area);

        let button_rows = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(1),
                ratatui::layout::Constraint::Length(1),
            ])
            .split(vertical_chunks[1]);

        let lang = &self.tool_config.language;
        let available_w = inner_area.width as usize;
        let btn_w = if lang == "zh" {
            14.min((available_w.saturating_sub(4)) / 2)
        } else {
            16.min((available_w.saturating_sub(4)) / 2)
        };

        let row1_cols = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Min(0),
                ratatui::layout::Constraint::Length(btn_w as u16),
                ratatui::layout::Constraint::Length(2),
                ratatui::layout::Constraint::Length(btn_w as u16),
                ratatui::layout::Constraint::Min(0),
            ])
            .split(button_rows[0]);

        let row2_cols = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                ratatui::layout::Constraint::Min(0),
                ratatui::layout::Constraint::Length(btn_w as u16),
                ratatui::layout::Constraint::Length(2),
                ratatui::layout::Constraint::Length(btn_w as u16),
                ratatui::layout::Constraint::Min(0),
            ])
            .split(button_rows[1]);

        vec![row1_cols[1], row1_cols[3], row2_cols[1], row2_cols[3]]
    }

    pub fn flash_summary_action_at(&self, col: u16, row: u16) -> Option<usize> {
        let area = self.layout_zones.flash_summary;
        if !self.is_inside_rect(col, row, area) {
            return None;
        }

        let rects = self.get_flash_summary_button_rects();
        for (idx, rect) in rects.iter().enumerate() {
            if col >= rect.x
                && col < rect.x + rect.width
                && row >= rect.y
                && row < rect.y + rect.height
            {
                return Some(idx);
            }
        }
        None
    }

    pub fn flash_table_row_at(&self, row: u16) -> Option<usize> {
        let area = self.layout_zones.flash_device_table;
        if row < area.y + 2 || row >= area.y + area.height.saturating_sub(1) {
            return None;
        }

        let visible_row = row.saturating_sub(area.y + 2) as usize;
        let idx = self.flash_table_scroll + visible_row;
        if idx < self.channels.len() {
            Some(idx)
        } else {
            None
        }
    }

    pub fn handle_mouse_right_click(
        &mut self,
        col: u16,
        row: u16,
        _tx: tokio::sync::mpsc::Sender<WorkerMessage>,
    ) -> bool {
        if self.active_tab == ActiveTab::Flasher
            && self.is_inside_rect(col, row, self.layout_zones.flash_manifest_table)
        {
            if self.manifest_locked {
                self.log(
                    "Firmware manifest is locked; click the status bar to unlock before deleting.",
                );
                return true;
            }

            let rect = self.layout_zones.flash_manifest_table;
            let relative_row = row.saturating_sub(rect.y + 1) as usize;
            let filtered_results = self.config.manifest_results_for_mode(self.use_merged_flash);
            if let Some(res) = filtered_results.get(relative_row) {
                if res.path.trim().is_empty() && res.offset.trim().is_empty() {
                    return true;
                }
                self.show_manifest_delete_confirm = true;
                self.manifest_delete_image_label = res.label.clone();
                self.log(format!(
                    "Delete confirmation opened for manifest image '{}'.",
                    res.label
                ));
            }
            return true;
        }

        if self.active_tab == ActiveTab::Serial {
            if self.is_inside_rect(col, row, self.layout_zones.serial_port_info) {
                let idx = row.saturating_sub(self.layout_zones.serial_port_info.y + 1) as usize;
                if idx == 1 {
                    self.cycle_serial_frame_format();
                    return true;
                }
            }
        }
        true
    }

    pub fn handle_mouse_click(
        &mut self,
        col: u16,
        row: u16,
        tx: tokio::sync::mpsc::Sender<WorkerMessage>,
    ) -> bool {
        if self.show_port_menu {
            if !self.is_inside_rect(col, row, self.layout_zones.port_menu_modal) {
                self.show_port_menu = false;
            } else {
                let relative_row =
                    row.saturating_sub(self.layout_zones.port_menu_modal.y + 1) as usize;
                let visible_indices = self.visible_port_indices_for_active_tab();
                if let Some(channel_idx) = visible_indices.get(relative_row).copied() {
                    self.selected_channel_idx = channel_idx;
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

        if self.show_manifest_delete_confirm {
            if !self.is_inside_rect(col, row, self.layout_zones.manifest_delete_modal) {
                self.show_manifest_delete_confirm = false;
                self.manifest_delete_image_label.clear();
            }
            return true;
        }

        // Manifest Edit Modal Check
        if self.show_manifest_edit_modal {
            if !self.is_inside_rect(col, row, self.layout_zones.manifest_edit_modal) {
                self.show_manifest_edit_modal = false;
            }
            return true;
        }

        // File Picker Modal Check
        if self.show_file_picker {
            if !self.is_inside_rect(col, row, self.layout_zones.file_picker_modal) {
                self.show_file_picker = false;
            } else if self.is_inside_rect(col, row, self.layout_zones.file_picker_table) {
                let rect = self.layout_zones.file_picker_table;
                let relative_row = row.saturating_sub(rect.y + 1) as usize;
                if relative_row < self.file_picker_items.len() {
                    if self.file_picker_selected_idx == relative_row {
                        self.select_file_picker_item();
                    } else {
                        self.file_picker_selected_idx = relative_row;
                    }
                }
            }
            return true;
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

        if self.active_tab == ActiveTab::Flasher {
            if self.is_inside_rect(col, row, self.layout_zones.flash_mode_toggle) {
                if self.manifest_locked {
                    self.log(
                        "Firmware manifest is locked; click the status bar to unlock before changing flash mode.",
                    );
                    return true;
                }
                self.toggle_merged_flash();
                return true;
            }
            if self.is_inside_rect(col, row, self.layout_zones.flash_donotchg_toggle) {
                if self.manifest_locked {
                    self.log(
                        "Firmware manifest is locked; click the status bar to unlock before changing DoNotChgBin.",
                    );
                    return true;
                }
                self.toggle_do_not_chg_bin();
                return true;
            }
            if self.is_inside_rect(col, row, self.layout_zones.flash_manifest_status) {
                self.toggle_manifest_lock();
                return true;
            }

            if self.channels.is_empty() {
                if self.is_inside_rect(col, row, self.layout_zones.flash_empty_state) {
                    self.scan_ports(Some(tx.clone()));
                    self.log("Manual port scan requested from flasher empty state.");
                    return true;
                }
            }

            if self.is_inside_rect(col, row, self.layout_zones.flash_summary) {
                if let Some(action) = self.flash_summary_action_at(col, row) {
                    match action {
                        0 => {
                            if self.flash_batch_mode {
                                self.start_flashing(tx.clone());
                            } else {
                                self.start_flashing_selected(tx.clone());
                            }
                        }
                        1 => {
                            self.flash_batch_mode = !self.flash_batch_mode;
                            self.log(format!(
                                "Flash Mode set to: {}",
                                if self.flash_batch_mode {
                                    "BATCH"
                                } else {
                                    "SINGLE"
                                }
                            ));
                        }
                        2 => {
                            self.auto_flash = !self.auto_flash;
                            self.log(format!(
                                "Auto-Flash mode: {}.",
                                if self.auto_flash {
                                    "ENABLED"
                                } else {
                                    "DISABLED"
                                }
                            ));
                        }
                        3 => {
                            if self.is_flashing {
                                self.log("Cannot clear statistics while flashing is active.");
                            } else {
                                self.stats.total_passed = 0;
                                self.stats.total_failed = 0;
                                self.stats.total_attempted = 0;
                                self.log("Production counters cleared.");
                            }
                        }
                        _ => {}
                    }
                }
                return true;
            }

            if self.is_inside_rect(col, row, self.layout_zones.flash_device_table) {
                if let Some(idx) = self.flash_table_row_at(row) {
                    self.selected_channel_idx = idx;
                    if let Some(port) = self.get_selected_port() {
                        self.log(format!("Selected flash channel: {}.", port));
                    }
                }
                return true;
            }

            if self.is_inside_rect(col, row, self.layout_zones.flash_manifest_table) {
                if self.manifest_locked {
                    self.log(
                        "Firmware manifest is locked; click the status bar to unlock before editing.",
                    );
                    return true;
                }

                let rect = self.layout_zones.flash_manifest_table;
                let relative_row = row.saturating_sub(rect.y + 1) as usize;

                let filtered_results = self.config.manifest_results_for_mode(self.use_merged_flash);

                if relative_row < filtered_results.len() {
                    let res = &filtered_results[relative_row];
                    let relative_col = col.saturating_sub(rect.x);

                    if relative_col < 11 {
                        self.show_manifest_edit_modal = true;
                        self.manifest_edit_image_label = res.label.clone();
                        self.manifest_edit_is_offset = true;
                        self.manifest_edit_input = res.offset.clone();
                        return true;
                    } else if relative_col >= 11 && rect.width.saturating_sub(relative_col) >= 32 {
                        self.show_file_picker = true;
                        self.file_picker_image_label = res.label.clone();
                        self.file_picker_search_input.clear();

                        let path = std::path::Path::new(&res.path);
                        if path.is_absolute() {
                            self.file_picker_current_dir = path
                                .parent()
                                .map(|p| p.to_path_buf())
                                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
                        } else {
                            self.file_picker_current_dir =
                                std::env::current_dir().unwrap_or_default();
                        }
                        self.file_picker_selected_idx = 0;
                        self.refresh_file_picker_items();
                        return true;
                    }
                }
            }
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
            let visible_indices = self.visible_port_indices_for_active_tab();
            if let Some(idx) = visible_indices.get(relative_row as usize).copied() {
                self.selected_channel_idx = idx;
            }
            return true;
        }

        // 3. Serial Settings & Toggles Check
        if self.active_tab == ActiveTab::Serial {
            if self.is_inside_rect(col, row, self.layout_zones.serial_port_info) {
                let click_row = row.saturating_sub(self.layout_zones.serial_port_info.y + 1);
                if click_row == 0 {
                    self.show_port_menu = true;
                    self.port_menu_selected = self.selected_channel_idx;
                } else if click_row == 1 {
                    self.show_custom_baud_modal = true;
                    self.custom_baud_input = self.serial_baud_rate.to_string();
                }
                return true;
            }

            if self.is_inside_rect(col, row, self.layout_zones.serial_options) {
                if let Some(option_idx) =
                    crate::ui::serial::serial_option_at(self.layout_zones.serial_options, col, row)
                {
                    match option_idx {
                        0 => self.toggle_serial_monitor(),
                        1 => {
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
                        2 => {
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
                        3 => {
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
                        4 => {
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
                        5 => {
                            self.serial_auto_reply_enabled = !self.serial_auto_reply_enabled;
                            if self.serial_auto_reply_enabled
                                && self.serial_auto_reply_pattern.is_empty()
                            {
                                self.show_auto_reply_modal = true;
                                self.auto_reply_pattern_input =
                                    self.serial_auto_reply_pattern.clone();
                                self.auto_reply_response_input =
                                    self.serial_auto_reply_response.clone();
                                self.auto_reply_focused_field = 0;
                            }
                            self.log(format!(
                                "Auto Reply: {}",
                                if self.serial_auto_reply_enabled {
                                    "ENABLED"
                                } else {
                                    "DISABLED"
                                }
                            ));
                        }
                        6 => self.toggle_serial_recording(),
                        7 => {
                            if self.serial_playback_active {
                                self.stop_serial_timeline_playback();
                            } else {
                                self.start_serial_timeline_playback();
                            }
                        }
                        8 => self.toggle_dtr(),
                        9 => self.toggle_rts(),
                        _ => {}
                    }
                }
                return true;
            }

            if self.is_inside_rect(col, row, self.layout_zones.serial_quick_commands) {
                let command_row = self.serial_quick_scroll
                    + row.saturating_sub(self.layout_zones.serial_quick_commands.y + 2) as usize;
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
                    if idx <= 2 {
                        self.hover_serial_port_info = Some(idx);
                    }
                    return;
                }

                if self.is_inside_rect(col, row, self.layout_zones.serial_options) {
                    if let Some(idx) = crate::ui::serial::serial_option_at(
                        self.layout_zones.serial_options,
                        col,
                        row,
                    ) {
                        self.hover_serial_option = Some(idx);
                    }
                    return;
                }

                if self.is_inside_rect(col, row, self.layout_zones.serial_quick_commands) {
                    let idx = self.serial_quick_scroll
                        + row.saturating_sub(self.layout_zones.serial_quick_commands.y + 2)
                            as usize;
                    if idx < serial_quick_commands().len() {
                        self.hover_serial_quick_command = Some(idx);
                    }
                }
            }
            ActiveTab::Flasher => {
                if self.is_inside_rect(col, row, self.layout_zones.flash_summary) {
                    self.hover_flash_action = self.flash_summary_action_at(col, row);
                    return;
                }

                if self.is_inside_rect(col, row, self.layout_zones.flash_device_table) {
                    self.hover_flash_row = self.flash_table_row_at(row);
                    return;
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
                    self.hover_dashboard_empty_action =
                        empty_dashboard_action_at(self.layout_zones.monitor_panel, col, row);
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
        self.hover_flash_row = None;
        self.hover_flash_action = None;
        self.hover_plotter_header_action = None;
        self.hover_plotter_quick_command = None;
        self.hover_dashboard_empty_action = None;
        self.hover_widget_control = None;
    }

    pub fn handle_mouse_scroll(&mut self, up: bool, col: u16, row: u16) {
        match self.active_tab {
            ActiveTab::Serial => {
                if self.is_inside_rect(col, row, self.layout_zones.serial_quick_commands) {
                    let visible_rows = self
                        .layout_zones
                        .serial_quick_commands
                        .height
                        .saturating_sub(3) as usize;
                    let max_scroll = serial_quick_commands().len().saturating_sub(visible_rows);
                    if up {
                        self.serial_quick_scroll = self.serial_quick_scroll.saturating_sub(1);
                    } else {
                        self.serial_quick_scroll =
                            self.serial_quick_scroll.saturating_add(1).min(max_scroll);
                    }
                    self.hover_serial_quick_command = None;
                }
            }
            ActiveTab::Flasher => {
                if self.is_inside_rect(col, row, self.layout_zones.flash_device_table) {
                    let visible_rows = self
                        .layout_zones
                        .flash_device_table
                        .height
                        .saturating_sub(3) as usize;
                    let max_scroll = self.channels.len().saturating_sub(visible_rows);
                    if up {
                        self.flash_table_scroll = self.flash_table_scroll.saturating_sub(1);
                    } else {
                        self.flash_table_scroll =
                            self.flash_table_scroll.saturating_add(1).min(max_scroll);
                    }
                    self.hover_flash_row = self.flash_table_row_at(row);
                }
            }
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
        }
    }

    pub fn toggle_dtr(&mut self) {
        self.serial_dtr_active = !self.serial_dtr_active;
        self.log(format!(
            "DTR Pin set to {}",
            if self.serial_dtr_active {
                "HIGH"
            } else {
                "LOW"
            }
        ));
        let port = self
            .get_selected_serial_port()
            .unwrap_or_else(|| "NONE".to_string());
        if let Some(sender) = self.serial_tx_senders.get(&port).cloned() {
            let _ = sender.send(crate::worker::MonitorCommand::SetDtr(
                self.serial_dtr_active,
            ));
        }
    }

    pub fn toggle_rts(&mut self) {
        self.serial_rts_active = !self.serial_rts_active;
        self.log(format!(
            "RTS Pin set to {}",
            if self.serial_rts_active {
                "HIGH"
            } else {
                "LOW"
            }
        ));
        let port = self
            .get_selected_serial_port()
            .unwrap_or_else(|| "NONE".to_string());
        if let Some(sender) = self.serial_tx_senders.get(&port).cloned() {
            let _ = sender.send(crate::worker::MonitorCommand::SetRts(
                self.serial_rts_active,
            ));
        }
    }

    pub fn apply_custom_baud_rate(&mut self) {
        if let Ok(parsed_baud) = self.custom_baud_input.trim().parse::<u32>() {
            self.serial_baud_rate = parsed_baud;
            if let Some(port) = self.get_selected_serial_port() {
                let had_monitor = self.serial_tx_senders.remove(&port).is_some()
                    || self.serial_monitor_baud_rates.remove(&port).is_some()
                    || self.serial_pending_monitors.remove(&port).is_some();
                if had_monitor {
                    self.show_serial_notice(
                        format!("Restarting {} at {} bps", port, self.serial_baud_rate),
                        SerialNoticeKind::Info,
                    );
                }
            }
            self.log(format!("Baud rate set to {} bps.", self.serial_baud_rate));
            self.show_custom_baud_modal = false;
        } else {
            self.log("Invalid custom baud rate entered.");
        }
    }

    pub fn toggle_manifest_lock(&mut self) {
        self.manifest_locked = !self.manifest_locked;
        self.config.manifest_locked = self.manifest_locked;
        let _ = self.config.save_to_file(&self.config_path);
        if self.manifest_locked {
            self.show_manifest_edit_modal = false;
            self.show_file_picker = false;
            self.show_manifest_delete_confirm = false;
            self.manifest_delete_image_label.clear();
            self.log("Firmware manifest locked.");
        } else {
            self.log("Firmware manifest unlocked.");
        }
    }

    pub fn confirm_manifest_delete(&mut self) {
        if self.manifest_locked {
            self.log("Firmware manifest is locked; unlock before deleting.");
            self.show_manifest_delete_confirm = false;
            self.manifest_delete_image_label.clear();
            return;
        }

        let label = self.manifest_delete_image_label.clone();
        if label.trim().is_empty() {
            self.show_manifest_delete_confirm = false;
            return;
        }

        if self.config.clear_manifest_image(&label) {
            self.config.sync_images_to_flat_fields();
            let _ = self.config.save_to_file(&self.config_path);
            self.log(format!("Deleted manifest image '{}'.", label));
        }

        self.show_manifest_delete_confirm = false;
        self.manifest_delete_image_label.clear();
    }

    pub fn save_manifest_edit(&mut self) {
        if self.manifest_locked {
            self.log("Firmware manifest is locked; unlock before saving changes.");
            self.show_manifest_edit_modal = false;
            return;
        }

        let label = self.manifest_edit_image_label.clone();
        let value = self.manifest_edit_input.trim().to_string();
        let is_offset = self.manifest_edit_is_offset;

        {
            let img = self.config.ensure_manifest_image(&label);
            if is_offset {
                img.offset = value.clone();
            } else {
                img.path = value.clone();
            }
            img.required = !img.path.trim().is_empty();
        }

        if is_offset {
            self.log(format!(
                "Updated manifest offset for '{}' to {}",
                label, value
            ));
        } else {
            self.log(format!(
                "Updated manifest path for '{}' to {}",
                label, value
            ));
        }

        self.config.sync_images_to_flat_fields();
        let _ = self.config.save_to_file(&self.config_path);
        self.show_manifest_edit_modal = false;
    }

    pub fn refresh_file_picker_items(&mut self) {
        let mut items = Vec::new();

        // 1. Add ".." (Parent directory) if parent exists
        if let Some(parent) = self.file_picker_current_dir.parent() {
            items.push(FilePickerItem {
                name: "..".to_string(),
                path: parent.to_path_buf(),
                is_dir: true,
                size_str: "<DIR>".to_string(),
            });
        }

        // 2. Read directory contents
        if let Ok(entries) = std::fs::read_dir(&self.file_picker_current_dir) {
            let mut dirs = Vec::new();
            let mut files = Vec::new();

            for entry in entries.flatten() {
                let path = entry.path();
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();

                if name.starts_with('.') {
                    // Skip hidden files/directories
                    continue;
                }

                // Filter by search input if not empty
                if !self.file_picker_search_input.is_empty() {
                    let search_lower = self.file_picker_search_input.to_lowercase();
                    if !name.to_lowercase().contains(&search_lower) {
                        continue;
                    }
                }

                let is_dir = path.is_dir();
                let size_str = if is_dir {
                    "<DIR>".to_string()
                } else if let Ok(meta) = entry.metadata() {
                    let sz = meta.len() as usize;
                    if sz >= 1024 * 1024 {
                        format!("{:.1}MB", sz as f64 / 1024.0 / 1024.0)
                    } else if sz >= 1024 {
                        format!("{:.1}KB", sz as f64 / 1024.0)
                    } else if sz == 0 {
                        "-".to_string()
                    } else {
                        format!("{}B", sz)
                    }
                } else {
                    "-".to_string()
                };

                let item = FilePickerItem {
                    name,
                    path,
                    is_dir,
                    size_str,
                };
                if is_dir {
                    dirs.push(item);
                } else {
                    files.push(item);
                }
            }

            // Sort directories and files alphabetically
            dirs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            items.extend(dirs);
            items.extend(files);
        }

        self.file_picker_items = items;
        self.file_picker_selected_idx = self
            .file_picker_selected_idx
            .min(self.file_picker_items.len().saturating_sub(1));
    }

    pub fn select_file_picker_item(&mut self) {
        if self.manifest_locked {
            self.log("Firmware manifest is locked; unlock before selecting files.");
            self.show_file_picker = false;
            return;
        }

        if self.file_picker_selected_idx >= self.file_picker_items.len() {
            return;
        }
        let item = self.file_picker_items[self.file_picker_selected_idx].clone();
        if item.is_dir {
            // Open directory
            self.file_picker_current_dir = item.path;
            self.file_picker_search_input.clear();
            self.file_picker_selected_idx = 0;
            self.refresh_file_picker_items();
        } else {
            // Select file and close
            let label = self.file_picker_image_label.clone();
            let value = item.path.to_string_lossy().to_string();

            {
                let img = self.config.ensure_manifest_image(&label);
                img.path = value.clone();
                img.required = !img.path.trim().is_empty();
            }
            self.log(format!(
                "Updated manifest path for '{}' to {}",
                label, value
            ));

            self.config.sync_images_to_flat_fields();
            let _ = self.config.save_to_file(&self.config_path);
            self.show_file_picker = false;
        }
    }

    pub fn save_auto_reply(&mut self) {
        self.serial_auto_reply_pattern = self.auto_reply_pattern_input.clone();
        self.serial_auto_reply_response = self.auto_reply_response_input.clone();
        self.serial_auto_reply_enabled = true;
        self.show_auto_reply_modal = false;
        self.log(format!(
            "Auto-Reply configured: Pattern='{}', Response='{}'",
            self.serial_auto_reply_pattern, self.serial_auto_reply_response
        ));
    }

    pub fn start_auto_baud_detection(&mut self) {
        let port = if let Some(p) = self.get_selected_serial_port() {
            p
        } else {
            self.log("No serial port selected for auto-baud detection.");
            return;
        };

        if self.serial_auto_baud_scanning {
            self.log("Auto-baud scan is already running.");
            return;
        }

        self.serial_auto_baud_scanning = true;
        self.show_serial_notice("Detecting Baud...".to_string(), SerialNoticeKind::Info);
        self.log(format!("[{}] Starting auto-baud rate scanning...", port));

        let app_tx = self.worker_tx.clone();
        let frame_format = self.serial_frame_format.clone();
        let port_clone = port.clone();

        tokio::spawn(async move {
            let rates = [9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600];
            let mut best_baud = 115200;
            let mut best_score = 0.0;

            for &baud in &rates {
                if let Some(ref tx) = app_tx {
                    let _ = tx
                        .send(WorkerMessage::Log {
                            port: port_clone.clone(),
                            message: format!("Scanning at {} bps...", baud),
                        })
                        .await;
                }

                let mut port_builder =
                    serialport::new(&port_clone, baud).timeout(Duration::from_millis(150));
                if let Some(db_char) = frame_format.chars().next() {
                    match db_char {
                        '5' => port_builder = port_builder.data_bits(serialport::DataBits::Five),
                        '6' => port_builder = port_builder.data_bits(serialport::DataBits::Six),
                        '7' => port_builder = port_builder.data_bits(serialport::DataBits::Seven),
                        _ => port_builder = port_builder.data_bits(serialport::DataBits::Eight),
                    }
                }
                if let Some(par_char) = frame_format.chars().nth(2) {
                    match par_char {
                        'E' | 'e' => port_builder = port_builder.parity(serialport::Parity::Even),
                        'O' | 'o' => port_builder = port_builder.parity(serialport::Parity::Odd),
                        _ => port_builder = port_builder.parity(serialport::Parity::None),
                    }
                }
                if let Some(sb_char) = frame_format.chars().nth(4) {
                    match sb_char {
                        '2' => port_builder = port_builder.stop_bits(serialport::StopBits::Two),
                        _ => port_builder = port_builder.stop_bits(serialport::StopBits::One),
                    }
                }

                if let Ok(mut sport) = port_builder.open_native() {
                    let mut read_buf = [0u8; 128];
                    let start = std::time::Instant::now();
                    let mut bytes_read = 0;

                    while start.elapsed() < Duration::from_millis(150) {
                        if let Ok(n) = sport.read(&mut read_buf) {
                            if n > 0 {
                                bytes_read = n;
                                break;
                            }
                        }
                    }

                    if bytes_read > 0 {
                        let score = score_printable_ratio(&read_buf[..bytes_read]);
                        if score > best_score {
                            best_score = score;
                            best_baud = baud;
                        }
                        if score > 0.92 {
                            break;
                        }
                    }
                }
            }

            if let Some(ref tx) = app_tx {
                let _ = tx
                    .send(WorkerMessage::BaudDetected {
                        port: port_clone.clone(),
                        baud_rate: best_baud,
                    })
                    .await;
            }
        });
    }

    fn is_inside_rect(&self, col: u16, row: u16, rect: Rect) -> bool {
        col >= rect.x
            && col < (rect.x + rect.width)
            && row >= rect.y
            && row < (rect.y + rect.height)
    }
}

fn score_printable_ratio(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut printable = 0;
    let mut nulls = 0;
    let mut control = 0;
    for &b in data {
        if b == 0 {
            nulls += 1;
        } else if b == 0x0A || b == 0x0D || b == 0x09 {
            control += 1;
        } else if b >= 0x20 && b <= 0x7E {
            printable += 1;
        }
    }
    let score = (printable as f64 * 0.6 + control as f64 * 0.3) - (nulls as f64 * 0.2);
    score.max(0.0)
}

fn unescape_string(s: &str) -> String {
    s.replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t")
}

fn make_trace_id(port: &str, attempt: u32) -> String {
    let sanitized_port: String = port
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect();
    format!("TRACE-{}-{:06}", sanitized_port, attempt)
}

fn open_session_log_file() -> (Option<File>, Option<std::path::PathBuf>, Option<String>) {
    let traces_dir = std::path::PathBuf::from(".piopulse").join("runs");
    if let Err(e) = std::fs::create_dir_all(&traces_dir) {
        return (
            None,
            None,
            Some(format!("failed to create {}: {}", traces_dir.display(), e)),
        );
    }

    let filename = format!(
        "piopulse-{}.txt",
        chrono::Local::now().format("%Y%m%d-%H%M%S")
    );
    let path = traces_dir.join(filename);
    match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(mut file) => {
            let _ = writeln!(
                file,
                "PioPulse session started at {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
            );
            let _ = writeln!(
                file,
                "cwd={}",
                std::env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "?".to_string())
            );
            let _ = writeln!(file, "version={}", env!("CARGO_PKG_VERSION"));
            let _ = writeln!(file, "---");
            (Some(file), Some(path), None)
        }
        Err(e) => (
            None,
            Some(path.clone()),
            Some(format!("failed to open {}: {}", path.display(), e)),
        ),
    }
}

fn manifest_has_errors(config: &ProjectConfig) -> bool {
    !config.validate_manifest().1.is_empty()
}

fn manifest_needs_platformio_refresh(config: &ProjectConfig) -> bool {
    std::path::Path::new("platformio.ini").exists()
        && (config.chip_type == "Auto" || manifest_has_empty_required_image(config))
}

fn manifest_has_empty_required_image(config: &ProjectConfig) -> bool {
    config.images.iter().any(|img| {
        img.required
            && !img.path.trim().is_empty()
            && std::fs::metadata(&img.path)
                .map(|metadata| metadata.len() == 0)
                .unwrap_or(false)
    })
}

fn create_factory_manifest_from_existing_artifacts(
    base_config: Option<&ProjectConfig>,
    factory_dir: &std::path::Path,
    manifest_path: &std::path::Path,
) -> Option<ProjectConfig> {
    if !factory_dir.is_dir() {
        return None;
    }

    let segmented_files = [
        ("bootloader", "bootloader.bin", "0x0000"),
        ("partitions", "partitions.bin", "0x8000"),
        ("boot_app0", "boot_app0.bin", "0xe000"),
        ("firmware", "firmware.bin", "0x10000"),
    ];
    let has_segmented = segmented_files
        .iter()
        .all(|(_, filename, _)| factory_dir.join(filename).is_file());
    let merged_path = factory_dir.join("factory_merged.bin");
    let mut has_merged = merged_path.is_file();

    if !has_segmented && !has_merged {
        return None;
    }

    if has_segmented && !has_merged {
        let merged_result = create_merged_flash_image(
            &[
                (0x0000, &factory_dir.join("bootloader.bin")),
                (0x8000, &factory_dir.join("partitions.bin")),
                (0xe000, &factory_dir.join("boot_app0.bin")),
                (0x10000, &factory_dir.join("firmware.bin")),
            ],
            &merged_path,
        );
        has_merged = merged_result.is_ok() && merged_path.is_file();
    }

    let mut config = base_config.cloned().unwrap_or_default();
    config.images.clear();
    config.merged_offset = "0x0000".to_string();
    config.flash_encryption_mode = if config.flash_encryption {
        "device_runtime".to_string()
    } else {
        "disabled".to_string()
    };

    if has_segmented {
        for (label, filename, offset) in segmented_files {
            config.images.push(FirmwareImage {
                label: label.to_string(),
                path: filename.to_string(),
                offset: offset.to_string(),
                required: true,
                encrypted: false,
                sha256: None,
            });
        }

        config.bootloader_path = "bootloader.bin".to_string();
        config.bootloader_offset = "0x0000".to_string();
        config.partitions_path = "partitions.bin".to_string();
        config.partitions_offset = "0x8000".to_string();
        config.otadata_path = "boot_app0.bin".to_string();
        config.otadata_offset = "0xe000".to_string();
        config.app_path = "firmware.bin".to_string();
        config.app_offset = "0x10000".to_string();
    } else {
        config.bootloader_path.clear();
        config.partitions_path.clear();
        config.otadata_path.clear();
        config.app_path.clear();
    }

    if has_merged {
        config.images.push(FirmwareImage {
            label: "factory_merged".to_string(),
            path: "factory_merged.bin".to_string(),
            offset: config.merged_offset.clone(),
            required: true,
            encrypted: false,
            sha256: None,
        });
    }

    config.use_merged_flash = has_merged;
    config.save_to_file(manifest_path).ok()?;
    ProjectConfig::load_from_file(manifest_path).ok()
}

fn format_platformio_build_error(err: &str) -> String {
    if err.contains("bootloader_") && err.contains(".elf") && err.contains("not found") {
        return format!(
            "{}\n诊断: PlatformIO 指定的 bootloader ELF 在当前 framework 包中不存在。请检查 platformio.ini 的 board_build.f_flash / board_build.flash_mode / board_build.arduino.custom_bootloader，或更新 espressif32/framework-arduinoespressif32。",
            err
        );
    }
    err.to_string()
}

#[allow(dead_code)]
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
    let titles = crate::ui::tab_titles_for_width(lang, tabs_area.width);

    let mut current_x = 0;
    for (idx, title) in titles.iter().enumerate() {
        let width = UnicodeWidthStr::width(*title);
        if (current_x..current_x + width).contains(&relative_col) {
            return Some(idx);
        }
        current_x += width + 3;
    }

    None
}

pub fn plotter_header_items(
    lang: &str,
    compact: bool,
    selected_port: String,
    protocol: String,
    view: String,
    state: String,
) -> [(String, String); 4] {
    let labels = if compact && lang == "zh" {
        ["端口", "协议(M)", "视图(V)", "状态(S)"]
    } else if compact {
        ["Port", "Proto(M)", "View(V)", "State(S)"]
    } else if lang == "zh" {
        ["端口", "协议", "视图", "状态"]
    } else {
        ["Port", "Protocol", "View", "State"]
    };

    [
        (labels[0].to_string(), selected_port),
        (labels[1].to_string(), protocol),
        (labels[2].to_string(), view),
        (labels[3].to_string(), state),
    ]
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
    fn test_sidebar_rendering() {
        let mut app = App::new("test_piopulse.toml".to_string());
        app.tool_config.language = "en".to_string();
        app.layout_zones.flash_summary = ratatui::layout::Rect::new(0, 0, 40, 8);

        let rects = app.get_flash_summary_button_rects();
        assert_eq!(rects.len(), 4);
        assert_eq!(rects[0].width, 16);
        assert_eq!(rects[0].height, 1);
        assert_eq!(rects[2].width, 16);
        assert_eq!(rects[2].height, 1);

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_port_menu_toggle_and_select() {
        // Use a temporary path or mock path
        let mut app = App::new("test_piopulse.toml".to_string());

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
        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_exit_menu_buttons_are_mouse_clickable() {
        let mut app = App::new("test_piopulse.toml".to_string());
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

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_exit_menu_buttons_select_on_mouse_hover() {
        let mut app = App::new("test_piopulse.toml".to_string());
        app.show_exit_menu = true;
        app.layout_zones.exit_menu_modal = ratatui::layout::Rect::new(10, 5, 48, 11);

        app.exit_menu_selected = 1;
        app.handle_mouse_move(16, 9);
        assert_eq!(app.exit_menu_selected, 0);

        app.handle_mouse_move(44, 9);
        assert_eq!(app.exit_menu_selected, 1);

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_serial_buttons_select_on_mouse_hover() {
        let mut app = App::new("test_piopulse.toml".to_string());
        app.active_tab = ActiveTab::Serial;
        app.layout_zones.serial_port_info = ratatui::layout::Rect::new(10, 5, 20, 7);
        app.layout_zones.serial_options = ratatui::layout::Rect::new(30, 5, 20, 8);
        app.layout_zones.serial_quick_commands = ratatui::layout::Rect::new(50, 5, 30, 16);

        app.handle_mouse_move(12, 6);
        assert_eq!(app.hover_serial_port_info, Some(0));

        app.handle_mouse_move(12, 8);
        assert_eq!(app.hover_serial_port_info, Some(2));

        app.handle_mouse_move(32, 7);
        assert_eq!(app.hover_serial_option, Some(2));

        app.handle_mouse_move(52, 10);
        assert_eq!(app.hover_serial_quick_command, Some(3));

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_serial_compact_option_grid_clicks_columns() {
        let mut app = App::new("test_piopulse.toml".to_string());
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        app.active_tab = ActiveTab::Serial;
        app.channels = vec![Channel::new(crate::worker::DetectedPort {
            name: "COM3".to_string(),
            vid: None,
            pid: None,
            product: Some("USB Serial".to_string()),
            manufacturer: None,
        })];
        app.layout_zones.serial_options = ratatui::layout::Rect::new(30, 5, 24, 8);

        assert!(!app.serial_monitor_enabled);
        assert!(app.handle_mouse_click(32, 6, tx.clone()));
        assert!(app.serial_monitor_enabled);

        assert!(app.serial_add_newline);
        assert!(app.handle_mouse_click(32, 7, tx.clone()));
        assert!(!app.serial_add_newline);

        assert!(!app.serial_hex_mode_rx);
        assert!(app.handle_mouse_click(45, 7, tx));
        assert!(app.serial_hex_mode_rx);

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_serial_option_hitbox_matches_rendered_columns() {
        let area = ratatui::layout::Rect::new(30, 5, 24, 8);

        assert_eq!(crate::ui::serial::serial_option_at(area, 31, 6), Some(0));
        assert_eq!(crate::ui::serial::serial_option_at(area, 41, 6), Some(0));
        assert_eq!(crate::ui::serial::serial_option_at(area, 42, 6), Some(1));
        assert_eq!(crate::ui::serial::serial_option_at(area, 52, 6), Some(1));

        assert_eq!(crate::ui::serial::serial_option_at(area, 30, 6), None);
        assert_eq!(crate::ui::serial::serial_option_at(area, 31, 5), None);
        assert_eq!(crate::ui::serial::serial_option_at(area, 31, 10), Some(8));
        assert_eq!(crate::ui::serial::serial_option_at(area, 31, 11), None);
    }

    #[test]
    fn test_serial_monitor_stop_sets_tui_notice() {
        let mut app = App::new("test_piopulse.toml".to_string());
        app.serial_monitor_enabled = true;
        let (serial_tx, _serial_rx) = tokio::sync::mpsc::unbounded_channel();
        app.serial_tx_senders.insert("COM3".to_string(), serial_tx);
        app.serial_monitor_baud_rates
            .insert("COM3".to_string(), 115200);

        app.toggle_serial_monitor();

        assert!(!app.serial_monitor_enabled);
        assert!(app.serial_tx_senders.is_empty());
        let notice = app.serial_notice.as_ref().expect("missing serial notice");
        assert_eq!(notice.kind, SerialNoticeKind::Success);
        assert!(notice.message.contains("released"));

        app.serial_notice.as_mut().unwrap().expires_at = Instant::now() - Duration::from_millis(1);
        app.tick();
        assert!(app.serial_notice.is_none());

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_worker_monitor_stopped_sets_release_notice() {
        let mut app = App::new("test_piopulse.toml".to_string());

        app.handle_worker_message(crate::worker::WorkerMessage::MonitorStopped {
            port: "COM4".to_string(),
        });

        let notice = app.serial_notice.as_ref().expect("missing serial notice");
        assert_eq!(notice.kind, SerialNoticeKind::Success);
        assert!(notice.message.contains("COM4"));
        assert!(notice.message.contains("released"));

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_serial_quick_commands_scroll_with_mouse_wheel() {
        let mut app = App::new("test_piopulse.toml".to_string());
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        app.active_tab = ActiveTab::Serial;
        app.layout_zones.serial_quick_commands = ratatui::layout::Rect::new(50, 5, 30, 6);

        app.handle_mouse_scroll(false, 52, 8);
        app.handle_mouse_scroll(false, 52, 8);
        assert_eq!(app.serial_quick_scroll, 2);

        app.handle_mouse_move(52, 7);
        assert_eq!(app.hover_serial_quick_command, Some(2));

        assert!(app.handle_mouse_click(52, 7, tx));
        assert!(app.logs.iter().any(|line| line.contains("[TX] ATI")));

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_flasher_table_scroll_hover_and_click_selects_channel() {
        let mut app = App::new("test_piopulse.toml".to_string());
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        app.active_tab = ActiveTab::Flasher;
        app.layout_zones.flash_device_table = ratatui::layout::Rect::new(0, 5, 80, 7);
        app.channels = (0..8)
            .map(|idx| {
                Channel::new(crate::worker::DetectedPort {
                    name: format!("COM{}", idx),
                    vid: None,
                    pid: None,
                    product: None,
                    manufacturer: None,
                })
            })
            .collect();

        app.handle_mouse_scroll(false, 10, 8);
        app.handle_mouse_scroll(false, 10, 8);
        assert_eq!(app.flash_table_scroll, 2);

        app.handle_mouse_move(10, 7);
        assert_eq!(app.hover_flash_row, Some(2));

        assert!(app.handle_mouse_click(10, 8, tx));
        assert_eq!(app.selected_channel_idx, 3);

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_auto_probe_absent_rearms_finished_station() {
        let mut app = App::new("test_piopulse.toml".to_string());
        app.auto_flash = true;
        app.channels = vec![Channel::new(crate::worker::DetectedPort {
            name: "COM9".to_string(),
            vid: None,
            pid: None,
            product: None,
            manufacturer: None,
        })];
        app.channels[0].finished = true;
        app.channels[0].success = true;
        app.channels[0].auto_flash_armed = false;
        app.channels[0].status = "SUCCESS".to_string();
        app.channels[0].progress = 100;

        app.handle_worker_message(crate::worker::WorkerMessage::AutoProbeResult {
            port: "COM9".to_string(),
            present: false,
            chip: None,
            mac: None,
            error_msg: Some("no bootloader response".to_string()),
        });

        let channel = &app.channels[0];
        assert!(channel.auto_flash_armed);
        assert!(!channel.finished);
        assert_eq!(channel.status, "Waiting for product");
        assert_eq!(app.stats.total_failed, 0);

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_auto_probe_absent_while_armed_does_not_count_failure() {
        let mut app = App::new("test_piopulse.toml".to_string());
        app.auto_flash = true;
        app.channels = vec![Channel::new(crate::worker::DetectedPort {
            name: "COM10".to_string(),
            vid: None,
            pid: None,
            product: None,
            manufacturer: None,
        })];

        app.handle_worker_message(crate::worker::WorkerMessage::AutoProbeResult {
            port: "COM10".to_string(),
            present: false,
            chip: None,
            mac: None,
            error_msg: Some("no product".to_string()),
        });

        let channel = &app.channels[0];
        assert!(channel.auto_flash_armed);
        assert_eq!(channel.status, "Waiting for product");
        assert_eq!(app.stats.total_failed, 0);
        assert_eq!(app.stats.total_attempted, 0);

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_flasher_manifest_table_click_and_edit() {
        let mut app = App::new("test_piopulse.toml".to_string());
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        app.active_tab = ActiveTab::Flasher;

        // Initialize images in app.config
        app.config.images = vec![crate::config::FirmwareImage {
            label: "bootloader".to_string(),
            path: "test_boot.bin".to_string(),
            offset: "0x1000".to_string(),
            required: true,
            encrypted: false,
            sha256: None,
        }];

        // Click coordinates inside layout zone
        app.layout_zones.flash_manifest_table = ratatui::layout::Rect::new(50, 5, 50, 10);

        // Click Offset column
        assert!(app.handle_mouse_click(52, 6, tx.clone()));
        assert!(app.show_manifest_edit_modal);
        assert_eq!(app.manifest_edit_image_label, "bootloader");
        assert!(app.manifest_edit_is_offset);
        assert_eq!(app.manifest_edit_input, "0x1000");

        // Close modal
        app.show_manifest_edit_modal = false;

        // Click File Name column (offset ~ 50 + 15 = 65)
        assert!(app.handle_mouse_click(65, 6, tx));
        assert!(app.show_file_picker);
        assert_eq!(app.file_picker_image_label, "bootloader");

        // Mock selection in picker
        app.file_picker_items = vec![crate::app::FilePickerItem {
            name: "new_boot.bin".to_string(),
            path: std::path::PathBuf::from("new_boot.bin"),
            is_dir: false,
            size_str: "10KB".to_string(),
        }];
        app.file_picker_selected_idx = 0;
        app.select_file_picker_item();

        assert!(!app.show_file_picker);
        assert_eq!(app.config.images[0].path, "new_boot.bin");
        assert_eq!(app.config.bootloader_path, "new_boot.bin"); // flat field synchronized!

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_manifest_right_click_confirms_before_delete() {
        let mut app = App::new("test_piopulse.toml".to_string());
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        app.active_tab = ActiveTab::Flasher;
        app.config.images = vec![crate::config::FirmwareImage {
            label: "bootloader".to_string(),
            path: "test_boot.bin".to_string(),
            offset: "0x1000".to_string(),
            required: true,
            encrypted: false,
            sha256: Some("abc".to_string()),
        }];
        app.config.sync_images_to_flat_fields();
        app.layout_zones.flash_manifest_table = ratatui::layout::Rect::new(50, 5, 50, 10);

        assert!(app.handle_mouse_right_click(52, 6, tx));
        assert!(app.show_manifest_delete_confirm);
        assert_eq!(app.manifest_delete_image_label, "bootloader");
        assert_eq!(app.config.images[0].path, "test_boot.bin");

        app.confirm_manifest_delete();
        assert!(!app.show_manifest_delete_confirm);
        assert_eq!(app.config.images[0].path, "");
        assert_eq!(app.config.images[0].offset, "");
        assert!(!app.config.images[0].required);
        assert!(app.config.images[0].sha256.is_none());
        assert_eq!(app.config.bootloader_path, "");

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_manifest_lock_blocks_edit_delete_and_mode_changes() {
        let mut app = App::new("test_piopulse.toml".to_string());
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        app.active_tab = ActiveTab::Flasher;
        app.config.images = vec![crate::config::FirmwareImage {
            label: "bootloader".to_string(),
            path: "test_boot.bin".to_string(),
            offset: "0x1000".to_string(),
            required: true,
            encrypted: false,
            sha256: None,
        }];
        app.layout_zones.flash_manifest_table = ratatui::layout::Rect::new(50, 5, 50, 10);
        app.layout_zones.flash_manifest_status = ratatui::layout::Rect::new(50, 16, 50, 3);
        app.layout_zones.flash_mode_toggle = ratatui::layout::Rect::new(10, 5, 60, 1);

        assert!(app.handle_mouse_click(52, 16, tx.clone()));
        assert!(app.manifest_locked);

        assert!(app.handle_mouse_click(52, 6, tx.clone()));
        assert!(!app.show_manifest_edit_modal);

        assert!(app.handle_mouse_right_click(52, 6, tx.clone()));
        assert!(!app.show_manifest_delete_confirm);

        let initial_mode = app.use_merged_flash;
        assert!(app.handle_mouse_click(12, 5, tx.clone()));
        assert_eq!(app.use_merged_flash, initial_mode);

        assert!(app.handle_mouse_click(52, 16, tx));
        assert!(!app.manifest_locked);

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_manifest_lock_persists_to_project_config() {
        let config_path = std::env::temp_dir().join(format!(
            "piopulse_manifest_lock_{}.toml",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut config = ProjectConfig::default();
        config.manifest_locked = false;
        config.save_to_file(&config_path).unwrap();

        let mut app = App::new(config_path.to_string_lossy().to_string());
        assert!(!app.manifest_locked);

        app.toggle_manifest_lock();
        assert!(app.manifest_locked);

        let loaded = ProjectConfig::load_from_file(&config_path).unwrap();
        assert!(loaded.manifest_locked);

        let _ = std::fs::remove_file(config_path);
    }

    #[test]
    fn test_flasher_header_click_toggles_mode_and_donotchg() {
        let mut app = App::new("test_piopulse.toml".to_string());
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        app.active_tab = ActiveTab::Flasher;
        app.layout_zones.flash_mode_toggle = ratatui::layout::Rect::new(10, 5, 60, 1);
        app.layout_zones.flash_donotchg_toggle = ratatui::layout::Rect::new(10, 6, 60, 1);

        let initial_mode = app.use_merged_flash;
        assert!(app.handle_mouse_click(12, 5, tx.clone()));
        assert_eq!(app.use_merged_flash, !initial_mode);
        assert_eq!(app.config.use_merged_flash, app.use_merged_flash);

        let initial_donotchg = app.config.do_not_chg_bin;
        assert!(app.handle_mouse_click(12, 6, tx));
        assert_eq!(app.config.do_not_chg_bin, !initial_donotchg);

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_flasher_keyboard_selection_tracks_scroll() {
        let mut app = App::new("test_piopulse.toml".to_string());
        app.active_tab = ActiveTab::Flasher;
        app.layout_zones.flash_device_table = ratatui::layout::Rect::new(0, 5, 80, 7);
        app.channels = (0..8)
            .map(|idx| {
                Channel::new(crate::worker::DetectedPort {
                    name: format!("COM{}", idx),
                    vid: None,
                    pid: None,
                    product: None,
                    manufacturer: None,
                })
            })
            .collect();

        app.move_flash_selection(4);
        assert_eq!(app.selected_channel_idx, 4);
        assert_eq!(app.flash_table_scroll, 1);

        app.move_flash_selection(-2);
        assert_eq!(app.selected_channel_idx, 2);
        assert_eq!(app.flash_table_scroll, 1);

        app.move_flash_selection(-5);
        assert_eq!(app.selected_channel_idx, 0);
        assert_eq!(app.flash_table_scroll, 0);

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_flasher_summary_config_action_is_clickable() {
        let mut app = App::new("test_piopulse.toml".to_string());
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        app.active_tab = ActiveTab::Flasher;
        app.layout_zones.flash_summary = ratatui::layout::Rect::new(0, 0, 40, 10);

        // Test hover / action detection
        // Row 5 Col 10 -> Action 0 (Start Flash)
        assert_eq!(app.flash_summary_action_at(10, 5), Some(0));
        // Row 5 Col 30 -> Action 1 (Toggle Mode)
        assert_eq!(app.flash_summary_action_at(30, 5), Some(1));
        // Row 6 Col 10 -> Action 2 (Auto Flash Toggle)
        assert_eq!(app.flash_summary_action_at(10, 6), Some(2));
        // Row 6 Col 30 -> Action 3 (Clear Stats)
        assert_eq!(app.flash_summary_action_at(30, 6), Some(3));

        // Click on Clear Stats
        assert!(app.handle_mouse_click(30, 6, tx));
        // Since we clicked Action 3, and app.is_flashing is false, stats should be cleared
        assert_eq!(app.stats.total_attempted, 0);

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_dashboard_empty_buttons_select_on_mouse_hover() {
        let mut app = App::new("test_piopulse.toml".to_string());
        app.active_tab = ActiveTab::Widgets;
        app.layout_zones.monitor_panel = ratatui::layout::Rect::new(10, 5, 90, 20);

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

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_dashboard_empty_compact_mode_does_not_add_from_blank_space() {
        let mut app = App::new("test_piopulse.toml".to_string());
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        app.active_tab = ActiveTab::Widgets;
        app.layout_zones.monitor_panel = ratatui::layout::Rect::new(0, 0, 70, 12);

        assert!(app.handle_mouse_click(4, 9, tx));
        assert!(app.dashboard_widgets.is_empty());

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_slider_click_uses_visual_track_columns() {
        let mut app = App::new("test_piopulse.toml".to_string());
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

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_top_tabs_select_on_mouse_hover() {
        use unicode_width::UnicodeWidthStr;

        let mut app = App::new("test_piopulse.toml".to_string());
        app.layout_zones.tabs = ratatui::layout::Rect::new(0, 3, 80, 3);

        app.handle_mouse_move(0, 3);
        assert_eq!(app.hover_tab, Some(0));

        let tab_plot_x =
            UnicodeWidthStr::width(crate::ui::tr("tab_serial", &app.tool_config.language)) + 3;
        app.handle_mouse_move(tab_plot_x as u16, 3);
        assert_eq!(app.hover_tab, Some(1));

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_top_tabs_compact_mode_uses_numbers_only() {
        let mut app = App::new("test_piopulse.toml".to_string());
        app.layout_zones.tabs = ratatui::layout::Rect::new(0, 3, 40, 2);

        assert_eq!(
            crate::ui::tab_titles_for_width(&app.tool_config.language, 40),
            [" [1] ", " [2] ", " [3] ", " [4] ", " [5] "]
        );

        app.handle_mouse_move(8, 3);
        assert_eq!(app.hover_tab, Some(1));

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_top_tabs_tiny_mode_fits_very_narrow_width() {
        let mut app = App::new("test_piopulse.toml".to_string());
        app.layout_zones.tabs = ratatui::layout::Rect::new(0, 3, 32, 2);

        assert_eq!(
            crate::ui::tab_titles_for_width(&app.tool_config.language, 32),
            ["1", "2", "3", "4", "5"]
        );

        app.handle_mouse_move(4, 3);
        assert_eq!(app.hover_tab, Some(1));

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_small_terminal_renders_core_tabs_and_modals() {
        use ratatui::{Terminal, backend::TestBackend};

        for (width, height) in [(40, 12), (52, 14), (64, 18)] {
            for tab in [
                ActiveTab::Serial,
                ActiveTab::Plotter,
                ActiveTab::Widgets,
                ActiveTab::Flasher,
                ActiveTab::Configuration,
            ] {
                let backend = TestBackend::new(width, height);
                let mut terminal = Terminal::new(backend).unwrap();
                let mut app = App::new("test_piopulse.toml".to_string());
                app.splash_ticks_remaining = None;
                app.active_tab = tab;
                app.channels = vec![Channel::new(crate::worker::DetectedPort {
                    name: "COM3".to_string(),
                    vid: None,
                    pid: None,
                    product: None,
                    manufacturer: None,
                })];

                terminal
                    .draw(|frame| crate::ui::draw(frame, &mut app))
                    .unwrap();
            }
        }

        let backend = TestBackend::new(40, 12);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut app = App::new("test_piopulse.toml".to_string());
        app.splash_ticks_remaining = None;
        app.show_exit_menu = true;
        app.show_tool_settings = true;
        app.show_port_menu = true;
        app.show_custom_baud_modal = true;
        app.show_auto_reply_modal = true;
        app.show_manifest_edit_modal = true;
        app.show_file_picker = true;

        terminal
            .draw(|frame| crate::ui::draw(frame, &mut app))
            .unwrap();

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_plotter_header_clicks_use_real_pill_bounds() {
        use unicode_width::UnicodeWidthStr;

        let mut app = App::new("test_piopulse.toml".to_string());
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        app.active_tab = ActiveTab::Plotter;
        app.layout_zones.plotter_header = ratatui::layout::Rect::new(0, 5, 120, 4);

        let lang = app.tool_config.language.clone();
        let items = plotter_header_items(
            &lang,
            true,
            "NONE".to_string(),
            "FireWater".to_string(),
            "Waveform".to_string(),
            if lang == "zh" {
                "运行中".to_string()
            } else {
                "RUNNING".to_string()
            },
        );
        let mut cursor = UnicodeWidthStr::width(crate::ui::tr("plot_title", &lang)) + 2;
        cursor += UnicodeWidthStr::width(format!(" {}: {} ", items[0].0, items[0].1).as_str()) + 2;
        cursor += UnicodeWidthStr::width(format!(" {}: {} ", items[1].0, items[1].1).as_str()) + 2;

        let protocol_before = app.vofa_mode;
        let view_before = app.plotter_mode;
        let view_x = cursor + 1;
        assert!(app.handle_mouse_click(view_x as u16, 5, tx.clone()));
        assert_eq!(app.vofa_mode, protocol_before);
        assert_ne!(app.plotter_mode, view_before);

        let after_view_items = plotter_header_items(
            &lang,
            true,
            "NONE".to_string(),
            "FireWater".to_string(),
            "BarChart".to_string(),
            items[3].1.clone(),
        );
        let view_width = UnicodeWidthStr::width(
            format!(" {}: {} ", after_view_items[2].0, after_view_items[2].1).as_str(),
        );
        let state_x = cursor + view_width + 2 + 1;
        let view_after = app.plotter_mode;
        assert!(app.handle_mouse_click(state_x as u16, 5, tx.clone()));
        assert_eq!(app.plotter_mode, view_after);
        assert!(!app.plotter_active);

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_plotter_header_compact_labels_keep_click_bounds() {
        use unicode_width::UnicodeWidthStr;

        let mut app = App::new("test_piopulse.toml".to_string());
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        app.active_tab = ActiveTab::Plotter;
        app.layout_zones.plotter_header = ratatui::layout::Rect::new(0, 5, 80, 3);

        let lang = &app.tool_config.language;
        let items = plotter_header_items(
            lang,
            true,
            "NONE".to_string(),
            "FireWater".to_string(),
            "Waveform".to_string(),
            if lang == "zh" {
                "运行中".to_string()
            } else {
                "RUNNING".to_string()
            },
        );
        assert!(items[2].0.contains("(V)"));

        let mut cursor = UnicodeWidthStr::width(crate::ui::tr("plot_title", lang)) + 2;
        cursor += UnicodeWidthStr::width(format!(" {}: {} ", items[0].0, items[0].1).as_str()) + 2;
        cursor += UnicodeWidthStr::width(format!(" {}: {} ", items[1].0, items[1].1).as_str()) + 2;

        let view_before = app.plotter_mode;
        assert!(app.handle_mouse_click(cursor as u16 + 1, 5, tx));
        assert_ne!(app.plotter_mode, view_before);

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_plotter_quick_commands_use_real_button_bounds() {
        use unicode_width::UnicodeWidthStr;

        let mut app = App::new("test_piopulse.toml".to_string());
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

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_header_mode_click_opens_sudo_prompt() {
        use unicode_width::UnicodeWidthStr;

        let mut app = App::new("test_piopulse.toml".to_string());
        let (tx, _rx) = tokio::sync::mpsc::channel(1);
        app.layout_zones.header = ratatui::layout::Rect::new(0, 0, 80, 2);

        let mode_x = UnicodeWidthStr::width(
            format!(" ☕ PIOPULSE v{} ", env!("CARGO_PKG_VERSION")).as_str(),
        ) + UnicodeWidthStr::width(" | ")
            + 1;
        assert!(app.handle_mouse_click(mode_x as u16, 0, tx));
        assert!(app.is_entering_password);

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_plotter_view_zoom_pan_and_reset() {
        let mut app = App::new("test_piopulse.toml".to_string());
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

        let _ = std::fs::remove_file("test_piopulse.toml");
    }

    #[test]
    fn test_serial_timeline_recording_and_playback_rebuilds_waveform() {
        let mut app = App::new("test_piopulse.toml".to_string());
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

        let _ = std::fs::remove_file("test_piopulse.toml");
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
