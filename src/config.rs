use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const PROJECT_CONFIG_FIELD_COUNT: usize = 29;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProjectConfig {
    pub name: String,
    pub bootloader_path: String,
    pub bootloader_offset: String,
    pub partitions_path: String,
    pub partitions_offset: String,
    pub otadata_path: String,
    pub otadata_offset: String,
    pub app_path: String,
    pub app_offset: String,
    pub baud_rate: u32,
    pub chip_type: String,
    pub flash_mode: String,
    pub flash_freq: String,
    pub flash_size: String,
    pub nvs_offset: String,
    pub verify_method: String,
    pub blank_check: bool,
    pub erase_mode: String,
    pub incremental_programming: bool,
    pub secure_boot: bool,
    pub flash_encryption: bool,
    pub lock_after_flash: bool,
    pub operator_role: String,
    pub firmware_version: String,
    pub sn_prefix: String,
    pub lot_code: String,
    pub mes_endpoint: String,
    pub label_template: String,
    pub qa_test_script: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: "PixelPad ESP32-S3 Project".to_string(),
            bootloader_path: "/home/waya/Projects/PixelPad/.pio/build/4d_systems_esp32s3_gen4_r8n16/bootloader.bin".to_string(),
            bootloader_offset: "0x0000".to_string(),
            partitions_path: "/home/waya/Projects/PixelPad/.pio/build/4d_systems_esp32s3_gen4_r8n16/partitions.bin".to_string(),
            partitions_offset: "0x8000".to_string(),
            otadata_path: "/home/waya/.platformio/packages/framework-arduinoespressif32/tools/partitions/boot_app0.bin".to_string(),
            otadata_offset: "0xe000".to_string(),
            app_path: "/home/waya/Projects/PixelPad/.pio/build/4d_systems_esp32s3_gen4_r8n16/firmware.bin".to_string(),
            app_offset: "0x10000".to_string(),
            baud_rate: 921600,
            chip_type: "ESP32-S3".to_string(),
            flash_mode: "dio".to_string(),
            flash_freq: "80m".to_string(),
            flash_size: "16MB".to_string(),
            nvs_offset: "0x9000".to_string(),
            verify_method: "ReadBack+SHA256".to_string(),
            blank_check: true,
            erase_mode: "Sector".to_string(),
            incremental_programming: false,
            secure_boot: false,
            flash_encryption: false,
            lock_after_flash: false,
            operator_role: "Operator".to_string(),
            firmware_version: "dev".to_string(),
            sn_prefix: "SN".to_string(),
            lot_code: "LOT-DEV".to_string(),
            mes_endpoint: String::new(),
            label_template: "QR+SN+MAC".to_string(),
            qa_test_script: "LED,BUTTON,WIFI".to_string(),
        }
    }
}

