//! The words of the interface, in each language it speaks.
//!
//! `Strings` has one field per text. A language is a `Strings` value, so the
//! compiler refuses a translation that leaves one out. Texts with a blank to
//! fill (a time, a tool's name) are functions.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Lang {
    #[default]
    English,
    French,
}

static CURRENT: AtomicU8 = AtomicU8::new(0);

impl Lang {
    fn index(self) -> u8 {
        match self {
            Lang::English => 0,
            Lang::French => 1,
        }
    }

    /// The language of the system, English when it is not one we speak.
    pub fn of_system() -> Lang {
        Self::from_locale(sys_locale::get_locale().as_deref())
    }

    fn from_locale(locale: Option<&str>) -> Lang {
        match locale {
            Some(l) if l.to_lowercase().starts_with("fr") => Lang::French,
            _ => Lang::English,
        }
    }

    /// Makes this the language every `t()` answers in.
    pub fn select(self) {
        CURRENT.store(self.index(), Ordering::Relaxed);
    }
}

/// The texts in the selected language.
pub fn t() -> &'static Strings {
    if CURRENT.load(Ordering::Relaxed) == 1 {
        &FRENCH
    } else {
        &ENGLISH
    }
}

pub struct Strings {
    // Start screen
    pub hero_first: &'static str,
    pub hero_second: &'static str,
    pub tagline: &'static str,
    pub link_label: &'static str,
    pub link_placeholder: &'static str,
    pub load: &'static str,
    pub loading: &'static str,
    pub clipboard_link: &'static str,
    pub fetching_details: &'static str,
    pub unknown_author: &'static str,
    pub video_loaded: fn(&str) -> String,
    pub video_loaded_plain: &'static str,

    // Tools
    pub install_failed: &'static str,
    pub install_auto: &'static str,
    pub try_again: &'static str,
    pub update: &'static str,
    pub later: &'static str,
    pub got_it: &'static str,
    pub preparing: &'static str,
    pub step_downloading: fn(&str, usize, usize) -> String,
    pub step_verifying: fn(&str) -> String,
    pub step_installing: fn(&str) -> String,
    pub missing_ytdlp: &'static str,
    pub missing_ffmpeg: &'static str,
    pub missing_both: &'static str,
    pub can_install: fn(u32) -> String,
    pub install_brew: fn(&str) -> String,
    pub install_winget: fn(&str) -> String,
    pub install_by_package_manager: &'static str,
    pub update_ytdlp: &'static str,
    pub update_ffmpeg: &'static str,
    pub update_both: &'static str,
    pub stay_up_to_date: &'static str,
    pub update_with_package_manager: fn(&str) -> String,
    pub and: &'static str,
    pub ffmpeg_is_old: &'static str,

    // App update
    pub app_update_headline: &'static str,
    pub app_update_cause: fn(&str, &str) -> String,
    pub see_whats_new: &'static str,

    // Timeline
    pub clip_range: &'static str,
    pub clip_described: fn(&str, &str) -> String,
    pub whole_described: fn(&str) -> String,
    pub clip_keys_help: &'static str,
    pub clip_summary: fn(&str, &str) -> String,
    pub select_whole: &'static str,
    pub all: &'static str,
    pub whole_hint: &'static str,

    // Dock
    pub export: &'static str,
    pub export_progress: &'static str,
    pub cancel: &'static str,
    pub cancel_export: &'static str,
    pub max_quality: &'static str,
    pub audio: &'static str,
    pub loop_gif: &'static str,
    pub gif_large: fn(&str) -> String,
    pub keep_seconds: fn(u64) -> String,
    pub switch_to_mp4: &'static str,
    pub unit_mb: &'static str,
    pub unit_gb: &'static str,
    pub decimal: char,
    pub export_finished: &'static str,
    pub open_folder: &'static str,

    // Settings
    pub settings: &'static str,
    pub export_folder: &'static str,
    pub choose_folder: &'static str,
    pub change: &'static str,
    pub reset: &'static str,
    pub use_downloads: &'static str,
    pub folder_dialog_title: &'static str,
    pub folder_dialog_failed: fn(&str) -> String,
    pub default_format: &'static str,
    pub default_format_hint: &'static str,
    pub format_as_default: fn(&str) -> String,
    pub pause_background: &'static str,
    pub pause_background_hint: &'static str,
    pub pause_background_label: &'static str,
    pub language: &'static str,
    pub language_hint: &'static str,
    pub done: &'static str,

