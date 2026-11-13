use crossbeam_channel::Sender;
use rune::{Live, LiveText, Pixels, TextEdit};
use std::path::PathBuf;
use std::sync::OnceLock;
use y2e_core::{format_time_seconds, CoreEvent, DownloadPreset, TimeRange, VideoMetadata};

/// Hue of the brand coral, used until artwork supplies its own.
pub const BRAND_HUE: f32 = 35.0;

pub static ACTION_TX: OnceLock<Sender<GuiAction>> = OnceLock::new();

#[derive(Debug, Clone)]
pub enum GuiAction {
    InspectMetadata(String),
    StartDownload {
        url: String,
        time_range: Option<TimeRange>,
        preset: DownloadPreset,
        output_dir: PathBuf,
    },
}

/// The pointer's position over the timeline. The line and its label move as
/// live values, so hovering never rebuilds the view.
pub struct Scrub {
    pub line: Live<[f32; 2]>,
    pub tip: Live<[f32; 2]>,
    pub label: LiveText,
    pub inside: bool,
    pub width: f32,
}

impl Default for Scrub {
    fn default() -> Self {
        Self {
            line: Live::new([0.0, 0.0]),
            tip: Live::new([0.0, 0.0]),
            label: LiveText::new("0:00"),
            inside: false,
            width: 0.0,
        }
    }
}

pub struct State {
    pub scrub: Scrub,
    /// Number of finished exports, so each completion replays its celebration.
    pub completed: u32,
    pub url_input: TextEdit,
    pub metadata: Option<VideoMetadata>,
    pub thumbnail: Option<Pixels>,
    pub accent_hue: f32,
    pub is_loading_metadata: bool,
    pub is_downloading: bool,
    pub enable_snipper: bool,
    pub start_time_secs: u64,
    pub end_time_secs: u64,
    pub preset: DownloadPreset,
    pub progress_percent: f32,
    pub speed_str: String,
    pub eta_str: String,
    pub status_text: String,
    pub downloaded_file: Option<PathBuf>,
    pub inspect_error: Option<String>,
    pub download_error: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        let mut url_input = TextEdit::new("");
        url_input.focused = true;
        Self {
            scrub: Scrub::default(),
            completed: 0,
            url_input,
            metadata: None,
            thumbnail: None,
            accent_hue: BRAND_HUE,
            is_loading_metadata: false,
            is_downloading: false,
            enable_snipper: false,
            start_time_secs: 0,
            end_time_secs: 30,
            preset: DownloadPreset::MaxQuality,
            progress_percent: 0.0,
            speed_str: "-".to_string(),
            eta_str: "-".to_string(),
            status_text: "Prêt. Collez une URL YouTube pour démarrer.".to_string(),
            downloaded_file: None,
            inspect_error: None,
            download_error: None,
        }
    }
}

impl State {
    pub fn trigger_inspect(&mut self) {
        let url = self.url_input.text.trim().to_string();
        if url.is_empty() {
            self.inspect_error = Some("Veuillez saisir ou coller une URL YouTube.".to_string());
            return;
        }

        self.inspect_error = None;
        self.thumbnail = None;
        self.accent_hue = BRAND_HUE;
        self.is_loading_metadata = true;

        if let Some(tx) = ACTION_TX.get() {
            let _ = tx.send(GuiAction::InspectMetadata(url));
        }
    }

    pub fn trigger_download(&mut self) {
        let url = self.url_input.text.trim().to_string();
        if url.is_empty() {
            self.download_error = Some("URL absente ou invalide.".to_string());
            return;
        }

        let time_range = if self.enable_snipper {
            match TimeRange::new(self.start_time_secs, self.end_time_secs) {
                Ok(range) => Some(range),
                Err(e) => {
                    self.download_error = Some(format!("Erreur d'intervalle: {}", e));
                    return;
                }
            }
        } else {
            None
        };

        self.is_downloading = true;
        self.progress_percent = 0.0;
        self.downloaded_file = None;
        self.download_error = None;
        self.status_text = "Démarrage du téléchargement...".to_string();

        let output_dir = dirs::download_dir().unwrap_or_else(|| PathBuf::from("./output"));

        if let Some(tx) = ACTION_TX.get() {
            let _ = tx.send(GuiAction::StartDownload {
                url,
                time_range,
                preset: self.preset,
                output_dir,
            });
        }
    }

