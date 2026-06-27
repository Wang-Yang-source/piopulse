use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

pub const PROJECT_CONFIG_FIELD_COUNT: usize = 33;
pub const MANIFEST_SLOT_COUNT: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct FirmwareImage {
    pub label: String,
    pub path: String,
    pub offset: String,
    pub required: bool,
    #[serde(default)]
    pub encrypted: bool,
    pub sha256: Option<String>,
}

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
    pub do_not_chg_bin: bool,
    pub flash_encryption_mode: String,
    pub merged_offset: String,
    pub images: Vec<FirmwareImage>,
    pub use_merged_flash: bool,
    pub manifest_locked: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub framework: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub upload_protocol: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub debug_tool: String,
    #[serde(skip)]
    pub platformio_project_dir: String,
    #[serde(skip)]
    pub factory_dir: String,
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
            do_not_chg_bin: true,
            flash_encryption_mode: "disabled".to_string(),
            merged_offset: "0x0000".to_string(),
            images: Vec::new(),
            use_merged_flash: false,
            manifest_locked: false,
            framework: String::new(),
            upload_protocol: String::new(),
            debug_tool: String::new(),
            platformio_project_dir: String::new(),
            factory_dir: String::new(),
        }
    }
}

impl ProjectConfig {
    pub fn fill_missing_tool_defaults(&mut self) -> bool {
        let mut changed = false;
        let chip_lower = self.chip_type.to_lowercase();
        let framework_lower = self.framework.to_lowercase();
        let is_esp = chip_lower.contains("esp")
            || framework_lower.contains("espidf")
            || framework_lower.contains("arduino")
            || !self.bootloader_path.trim().is_empty();

        if self.upload_protocol.trim().is_empty() {
            self.upload_protocol = if is_esp {
                "esptool".to_string()
            } else if !self.debug_tool.trim().is_empty() {
                self.debug_tool.clone()
            } else {
                "default".to_string()
            };
            changed = true;
        }

        if self.debug_tool.trim().is_empty() {
            self.debug_tool = "probe-rs".to_string();
            changed = true;
        }

        changed
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let is_toml = path
            .as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"));
        if !is_toml {
            return Err(format!(
                "PioPulse project config must be TOML: {}",
                path.as_ref().display()
            ));
        }

        let mut file = File::open(&path).map_err(|e| e.to_string())?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| e.to_string())?;
        let mut cfg: ProjectConfig = toml::from_str(&contents).map_err(|e| e.to_string())?;

        let base_dir = path.as_ref().parent().unwrap_or_else(|| Path::new(""));
        cfg.resolve_relative_paths(base_dir);
        cfg.populate_default_images_if_empty(base_dir);
        cfg.fill_missing_tool_defaults();

