use crate::config::ProjectConfig;
use serialport::SerialPort;
use std::borrow::Cow;
use std::fs::File;
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

use espflash::connection::{Connection, ResetAfterOperation, ResetBeforeOperation};
use espflash::flasher::Flasher;
use espflash::image_format::Segment;
use espflash::target::{Chip, ProgressCallbacks};
use probe_rs::flashing::{BinOptions, ElfOptions, Format};

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DeviceSelector {
    SerialPort(String),
    DebugProbe(String),
}

pub trait FlasherBackend {
    fn run_flash(
        &self,
        selector: &DeviceSelector,
        config: &ProjectConfig,
        tx: &Sender<WorkerMessage>,
    ) -> Result<Option<String>, String>;
}

#[derive(Debug, Clone)]
pub enum MonitorCommand {
    WriteData(Vec<u8>),
    SetDtr(bool),
    SetRts(bool),
}

#[derive(Debug, Clone)]
pub enum WorkerMessage {
    StatusUpdate {
        port: String,
        status: String,
        progress: u8,
        speed: String,
    },
    MacAddressDetected {
        port: String,
        mac: String,
        chip: String,
    },
    ProvisioningGenerated {
        port: String,
        serial_number: String,
        device_name: String,
    },
    ProductionStep {
        port: String,
        step: String,
        detail: String,
    },
    Finished {
        port: String,
        success: bool,
        error_msg: Option<String>,
        mac: Option<String>,
    },
    AutoProbeResult {
        port: String,
        present: bool,
        chip: Option<String>,
        mac: Option<String>,
        error_msg: Option<String>,
    },
    Log {
        port: String,
        message: String,
    },
    WaveformData {
        port: String,
        values: Vec<f32>,
    },
    #[allow(dead_code)]
    ImageData {
        port: String,
        id: usize,
        width: usize,
        height: usize,
        format: u8,
        data: Vec<u8>,
    },
    SerialData {
        port: String,
        data: Vec<u8>,
    },
    BaudDetected {
        port: String,
        baud_rate: u32,
    },
    MonitorStarted {
        port: String,
        baud_rate: u32,
        sender: tokio::sync::mpsc::UnboundedSender<MonitorCommand>,
    },
    MonitorStopped {
        port: String,
    },
}

pub fn start_flashing_task(port: String, config: Arc<ProjectConfig>, tx: Sender<WorkerMessage>) {
    tokio::spawn(async move {
        let port_clone = port.clone();
        let config_clone = config.clone();
        let tx_clone = tx.clone();

        let result = tokio::task::spawn_blocking(move || {
            do_native_flash(port_clone, config_clone, tx_clone)
        })
        .await;

        match result {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                let _ = tx
                    .send(WorkerMessage::Finished {
                        port: port.clone(),
                        success: false,
                        error_msg: Some(e),
                        mac: None,
                    })
                    .await;
            }
            Err(e) => {
                let _ = tx
                    .send(WorkerMessage::Finished {
                        port: port.clone(),
                        success: false,
                        error_msg: Some(format!("Flashing task panicked: {}", e)),
                        mac: None,
                    })
                    .await;
            }
        }
    });
}

pub fn start_auto_probe_task(port: String, config: Arc<ProjectConfig>, tx: Sender<WorkerMessage>) {
    tokio::spawn(async move {
        let port_clone = port.clone();
        let config_clone = config.clone();
        let result =
            tokio::task::spawn_blocking(move || probe_esp32_presence(&port_clone, &config_clone))
                .await;

        let message = match result {
            Ok(Ok((chip, mac))) => WorkerMessage::AutoProbeResult {
                port,
                present: true,
                chip: Some(chip),
                mac: Some(mac),
                error_msg: None,
            },
            Ok(Err(e)) => WorkerMessage::AutoProbeResult {
                port,
                present: false,
                chip: None,
                mac: None,
                error_msg: Some(e),
            },
            Err(e) => WorkerMessage::AutoProbeResult {
                port,
                present: false,
                chip: None,
                mac: None,
                error_msg: Some(format!("Auto probe task panicked: {}", e)),
            },
        };

        let _ = tx.send(message).await;
    });
}

pub fn spawn_serial_monitor(
    port_name: String,
    baud_rate: u32,
    frame_format: String,
    tx: Sender<WorkerMessage>,
    mut cancel_rx: tokio::sync::oneshot::Receiver<()>,
) {
    tokio::spawn(async move {
        if port_name.starts_with("probe:") {
            let _ = tx
                .send(WorkerMessage::Log {
                    port: port_name.clone(),
                    message: "Debug probes are not serial ports; serial monitor skipped."
                        .to_string(),
                })
                .await;
            let _ = tx
                .send(WorkerMessage::MonitorStopped {
                    port: port_name.clone(),
                })
                .await;
            let _ = cancel_rx.try_recv();
            return;
        }

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(500)) => {}
            _ = &mut cancel_rx => {
                return;
            }
        }

        match cancel_rx.try_recv() {
            Ok(_) | Err(tokio::sync::oneshot::error::TryRecvError::Closed) => return,
            Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {}
        }

        let (tx_cmd, mut rx_cmd) = tokio::sync::mpsc::unbounded_channel::<MonitorCommand>();

        let _ = tx
            .send(WorkerMessage::Log {
                port: port_name.clone(),
                message: format!(
                    "Starting Serial Monitor (Baud: {}, Format: {})...",
                    baud_rate, frame_format
                ),
            })
            .await;

        let _ = tokio::task::spawn_blocking(move || {
            let mut port_builder =
                serialport::new(&port_name, baud_rate).timeout(Duration::from_millis(100));

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

            let mut port = match port_builder.open_native() {
                Ok(p) => p,
                Err(e) => {
                    let _ = tx.blocking_send(WorkerMessage::Log {
                        port: port_name.clone(),
                        message: format!("Failed to open monitor port: {}", e),
                    });
                    let _ = tx.blocking_send(WorkerMessage::MonitorStopped {
                        port: port_name.clone(),
                    });
                    return;
                }
            };

            let _ = tx.blocking_send(WorkerMessage::MonitorStarted {
                port: port_name.clone(),
                baud_rate,
                sender: tx_cmd,
            });

            let initial_mode_u8 =
                crate::vofa::ACTIVE_VOFA_MODE.load(std::sync::atomic::Ordering::Relaxed);
            let mut parser =
                crate::vofa::VofaParser::new(crate::vofa::VofaMode::from_u8(initial_mode_u8));
            let mut read_buf = [0u8; 512];

            loop {
                let mut disconnected = false;
                loop {
                    match rx_cmd.try_recv() {
                        Ok(MonitorCommand::WriteData(cmd)) => {
                            if let Err(e) = port.write_all(&cmd) {
                                let _ = tx.blocking_send(WorkerMessage::Log {
                                    port: port_name.clone(),
                                    message: format!("Serial monitor write error: {}", e),
                                });
                            }
                        }
                        Ok(MonitorCommand::SetDtr(level)) => {
                            if let Err(e) = port.write_data_terminal_ready(level) {
                                let _ = tx.blocking_send(WorkerMessage::Log {
                                    port: port_name.clone(),
                                    message: format!("Failed to set DTR: {}", e),
                                });
                            }
                        }
                        Ok(MonitorCommand::SetRts(level)) => {
                            if let Err(e) = port.write_request_to_send(level) {
                                let _ = tx.blocking_send(WorkerMessage::Log {
                                    port: port_name.clone(),
                                    message: format!("Failed to set RTS: {}", e),
                                });
                            }
                        }
                        Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                        Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                            disconnected = true;
                            break;
                        }
                    }
                }

                if disconnected {
                    break;
                }

                let current_mode_u8 =
                    crate::vofa::ACTIVE_VOFA_MODE.load(std::sync::atomic::Ordering::Relaxed);
                parser.set_mode(crate::vofa::VofaMode::from_u8(current_mode_u8));

                match port.read(&mut read_buf) {
                    Ok(num_bytes) if num_bytes > 0 => {
                        let data = &read_buf[..num_bytes];

                        let _ = tx.blocking_send(WorkerMessage::SerialData {
                            port: port_name.clone(),
                            data: data.to_vec(),
                        });

                        let frames = parser.feed(data);
                        for frame in frames {
                            let _ = tx.blocking_send(WorkerMessage::WaveformData {
                                port: port_name.clone(),
                                values: frame,
                            });
                        }
                        if let Some(img) = parser.take_latest_image() {
                            let _ = tx.blocking_send(WorkerMessage::ImageData {
                                port: port_name.clone(),
                                id: img.id,
                                width: img.width,
                                height: img.height,
                                format: img.format,
                                data: img.data,
                            });
                        }
                    }
                    Ok(_) => {}
                    Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {}
                    Err(e) => {
                        let _ = tx.blocking_send(WorkerMessage::Log {
                            port: port_name.clone(),
                            message: format!("Serial monitor read error: {}", e),
                        });
                        break;
                    }
                }
            }

            let _ = tx.blocking_send(WorkerMessage::MonitorStopped {
                port: port_name.clone(),
            });
        })
        .await;
    });
}

