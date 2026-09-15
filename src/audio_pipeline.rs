// src/audio_pipeline.rs
use crate::audio::transcriber;
use crate::podman::llm_client::LlmClient;
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::sync::mpsc::Sender;

#[derive(Clone, Debug)]
pub struct PipelineResult {
    pub transcript_path: String,
    pub transcript_text: String,
    pub summary_path: Option<String>,
    pub summary_text: Option<String>,
}

pub enum PipelineMessage {
    Log(String),
    Progress { progress: f32, text: String },
    Finished(Result<PipelineResult, String>),
}

pub enum TaskType {
    StandardTranscription,
}

fn format_timestamp(seconds: f64) -> String {
    let total = seconds as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}

pub fn run_audio_pipeline(
    wav_path: String,
    whisper_port: u16,
    qwen_port: u16,
    generate_summary: bool,
    task_type: TaskType,
    tx: Sender<PipelineMessage>,
) -> Result<PipelineResult, String> {
    let input_path = Path::new(&wav_path);
    let whisper_url = format!("http://127.0.0.1:{}/inference", whisper_port);

    // 1. Run core transcription engine (max_len = 0 for full natural sentences)
    let tx_clone = tx.clone();
    let segments = transcriber::transcribe_audio(
        input_path,
        &whisper_url,
        0,
        move |progress, text| {
            let _ = tx_clone.send(PipelineMessage::Progress { progress, text: text.clone() });
            let _ = tx_clone.send(PipelineMessage::Log(text));
        },
    )
    .map_err(|e| format!("{:#}", e))?;

    // 2. Format segments into timestamped text
    let mut full_transcript = String::new();
    for seg in segments {
        let line = format!("[{}] {}", format_timestamp(seg.start), seg.text);
        full_transcript.push_str(&line);
        full_transcript.push('\n');
    }

    let transcript_path = input_path.with_extension("transcript.txt");
    fs::write(&transcript_path, &full_transcript)
        .map_err(|e| format!("Failed to save transcript file: {:#}", e))?;

    let _ = tx.send(PipelineMessage::Log(format!(
        "Saved transcript to: {}",
        transcript_path.display()
    )));

    // 3. Optional LLM stage with Qwen 2.5 Coder (0.85 -> 1.0)
    let mut summary_path_out = None;
    let mut summary_text_out = None;

    if generate_summary {
        let (system_prompt, out_ext, label) = match task_type {
            TaskType::StandardTranscription => (
                "You are an expert transcriber and summarizer. \
                 Given the transcript, provide a clear executive summary followed by key bullet points.",
                "summary.txt",
                "Summary",
            ),
        };

        let _ = tx.send(PipelineMessage::Progress {
            progress: 0.88,
            text: format!("Querying Qwen 2.5 Coder for {}...", label),
        });
        let _ = tx.send(PipelineMessage::Log(format!(
            "Sending prompt to Qwen LLM on port {}...",
            qwen_port
        )));

        let llm = LlmClient::new(qwen_port);
        match llm.complete(system_prompt, &full_transcript) {
            Ok(summary) => {
                let s_path = input_path.with_extension(out_ext);
                let _ = fs::write(&s_path, &summary);
                let _ = tx.send(PipelineMessage::Log(format!(
                    "Successfully saved {} to: {}",
                    label,
                    s_path.display()
                )));
                summary_path_out = Some(s_path.display().to_string());
                summary_text_out = Some(summary);
            }
            Err(e) => {
                let _ = tx.send(PipelineMessage::Log(format!(
                    "Warning: LLM generation failed: {:#}. Transcript was preserved.",
                    e
                )));
            }
        }
    }

    Ok(PipelineResult {
        transcript_path: transcript_path.display().to_string(),
        transcript_text: full_transcript,
        summary_path: summary_path_out,
        summary_text: summary_text_out,
    })
}