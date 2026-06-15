use crate::config::ProjectConfig;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

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
}

pub fn start_flashing_task(
    port: String,
    config: Arc<ProjectConfig>,
    simulate: bool,
    tx: Sender<WorkerMessage>,
) {
    tokio::spawn(async move {
        if simulate {
            run_simulation(port, config, tx).await;
        } else {
            run_esptool(port, config, tx).await;
        }
    });
}

async fn run_simulation(port: String, config: Arc<ProjectConfig>, tx: Sender<WorkerMessage>) {
    let _ = tx.send(WorkerMessage::Log {
        port: port.clone(),
        message: "Starting simulated flash sequence...".to_string(),
    }).await;

    // Stage 1: Connecting
    let _ = tx.send(WorkerMessage::StatusUpdate {
        port: port.clone(),
        status: "Connecting...".to_string(),
        progress: 0,
        speed: "N/A".to_string(),
    }).await;
    sleep(Duration::from_millis(800)).await;

    // Simulate random failure during connection (5% chance)
    if rand::random::<f64>() < 0.05 {
        let err = "Failed to connect to ESP32: Timed out waiting for packet header".to_string();
        let _ = tx.send(WorkerMessage::Log {
            port: port.clone(),
            message: format!("Error: {}", err),
        }).await;
        let _ = tx.send(WorkerMessage::Finished {
            port: port.clone(),
            success: false,
            error_msg: Some(err),
            mac: None,
        }).await;
        return;
    }

    // Stage 2: Chip details
    let mock_mac = format!(
        "7C:DF:A1:{:02X}:{:02X}:{:02X}",
        rand::random::<u8>(),
        rand::random::<u8>(),
        rand::random::<u8>()
    );
    let chip = config.chip_type.clone();
    let _ = tx.send(WorkerMessage::MacAddressDetected {
        port: port.clone(),
        mac: mock_mac.clone(),
        chip: chip.clone(),
    }).await;
    let _ = tx.send(WorkerMessage::Log {
        port: port.clone(),
        message: format!("Chip detected: {}, MAC: {}", chip, mock_mac),
    }).await;
    sleep(Duration::from_millis(500)).await;

    // Stage 3: Erasing
    let _ = tx.send(WorkerMessage::StatusUpdate {
        port: port.clone(),
        status: "Erasing...".to_string(),
        progress: 10,
        speed: "N/A".to_string(),
    }).await;
    let _ = tx.send(WorkerMessage::Log {
        port: port.clone(),
        message: "Erasing flash sectors...".to_string(),
    }).await;
    sleep(Duration::from_millis(800)).await;

    // Stage 4: Writing Flash
    let _ = tx.send(WorkerMessage::Log {
        port: port.clone(),
        message: "Writing flash memory...".to_string(),
    }).await;
    
    // Simulate flashing progress
    let speeds = ["885.2 Kbps", "921.6 Kbps", "905.0 Kbps", "912.4 Kbps"];
    for progress in (15..=90).step_by(5) {
        let speed = speeds[rand::random::<usize>() % speeds.len()].to_string();
        let _ = tx.send(WorkerMessage::StatusUpdate {
            port: port.clone(),
            status: format!("Flashing ({}%)", progress),
            progress,
            speed: speed.clone(),
        }).await;
        
        // Simulating random write failure at 50% (3% chance)
        if progress == 50 && rand::random::<f64>() < 0.03 {
            let err = "Flash write error: Serial connection lost mid-transfer".to_string();
            let _ = tx.send(WorkerMessage::Log {
                port: port.clone(),
                message: format!("Error at 50%: {}", err),
            }).await;
            let _ = tx.send(WorkerMessage::Finished {
                port: port.clone(),
                success: false,
                error_msg: Some(err),
                mac: Some(mock_mac),
            }).await;
            return;
        }
        sleep(Duration::from_millis(250)).await;
    }

    // Stage 5: Verifying
    let _ = tx.send(WorkerMessage::StatusUpdate {
        port: port.clone(),
        status: "Verifying...".to_string(),
        progress: 95,
        speed: "N/A".to_string(),
    }).await;
    let _ = tx.send(WorkerMessage::Log {
        port: port.clone(),
        message: "Verifying checksums...".to_string(),
    }).await;
    sleep(Duration::from_millis(600)).await;

    // Stage 6: Finished
    let _ = tx.send(WorkerMessage::StatusUpdate {
        port: port.clone(),
        status: "Success".to_string(),
        progress: 100,
        speed: "N/A".to_string(),
    }).await;
    let _ = tx.send(WorkerMessage::Log {
        port: port.clone(),
        message: format!("Verification successful. Resetting chip via EN pin..."),
    }).await;
    let _ = tx.send(WorkerMessage::Finished {
        port: port.clone(),
        success: true,
        error_msg: None,
        mac: Some(mock_mac),
    }).await;
}