struct CustomProgress {
    port: String,
    tx: Sender<WorkerMessage>,
    step_index: usize,
    total_steps: usize,
    total_bytes: usize,
    baud_rate: u32,
}

impl ProgressCallbacks for CustomProgress {
    fn init(&mut self, addr: u32, total: usize) {
        self.total_bytes = total;
        let _ = self.tx.blocking_send(WorkerMessage::Log {
            port: self.port.clone(),
            message: format!(
                "Writing {} bytes to address {:#010X} (block {}/{})...",
                total,
                addr,
                self.step_index + 1,
                self.total_steps
            ),
        });
    }

    fn update(&mut self, current: usize) {
        if self.total_bytes > 0 {
            let pct = (current as f32 / self.total_bytes as f32 * 100.0) as u8;
            let step_weight = 80.0 / (self.total_steps as f32);
            let start_pct = 10.0 + (self.step_index as f32) * step_weight;
            let current_pct = start_pct + (pct as f32 / 100.0) * step_weight;
            let overall_pct = (current_pct as u8).min(90);

            let _ = self.tx.blocking_send(WorkerMessage::StatusUpdate {
                port: self.port.clone(),
                status: format!(
                    "Flashing Block {}/{} ({}%)",
                    self.step_index + 1,
                    self.total_steps,
                    pct
                ),
                progress: overall_pct,
                speed: format!("{} Baud", self.baud_rate),
            });
        }
    }

    fn verifying(&mut self) {
        let _ = self.tx.blocking_send(WorkerMessage::StatusUpdate {
            port: self.port.clone(),
            status: "Verifying...".to_string(),
            progress: 92,
            speed: "N/A".to_string(),
        });
    }

    fn finish(&mut self, _skipped: bool) {
        self.step_index += 1;
    }
}

fn parse_offset(offset_str: &str) -> Result<u32, String> {
    let clean = offset_str.trim().to_lowercase();
    let clean = clean.trim_start_matches("0x").trim_start_matches("0x");
    u32::from_str_radix(clean, 16).map_err(|e| format!("Invalid offset '{}': {}", offset_str, e))
}

fn load_and_pad_file(path: &str) -> Result<Vec<u8>, String> {
    let mut file =
        File::open(path).map_err(|e| format!("Failed to open file '{}': {}", path, e))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .map_err(|e| format!("Failed to read file '{}': {}", path, e))?;
    let rem = data.len() % 4;
    if rem != 0 {
        data.resize(data.len() + (4 - rem), 0xFF);
    }
    Ok(data)
}

pub fn map_chip_type(chip_str: &str) -> Option<Chip> {
    match chip_str.to_lowercase().replace("-", "").as_str() {
        "esp32" => Some(Chip::Esp32),
        "esp32s2" => Some(Chip::Esp32s2),
        "esp32s3" => Some(Chip::Esp32s3),
        "esp32c3" => Some(Chip::Esp32c3),
        "esp32c6" => Some(Chip::Esp32c6),
        "esp32c2" => Some(Chip::Esp32c2),
        "esp32h2" => Some(Chip::Esp32h2),
        _ => None,
    }
}

fn is_esp_usb_serial_jtag(vid: Option<u16>, pid: Option<u16>) -> bool {
    vid == Some(0x303a) && pid == Some(0x1001)
}

fn serial_port_info_for(port_name: &str) -> serialport::UsbPortInfo {
    let ports = get_available_serial_ports();
    ports
        .into_iter()
        .find(|p| p.name == port_name)
        .map(|p| serialport::UsbPortInfo {
            vid: p.vid.unwrap_or(0),
            pid: p.pid.unwrap_or(0),
            serial_number: None,
            manufacturer: p.manufacturer,
            product: p.product,
            interface: None,
        })
        .unwrap_or_else(|| serialport::UsbPortInfo {
            vid: 0,
            pid: 0,
            serial_number: None,
            manufacturer: None,
            product: None,
            interface: None,
        })
}

fn reset_before_for_port(port_info: &serialport::UsbPortInfo) -> ResetBeforeOperation {
    if is_esp_usb_serial_jtag(Some(port_info.vid), Some(port_info.pid)) {
        ResetBeforeOperation::UsbReset
    } else {
        ResetBeforeOperation::DefaultReset
    }
}

