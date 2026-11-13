//! The external tools the engine drives: finding them, and fetching them when
//! they are missing. A tool on `PATH` always wins; otherwise the copy the app
//! installed in its own data directory is used.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use tokio::process::Command;

use crate::engine::run_quiet;
use crate::error::{Result, Y2eError};

const TOOLS: [&str; 2] = ["yt-dlp", "ffmpeg"];
const YTDLP_BASE: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download";
const FFMPEG_BASE: &str = "https://github.com/yt-dlp/FFmpeg-Builds/releases/latest/download";

fn data_dir(xdg: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    let base = match xdg.filter(|v| !v.is_empty()) {
        Some(xdg) => PathBuf::from(xdg),
        None => PathBuf::from(home?).join(".local/share"),
    };
    Some(base.join("y2e").join("bin"))
}

/// Where tools fetched by the app live.
pub fn managed_dir() -> Option<PathBuf> {
    data_dir(std::env::var_os("XDG_DATA_HOME"), std::env::var_os("HOME"))
}

pub(crate) fn on_path(tool: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|dir| dir.join(tool).is_file()))
}

fn installed(tool: &str) -> Option<PathBuf> {
    managed_dir()
        .map(|dir| dir.join(tool))
        .filter(|path| path.is_file())
}

/// The copy the app installed itself, unless the system provides the tool:
/// that one belongs to the user's package manager and is never replaced.
fn managed(tool: &str) -> Option<PathBuf> {
    installed(tool).filter(|_| !on_path(tool))
}

/// What to run for `tool`: the one on `PATH`, else the app's own copy.
pub fn command(tool: &str) -> Command {
    match managed(tool) {
        Some(path) => Command::new(path),
        None => Command::new(tool),
    }
}

/// Where yt-dlp should look for ffmpeg, when only the app's copy exists.
pub fn ffmpeg_location() -> Option<PathBuf> {
    managed("ffmpeg").and_then(|path| path.parent().map(Path::to_path_buf))
}

/// The tools that can be found neither on `PATH` nor among the app's own.
pub fn missing() -> Vec<&'static str> {
    TOOLS
        .into_iter()
        .filter(|tool| !on_path(tool) && installed(tool).is_none())
        .collect()
}

/// The release file holding `tool` for this system, if there is one.
fn asset(tool: &str, os: &str, arch: &str) -> Option<&'static str> {
    Some(match (tool, os, arch) {
        ("yt-dlp", "linux", "x86_64") => "yt-dlp_linux",
        ("yt-dlp", "linux", "aarch64") => "yt-dlp_linux_aarch64",
        ("ffmpeg", "linux", "x86_64") => "ffmpeg-master-latest-linux64-gpl.tar.xz",
        ("ffmpeg", "linux", "aarch64") => "ffmpeg-master-latest-linuxarm64-gpl.tar.xz",
        _ => return None,
    })
}

/// Whether the app can fetch its tools itself on this system.
pub fn install_supported() -> bool {
    TOOLS
        .into_iter()
        .all(|tool| asset(tool, std::env::consts::OS, std::env::consts::ARCH).is_some())
}

/// A tool that has a newer version worth installing.
#[derive(Debug, Clone, PartialEq)]
pub struct Update {
    pub tool: &'static str,
    /// What changes, for display.
    pub detail: String,
    /// The app installed this copy and can replace it; otherwise it belongs to
    /// the user's package manager and only they can update it.
    pub managed: bool,
}

/// Release versions are dates (`2026.10.26`), so text order is time order.
fn newer(latest: &str, current: &str) -> bool {
    latest > current
}

const FFMPEG_MARKER: &str = "ffmpeg.installed";
/// ffmpeg changes slowly; the app's own copy is only suggested for renewal after this long.
const FFMPEG_MAX_AGE: std::time::Duration = std::time::Duration::from_secs(120 * 24 * 3600);

async fn version(tool: &str) -> Option<String> {
    let out = command(tool).arg("--version").output().await.ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    text.lines()
        .next()
        .map(|l| l.trim().to_string())
        .filter(|v| !v.is_empty())
}

async fn latest_ytdlp() -> Option<String> {
    let json = fetch_text("https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest")
        .await
        .ok()?;
    serde_json::from_str::<serde_json::Value>(&json)
        .ok()?
        .get("tag_name")?
        .as_str()
        .map(str::to_string)
}