        Ok(cfg)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let is_toml = path
            .as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"));
        if !is_toml {
            return Err(format!(
                "PioPulse project config must be saved as TOML: {}",
                path.as_ref().display()
            ));
        }
        let path = path.as_ref();
        let contents = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create {}: {}", parent.display(), e))?;
            }
        }

        let parent = path.parent().unwrap_or_else(|| Path::new(""));
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| format!("Invalid config file path: {}", path.display()))?;
        let temp_suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let temp_path = parent.join(format!(
            ".{}.{}.{}.tmp",
            file_name,
            std::process::id(),
            temp_suffix
        ));

        let mut file = File::create(&temp_path)
            .map_err(|e| format!("Failed to create {}: {}", temp_path.display(), e))?;
        file.write_all(contents.as_bytes())
            .map_err(|e| format!("Failed to write {}: {}", temp_path.display(), e))?;
        file.sync_all()
            .map_err(|e| format!("Failed to sync {}: {}", temp_path.display(), e))?;
        drop(file);

        fs::rename(&temp_path, path).map_err(|e| {
            let _ = fs::remove_file(&temp_path);
            format!(
                "Failed to replace {} with {}: {}",
                path.display(),
                temp_path.display(),
                e
            )
        })
    }

    pub fn detect_platformio_config() -> Option<Self> {
        let current_dir = std::env::current_dir().ok()?;
        let pio_ini_path = current_dir.join("platformio.ini");
        if !pio_ini_path.exists() {
            return None;
        }

        Self::detect_platformio_config_from_ini(&pio_ini_path, &current_dir, None).ok()
    }

    pub fn prepare_external_platformio_project<S: AsRef<Path>, I: AsRef<Path>>(
        source_dir: S,
        pio_ini_path: I,
    ) -> Result<Self, String> {
        let source_dir = source_dir.as_ref();
        let pio_ini_path = pio_ini_path.as_ref();
        if !pio_ini_path.exists() {
            return Err(format!(
                "External PlatformIO config not found: {}",
                pio_ini_path.display()
            ));
        }

        let build_root = std::env::temp_dir()
            .join("piopulse-platformio")
            .join(format!(
                "{}-{}",
                source_dir
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("project"),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|e| e.to_string())?
                    .as_nanos()
            ));
        fs::create_dir_all(&build_root)
            .map_err(|e| format!("Failed to create temp PlatformIO project: {}", e))?;
        fs::copy(pio_ini_path, build_root.join("platformio.ini")).map_err(|e| {
            format!(
                "Failed to copy {} to temp PlatformIO project: {}",
                pio_ini_path.display(),
                e
            )
        })?;
        copy_source_tree_for_platformio(source_dir, &build_root)?;
        copy_platformio_referenced_assets(source_dir, pio_ini_path, &build_root)?;

        let mut config = Self::detect_platformio_config_from_ini(
            &build_root.join("platformio.ini"),
            &build_root,
            Some(source_dir.join("build")),
        )?;
        config.name = source_dir
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("PlatformIO Project")
            .to_string();
        Ok(config)
    }

    pub fn detect_platformio_config_from_ini<P: AsRef<Path>>(
        pio_ini_path: P,
        project_dir: P,
        factory_dir: Option<PathBuf>,
    ) -> Result<Self, String> {
        let pio_ini_path = pio_ini_path.as_ref();
        let project_dir = project_dir.as_ref();
        if !pio_ini_path.exists() {
            return Err(format!(
                "PlatformIO config not found: {}",
                pio_ini_path.display()
            ));
        }

        let content = std::fs::read_to_string(pio_ini_path)
            .map_err(|e| format!("Failed to read {}: {}", pio_ini_path.display(), e))?;

        let mut current_section = String::new();
        let mut env_order = Vec::new();
        let mut default_env = None;
        let mut env_values: std::collections::HashMap<
            String,
            std::collections::HashMap<String, String>,
        > = std::collections::HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                let sec = &line[1..line.len() - 1];
                current_section = sec.to_string();
                if sec.starts_with("env:") {
                    let name = sec["env:".len()..].to_string();
                    env_order.push(name.clone());
                    env_values.entry(name).or_default();
                }
            } else if let Some(idx) = line.find('=') {
                let key = line[..idx].trim().to_lowercase();
                let val = line[idx + 1..].trim().trim_matches('"').to_string();
                if current_section == "platformio" && key == "default_envs" {
                    default_env = parse_first_platformio_env(&val);
                } else if let Some(env) = current_section.strip_prefix("env:") {
                    env_values
                        .entry(env.to_string())
                        .or_default()
                        .insert(key, val);
                }
            }
        }

        let env_name = default_env
            .or_else(|| env_order.first().cloned())
            .ok_or_else(|| "No [env:*] section found in platformio.ini".to_string())?;
        let selected_values = env_values.get(&env_name);
        let get_env_value = |key: &str| -> Option<String> {
            selected_values.and_then(|values| values.get(key).cloned())
        };

        let platform = get_env_value("platform")
            .map(|value| value.to_lowercase())
            .unwrap_or_default();
        let upload_speed =
            get_env_value("upload_speed").or_else(|| get_env_value("board_upload.speed"));
        let board = get_env_value("board");
        let flash_mode = get_env_value("board_build.flash_mode");
        let flash_freq =
            get_env_value("board_build.f_flash").and_then(|value| parse_flash_frequency(&value));
        let flash_size = get_env_value("board_upload.flash_size");
        let custom_bootloader = get_env_value("board_build.arduino.custom_bootloader");
        let variants_dir = get_env_value("board_build.variants_dir");
        let framework = get_env_value("framework").unwrap_or_default();
        let upload_protocol = get_env_value("upload_protocol").unwrap_or_default();
        let debug_tool = get_env_value("debug_tool").unwrap_or_default();
        let monitor_speed_val = get_env_value("monitor_speed");
        let board = board.unwrap_or_default();
        let board_manifest = read_platformio_board_manifest(&board);

        let chip_type = chip_type_from_board(&board, board_manifest.as_ref());

        let is_zephyr = framework.to_lowercase().contains("zephyr");
        let is_stm32cube = framework.to_lowercase().contains("stm32cube");
        let is_rtos_framework = is_zephyr || is_stm32cube;

        let baud_rate = upload_speed
            .and_then(|s| s.parse::<u32>().ok())
            .or_else(|| {
                monitor_speed_val
                    .as_deref()
                    .and_then(|s| s.parse::<u32>().ok())
            })
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

        let build_dir = project_dir.join(".pio").join("build").join(&env_name);

        let is_esp_platformio_env =
            platform.contains("espressif") || board.to_lowercase().contains("esp32");

        let bootloader_path = if is_esp_platformio_env {
            resolve_platformio_bootloader_path(
                project_dir,
                &build_dir,
                board_manifest.as_ref(),
                custom_bootloader.as_deref(),
                variants_dir.as_deref(),
            )
            .to_string_lossy()
            .to_string()
        } else {
            String::new()
        };
        let partitions_path = if is_esp_platformio_env {
            build_dir
                .join("partitions.bin")
                .to_string_lossy()
                .to_string()
        } else {
            String::new()
        };
        let app_path = if is_esp_platformio_env {
            build_dir.join("firmware.bin").to_string_lossy().to_string()
        } else if is_rtos_framework {
            // Zephyr/STM32Cube: prefer .elf for probe-rs, fall back to .bin/.hex
            let elf = build_dir.join("firmware.elf");
            let bin = build_dir.join("firmware.bin");
            let hex = build_dir.join("firmware.hex");
            if elf.exists() {
                elf.to_string_lossy().to_string()
            } else if bin.exists() {
                bin.to_string_lossy().to_string()
            } else if hex.exists() {
                hex.to_string_lossy().to_string()
            } else {
                elf.to_string_lossy().to_string() // default to .elf
            }
        } else {
            build_dir.join("program").to_string_lossy().to_string()
        };

        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| "/home/waya".to_string());
        let otadata_path = if is_esp_platformio_env {
            std::path::PathBuf::from(&home)
                .join(".platformio")
                .join("packages")
                .join("framework-arduinoespressif32")
                .join("tools")
                .join("partitions")
                .join("boot_app0.bin")
                .to_string_lossy()
                .to_string()
        } else {
            String::new()
        };

        let folder_name = project_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("PlatformIO Project")
            .to_string();

        let mut config = ProjectConfig {
            name: folder_name,
            bootloader_path,
            bootloader_offset: "0x0000".to_string(),
            partitions_path,
            partitions_offset: "0x8000".to_string(),
            otadata_path,
            otadata_offset: "0xe000".to_string(),
            app_path,
            app_offset: if is_esp_platformio_env {
                "0x10000".to_string()
            } else {
                "0x0000".to_string()
            },
            baud_rate,
            chip_type: if is_esp_platformio_env || chip_type != "Auto" {
                chip_type
            } else {
                "Auto".to_string()
            },
            flash_mode: if is_esp_platformio_env {
                flash_mode
            } else {
                String::new()
            },
            flash_freq: if is_esp_platformio_env {
                flash_freq
            } else {
                String::new()
            },
            flash_size: if is_esp_platformio_env {
                flash_size
            } else {
                String::new()
            },
            framework: framework.clone(),
            upload_protocol: upload_protocol.clone(),
            debug_tool: debug_tool.clone(),
            ..ProjectConfig::default()
        };
        if !is_esp_platformio_env {
            config.nvs_offset.clear();
            config.blank_check = false;
            config.label_template.clear();
            config.qa_test_script.clear();
            config.sn_prefix.clear();
        }
        config.platformio_project_dir = project_dir.to_string_lossy().to_string();
        config.factory_dir = factory_dir
            .unwrap_or_else(|| project_dir.join("build"))
            .to_string_lossy()
            .to_string();
        config.fill_missing_tool_defaults();

        Ok(config)
    }

    pub fn materialize_platformio_factory_package(&self) -> Result<Self, String> {
        let build_dir = self
            .platformio_build_dir()
            .ok_or_else(|| "Could not determine PlatformIO build directory".to_string())?;
        let project_dir = if self.platformio_project_dir.is_empty() {
            build_dir
                .parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
                .ok_or_else(|| "Could not determine PlatformIO project root".to_string())?
                .to_path_buf()
        } else {
            PathBuf::from(&self.platformio_project_dir)
        };

        let is_esp_package = self.chip_type.to_lowercase().starts_with("esp")
            || !self.bootloader_path.is_empty()
            || !self.partitions_path.is_empty()
            || !self.otadata_path.is_empty();

        let bootloader_candidates = vec![
            PathBuf::from(&self.bootloader_path),
            build_dir.join("bootloader.bin"),
            build_dir.join("bootloader").join("bootloader.bin"),
        ];
        let partitions_candidates = vec![
            PathBuf::from(&self.partitions_path),
            build_dir.join("partitions.bin"),
        ];
        let firmware_candidates = vec![
            PathBuf::from(&self.app_path),
            build_dir.join("firmware.bin"),
        ];

        let native_program_candidates = vec![
            build_dir.join("firmware.elf"),
            build_dir.join("firmware.bin"),
            build_dir.join("program"),
            PathBuf::from(&self.app_path),
        ];

        let needs_build = if is_esp_package {
            existing_path(&bootloader_candidates).is_none()
                || existing_path(&partitions_candidates).is_none()
                || existing_path(&firmware_candidates).is_none()
        } else {
            existing_path(&native_program_candidates).is_none()
        };

        if needs_build {
            return Err("PlatformIO build artifacts (binaries) are missing. Please build the PlatformIO project first (e.g. run `pio run` in the project directory) to generate the flashing manifest.".to_string());
        }

        if !is_esp_package {
            let program_src = existing_path(&native_program_candidates).ok_or_else(|| {
                format!(
                    "PlatformIO build did not produce a supported program image under {}",
                    build_dir.display()
                )
            })?;
            let factory_dir = if self.factory_dir.is_empty() {
                project_dir.join("build")
            } else {
                PathBuf::from(&self.factory_dir)
            };
            fs::create_dir_all(&factory_dir)
                .map_err(|e| format!("Failed to create {}: {}", factory_dir.display(), e))?;

            let mut packaged = self.clone();
            packaged.bootloader_path.clear();
            packaged.partitions_path.clear();
            packaged.otadata_path.clear();
            packaged.app_path = program_src.to_string_lossy().to_string();
            packaged.app_offset = "0x0000".to_string();
            packaged.nvs_offset.clear();
            packaged.use_merged_flash = false;
            packaged.images = vec![FirmwareImage {
                label: "program".to_string(),
                path: packaged.app_path.clone(),
                offset: packaged.app_offset.clone(),
                required: true,
                encrypted: false,
                sha256: None,
            }];
            packaged.save_to_file(factory_dir.join("piopulse.toml"))?;
            return Ok(packaged);
        }

        let bootloader_src = existing_path(&bootloader_candidates).ok_or_else(|| {
            format!(
                "bootloader.bin was not produced under {}",
                build_dir.display()
            )
        })?;
        let partitions_src = existing_path(&partitions_candidates).ok_or_else(|| {
            format!(
                "partitions.bin was not produced under {}",
                build_dir.display()
            )
        })?;
        let boot_app0_src = existing_path(&[PathBuf::from(&self.otadata_path)])
            .ok_or_else(|| format!("boot_app0.bin was not found at {}", self.otadata_path))?;
        let firmware_src = existing_path(&firmware_candidates).ok_or_else(|| {
            format!(
                "firmware.bin was not produced under {}",
                build_dir.display()
            )
        })?;

        let factory_dir = if self.factory_dir.is_empty() {
            project_dir.join("build")
        } else {
            PathBuf::from(&self.factory_dir)
        };
        fs::create_dir_all(&factory_dir)
            .map_err(|e| format!("Failed to create {}: {}", factory_dir.display(), e))?;

        copy_factory_file(&bootloader_src, &factory_dir.join("bootloader.bin"))?;
        copy_factory_file(&partitions_src, &factory_dir.join("partitions.bin"))?;
        copy_factory_file(&boot_app0_src, &factory_dir.join("boot_app0.bin"))?;
        copy_factory_file(&firmware_src, &factory_dir.join("firmware.bin"))?;
        create_merged_flash_image(
            &[
                (0x0000, &bootloader_src),
                (0x8000, &partitions_src),
                (0xe000, &boot_app0_src),
                (0x10000, &firmware_src),
            ],
            &factory_dir.join("factory_merged.bin"),
        )?;

        let mut packaged = self.clone();
        packaged.bootloader_path = "bootloader.bin".to_string();
        packaged.bootloader_offset = "0x0000".to_string();
        packaged.partitions_path = "partitions.bin".to_string();
        packaged.partitions_offset = "0x8000".to_string();
        packaged.otadata_path = "boot_app0.bin".to_string();
        packaged.otadata_offset = "0xe000".to_string();
        packaged.app_path = "firmware.bin".to_string();
        packaged.app_offset = "0x10000".to_string();
        packaged.use_merged_flash = true;
        packaged.merged_offset = "0x0000".to_string();
        packaged.flash_encryption_mode = if packaged.flash_encryption {
            "device_runtime".to_string()
        } else {
            "disabled".to_string()
        };
        packaged.images = vec![
            FirmwareImage {
                label: "bootloader".to_string(),
                path: "bootloader.bin".to_string(),
                offset: "0x0000".to_string(),
                required: true,
                encrypted: false,
                sha256: None,
            },
            FirmwareImage {
                label: "partitions".to_string(),
                path: "partitions.bin".to_string(),
                offset: "0x8000".to_string(),
                required: true,
                encrypted: false,
                sha256: None,
            },
            FirmwareImage {
                label: "boot_app0".to_string(),
                path: "boot_app0.bin".to_string(),
                offset: "0xe000".to_string(),
                required: true,
                encrypted: false,
                sha256: None,
            },
            FirmwareImage {
                label: "firmware".to_string(),
                path: "firmware.bin".to_string(),
                offset: "0x10000".to_string(),
                required: true,
                encrypted: false,
                sha256: None,
            },
            FirmwareImage {
                label: "factory_merged".to_string(),
                path: "factory_merged.bin".to_string(),
                offset: packaged.merged_offset.clone(),
                required: true,
                encrypted: false,
                sha256: None,
            },
        ];

        let manifest_path = factory_dir.join("piopulse.toml");
        packaged.save_to_file(&manifest_path)?;
        ProjectConfig::load_from_file(&manifest_path)
    }

    fn platformio_build_dir(&self) -> Option<PathBuf> {
        Path::new(&self.app_path).parent().map(Path::to_path_buf)
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
            29 => self.do_not_chg_bin.to_string(),
            30 => self.framework.clone(),
            31 => self.upload_protocol.clone(),
            32 => self.debug_tool.clone(),
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
            29 => self.do_not_chg_bin = parse_bool_field(&value, self.do_not_chg_bin),
            30 => self.framework = value,
            31 => self.upload_protocol = value,
            32 => self.debug_tool = value,
            _ => {}
        }
        self.sync_flat_fields_to_images();
    }

    pub fn resolve_relative_paths(&mut self, base_dir: &Path) {
        let resolve = |p: &str| -> String {
            if p.is_empty() {
                return String::new();
            }
            let path = Path::new(p);
            if path.is_absolute() {
                p.to_string()
            } else {
                base_dir.join(path).to_string_lossy().to_string()
            }
        };

        self.bootloader_path = resolve(&self.bootloader_path);
        self.partitions_path = resolve(&self.partitions_path);
        self.otadata_path = resolve(&self.otadata_path);
        self.app_path = resolve(&self.app_path);

        for img in &mut self.images {
            img.path = resolve(&img.path);
        }
    }

    pub fn populate_default_images_if_empty(&mut self, base_dir: &Path) {
        if !self.images.is_empty() {
            return;
        }

        // Check default segmented names
        let seg_files = [
            ("bootloader", "bootloader.bin", "0x0000"),
            ("partitions", "partitions.bin", "0x8000"),
            ("boot_app0", "boot_app0.bin", "0xe000"),
            ("firmware", "firmware.bin", "0x10000"),
        ];

        let mut found_any = false;
        for &(label, filename, offset) in &seg_files {
            let p = base_dir.join(filename);
            if p.exists() {
                self.images.push(FirmwareImage {
                    label: label.to_string(),
                    path: p.to_string_lossy().to_string(),
                    offset: offset.to_string(),
                    required: true,
                    encrypted: false,
                    sha256: None,
                });
                found_any = true;
            }
        }

        // Add factory_merged.bin when present. It is a complete flash image written at 0x0000.
        let merged_path = base_dir.join("factory_merged.bin");
        if merged_path.exists() {
            self.images.push(FirmwareImage {
                label: "factory_merged".to_string(),
                path: merged_path.to_string_lossy().to_string(),
                offset: self.merged_offset.clone(),
                required: true,
                encrypted: false,
                sha256: None,
            });
            found_any = true;
        }

        if found_any {
            return;
        }

        // If nothing was found on disk in base_dir, populate from config's old fields if they are not empty
        if !self.bootloader_path.is_empty() {
            self.images.push(FirmwareImage {
                label: "bootloader".to_string(),
                path: self.bootloader_path.clone(),
                offset: self.bootloader_offset.clone(),
                required: true,
                encrypted: false,
                sha256: None,
            });
        }
        if !self.partitions_path.is_empty() {
            self.images.push(FirmwareImage {
                label: "partitions".to_string(),
                path: self.partitions_path.clone(),
                offset: self.partitions_offset.clone(),
                required: true,
                encrypted: false,
                sha256: None,
            });
        }
        if !self.otadata_path.is_empty() {
            self.images.push(FirmwareImage {
                label: "boot_app0".to_string(),
                path: self.otadata_path.clone(),
                offset: self.otadata_offset.clone(),
                required: true,
                encrypted: false,
                sha256: None,
            });
        }
        if !self.app_path.is_empty() {
            self.images.push(FirmwareImage {
                label: "firmware".to_string(),
                path: self.app_path.clone(),
                offset: self.app_offset.clone(),
                required: true,
                encrypted: false,
                sha256: None,
            });
        }
    }

    pub fn validate_manifest(&self) -> (Vec<ImageValidationResult>, Vec<String>) {
        let mut image_results = Vec::new();
        let mut errors = Vec::new();

        // 1. Do not whitelist chip families here. Debug-probe targets are resolved by
        // probe-rs at flash time, and unsupported probe-rs targets can fall back to
        // PlatformIO's uploader.
        let chip_lower = self.chip_type.to_lowercase();
        if chip_lower.trim().is_empty() {
            errors.push("Target chip type is empty".to_string());
        }

        // 2. Track offsets to check for duplicates
        let mut offset_map = std::collections::HashMap::new();

        // 3. Validate each image in self.images
        for img in &self.images {
            let mut exists = false;
            let mut size_bytes = None;
            let mut sha256_match = None;
            let mut img_error = None;
            let path_is_empty = img.path.trim().is_empty();

            if !img.required && path_is_empty {
                image_results.push(ImageValidationResult {
                    label: img.label.clone(),
                    offset: img.offset.clone(),
                    path: img.path.clone(),
                    size_bytes,
                    exists,
                    sha256_match,
                    error: img_error,
                });
                continue;
            }

            // Offset validation
            let _offset_val = match parse_offset(&img.offset) {
                Ok(val) => {
                    // Check duplicate
                    if let Some(prev_label) = offset_map.insert(val, img.label.clone()) {
                        errors.push(format!(
                            "Duplicate offset {:#x} configured for '{}' and '{}'",
                            val, prev_label, img.label
                        ));
                    }
                    Some(val)
                }
                Err(e) => {
                    let err_msg = format!("Invalid offset '{}': {}", img.offset, e);
                    errors.push(err_msg.clone());
                    img_error = Some(err_msg);
                    None
                }
            };

            // Path & Exists validation
            if path_is_empty {
                if img.required {
                    errors.push(format!("Required image '{}' path is empty", img.label));
                    img_error = Some("Path is empty".to_string());
                }
            } else {
                let path = Path::new(&img.path);
                if path.exists() {
                    exists = true;
                    if let Ok(metadata) = std::fs::metadata(path) {
                        size_bytes = Some(metadata.len());
                    }

                    // SHA256 validation if specified
                    if let Some(ref expected_sha) = img.sha256 {
                        if !expected_sha.trim().is_empty() {
                            match compute_file_sha256(&img.path) {
                                Ok(computed) => {
                                    let matches =
                                        computed.eq_ignore_ascii_case(expected_sha.trim());
                                    sha256_match = Some(matches);
                                    if !matches {
                                        let err_msg = format!(
                                            "SHA256 mismatch for '{}': expected {}, got {}",
                                            img.label, expected_sha, computed
                                        );
                                        errors.push(err_msg.clone());
                                        img_error = Some("SHA256 mismatch".to_string());
                                    }
                                }
                                Err(e) => {
                                    errors.push(format!(
                                        "Failed to read file '{}' for SHA256 check: {}",
                                        img.label, e
                                    ));
                                    sha256_match = Some(false);
                                    img_error = Some("SHA256 calculation failed".to_string());
                                }
                            }
                        }
                    }
                } else if img.required {
                    let err_msg =
                        format!("Required file '{}' not found at: {}", img.label, img.path);
                    errors.push(err_msg.clone());
                    img_error = Some("File not found".to_string());
                }
            }

            image_results.push(ImageValidationResult {
                label: img.label.clone(),
                offset: img.offset.clone(),
                path: img.path.clone(),
                size_bytes,
                exists,
                sha256_match,
                error: img_error,
            });
        }

        (image_results, errors)
    }

    pub fn manifest_results_for_mode(&self, use_merged_flash: bool) -> Vec<ImageValidationResult> {
        let (image_results, _) = self.validate_manifest();
        let mut filtered_results: Vec<ImageValidationResult> = image_results
            .into_iter()
            .filter(|res| use_merged_flash == is_merged_manifest_entry(&res.label, &res.path))
            .collect();

        if !use_merged_flash {
            let mut slot_index = 1;
            while filtered_results.len() < MANIFEST_SLOT_COUNT {
                let label = manifest_slot_label(slot_index);
                slot_index += 1;
                if filtered_results.iter().any(|res| res.label == label) {
                    continue;
                }
                filtered_results.push(ImageValidationResult {
                    label,
                    offset: String::new(),
                    path: String::new(),
                    size_bytes: None,
                    exists: false,
                    sha256_match: None,
                    error: None,
                });
            }
        } else if filtered_results.is_empty() {
            filtered_results.push(ImageValidationResult {
                label: "factory_merged".to_string(),
                offset: if self.merged_offset.trim().is_empty() {
                    "0x0000".to_string()
                } else {
                    self.merged_offset.clone()
                },
                path: String::new(),
                size_bytes: None,
                exists: false,
                sha256_match: None,
                error: None,
            });
        }

        filtered_results
    }

    pub fn ensure_manifest_image(&mut self, label: &str) -> &mut FirmwareImage {
        if let Some(pos) = self.images.iter().position(|img| img.label == label) {
            return &mut self.images[pos];
        }

        let default_offset = match label {
            "factory_merged" | "merged" => {
                if self.merged_offset.trim().is_empty() {
                    "0x0000"
                } else {
                    self.merged_offset.as_str()
                }
            }
            _ => "",
        };

        self.images.push(FirmwareImage {
            label: label.to_string(),
            path: String::new(),
            offset: default_offset.to_string(),
            required: false,
            encrypted: false,
            sha256: None,
        });
        let last = self.images.len().saturating_sub(1);
        &mut self.images[last]
    }

    pub fn clear_manifest_image(&mut self, label: &str) -> bool {
        let Some(pos) = self.images.iter().position(|img| img.label == label) else {
            return false;
        };

        let had_content = {
            let img = &self.images[pos];
            !img.path.trim().is_empty()
                || !img.offset.trim().is_empty()
                || img
                    .sha256
                    .as_deref()
                    .is_some_and(|sha| !sha.trim().is_empty())
        };

        let is_builtin = matches!(
            label,
            "bootloader"
                | "partitions"
                | "boot_app0"
                | "firmware"
                | "program"
                | "factory_merged"
                | "merged"
        );

        if !is_builtin {
            self.images.remove(pos);
            return had_content;
        }

        let img = &mut self.images[pos];
        img.path.clear();
        img.offset.clear();
        img.required = false;
        img.encrypted = false;
        img.sha256 = None;

        match label {
            "bootloader" => {
                self.bootloader_path.clear();
                self.bootloader_offset.clear();
            }
            "partitions" => {
                self.partitions_path.clear();
                self.partitions_offset.clear();
            }
            "boot_app0" => {
                self.otadata_path.clear();
                self.otadata_offset.clear();
            }
            "firmware" | "program" => {
                self.app_path.clear();
                self.app_offset.clear();
            }
            "factory_merged" | "merged" => {
                self.merged_offset.clear();
            }
            _ => {}
        }

        had_content
    }

    pub fn sync_flat_fields_to_images(&mut self) {
        let mut update_or_insert = |label: &str, path: &str, offset: &str| {
            if path.is_empty() {
                if let Some(pos) = self.images.iter().position(|img| img.label == label) {
                    self.images.remove(pos);
                }
                return;
            }
            if let Some(img) = self.images.iter_mut().find(|img| img.label == label) {
                img.path = path.to_string();
                img.offset = offset.to_string();
            } else {
                self.images.push(FirmwareImage {
                    label: label.to_string(),
                    path: path.to_string(),
                    offset: offset.to_string(),
                    required: true,
                    encrypted: false,
                    sha256: None,
                });
            }
        };

        update_or_insert("bootloader", &self.bootloader_path, &self.bootloader_offset);
        update_or_insert("partitions", &self.partitions_path, &self.partitions_offset);
        update_or_insert("boot_app0", &self.otadata_path, &self.otadata_offset);
        update_or_insert("firmware", &self.app_path, &self.app_offset);
    }

    pub fn sync_images_to_flat_fields(&mut self) {
        for img in &self.images {
            match img.label.as_str() {
                "bootloader" => {
                    self.bootloader_path = img.path.clone();
                    self.bootloader_offset = img.offset.clone();
                }
                "partitions" => {
                    self.partitions_path = img.path.clone();
                    self.partitions_offset = img.offset.clone();
                }
                "boot_app0" => {
                    self.otadata_path = img.path.clone();
                    self.otadata_offset = img.offset.clone();
                }
                "firmware" => {
                    self.app_path = img.path.clone();
                    self.app_offset = img.offset.clone();
                }
                "factory_merged" | "merged" => {
                    self.merged_offset = img.offset.clone();
                    if !img.path.trim().is_empty() {
                        self.use_merged_flash = true;
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn is_merged_manifest_entry(label: &str, path: &str) -> bool {
    label.contains("merged") || path.ends_with("factory_merged.bin") || path.ends_with("merged.bin")
}

pub fn manifest_slot_label(index: usize) -> String {
    format!("slot_{}", index)
}

fn existing_path(candidates: &[PathBuf]) -> Option<PathBuf> {
    candidates.iter().find(|path| path.exists()).cloned()
}

fn copy_factory_file(src: &Path, dest: &Path) -> Result<(), String> {
    fs::copy(src, dest).map(|_| ()).map_err(|e| {
        format!(
            "Failed to copy {} to {}: {}",
            src.display(),
            dest.display(),
            e
        )
    })
}

pub fn create_merged_flash_image(segments: &[(usize, &Path)], dest: &Path) -> Result<(), String> {
    let mut loaded_segments = Vec::new();
    let mut total_len = 0usize;

    for (offset, path) in segments {
        let data =
            fs::read(path).map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        total_len = total_len.max(offset.saturating_add(data.len()));
        loaded_segments.push((*offset, data));
    }

    let mut merged = vec![0xff; total_len];
    for (offset, data) in loaded_segments {
        let end = offset + data.len();
        merged[offset..end].copy_from_slice(&data);
    }

    fs::write(dest, merged).map_err(|e| format!("Failed to write {}: {}", dest.display(), e))
}

fn copy_source_tree_for_platformio(source_dir: &Path, dest_dir: &Path) -> Result<(), String> {
    let mut copied_under_src = false;
    copy_source_tree_recursive(source_dir, source_dir, dest_dir, &mut copied_under_src)?;

    if !copied_under_src {
        let src_dest = dest_dir.join("src");
        let include_dest = dest_dir.join("include");
        for entry in fs::read_dir(source_dir)
            .map_err(|e| format!("Failed to read source dir {}: {}", source_dir.display(), e))?
        {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if is_platformio_compile_source(&path) {
                fs::create_dir_all(&src_dest).map_err(|e| e.to_string())?;
                fs::copy(&path, src_dest.join(entry.file_name())).map_err(|e| {
                    format!("Failed to copy {} into temp src: {}", path.display(), e)
                })?;
            } else if is_platformio_header(&path) {
                fs::create_dir_all(&include_dest).map_err(|e| e.to_string())?;
                fs::copy(&path, include_dest.join(entry.file_name())).map_err(|e| {
                    format!("Failed to copy {} into temp include: {}", path.display(), e)
                })?;
            }
        }
    }

    Ok(())
}

fn copy_source_tree_recursive(
    source_root: &Path,
    current: &Path,
    dest_root: &Path,
    copied_under_src: &mut bool,
) -> Result<(), String> {
    for entry in fs::read_dir(current)
        .map_err(|e| format!("Failed to read source dir {}: {}", current.display(), e))?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if path.is_dir() {
            if matches!(
                name_str.as_ref(),
                ".git" | ".pio" | "build" | "factory" | "target" | ".agents" | ".codex"
            ) {
                continue;
            }
            copy_source_tree_recursive(source_root, &path, dest_root, copied_under_src)?;
            continue;
        }

        if !path.is_file() || !is_platformio_source_asset(&path) {
            continue;
        }

        let relative = path.strip_prefix(source_root).map_err(|e| e.to_string())?;
        if relative
            .components()
            .next()
            .is_some_and(|part| part.as_os_str() == "src")
        {
            *copied_under_src = true;
        }
        let dest = dest_root.join(relative);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::copy(&path, &dest).map_err(|e| {
            format!(
                "Failed to copy {} to {}: {}",
                path.display(),
                dest.display(),
                e
            )
        })?;
    }

    Ok(())
}

fn is_platformio_source_asset(path: &Path) -> bool {
    is_platformio_compile_source(path) || is_platformio_header(path)
}

fn is_platformio_compile_source(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "c" | "cc" | "cpp" | "cxx" | "ino" | "s"
            )
        })
}

fn is_platformio_header(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "h" | "hh" | "hpp" | "hxx"
            )
        })
}