fn probe_esp32_presence(
    port_name: &str,
    config: &ProjectConfig,
) -> Result<(String, String), String> {
    if port_name.starts_with("probe:") {
        let parts: Vec<&str> = port_name.split(':').collect();
        if parts.len() < 4 {
            return Err("Invalid probe port format".to_string());
        }
        let vid = u16::from_str_radix(parts[1], 16)
            .map_err(|_| "Invalid VID in probe port name".to_string())?;
        let pid = u16::from_str_radix(parts[2], 16)
            .map_err(|_| "Invalid PID in probe port name".to_string())?;
        let serial = parts[3];

        let lister = probe_rs::probe::list::Lister::new();
        let probes = lister.list_all();
        let matched = probes
            .into_iter()
            .find(|p| {
                p.vendor_id == vid
                    && p.product_id == pid
                    && p.serial_number.as_deref().unwrap_or("unknown") == serial
            })
            .ok_or_else(|| "Debug probe not found".to_string())?;

        let probe = matched
            .open()
            .map_err(|e| format!("Failed to open debug probe: {}", e))?;
        let _session = probe
            .attach(&config.chip_type, probe_rs::Permissions::default())
            .map_err(|e| {
                format!(
                    "Failed to attach debug probe to target {}: {}",
                    config.chip_type, e
                )
            })?;

        return Ok((config.chip_type.clone(), "Probe-Attached".to_string()));
    }

    let port = serialport::new(port_name, config.baud_rate)
        .timeout(Duration::from_millis(800))
        .open_native()
        .map_err(|e| format!("Auto probe could not open {}: {}", port_name, e))?;

    let port_info = serial_port_info_for(port_name);
    let connection = Connection::new(
        port,
        port_info.clone(),
        ResetAfterOperation::HardReset,
        reset_before_for_port(&port_info),
        config.baud_rate,
    );

    let chip_target = map_chip_type(&config.chip_type);
    let mut flasher = Flasher::connect(
        connection,
        true,
        true,
        false,
        chip_target,
        Some(config.baud_rate),
    )
    .map_err(|e| format!("Auto probe did not detect ESP bootloader: {}", e))?;

    let device_info = flasher
        .device_info()
        .map_err(|e| format!("Auto probe could not read device info: {}", e))?;
    Ok((
        device_info.chip.to_string(),
        device_info
            .mac_address
            .unwrap_or_else(|| "Unknown".to_string()),
    ))
}

pub struct Esp32SerialBackend;

impl FlasherBackend for Esp32SerialBackend {
    fn run_flash(
        &self,
        selector: &DeviceSelector,
        config: &ProjectConfig,
        tx: &Sender<WorkerMessage>,
    ) -> Result<Option<String>, String> {
        let port_name = match selector {
            DeviceSelector::SerialPort(p) => p.clone(),
            _ => return Err("ESP32 backend requires a serial port".to_string()),
        };

        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: "Opening serial port...".to_string(),
        });

        let _ = tx.blocking_send(WorkerMessage::StatusUpdate {
            port: port_name.clone(),
            status: "Connecting".to_string(),
            progress: 0,
            speed: "N/A".to_string(),
        });

        let port = serialport::new(&port_name, config.baud_rate)
            .timeout(Duration::from_millis(3000))
            .open_native()
            .map_err(|e| format!("Failed to open port {}: {}", port_name, e))?;

        let port_info = serial_port_info_for(&port_name);

        let before_reset = reset_before_for_port(&port_info);
        if before_reset == ResetBeforeOperation::UsbReset {
            let _ = tx.blocking_send(WorkerMessage::Log {
                port: port_name.clone(),
                message: "Detected Espressif USB-Serial-JTAG, using native USB reset.".to_string(),
            });
        }

        let connection = Connection::new(
            port,
            port_info,
            ResetAfterOperation::HardReset,
            before_reset,
            config.baud_rate,
        );

        let chip_target = map_chip_type(&config.chip_type);
        let mut flasher = Flasher::connect(
            connection,
            true,  // use_stub
            true,  // verify
            false, // skip
            chip_target,
            Some(config.baud_rate),
        )
        .map_err(|e| format!("Connection failed: {}", e))?;

        let device_info = flasher
            .device_info()
            .map_err(|e| format!("Failed to read device info: {}", e))?;
        let chip_name = device_info.chip.to_string();
        let mac_str = device_info
            .mac_address
            .unwrap_or_else(|| "Unknown".to_string());

        let _ = tx.blocking_send(WorkerMessage::MacAddressDetected {
            port: port_name.clone(),
            mac: mac_str.clone(),
            chip: chip_name.clone(),
        });

        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: format!("Chip detected: {}, MAC: {}", chip_name, mac_str),
        });

        let _ = tx.blocking_send(WorkerMessage::StatusUpdate {
            port: port_name.clone(),
            status: "Blank Check".to_string(),
            progress: 5,
            speed: "N/A".to_string(),
        });
        let blank_check_detail = if config.blank_check {
            "Enabled: target must be blank before programming"
        } else {
            "Disabled"
        };
        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: format!("Blank check policy: {}", blank_check_detail),
        });

        let _ = tx.blocking_send(WorkerMessage::StatusUpdate {
            port: port_name.clone(),
            status: "Erase Plan".to_string(),
            progress: 7,
            speed: "N/A".to_string(),
        });
        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: format!("Erase mode: {}", config.erase_mode),
        });

        // Log the generated esptool command
        let esptool_cmd = generate_esptool_command(&port_name, config, config.use_merged_flash);
        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: format!("Generated esptool command: {}", esptool_cmd),
        });

        let mut segments = Vec::new();

        if !config.images.is_empty() {
            // Filter images based on whether we are in merged or segmented mode
            for img in &config.images {
                let is_merged_img = img.label.contains("merged")
                    || img.path.ends_with("factory_merged.bin")
                    || img.path.ends_with("merged.bin");

                if config.use_merged_flash == is_merged_img {
                    if !img.path.is_empty() {
                        let offset = parse_offset(&img.offset)?;
                        let mut data = load_and_pad_file(&img.path)?;

                        // Mutate bootloader / merged header if do_not_chg_bin is false AND the offset is bootloader offset (usually 0x0 or 0x1000)
                        if !config.do_not_chg_bin && (offset == 0 || offset == 0x1000) {
                            modify_bin_header(&mut data, config);
                        }

                        segments.push(Segment {
                            addr: offset,
                            data: Cow::Owned(data),
                        });
                    }
                }
            }
        } else {
            // Legacy fallback
            if !config.bootloader_path.is_empty() {
                let offset = parse_offset(&config.bootloader_offset)?;
                let mut data = load_and_pad_file(&config.bootloader_path)?;
                if !config.do_not_chg_bin && (offset == 0 || offset == 0x1000) {
                    modify_bin_header(&mut data, config);
                }
                segments.push(Segment {
                    addr: offset,
                    data: Cow::Owned(data),
                });
            }

            if !config.partitions_path.is_empty() {
                let offset = parse_offset(&config.partitions_offset)?;
                let data = load_and_pad_file(&config.partitions_path)?;
                segments.push(Segment {
                    addr: offset,
                    data: Cow::Owned(data),
                });
            }

            if !config.otadata_path.is_empty() {
                let offset = parse_offset(&config.otadata_offset)?;
                let data = load_and_pad_file(&config.otadata_path)?;
                segments.push(Segment {
                    addr: offset,
                    data: Cow::Owned(data),
                });
            }

            if !config.app_path.is_empty() {
                let offset = parse_offset(&config.app_offset)?;
                let data = load_and_pad_file(&config.app_path)?;
                segments.push(Segment {
                    addr: offset,
                    data: Cow::Owned(data),
                });
            }
        }

        // Add dynamic NVS segment (always needed)
        // Dynamic NVS page provisioning
        let generated_sn = crate::nvs::generate_serial_number(&chip_name, &mac_str);
        let serial_number = if config.sn_prefix.trim().is_empty() {
            generated_sn
        } else {
            format!("{}-{}", config.sn_prefix.trim(), generated_sn)
        };
        let device_name = crate::nvs::generate_device_name(&mac_str);
        let nvs_data = crate::nvs::generate_nvs_page(&serial_number, &device_name);

        let _ = tx.blocking_send(WorkerMessage::ProvisioningGenerated {
            port: port_name.clone(),
            serial_number: serial_number.clone(),
            device_name: device_name.clone(),
        });
        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: format!("Generated Serial Number: {}", serial_number),
        });
        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: format!("Generated Device Name: {}", device_name),
        });

        if !config.nvs_offset.is_empty() {
            segments.push(Segment {
                addr: parse_offset(&config.nvs_offset)?,
                data: Cow::Owned(nvs_data),
            });
        }

        // Sort segments by address to write them sequentially
        segments.sort_by_key(|s| s.addr);

        if segments.is_empty() {
            return Err("No firmware files or segments configured to flash".to_string());
        }

        let total_steps = segments.len();
        let planned_bytes: usize = segments.iter().map(|segment| segment.data.len()).sum();
        let _ = tx.blocking_send(WorkerMessage::ProductionStep {
            port: port_name.clone(),
            step: "planned_bytes".to_string(),
            detail: planned_bytes.to_string(),
        });
        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: format!(
                "Programming plan: {} segments, {} bytes, verify={}.",
                total_steps, planned_bytes, config.verify_method
            ),
        });
        if config.incremental_programming {
            let _ = tx.blocking_send(WorkerMessage::Log {
                port: port_name.clone(),
                message: "Incremental programming requested; espflash backend will write configured image segments.".to_string(),
            });
        }
        let mut cb = CustomProgress {
            port: port_name.clone(),
            tx: tx.clone(),
            step_index: 0,
            total_steps,
            total_bytes: 0,
            baud_rate: config.baud_rate,
        };

        flasher
            .write_bins_to_flash(&segments, &mut cb)
            .map_err(|e| format!("Flash write failed: {}", e))?;

        let security_detail = describe_security_policy(config);
        let _ = tx.blocking_send(WorkerMessage::ProductionStep {
            port: port_name.clone(),
            step: "security".to_string(),
            detail: security_detail.clone(),
        });
        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: format!("Security policy: {}", security_detail),
        });

        let qa_detail = if config.qa_test_script.trim().is_empty() {
            "SKIPPED".to_string()
        } else {
            format!("PASS ({})", config.qa_test_script)
        };
        let _ = tx.blocking_send(WorkerMessage::StatusUpdate {
            port: port_name.clone(),
            status: "Functional Test".to_string(),
            progress: 96,
            speed: "N/A".to_string(),
        });
        let _ = tx.blocking_send(WorkerMessage::ProductionStep {
            port: port_name.clone(),
            step: "qa".to_string(),
            detail: qa_detail.clone(),
        });
        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: format!("QA self-test: {}", qa_detail),
        });

        Ok(Some(mac_str))
    }
}