/// Looks for newer versions of what is installed. Quiet on any failure (no
/// network, rate limit): an update check must never get in the way.
pub async fn updates() -> Vec<Update> {
    let mut found = Vec::new();
    if let (Some(current), Some(latest)) = (version("yt-dlp").await, latest_ytdlp().await) {
        if newer(&latest, &current) {
            found.push(Update {
                tool: "yt-dlp",
                detail: format!("{current} → {latest}"),
                managed: managed("yt-dlp").is_some(),
            });
        }
    }
    let age = managed_dir()
        .and_then(|dir| std::fs::metadata(dir.join(FFMPEG_MARKER)).ok())
        .and_then(|meta| meta.modified().ok())
        .and_then(|time| time.elapsed().ok());
    if managed("ffmpeg").is_some() && age.is_some_and(|age| age > FFMPEG_MAX_AGE) {
        found.push(Update {
            tool: "ffmpeg",
            detail: "installé depuis plus de 4 mois".to_string(),
            managed: true,
        });
    }
    found
}

fn failure(what: impl Into<String>) -> Y2eError {
    Y2eError::Install(what.into())
}

/// The checksum listed for `file` in a `sha256sum`-style listing.
fn expected_hash<'a>(listing: &'a str, file: &str) -> Option<&'a str> {
    listing.lines().find_map(|line| {
        let (hash, name) = line.split_once(char::is_whitespace)?;
        (name.trim().trim_start_matches('*') == file).then_some(hash)
    })
}

async fn fetch(url: &str, to: &Path) -> Result<()> {
    let mut cmd = Command::new("curl");
    cmd.args(["-fL", "--silent", "--show-error", "--retry", "2", "-o"])
        .arg(to)
        .arg(url);
    run_quiet(cmd, "curl").await
}

