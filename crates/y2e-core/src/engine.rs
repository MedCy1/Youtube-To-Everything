use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::Sender;

use crate::error::{Result, Y2eError};
use crate::events::CoreEvent;
use crate::metadata::{TimeRange, VideoMetadata};
use crate::presets::DownloadPreset;
use crate::progress::parse_progress_line;

#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub url: String,
    pub time_range: Option<TimeRange>,
    pub preset: DownloadPreset,
    pub output_dir: PathBuf,
}

pub struct Engine;

impl Engine {
    pub async fn fetch_metadata(url: &str) -> Result<VideoMetadata> {
        let output = Command::new("yt-dlp")
            .arg("--dump-json")
            .arg("--skip-download")
            .arg("--no-playlist")
            .arg(url)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(Y2eError::ProcessFailed {
                command: format!("yt-dlp --dump-json {}", url),
                code: output.status.code(),
                stderr,
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let metadata: VideoMetadata = serde_json::from_str(&stdout)?;
        Ok(metadata)
    }

    pub async fn download(options: DownloadOptions, event_tx: Sender<CoreEvent>) -> Result<PathBuf> {
        tokio::fs::create_dir_all(&options.output_dir).await?;

        let _ = event_tx.send(CoreEvent::DownloadStarted).await;
        let _ = event_tx.send(CoreEvent::StatusChanged("Initialisation du téléchargement...".to_string())).await;

        let output_template = options.output_dir.join("%(title).100B.%(ext)s");
        let output_template_str = output_template.to_string_lossy().to_string();

        let mut cmd = Command::new("yt-dlp");
        cmd.arg("--newline")
            .arg("--no-playlist")
            .arg("--progress-template")
            .arg("DOWNLOAD_PROGRESS:%(progress._percent_str)s:%(progress._speed_str)s:%(progress._eta_str)s")
            .arg("-o")
            .arg(&output_template_str);

        if let Some(range) = &options.time_range {
            cmd.arg("--download-sections")
                .arg(range.to_yt_dlp_section_arg())
                .arg("--force-keyframes-at-cuts");
        }

        match options.preset {
            DownloadPreset::MaxQuality => {
                cmd.arg("-f")
                    .arg("bv*[ext=mp4]+ba[ext=m4a]/b[ext=mp4]/bv*+ba/b")
                    .arg("--merge-output-format")
                    .arg("mp4");
            }
            DownloadPreset::AudioMp3 => {
                cmd.arg("-x")
                    .arg("--audio-format")
                    .arg("mp3")
                    .arg("--audio-quality")
                    .arg("0");
            }
            DownloadPreset::Discord25Mb => {
                cmd.arg("-f")
                    .arg("bv*[ext=mp4]+ba[ext=m4a]/b[ext=mp4]/bv*+ba/b")
                    .arg("--merge-output-format")
                    .arg("mp4");
            }
            DownloadPreset::GifMeme => {
                cmd.arg("-f")
                    .arg("bv*[ext=mp4]/b[ext=mp4]/bv*/b")
                    .arg("--merge-output-format")
                    .arg("mp4");
            }
        }

        cmd.arg(&options.url);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().expect("Failed to open child stdout");
        let mut reader = BufReader::new(stdout).lines();

        let mut last_destination: Option<PathBuf> = None;

        while let Ok(Some(line)) = reader.next_line().await {
            if let Some(progress) = parse_progress_line(&line) {
                let _ = event_tx.send(CoreEvent::Progress(progress)).await;
            } else if line.contains("[download] Destination: ") {
                if let Some(dest) = line.strip_prefix("[download] Destination: ") {
                    last_destination = Some(PathBuf::from(dest.trim()));
                }
            } else if line.contains("[Merger] Merging formats into ") {
                if let Some(dest) = line.strip_prefix("[Merger] Merging formats into ") {
                    let cleaned = dest.trim().trim_matches('"');
                    last_destination = Some(PathBuf::from(cleaned));
                }
            } else if line.contains("[ExtractAudio] Destination: ") {
                if let Some(dest) = line.strip_prefix("[ExtractAudio] Destination: ") {
                    last_destination = Some(PathBuf::from(dest.trim()));
                }
            }
        }

        let status = child.wait().await?;
        if !status.success() {
            let error_msg = format!("yt-dlp exited with status {:?}", status.code());
            let _ = event_tx.send(CoreEvent::Error(error_msg.clone())).await;
            return Err(Y2eError::ProcessFailed {
                command: "yt-dlp".to_string(),
                code: status.code(),
                stderr: error_msg,
            });
        }

        let downloaded_file = last_destination.unwrap_or_else(|| {
            options.output_dir.join("downloaded_media")
        });

        let final_path = match options.preset {
            DownloadPreset::Discord25Mb => {
                let _ = event_tx.send(CoreEvent::StatusChanged("Compression Discord (< 25 Mo)...".to_string())).await;
                let discord_output = Self::compress_for_discord(&downloaded_file, options.time_range).await?;
                discord_output
            }
            DownloadPreset::GifMeme => {
                let _ = event_tx.send(CoreEvent::StatusChanged("Génération de l'animation GIF...".to_string())).await;
                let gif_output = Self::convert_to_gif(&downloaded_file).await?;
                gif_output
            }
            _ => downloaded_file,
        };

        let _ = event_tx.send(CoreEvent::Finished { output_path: final_path.clone() }).await;
        Ok(final_path)
    }

    async fn compress_for_discord(input_path: &Path, time_range: Option<TimeRange>) -> Result<PathBuf> {
        let output_path = input_path.with_extension("discord.mp4");
        let duration = time_range.map(|r| r.duration_seconds()).unwrap_or(60);
        let video_bitrate = DownloadPreset::calculate_discord_video_bitrate_kbps(duration);

        let status = Command::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(input_path)
            .arg("-c:v")
            .arg("libx264")
            .arg("-b:v")
            .arg(format!("{}k", video_bitrate))
            .arg("-maxrate")
            .arg(format!("{}k", video_bitrate))
            .arg("-bufsize")
            .arg(format!("{}k", video_bitrate * 2))
            .arg("-c:a")
            .arg("aac")
            .arg("-b:a")
            .arg("96k")
            .arg(&output_path)
            .status()
            .await?;

        if !status.success() {
            return Ok(input_path.to_path_buf());
        }

        Ok(output_path)
    }

    async fn convert_to_gif(input_path: &Path) -> Result<PathBuf> {
        let output_path = input_path.with_extension("gif");

        let filter = "fps=15,scale=480:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse";
        let status = Command::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(input_path)
            .arg("-vf")
            .arg(filter)
            .arg(&output_path)
            .status()
            .await?;

        if !status.success() {
            return Ok(input_path.to_path_buf());
        }

        Ok(output_path)
    }
}