async fn run_esptool(port: String, config: Arc<ProjectConfig>, tx: Sender<WorkerMessage>) {
    let _ = tx.send(WorkerMessage::Log {
        port: port.clone(),
        message: "Invoking esptool.py...".to_string(),
    }).await;

    // Build esptool command line args
    let mut args = vec![
        "--port".to_string(),
        port.clone(),
        "--baud".to_string(),
        config.baud_rate.to_string(),
    ];

    if config.chip_type != "Auto" {
        args.push("--chip".to_string());
        args.push(config.chip_type.to_lowercase());
    }

    args.extend(vec![
        "write_flash".to_string(),
        "--flash_mode".to_string(),
        config.flash_mode.clone(),
        "--flash_freq".to_string(),
        config.flash_freq.clone(),
        "--flash_size".to_string(),
        config.flash_size.clone(),
    ]);

    // Append offsets and file paths
    args.push(config.bootloader_offset.clone());
    args.push(config.bootloader_path.clone());
    args.push(config.partitions_offset.clone());
    args.push(config.partitions_path.clone());
    args.push(config.otadata_offset.clone());
    args.push(config.otadata_path.clone());
    args.push(config.app_offset.clone());
    args.push(config.app_path.clone());

    let _ = tx.send(WorkerMessage::Log {
        port: port.clone(),
        message: format!("Command: esptool.py {}", args.join(" ")),
    }).await;

    let mut cmd = Command::new("esptool.py");
    cmd.args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            let err_msg = format!("Failed to start esptool.py: {}. Make sure it is installed and in PATH.", e);
            let _ = tx.send(WorkerMessage::Log {
                port: port.clone(),
                message: err_msg.clone(),
            }).await;
            let _ = tx.send(WorkerMessage::Finished {
                port: port.clone(),
                success: false,
                error_msg: Some(err_msg),
                mac: None,
            }).await;
            return;
        }
    };

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    
    let tx_clone = tx.clone();
    let port_clone = port.clone();
    
    // Spawn a separate task to read stderr for debugging logs
    tokio::spawn(async move {
        let mut reader = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = reader.next_line().await {
            let _ = tx_clone.send(WorkerMessage::Log {
                port: port_clone.clone(),
                message: format!("[stderr] {}", line),
            }).await;
        }
    });

    let mut reader = BufReader::new(stdout).lines();
    let mut mac_detected = None;
    let mut chip_detected = None;
    
    let _ = tx.send(WorkerMessage::StatusUpdate {
        port: port.clone(),
        status: "Connecting...".to_string(),
        progress: 0,
        speed: "N/A".to_string(),
    }).await;

    while let Ok(Some(line)) = reader.next_line().await {
        let _ = tx.send(WorkerMessage::Log {
            port: port.clone(),
            message: line.clone(),
        }).await;

        // Parse MAC
        // Example: "MAC: 7c:df:a1:02:11:bc"
        if line.contains("MAC:") {
            if let Some(mac_part) = line.split("MAC:").nth(1) {
                let clean_mac = mac_part.trim().split_whitespace().next().unwrap_or("").to_string();
                if !clean_mac.is_empty() {
                    mac_detected = Some(clean_mac.clone());
                    let _ = tx.send(WorkerMessage::MacAddressDetected {
                        port: port.clone(),
                        mac: clean_mac,
                        chip: chip_detected.clone().unwrap_or_else(|| "ESP32".to_string()),
                    }).await;
                }
            }
        }

        // Parse Chip type
        // Example: "Detecting chip type... ESP32-S3"
        if line.contains("Detecting chip type...") {
            if let Some(chip_part) = line.split("type...").nth(1) {
                let clean_chip = chip_part.trim().to_string();
                chip_detected = Some(clean_chip);
            }
        }

        // Parse Progress
        // Example: "Writing at 0x00010000... (33 %)"
        if line.contains("Writing at") && line.contains("%") {
            if let Some(percentage_str) = line.split('(').nth(1).and_then(|s| s.split('%').next()) {
                if let Ok(pct) = percentage_str.trim().parse::<u8>() {
                    let progress = 10 + (pct as f32 * 0.8) as u8;
                    let _ = tx.send(WorkerMessage::StatusUpdate {
                        port: port.clone(),
                        status: format!("Flashing ({}%)", pct),
                        progress,
                        speed: format!("{} Baud", config.baud_rate),
                    }).await;
                }
            }
        }

        // Parse specific phases
        if line.contains("Erasing flash...") {
            let _ = tx.send(WorkerMessage::StatusUpdate {
                port: port.clone(),
                status: "Erasing...".to_string(),
                progress: 5,
                speed: "N/A".to_string(),
            }).await;
        }

        if line.contains("Hash of data verified") {
            let _ = tx.send(WorkerMessage::StatusUpdate {
                port: port.clone(),
                status: "Verifying...".to_string(),
                progress: 95,
                speed: "N/A".to_string(),
            }).await;
        }
    }

    match child.wait().await {
        Ok(status) if status.success() => {
            let _ = tx.send(WorkerMessage::StatusUpdate {
                port: port.clone(),
                status: "Success".to_string(),
                progress: 100,
                speed: "N/A".to_string(),
            }).await;
            let _ = tx.send(WorkerMessage::Finished {
                port: port.clone(),
                success: true,
                error_msg: None,
                mac: mac_detected,
            }).await;
        }
        _ => {
            let _ = tx.send(WorkerMessage::Finished {
                port: port.clone(),
                success: false,
                error_msg: Some("esptool.py process exited with an error".to_string()),
                mac: mac_detected,
            }).await;
        }
    }
}

pub fn get_available_serial_ports() -> Vec<String> {
    match serialport::available_ports() {
        Ok(ports) => ports
            .into_iter()
            .map(|p| p.port_name)
            .filter(|name| {
                let name_lower = name.to_lowercase();
                name_lower.contains("usb") 
                    || name_lower.contains("acm") 
                    || name_lower.contains("com") 
                    || name_lower.contains("rfcomm")
            })
            .collect(),
        Err(_) => vec![],
    }
}