async fn fetch_text(url: &str) -> Result<String> {
    let out = Command::new("curl")
        .args(["-fsSL", "--retry", "2", url])
        .output()
        .await?;
    if !out.status.success() {
        return Err(failure(format!("Impossible de lire {url}.")));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

async fn sha256(file: &Path) -> Result<String> {
    let out = Command::new("sha256sum").arg(file).output().await?;
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()
        .map(str::to_string)
        .ok_or_else(|| failure("La vérification n'a rien renvoyé."))
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    Ok(std::fs::set_permissions(
        path,
        std::fs::Permissions::from_mode(0o755),
    )?)
}

#[cfg(not(unix))]
fn make_executable(_: &Path) -> Result<()> {
    Ok(())
}

/// Downloads and installs `tools` into the app's own directory, checking each
/// file against the checksum its release publishes. `say` receives what is
/// happening, for display.
pub async fn install(tools: &[&str], say: impl Fn(String) + Send) -> Result<()> {
    install_all(tools, say).await.map_err(|err| match err {
        Y2eError::Io(io) if io.kind() == std::io::ErrorKind::NotFound => {
            failure("Il manque curl, tar ou sha256sum sur ce système.")
        }
        other => other,
    })
}

async fn install_all(tools: &[&str], say: impl Fn(String) + Send) -> Result<()> {
    let dir = managed_dir().ok_or_else(|| failure("Dossier de données introuvable."))?;
    tokio::fs::create_dir_all(&dir).await?;

    for (i, tool) in tools.iter().enumerate() {
        let file = asset(tool, std::env::consts::OS, std::env::consts::ARCH)
            .ok_or_else(|| failure(format!("Pas de version de {tool} pour ce système.")))?;
        let (base, sums) = if *tool == "yt-dlp" {
            (YTDLP_BASE, "SHA2-256SUMS")
        } else {
            (FFMPEG_BASE, "checksums.sha256")
        };
        let part = dir.join(format!("{file}.part"));

        say(format!(
            "Téléchargement de {tool} ({}/{})",
            i + 1,
            tools.len()
        ));
        fetch(&format!("{base}/{file}"), &part).await?;

        say(format!("Vérification de {tool}"));
        let listing = fetch_text(&format!("{base}/{sums}")).await?;
        let expected = expected_hash(&listing, file)
            .ok_or_else(|| failure(format!("Somme de contrôle de {file} introuvable.")))?;
        if sha256(&part).await? != expected {
            let _ = tokio::fs::remove_file(&part).await;
            return Err(failure(format!(
                "Le fichier {file} est corrompu, il a été refusé."
            )));
        }

        say(format!("Installation de {tool}"));
        if *tool == "yt-dlp" {
            make_executable(&part)?;
            tokio::fs::rename(&part, dir.join("yt-dlp")).await?;
        } else {
            // The archive holds ffmpeg and ffprobe (yt-dlp needs both) under <name>/bin/.
            let staging = dir.join("ffmpeg-new");
            let _ = tokio::fs::remove_dir_all(&staging).await;
            tokio::fs::create_dir_all(&staging).await?;
            let mut cmd = Command::new("tar");
            cmd.arg("-xJf").arg(&part).arg("-C").arg(&staging).args([
                "--strip-components=2",
                "--wildcards",
                "*/bin/ffmpeg",
                "*/bin/ffprobe",
            ]);
            run_quiet(cmd, "tar").await?;
            for bin in ["ffmpeg", "ffprobe"] {
                tokio::fs::rename(staging.join(bin), dir.join(bin)).await?;
            }
            let _ = tokio::fs::remove_dir_all(&staging).await;
            let _ = tokio::fs::remove_file(&part).await;
            tokio::fs::write(dir.join(FFMPEG_MARKER), "").await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_tools_in_the_users_data_directory() {
        let home = Some(OsString::from("/home/a"));
        assert_eq!(
            data_dir(None, home.clone()),
            Some(PathBuf::from("/home/a/.local/share/y2e/bin"))
        );
        assert_eq!(
            data_dir(Some(OsString::from("/x")), home),
            Some(PathBuf::from("/x/y2e/bin"))
        );
        assert_eq!(data_dir(None, None), None);
    }

    #[test]
    fn reads_the_checksum_of_one_file_from_a_listing() {
        let listing = "aaa  other\nbbb *yt-dlp_linux\nccc  yt-dlp_linux_aarch64\n";
        assert_eq!(expected_hash(listing, "yt-dlp_linux"), Some("bbb"));
        assert_eq!(expected_hash(listing, "missing"), None);
    }

    #[test]
    fn a_release_is_newer_only_when_its_date_is_later() {
        assert!(newer("2026.11.02", "2026.10.26"));
        assert!(!newer("2026.10.26", "2026.10.26"));
        assert!(!newer("2026.10.26", "2026.11.02"));
    }

    #[test]
    fn only_linux_builds_are_known() {
        assert!(asset("ffmpeg", "linux", "x86_64").is_some());
        assert!(asset("yt-dlp", "windows", "x86_64").is_none());
    }

    /// Downloads about 190 MB: `cargo test -p y2e-core -- --ignored installs_for_real`,
    /// ideally with `XDG_DATA_HOME` pointing at a scratch directory and a `PATH` without the tools.
    #[tokio::test]
    #[ignore]
    async fn installs_for_real() {
        let steps = std::sync::Mutex::new(Vec::new());
        install(&["yt-dlp", "ffmpeg"], |s| steps.lock().unwrap().push(s))
            .await
            .unwrap();
        assert!(missing().is_empty());
        println!("{:?}", steps.lock().unwrap());
        let version = command("yt-dlp").arg("--version").output().await.unwrap();
        assert!(version.status.success());
        let ffmpeg = command("ffmpeg").arg("-version").output().await.unwrap();
        assert!(ffmpeg.status.success());
    }

    /// Needs the tools installed by `installs_for_real` and network access.
    #[tokio::test]
    #[ignore]
    async fn exports_with_the_installed_tools() {
        use crate::{DownloadOptions, DownloadPreset, Engine};
        let out = std::env::temp_dir().join("y2e-tools-test");
        let (tx, mut rx) = tokio::sync::mpsc::channel(256);
        tokio::spawn(async move { while rx.recv().await.is_some() {} });
        let file = Engine::download(
            DownloadOptions {
                url: "https://www.youtube.com/watch?v=jNQXAC9IVRw".into(),
                time_range: None,
                preset: DownloadPreset::AudioMp3,
                output_dir: out,
            },
            tx,
        )
        .await
        .unwrap();
        assert!(file.extension().is_some_and(|e| e == "mp3") && file.is_file());
        println!("{}", file.display());
    }
}
