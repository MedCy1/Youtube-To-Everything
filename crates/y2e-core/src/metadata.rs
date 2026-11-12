use serde::{Deserialize, Serialize};
use crate::error::{Result, Y2eError};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoChapter {
    pub title: String,
    #[serde(rename = "start_time")]
    pub start_seconds: f64,
    #[serde(rename = "end_time")]
    pub end_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoMetadata {
    pub id: String,
    pub title: String,
    pub uploader: Option<String>,
    pub duration: Option<f64>,
    pub thumbnail: Option<String>,
    #[serde(default)]
    pub chapters: Vec<VideoChapter>,
    #[serde(default)]
    pub webpage_url: Option<String>,
}

impl VideoMetadata {
    pub fn duration_seconds(&self) -> u64 {
        self.duration.unwrap_or(0.0).round() as u64
    }

    pub fn formatted_duration(&self) -> String {
        format_time_seconds(self.duration_seconds())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    pub start_seconds: u64,
    pub end_seconds: u64,
}

impl TimeRange {
    pub fn new(start_seconds: u64, end_seconds: u64) -> Result<Self> {
        if start_seconds >= end_seconds {
            return Err(Y2eError::InvalidTimeRange {
                start_seconds,
                end_seconds,
            });
        }
        Ok(Self {
            start_seconds,
            end_seconds,
        })
    }

    pub fn duration_seconds(&self) -> u64 {
        self.end_seconds.saturating_sub(self.start_seconds)
    }

    pub fn to_yt_dlp_section_arg(&self) -> String {
        format!(
            "*{}-{}",
            format_time_hhmmss(self.start_seconds),
            format_time_hhmmss(self.end_seconds)
        )
    }
}

pub fn format_time_seconds(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}

pub fn format_time_hhmmss(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}
