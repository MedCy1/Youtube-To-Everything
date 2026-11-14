use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc::Sender;

use crate::error::{Result, Y2eError};
use crate::events::CoreEvent;
use crate::metadata::{TimeRange, VideoMetadata};
use crate::presets::DownloadPreset;
use crate::progress::{ffmpeg_progress, parse_ffmpeg_stats, parse_progress_line};
use crate::tools;

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
        let output = tools::command("yt-dlp")
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

    pub async fn download(
        options: DownloadOptions,
        event_tx: Sender<CoreEvent>,
    ) -> Result<PathBuf> {
        tokio::fs::create_dir_all(&options.output_dir).await?;

        let _ = event_tx.send(CoreEvent::DownloadStarted).await;
        let _ = event_tx
            .send(CoreEvent::StatusChanged("Starting…".to_string()))
            .await;

        let output_template = options.output_dir.join("%(title).100B.%(ext)s");
        let output_template_str = output_template.to_string_lossy().to_string();

        let mut cmd = tools::command("yt-dlp");
        cmd.arg("--newline")
            .arg("--no-playlist")
            .arg("--progress-template")
            .arg("DOWNLOAD_PROGRESS:%(progress._percent_str)s:%(progress._speed_str)s:%(progress._eta_str)s")
            .arg("-o")
            .arg(&output_template_str);

        if let Some(range) = &options.time_range {
            // `--print` below makes yt-dlp quiet, and with it ffmpeg, which is
            // the one that reports how far a cut has got.
            cmd.arg("--no-quiet")
                .arg("--download-sections")
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

        // `--print` implies quiet and simulate: ask for progress explicitly and
        // let yt-dlp report the final path once every post-processor has run.
        if let Some(dir) = tools::ffmpeg_location() {
            cmd.arg("--ffmpeg-location").arg(dir);
        }
        cmd.arg("--progress")
            .arg("--no-simulate")
            .arg("--print")
            .arg("after_move:FILEPATH:%(filepath)s");

        cmd.arg(&options.url);
        // ffmpeg, started by yt-dlp, would otherwise read our terminal.
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn()?;
        let stdout = child.stdout.take().expect("Failed to open child stdout");
        let mut reader = BufReader::new(stdout).lines();

        // Read stderr as it comes, so a chatty yt-dlp can never fill the pipe. ffmpeg
        // reports a cut there, and ends each of its progress lines with `\r`, not a
        // newline. What is not progress is kept for the error message.
        let mut stderr = BufReader::new(child.stderr.take().expect("Failed to open child stderr"));
        let total = options
            .time_range
            .map_or(0, |range| range.duration_seconds());
        let progress_tx = event_tx.clone();
        let share = download_share(options.preset);
        let stderr_text = tokio::spawn(async move {
            let mut text = Vec::new();
            let mut chunk = Vec::new();
            // A video and its audio are cut one after the other; the bar must not go back.
            let mut furthest = 0.0_f32;
            while matches!(stderr.read_until(b'\r', &mut chunk).await, Ok(n) if n > 0) {
                for line in String::from_utf8_lossy(&chunk).split('\n') {
                    match parse_ffmpeg_stats(line, total) {
                        Some(mut update) if total > 0 => {
                            furthest = furthest.max(update.percent * share / 100.0);
                            update.percent = furthest;
                            let _ = progress_tx.send(CoreEvent::Progress(update)).await;
                        }
                        _ => {
                            text.extend_from_slice(line.as_bytes());
                            text.push(b'\n');
                        }
                    }
                }
                chunk.clear();
            }
            text
        });

        let mut last_destination: Option<PathBuf> = None;

        while let Ok(Some(line)) = reader.next_line().await {
            if let Some(mut progress) = parse_progress_line(&line) {
                progress.percent *= share / 100.0;
                let _ = event_tx.send(CoreEvent::Progress(progress)).await;
            } else if let Some(path) = line.strip_prefix("FILEPATH:") {
                last_destination = Some(PathBuf::from(path.trim()));
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
            let detail = stderr_text.await.unwrap_or_default();
            let error_msg = format!(
                "yt-dlp exited with status {:?}, stderr: {}",
                status.code(),
                tail(&detail)
            );
            let _ = event_tx.send(CoreEvent::Error(error_msg.clone())).await;
            return Err(Y2eError::ProcessFailed {
                command: "yt-dlp".to_string(),
                code: status.code(),
                stderr: tail(&detail),
            });
        }

        let downloaded_file =
            last_destination.unwrap_or_else(|| options.output_dir.join("downloaded_media"));

        let final_path = match options.preset {
            DownloadPreset::Discord25Mb => {
                let _ = event_tx
                    .send(CoreEvent::StatusChanged(
                        "Compressing for Discord (< 25 MB)…".to_string(),
                    ))
                    .await;
                let discord_output =
                    Self::compress_for_discord(&downloaded_file, options.time_range, &event_tx)
                        .await?;
                let _ = tokio::fs::remove_file(&downloaded_file).await;
                discord_output
            }
            DownloadPreset::GifMeme => {
                let _ = event_tx
                    .send(CoreEvent::StatusChanged("Creating the GIF…".to_string()))
                    .await;
                let gif_output = Self::convert_to_gif(&downloaded_file, &event_tx).await?;
                let _ = tokio::fs::remove_file(&downloaded_file).await;
                gif_output
            }
            _ => downloaded_file,
        };

        let _ = event_tx
            .send(CoreEvent::Finished {
                output_path: final_path.clone(),
            })
            .await;
        Ok(final_path)
    }

    async fn compress_for_discord(
        input_path: &Path,
        time_range: Option<TimeRange>,
        event_tx: &Sender<CoreEvent>,
    ) -> Result<PathBuf> {
        let output_path = input_path.with_extension("discord.mp4");
        // The real length of the file: with no cut selected the whole video is encoded.
        let duration = match probe_duration(input_path).await {
            Some(seconds) => seconds,
            None => time_range.map_or(60, |r| r.duration_seconds()),
        };
        let video_bitrate = DownloadPreset::calculate_discord_video_bitrate_kbps(duration);

        let mut cmd = tools::command("ffmpeg");
        cmd.args(REPORTING)
            .arg(input_path)
            .args(["-c:v", "libx264", "-b:v"])
            .arg(format!("{video_bitrate}k"))
            .arg("-maxrate")
            .arg(format!("{video_bitrate}k"))
            .arg("-bufsize")
            .arg(format!("{}k", video_bitrate * 2))
            .args(["-c:a", "aac", "-b:a", "96k"])
            .arg(&output_path);
        run_reporting(cmd, duration, event_tx).await?;

        Ok(output_path)
    }

    async fn convert_to_gif(input_path: &Path, event_tx: &Sender<CoreEvent>) -> Result<PathBuf> {
        let output_path = input_path.with_extension("gif");

        let filter =
            "fps=15,scale=480:-1:flags=lanczos,split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse";
        let mut cmd = tools::command("ffmpeg");
        cmd.args(REPORTING)
            .arg(input_path)
            .args(["-vf", filter])
            .arg(&output_path);
        let duration = probe_duration(input_path).await.unwrap_or(0);
        run_reporting(cmd, duration, event_tx).await?;

        Ok(output_path)
    }
}

/// The share of the bar, in percent, that the download takes. Discord and GIF
/// exports are encoded afterwards, and fill what is left.
fn download_share(preset: DownloadPreset) -> f32 {
    match preset {
        DownloadPreset::Discord25Mb | DownloadPreset::GifMeme => 40.0,
        _ => 100.0,
    }
}

/// What ffmpeg gets before its input, to report its progress on stdout.
const REPORTING: [&str; 8] = [
    "-hide_banner",
    "-loglevel",
    "error",
    "-progress",
    "pipe:1",
    "-nostats",
    "-y",
    "-i",
];

/// Runs ffmpeg, built with `REPORTING`, over a file `total` seconds long, and
/// reports how far it has got in the last part of the bar.
async fn run_reporting(mut cmd: Command, total: u64, event_tx: &Sender<CoreEvent>) -> Result<()> {
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut lines =
        BufReader::new(child.stdout.take().expect("Failed to open child stdout")).lines();
    let mut stderr = child.stderr.take().expect("Failed to open child stderr");
    let errors = tokio::spawn(async move {
        let mut text = Vec::new();
        let _ = stderr.read_to_end(&mut text).await;
        text
    });

    let mut speed = String::from("N/A");
    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(value) = line.strip_prefix("speed=") {
            speed = value.trim().to_string();
        } else if let Some(micros) = line
            .strip_prefix("out_time_us=")
            .and_then(|v| v.trim().parse::<f32>().ok())
        {
            let mut update = ffmpeg_progress(micros / 1e6, total, &speed, &line);
            let from = download_share(DownloadPreset::Discord25Mb);
            update.percent = from + update.percent * (100.0 - from) / 100.0;
            let _ = event_tx.send(CoreEvent::Progress(update)).await;
        }
    }
    let status = child.wait().await?;
    if status.success() {
        return Ok(());
    }
    Err(Y2eError::ProcessFailed {
        command: "ffmpeg".to_string(),
        code: status.code(),
        stderr: tail(&errors.await.unwrap_or_default()),
    })
}

