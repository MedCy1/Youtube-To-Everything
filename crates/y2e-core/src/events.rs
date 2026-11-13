use crate::progress::ProgressUpdate;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum CoreEvent {
    DownloadStarted,
    Progress(ProgressUpdate),
    StatusChanged(String),
    Finished { output_path: PathBuf },
    Error(String),
}