impl ProjectConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let mut file = File::open(path).map_err(|e| e.to_string())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&contents).map_err(|e| e.to_string())
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let contents = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        let mut file = File::create(path).map_err(|e| e.to_string())?;
        file.write_all(contents.as_bytes())
            .map_err(|e| e.to_string())
    }

    pub fn detect_platformio_config() -> Option<Self> {
        let current_dir = std::env::current_dir().ok()?;
        let pio_ini_path = current_dir.join("platformio.ini");
        if !pio_ini_path.exists() {
            return None;
        }

        let content = std::fs::read_to_string(&pio_ini_path).ok()?;

        let mut env_name = None;
        let mut upload_speed = None;
        let mut board = None;
        let mut flash_mode = None;
        let mut flash_freq = None;
        let mut flash_size = None;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                let sec = &line[1..line.len() - 1];
                if sec.starts_with("env:") {
                    env_name = Some(sec["env:".len()..].to_string());
                }
            } else if let Some(idx) = line.find('=') {
                let key = line[..idx].trim().to_lowercase();
                let val = line[idx + 1..].trim();
                if key == "upload_speed" {
                    upload_speed = Some(val.to_string());
                } else if key == "board_upload.speed" {
                    upload_speed = Some(val.to_string());
                } else if key == "board" {
                    board = Some(val.to_string());
                } else if key == "board_build.flash_mode" {
                    flash_mode = Some(val.to_string());
                } else if key == "board_build.f_flash" {
                    flash_freq = parse_flash_frequency(val);
                } else if key == "board_upload.flash_size" {
                    flash_size = Some(val.to_string());
                }
            }
        }

        let env_name = env_name?;
        let board = board.unwrap_or_default();
        let board_manifest = read_platformio_board_manifest(&board);

        let chip_type = chip_type_from_board(&board, board_manifest.as_ref());

        let baud_rate = upload_speed
            .and_then(|s| s.parse::<u32>().ok())
            .or_else(|| board_manifest_upload_speed(board_manifest.as_ref()))
            .unwrap_or(921600);

        let flash_mode = flash_mode
            .or_else(|| board_manifest_build_string(board_manifest.as_ref(), "flash_mode"))
            .unwrap_or_else(|| "dio".to_string());
        let flash_freq = flash_freq
            .or_else(|| {
                board_manifest_build_string(board_manifest.as_ref(), "f_flash")
                    .and_then(|value| parse_flash_frequency(&value))
            })
            .unwrap_or_else(|| "80m".to_string());
        let flash_size = flash_size
            .or_else(|| board_manifest_upload_string(board_manifest.as_ref(), "flash_size"))
            .unwrap_or_else(|| "16MB".to_string());

        let build_dir = current_dir.join(".pio").join("build").join(&env_name);

        let bootloader_path = build_dir
            .join("bootloader.bin")
            .to_string_lossy()
            .to_string();
        let partitions_path = build_dir
            .join("partitions.bin")
            .to_string_lossy()
            .to_string();
        let app_path = build_dir.join("firmware.bin").to_string_lossy().to_string();

        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| "/home/waya".to_string());
        let otadata_path = std::path::PathBuf::from(&home)
            .join(".platformio")
            .join("packages")
            .join("framework-arduinoespressif32")
            .join("tools")
            .join("partitions")
            .join("boot_app0.bin")
            .to_string_lossy()
            .to_string();

        let folder_name = current_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("PlatformIO Project")
            .to_string();

        Some(ProjectConfig {
            name: folder_name,
            bootloader_path,
            bootloader_offset: "0x0000".to_string(),
            partitions_path,
            partitions_offset: "0x8000".to_string(),
            otadata_path,
            otadata_offset: "0xe000".to_string(),
            app_path,
            app_offset: "0x10000".to_string(),
            baud_rate,
            chip_type,
            flash_mode,
            flash_freq,
            flash_size,
            ..ProjectConfig::default()
        })
    }

    pub fn get_field(&self, index: usize) -> String {
        match index {
            0 => self.name.clone(),
            1 => self.chip_type.clone(),
            2 => self.baud_rate.to_string(),
            3 => self.flash_mode.clone(),
            4 => self.flash_freq.clone(),
            5 => self.flash_size.clone(),
            6 => self.bootloader_offset.clone(),
            7 => self.bootloader_path.clone(),
            8 => self.partitions_offset.clone(),
            9 => self.partitions_path.clone(),
            10 => self.otadata_offset.clone(),
            11 => self.otadata_path.clone(),
            12 => self.app_offset.clone(),
            13 => self.app_path.clone(),
            14 => self.nvs_offset.clone(),
            15 => self.verify_method.clone(),
            16 => self.blank_check.to_string(),
            17 => self.erase_mode.clone(),
            18 => self.incremental_programming.to_string(),
            19 => self.secure_boot.to_string(),
            20 => self.flash_encryption.to_string(),
            21 => self.lock_after_flash.to_string(),
            22 => self.operator_role.clone(),
            23 => self.firmware_version.clone(),
            24 => self.sn_prefix.clone(),
            25 => self.lot_code.clone(),
            26 => self.mes_endpoint.clone(),
            27 => self.label_template.clone(),
            28 => self.qa_test_script.clone(),
            _ => String::new(),
        }
    }

    pub fn set_field(&mut self, index: usize, value: String) {
        match index {
            0 => self.name = value,
            1 => self.chip_type = value,
            2 => {
                if let Ok(b) = value.parse::<u32>() {
                    self.baud_rate = b;
                }
            }
            3 => self.flash_mode = value,
            4 => self.flash_freq = value,
            5 => self.flash_size = value,
            6 => self.bootloader_offset = value,
            7 => self.bootloader_path = value,
            8 => self.partitions_offset = value,
            9 => self.partitions_path = value,
            10 => self.otadata_offset = value,
            11 => self.otadata_path = value,
            12 => self.app_offset = value,
            13 => self.app_path = value,
            14 => self.nvs_offset = value,
            15 => self.verify_method = value,
            16 => self.blank_check = parse_bool_field(&value, self.blank_check),
            17 => self.erase_mode = value,
            18 => {
                self.incremental_programming =
                    parse_bool_field(&value, self.incremental_programming)
            }
            19 => self.secure_boot = parse_bool_field(&value, self.secure_boot),
            20 => self.flash_encryption = parse_bool_field(&value, self.flash_encryption),
            21 => self.lock_after_flash = parse_bool_field(&value, self.lock_after_flash),
            22 => self.operator_role = value,
            23 => self.firmware_version = value,
            24 => self.sn_prefix = value,
            25 => self.lot_code = value,
            26 => self.mes_endpoint = value,
            27 => self.label_template = value,
            28 => self.qa_test_script = value,
            _ => {}
        }
    }
}

