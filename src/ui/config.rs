// src/ui/config.rs
use eframe::egui;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)] // Prevents crashing if older config files miss new fields
pub struct AppConfig {
    // General Audio & Subtitles
    pub wav_default_path: PathBuf,
    pub max_chars_per_line: usize,
    pub subtitle_gap_ms: u32,

    // Whisper Server Settings
    pub whisper_port: u16,
    pub manage_whisper_container: bool,
    pub whisper_container_name: String,

    // Qwen 2.5 Coder Settings
    pub qwen_port: u16,
    pub manage_qwen_container: bool,
    pub qwen_container_name: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            wav_default_path: dirs::audio_dir().unwrap_or_else(|| PathBuf::from(".")),
            max_chars_per_line: 42,
            subtitle_gap_ms: 100,

            // Whisper defaults
            whisper_port: 8081,
            manage_whisper_container: false,
            whisper_container_name: "whisper-server".to_string(),

            // Qwen defaults (defaulting to 8080 to avoid collision with 8081)
            qwen_port: 8080,
            manage_qwen_container: false,
            qwen_container_name: "qwen-coder".to_string(),
        }
    }
}

impl AppConfig {
    fn config_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("writehear");
        path.push("config.toml");
        path
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(cfg) = toml::from_str::<AppConfig>(&contents) {
                return cfg;
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let toml_str = toml::to_string_pretty(self)?;
        fs::write(path, toml_str)?;
        Ok(())
    }
}

/// Spawns `podman start <container>` in a background thread
pub fn start_container(container_name: &str) {
    let name = container_name.to_string();
    std::thread::spawn(move || {
        println!("[WriteHear] Starting Podman container: {}", name);
        match Command::new("podman").args(["start", &name]).output() {
            Ok(output) if output.status.success() => {
                println!("[WriteHear] Podman container '{}' started.", name);
            }
            Ok(output) => {
                eprintln!(
                    "[WriteHear] Error starting container '{}': {}",
                    name,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Err(e) => eprintln!("[WriteHear] Failed to invoke podman: {}", e),
        }
    });
}

/// Stops the container synchronously on exit (-t 3 gives a 3-second grace period)
pub fn stop_container(container_name: &str) {
    println!("[WriteHear] Stopping Podman container: {}", container_name);
    let _ = Command::new("podman")
        .args(["stop", "-t", "3", container_name])
        .status();
}

pub fn show(config: &mut AppConfig, ui: &mut egui::Ui) {
    ui.heading("Settings & Configuration");
    ui.add_space(10.0);

    let mut changed = false;

    // ----------------------------------------------------
    // 1. General Subtitle & Audio Settings
    // ----------------------------------------------------
    ui.label(egui::RichText::new("General Audio & Subtitle Defaults").strong());
    ui.add_space(4.0);

    egui::Grid::new("general_config_grid")
        .num_columns(2)
        .spacing([20.0, 12.0])
        .show(ui, |ui| {
            ui.label("Default WAV Directory:");
            ui.horizontal(|ui| {
                ui.monospace(config.wav_default_path.to_string_lossy().to_string());
                if ui.button("📁 Browse...").clicked() {
                    let mut dialog = rfd::FileDialog::new();
                    if config.wav_default_path.exists() {
                        dialog = dialog.set_directory(&config.wav_default_path);
                    }
                    if let Some(folder) = dialog.pick_folder() {
                        config.wav_default_path = folder;
                        changed = true;
                    }
                }
            });
            ui.end_row();

            ui.label("Max characters per line:");
            if ui.add(egui::Slider::new(&mut config.max_chars_per_line, 10..=80).text("chars")).changed() {
                changed = true;
            }
            ui.end_row();

            ui.label("Subtitle gap:");
            if ui.add(egui::DragValue::new(&mut config.subtitle_gap_ms).speed(10.0).suffix(" ms")).changed() {
                changed = true;
            }
            ui.end_row();
        });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    // ----------------------------------------------------
    // 2. Whisper Server Settings
    // ----------------------------------------------------
    ui.heading("Whisper Model (Speech-to-Text)");
    ui.add_space(4.0);

    egui::Grid::new("whisper_config_grid")
        .num_columns(2)
        .spacing([20.0, 12.0])
        .show(ui, |ui| {
            ui.label("Whisper Port:");
            ui.vertical(|ui| {
                if ui.add(egui::DragValue::new(&mut config.whisper_port).range(1024..=65535)).changed() {
                    changed = true;
                }
                ui.label(
                    egui::RichText::new(format!("Inference URL: http://127.0.0.1:{}/inference", config.whisper_port))
                        .weak()
                        .small(),
                );
            });
            ui.end_row();
        });

    ui.add_space(6.0);
    if ui.checkbox(&mut config.manage_whisper_container, "Manage local Whisper container").changed() {
        changed = true;
    }
    ui.label(
        egui::RichText::new("Start container on startup and stop on exit")
            .weak()
            .small(),
    );

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("Container name:");
        let response = ui.add_enabled(
            config.manage_whisper_container,
            egui::TextEdit::singleline(&mut config.whisper_container_name).hint_text("whisper-server"),
        );
        if response.changed() {
            changed = true;
        }

        if config.manage_whisper_container {
            if ui.button("▶ Start").clicked() {
                start_container(&config.whisper_container_name);
            }
            if ui.button("⏹ Stop").clicked() {
                let name = config.whisper_container_name.clone();
                std::thread::spawn(move || stop_container(&name));
            }
        }
    });

    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);

    // ----------------------------------------------------
    // 3. Qwen 2.5 Coder Settings
    // ----------------------------------------------------
    ui.heading("Qwen 2.5 Coder (Notes & Summarization)");
    ui.add_space(4.0);

    egui::Grid::new("qwen_config_grid")
        .num_columns(2)
        .spacing([20.0, 12.0])
        .show(ui, |ui| {
            ui.label("Qwen LLM Port:");
            ui.vertical(|ui| {
                if ui.add(egui::DragValue::new(&mut config.qwen_port).range(1024..=65535)).changed() {
                    changed = true;
                }
                ui.label(
                    egui::RichText::new(format!("Endpoint: http://127.0.0.1:{}/v1/chat/completions", config.qwen_port))
                        .weak()
                        .small(),
                );
            });
            ui.end_row();
        });

    ui.add_space(6.0);
    if ui.checkbox(&mut config.manage_qwen_container, "Manage local Qwen container").changed() {
        changed = true;
    }
    ui.label(
        egui::RichText::new("Start container on startup and stop on exit")
            .weak()
            .small(),
    );

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label("Container name:");
        let response = ui.add_enabled(
            config.manage_qwen_container,
            egui::TextEdit::singleline(&mut config.qwen_container_name).hint_text("qwen-coder"),
        );
        if response.changed() {
            changed = true;
        }

        if config.manage_qwen_container {
            if ui.button("▶ Start").clicked() {
                start_container(&config.qwen_container_name);
            }
            if ui.button("⏹ Stop").clicked() {
                let name = config.qwen_container_name.clone();
                std::thread::spawn(move || stop_container(&name));
            }
        }
    });

    // Auto-save changes
    if changed {
        if let Err(e) = config.save() {
            eprintln!("Failed to save config: {e}");
        }
    }
}