fn copy_platformio_referenced_assets(
    source_dir: &Path,
    pio_ini_path: &Path,
    dest_dir: &Path,
) -> Result<(), String> {
    let content = fs::read_to_string(pio_ini_path)
        .map_err(|e| format!("Failed to read {}: {}", pio_ini_path.display(), e))?;

    for raw_path in referenced_platformio_asset_paths(&content) {
        let source_path = source_dir.join(&raw_path);
        if !source_path.exists() {
            continue;
        }
        copy_path_into_temp_project(&source_path, &dest_dir.join(&raw_path))?;
    }

    Ok(())
}

fn referenced_platformio_asset_paths(content: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut current_section = String::new();
    let mut selected_env = None;
    let mut env_order = Vec::new();
    let mut env_values: std::collections::HashMap<
        String,
        std::collections::HashMap<String, String>,
    > = std::collections::HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len() - 1].to_string();
            if let Some(env) = current_section.strip_prefix("env:") {
                env_order.push(env.to_string());
                env_values.entry(env.to_string()).or_default();
            }
            continue;
        }

        let Some(idx) = line.find('=') else {
            continue;
        };
        let key = line[..idx].trim().to_lowercase();
        let value = line[idx + 1..].trim().trim_matches('"').to_string();
        if current_section == "platformio" && key == "default_envs" {
            selected_env = parse_first_platformio_env(&value);
        } else if let Some(env) = current_section.strip_prefix("env:") {
            env_values
                .entry(env.to_string())
                .or_default()
                .insert(key, value);
        }
    }

    let env_name = selected_env.or_else(|| env_order.first().cloned());
    let Some(values) = env_name.and_then(|env| env_values.get(&env).cloned()) else {
        return paths;
    };

    let direct_keys = [
        "board_build.partitions",
        "board_build.variants_dir",
        "board_build.embed_txtfiles",
        "board_build.embed_files",
        "board_build.filesystem",
    ];
    for key in direct_keys {
        if let Some(value) = values.get(key) {
            push_platformio_asset_paths(&mut paths, value);
        }
    }

    let variants_dir = values
        .get("board_build.variants_dir")
        .cloned()
        .unwrap_or_else(|| "variants".to_string());
    if let Some(custom_bootloader) = values.get("board_build.arduino.custom_bootloader") {
        paths.push(PathBuf::from(&variants_dir));
        paths.push(PathBuf::from(custom_bootloader));
    }

    paths.sort();
    paths.dedup();
    paths
}

