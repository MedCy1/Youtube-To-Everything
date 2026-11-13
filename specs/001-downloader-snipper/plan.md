# Implementation Plan: téléchargeur et découpe précise

Branch: `001-downloader-snipper` | Date: 2026-11-12 | Spec: [specs/001-downloader-snipper/spec.md](spec.md)

Input: Spécification fonctionnelle issue de `/specs/001-downloader-snipper/spec.md`

## Summary

Un moteur asynchrone en Rust (`y2e-core`) pilote `yt-dlp` et `ffmpeg`, et une interface graphique native GPU est construite avec le framework Rune (`y2e-gui`). L'application lit les métadonnées, ne télécharge que le segment demandé (Snipper) et propose quatre presets : qualité maximale, Discord < 25 Mo, audio MP3 et GIF.

## Technical Context

- Language/Version: Rust 2021/2024 (rustc 1.85+)
- Primary Dependencies: 
  - `tokio` (runtime asynchrone pour sous-processus)
  - `serde`, `serde_json` (désérialisation des métadonnées yt-dlp)
  - `thiserror` (gestion d'erreurs typées)
  - `tokio::sync` et `std::sync::mpsc` (canaux entre le moteur et l'interface)
  - `rune`, `rune-kit`, `rune-fx`, `rune-theme-opencode` (framework UI natif GPU)
- External CLI Tools: `yt-dlp`, `ffmpeg`
- Testing: `cargo test --all`
- Target Platform: Desktop Linux (X11 / Wayland), macOS, Windows
- Project Type: Cargo Workspace Monorepo
- Performance Goals: 
  - Démarrage de l'interface < 200 ms
  - Rendu GPU à 60 FPS minimum
  - Inspection des métadonnées < 2 s
  - Snipper 30 s sur vidéo longue < 10 s

## Constitution Check

- [x] I. Architecture Découplée : `y2e-core` est une bibliothèque indépendante sans dépendance graphique.
- [x] II. Performance GPU & Fluidité : Rune exploite le GPU ; tous les transferts tournent hors du thread UI.
- [x] III. Respect des Outils Amonts : Décodage YouTube via `yt-dlp`, coupe via `--download-sections`.
- [x] IV. Extensibilité Multi-Frontend : Événements `CoreEvent` consommables par Rune ou OpenTUI.
- [x] V. Code Propre & Discrétion Absolue : Zéro watermark/mention IA dans les fichiers du repo.

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

### Source code layout

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
        ├── theme.rs            # Design tokens & color palettes
        ├── tint.rs, artwork.rs # Hue extraction from the thumbnail
        └── link.rs, report.rs  # Clipboard link detection, readable errors
```
