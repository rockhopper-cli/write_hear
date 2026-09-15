use anyhow::Result;

use crate::podman::client::WhisperSegment;

#[derive(Debug, Clone)]
pub struct Subtitle {
    pub index: usize,
    pub start: f64,
    pub end: f64,
    pub text: String,
}

/// Convert Whisper segments into formatted subtitles.
pub fn format(
    segments: &[WhisperSegment],
    max_chars_per_line: usize,
    two_lines: bool,
    min_gap_ms: u64, // <-- 1. Add parameter here
) -> Result<Vec<Subtitle>> {
    let mut subtitles = Vec::new();

    // 1. Collect and wrap text for all segments
    for segment in segments {
        let text = segment.text.trim();

        if text.is_empty() {
            continue;
        }

        let lines = wrap_text(
            text,
            max_chars_per_line,
            two_lines,
        );

        if lines.is_empty() {
            continue;
        }

        subtitles.push(Subtitle {
            index: subtitles.len() + 1,
            start: segment.start,
            end: segment.end,
            text: lines.join("\n"),
        });
    }

    // ========================================================
    // <-- 2. PLACE THE GAP ADJUSTMENT CODE HERE
    // Adjust gaps between consecutive subtitles to prevent sticking/overlapping
    // ========================================================
    let gap_secs = (min_gap_ms as f64) / 1000.0;

    for i in 0..subtitles.len().saturating_sub(1) {
        let next_start = subtitles[i + 1].start;
        if subtitles[i].end + gap_secs > next_start {
            // Enforce the gap, ensuring end never precedes its own start
            subtitles[i].end = (next_start - gap_secs).max(subtitles[i].start);
        }
    }

    Ok(subtitles)
}

/// Split text into lines without breaking words.
fn wrap_text(
    text: &str,
    max_chars: usize,
    two_lines: bool,
) -> Vec<String> {
    if max_chars == 0 {
        return vec![text.to_string()];
    }

    let words: Vec<&str> = text.split_whitespace().collect();

    let mut lines = Vec::new();
    let mut current = String::new();

    for word in words {
        let proposed_len = if current.is_empty() {
            word.len()
        } else {
            current.len() + 1 + word.len()
        };

        if proposed_len <= max_chars {
            if !current.is_empty() {
                current.push(' ');
            }

            current.push_str(word);
        } else {
            if !current.is_empty() {
                lines.push(current);
            }

            current = word.to_string();

            if two_lines && lines.len() == 2 {
                break;
            }
        }
    }

    if !current.is_empty() && (!two_lines || lines.len() < 2) {
        lines.push(current);
    }

    lines
}

/// Convert subtitles into SRT text.
pub fn to_srt(subtitles: &[Subtitle]) -> String {
    let mut output = String::new();

    for subtitle in subtitles {
        output.push_str(&format!(
            "{}\n",
            subtitle.index
        ));

        output.push_str(&format!(
            "{} --> {}\n",
            format_timestamp(subtitle.start),
            format_timestamp(subtitle.end)
        ));

        output.push_str(&subtitle.text);
        output.push_str("\n\n");
    }

    output
}

/// Format seconds into SRT timestamp:
///
/// HH:MM:SS,mmm
fn format_timestamp(seconds: f64) -> String {
    let total_ms = (seconds * 1000.0).round() as u64;

    let milliseconds = total_ms % 1000;
    let total_seconds = total_ms / 1000;

    let seconds = total_seconds % 60;
    let total_minutes = total_seconds / 60;

    let minutes = total_minutes % 60;
    let hours = total_minutes / 60;

    format!(
        "{:02}:{:02}:{:02},{:03}",
        hours,
        minutes,
        seconds,
        milliseconds
    )
}