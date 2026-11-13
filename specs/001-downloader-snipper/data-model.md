# Data Model: YouTube to Everything Engine

## Entities and value objects

### 1. `VideoMetadata`
Ce que `yt-dlp --dump-json` renvoie d'une vidéo inspectée. Seuls l'identifiant et le titre sont toujours présents.
```rust
pub struct VideoMetadata {
    pub id: String,
    pub title: String,
    pub uploader: Option<String>,
    pub duration: Option<f64>,       // secondes
    pub thumbnail: Option<String>,   // URL
    pub chapters: Option<Vec<VideoChapter>>,
    pub webpage_url: Option<String>,
}

pub struct VideoChapter {
    pub title: String,
    pub start_seconds: f64,
    pub end_seconds: f64,
}
```

### 2. `TimeRange` (Snipper)
Intervalle de coupe. `TimeRange::new` refuse un début supérieur ou égal à la fin.
```rust
pub struct TimeRange {
    pub start_seconds: u64,
    pub end_seconds: u64,
}
```

### 3. `DownloadPreset`
Mode de transformation choisi pour l'export.
```rust
pub enum DownloadPreset {
    MaxQuality,
    Discord25Mb,
    AudioMp3,
    GifMeme,
}
```
`estimated_size_mb(seconds)` donne une taille approximative du fichier produit, ou `None` quand elle dépend trop de la source (qualité maximale).

### 4. `ProgressUpdate` et `CoreEvent`
Événements envoyés du moteur asynchrone vers l'interface.
```rust
pub struct ProgressUpdate {
    pub percent: f32,        // 0.0 à 100.0
    pub speed: String,       // ex. "12.5MiB/s"
    pub eta: String,         // ex. "00:15"
    pub raw: String,         // la ligne d'origine de yt-dlp
}

pub enum CoreEvent {
    DownloadStarted,
    Progress(ProgressUpdate),
    StatusChanged(String),   // ex. "Compression Discord (< 25 Mo)…"
    Finished { output_path: std::path::PathBuf },
    Error(String),
}
```

### 5. `tools`
Module qui trouve `yt-dlp` et `ffmpeg` (le `PATH` d'abord, puis la copie de l'application dans `~/.local/share/y2e/bin`), les installe en vérifiant leur somme SHA-256 et signale les mises à jour disponibles.
