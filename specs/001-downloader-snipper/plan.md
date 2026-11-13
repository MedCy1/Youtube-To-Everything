# Implementation Plan: downloader and precise cut

Branch: `001-downloader-snipper` | Date: 2026-11-12 | Spec: [spec.md](spec.md)

Input: functional specification from [spec.md](spec.md)

## Summary

An asynchronous Rust engine (`y2e-core`) drives `yt-dlp` and `ffmpeg`, and a native GPU interface is built with the Rune framework (`y2e-gui`). The application reads the metadata, downloads only the requested segment (Snipper) and offers four presets: maximum quality, Discord under 25 MB, MP3 audio and GIF.

## Technical context

- Language: Rust 2021
- Main dependencies:
  - `tokio` (async runtime for child processes)
  - `serde`, `serde_json` (deserialising the yt-dlp metadata)
  - `thiserror` (typed errors)
  - `tokio::sync` and `std::sync::mpsc` (channels between the engine and the UI)
  - `rune`, `rune-kit`, `rune-fx`, `rune-theme-opencode` (native GPU UI framework)
- External tools: `yt-dlp`, `ffmpeg`
- Testing: `cargo test --all`
- Target platform: desktop Linux (X11 and Wayland), macOS, Windows
- Project type: Cargo workspace
- Performance goals:
  - Interface starts in under 200 ms
  - GPU rendering at 60 FPS or better
  - Metadata read in under 2 s
  - 30 s clip from a long video in under 10 s

## Constitution check

- [x] I. Decoupled architecture: `y2e-core` is an independent library with no graphical dependency.
- [x] II. GPU performance and fluidity: Rune uses the GPU, and every transfer runs off the UI thread.
- [x] III. Respect for upstream tools: YouTube decoding goes through `yt-dlp`, cutting through `--download-sections`.
- [x] IV. Multi-frontend extensibility: `CoreEvent` can be consumed by Rune or by a terminal UI.
- [x] V. Clean code: no watermark or tool attribution in the repository files.

## Project structure

### Documentation (this feature)

```text
specs/001-downloader-snipper/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
└── tasks.md
```

### Source code layout

```text
Cargo.toml                      # Workspace root
crates/
├── y2e-core/                   # Shared engine library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── error.rs            # Typed errors
│       ├── metadata.rs         # VideoMetadata and JSON deserialisation
│       ├── progress.rs         # Parsing yt-dlp progress lines
│       ├── presets.rs          # Presets (Discord, MP3, GIF) and size estimates
│       ├── events.rs           # CoreEvent, sent from the engine to the UI
│       ├── tools.rs            # Finding, installing and updating yt-dlp and ffmpeg
│       └── engine.rs           # Tokio process runner
└── y2e-gui/                    # Native GPU UI (Rune)
    ├── Cargo.toml
    └── src/
        ├── main.rs
        ├── state.rs            # Application state
        ├── view/               # Landing, studio, timeline and dock views
        ├── gfx.rs              # GPU effects (aurora, export glow)
        ├── theme.rs            # Design tokens and colour palettes
        ├── tint.rs, artwork.rs # Hue extraction from the thumbnail
        └── link.rs, report.rs  # Clipboard link detection, readable errors
```
