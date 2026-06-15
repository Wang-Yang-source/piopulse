use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
                } else if key == "board" {
                    board = Some(val.to_string());
                }
            }
        }

        let env_name = env_name?;
        let board = board.unwrap_or_default();

        let chip_type = if board.contains("esp32s3") {
            "ESP32-S3".to_string()
        } else if board.contains("esp32c3") {
            "ESP32-C3".to_string()
        } else if board.contains("esp32c6") {
            "ESP32-C6".to_string()
        } else if board.contains("esp32s2") {
            "ESP32-S2".to_string()
        } else {
            "Auto".to_string()
        };

        let baud_rate = upload_speed
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(921600);

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
            flash_mode: "dio".to_string(),
            flash_freq: "80m".to_string(),
            flash_size: "16MB".to_string(),
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
            _ => {}
        }
    }
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
}

