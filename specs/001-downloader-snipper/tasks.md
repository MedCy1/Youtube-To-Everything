# Tasks: téléchargeur et découpe précise

Input: Spécifications et documents de conception issus de `/specs/001-downloader-snipper/`
Prerequisites: `spec.md`, `plan.md`, `research.md`, `data-model.md`

## Phase 1: Setup (Shared Infrastructure)

Purpose: Initialisation du workspace Cargo et configuration des dépendances.

- [x] T001 Create Cargo workspace configuration in `Cargo.toml`
- [x] T002 [P] Create `crates/y2e-core` package with `tokio`, `serde`, `serde_json`, `thiserror` in `crates/y2e-core/Cargo.toml`
- [x] T003 [P] Create `crates/y2e-gui` package with Rune dependencies in `crates/y2e-gui/Cargo.toml`

---

## Phase 2: Foundational (Blocking Prerequisites)

Purpose: Structures de données fondamentales, types d'erreurs et communication asynchrone.

- [x] T004 Define strongly typed errors in `crates/y2e-core/src/error.rs`
- [x] T005 [P] Define `VideoMetadata` and `VideoChapter` models with serde deserializers in `crates/y2e-core/src/metadata.rs`
- [x] T006 [P] Implement `ProgressUpdate` parser for yt-dlp stdout lines in `crates/y2e-core/src/progress.rs`
- [x] T007 Define `CoreEvent` enum and communication channels in `crates/y2e-core/src/events.rs`

Checkpoint: La fondation de données et de flux d'événements est prête.

---

## Phase 3: User Story 1 - Téléchargement et métadonnées (Priority: P1, MVP)

Goal: Lecture des métadonnées et téléchargement en qualité maximale, en tâche de fond.

- [x] T008 [US1] Implement `fetch_metadata(url: &str)` via `yt-dlp --dump-json` in `crates/y2e-core/src/engine.rs`
- [x] T009 [US1] Implement full video download execution with real-time stdout progress streaming in `crates/y2e-core/src/engine.rs`
- [x] T010 [US1] Write unit tests for metadata parsing and progress parsing in `crates/y2e-core/src/progress.rs`

Checkpoint: Le Core est capable d'inspecter et de télécharger une vidéo complète de manière asynchrone.

---

## Phase 4: User Story 2 - Découpe précise (Priority: P2)

Goal: Extraction d'un extrait temporel sans télécharger le reste du flux.

- [x] T011 [US2] Implement `TimeRange` structure and validation methods in `crates/y2e-core/src/metadata.rs`
- [x] T012 [US2] Add `--download-sections` and `--force-keyframes-at-cuts` options to `download()` runner in `crates/y2e-core/src/engine.rs`
- [x] T013 [US2] Write unit tests for time range argument formatting in `crates/y2e-core/src/presets.rs`

Checkpoint: Le Snipper permet de télécharger 30 secondes d'une vidéo en quelques secondes.

---

## Phase 5: User Story 3 - Presets (Priority: P3)

Goal: Prise en charge des presets Discord (< 25 Mo), Audio Podcast (MP3), et GIF.

- [x] T014 [US3] Implement `DownloadPreset` enum and command builder in `crates/y2e-core/src/presets.rs`
- [x] T015 [US3] Implement dynamic bitrate calculation and FFmpeg 2-pass/transcode for Discord (<25 Mo) in `crates/y2e-core/src/presets.rs`
- [x] T016 [US3] Implement Audio extraction (`yt-dlp -x --audio-format mp3`) with embedded thumbnail in `crates/y2e-core/src/presets.rs`
- [x] T017 [US3] Implement GIF generation via FFmpeg palette in `crates/y2e-core/src/presets.rs`

---

## Phase 6: GUI native GPU avec Rune (`y2e-gui`)

Goal: Interface graphique native avec Rune.

- [x] T018 Define GUI State and actions in `crates/y2e-gui/src/state.rs`
- [x] T019 Implement Dark theme and typography tokens in `crates/y2e-gui/src/theme.rs`
- [x] T020 Build URL input component with Paste button in `crates/y2e-gui/src/view/`
- [x] T021 Build Metadata preview card with spring transitions in `crates/y2e-gui/src/view/`
- [x] T022 Build Snipper interactive range controls in `crates/y2e-gui/src/view/`
- [x] T023 Build Preset selector cards with bouncy feedback in `crates/y2e-gui/src/view/`
- [x] T024 Build animated progress bar connected to `CoreEvent` in `crates/y2e-gui/src/view/`
- [x] T025 Connect Tokio async channel to Rune application event loop in `crates/y2e-gui/src/main.rs`
- [x] T026 Build and verify complete application with `cargo test --all` and `cargo check --all`
