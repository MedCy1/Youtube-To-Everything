use crate::progress::ProgressUpdate;
use std::path::PathBuf;

/// What an export is busy with. Each stage reports its own progress, from 0 to 100.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportStage {
    /// Reading the link and starting the tools. Nothing can be measured yet:
    /// the first progress report ends it.
    #[default]
    Preparing,
    Downloading,
    /// Shrinking the file for Discord.
    Compressing,
    /// Making the GIF.
    Converting,
}

#[derive(Debug, Clone)]
pub enum CoreEvent {
    DownloadStarted,
    /// A new stage begins, and its progress starts again from 0.
    StageChanged(ExportStage),
    Progress(ProgressUpdate),
    Finished {
        output_path: PathBuf,
    },
    Error(String),
}
