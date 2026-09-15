use eframe::egui;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use crate::ui::config::AppConfig;

/// Messages sent from the background worker thread to egui
pub enum PipelineMessage {
    Log(String),
    Progress { progress: f32, text: String },
    Finished(Result<String, String>),
}

pub struct SubtitleState {
    pub wav_file_path: Option<String>,
    pub max_chars_per_line: usize,
    pub two_lines_subtitles: bool,
    pub min_gap_ms: u64,

    // UI Async Tracking
    pub is_processing: bool,
    pub progress: f32,
    pub progress_text: String,
    pub logs: Vec<String>,
    receiver: Option<Receiver<PipelineMessage>>,
}

impl Default for SubtitleState {
    fn default() -> Self {
        Self {
            wav_file_path: None,
            max_chars_per_line: 42,
            two_lines_subtitles: true,
            min_gap_ms: 100,
            is_processing: false,
            progress: 0.0,
            progress_text: String::from("Ready"),
            logs: vec![String::from("System ready.")],
            receiver: None,
        }
    }
}

impl SubtitleState {
    /// Initialize with values loaded from the user's config file
    pub fn new(config: &AppConfig) -> Self {
        Self {
            max_chars_per_line: config.max_chars_per_line,
            min_gap_ms: config.subtitle_gap_ms as u64,
            ..Default::default()
        }
    }
}

pub fn show(state: &mut SubtitleState, config: &AppConfig, ui: &mut egui::Ui) {
    // --------------------------------------------------
    // Drain background channel messages & keep UI active
    // --------------------------------------------------
    if let Some(ref rx) = state.receiver {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                PipelineMessage::Log(line) => {
                    state.logs.push(line);
                }
                PipelineMessage::Progress { progress, text } => {
                    state.progress = progress;
                    state.progress_text = text;
                }
                PipelineMessage::Finished(result) => {
                    state.is_processing = false;
                    match result {
                        Ok(srt_path) => {
                            state.progress = 1.0;
                            state.progress_text = "Completed".to_string();
                            state.logs.push(format!("Saved SRT to: {}", srt_path));
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

    // Force UI continuous redraw while processing so progress bars and logs update smoothly
    if state.is_processing {
        ui.ctx().request_repaint();
    }

    // --------------------------------------------------
    // Header
    // --------------------------------------------------
    ui.heading(
        egui::RichText::new("Subtitle Creator")
            .color(egui::Color32::from_rgb(100, 180, 255))
            .strong(),
    );

    ui.add_space(5.0);
    ui.label(
        egui::RichText::new("Generate non-overlapping, formatted subtitles using Whisper.")
            .color(egui::Color32::from_rgb(180, 180, 180)),
    );
    ui.add_space(15.0);

    // --------------------------------------------------
    // 1. WAV File Picker (Uses config.wav_default_path)
    // --------------------------------------------------
    ui.horizontal(|ui| {
        let pick_btn = ui.add_enabled(!state.is_processing, egui::Button::new("Select WAV File"));
        if pick_btn.clicked() {
            let mut dialog = rfd::FileDialog::new().add_filter("Audio", &["wav"]);
            
            // Open to the configured default directory
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
    ui.add_space(10.0);

    // --------------------------------------------------
    // 2. Settings (Max chars, 2 lines, Min Gap)
    // --------------------------------------------------
    ui.horizontal(|ui| {
        ui.label("Max characters per line (max 80):");
        ui.add_enabled(
            !state.is_processing,
            egui::DragValue::new(&mut state.max_chars_per_line).range(10..=80),
        );
    });
    ui.add_space(8.0);

    ui.add_enabled(
        !state.is_processing,
        egui::Checkbox::new(&mut state.two_lines_subtitles, "Limit to 2 lines per subtitle"),
    );
    ui.add_space(8.0);

    ui.horizontal(|ui| {
        ui.label("Gap between subtitles:");
        ui.add_enabled(
            !state.is_processing,
            egui::DragValue::new(&mut state.min_gap_ms)
                .range(0..=500)
                .suffix(" ms"),
        );
        ui.label(
            egui::RichText::new("(prevents subtitles from sticking/overlapping)")
                .weak()
                .size(11.0),
        );
    });
    ui.add_space(8.0);

    // Reset button to restore to config defaults
    if !state.is_processing && ui.button("↺ Reset to Config Defaults").clicked() {
        state.max_chars_per_line = config.max_chars_per_line;
        state.min_gap_ms = config.subtitle_gap_ms as u64;
    }

    ui.add_space(15.0);

    // --------------------------------------------------
    // 3. Action Button
    // --------------------------------------------------
    let can_start = !state.is_processing && state.wav_file_path.is_some();
    let generate_btn = ui.add_enabled(
        can_start,
        egui::Button::new(if state.is_processing {
            "Processing..."
        } else {
            "Generate Subtitles"
        }),
    );

    if generate_btn.clicked() {
        if let Some(wav_path) = state.wav_file_path.clone() {
            state.is_processing = true;
            state.progress = 0.05;
            state.progress_text = "Starting...".to_string();
            state.logs.push(format!("--- Starting job for: {} ---", wav_path));

            let (tx, rx): (Sender<PipelineMessage>, Receiver<PipelineMessage>) = channel();
            state.receiver = Some(rx);

            let max_chars = state.max_chars_per_line;
            let two_lines = state.two_lines_subtitles;
            let min_gap_ms = state.min_gap_ms;
            
            // Build the URL using config.port
            let inference_url = format!("http://127.0.0.1:{}/inference", config.whisper_port);

            // Run in background thread to keep UI responsive
            thread::spawn(move || {
                let result = run_pipeline_task(wav_path, inference_url, max_chars, two_lines, min_gap_ms, tx.clone());
                let _ = tx.send(PipelineMessage::Finished(result));
            });
        }
    }

    ui.add_space(15.0);

    // --------------------------------------------------
    // Progress Bar
    // --------------------------------------------------
    ui.horizontal(|ui| {
        let progress_bar = egui::ProgressBar::new(state.progress)
            .text(&state.progress_text)
            .animate(state.is_processing);
        ui.add(progress_bar);
    });

    ui.add_space(15.0);

    // --------------------------------------------------
    // Logs Window
    // --------------------------------------------------
    ui.label(egui::RichText::new("Process Logs:").strong());
    
    egui::Frame::canvas(ui.style())
        .fill(egui::Color32::from_rgb(20, 22, 25))
        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(60, 60, 60)))
        .inner_margin(8.0)
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .max_height(140.0)
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
}

/// Helper executed inside the worker thread
fn run_pipeline_task(
    wav_path: String,
    inference_url: String, // Dynamic URL from config
    max_chars: usize,
    two_lines: bool,
    min_gap_ms: u64,
    tx: Sender<PipelineMessage>,
) -> Result<String, String> {
    let srt = crate::subtitle_pipeline::generate_subtitles(
        &wav_path,
        &inference_url,
        max_chars,
        two_lines,
        min_gap_ms,
        tx.clone(),
    )
    .map_err(|e| format!("{:#}", e))?;

    // Save SRT right next to the original WAV file
    let srt_path = std::path::Path::new(&wav_path).with_extension("srt");
    std::fs::write(&srt_path, srt).map_err(|e| format!("Failed to save SRT file: {}", e))?;

    Ok(srt_path.display().to_string())
}