/// Length of a media file in whole seconds, as `ffprobe` reports it.
async fn probe_duration(path: &Path) -> Option<u64> {
    let out = tools::command("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "csv=p=0",
        ])
        .arg(path)
        .output()
        .await
        .ok()?;
    let seconds: f64 = String::from_utf8_lossy(&out.stdout).trim().parse().ok()?;
    Some(seconds.ceil() as u64)
}

/// The last few lines of a tool's error output, which is where the cause is.
fn tail(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    lines[lines.len().saturating_sub(4)..].join("\n")
}

/// Runs ffmpeg without letting it write into our terminal; a failure carries
/// ffmpeg's own message.
pub(crate) async fn run_quiet(mut cmd: Command, name: &str) -> Result<()> {
    let out = cmd.stdin(Stdio::null()).output().await?;
    if out.status.success() {
        return Ok(());
    }
    Err(Y2eError::ProcessFailed {
        command: name.to_string(),
        code: out.status.code(),
        stderr: tail(&out.stderr),
    })
}

#[cfg(test)]
mod tests {
    use super::{probe_duration, run_quiet, tail, Engine, Path, Y2eError};
    use crate::events::CoreEvent;
    use crate::tools;

    #[test]
    fn keeps_only_the_last_lines_of_a_tool_output() {
        assert_eq!(tail(b"a\n\nb\nc\nd\ne\n"), "b\nc\nd\ne");
        assert_eq!(tail(b""), "");
    }

