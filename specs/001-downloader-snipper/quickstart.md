# Quickstart: YouTube to Everything (y2e)

## Prérequis système
- Linux / macOS / Windows
- Rust 1.80+ (`cargo`)
- `yt-dlp` et `ffmpeg` dans le `PATH`, ou installés par l'application au premier lancement (Linux uniquement)
- `curl`

## Commandes principales

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
