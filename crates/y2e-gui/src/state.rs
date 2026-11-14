use crate::artwork::Artwork;
use crate::settings::Settings;
use rune::{Key, KeyEvent, Live, LiveText, Pixels, TextEdit};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::OnceLock;
use y2e_core::{
    format_time_seconds, CoreEvent, DownloadPreset, InstallProblem, TimeRange, VideoMetadata,
};

/// Hue of the brand coral, used until artwork supplies its own.
pub const BRAND_HUE: f32 = 35.0;

pub static ACTION_TX: OnceLock<Sender<GuiAction>> = OnceLock::new();

#[derive(Debug, Clone)]
pub enum GuiAction {
    InspectMetadata(String),
    InstallTools(Vec<&'static str>),
    CheckUpdates,
    CheckAppUpdate,
    CancelDownload,
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
    /// The pointer is over the export button.
    pub export_hover: bool,
    /// The field was filled from a link found in the clipboard.
    pub clipboard_link: bool,
    /// External tools not found at startup (`yt-dlp`, `ffmpeg`).
    pub missing_tools: Vec<&'static str>,
    /// What the installer is doing, while it runs.
    /// Tools are being installed.
    pub installing: bool,
    /// What the installation is doing now, once it has started.
    pub install_stage: Option<y2e_core::tools::Stage>,
    /// How far the installation is, from 0 to 1, when that is known.
    pub install_fraction: Option<f32>,
    pub install_error: Option<InstallProblem>,
    /// Newer versions of the tools, once the startup check has answered.
    pub updates: Vec<y2e_core::tools::Update>,
    /// The user chose to carry on without updating, for this session.
    pub updates_dismissed: bool,
    /// A newer release of the app itself, once GitHub has said so.
    pub app_update: Option<y2e_core::app_update::AppUpdate>,
    pub app_update_dismissed: bool,
    pub settings: Settings,
    pub settings_open: bool,
    pub chapters_open: bool,
    /// Why the last attempt to choose a folder did not work.
    pub settings_error: Option<String>,
    /// The window is in front.
    pub focused: bool,
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
    pub downloaded_file: Option<PathBuf>,
    pub inspect_error: Option<String>,
    pub download_error: Option<String>,
}

impl State {
    /// The state at startup, with what the user saved last time.
    pub fn load() -> Self {
        let settings = Settings::load();
        settings.lang().select();
        Self {
            preset: settings.default_preset,
            settings,
            ..Self::default()
        }
    }

    pub fn set_default_preset(&mut self, preset: DownloadPreset) {
        self.settings.default_preset = preset;
        self.preset = preset;
        self.settings.save();
    }

    pub fn set_language(&mut self, lang: crate::i18n::Lang) {
        self.settings.language = Some(lang);
        lang.select();
        self.settings.save();
    }

    pub fn set_pause_when_unfocused(&mut self, on: bool) {
        self.settings.pause_when_unfocused = on;
        self.settings.save();
    }

    pub fn set_export_dir(&mut self, dir: Option<PathBuf>) {
        self.settings.output_dir = dir;
        self.settings_error = None;
        self.settings.save();
    }

    /// What the folder chooser answered: a folder, a cancellation, or a failure
    /// (a Linux desktop with no file chooser portal, for one).
    pub fn folder_chosen(&mut self, result: rune::FileDialogResult) {
        match result {
            Ok(Some(dir)) => self.set_export_dir(Some(dir)),
            Ok(None) => {}
            Err(error) => {
                self.settings_error =
                    Some((crate::i18n::t().folder_dialog_failed)(&error.to_string()))
            }
        }
    }
}

impl Default for State {
    fn default() -> Self {
        let mut url_input = TextEdit::new("");
        let missing_tools = y2e_core::tools::missing();
        url_input.focused = missing_tools.is_empty();
        Self {
            scrub: Scrub::default(),
            export_hover: false,
            clipboard_link: false,
            missing_tools,
            installing: false,
            install_stage: None,
            install_fraction: None,
            install_error: None,
            updates: Vec::new(),
            updates_dismissed: false,
            app_update: None,
            app_update_dismissed: false,
            settings: Settings::default(),
            settings_open: false,
            chapters_open: false,
            settings_error: None,
            focused: true,
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
            downloaded_file: None,
            inspect_error: None,
            download_error: None,
        }
    }
}

impl State {
    /// Nothing can be read or exported until the external tools are there.
    pub fn tools_ready(&self) -> bool {
        self.missing_tools.is_empty()
    }

