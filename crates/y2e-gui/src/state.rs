use crate::artwork::Artwork;
use crossbeam_channel::Sender;
use rune::{Key, KeyEvent, Live, LiveText, Pixels, TextEdit};
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
    /// The field was filled from a link found in the clipboard.
    pub clipboard_link: bool,
    /// Number of finished exports, so each completion replays its celebration.
    pub completed: u32,
    pub url_input: TextEdit,
    pub metadata: Option<VideoMetadata>,
    pub thumbnail: Option<Pixels>,
    pub accent_hue: f32,
    /// Lightness of the artwork's lower part: the lighter, the stronger the scrim.
    pub art_lightness: f32,
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
            clipboard_link: false,
            completed: 0,
            url_input,
            metadata: None,
            thumbnail: None,
            accent_hue: BRAND_HUE,
            art_lightness: 0.3,
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
        self.clipboard_link = false;
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

    /// Fills the empty field with a YouTube link found in the clipboard, when
    /// the landing view is waiting for one.
    pub fn offer_clipboard_link(&mut self, clipboard_text: &str) {
        let idle = !self.showing_source() && !self.is_loading_metadata;
        if !idle || !self.url_input.text.is_empty() {
            return;
        }
        if let Some(link) = crate::link::youtube_link(clipboard_text) {
            self.url_input = TextEdit::new(&link);
            self.url_input.focused = true;
            self.inspect_error = None;
            self.clipboard_link = true;
        }
    }

    /// A source is on screen only once it is fully loaded, artwork included.
    pub fn showing_source(&self) -> bool {
        self.metadata.is_some() && !self.is_loading_metadata
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

    /// Length of what will be exported, in seconds (0 when unknown).
    pub fn selected_seconds(&self) -> u64 {
        if self.enable_snipper {
            self.end_time_secs.saturating_sub(self.start_time_secs)
        } else {
            self.duration()
        }
    }

    /// Narrows the cut to a short clip starting at the current start.
    pub fn keep_short_clip(&mut self, seconds: u64) {
        let total = self.duration();
        if total == 0 {
            return;
        }
        let start = if self.enable_snipper {
            self.start_time_secs
        } else {
            0
        };
        self.start_time_secs = start.min(total.saturating_sub(1));
        self.end_time_secs = (self.start_time_secs + seconds).min(total);
        self.enable_snipper = true;
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

    /// Moves one end of the cut by `delta` seconds (keyboard control).
    pub fn nudge_timeline(&mut self, start: bool, delta: i64) {
        let total = self.duration();
        if total == 0 {
            return;
        }
        let (from, to) = if self.enable_snipper {
            (self.start_time_secs, self.end_time_secs)
        } else {
            (0, total)
        };
        let (from, to) = nudged((from, to), total, start, delta);
        self.start_time_secs = from;
        self.end_time_secs = to;
        self.enable_snipper = !(from == 0 && to >= total);
    }

    /// Keyboard control of the cut: arrows move its end (Shift: its start) by
    /// one second, Up and Down by ten, Home selects the whole video.
    pub fn timeline_key(&mut self, key: &KeyEvent) -> bool {
        if !self.showing_source() || key.mods.cmd || key.mods.alt || key.mods.ctrl {
            return false;
        }
        let step = match key.key {
            Key::Left => -1,
            Key::Right => 1,
            Key::Down => -10,
            Key::Up => 10,
            Key::Home => {
                self.reset_range();
                return true;
            }
            _ => return false,
        };
        self.nudge_timeline(key.mods.shift, step);
        true
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

    pub fn on_inspect_success(&mut self, meta: VideoMetadata, artwork: Option<Artwork>) {
        let duration = meta.duration_seconds();
        self.start_time_secs = 0;
        self.end_time_secs = if duration > 0 { duration } else { 30 };
        self.enable_snipper = false;
        self.downloaded_file = None;
        self.download_error = None;
        self.progress_percent = 0.0;
        self.metadata = Some(meta);
        self.accent_hue = artwork
            .as_ref()
            .and_then(|art| art.hue)
            .unwrap_or(BRAND_HUE);
        self.art_lightness = artwork.as_ref().map_or(0.3, |art| art.lightness);
        self.thumbnail = artwork.map(|art| art.pixels);
        self.is_loading_metadata = false;
        self.inspect_error = None;
    }

    pub fn on_inspect_error(&mut self, err: String) {
        // A failed lookup leaves no source on screen: the previous one would be
        // shown again without its artwork.
        self.metadata = None;
        self.thumbnail = None;
        self.accent_hue = BRAND_HUE;
        self.art_lightness = 0.3;
        self.reset_range();
        self.is_loading_metadata = false;
        self.inspect_error = Some(err);
    }

    pub fn apply_core_event(&mut self, event: CoreEvent) {
        match event {
            CoreEvent::MetadataLoaded(meta) => {
                self.on_inspect_success(meta, None);
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

/// One end of `range` moved by `delta` seconds, kept inside `0..=total` and
/// at least one second long.
fn nudged(range: (u64, u64), total: u64, start: bool, delta: i64) -> (u64, u64) {
    let (from, to) = range;
    let shift = |value: u64| (value as i64 + delta).clamp(0, total as i64) as u64;
    if start {
        (shift(from).min(to.saturating_sub(1)), to)
    } else {
        (from, shift(to).max(from + 1).min(total))
    }
}

#[cfg(test)]
mod tests {
    use super::nudged;

    #[test]
    fn keeps_the_cut_inside_the_video_and_non_empty() {
        assert_eq!(nudged((0, 100), 100, false, -10), (0, 90));
        assert_eq!(nudged((0, 100), 100, false, 10), (0, 100));
        assert_eq!(nudged((10, 20), 100, true, 50), (19, 20));
        assert_eq!(nudged((10, 20), 100, false, -50), (10, 11));
        assert_eq!(nudged((3, 20), 100, true, -10), (0, 20));
    }
}