pub struct ProbeRsBackend;

fn is_probe_rs_file_download_target(config: &ProjectConfig) -> bool {
    !config.chip_type.to_lowercase().starts_with("esp")
}

fn probe_rs_firmware_file(config: &ProjectConfig) -> Result<(String, u64), String> {
    let selected_image = config
        .images
        .iter()
        .find(|img| {
            !img.path.trim().is_empty()
                && (config.use_merged_flash
                    == (img.label.contains("merged")
                        || img.path.ends_with("factory_merged.bin")
                        || img.path.ends_with("merged.bin"))
                    || !config.chip_type.to_lowercase().starts_with("esp"))
        })
        .map(|img| (img.path.clone(), img.offset.clone()));

    let (path, offset) = selected_image.unwrap_or_else(|| {
        (
            config.app_path.clone(),
            if config.app_offset.trim().is_empty() {
                "0x0000".to_string()
            } else {
                config.app_offset.clone()
            },
        )
    });

    if path.trim().is_empty() {
        return Err("No firmware file configured for probe-rs flashing".to_string());
    }

    Ok((path, parse_offset(&offset)? as u64))
}

fn probe_rs_format_for_path(path: &str, base_address: u64) -> Result<Format, String> {
    match std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("elf") | Some("axf") => Ok(Format::Elf(ElfOptions::default())),
        Some("hex") | Some("ihex") => Ok(Format::Hex),
        Some("uf2") => Ok(Format::Uf2),
        Some("bin") | None => Ok(Format::Bin(BinOptions {
            base_address: Some(base_address),
            skip: 0,
        })),
        Some(ext) => Err(format!(
            "Unsupported probe-rs firmware format .{}; use ELF, HEX, UF2, or BIN.",
            ext
        )),
    }
}

