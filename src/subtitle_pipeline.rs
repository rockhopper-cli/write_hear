// src/subtitle_pipeline.rs
use anyhow::Result;
use std::path::Path;
use std::sync::mpsc::Sender;

use crate::audio::transcriber;
use crate::subtitles::formatter;
use crate::ui::subtitle_creator::PipelineMessage;

pub fn generate_subtitles<P: AsRef<Path>>(
    wav_path: P,
    whisper_endpoint: &str,
    max_chars_per_line: usize,
    two_lines: bool,
    min_gap_ms: u64,
    tx: Sender<PipelineMessage>,
) -> Result<String> {
    let wav_path = wav_path.as_ref();

    // 1. Run core transcription engine
    let segments = transcriber::transcribe_audio(
        wav_path,
        whisper_endpoint,
        max_chars_per_line,
        |progress, text| {
            let _ = tx.send(PipelineMessage::Progress { progress, text: text.clone() });
            let _ = tx.send(PipelineMessage::Log(text));
        },
    )?;

    // 2. Subtitle formatting stage (0.85 -> 1.0)
    let _ = tx.send(PipelineMessage::Progress {
        progress: 0.90,
        text: "Formatting subtitles & adjusting gaps...".into(),
    });
    let _ = tx.send(PipelineMessage::Log("Formatting subtitles & adjusting gaps...".into()));

    let subtitles = formatter::format(&segments, max_chars_per_line, two_lines, min_gap_ms)?;
    let srt = formatter::to_srt(&subtitles);

    Ok(srt)
}