fn push_platformio_asset_paths(paths: &mut Vec<PathBuf>, value: &str) {
    for item in value
        .split(|ch| ch == ',' || ch == '\n' || ch == ' ' || ch == '\t')
        .map(str::trim)
        .filter(|item| !item.is_empty())
    {
        if item.starts_with('$') || item.starts_with('-') {
            continue;
        }
        paths.push(PathBuf::from(item));
    }
}

fn copy_path_into_temp_project(src: &Path, dest: &Path) -> Result<(), String> {
    if src.is_dir() {
        for entry in
            fs::read_dir(src).map_err(|e| format!("Failed to read {}: {}", src.display(), e))?
        {
            let entry = entry.map_err(|e| e.to_string())?;
            let child_src = entry.path();
            let child_dest = dest.join(entry.file_name());
            copy_path_into_temp_project(&child_src, &child_dest)?;
        }
        return Ok(());
    }

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::copy(src, dest).map(|_| ()).map_err(|e| {
        format!(
            "Failed to copy {} to {}: {}",
            src.display(),
            dest.display(),
            e
        )
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageValidationResult {
    pub label: String,
    pub offset: String,
    pub path: String,
    pub size_bytes: Option<u64>,
    pub exists: bool,
    pub sha256_match: Option<bool>,
    pub error: Option<String>,
}

pub fn parse_offset(offset_str: &str) -> Result<u32, String> {
    let clean = offset_str.trim().to_lowercase();
    let clean = clean.trim_start_matches("0x").trim_start_matches("0x");
    u32::from_str_radix(clean, 16).map_err(|e| format!("Invalid offset '{}': {}", offset_str, e))
}

fn compute_file_sha256(path: &str) -> Result<String, String> {
    use sha2::{Digest, Sha256};
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 4096];
    loop {
        let count = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
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
    let platforms_dir = PathBuf::from(home).join(".platformio").join("platforms");

    let espressif_path = platforms_dir
        .join("espressif32")
        .join("boards")
        .join(format!("{}.json", board));
    if let Ok(content) = std::fs::read_to_string(espressif_path) {
        return serde_json::from_str(&content).ok();
    }

    let entries = std::fs::read_dir(platforms_dir).ok()?;
    for entry in entries.flatten() {
        let path = entry.path().join("boards").join(format!("{}.json", board));
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(manifest) = serde_json::from_str(&content) {
                return Some(manifest);
            }
        }
    }

    None
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

fn resolve_platformio_bootloader_path(
    project_dir: &Path,
    build_dir: &Path,
    board_manifest: Option<&serde_json::Value>,
    custom_bootloader: Option<&str>,
    variants_dir: Option<&str>,
) -> PathBuf {
    let fallback = build_dir.join("bootloader.bin");
    let Some(custom_bootloader) = custom_bootloader
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return fallback;
    };

    let custom_path = Path::new(custom_bootloader);
    if custom_path.is_absolute() && custom_path.is_file() {
        return custom_path.to_path_buf();
    }

    let variant = board_manifest_build_string(board_manifest, "variant");
    let variants_dir = variants_dir
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("variants");
    let variants_root = project_dir.join(variants_dir);

    let mut candidates = Vec::new();
    candidates.push(project_dir.join(custom_bootloader));
    if let Some(variant) = variant.as_deref() {
        candidates.push(variants_root.join(variant).join(custom_bootloader));
    }
    candidates.push(variants_root.join(custom_bootloader));

    for candidate in candidates {
        if candidate.is_file() {
            return candidate;
        }
    }

    if let Ok(entries) = std::fs::read_dir(&variants_root) {
        for entry in entries.flatten() {
            let path = entry.path().join(custom_bootloader);
            if path.is_file() {
                return path;
            }
        }
    }

    fallback
}

fn chip_type_from_board(board: &str, manifest: Option<&serde_json::Value>) -> String {
    let candidates = [
        Some(board),
        manifest
            .and_then(|value| value.get("debug"))
            .and_then(|debug| debug.get("jlink_device"))
            .and_then(|value| value.as_str()),
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
        if let Some(stm32_chip) = stm32_chip_type_from_candidate(candidate) {
            return stm32_chip;
        }
    }

    "Auto".to_string()
}

fn stm32_chip_type_from_candidate(candidate: &str) -> Option<String> {
    let normalized = normalize_chip_name(candidate);
    let start = normalized.find("stm32")?;
    let chip = &normalized[start..];
    if chip.len() < "stm32f103c8".len() {
        return None;
    }

    let mut end = chip.len();
    if end >= 13 {
        let suffix = &chip[end - 2..];
        let mut chars = suffix.chars();
        if matches!(chars.next(), Some('a'..='z'))
            && matches!(chars.next(), Some('0'..='9' | 'a'..='z'))
        {
            end -= 2;
        }
    }

    Some(chip[..end].to_ascii_uppercase())
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

fn parse_first_platformio_env(value: &str) -> Option<String> {
    value
        .split(|ch| ch == ',' || ch == ' ' || ch == '\t')
        .map(str::trim)
        .find(|env| !env.is_empty())
        .map(ToString::to_string)
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
            if let Ok(config) = toml::from_str::<ToolConfig>(&content) {
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
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }

    fn get_path() -> std::path::PathBuf {
        if cfg!(test) {
            return std::env::temp_dir().join(".piopulse_tool_settings_test.toml");
        }
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| "/home/waya".to_string());
        std::path::Path::new(&home).join(".piopulse_tool_settings.toml")
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
    fn test_toml_project_config_loads_relative_images() {
        let temp_dir = std::env::temp_dir().join(format!(
            "piopulse_toml_config_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let config_path = temp_dir.join("piopulse.toml");
        let toml_config = r#"
name = "Factory Package"
chip_type = "ESP32-S3"
baud_rate = 921600
flash_mode = "dio"
flash_freq = "80m"
flash_size = "16MB"
do_not_chg_bin = true
manifest_locked = true

[[images]]
label = "firmware"
path = "firmware.bin"
offset = "0x10000"
required = true
"#;

        std::fs::write(&config_path, toml_config).unwrap();
        let config = ProjectConfig::load_from_file(&config_path).unwrap();

        assert_eq!(config.name, "Factory Package");
        assert!(config.do_not_chg_bin);
        assert!(config.manifest_locked);
        assert_eq!(config.images.len(), 1);
        assert_eq!(
            config.images[0].path,
            temp_dir.join("firmware.bin").to_string_lossy().to_string()
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_project_config_save_to_file_is_reloadable_and_cleans_temp_file() {
        let temp_dir = std::env::temp_dir().join(format!(
            "piopulse_atomic_save_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let config_path = temp_dir.join("piopulse.toml");
        std::fs::write(&config_path, "name = \"old\"\n").unwrap();

        let mut config = ProjectConfig::default();
        config.name = "Saved Config".to_string();
        config.chip_type = "ESP32-C3".to_string();
        config.save_to_file(&config_path).unwrap();

        let loaded = ProjectConfig::load_from_file(&config_path).unwrap();
        assert_eq!(loaded.name, "Saved Config");
        assert_eq!(loaded.chip_type, "ESP32-C3");

        let temp_files: Vec<_> = std::fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".piopulse.toml.")
            })
            .collect();
        assert!(temp_files.is_empty());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_project_config_fills_and_saves_missing_platformio_tool_defaults() {
        let temp_dir = std::env::temp_dir().join(format!(
            "piopulse_tool_defaults_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let config_path = temp_dir.join("piopulse.toml");
        std::fs::write(
            &config_path,
            r#"
name = "PixelPad"
chip_type = "ESP32-S3"
framework = "arduino"
upload_protocol = ""
debug_tool = ""
baud_rate = 921600
"#,
        )
        .unwrap();

        let config = ProjectConfig::load_from_file(&config_path).unwrap();
        assert_eq!(config.upload_protocol, "esptool");
        assert_eq!(config.debug_tool, "probe-rs");

        config.save_to_file(&config_path).unwrap();
        let saved = std::fs::read_to_string(&config_path).unwrap();
        assert!(saved.contains("upload_protocol = \"esptool\""));
        assert!(saved.contains("debug_tool = \"probe-rs\""));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_materialize_platformio_factory_package_from_build_outputs() {
        let temp_dir = std::env::temp_dir().join(format!(
            "piopulse_factory_package_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let project_dir = temp_dir.join("firmware_project");
        let build_dir = project_dir.join(".pio").join("build").join("prod");
        std::fs::create_dir_all(&build_dir).unwrap();

        std::fs::write(build_dir.join("bootloader.bin"), [0x01]).unwrap();
        std::fs::write(build_dir.join("partitions.bin"), [0x02]).unwrap();
        std::fs::write(build_dir.join("firmware.bin"), [0x03]).unwrap();
        let boot_app0 = temp_dir.join("boot_app0.bin");
        std::fs::write(&boot_app0, [0x04]).unwrap();

        let mut config = ProjectConfig::default();
        config.name = "Factory Project".to_string();
        config.bootloader_path = build_dir
            .join("bootloader.bin")
            .to_string_lossy()
            .to_string();
        config.partitions_path = build_dir
            .join("partitions.bin")
            .to_string_lossy()
            .to_string();
        config.otadata_path = boot_app0.to_string_lossy().to_string();
        config.app_path = build_dir.join("firmware.bin").to_string_lossy().to_string();

        let packaged = config.materialize_platformio_factory_package().unwrap();
        let factory_dir = project_dir.join("build");

        assert!(factory_dir.join("piopulse.toml").exists());
        assert!(factory_dir.join("bootloader.bin").exists());
        assert!(factory_dir.join("partitions.bin").exists());
        assert!(factory_dir.join("boot_app0.bin").exists());
        assert!(factory_dir.join("firmware.bin").exists());
        assert!(factory_dir.join("factory_merged.bin").exists());
        assert_eq!(
            packaged.app_path,
            factory_dir
                .join("firmware.bin")
                .to_string_lossy()
                .to_string()
        );
        assert_eq!(packaged.images.len(), 5);
        let merged = packaged
            .images
            .iter()
            .find(|img| img.label == "factory_merged")
            .unwrap();
        assert_eq!(merged.offset, "0x0000");
        assert_eq!(
            merged.path,
            factory_dir.join("factory_merged.bin").to_string_lossy()
        );
        assert!(!merged.encrypted);
        assert_eq!(packaged.merged_offset, "0x0000");
        assert!(packaged.use_merged_flash);
        assert_eq!(packaged.flash_encryption_mode, "disabled");

        let merged_data = std::fs::read(factory_dir.join("factory_merged.bin")).unwrap();
        assert_eq!(merged_data[0x0000], 0x01);
        assert_eq!(merged_data[0x8000], 0x02);
        assert_eq!(merged_data[0xe000], 0x04);
        assert_eq!(merged_data[0x10000], 0x03);
        assert_eq!(merged_data[0x0001], 0xff);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_materialize_platformio_native_package_from_program_output() {
        let temp_dir = std::env::temp_dir().join(format!(
            "piopulse_native_package_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let project_dir = temp_dir.join("native_project");
        let build_dir = project_dir.join(".pio").join("build").join("native-env");
        std::fs::create_dir_all(&build_dir).unwrap();
        std::fs::write(
            project_dir.join("platformio.ini"),
            r#"
[platformio]
default_envs = native-env

[env:native-env]
platform = native
"#,
        )
        .unwrap();
        std::fs::write(build_dir.join("program"), [0x7f, b'E', b'L', b'F']).unwrap();

        let config = ProjectConfig::detect_platformio_config_from_ini(
            &project_dir.join("platformio.ini"),
            &project_dir,
            None,
        )
        .unwrap();

        assert_eq!(config.chip_type, "Auto");
        assert!(config.bootloader_path.is_empty());
        assert!(config.partitions_path.is_empty());
        assert!(config.otadata_path.is_empty());
        assert_eq!(config.app_offset, "0x0000");

        let packaged = config.materialize_platformio_factory_package().unwrap();
        let factory_dir = project_dir.join("build");

        assert!(factory_dir.join("piopulse.toml").exists());
        assert!(!factory_dir.join("program").exists());
        assert!(!factory_dir.join("bootloader.bin").exists());
        assert!(!factory_dir.join("partitions.bin").exists());
        assert_eq!(packaged.images.len(), 1);
        assert_eq!(packaged.images[0].label, "program");
        assert_eq!(
            packaged.images[0].path,
            build_dir.join("program").to_string_lossy().to_string()
        );
        assert_eq!(packaged.images[0].offset, "0x0000");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_platformio_custom_bootloader_resolves_from_project_variants() {
        let temp_dir = std::env::temp_dir().join(format!(
            "piopulse_custom_bootloader_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let project_dir = temp_dir.join("firmware_project");
        let bootloader = project_dir
            .join("variants")
            .join("esp32_s3r8n16")
            .join("bootloader_dio_40m.bin");
        std::fs::create_dir_all(bootloader.parent().unwrap()).unwrap();
        std::fs::write(&bootloader, [0x01]).unwrap();

        let build_dir = project_dir.join(".pio").join("build").join("prod");
        let manifest = serde_json::json!({
            "build": {
                "variant": "esp32_s3r8n16"
            }
        });

        let resolved = resolve_platformio_bootloader_path(
            &project_dir,
            &build_dir,
            Some(&manifest),
            Some("bootloader_dio_40m.bin"),
            Some("variants"),
        );

        assert_eq!(resolved, bootloader);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_platformio_board_name_detects_hyphenated_esp32c3() {
        assert_eq!(chip_type_from_board("esp32-c3-devkitm-1", None), "ESP32-C3");
    }

    #[test]
    fn test_platformio_stm32_board_manifest_detects_probe_rs_chip() {
        let manifest = serde_json::json!({
            "build": {
                "mcu": "stm32f103c8t6"
            },
            "debug": {
                "jlink_device": "STM32F103C8"
            },
            "name": "BluePill F103C8"
        });

        assert_eq!(
            chip_type_from_board("bluepill_f103c8", Some(&manifest)),
            "STM32F103C8"
        );
        assert_eq!(
            stm32_chip_type_from_candidate("stm32f103c8t6"),
            Some("STM32F103C8".to_string())
        );
    }

    #[test]
    fn test_platformio_flash_frequency_from_hz_literal() {
        assert_eq!(parse_flash_frequency("40000000L"), Some("40m".to_string()));
        assert_eq!(parse_flash_frequency("80000000L"), Some("80m".to_string()));
    }

    #[test]
    fn test_platformio_default_envs_parser_uses_first_env() {
        assert_eq!(
            parse_first_platformio_env("esp32s3_prod, esp32s3_debug"),
            Some("esp32s3_prod".to_string())
        );
        assert_eq!(
            parse_first_platformio_env(" esp32c3 "),
            Some("esp32c3".to_string())
        );
        assert_eq!(parse_first_platformio_env(""), None);
    }

    #[test]
    fn test_platformio_referenced_assets_include_partitions_and_variants() {
        let content = r#"
[platformio]
default_envs = prod

[env:prod]
board_build.partitions = partitions.csv
board_build.variants_dir = variants
board_build.arduino.custom_bootloader = bootloader_dio_80m.bin
"#;

        let assets = referenced_platformio_asset_paths(content);
        assert!(assets.contains(&PathBuf::from("partitions.csv")));
        assert!(assets.contains(&PathBuf::from("variants")));
        assert!(assets.contains(&PathBuf::from("bootloader_dio_80m.bin")));
    }

    #[test]
    fn test_offset_validation() {
        assert_eq!(parse_offset("0x0000").unwrap(), 0);
        assert_eq!(parse_offset("0x8000").unwrap(), 0x8000);
        assert_eq!(parse_offset("0xe000").unwrap(), 0xe000);
        assert_eq!(parse_offset("10000").unwrap(), 0x10000);
        assert_eq!(parse_offset("0x10000").unwrap(), 0x10000);

        assert!(parse_offset("0xG000").is_err());
        assert!(parse_offset("xyz").is_err());
    }

    #[test]
    fn test_manifest_validation() {
        let mut config = ProjectConfig::default();
        config.chip_type = "ESP32-S3".to_string();
        config.images = vec![
            FirmwareImage {
                label: "bootloader".to_string(),
                path: "".to_string(),
                offset: "0x0000".to_string(),
                required: true,
                encrypted: false,
                sha256: None,
            },
            FirmwareImage {
                label: "duplicate".to_string(),
                path: "".to_string(),
                offset: "0x0000".to_string(),
                required: true,
                encrypted: false,
                sha256: None,
            },
        ];

        let (results, errors) = config.validate_manifest();
        assert_eq!(results.len(), 2);

        assert!(
            errors
                .iter()
                .any(|e| e.contains("Required image") && e.contains("empty"))
        );
        assert!(errors.iter().any(|e| e.contains("Duplicate offset")));
    }

    #[test]
    fn test_optional_empty_manifest_slot_is_ignored() {
        let mut config = ProjectConfig::default();
        config.images = vec![FirmwareImage {
            label: "slot_1".to_string(),
            path: "".to_string(),
            offset: "0x0000".to_string(),
            required: false,
            encrypted: false,
            sha256: None,
        }];

        let (results, errors) = config.validate_manifest();
        assert_eq!(results.len(), 1);
        assert!(errors.is_empty());
        assert_eq!(results[0].label, "slot_1");
    }

    #[test]
    fn test_merged_manifest_mode_exposes_empty_editable_slot() {
        let config = ProjectConfig::default();
        let results = config.manifest_results_for_mode(true);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].label, "factory_merged");
        assert_eq!(results[0].offset, "0x0000");
        assert_eq!(results[0].path, "");
        assert!(results[0].error.is_none());
    }

    #[test]
    fn test_manifest_validation_accepts_stm32_probe_rs_target() {
        let temp_dir = std::env::temp_dir().join(format!(
            "piopulse_stm32_manifest_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&temp_dir).unwrap();
        let firmware_path = temp_dir.join("firmware.elf");
        std::fs::write(&firmware_path, [0x7f, b'E', b'L', b'F']).unwrap();

        let mut config = ProjectConfig::default();
        config.chip_type = "STM32F103C8".to_string();
        config.images = vec![FirmwareImage {
            label: "program".to_string(),
            path: firmware_path.to_string_lossy().to_string(),
            offset: "0x0000".to_string(),
            required: true,
            encrypted: false,
            sha256: None,
        }];

        let (_, errors) = config.validate_manifest();
        assert!(errors.is_empty(), "{errors:?}");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_sha256_validation() {
        use std::io::Write;

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("piopulse_test_file.bin");

        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"piopulse_test_data").unwrap();
        drop(file);

        let correct_sha256 = "2dfe6dc78f8322abbfb36af2086218ea051519582191588cb0ff0b38d74a6a37";
        let incorrect_sha256 = "0000000000000000000000000000000000000000000000000000000000000000";

        let mut config = ProjectConfig::default();
        config.images = vec![
            FirmwareImage {
                label: "test_correct".to_string(),
                path: file_path.to_string_lossy().to_string(),
                offset: "0x0000".to_string(),
                required: true,
                encrypted: false,
                sha256: Some(correct_sha256.to_string()),
            },
            FirmwareImage {
                label: "test_incorrect".to_string(),
                path: file_path.to_string_lossy().to_string(),
                offset: "0x1000".to_string(),
                required: true,
                encrypted: false,
                sha256: Some(incorrect_sha256.to_string()),
            },
        ];

        let (results, errors) = config.validate_manifest();

        assert_eq!(results[0].sha256_match, Some(true));
        assert_eq!(results[1].sha256_match, Some(false));

        assert!(errors.iter().any(|e| e.contains("SHA256 mismatch")));

        let _ = std::fs::remove_file(file_path);
    }

    #[test]
    fn test_esptool_command_generation() {
        let mut config = ProjectConfig::default();
        config.chip_type = "ESP32-S3".to_string();
        config.baud_rate = 921600;
        config.flash_mode = "dio".to_string();
        config.flash_freq = "80m".to_string();
        config.flash_size = "16MB".to_string();
        config.nvs_offset = "0x9000".to_string();
        config.do_not_chg_bin = false;

        config.images = vec![
            FirmwareImage {
                label: "bootloader".to_string(),
                path: "bootloader.bin".to_string(),
                offset: "0x0000".to_string(),
                required: true,
                encrypted: false,
                sha256: None,
            },
            FirmwareImage {
                label: "merged".to_string(),
                path: "factory_merged.bin".to_string(),
                offset: "0x0000".to_string(),
                required: true,
                encrypted: false,
                sha256: None,
            },
        ];

        let cmd_segmented = crate::worker::generate_esptool_command("/dev/ttyUSB0", &config, false);
        assert!(
            cmd_segmented.contains(
                "esptool.py --chip esp32s3 --port /dev/ttyUSB0 --baud 921600 write_flash"
            )
        );
        assert!(cmd_segmented.contains("--flash_mode dio"));
        assert!(cmd_segmented.contains("--flash_freq 80m"));
        assert!(cmd_segmented.contains("--flash_size 16MB"));
        assert!(cmd_segmented.contains("0x0000 bootloader.bin"));
        assert!(!cmd_segmented.contains("factory_merged.bin"));
        assert!(cmd_segmented.contains("0x9000 <dynamic_nvs.bin>"));

        let cmd_merged = crate::worker::generate_esptool_command("/dev/ttyUSB0", &config, true);
        assert!(cmd_merged.contains("0x0000 factory_merged.bin"));
        assert!(!cmd_merged.contains("bootloader.bin"));

        config.do_not_chg_bin = true;
        let cmd_dont_chg = crate::worker::generate_esptool_command("/dev/ttyUSB0", &config, true);
        assert!(cmd_dont_chg.contains("--flash_mode keep"));
        assert!(cmd_dont_chg.contains("--flash_freq keep"));
        assert!(cmd_dont_chg.contains("--flash_size keep"));
    }

    #[test]
    fn test_set_field_syncs_to_images() {
        let mut config = ProjectConfig::default();
        assert!(config.images.is_empty());

        // Set application firmware path
        config.set_field(13, "new_firmware.bin".to_string());
        assert_eq!(config.app_path, "new_firmware.bin");

        // The images list should now have one entry for firmware
        let fw_img = config
            .images
            .iter()
            .find(|img| img.label == "firmware")
            .unwrap();
        assert_eq!(fw_img.path, "new_firmware.bin");
        assert_eq!(fw_img.offset, "0x10000");

        // Change offset
        config.set_field(12, "0x20000".to_string());
        let fw_img2 = config
            .images
            .iter()
            .find(|img| img.label == "firmware")
            .unwrap();
        assert_eq!(fw_img2.offset, "0x20000");

        // Clear path should remove it from images
        config.set_field(13, "".to_string());
        assert!(
            config
                .images
                .iter()
                .find(|img| img.label == "firmware")
                .is_none()
        );
    }
}