    // Errors
    pub err_generic: &'static str,
    pub err_unavailable: &'static str,
    pub err_truncated: &'static str,
    pub err_unsupported: &'static str,
    pub err_verification: &'static str,
    pub err_network: &'static str,
    pub err_tools_missing: &'static str,
    pub err_disk_full: &'static str,
    pub err_paste_link: &'static str,
    pub err_bad_range: &'static str,
    pub problem_no_data_folder: &'static str,
    pub problem_no_build: fn(&str) -> String,
    pub problem_missing_helper: &'static str,
    pub problem_server_silent: &'static str,
    pub problem_unreadable: fn(&str) -> String,
    pub problem_checksum_missing: fn(&str) -> String,
    pub problem_corrupted: fn(&str) -> String,
}

pub static ENGLISH: Strings = Strings {
    hero_first: "One video.",
    hero_second: "Every format.",
    tagline: "MP4 · MP3 · Discord · GIF. Just the part you want.",
    link_label: "YouTube video link",
    link_placeholder: "Paste a YouTube link",
    load: "Load",
    loading: "Loading…",
    clipboard_link: "Link found on the clipboard. Press Enter to load it.",
    fetching_details: "Fetching video details",
    unknown_author: "Unknown author",
    video_loaded: |length| format!("Video loaded · {length}"),
    video_loaded_plain: "Video loaded",

    install_failed: "Installation failed.",
    install_auto: "Install automatically",
    try_again: "Try again",
    update: "Update",
    later: "Later",
    got_it: "Got it",
    preparing: "Preparing",
    step_downloading: |tool, n, of| format!("Downloading {tool} ({n}/{of})"),
    step_verifying: |tool| format!("Verifying {tool}"),
    step_installing: |tool| format!("Installing {tool}"),
    missing_ytdlp: "yt-dlp was not found.",
    missing_ffmpeg: "ffmpeg was not found.",
    missing_both: "yt-dlp and ffmpeg were not found.",
    can_install: |mb| format!("y2e can install them for you (about {mb} MB to download, once)."),
    install_brew: |list| format!("Install them with Homebrew (brew install {list}), then restart y2e."),
    install_winget: |list| format!("Install them with winget (winget install {list}), then restart y2e."),
    install_by_package_manager: "Install them with your package manager, then restart y2e.",
    update_ytdlp: "A yt-dlp update is available.",
    update_ffmpeg: "An ffmpeg update is available.",
    update_both: "Updates are available.",
    stay_up_to_date: "Staying up to date avoids errors when YouTube changes.",
    update_with_package_manager: |list| format!("Update with your package manager: {list}."),
    and: " and ",
    ffmpeg_is_old: "installed more than 4 months ago",

    app_update_headline: "A new version of y2e is available.",
    app_update_cause: |new, current| format!("Version {new} is out, you have {current}."),
    see_whats_new: "See what's new",

    clip_range: "Clip range",
    clip_described: |from, to| format!("Clip from {from} to {to}."),
    whole_described: |length| format!("Whole video, {length}."),
    clip_keys_help: "The arrow keys move the end of the clip, Shift with an arrow moves the start, Home selects the whole video.",
    clip_summary: |from, to| format!("Clip  {from} → {to}"),
    select_whole: "Select the whole video",
    all: "All",
    whole_hint: "Whole video. Drag across the waveform to cut.",

    export: "Export",
    export_progress: "Export progress",
    cancel: "Cancel",
    cancel_export: "Cancel export",
    max_quality: "Max quality",
    audio: "Audio",
    loop_gif: "Loop",
    gif_large: |length| format!("A {length} GIF will be large. A short clip is easier to share."),
    keep_seconds: |n| format!("Keep {n} s"),
    switch_to_mp4: "Switch to MP4",
    unit_mb: "MB",
    unit_gb: "GB",
    decimal: '.',
    export_finished: "Export finished",
    open_folder: "Open folder",

    settings: "Settings",
    export_folder: "Export folder",
    choose_folder: "Choose the export folder",
    change: "Change…",
    reset: "Reset",
    use_downloads: "Use the Downloads folder",
    folder_dialog_title: "Export folder",
    folder_dialog_failed: |why| format!("Could not open the folder chooser: {why}"),
    default_format: "Default format",
    default_format_hint: "Selected when y2e starts.",
    format_as_default: |name| format!("{name} as the default format"),
    pause_background: "Pause the background",
    pause_background_hint: "Hold the animated background still while y2e is not in front.",
    pause_background_label: "Pause the background when y2e is not in front",
    language: "Language",
    language_hint: "The language of the interface.",
    done: "Done",

    err_generic: "Something went wrong.",
    err_unavailable: "This video is not available.",
    err_truncated: "This link looks incomplete. Copy the whole link.",
    err_unsupported: "This link is not supported.",
    err_verification: "YouTube is asking for a verification.",
    err_network: "Could not connect. Check your network.",
    err_tools_missing: "yt-dlp or ffmpeg was not found on this computer.",
    err_disk_full: "There is no disk space left.",
    err_paste_link: "Paste or type a YouTube link.",
    err_bad_range: "The start of the clip must come before its end.",
    problem_no_data_folder: "The data folder was not found.",
    problem_no_build: |tool| format!("No {tool} build exists for this system."),
    problem_missing_helper: "curl or tar is missing on this system.",
    problem_server_silent: "The ffmpeg download server did not answer.",
    problem_unreadable: |url| format!("Could not read {url}."),
    problem_checksum_missing: |file| format!("The checksum of {file} was not found."),
    problem_corrupted: |file| format!("The file {file} is corrupted and was rejected."),
};

