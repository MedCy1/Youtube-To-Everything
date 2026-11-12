# Tasks: Core Downloader & Precision Snipper

**Input**: Spécifications et documents de conception issus de `/specs/001-downloader-snipper/`
**Prerequisites**: `spec.md`, `plan.md`, `research.md`, `data-model.md`

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Initialisation du workspace Cargo et configuration des dépendances.

- [ ] T001 Create Cargo workspace configuration in `Cargo.toml`
- [ ] T002 [P] Create `crates/y2e-core` package with `tokio`, `serde`, `serde_json`, `thiserror`, `regex` in `crates/y2e-core/Cargo.toml`
- [ ] T003 [P] Create `crates/y2e-gui` package with Rune dependencies in `crates/y2e-gui/Cargo.toml`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Structures de données fondamentales, types d'erreurs et communication asynchrone.

- [ ] T004 Define strongly typed errors in `crates/y2e-core/src/error.rs`
- [ ] T005 [P] Define `VideoMetadata` and `VideoChapter` models with serde deserializers in `crates/y2e-core/src/metadata.rs`
- [ ] T006 [P] Implement `ProgressUpdate` parser for yt-dlp stdout lines in `crates/y2e-core/src/progress.rs`
- [ ] T007 Define `CoreEvent` enum and communication channels in `crates/y2e-core/src/events.rs`

**Checkpoint**: La fondation de données et de flux d'événements est prête.

---

## Phase 3: User Story 1 - Fast Downloader & Metadata (Priority: P1) 🎯 MVP

**Goal**: Inspection instantanée des métadonnées et téléchargement qualité maximale en tâche de fond.

- [ ] T008 [US1] Implement `fetch_metadata(url: &str)` via `yt-dlp --dump-json` in `crates/y2e-core/src/engine.rs`
- [ ] T009 [US1] Implement full video download execution with real-time stdout progress streaming in `crates/y2e-core/src/engine.rs`
- [ ] T010 [US1] Write unit tests for metadata parsing and progress parsing in `crates/y2e-core/tests/unit_tests.rs`

**Checkpoint**: Le Core est capable d'inspecter et de télécharger une vidéo complète de manière asynchrone.

---

## Phase 4: User Story 2 - Precision Snipper (Priority: P2)

**Goal**: Extraction chirurgicale d'un extrait temporel sans télécharger le reste du flux.

- [ ] T011 [US2] Implement `TimeRange` structure and validation methods in `crates/y2e-core/src/metadata.rs`
- [ ] T012 [US2] Add `--download-sections` and `--force-keyframes-at-cuts` options to `download()` runner in `crates/y2e-core/src/engine.rs`
- [ ] T013 [US2] Write unit tests for time range argument formatting in `crates/y2e-core/tests/snipper_tests.rs`

**Checkpoint**: Le Snipper permet de télécharger 30 secondes d'une vidéo en quelques secondes.

---

## Phase 5: User Story 3 - Smart Presets (Priority: P3)

**Goal**: Prise en charge des presets Discord (< 25 Mo), Audio Podcast (MP3), et GIF.

- [ ] T014 [US3] Implement `DownloadPreset` enum and command builder in `crates/y2e-core/src/presets.rs`
- [ ] T015 [US3] Implement dynamic bitrate calculation and FFmpeg 2-pass/transcode for Discord (<25 Mo) in `crates/y2e-core/src/presets.rs`
- [ ] T016 [US3] Implement Audio extraction (`yt-dlp -x --audio-format mp3`) with embedded thumbnail in `crates/y2e-core/src/presets.rs`
- [ ] T017 [US3] Implement GIF generation via FFmpeg palette in `crates/y2e-core/src/presets.rs`

---

## Phase 6: Rune Native GPU GUI (`y2e-gui`)

**Goal**: Interface graphique native ultra-réactive et tactile avec Rune.

- [ ] T018 Define GUI State and actions in `crates/y2e-gui/src/state.rs`
- [ ] T019 Implement modern Dark theme and typography tokens in `crates/y2e-gui/src/theme.rs`
- [ ] T020 Build URL input component with Paste button in `crates/y2e-gui/src/views/input_bar.rs`
- [ ] T021 Build Metadata & Thumbnail preview card with spring transitions in `crates/y2e-gui/src/views/preview_card.rs`
- [ ] T022 Build Snipper interactive range slider component in `crates/y2e-gui/src/views/snipper.rs`
- [ ] T023 Build Preset selector buttons with bouncy feedback in `crates/y2e-gui/src/views/presets_bar.rs`
- [ ] T024 Build animated progress bar connected to `CoreEvent` in `crates/y2e-gui/src/views/progress_view.rs`
- [ ] T025 Connect Tokio async channel to Rune application event loop in `crates/y2e-gui/src/main.rs`
- [ ] T026 Build and verify complete application with `cargo run --bin y2e-gui`
