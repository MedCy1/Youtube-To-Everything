# Data Model: YouTube to Everything Engine

## Entities & Value Objects

### 1. `VideoMetadata`
Représente l'état inspecté d'une ressource vidéo.
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMetadata {
    pub id: String,
    pub url: String,
    pub title: String,
    pub uploader: String,
    pub duration_seconds: u64,
    pub thumbnail_url: String,
    pub chapters: Vec<VideoChapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoChapter {
    pub title: String,
    pub start_time: f64,
    pub end_time: f64,
}
```

### 2. `TimeRange` (Snipper)
Définit un intervalle temporel de coupe optionnel.
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeRange {
    pub start_seconds: u64,
    pub end_seconds: u64,
}
```

### 3. `DownloadPreset`
Sélecteur du mode de transformation.
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadPreset {
    MaxQuality,
    Discord25Mb,
    AudioMp3,
    GifMeme,
}
```

### 4. `ProgressUpdate` & `CoreEvent`
Événements transmis du moteur asynchrone vers l'UI.
```rust
#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub percent: f32,          // 0.0 à 100.0
    pub speed_bytes_per_sec: u64,
    pub eta_seconds: u64,
    pub status_text: String,
}

#[derive(Debug, Clone)]
pub enum CoreEvent {
    MetadataLoaded(VideoMetadata),
    DownloadStarted,
    Progress(ProgressUpdate),
    StepChanged(String),       // ex: "Transcodage Discord..."
    Completed { file_path: std::path::PathBuf },
    Failed { reason: String },
}
```
