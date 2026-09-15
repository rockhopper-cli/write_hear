// src/app.rs
use eframe::egui;
use crate::ui::{config, setup_model, subtitle_creator, transcription};

#[derive(PartialEq)]
enum Tab {
    SubtitleCreator,
    AudioTranscription,
    SetUpLocalModel,
    Configuration,
}

// src/app.rs

pub struct WriteHear {
    current_tab: Tab,
    subtitle_state: subtitle_creator::SubtitleState,
    transcription_state: transcription::TranscriptionState,
    pub config: config::AppConfig,
}

impl Default for WriteHear {
    fn default() -> Self {
        // 1. Load config into a local variable first
        let loaded_config = config::AppConfig::load();

        // 1. Startup: Launch enabled containers concurrently
        if loaded_config.manage_whisper_container {
            config::start_container(&loaded_config.whisper_container_name);
        }
        if loaded_config.manage_qwen_container {
            config::start_container(&loaded_config.qwen_container_name);
        }

        Self {
            current_tab: Tab::SubtitleCreator,
            subtitle_state: subtitle_creator::SubtitleState::new(&loaded_config),
            transcription_state: transcription::TranscriptionState::default(),
            config: loaded_config,
        }
    }
}

// 2. SHUTDOWN: Executed when the user closes the window
impl Drop for WriteHear {
    fn drop(&mut self) {
        if self.config.manage_whisper_container {
            config::stop_container(&self.config.whisper_container_name);
        }
        if self.config.manage_qwen_container {
            config::stop_container(&self.config.qwen_container_name);
        }
    }
}

impl eframe::App for WriteHear {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        
        // 1. Use the new unified `Panel::left` API
        egui::Panel::left("tabs_panel").show(ui, |ui| {
            ui.vertical(|ui| {
                ui.selectable_value(&mut self.current_tab, Tab::SubtitleCreator, "Subtitle Creator");
                ui.selectable_value(&mut self.current_tab, Tab::AudioTranscription, "Audio Transcription");
            
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(4.0);

                ui.selectable_value(&mut self.current_tab, Tab::SetUpLocalModel, "Set up local model");
                ui.selectable_value(&mut self.current_tab, Tab::Configuration, "Configuration");
            });
        });

        // 2. CentralPanel remains the same for the remaining space
        egui::CentralPanel::default().show(ui, |ui| {
            match self.current_tab {
                Tab::SubtitleCreator => subtitle_creator::show(&mut self.subtitle_state, &self.config, ui),
                Tab::AudioTranscription => transcription::show(&mut self.transcription_state, &self.config, ui),
                Tab::SetUpLocalModel => setup_model::show(ui),
                Tab::Configuration => config::show(&mut self.config, ui),
            }
        });
    }
}