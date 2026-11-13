# Tasks: downloader and precise cut

Input: specification and design documents from `/specs/001-downloader-snipper/`
Prerequisites: `spec.md`, `plan.md`, `research.md`, `data-model.md`

## Phase 1: Setup (Shared Infrastructure)

Purpose: initialise the Cargo workspace and set up the dependencies.

- [x] T001 Create Cargo workspace configuration in `Cargo.toml`
- [x] T002 [P] Create `crates/y2e-core` package with `tokio`, `serde`, `serde_json`, `thiserror` in `crates/y2e-core/Cargo.toml`
- [x] T003 [P] Create `crates/y2e-gui` package with Rune dependencies in `crates/y2e-gui/Cargo.toml`

---

## Phase 2: Foundational (Blocking Prerequisites)

Purpose: core data structures, error types and asynchronous communication.

- [x] T004 Define strongly typed errors in `crates/y2e-core/src/error.rs`
- [x] T005 [P] Define `VideoMetadata` and `VideoChapter` models with serde deserializers in `crates/y2e-core/src/metadata.rs`
- [x] T006 [P] Implement `ProgressUpdate` parser for yt-dlp stdout lines in `crates/y2e-core/src/progress.rs`
- [x] T007 Define `CoreEvent` enum and communication channels in `crates/y2e-core/src/events.rs`

Checkpoint: the data and event-flow foundation is ready.

---

## Phase 3: User Story 1 - Download and metadata (Priority: P1, MVP)

Goal: read the metadata and download in maximum quality, in the background.

- [x] T008 [US1] Implement `fetch_metadata(url: &str)` via `yt-dlp --dump-json` in `crates/y2e-core/src/engine.rs`
- [x] T009 [US1] Implement full video download execution with real-time stdout progress streaming in `crates/y2e-core/src/engine.rs`
- [x] T010 [US1] Write unit tests for metadata parsing and progress parsing in `crates/y2e-core/src/progress.rs`

Checkpoint: the core can inspect and download a whole video asynchronously.

---

## Phase 4: User Story 2 - Precise cut (Priority: P2)

Goal: extract a time segment without downloading the rest of the stream.

- [x] T011 [US2] Implement `TimeRange` structure and validation methods in `crates/y2e-core/src/metadata.rs`
- [x] T012 [US2] Add `--download-sections` and `--force-keyframes-at-cuts` options to `download()` runner in `crates/y2e-core/src/engine.rs`
- [x] T013 [US2] Write unit tests for time range argument formatting in `crates/y2e-core/src/presets.rs`

Checkpoint: the Snipper downloads 30 seconds of a video in a few seconds.

---

## Phase 5: User Story 3 - Presets (Priority: P3)

Goal: support the Discord (< 25 MB), audio (MP3) and GIF presets.

- [x] T014 [US3] Implement `DownloadPreset` enum and command builder in `crates/y2e-core/src/presets.rs`
- [x] T015 [US3] Implement dynamic bitrate calculation and FFmpeg 2-pass/transcode for Discord (< 25 MB) in `crates/y2e-core/src/presets.rs`
- [x] T016 [US3] Implement Audio extraction (`yt-dlp -x --audio-format mp3`) with embedded thumbnail in `crates/y2e-core/src/presets.rs`
- [x] T017 [US3] Implement GIF generation via FFmpeg palette in `crates/y2e-core/src/presets.rs`

---

## Phase 6: native GPU interface with Rune (`y2e-gui`)

Goal: native graphical interface with Rune.

- [x] T018 Define GUI State and actions in `crates/y2e-gui/src/state.rs`
- [x] T019 Implement Dark theme and typography tokens in `crates/y2e-gui/src/theme.rs`
- [x] T020 Build URL input component with Paste button in `crates/y2e-gui/src/view/`
- [x] T021 Build Metadata preview card with spring transitions in `crates/y2e-gui/src/view/`
- [x] T022 Build Snipper interactive range controls in `crates/y2e-gui/src/view/`
- [x] T023 Build Preset selector cards with bouncy feedback in `crates/y2e-gui/src/view/`
- [x] T024 Build animated progress bar connected to `CoreEvent` in `crates/y2e-gui/src/view/`
- [x] T025 Connect Tokio async channel to Rune application event loop in `crates/y2e-gui/src/main.rs`
- [x] T026 Build and verify complete application with `cargo test --all` and `cargo check --all`
