# Quickstart: YouTube to Everything (y2e)

## Prérequis Système
- Linux / macOS / Windows
- Rust 1.80+ (`cargo`)
- `yt-dlp` installé et accessible dans le `PATH`
- `ffmpeg` installé et accessible dans le `PATH`

## Commandes Principales

### 1. Compilation de la bibliothèque et des exécutables
```bash
cargo build --release
```

### 2. Lancement de l'interface graphique native Rune
```bash
cargo run --bin y2e-gui
```

### 3. Exécution des tests unitaires et d'intégration
```bash
cargo test --all
```
