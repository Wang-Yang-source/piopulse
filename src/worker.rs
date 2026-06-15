use crate::config::ProjectConfig;
use std::borrow::Cow;
use std::fs::File;
use std::io::Read;
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
            Ok(Ok(())) => {
                spawn_serial_monitor(port, config, tx);
            }
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

fn spawn_serial_monitor(port_name: String, _config: Arc<ProjectConfig>, tx: Sender<WorkerMessage>) {
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(500)).await;

        let _ = tx
            .send(WorkerMessage::Log {
                port: port_name.clone(),
                message: "Starting Serial Monitor (Baud: 115200)...".to_string(),
            })
            .await;

        let _ = tokio::task::spawn_blocking(move || {
            let mut port = match serialport::new(&port_name, 115200)
                .timeout(Duration::from_millis(100))
                .open_native()
            {
                Ok(p) => p,
                Err(e) => {
                    let _ = tx.blocking_send(WorkerMessage::Log {
                        port: port_name.clone(),
                        message: format!("Failed to open monitor port: {}", e),
                    });
                    return;
                }
            };

            let initial_mode_u8 =
                crate::vofa::ACTIVE_VOFA_MODE.load(std::sync::atomic::Ordering::Relaxed);
            let mut parser =
                crate::vofa::VofaParser::new(crate::vofa::VofaMode::from_u8(initial_mode_u8));
            let mut read_buf = [0u8; 512];

            loop {
                let current_mode_u8 =
                    crate::vofa::ACTIVE_VOFA_MODE.load(std::sync::atomic::Ordering::Relaxed);
                parser.set_mode(crate::vofa::VofaMode::from_u8(current_mode_u8));

                match port.read(&mut read_buf) {
                    Ok(num_bytes) if num_bytes > 0 => {
                        let data = &read_buf[..num_bytes];

                        if let Ok(text) = std::str::from_utf8(data) {
                            for line in text.lines() {
                                if !line.trim().is_empty() {
                                    let _ = tx.blocking_send(WorkerMessage::Log {
                                        port: port_name.clone(),
                                        message: line.to_string(),
                                    });
                                }
                            }
                        }

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
            status: "Connecting...".to_string(),
            progress: 0,
            speed: "N/A".to_string(),
        });

        let port = serialport::new(&port_name, 115200)
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

        let connection = Connection::new(
            port,
            port_info,
            ResetAfterOperation::HardReset,
            ResetBeforeOperation::DefaultReset,
            115200,
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
        let serial_number = crate::nvs::generate_serial_number(&chip_name, &mac_str);
        let device_name = crate::nvs::generate_device_name(&mac_str);
        let nvs_data = crate::nvs::generate_nvs_page(&serial_number, &device_name);

        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: format!("Generated Serial Number: {}", serial_number),
        });
        let _ = tx.blocking_send(WorkerMessage::Log {
            port: port_name.clone(),
            message: format!("Generated Device Name: {}", device_name),
        });

        segments.push(Segment {
            addr: 0x9000,
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

        Ok(Some(mac_str))
    }
}

pub struct ProbeRsBackend;

impl FlasherBackend for ProbeRsBackend {
    fn run_flash(
        &self,
        selector: &DeviceSelector,
        _config: &ProjectConfig,
        tx: &Sender<WorkerMessage>,
    ) -> Result<Option<String>, String> {
        let label = match selector {
            DeviceSelector::SerialPort(p) => p.clone(),
            DeviceSelector::DebugProbe(s) => s.clone(),
        };
        let _ = tx.blocking_send(WorkerMessage::Log {
            port: label.clone(),
            message: "probe-rs backend is not enabled in this build.".to_string(),
        });
        Err("probe-rs backend not enabled".to_string())
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
            "stm32" | "rp2040" => Box::new(ProbeRsBackend),
            _ => Box::new(Esp32SerialBackend),
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
