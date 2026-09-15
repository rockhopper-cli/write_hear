<div align="center">

  <img src="assets/logo.png" alt="WriteHear Logo" width="128" />

  # WriteHear

  **Privacy-First, 100% Offline Audio Transcriber, Subtitle Generator & Meeting Summarizer**

  [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
  [![Rust: 2024](https://img.shields.io/badge/Rust-2024_Edition-orange.svg)](https://www.rust-lang.org/)
  [![GUI: egui](https://img.shields.io/badge/GUI-egui%20%2F%20eframe-lightblue.svg)](https://github.com/emilk/egui)
  [![Platform: Linux](https://img.shields.io/badge/Platform-Linux-green.svg)](https://www.kernel.org/)

</div>

---

## 📖 Overview

**WriteHear** is a native, privacy-focused Linux desktop application built with **Rust** and **egui**. It allows you to transcribe audio recordings, generate subtitle files, and create structured meeting notes completely **offline**—with zero cloud dependencies.

By orchestrating **FFmpeg** for audio extraction and containerized AI models (**Whisper** for speech-to-text and **Qwen 2.5** for summarization) via **Podman**, WriteHear guarantees that your audio, transcripts, and confidential meeting discussions never leave your machine.

---

## ✨ Features

- 🎙️ **Offline Audio Transcription**  
  Convert any audio or video file with FFmpeg and transcribe it locally using Whisper with real-time progress updates and process logs.

- ⏱️ **Subtitle Creator**  
  Generate timestamped subtitle files (SRT / VTT) formatted for video editing and accessibility workflows.

- 📝 **AI Meeting Notes & Summarization**  
  Extract key takeaways, action items, and executive summaries directly from transcripts using local instances of Qwen 2.5 Coder.

- 🐳 **Containerized AI Backends (Podman)**  
  Isolate and manage local Whisper and LLM server containers seamlessly without cluttering your host system.

- ⚡ **Lightweight & Snappy GUI**  
  Instant startup and low memory footprint powered by `egui` and `eframe`.

- 🔒 **Zero Telemetry, 100% Air-Gapped**  
  Designed for privacy-conscious developers, journalists, researchers, and enterprise environments.

---

## 🛠️ Architecture & Workflow

```text
[ Audio / Video File ]
          │
          ▼
   [ FFmpeg Audio Extraction ]  ──▶  16kHz WAV
          │
          ▼
 [ Podman: Whisper API ]       ──▶  Raw Transcript / Timestamps
          │
          ├──▶ [ Subtitle Formatter ]  ──▶ .srt / .vtt File
          │
          ▼
 [ Podman: Qwen 2.5 API ]      ──▶  Summary & Action Items

```

---

## 📋 Prerequisites

Before running WriteHear, ensure the following tools are installed on your Linux system:

### 1. FFmpeg
Used for audio extraction and conversion:
* **openSUSE Tumbleweed:** `sudo zypper install ffmpeg`
* **Fedora:** `sudo dnf install ffmpeg`
* **Ubuntu/Debian:** `sudo apt install ffmpeg`

### 2. Podman
Used to run the containerized Whisper and LLM backends:
* **openSUSE Tumbleweed:** `sudo zypper install podman`
* **Fedora:** `sudo dnf install podman`
* **Ubuntu/Debian:** `sudo apt install podman`

### 3. Rust Toolchain (For Building from Source)
Requires Rust 1.85+ (Rust 2024 edition):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## 🚀 Installation & Running

### Running in Development Mode
```bash
# Clone the repository
git clone https://github.com/rockhopper-cli/write_hear.git
cd write_hear

# Run the app
cargo run
```

### Compiling Release Binary
```bash
cargo build --release
./target/release/writehear
```

---

## 📦 Packaging as an RPM (openSUSE / Fedora)

WriteHear includes metadata for `cargo-generate-rpm`:

```bash
# 1. Install cargo-generate-rpm
cargo install cargo-generate-rpm

# 2. Build the optimized release binary
cargo build --release

# 3. Strip symbols to reduce RPM size
strip -s target/release/writehear

# 4. Generate the RPM package
cargo generate-rpm
```

The generated package will be placed in `target/generate-rpm/writehear-*.rpm`. Install it using Zypper:
```bash
sudo zypper install target/generate-rpm/writehear-*.rpm
```

---

## 🗂️ Project Structure

```text
write_hear/
├── assets/
│   ├── InclusiveSans-VariableFont_wght.ttf   # Application typography
│   ├── logo.png                              # App icon / branding
│   └── writehear.desktop                     # Desktop launcher configuration
├── src/
│   ├── main.rs                               # Application entrypoint
│   ├── app.rs                                # Central egui application state
│   ├── audio/                                # FFmpeg process wrapper & transcriber
│   ├── audio_pipeline.rs                     # Transcription pipeline orchestration
│   ├── subtitle_pipeline.rs                  # Subtitle pipeline orchestration
│   ├── subtitles/                            # SRT/VTT formatters
│   ├── podman/                               # Podman client & LLM HTTP connectors
│   └── ui/                                   # egui UI views & components
│       ├── config.rs                         # User configurations & port bindings
│       ├── meeting_notes.rs                  # Meeting summarization panel
│       ├── setup_model.rs                    # Container setup & model manager
│       ├── subtitle_creator.rs               # Subtitle generator tab
│       └── transcription.rs                  # Audio transcription tab
├── Cargo.toml
└── LICENSE
```

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).