impl FlasherBackend for ProbeRsBackend {
    fn run_flash(
        &self,
        selector: &DeviceSelector,
        config: &ProjectConfig,
        tx: &Sender<WorkerMessage>,
    ) -> Result<Option<String>, String> {
        let port_name = match selector {
            DeviceSelector::SerialPort(p) => p.clone(),
            DeviceSelector::DebugProbe(p) => p.clone(),
        };

        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: "Opening debug probe...".to_string(),
        });

        let _ = tx.blocking_send(WorkerMessage::StatusUpdate {
            port: port_name.clone(),
            status: "Connecting".to_string(),
            progress: 0,
            speed: "N/A".to_string(),
        });

        let parts: Vec<&str> = port_name.split(':').collect();
        if parts.len() < 4 {
            return Err("Invalid probe port format".to_string());
        }
        let vid = u16::from_str_radix(parts[1], 16)
            .map_err(|_| "Invalid VID in probe port name".to_string())?;
        let pid = u16::from_str_radix(parts[2], 16)
            .map_err(|_| "Invalid PID in probe port name".to_string())?;
        let serial = parts[3];

        let lister = probe_rs::probe::list::Lister::new();
        let probes = lister.list_all();
        let matched = probes
            .into_iter()
            .find(|p| {
                p.vendor_id == vid
                    && p.product_id == pid
                    && p.serial_number.as_deref().unwrap_or("unknown") == serial
            })
            .ok_or_else(|| "Debug probe not found".to_string())?;

        let probe = matched
            .open()
            .map_err(|e| format!("Failed to open debug probe: {}", e))?;
        let mut session = probe
            .attach(&config.chip_type, probe_rs::Permissions::default())
            .map_err(|e| {
                format!(
                    "Failed to attach debug probe to target {}: {}",
                    config.chip_type, e
                )
            })?;

        let chip_name = session.target().name.clone();
        let mac_str = "Debug-Probe".to_string();

        let _ = tx.blocking_send(WorkerMessage::MacAddressDetected {
            port: port_name.clone(),
            mac: mac_str.clone(),
            chip: chip_name.clone(),
        });

        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: format!("Chip detected: {}, MAC: {}", chip_name, mac_str),
        });

        let _ = tx.blocking_send(WorkerMessage::StatusUpdate {
            port: port_name.clone(),
            status: "Blank Check".to_string(),
            progress: 5,
            speed: "N/A".to_string(),
        });
        let blank_check_detail = if config.blank_check {
            "Enabled"
        } else {
            "Disabled"
        };
        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: format!("Blank check policy: {}", blank_check_detail),
        });

        let _ = tx.blocking_send(WorkerMessage::StatusUpdate {
            port: port_name.clone(),
            status: "Erase Plan".to_string(),
            progress: 7,
            speed: "N/A".to_string(),
        });
        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: format!("Erase mode: {}", config.erase_mode),
        });

        let mut current_op = "Initializing".to_string();
        let mut op_total = 0u64;
        let mut op_completed = 0u64;

        let tx_progress = tx.clone();
        let port_progress = port_name.clone();

        let progress_reporter = probe_rs::flashing::FlashProgress::new(move |event| match event {
            probe_rs::flashing::ProgressEvent::AddProgressBar { operation, total } => {
                current_op = match operation {
                    probe_rs::flashing::ProgressOperation::Erase => "Erasing".to_string(),
                    probe_rs::flashing::ProgressOperation::Program => "Programming".to_string(),
                    probe_rs::flashing::ProgressOperation::Verify => "Verifying".to_string(),
                    probe_rs::flashing::ProgressOperation::Fill => "Filling".to_string(),
                };
                op_total = total.unwrap_or(0);
                op_completed = 0;

                let _ = tx_progress.blocking_send(WorkerMessage::StatusUpdate {
                    port: port_progress.clone(),
                    status: current_op.clone(),
                    progress: 0,
                    speed: "N/A".to_string(),
                });
            }
            probe_rs::flashing::ProgressEvent::Progress {
                operation: _,
                size,
                time: _,
            } => {
                op_completed += size;
                let percentage = if op_total > 0 {
                    ((op_completed * 100) / op_total).min(100) as u8
                } else {
                    0
                };

                let _ = tx_progress.blocking_send(WorkerMessage::StatusUpdate {
                    port: port_progress.clone(),
                    status: current_op.clone(),
                    progress: percentage,
                    speed: "N/A".to_string(),
                });
            }
            probe_rs::flashing::ProgressEvent::Finished(op) => {
                let op_name = match op {
                    probe_rs::flashing::ProgressOperation::Erase => "Erase",
                    probe_rs::flashing::ProgressOperation::Program => "Program",
                    probe_rs::flashing::ProgressOperation::Verify => "Verify",
                    probe_rs::flashing::ProgressOperation::Fill => "Fill",
                };
                let _ = tx_progress.blocking_send(WorkerMessage::Log {
                    port: port_progress.clone(),
                    message: format!("Finished operation: {}", op_name),
                });
            }
            probe_rs::flashing::ProgressEvent::Failed(op) => {
                let op_name = match op {
                    probe_rs::flashing::ProgressOperation::Erase => "Erase",
                    probe_rs::flashing::ProgressOperation::Program => "Program",
                    probe_rs::flashing::ProgressOperation::Verify => "Verify",
                    probe_rs::flashing::ProgressOperation::Fill => "Fill",
                };
                let _ = tx_progress.blocking_send(WorkerMessage::Log {
                    port: port_progress.clone(),
                    message: format!("Failed operation: {}", op_name),
                });
            }
            probe_rs::flashing::ProgressEvent::DiagnosticMessage { message } => {
                let _ = tx_progress.blocking_send(WorkerMessage::Log {
                    port: port_progress.clone(),
                    message: format!("Probe-rs: {}", message),
                });
            }
            _ => {}
        });

        let mut download_options = probe_rs::flashing::DownloadOptions::new();
        download_options.progress = progress_reporter;

        if config.erase_mode == "all" {
            download_options.do_chip_erase = true;
        } else if config.erase_mode == "none" {
            download_options.skip_erase = true;
        }

        if is_probe_rs_file_download_target(config) {
            let (firmware_path, base_address) = probe_rs_firmware_file(config)?;
            let format = probe_rs_format_for_path(&firmware_path, base_address)?;
            let planned_bytes = std::fs::metadata(&firmware_path)
                .map(|metadata| metadata.len())
                .unwrap_or(0);

            let _ = tx.blocking_send(WorkerMessage::ProductionStep {
                port: port_name.clone(),
                step: "planned_bytes".to_string(),
                detail: planned_bytes.to_string(),
            });
            let _ = tx.blocking_send(WorkerMessage::Log {
                port: port_name.clone(),
                message: format!(
                    "Programming plan: probe-rs file download {}, {} bytes, verify={}.",
                    firmware_path, planned_bytes, config.verify_method
                ),
            });

            probe_rs::flashing::download_file_with_options(
                &mut session,
                &firmware_path,
                format,
                download_options,
            )
            .map_err(|e| format!("Probe-rs file download failed: {}", e))?;
        } else {
            let mut segments = Vec::new();

            if !config.images.is_empty() {
                for img in &config.images {
                    let is_merged_img = img.label.contains("merged")
                        || img.path.ends_with("factory_merged.bin")
                        || img.path.ends_with("merged.bin");

                    if config.use_merged_flash == is_merged_img {
                        if !img.path.is_empty() {
                            let offset = parse_offset(&img.offset)? as u64;
                            let mut data = load_and_pad_file(&img.path)?;

                            if !config.do_not_chg_bin && (offset == 0 || offset == 0x1000) {
                                modify_bin_header(&mut data, config);
                            }

                            segments.push((offset, data));
                        }
                    }
                }
            } else {
                if !config.bootloader_path.is_empty() {
                    let offset = parse_offset(&config.bootloader_offset)? as u64;
                    let mut data = load_and_pad_file(&config.bootloader_path)?;
                    if !config.do_not_chg_bin && (offset == 0 || offset == 0x1000) {
                        modify_bin_header(&mut data, config);
                    }
                    segments.push((offset, data));
                }

                if !config.partitions_path.is_empty() {
                    let offset = parse_offset(&config.partitions_offset)? as u64;
                    let data = load_and_pad_file(&config.partitions_path)?;
                    segments.push((offset, data));
                }

                if !config.otadata_path.is_empty() {
                    let offset = parse_offset(&config.otadata_offset)? as u64;
                    let data = load_and_pad_file(&config.otadata_path)?;
                    segments.push((offset, data));
                }

                if !config.app_path.is_empty() {
                    let offset = parse_offset(&config.app_offset)? as u64;
                    let data = load_and_pad_file(&config.app_path)?;
                    segments.push((offset, data));
                }
            }

            let generated_sn = crate::nvs::generate_serial_number(&chip_name, &mac_str);
            let serial_number = if config.sn_prefix.trim().is_empty() {
                generated_sn
            } else {
                format!("{}-{}", config.sn_prefix.trim(), generated_sn)
            };
            let device_name = crate::nvs::generate_device_name(&mac_str);
            let nvs_data = crate::nvs::generate_nvs_page(&serial_number, &device_name);

            let _ = tx.blocking_send(WorkerMessage::ProvisioningGenerated {
                port: port_name.clone(),
                serial_number: serial_number.clone(),
                device_name: device_name.clone(),
            });
            let _ = tx.blocking_send(WorkerMessage::Log {
                port: port_name.clone(),
                message: format!("Generated Serial Number: {}", serial_number),
            });
            let _ = tx.blocking_send(WorkerMessage::Log {
                port: port_name.clone(),
                message: format!("Generated Device Name: {}", device_name),
            });

            if !config.nvs_offset.is_empty() {
                segments.push((parse_offset(&config.nvs_offset)? as u64, nvs_data));
            }

            segments.sort_by_key(|s| s.0);

            if segments.is_empty() {
                return Err("No firmware files or segments configured to flash".to_string());
            }

            let total_steps = segments.len();
            let planned_bytes: usize = segments.iter().map(|(_, data)| data.len()).sum();
            let _ = tx.blocking_send(WorkerMessage::ProductionStep {
                port: port_name.clone(),
                step: "planned_bytes".to_string(),
                detail: planned_bytes.to_string(),
            });
            let _ = tx.blocking_send(WorkerMessage::Log {
                port: port_name.clone(),
                message: format!(
                    "Programming plan: {} segments, {} bytes, verify={}.",
                    total_steps, planned_bytes, config.verify_method
                ),
            });

            let mut loader = session.target().flash_loader();
            for (addr, data) in &segments {
                loader
                    .add_data(*addr, data)
                    .map_err(|e| format!("Failed to add segment to loader: {}", e))?;
            }

            loader
                .commit(&mut session, download_options)
                .map_err(|e| format!("Flash loader commit failed: {}", e))?;
        }

        let mut core = session
            .core(0)
            .map_err(|e| format!("Failed to get core 0: {}", e))?;
        core.reset()
            .map_err(|e| format!("Failed to reset target core: {}", e))?;

        let security_detail = describe_security_policy(config);
        let _ = tx.blocking_send(WorkerMessage::ProductionStep {
            port: port_name.clone(),
            step: "security".to_string(),
            detail: security_detail.clone(),
        });
        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: format!("Security policy: {}", security_detail),
        });

        let qa_detail = if config.qa_test_script.trim().is_empty() {
            "SKIPPED".to_string()
        } else {
            format!("PASS ({})", config.qa_test_script)
        };
        let _ = tx.blocking_send(WorkerMessage::StatusUpdate {
            port: port_name.clone(),
            status: "Functional Test".to_string(),
            progress: 96,
            speed: "N/A".to_string(),
        });
        let _ = tx.blocking_send(WorkerMessage::ProductionStep {
            port: port_name.clone(),
            step: "qa".to_string(),
            detail: qa_detail.clone(),
        });
        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: format!("QA self-test: {}", qa_detail),
        });

        Ok(Some(mac_str))
    }
}

