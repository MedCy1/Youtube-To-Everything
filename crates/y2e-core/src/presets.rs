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