fn parse_bool_field(value: &str, current: bool) -> bool {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "y" | "on" | "enable" | "enabled" => true,
        "0" | "false" | "no" | "n" | "off" | "disable" | "disabled" => false,
        _ => current,
    }
}

fn read_platformio_board_manifest(board: &str) -> Option<serde_json::Value> {
    if board.trim().is_empty() {
        return None;
    }

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()?;
    let path = PathBuf::from(home)
        .join(".platformio")
        .join("platforms")
        .join("espressif32")
        .join("boards")
        .join(format!("{}.json", board));
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn board_manifest_build_string(manifest: Option<&serde_json::Value>, key: &str) -> Option<String> {
    manifest?
        .get("build")?
        .get(key)?
        .as_str()
        .map(|value| value.to_string())
}

fn board_manifest_upload_string(manifest: Option<&serde_json::Value>, key: &str) -> Option<String> {
    manifest?
        .get("upload")?
        .get(key)?
        .as_str()
        .map(|value| value.to_string())
}

fn board_manifest_upload_speed(manifest: Option<&serde_json::Value>) -> Option<u32> {
    manifest?
        .get("upload")?
        .get("speed")?
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
}

fn chip_type_from_board(board: &str, manifest: Option<&serde_json::Value>) -> String {
    let candidates = [
        Some(board),
        manifest
            .and_then(|value| value.get("build"))
            .and_then(|build| build.get("mcu"))
            .and_then(|value| value.as_str()),
        manifest
            .and_then(|value| value.get("build"))
            .and_then(|build| build.get("variant"))
            .and_then(|value| value.as_str()),
        manifest
            .and_then(|value| value.get("name"))
            .and_then(|value| value.as_str()),
    ];

    for candidate in candidates.into_iter().flatten() {
        let normalized = normalize_chip_name(candidate);
        if normalized.contains("esp32s3") {
            return "ESP32-S3".to_string();
        }
        if normalized.contains("esp32c3") {
            return "ESP32-C3".to_string();
        }
        if normalized.contains("esp32c6") {
            return "ESP32-C6".to_string();
        }
        if normalized.contains("esp32s2") {
            return "ESP32-S2".to_string();
        }
        if normalized.contains("esp32c2") {
            return "ESP32-C2".to_string();
        }
        if normalized.contains("esp32h2") {
            return "ESP32-H2".to_string();
        }
        if normalized.contains("esp32") {
            return "ESP32".to_string();
        }
    }

    "Auto".to_string()
}

fn normalize_chip_name(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect()
}

fn parse_flash_frequency(value: &str) -> Option<String> {
    let trimmed = value
        .trim()
        .trim_matches('"')
        .trim_end_matches(|ch| ch == 'L' || ch == 'l');
    let lower = trimmed.to_ascii_lowercase();

    if let Some(mhz) = lower.strip_suffix("mhz") {
        return mhz
            .trim()
            .parse::<u32>()
            .ok()
            .map(|value| format!("{}m", value));
    }
    if lower.ends_with('m') {
        return Some(lower);
    }

    let hz = trimmed.parse::<u64>().ok()?;
    if hz >= 1_000_000 {
        return Some(format!("{}m", hz / 1_000_000));
    }
    if hz >= 1_000 {
        return Some(format!("{}k", hz / 1_000));
    }

    None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub language: String,
}

impl Default for ToolConfig {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
        }
    }
}

impl ToolConfig {
    pub fn load() -> Self {
        let path = Self::get_path();
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str::<ToolConfig>(&content) {
                return config;
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }

    fn get_path() -> std::path::PathBuf {
        if cfg!(test) {
            return std::env::temp_dir().join(".piopulse_tool_settings_test.json");
        }
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| "/home/waya".to_string());
        std::path::Path::new(&home).join(".piopulse_tool_settings.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_config_save_load() {
        let path = ToolConfig::get_path();
        let _ = std::fs::remove_file(&path);

        let mut config = ToolConfig::load();
        assert_eq!(config.language, "en");

        config.language = "zh".to_string();
        config.save().unwrap();

        let loaded = ToolConfig::load();
        assert_eq!(loaded.language, "zh");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn test_platformio_board_name_detects_hyphenated_esp32c3() {
        assert_eq!(chip_type_from_board("esp32-c3-devkitm-1", None), "ESP32-C3");
    }

    #[test]
    fn test_platformio_flash_frequency_from_hz_literal() {
        assert_eq!(parse_flash_frequency("40000000L"), Some("40m".to_string()));
        assert_eq!(parse_flash_frequency("80000000L"), Some("80m".to_string()));
    }
}
