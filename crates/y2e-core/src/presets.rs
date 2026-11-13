use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DownloadPreset {
    #[default]
    MaxQuality,
    Discord25Mb,
    AudioMp3,
    GifMeme,
}

impl DownloadPreset {
    pub fn label(&self) -> &'static str {
        match self {
            Self::MaxQuality => "Qualité Max (MP4)",
            Self::Discord25Mb => "Discord (<25 Mo)",
            Self::AudioMp3 => "Audio Podcast (MP3)",
            Self::GifMeme => "Animation GIF",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::MaxQuality => "Meilleure résolution disponible sans compromis",
            Self::Discord25Mb => "Compressé automatiquement pour être partagé sur Discord sans Nitro",
            Self::AudioMp3 => "Piste sonore seule avec jaquette et métadonnées",
            Self::GifMeme => "Boucle GIF courte optimisée pour les mèmes",
        }
    }

    /// Rough size of the exported file in megabytes, from the measured output of
    /// the encoders used here. `None` when it depends too much on the source
    /// (the original streams are kept untouched).
    pub fn estimated_size_mb(&self, seconds: u64) -> Option<f64> {
        let seconds = seconds as f64;
        match self {
            Self::MaxQuality => None,
            Self::AudioMp3 => Some(seconds * 0.03),
            Self::Discord25Mb => Some(24.0_f64.min(seconds * 5.0)),
            Self::GifMeme => Some(seconds * 1.9),
        }
    }

    pub fn calculate_discord_video_bitrate_kbps(duration_seconds: u64) -> u64 {
        let duration = duration_seconds.max(1);
        let target_size_kilobits = 24.0 * 8.0 * 1024.0; // 24 MB target in kilobits
        let total_bitrate_kbps = (target_size_kilobits / duration as f64) as u64;
        let audio_bitrate = 96; // 96 kbps audio

        total_bitrate_kbps.saturating_sub(audio_bitrate).max(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimates_grow_with_duration_and_discord_stays_capped() {
        assert_eq!(DownloadPreset::MaxQuality.estimated_size_mb(60), None);
        let gif_short = DownloadPreset::GifMeme.estimated_size_mb(5).unwrap();
        let gif_long = DownloadPreset::GifMeme.estimated_size_mb(600).unwrap();
        assert!(gif_long > gif_short * 50.0);
        assert!(DownloadPreset::Discord25Mb.estimated_size_mb(3600).unwrap() <= 24.0);
    }

    #[test]
    fn test_discord_bitrate_calculation() {
        // 60 seconds video
        let bitrate_60s = DownloadPreset::calculate_discord_video_bitrate_kbps(60);
        // Total kbits = 196608 / 60 = 3276 kbps - 96 = ~3180 kbps
        assert!(bitrate_60s > 3000 && bitrate_60s < 3300);

        // 10 minutes video (600s)
        let bitrate_600s = DownloadPreset::calculate_discord_video_bitrate_kbps(600);
        // Total kbits = 196608 / 600 = 327 kbps - 96 = ~231 kbps
        assert!(bitrate_600s > 200 && bitrate_600s < 260);
    }
}
