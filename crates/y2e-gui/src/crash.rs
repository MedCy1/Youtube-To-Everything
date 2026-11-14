//! A crash leaves a trace on disk. On Windows the app has no console, so a
//! panic would otherwise vanish; the file is what a bug report can attach.

use std::io::Write;
use std::path::PathBuf;

/// `y2e.log` in the app's data folder (next to the tools it installs).
pub fn log_path() -> Option<PathBuf> {
    y2e_core::tools::managed_dir()?
        .parent()
        .map(|dir| dir.join("y2e.log"))
}

/// Appends every panic, with the seconds since the Unix epoch, to the log.
pub fn keep_a_log() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        previous(info);
        let Some(path) = log_path() else { return };
        let _ = std::fs::create_dir_all(path.parent().unwrap_or(&path));
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(file, "[{now}] {info}");
        }
    }));
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_log_sits_beside_the_tools_folder() {
        let path = super::log_path().expect("a data folder");
        assert_eq!(path.file_name().unwrap(), "y2e.log");
        assert!(path.to_string_lossy().contains("y2e"));
    }
}
