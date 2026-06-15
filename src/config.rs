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
        file.read_to_string(&mut contents).map_err(|e| e.to_string())?;
        serde_json::from_str(&contents).map_err(|e| e.to_string())
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let contents = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        let mut file = File::create(path).map_err(|e| e.to_string())?;
        file.write_all(contents.as_bytes()).map_err(|e| e.to_string())
    }
}
