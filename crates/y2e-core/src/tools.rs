//! The external tools the engine drives: finding them, and fetching them when
//! they are missing. A tool on `PATH` always wins; otherwise the copy the app
//! installed in its own data directory is used.

use std::env::consts::{ARCH, EXE_SUFFIX, OS};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tokio::process::Command;

use crate::engine::run_quiet;
use crate::error::{Result, Y2eError};

const TOOLS: [&str; 2] = ["yt-dlp", "ffmpeg"];
const YTDLP_BASE: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download";
const FFMPEG_BASE: &str = "https://github.com/yt-dlp/FFmpeg-Builds/releases/latest/download";

/// The file name of a tool on this system (`yt-dlp.exe` on Windows).
fn exe(tool: &str) -> String {
    format!("{tool}{EXE_SUFFIX}")
}

/// A command for a helper program. On Windows it gets no console window, which
/// would otherwise flash on screen each time the app runs something.
pub fn background(program: impl AsRef<OsStr>) -> Command {
    #[cfg_attr(not(windows), allow(unused_mut))]
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    cmd
}

/// Where tools fetched by the app live: `~/.local/share/y2e/bin` on Linux,
/// `~/Library/Application Support/y2e/bin` on macOS, `%LOCALAPPDATA%\y2e\bin` on Windows.
pub fn managed_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|dir| dir.join("y2e").join("bin"))
}

/// Where the system keeps its tools: `PATH`, plus the Homebrew folders on
/// macOS, which an app started from the Finder does not get in its `PATH`.
fn search_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).collect())
        .unwrap_or_default();
    if cfg!(target_os = "macos") {
        dirs.extend(["/opt/homebrew/bin", "/usr/local/bin"].map(PathBuf::from));
    }
    dirs
}

/// The copy of `tool` the system provides.
fn system_copy(tool: &str) -> Option<PathBuf> {
    let name = exe(tool);
    search_dirs()
        .into_iter()
        .map(|dir| dir.join(&name))
        .find(|path| path.is_file())
}

pub(crate) fn on_path(tool: &str) -> bool {
    system_copy(tool).is_some()
}

fn installed(tool: &str) -> Option<PathBuf> {
    managed_dir()
        .map(|dir| dir.join(exe(tool)))
        .filter(|path| path.is_file())
}

/// The copy the app installed itself, unless the system provides the tool:
/// that one belongs to the user's package manager and is never replaced.
fn managed(tool: &str) -> Option<PathBuf> {
    installed(tool).filter(|_| !on_path(tool))
}

/// What to run for `tool`: the one on `PATH`, else the app's own copy.
pub fn command(tool: &str) -> Command {
    match system_copy(tool).or_else(|| installed(tool)) {
        Some(path) => background(path),
        None => background(tool),
    }
}

/// Where yt-dlp should look for ffmpeg, when it is not in `PATH` (the app's own
/// copy, or Homebrew's on macOS).
pub fn ffmpeg_location() -> Option<PathBuf> {
    let dir = system_copy("ffmpeg")
        .or_else(|| installed("ffmpeg"))?
        .parent()?
        .to_path_buf();
    let in_path = std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|entry| entry == dir));
    (!in_path).then_some(dir)
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
        ("yt-dlp", "macos", _) => "yt-dlp_macos",
        ("yt-dlp", "windows", "x86_64") => "yt-dlp.exe",
        ("yt-dlp", "windows", "aarch64") => "yt-dlp_arm64.exe",
        ("ffmpeg", "linux", "x86_64") => "ffmpeg-master-latest-linux64-gpl.tar.xz",
        ("ffmpeg", "linux", "aarch64") => "ffmpeg-master-latest-linuxarm64-gpl.tar.xz",
        ("ffmpeg", "windows", "x86_64") => "ffmpeg-master-latest-win64-gpl.zip",
        ("ffmpeg", "windows", "aarch64") => "ffmpeg-master-latest-winarm64-gpl.zip",
        // No ffmpeg builds are published for macOS: Homebrew provides it.
        _ => return None,
    })
}

/// Whether the app can fetch `tool` itself on this system.
pub fn installable(tool: &str) -> bool {
    asset(tool, OS, ARCH).is_some()
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
            detail: "installed more than 4 months ago".to_string(),
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
    let mut cmd = background("curl");
    cmd.args(["-fL", "--silent", "--show-error", "--retry", "2", "-o"])
        .arg(to)
        .arg(url);
    run_quiet(cmd, "curl").await
}

async fn fetch_text(url: &str) -> Result<String> {
    let out = background("curl")
        .args(["-fsSL", "--retry", "2", url])
        .output()
        .await?;
    if !out.status.success() {
        return Err(failure(format!("Could not read {url}.")));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

async fn sha256(file: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let file = file.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let mut hasher = Sha256::new();
        std::io::copy(&mut std::fs::File::open(file)?, &mut hasher)?;
        Ok(format!("{:x}", hasher.finalize()))
    })
    .await
    .map_err(|err| failure(err.to_string()))?
}