    #[tokio::test]
    async fn measures_the_length_of_a_media_file() {
        let file = std::env::temp_dir().join("y2e-probe-test.mp4");
        let mut cmd = tools::command("ffmpeg");
        cmd.args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=black:s=64x64:d=3",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(&file);
        if run_quiet(cmd, "ffmpeg").await.is_err() {
            return; // ffmpeg is not installed here
        }
        assert_eq!(probe_duration(&file).await, Some(3));
        let _ = std::fs::remove_file(&file);
        assert_eq!(probe_duration(Path::new("/nonexistent.mp4")).await, None);
    }

    #[tokio::test]
    async fn a_failing_ffmpeg_reports_its_own_message() {
        let mut cmd = tools::command("ffmpeg");
        cmd.args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            "/nonexistent/in.mp4",
            "/tmp/out.mp4",
        ]);
        match run_quiet(cmd, "ffmpeg").await {
            Err(Y2eError::ProcessFailed { stderr, .. }) => assert!(stderr.contains("nonexistent")),
            Err(Y2eError::Io(_)) => {} // ffmpeg is not installed here
            other => panic!("expected a failure, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn encoding_for_discord_fills_the_end_of_the_bar() {
        let file = std::env::temp_dir().join("y2e-report-test.mp4");
        let mut cmd = tools::command("ffmpeg");
        cmd.args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc=s=320x240:d=3",
            "-pix_fmt",
            "yuv420p",
        ])
        .arg(&file);
        if run_quiet(cmd, "ffmpeg").await.is_err() {
            return; // ffmpeg is not installed here
        }
        let (tx, mut rx) = tokio::sync::mpsc::channel(256);
        let out = Engine::compress_for_discord(&file, None, &tx)
            .await
            .unwrap();
        drop(tx);
        let mut seen = Vec::new();
        while let Some(event) = rx.recv().await {
            if let CoreEvent::Progress(update) = event {
                seen.push(update.percent);
            }
        }
        let _ = std::fs::remove_file(&file);
        let _ = std::fs::remove_file(out);
        assert!(!seen.is_empty(), "no progress was reported");
        assert!(seen.iter().all(|p| (40.0..100.0).contains(p)), "{seen:?}");
        assert!(seen.windows(2).all(|w| w[0] <= w[1]), "{seen:?}");
    }
}