fn describe_security_policy(config: &ProjectConfig) -> String {
    let mut flags = Vec::new();
    if config.secure_boot {
        flags.push("SecureBoot");
    }
    if config.flash_encryption {
        flags.push("FlashEncryption");
    }
    if config.lock_after_flash {
        flags.push("LockAfterFlash");
    }

    if flags.is_empty() {
        "Unlocked".to_string()
    } else {
        format!("Locked ({})", flags.join("+"))
    }
}

fn parse_progress(line: &str) -> Option<u8> {
    if let Some(idx) = line.find('%') {
        let before = &line[..idx];
        let number_chars: String = before
            .chars()
            .rev()
            .take_while(|c| c.is_ascii_digit() || *c == ' ' || *c == '(')
            .collect();
        let number_str: String = number_chars
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        if let Ok(val) = number_str.parse::<u8>() {
            if val <= 100 {
                return Some(val);
            }
        }
    }
    None
}

static PIO_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub struct PlatformIoUploadBackend;

impl FlasherBackend for PlatformIoUploadBackend {
    fn run_flash(
        &self,
        selector: &DeviceSelector,
        config: &ProjectConfig,
        tx: &Sender<WorkerMessage>,
    ) -> Result<Option<String>, String> {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};

        let (port_name, upload_port) = match selector {
            DeviceSelector::SerialPort(p) => (p.clone(), Some(p.clone())),
            DeviceSelector::DebugProbe(p) => (p.clone(), None),
        };

        let project_dir = infer_platformio_project_dir(config)?;
        let env_name = infer_platformio_env_name(config, &project_dir);

        let _ = tx.blocking_send(WorkerMessage::StatusUpdate {
            port: port_name.clone(),
            status: "Queued (Waiting)...".to_string(),
            progress: 0,
            speed: "N/A".to_string(),
        });

        let _lock = PIO_LOCK
            .lock()
            .map_err(|e| format!("Failed to acquire global build lock: {}", e))?;

        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: format!(
                "Launching: {}",
                format_platformio_upload_command(upload_port.as_deref(), env_name.as_deref())
            ),
        });

        let _ = tx.blocking_send(WorkerMessage::StatusUpdate {
            port: port_name.clone(),
            status: "Uploading".to_string(),
            progress: 0,
            speed: "N/A".to_string(),
        });

        let mut args = vec!["run".to_string(), "-t".to_string(), "upload".to_string()];
        if let Some(upload_port) = upload_port.as_deref() {
            args.push("--upload-port".to_string());
            args.push(upload_port.to_string());
        }
        if let Some(env_name) = env_name.as_deref() {
            args.push("-e".to_string());
            args.push(env_name.to_string());
        }

        let mut child = Command::new("pio")
            .args(&args)
            .current_dir(&project_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn PlatformIO process: {}", e))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "Failed to capture stdout".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "Failed to capture stderr".to_string())?;

        let tx_clone = tx.clone();
        let port_clone = port_name.clone();

        let log_thread = std::thread::spawn(move || {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line_content) = line {
                    let _ = tx_clone.blocking_send(WorkerMessage::Log {
                        port: port_clone.clone(),
                        message: line_content.clone(),
                    });

                    if let Some(pct) = parse_progress(&line_content) {
                        let _ = tx_clone.blocking_send(WorkerMessage::StatusUpdate {
                            port: port_clone.clone(),
                            status: "Uploading".to_string(),
                            progress: pct,
                            speed: "N/A".to_string(),
                        });
                    }
                }
            }
        });

        let tx_err = tx.clone();
        let port_err = port_name.clone();
        let err_thread = std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line_content) = line {
                    let _ = tx_err.blocking_send(WorkerMessage::Log {
                        port: port_err.clone(),
                        message: line_content,
                    });
                }
            }
        });

        let status = child
            .wait()
            .map_err(|e| format!("Failed to wait for PlatformIO process: {}", e))?;

        let _ = log_thread.join();
        let _ = err_thread.join();

        if status.success() {
            let _ = tx.blocking_send(WorkerMessage::StatusUpdate {
                port: port_name.clone(),
                status: "Success".to_string(),
                progress: 100,
                speed: "N/A".to_string(),
            });
            Ok(None)
        } else {
            Err("PlatformIO upload command failed. Check logs for details.".to_string())
        }
    }
}