/// The `tar` that reads zip and xz archives alike: GNU tar on Linux, bsdtar on
/// macOS and Windows (where the system one is used, not one a PATH entry shadows).
fn tar() -> Command {
    match std::env::var_os("SystemRoot") {
        Some(root) if cfg!(windows) => {
            background(Path::new(&root).join("System32").join("tar.exe"))
        }
        _ => background("tar"),
    }
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
            failure("curl or tar is missing on this system.")
        }
        other => other,
    })
}

async fn install_all(tools: &[&str], say: impl Fn(String) + Send) -> Result<()> {
    let dir = managed_dir().ok_or_else(|| failure("Data folder not found."))?;
    tokio::fs::create_dir_all(&dir).await?;

    for (i, tool) in tools.iter().enumerate() {
        let file = asset(tool, std::env::consts::OS, std::env::consts::ARCH)
            .ok_or_else(|| failure(format!("No {tool} build for this system.")))?;
        let (base, sums) = if *tool == "yt-dlp" {
            (YTDLP_BASE, "SHA2-256SUMS")
        } else {
            (FFMPEG_BASE, "checksums.sha256")
        };
        let part = dir.join(format!("{file}.part"));

        say(format!("Downloading {tool} ({}/{})", i + 1, tools.len()));
        fetch(&format!("{base}/{file}"), &part).await?;

        say(format!("Verifying {tool}"));
        let listing = fetch_text(&format!("{base}/{sums}")).await?;
        let expected = expected_hash(&listing, file)
            .ok_or_else(|| failure(format!("Checksum for {file} not found.")))?;
        if sha256(&part).await? != expected {
            let _ = tokio::fs::remove_file(&part).await;
            return Err(failure(format!(
                "The file {file} is corrupted and was rejected."
            )));
        }

        say(format!("Installing {tool}"));
        if *tool == "yt-dlp" {
            make_executable(&part)?;
            tokio::fs::rename(&part, dir.join(exe("yt-dlp"))).await?;
        } else {
            // The archive holds ffmpeg and ffprobe (yt-dlp needs both) under <name>/bin/.
            let staging = dir.join("ffmpeg-new");
            let _ = tokio::fs::remove_dir_all(&staging).await;
            tokio::fs::create_dir_all(&staging).await?;
            let mut cmd = tar();
            cmd.arg("-xf").arg(&part).arg("-C").arg(&staging);
            cmd.arg("--strip-components=2");
            if OS == "linux" {
                cmd.arg("--wildcards"); // GNU tar needs it, bsdtar does not know it
            }
            cmd.args([
                format!("*/bin/{}", exe("ffmpeg")),
                format!("*/bin/{}", exe("ffprobe")),
            ]);
            run_quiet(cmd, "tar").await?;
            for bin in ["ffmpeg", "ffprobe"] {
                tokio::fs::rename(staging.join(exe(bin)), dir.join(exe(bin))).await?;
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
    fn keeps_the_tools_in_a_folder_of_its_own() {
        let dir = managed_dir().expect("a data folder");
        assert!(dir.ends_with("y2e/bin") || dir.ends_with(r"y2e\bin"));
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
    fn builds_are_known_where_they_are_published() {
        assert_eq!(asset("yt-dlp", "windows", "x86_64"), Some("yt-dlp.exe"));
        assert!(asset("ffmpeg", "windows", "x86_64").is_some_and(|f| f.ends_with(".zip")));
        assert!(asset("yt-dlp", "macos", "aarch64").is_some());
        assert!(asset("ffmpeg", "macos", "aarch64").is_none());
        assert!(asset("ffmpeg", "freebsd", "x86_64").is_none());
    }

    #[test]
    fn a_hash_is_computed_without_external_tools() {
        let file = std::env::temp_dir().join("y2e-sha-test");
        std::fs::write(&file, "abc").unwrap();
        let hash = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(sha256(&file))
            .unwrap();
        let _ = std::fs::remove_file(&file);
        assert_eq!(
            hash,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    /// Downloads about 190 MB: `cargo test -p y2e-core -- --ignored installs_for_real`,
    /// ideally with `XDG_DATA_HOME` pointing at a scratch directory and a `PATH` without the tools.
    #[tokio::test]
    #[ignore]
    async fn installs_for_real() {
        let steps = std::sync::Mutex::new(Vec::new());
        let ours: Vec<&str> = TOOLS.into_iter().filter(|t| installable(t)).collect();
        install(&ours, |s| steps.lock().unwrap().push(s))
            .await
            .unwrap();
        println!("{:?}", steps.lock().unwrap());
        for tool in ours {
            let flag = if tool == "ffmpeg" {
                "-version"
            } else {
                "--version"
            };
            let out = command(tool).arg(flag).output().await.unwrap();
            assert!(out.status.success(), "{tool} does not run");
        }
        // What the system gives (Homebrew's ffmpeg on macOS) completes the set.
        assert!(missing().is_empty(), "still missing: {:?}", missing());
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
