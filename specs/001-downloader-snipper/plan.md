# Implementation Plan: Core Downloader & Precision Snipper

**Branch**: `001-downloader-snipper` | **Date**: 2026-11-12 | **Spec**: [specs/001-downloader-snipper/spec.md](spec.md)

**Input**: Spécification fonctionnelle issue de `/specs/001-downloader-snipper/spec.md`

## Summary

Implémentation d'une solution de téléchargement et d'extraction vidéo moderne composée d'un moteur asynchrone Rust pur (`y2e-core`) wrappant `yt-dlp` et `ffmpeg`, et d'une interface graphique native GPU conçue avec le framework **Rune** (`y2e-gui`). La solution prend en charge l'inspection immédiate des métadonnées, le découpage temporel chirurgical sans surcoût réseau (Snipper) et 4 presets clés (Qualité Maximale, Discord < 25 Mo, Audio Podcast, et GIF).

## Technical Context

- **Language/Version**: Rust 2021/2024 (rustc 1.85+)
- **Primary Dependencies**: 
  - `tokio` (runtime asynchrone pour sous-processus)
  - `serde`, `serde_json` (désérialisation des métadonnées yt-dlp)
  - `thiserror` (gestion d'erreurs typées)
  - `crossbeam-channel` / `tokio::sync` (canaux de communication Core <-> UI)
  - `rune`, `rune-kit`, `rune-motion` (framework UI natif GPU)
- **External CLI Tools**: `yt-dlp`, `ffmpeg`
- **Testing**: `cargo test --all`
- **Target Platform**: Desktop Linux (X11 / Wayland), macOS, Windows
- **Project Type**: Cargo Workspace Monorepo
- **Performance Goals**: 
  - Démarrage de l'interface < 50 ms
  - Rendu GPU fluide à 60/120 FPS
  - Inspection des métadonnées < 2 s
  - Snipper 30 s sur vidéo longue < 10 s

## Constitution Check

- [x] **I. Architecture Découplée :** `y2e-core` est une bibliothèque indépendante sans dépendance graphique.
- [x] **II. Performance GPU & Fluidité :** Rune exploite le GPU ; tous les transferts tournent hors du thread UI.
- [x] **III. Respect des Outils Amonts :** Décodage YouTube via `yt-dlp`, coupe via `--download-sections`.
- [x] **IV. Extensibilité Multi-Frontend :** Événements `CoreEvent` consommables par Rune ou OpenTUI.
- [x] **V. Code Propre & Discrétion Absolue :** Zéro watermark/mention IA dans les fichiers du repo.

## Project Structure

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

### Source Code Layout

```text
Cargo.toml                      # Workspace root
crates/
├── y2e-core/                   # Shared Core Library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── error.rs            # Typed error handling
│       ├── metadata.rs         # VideoMetadata & JSON deserialization
│       ├── progress.rs         # Parsing yt-dlp progress streams
│       ├── presets.rs          # Preset execution commands (Discord, MP3, GIF)
│       └── engine.rs           # Tokio process runner
└── y2e-gui/                    # Native GPU UI (Rune)
    ├── Cargo.toml
    └── src/
        ├── main.rs
        ├── state.rs            # Application state machine
        ├── app.rs              # Rune view builder & spring layout
        └── theme.rs            # Design tokens & color palettes
```
