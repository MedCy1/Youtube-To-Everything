# Data Model: YouTube to Everything engine

## Entities and value objects

### 1. `VideoMetadata`

What `yt-dlp --dump-json` returns for an inspected video. Only the identifier and the title are always present.

```rust
pub struct VideoMetadata {
    pub id: String,
    pub title: String,
    pub uploader: Option<String>,
    pub duration: Option<f64>,       // seconds
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

A cut interval. `TimeRange::new` rejects a start greater than or equal to the end.

```rust
pub struct TimeRange {
    pub start_seconds: u64,
    pub end_seconds: u64,
}
```

### 3. `DownloadPreset`

The transformation chosen for the export.

```rust
pub enum DownloadPreset {
    MaxQuality,
    Discord25Mb,
    AudioMp3,
    GifMeme,
}
```

`estimated_size_mb(seconds)` gives an approximate size for the resulting file, or `None` when it depends too much on the source (maximum quality).

### 4. `ProgressUpdate` and `CoreEvent`

Events sent from the asynchronous engine to the interface.

```rust
pub struct ProgressUpdate {
    pub percent: f32,        // 0.0 to 100.0
    pub speed: String,       // for example "12.5MiB/s"
    pub eta: String,         // for example "00:15"
    pub raw: String,         // the original yt-dlp line
}

pub enum CoreEvent {
    DownloadStarted,
    Progress(ProgressUpdate),
    StatusChanged(String),   // for example "Compressing for Discord (< 25 MB)…"
    Finished { output_path: std::path::PathBuf },
    Error(String),
}
```

### 5. `tools`

The module that finds `yt-dlp` and `ffmpeg` (the `PATH` first, then the application's own copy in `~/.local/share/y2e/bin`), installs them after checking their SHA-256 sum, and reports available updates.