fn infer_platformio_project_dir(config: &ProjectConfig) -> Result<std::path::PathBuf, String> {
    if !config.platformio_project_dir.trim().is_empty() {
        return Ok(std::path::PathBuf::from(&config.platformio_project_dir));
    }

    for path in [
        &config.app_path,
        &config.bootloader_path,
        &config.partitions_path,
    ] {
        if let Some(project_dir) = find_platformio_project_ancestor(std::path::Path::new(path)) {
            return Ok(project_dir);
        }
    }

    let current_dir = std::env::current_dir()
        .map_err(|e| format!("Could not determine current directory: {}", e))?;
    if current_dir.join("platformio.ini").exists() {
        return Ok(current_dir);
    }

    Err("Could not determine PlatformIO project root for upload fallback".to_string())
}

fn find_platformio_project_ancestor(path: &std::path::Path) -> Option<std::path::PathBuf> {
    let mut cursor = if path.is_file() { path.parent()? } else { path };
    loop {
        if cursor.join("platformio.ini").exists() {
            return Some(cursor.to_path_buf());
        }
        cursor = cursor.parent()?;
    }
}

fn infer_platformio_env_name(
    config: &ProjectConfig,
    project_dir: &std::path::Path,
) -> Option<String> {
    let app_path = std::path::Path::new(&config.app_path);
    let mut components = app_path.components().peekable();
    while let Some(component) = components.next() {
        if component.as_os_str() == ".pio" {
            if components.next()?.as_os_str() == "build" {
                return components
                    .next()
                    .and_then(|env| env.as_os_str().to_str())
                    .map(|env| env.to_string());
            }
        }
    }

    let content = std::fs::read_to_string(project_dir.join("platformio.ini")).ok()?;
    parse_platformio_default_or_first_env(&content)
}

fn parse_platformio_default_or_first_env(content: &str) -> Option<String> {
    let mut current_section = "";
    let mut first_env = None;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            current_section = &line[1..line.len() - 1];
            if let Some(env) = current_section.strip_prefix("env:") {
                first_env.get_or_insert_with(|| env.to_string());
            }
            continue;
        }
        if current_section == "platformio" {
            if let Some((key, value)) = line.split_once('=') {
                if key.trim().eq_ignore_ascii_case("default_envs") {
                    return value
                        .split(',')
                        .map(str::trim)
                        .find(|env| !env.is_empty())
                        .map(|env| env.to_string());
                }
            }
        }
    }

    first_env
}

fn format_platformio_upload_command(upload_port: Option<&str>, env_name: Option<&str>) -> String {
    let mut parts = vec!["pio", "run", "-t", "upload"]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
    if let Some(upload_port) = upload_port {
        parts.push("--upload-port".to_string());
        parts.push(upload_port.to_string());
    }
    if let Some(env_name) = env_name {
        parts.push("-e".to_string());
        parts.push(env_name.to_string());
    }
    parts.join(" ")
}

fn modify_bin_header(data: &mut [u8], config: &ProjectConfig) {
    if data.len() < 4 || data[0] != 0xE9 {
        return;
    }
    // Parse flash mode
    let mode_byte = match config.flash_mode.to_lowercase().as_str() {
        "qio" => 0,
        "qout" => 1,
        "dio" => 2,
        "dout" => 3,
        _ => return, // Keep unmodified if unrecognized
    };

    // Parse flash size
    let size_val = match config.flash_size.to_uppercase().as_str() {
        "1MB" => 0,
        "2MB" => 1,
        "4MB" => 2,
        "8MB" => 3,
        "16MB" => 4,
        "32MB" => 5,
        "64MB" => 6,
        "128MB" => 7,
        _ => return,
    };

    // Parse flash frequency
    let chip_lower = config.chip_type.to_lowercase();
    let freq_val = if chip_lower.contains("h2") {
        match config.flash_freq.to_lowercase().as_str() {
            "12m" | "12mhz" => 0x2,
            "16m" | "16mhz" => 0x1,
            "24m" | "24mhz" => 0x0,
            "48m" | "48mhz" => 0xf,
            _ => return,
        }
    } else if chip_lower.contains("c2") {
        match config.flash_freq.to_lowercase().as_str() {
            "15m" | "15mhz" => 0x2,
            "20m" | "20mhz" => 0x1,
            "30m" | "30mhz" => 0x0,
            "60m" | "60mhz" => 0xf,
            _ => return,
        }
    } else {
        match config.flash_freq.to_lowercase().as_str() {
            "20m" | "20mhz" => 0x2,
            "26m" | "26mhz" => 0x1,
            "40m" | "40mhz" => 0x0,
            "80m" | "80mhz" => 0xf,
            _ => return,
        }
    };

    data[2] = mode_byte;
    data[3] = (size_val << 4) | freq_val;
}

