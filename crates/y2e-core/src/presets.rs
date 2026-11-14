use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DownloadPreset {
    #[default]
    MaxQuality,
    // Named for the limit it once targeted; settings saved then still load.
    #[serde(alias = "Discord25Mb")]
    Discord20Mb,
    AudioMp3,
    GifMeme,
}

/// Discord accepts 20 MB; aim a little under it.
const DISCORD_TARGET_MB: f64 = 19.0;

impl DownloadPreset {
    /// Rough size of the exported file in megabytes, from the measured output of
    /// the encoders used here. `None` when it depends too much on the source
    /// (the original streams are kept untouched).
    pub fn estimated_size_mb(&self, seconds: u64) -> Option<f64> {
        let seconds = seconds as f64;
        match self {
            Self::MaxQuality => None,
            Self::AudioMp3 => Some(seconds * 0.03),
            Self::Discord20Mb => Some(DISCORD_TARGET_MB.min(seconds * 5.0)),
            Self::GifMeme => Some(seconds * 1.9),
        }
    }

    pub fn calculate_discord_video_bitrate_kbps(duration_seconds: u64) -> u64 {
        let duration = duration_seconds.max(1);
        let target_size_kilobits = DISCORD_TARGET_MB * 8.0 * 1024.0;
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
        assert!(DownloadPreset::Discord20Mb.estimated_size_mb(3600).unwrap() <= 19.0);
    }

    #[test]
    fn test_discord_bitrate_calculation() {
        // 60 seconds video
        let bitrate_60s = DownloadPreset::calculate_discord_video_bitrate_kbps(60);
        // Total kbits = 155648 / 60 = 2594 kbps - 96 = ~2498 kbps
        assert!(bitrate_60s > 2400 && bitrate_60s < 2600);

        // 10 minutes video (600s)
        let bitrate_600s = DownloadPreset::calculate_discord_video_bitrate_kbps(600);
        // Total kbits = 155648 / 600 = 259 kbps - 96 = ~163 kbps
        assert!(bitrate_600s > 150 && bitrate_600s < 180);
    }

    #[test]
    fn settings_saved_with_the_old_name_still_load() {
        let preset: DownloadPreset = serde_json::from_str("\"Discord25Mb\"").unwrap();
        assert_eq!(preset, DownloadPreset::Discord20Mb);
    }
}