pub static FRENCH: Strings = Strings {
    hero_first: "Une vidéo.",
    hero_second: "Tous les formats.",
    tagline: "MP4 · MP3 · Discord · GIF. Juste le passage qui vous intéresse.",
    link_label: "Lien de la vidéo YouTube",
    link_placeholder: "Collez un lien YouTube",
    load: "Charger",
    loading: "Chargement…",
    clipboard_link: "Lien trouvé dans le presse-papiers. Appuyez sur Entrée pour le charger.",
    fetching_details: "Récupération des informations de la vidéo",
    unknown_author: "Auteur inconnu",
    video_loaded: |length| format!("Vidéo chargée · {length}"),
    video_loaded_plain: "Vidéo chargée",

    install_failed: "L’installation a échoué.",
    install_auto: "Installer automatiquement",
    try_again: "Réessayer",
    update: "Mettre à jour",
    later: "Plus tard",
    got_it: "Compris",
    preparing: "Préparation",
    step_downloading: |tool, n, of| format!("Téléchargement de {tool} ({n}/{of})"),
    step_verifying: |tool| format!("Vérification de {tool}"),
    step_installing: |tool| format!("Installation de {tool}"),
    missing_ytdlp: "yt-dlp est introuvable.",
    missing_ffmpeg: "ffmpeg est introuvable.",
    missing_both: "yt-dlp et ffmpeg sont introuvables.",
    can_install: |mb| format!("y2e peut les installer pour vous (environ {mb} Mo à télécharger, une seule fois)."),
    install_brew: |list| format!("Installez-les avec Homebrew (brew install {list}), puis relancez y2e."),
    install_winget: |list| format!("Installez-les avec winget (winget install {list}), puis relancez y2e."),
    install_by_package_manager: "Installez-les avec votre gestionnaire de paquets, puis relancez y2e.",
    update_ytdlp: "Une mise à jour de yt-dlp est disponible.",
    update_ffmpeg: "Une mise à jour de ffmpeg est disponible.",
    update_both: "Des mises à jour sont disponibles.",
    stay_up_to_date: "Rester à jour évite les erreurs quand YouTube change.",
    update_with_package_manager: |list| format!("Mettez à jour avec votre gestionnaire de paquets : {list}."),
    and: " et ",
    ffmpeg_is_old: "installé il y a plus de 4 mois",

    app_update_headline: "Une nouvelle version de y2e est disponible.",
    app_update_cause: |new, current| format!("La version {new} est sortie, vous avez la {current}."),
    see_whats_new: "Voir les nouveautés",

    clip_range: "Plage du clip",
    clip_described: |from, to| format!("Clip de {from} à {to}."),
    whole_described: |length| format!("Vidéo entière, {length}."),
    clip_keys_help: "Les flèches déplacent la fin du clip, Maj avec une flèche déplace le début, Origine sélectionne toute la vidéo.",
    clip_summary: |from, to| format!("Clip  {from} → {to}"),
    select_whole: "Sélectionner toute la vidéo",
    all: "Tout",
    whole_hint: "Vidéo entière. Faites glisser sur la forme d’onde pour découper.",

    export: "Exporter",
    export_progress: "Progression de l’export",
    cancel: "Annuler",
    cancel_export: "Annuler l’export",
    max_quality: "Qualité max",
    audio: "Audio",
    loop_gif: "Boucle",
    gif_large: |length| format!("Un GIF de {length} sera lourd. Un extrait court se partage plus facilement."),
    keep_seconds: |n| format!("Garder {n} s"),
    switch_to_mp4: "Passer en MP4",
    unit_mb: "Mo",
    unit_gb: "Go",
    decimal: ',',
    export_finished: "Export terminé",
    open_folder: "Ouvrir le dossier",

    settings: "Réglages",
    export_folder: "Dossier d’export",
    choose_folder: "Choisir le dossier d’export",
    change: "Modifier…",
    reset: "Réinitialiser",
    use_downloads: "Utiliser le dossier Téléchargements",
    folder_dialog_title: "Dossier d’export",
    folder_dialog_failed: |why| format!("Impossible d’ouvrir le sélecteur de dossier : {why}"),
    default_format: "Format par défaut",
    default_format_hint: "Sélectionné au démarrage de y2e.",
    format_as_default: |name| format!("{name} comme format par défaut"),
    pause_background: "Figer l’arrière-plan",
    pause_background_hint: "Immobilise l’arrière-plan animé quand y2e n’est pas au premier plan.",
    pause_background_label: "Figer l’arrière-plan quand y2e n’est pas au premier plan",
    language: "Langue",
    language_hint: "La langue de l’interface.",
    done: "Terminé",

    err_generic: "Une erreur s’est produite.",
    err_unavailable: "Cette vidéo n’est pas disponible.",
    err_truncated: "Ce lien semble incomplet. Copiez le lien en entier.",
    err_unsupported: "Ce lien n’est pas pris en charge.",
    err_verification: "YouTube demande une vérification.",
    err_network: "Connexion impossible. Vérifiez votre réseau.",
    err_tools_missing: "yt-dlp ou ffmpeg est introuvable sur cet ordinateur.",
    err_disk_full: "Il n’y a plus d’espace disque disponible.",
    err_paste_link: "Collez ou saisissez un lien YouTube.",
    err_bad_range: "Le début du clip doit précéder sa fin.",
    problem_no_data_folder: "Le dossier de données est introuvable.",
    problem_no_build: |tool| format!("Il n’existe pas de version de {tool} pour ce système."),
    problem_missing_helper: "curl ou tar est absent de ce système.",
    problem_server_silent: "Le serveur de téléchargement de ffmpeg n’a pas répondu.",
    problem_unreadable: |url| format!("Impossible de lire {url}."),
    problem_checksum_missing: |file| format!("La somme de contrôle de {file} est introuvable."),
    problem_corrupted: |file| format!("Le fichier {file} est corrompu et a été rejeté."),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_system_language_is_french_only_for_french_locales() {
        assert_eq!(Lang::from_locale(Some("fr-FR")), Lang::French);
        assert_eq!(Lang::from_locale(Some("fr_CA.UTF-8")), Lang::French);
        assert_eq!(Lang::from_locale(Some("de-DE")), Lang::English);
        assert_eq!(Lang::from_locale(Some("en-US")), Lang::English);
        assert_eq!(Lang::from_locale(None), Lang::English);
    }

    #[test]
    fn texts_with_blanks_fill_them_in_both_languages() {
        assert_eq!(
            (ENGLISH.step_downloading)("yt-dlp", 1, 2),
            "Downloading yt-dlp (1/2)"
        );
        assert_eq!(
            (FRENCH.step_downloading)("yt-dlp", 1, 2),
            "Téléchargement de yt-dlp (1/2)"
        );
        assert_eq!((FRENCH.keep_seconds)(8), "Garder 8 s");
    }

    #[test]
    fn no_text_is_left_empty() {
        for strings in [&ENGLISH, &FRENCH] {
            for text in [
                strings.hero_first,
                strings.hero_second,
                strings.load,
                strings.export,
                strings.cancel,
                strings.settings,
                strings.done,
                strings.err_generic,
                strings.err_unavailable,
                strings.language,
            ] {
                assert!(!text.is_empty());
            }
        }
    }
}
