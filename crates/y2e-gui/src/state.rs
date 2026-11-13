use crossbeam_channel::Sender;
use rune::{Pixels, TextEdit};
use std::path::PathBuf;
use std::sync::OnceLock;
use y2e_core::{CoreEvent, DownloadPreset, TimeRange, VideoMetadata};

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

pub struct State {
    pub url_input: TextEdit,
    pub metadata: Option<VideoMetadata>,
    pub thumbnail: Option<Pixels>,
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
            url_input,
            metadata: None,
            thumbnail: None,
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

    pub fn on_inspect_success(&mut self, meta: VideoMetadata) {
        let duration = meta.duration_seconds();
        self.end_time_secs = if duration > 0 { duration.min(60) } else { 30 };
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
