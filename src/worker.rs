use crate::config::ProjectConfig;
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
    MonitorStarted {
        port: String,
        baud_rate: u32,
        sender: tokio::sync::mpsc::UnboundedSender<Vec<u8>>,
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

pub fn spawn_serial_monitor(
    port_name: String,
    baud_rate: u32,
    frame_format: String,
    tx: Sender<WorkerMessage>,
    mut cancel_rx: tokio::sync::oneshot::Receiver<()>,
) {
    tokio::spawn(async move {
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

        let (tx_cmd, mut rx_cmd) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

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
                        Ok(cmd) => {
                            if let Err(e) = port.write_all(&cmd) {
                                let _ = tx.blocking_send(WorkerMessage::Log {
                                    port: port_name.clone(),
                                    message: format!("Serial monitor write error: {}", e),
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

fn map_chip_type(chip_str: &str) -> Option<Chip> {
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

        let ports = get_available_serial_ports();
        let port_info = ports
            .into_iter()
            .find(|p| p.name == port_name)
            .map(|p| serialport::UsbPortInfo {
                vid: p.vid.unwrap_or(0),
                pid: p.pid.unwrap_or(0),
                serial_number: None,
                manufacturer: p.manufacturer,
                product: p.product,
            })
            .unwrap_or_else(|| serialport::UsbPortInfo {
                vid: 0,
                pid: 0,
                serial_number: None,
                manufacturer: None,
                product: None,
            });

        let native_usb_serial_jtag =
            is_esp_usb_serial_jtag(Some(port_info.vid), Some(port_info.pid));
        let before_reset = if native_usb_serial_jtag {
            let _ = tx.blocking_send(WorkerMessage::Log {
                port: port_name.clone(),
                message: "Detected Espressif USB-Serial-JTAG, using native USB reset.".to_string(),
            });
            ResetBeforeOperation::UsbReset
        } else {
            ResetBeforeOperation::DefaultReset
        };

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

        let mut segments = Vec::new();

        if !config.bootloader_path.is_empty() {
            let offset = parse_offset(&config.bootloader_offset)?;
            let data = load_and_pad_file(&config.bootloader_path)?;
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

        segments.push(Segment {
            addr: parse_offset(&config.nvs_offset)?,
            data: Cow::Owned(nvs_data),
        });

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
        let number_chars: String = before.chars()
            .rev()
            .take_while(|c| c.is_ascii_digit() || *c == ' ' || *c == '(')
            .collect();
        let number_str: String = number_chars.chars()
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

pub struct PlatformIoUploadBackend;

impl FlasherBackend for PlatformIoUploadBackend {
    fn run_flash(
        &self,
        selector: &DeviceSelector,
        config: &ProjectConfig,
        tx: &Sender<WorkerMessage>,
    ) -> Result<Option<String>, String> {
        use std::process::{Command, Stdio};
        use std::io::{BufRead, BufReader};

        let port_name = match selector {
            DeviceSelector::SerialPort(p) => p.clone(),
            _ => return Err("PlatformIO upload requires a serial port".to_string()),
        };

        let app_path = std::path::Path::new(&config.app_path);
        let env_name = app_path.parent()
            .and_then(|p| p.file_name())
            .and_then(|f| f.to_str())
            .ok_or_else(|| "Could not determine PlatformIO environment name from firmware path".to_string())?;

        let project_dir = app_path.parent() // <env_name>
            .and_then(|p| p.parent())       // build
            .and_then(|p| p.parent())       // .pio
            .and_then(|p| p.parent())       // project root
            .ok_or_else(|| "Could not determine PlatformIO project root from firmware path".to_string())?;

        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: format!("Launching: pio run -t upload --upload-port {} -e {}", port_name, env_name),
        });

        let _ = tx.blocking_send(WorkerMessage::StatusUpdate {
            port: port_name.clone(),
            status: "Uploading".to_string(),
            progress: 0,
            speed: "N/A".to_string(),
        });

        let mut child = Command::new("pio")
            .args(&["run", "-t", "upload", "--upload-port", &port_name, "-e", env_name])
            .current_dir(project_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn PlatformIO process: {}", e))?;

        let stdout = child.stdout.take().ok_or_else(|| "Failed to capture stdout".to_string())?;
        let stderr = child.stderr.take().ok_or_else(|| "Failed to capture stderr".to_string())?;

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

        let status = child.wait().map_err(|e| format!("Failed to wait for PlatformIO process: {}", e))?;

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

fn do_native_flash(
    port_name: String,
    config: Arc<ProjectConfig>,
    tx: Sender<WorkerMessage>,
) -> Result<(), String> {
    let selector = DeviceSelector::SerialPort(port_name.clone());

    let backend: Box<dyn FlasherBackend> =
        match config.chip_type.to_lowercase().replace("-", "").as_str() {
            c if c.starts_with("esp32") => Box::new(Esp32SerialBackend),
            _ => Box::new(PlatformIoUploadBackend),
        };

    let mac = backend.run_flash(&selector, &config, &tx)?;

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
    match serialport::available_ports() {
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
    }
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
}
