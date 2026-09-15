// src/ui/transcription.rs
use crate::audio_pipeline::{self, PipelineMessage, PipelineResult, TaskType};
use crate::ui::config::AppConfig;
use eframe::egui;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

pub struct TranscriptionState {
    pub wav_file_path: Option<String>,
    pub generate_summary: bool,

    pub is_processing: bool,
    pub progress: f32,
    pub progress_text: String,
    pub logs: Vec<String>,

    pub result: Option<PipelineResult>,
    receiver: Option<Receiver<PipelineMessage>>,
}

impl Default for TranscriptionState {
    fn default() -> Self {
        Self {
            wav_file_path: None,
            generate_summary: false,
            is_processing: false,
            progress: 0.0,
            progress_text: "Ready".to_string(),
            logs: vec!["System ready.".to_string()],
            result: None,
            receiver: None,
        }
    }
}

pub fn show(state: &mut TranscriptionState, config: &AppConfig, ui: &mut egui::Ui) {
    // 1. Drain background channel messages
    if let Some(ref rx) = state.receiver {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                PipelineMessage::Log(line) => state.logs.push(line),
                PipelineMessage::Progress { progress, text } => {
                    state.progress = progress;
                    state.progress_text = text;
                }
                PipelineMessage::Finished(res) => {
                    state.is_processing = false;
                    match res {
                        Ok(res_data) => {
                            state.progress = 1.0;
                            state.progress_text = "Completed".to_string();
                            state.logs.push("Job completed successfully!".into());
                            state.result = Some(res_data);
                        }
                        Err(err) => {
                            state.progress = 0.0;
                            state.progress_text = "Failed".to_string();
                            state.logs.push(format!("Error: {}", err));
                        }
                    }
                }
            }
        }
    }

    if state.is_processing {
        ui.ctx().request_repaint();
    }

    egui::ScrollArea::vertical()
        .id_salt("transcription_main_scroll")
        .auto_shrink([false; 2])
        .show(ui, |ui| {

        // Header
        ui.heading(
            egui::RichText::new("Audio Transcription")
                .color(egui::Color32::LIGHT_GREEN)
                .strong(),
        );
        ui.label("Converts audio using FFmpeg and transcribes with Whisper.");
        ui.add_space(12.0);

        // 1. WAV File Picker
        ui.horizontal(|ui| {
            let pick_btn = ui.add_enabled(!state.is_processing, egui::Button::new("Select WAV File"));
            if pick_btn.clicked() {
                let mut dialog = rfd::FileDialog::new().add_filter("Audio", &["wav"]);
                if config.wav_default_path.exists() {
                    dialog = dialog.set_directory(&config.wav_default_path);
                }
                if let Some(path) = dialog.pick_file() {
                    state.wav_file_path = Some(path.display().to_string());
                }
            }

            if let Some(path) = &state.wav_file_path {
                ui.label(path);
            } else {
                ui.label(egui::RichText::new("No file selected.").color(egui::Color32::RED));
            }
        });
        ui.add_space(8.0);

        // Options
        ui.add_enabled(
            !state.is_processing,
            egui::Checkbox::new(&mut state.generate_summary, "Generate Summary with Qwen 2.5 Coder"),
        );
        ui.add_space(12.0);

        // Action Button
        let can_start = !state.is_processing && state.wav_file_path.is_some();
        if ui
            .add_enabled(
                can_start,
                egui::Button::new(if state.is_processing {
                    "Transcribing..."
                } else {
                    "Start Transcription"
                }),
            )
            .clicked()
        {
            if let Some(wav_path) = state.wav_file_path.clone() {
                state.is_processing = true;
                state.progress = 0.05;
                state.progress_text = "Starting...".into();
                state.logs.push(format!("--- Starting Transcription: {} ---", wav_path));
                state.result = None;

                let (tx, rx) = channel();
                state.receiver = Some(rx);

                let whisper_port = config.whisper_port;
                let qwen_port = config.qwen_port;
                let gen_sum = state.generate_summary;

                thread::spawn(move || {
                    let res = audio_pipeline::run_audio_pipeline(
                        wav_path,
                        whisper_port,
                        qwen_port,
                        gen_sum,
                        TaskType::StandardTranscription,
                        tx.clone(),
                    );
                    let _ = tx.send(PipelineMessage::Finished(res));
                });
            }
        }
        ui.add_space(10.0);

        // Progress Bar
        ui.horizontal(|ui| {
            ui.add(
                egui::ProgressBar::new(state.progress)
                    .text(&state.progress_text)
                    .animate(state.is_processing),
            );
        });
        ui.add_space(10.0);

        // Terminal Logs
        ui.label(egui::RichText::new("Process Logs:").strong());
        egui::Frame::canvas(ui.style())
            .fill(egui::Color32::from_rgb(20, 22, 25))
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)))
            .inner_margin(8.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(120.0)
                    .auto_shrink([false; 2])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        for log in &state.logs {
                            ui.label(
                                egui::RichText::new(log)
                                    .font(egui::FontId::monospace(12.0))
                                    .color(egui::Color32::from_rgb(210, 210, 210)),
                            );
                        }
                    });
            });

        // Output Result Viewer
        if let Some(res) = &state.result {
            ui.add_space(12.0);
            ui.separator();
            ui.heading("Transcription Results");

            // Display the path where the transcript was saved:
            ui.label(
                egui::RichText::new(format!("📁 Saved to: {}", res.transcript_path))
                    .weak()
                    .small(),
            );
            ui.add_space(8.0);

            if let Some(summary) = &res.summary_text {
                ui.label(egui::RichText::new("Summary:").strong());
                if let Some(path) = &res.summary_path {
                    ui.label(egui::RichText::new(format!("📁 Summary saved to: {}", path)).weak().small());
                }
                egui::ScrollArea::vertical()
                    .id_salt("summary_scroll")
                    .max_height(140.0)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(summary);
                    });
                ui.add_space(8.0);
            }

            ui.label(egui::RichText::new("Full Transcript:").strong());
            egui::ScrollArea::vertical()
                .id_salt("transcript_scroll")
                .max_height(180.0)
                .show(ui, |ui| {
                    ui.label(&res.transcript_text);
                });
        }

    });
}