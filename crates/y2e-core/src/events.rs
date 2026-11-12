use std::path::PathBuf;
use crate::metadata::VideoMetadata;
use crate::progress::ProgressUpdate;

#[derive(Debug, Clone)]
pub enum CoreEvent {
    MetadataLoaded(VideoMetadata),
    DownloadStarted,
    Progress(ProgressUpdate),
    StatusChanged(String),
    Finished { output_path: PathBuf },
    Error(String),
}