    pub fn duration(&self) -> u64 {
        self.metadata
            .as_ref()
            .map(|meta| meta.duration_seconds())
            .unwrap_or(0)
    }

    pub fn scrub_to(&mut self, x: f32) {
        let total = self.duration();
        if total == 0 || self.scrub.width <= 0.0 {
            return;
        }
        let x = x.clamp(0.0, self.scrub.width);
        let seconds = (x / self.scrub.width * total as f32).round() as u64;
        self.scrub.line.set([x, 0.0]);
        self.scrub.tip.set([x - 38.0, 0.0]);
        self.scrub.label.set(format_time_seconds(seconds));
    }

    /// Opens the folder holding the last export in the system file manager.
    pub fn reveal_download(&self) {
        let Some(folder) = self.downloaded_file.as_ref().and_then(|path| path.parent()) else {
            return;
        };
        let opener = if cfg!(target_os = "macos") {
            "open"
        } else if cfg!(target_os = "windows") {
            "explorer"
        } else {
            "xdg-open"
        };
        let _ = std::process::Command::new(opener).arg(folder).spawn();
    }

    /// Whole video selected again: the cut is switched off.
    pub fn reset_range(&mut self) {
        self.start_time_secs = 0;
        self.end_time_secs = self.duration().max(1);
        self.enable_snipper = false;
    }

    /// Timeline gesture. `data` remembers which handle the press grabbed
    /// (0 = start, 1 = end) for the rest of the drag.
    pub fn drag_timeline(
        &mut self,
        phase: rune::Phase,
        x: f32,
        start_x: f32,
        w: f32,
        data: &mut f32,
    ) {
        let total = self.duration();
        if total == 0 || w <= 0.0 {
            return;
        }
        let at = |px: f32| ((px / w).clamp(0.0, 1.0) * total as f32).round() as u64;
        if matches!(phase, rune::Phase::Down) {
            let pressed = at(start_x);
            let to_start = pressed.abs_diff(self.start_time_secs);
            let to_end = pressed.abs_diff(self.end_time_secs);
            *data = if to_start <= to_end { 0.0 } else { 1.0 };
        }
        let t = at(x);
        if *data < 0.5 {
            self.start_time_secs = t.min(self.end_time_secs.saturating_sub(1));
        } else {
            self.end_time_secs = t.max(self.start_time_secs + 1).min(total);
        }
        self.enable_snipper = true;
        if matches!(phase, rune::Phase::Up)
            && self.start_time_secs == 0
            && self.end_time_secs >= total
        {
            self.enable_snipper = false;
        }
    }

    pub fn on_inspect_success(&mut self, meta: VideoMetadata) {
        let duration = meta.duration_seconds();
        self.start_time_secs = 0;
        self.end_time_secs = if duration > 0 { duration } else { 30 };
        self.enable_snipper = false;
        self.downloaded_file = None;
        self.download_error = None;
        self.progress_percent = 0.0;
        self.metadata = Some(meta);
        self.is_loading_metadata = false;
        self.inspect_error = None;
    }

    pub fn on_inspect_error(&mut self, err: String) {
        self.is_loading_metadata = false;
        self.inspect_error = Some(err);
    }

    pub fn apply_core_event(&mut self, event: CoreEvent) {
        match event {
            CoreEvent::MetadataLoaded(meta) => {
                self.on_inspect_success(meta);
            }
            CoreEvent::DownloadStarted => {
                self.is_downloading = true;
                self.download_error = None;
                self.status_text = "Téléchargement en cours...".to_string();
            }
            CoreEvent::Progress(update) => {
                self.progress_percent = update.percent.clamp(0.0, 100.0);
                self.speed_str = update.speed;
                self.eta_str = update.eta;
            }
            CoreEvent::StatusChanged(text) => {
                self.status_text = text;
            }
            CoreEvent::Finished { output_path } => {
                self.is_downloading = false;
                self.progress_percent = 100.0;
                self.status_text = "Opération terminée avec succès !".to_string();
                self.downloaded_file = Some(output_path);
                self.completed += 1;
            }
            CoreEvent::Error(err) => {
                self.is_downloading = false;
                self.download_error = Some(err);
                self.status_text = "Une erreur est survenue pendant le traitement.".to_string();
            }
        }
    }
}

mod dirs {
    use std::path::PathBuf;
    pub fn download_dir() -> Option<PathBuf> {
        std::env::var("HOME")
            .ok()
            .map(|h| PathBuf::from(h).join("Downloads"))
    }
}
