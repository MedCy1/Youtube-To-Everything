//! What the user can change, kept in a small JSON file in the config folder.

use crate::i18n::Lang;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use y2e_core::DownloadPreset;

/// How much the interface moves, and how much it asks of the graphics card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Motion {
    #[default]
    Full,
    /// No drifting background and only short fades, as the system's own
    /// "reduce motion" asks.
    Reduced,
    /// Reduced, and without the blur and glow behind the glass: for older computers.
    Performance,
}

impl Motion {
    /// Nothing moves on its own, and travel is replaced by short fades.
    pub fn is_calm(self) -> bool {
        self != Motion::Full
    }

    /// The effects that cost the most are left out.
    pub fn is_light(self) -> bool {
        self == Motion::Performance
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Settings {
    /// Where exports go. The system's Downloads folder when unset.
    pub output_dir: Option<PathBuf>,
    /// The format selected when the app starts.
    pub default_preset: DownloadPreset,
    /// Hold the animated background still while the window is not in front.
    pub pause_when_unfocused: bool,
    /// The language of the interface. The system's, when unset.
    pub language: Option<Lang>,
    /// How much the interface moves.
    pub motion: Motion,
}

/// `~/.config/y2e/settings.json` on Linux, the matching folder elsewhere.
fn file() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("y2e").join("settings.json"))
}

impl Settings {
    /// What was saved, or the defaults when nothing was or the file is unreadable.
    pub fn load() -> Self {
        file().map_or_else(Self::default, |path| Self::read(&path))
    }

    pub fn save(&self) {
        if let Some(path) = file() {
            let _ = self.write(&path);
        }
    }

    fn read(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    /// Written to a temporary file first, so a crash never leaves half a file.
    fn write(&self, path: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(path.parent().unwrap_or(path))?;
        let partial = path.with_extension("json.tmp");
        std::fs::write(&partial, serde_json::to_string_pretty(self)?)?;
        std::fs::rename(partial, path)
    }

    pub fn lang(&self) -> Lang {
        self.language.unwrap_or_else(Lang::of_system)
    }

    /// The chosen folder, else Downloads, else the home folder, and the
    /// current folder only when the system names none of them.
    pub fn export_dir(&self) -> PathBuf {
        self.pick_dir(dirs::download_dir(), dirs::home_dir())
    }

    fn pick_dir(&self, downloads: Option<PathBuf>, home: Option<PathBuf>) -> PathBuf {
        self.output_dir
            .clone()
            .or(downloads)
            .or(home)
            .unwrap_or_else(|| PathBuf::from("./output"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("y2e-settings-{name}-{}", std::process::id()))
    }

    #[test]
    fn settings_survive_a_save_and_a_load() {
        let path = scratch("roundtrip").join("settings.json");
        let saved = Settings {
            output_dir: Some(PathBuf::from("/exports")),
            default_preset: DownloadPreset::GifMeme,
            pause_when_unfocused: true,
            language: Some(Lang::French),
            motion: Motion::Performance,
        };
        saved.write(&path).unwrap();
        assert_eq!(Settings::read(&path), saved);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn the_export_folder_falls_back_step_by_step() {
        let mut settings = Settings::default();
        let (downloads, home) = (Some(PathBuf::from("/d")), Some(PathBuf::from("/h")));
        assert_eq!(
            settings.pick_dir(downloads.clone(), home.clone()),
            PathBuf::from("/d")
        );
        assert_eq!(settings.pick_dir(None, home), PathBuf::from("/h"));
        assert_eq!(settings.pick_dir(None, None), PathBuf::from("./output"));
        settings.output_dir = Some(PathBuf::from("/chosen"));
        assert_eq!(settings.pick_dir(downloads, None), PathBuf::from("/chosen"));
    }

    #[test]
    fn a_missing_or_broken_file_gives_the_defaults() {
        let dir = scratch("broken");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        assert_eq!(Settings::read(&path), Settings::default());
        std::fs::write(&path, "{ not json").unwrap();
        assert_eq!(Settings::read(&path), Settings::default());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_setting_added_later_takes_its_default_in_an_older_file() {
        let dir = scratch("older");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("settings.json");
        std::fs::write(&path, r#"{ "pause_when_unfocused": true }"#).unwrap();
        let read = Settings::read(&path);
        assert!(read.pause_when_unfocused);
        assert_eq!(read.default_preset, DownloadPreset::MaxQuality);
        assert_eq!(read.motion, Motion::Full);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