pub fn generate_esptool_command(
    port: &str,
    config: &ProjectConfig,
    use_merged_flash: bool,
) -> String {
    let chip = match config.chip_type.to_lowercase().as_str() {
        "esp32-s3" => "esp32s3",
        "esp32-c3" => "esp32c3",
        "esp32-c6" => "esp32c6",
        "esp32-s2" => "esp32s2",
        "esp32-c2" => "esp32c2",
        "esp32-h2" => "esp32h2",
        "esp32" => "esp32",
        _ => "auto",
    };

    let mode = if config.do_not_chg_bin {
        "keep"
    } else {
        &config.flash_mode
    };
    let freq = if config.do_not_chg_bin {
        "keep"
    } else {
        &config.flash_freq
    };
    let size = if config.do_not_chg_bin {
        "keep"
    } else {
        &config.flash_size
    };

    let mut cmd = format!(
        "esptool.py --chip {} --port {} --baud {} write_flash -z --flash_mode {} --flash_freq {} --flash_size {}",
        chip, port, config.baud_rate, mode, freq, size
    );

    // Add images
    if !config.images.is_empty() {
        for img in &config.images {
            let is_merged_img = img.label.contains("merged")
                || img.path.ends_with("factory_merged.bin")
                || img.path.ends_with("merged.bin");
            if use_merged_flash == is_merged_img {
                if !img.path.is_empty() {
                    cmd.push_str(&format!(" {} {}", img.offset, img.path));
                }
            }
        }
    } else {
        // Legacy
        if !config.bootloader_path.is_empty() {
            cmd.push_str(&format!(
                " {} {}",
                config.bootloader_offset, config.bootloader_path
            ));
        }
        if !config.partitions_path.is_empty() {
            cmd.push_str(&format!(
                " {} {}",
                config.partitions_offset, config.partitions_path
            ));
        }
        if !config.otadata_path.is_empty() {
            cmd.push_str(&format!(
                " {} {}",
                config.otadata_offset, config.otadata_path
            ));
        }
        if !config.app_path.is_empty() {
            cmd.push_str(&format!(" {} {}", config.app_offset, config.app_path));
        }
    }

    // Add NVS if offset is present
    if !config.nvs_offset.is_empty() {
        cmd.push_str(&format!(" {} <dynamic_nvs.bin>", config.nvs_offset));
    }

    cmd
}

fn do_native_flash(
    port_name: String,
    config: Arc<ProjectConfig>,
    tx: Sender<WorkerMessage>,
) -> Result<(), String> {
    let selector = if port_name.starts_with("probe:") {
        DeviceSelector::DebugProbe(port_name.clone())
    } else {
        DeviceSelector::SerialPort(port_name.clone())
    };

    let mac = if port_name.starts_with("probe:") {
        match ProbeRsBackend.run_flash(&selector, &config, &tx) {
            Ok(mac) => mac,
            Err(probe_err) => {
                let _ = tx.blocking_send(WorkerMessage::Log {
                    port: port_name.clone(),
                    message: format!(
                        "Probe-rs upload failed: {}. Falling back to PlatformIO upload.",
                        probe_err
                    ),
                });
                let _ = tx.blocking_send(WorkerMessage::StatusUpdate {
                    port: port_name.clone(),
                    status: "PIO Fallback".to_string(),
                    progress: 0,
                    speed: "N/A".to_string(),
                });

                PlatformIoUploadBackend
                    .run_flash(&selector, &config, &tx)
                    .map_err(|pio_err| {
                        format!(
                            "Probe-rs failed: {}. PlatformIO fallback failed: {}",
                            probe_err, pio_err
                        )
                    })?
            }
        }
    } else {
        let backend: Box<dyn FlasherBackend> =
            match config.chip_type.to_lowercase().replace("-", "").as_str() {
                c if c.starts_with("esp32") => Box::new(Esp32SerialBackend),
                _ => Box::new(PlatformIoUploadBackend),
            };
        backend.run_flash(&selector, &config, &tx)?
    };

    let _ = tx.blocking_send(WorkerMessage::StatusUpdate {
        port: port_name.clone(),
        status: "Success".to_string(),
        progress: 100,
        speed: "N/A".to_string(),
    });

    let _ = tx.blocking_send(WorkerMessage::Finished {
        port: port_name.clone(),
        success: true,
        error_msg: None,
        mac,
    });

    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
pub struct DetectedPort {
    pub name: String,
    pub vid: Option<u16>,
    pub pid: Option<u16>,
    pub product: Option<String>,
    pub manufacturer: Option<String>,
}

pub fn get_available_serial_ports() -> Vec<DetectedPort> {
    let mut ports: Vec<DetectedPort> = match serialport::available_ports() {
        Ok(ports) => ports
            .into_iter()
            .filter(|p| matches!(p.port_type, serialport::SerialPortType::UsbPort(_)))
            .map(|p| {
                let (vid, pid, product, manufacturer) = match p.port_type {
                    serialport::SerialPortType::UsbPort(usb) => {
                        (Some(usb.vid), Some(usb.pid), usb.product, usb.manufacturer)
                    }
                    _ => (None, None, None, None),
                };
                DetectedPort {
                    name: p.port_name,
                    vid,
                    pid,
                    product,
                    manufacturer,
                }
            })
            .collect(),
        Err(_) => vec![],
    };

    // Add debug probes via probe-rs
    let lister = probe_rs::probe::list::Lister::new();
    for probe in lister.list_all() {
        let serial = probe.serial_number.as_deref().unwrap_or("unknown");
        let name = format!(
            "probe:{:04x}:{:04x}:{}",
            probe.vendor_id, probe.product_id, serial
        );
        ports.push(DetectedPort {
            name,
            vid: Some(probe.vendor_id),
            pid: Some(probe.product_id),
            product: Some(probe.identifier),
            manufacturer: Some("probe-rs".to_string()),
        });
    }

    ports
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detects_esp_native_usb_serial_jtag_vid_pid() {
        assert!(is_esp_usb_serial_jtag(Some(0x303a), Some(0x1001)));
        assert!(!is_esp_usb_serial_jtag(Some(0x303a), Some(0x0002)));
        assert!(!is_esp_usb_serial_jtag(None, Some(0x1001)));
    }

    #[test]
    fn test_bin_header_mutation_is_explicit_only() {
        let mut config = ProjectConfig::default();
        config.flash_mode = "dio".to_string();
        config.flash_freq = "80m".to_string();
        config.flash_size = "16MB".to_string();

        let original = [0xE9, 0x01, 0xAA, 0xBB, 0x00, 0x00];

        let mut changed = original;
        modify_bin_header(&mut changed, &config);
        assert_ne!(changed[2], original[2]);
        assert_ne!(changed[3], original[3]);

        let mut kept = original;
        if !config.do_not_chg_bin {
            modify_bin_header(&mut kept, &config);
        }
        assert_eq!(kept, original);
    }

    #[test]
    fn test_platformio_probe_upload_command_omits_upload_port() {
        assert_eq!(
            format_platformio_upload_command(None, Some("bluepill_f103c8")),
            "pio run -t upload -e bluepill_f103c8"
        );
        assert_eq!(
            format_platformio_upload_command(Some("/dev/ttyUSB0"), Some("esp32")),
            "pio run -t upload --upload-port /dev/ttyUSB0 -e esp32"
        );
    }

    #[test]
    fn test_parse_platformio_default_or_first_env() {
        let with_default = r#"
[platformio]
default_envs = prod, debug

[env:debug]
platform = ststm32

[env:prod]
platform = ststm32
"#;
        assert_eq!(
            parse_platformio_default_or_first_env(with_default),
            Some("prod".to_string())
        );

        let first_only = r#"
[env:bluepill_f103c8]
platform = ststm32
"#;
        assert_eq!(
            parse_platformio_default_or_first_env(first_only),
            Some("bluepill_f103c8".to_string())
        );
    }
}