    pub fn trigger_inspect(&mut self) {
        if !self.tools_ready() {
            return;
        }
        let url = self.url_input.text.trim().to_string();
        if url.is_empty() {
            self.inspect_error = Some("Paste or type a YouTube link.".to_string());
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
            self.download_error = Some("Missing or invalid URL.".to_string());
            return;
        }

        let time_range = if self.enable_snipper {
            match TimeRange::new(self.start_time_secs, self.end_time_secs) {
                Ok(range) => Some(range),
                Err(e) => {
                    self.download_error = Some(format!("Invalid time range: {}", e));
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

        let output_dir = self.settings.export_dir();

        if let Some(tx) = ACTION_TX.get() {
            let _ = tx.send(GuiAction::StartDownload {
                url,
                time_range,
                preset: self.preset,
                output_dir,
            });
        }
    }

    /// Stops the export in progress. What was already written is left to
    /// yt-dlp's own partial file, which a new export picks up again.
    pub fn cancel_download(&mut self) {
        if !self.is_downloading {
            return;
        }
        if let Some(tx) = ACTION_TX.get() {
            let _ = tx.send(GuiAction::CancelDownload);
        }
        self.is_downloading = false;
        self.progress_percent = 0.0;
    }

    /// Fills the empty field with a YouTube link found in the clipboard, when
    /// the landing view is waiting for one.
    pub fn offer_clipboard_link(&mut self, clipboard_text: &str) {
        let idle = !self.showing_source() && !self.is_loading_metadata;
        if !idle || !self.tools_ready() || !self.url_input.text.is_empty() {
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

    /// Opens the page of the newer release in the browser.
    pub fn open_app_update(&mut self) {
        if let Some(update) = &self.app_update {
            open_with_system(update.url.as_ref());
        }
        self.app_update_dismissed = true;
    }

    /// Shows the last export in the system file manager: selected in its
    /// folder on macOS and Windows, the folder itself on Linux.
    pub fn reveal_download(&self) {
        let Some(file) = self.downloaded_file.as_ref() else {
            return;
        };
        if cfg!(target_os = "macos") {
            let _ = std::process::Command::new("open")
                .arg("-R")
                .arg(file)
                .spawn();
        } else if cfg!(target_os = "windows") {
            let select = format!("/select,{}", file.display());
            let _ = std::process::Command::new("explorer").arg(select).spawn();
        } else if let Some(folder) = file.parent() {
            open_with_system(folder.as_os_str());
        }
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
        if self.is_downloading && key.key == Key::Escape {
            self.cancel_download();
            return true;
        }
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

    /// The chapters worth listing: at least two, each with some length left.
    pub fn chapter_ranges(&self) -> Vec<(String, u64, u64)> {
        let Some(meta) = &self.metadata else {
            return Vec::new();
        };
        let total = self.duration();
        let ranges: Vec<_> = meta
            .chapters()
            .iter()
            .filter_map(|c| c.range(total).map(|(a, b)| (c.title.clone(), a, b)))
            .collect();
        if ranges.len() >= 2 {
            ranges
        } else {
            Vec::new()
        }
    }

    /// Keeps only that chapter. One that covers the whole video leaves the cut off.
    pub fn select_chapter(&mut self, from: u64, to: u64) {
        let total = self.duration();
        self.start_time_secs = from;
        self.end_time_secs = to;
        self.enable_snipper = !(from == 0 && to >= total);
        self.chapters_open = false;
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

    pub fn install_tools(&mut self) {
        self.install(self.missing_tools.clone());
    }

    pub fn update_tools(&mut self) {
        let ours = self.updates.iter().filter(|u| u.managed);
        self.install(ours.map(|u| u.tool).collect());
    }

    fn install(&mut self, tools: Vec<&'static str>) {
        if self.installing || tools.is_empty() {
            return;
        }
        if let Some(tx) = ACTION_TX.get() {
            self.installing = true;
            self.install_stage = None;
            self.install_fraction = None;
            self.install_error = None;
            let _ = tx.send(GuiAction::InstallTools(tools));
        }
    }

    pub fn finish_install(&mut self, result: Result<(), y2e_core::Y2eError>) {
        self.installing = false;
        self.install_stage = None;
        match result {
            Ok(()) => {
                self.missing_tools = y2e_core::tools::missing();
                self.updates.clear();
            }
            Err(y2e_core::Y2eError::Install(problem)) => self.install_error = Some(problem),
            Err(other) => self.install_error = Some(InstallProblem::Other(other.to_string())),
        }
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
            CoreEvent::DownloadStarted => {
                self.is_downloading = true;
                self.download_error = None;
                self.progress_percent = 0.0;
                self.speed_str = "-".to_string();
                self.eta_str = "-".to_string();
            }
            CoreEvent::Progress(update) => {
                self.progress_percent = update.percent.clamp(0.0, 100.0);
                self.speed_str = update.speed;
                self.eta_str = update.eta;
            }
            // The phase of an export is not shown: the progress bar says enough.
            CoreEvent::StatusChanged(_) => {}
            CoreEvent::Finished { output_path } => {
                self.is_downloading = false;
                self.progress_percent = 100.0;
                self.downloaded_file = Some(output_path);
                self.completed += 1;
            }
            CoreEvent::Error(err) => {
                self.is_downloading = false;
                self.download_error = Some(err);
            }
        }
    }
}

/// Opens a folder or a web address with whatever the system uses for it.
fn open_with_system(target: &std::ffi::OsStr) {
    let opener = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "explorer"
    } else {
        "xdg-open"
    };
    let _ = std::process::Command::new(opener).arg(target).spawn();
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
    use super::{nudged, State};
    use y2e_core::{CoreEvent, ProgressUpdate, VideoChapter, VideoMetadata};

    fn with_chapters(chapters: &[(&str, f64, f64)]) -> State {
        State {
            metadata: Some(VideoMetadata {
                id: "abc".into(),
                title: "t".into(),
                uploader: None,
                duration: Some(100.0),
                thumbnail: None,
                chapters: Some(
                    chapters
                        .iter()
                        .map(|(title, a, b)| VideoChapter {
                            title: title.to_string(),
                            start_seconds: *a,
                            end_seconds: *b,
                        })
                        .collect(),
                ),
                webpage_url: None,
            }),
            ..State::default()
        }
    }

    #[test]
    fn keeps_the_cut_inside_the_video_and_non_empty() {
        assert_eq!(nudged((0, 100), 100, false, -10), (0, 90));
        assert_eq!(nudged((0, 100), 100, false, 10), (0, 100));
        assert_eq!(nudged((10, 20), 100, true, 50), (19, 20));
        assert_eq!(nudged((10, 20), 100, false, -50), (10, 11));
        assert_eq!(nudged((3, 20), 100, true, -10), (0, 20));
    }

    #[test]
    fn a_single_chapter_is_not_worth_a_list() {
        assert!(with_chapters(&[("all", 0.0, 100.0)])
            .chapter_ranges()
            .is_empty());
        assert!(with_chapters(&[]).chapter_ranges().is_empty());
    }

    #[test]
    fn picking_a_chapter_cuts_to_it_and_closes_the_list() {
        let mut state = with_chapters(&[("a", 0.0, 40.0), ("b", 40.0, 100.0)]);
        assert_eq!(state.chapter_ranges().len(), 2);
        state.chapters_open = true;
        state.select_chapter(40, 100);
        assert!(state.enable_snipper);
        assert_eq!((state.start_time_secs, state.end_time_secs), (40, 100));
        assert!(!state.chapters_open);
    }

    #[test]
    fn a_chapter_that_is_the_whole_video_leaves_the_cut_off() {
        let mut state = with_chapters(&[("a", 0.0, 100.0), ("b", 100.0, 120.0)]);
        state.select_chapter(0, 100);
        assert!(!state.enable_snipper);
    }

    #[test]
    fn a_new_export_does_not_show_the_last_ones_figures() {
        let mut s = State::default();
        s.apply_core_event(CoreEvent::Progress(ProgressUpdate {
            percent: 80.0,
            speed: "1.5x".to_string(),
            eta: "00:05".to_string(),
            raw: String::new(),
        }));
        s.apply_core_event(CoreEvent::DownloadStarted);
        assert_eq!(s.progress_percent, 0.0);
        assert_eq!((s.speed_str.as_str(), s.eta_str.as_str()), ("-", "-"));
    }
}
