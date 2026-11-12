use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq)]
pub struct ProgressUpdate {
    pub percent: f32,
    pub speed: String,
    pub eta: String,
    pub raw: String,
}

static TEMPLATE_RE: OnceLock<Regex> = OnceLock::new();
static STANDARD_RE: OnceLock<Regex> = OnceLock::new();

pub fn parse_progress_line(line: &str) -> Option<ProgressUpdate> {
    let template_re = TEMPLATE_RE.get_or_init(|| {
        Regex::new(r"DOWNLOAD_PROGRESS:\s*([0-9.]+)%:\s*([^:]*):\s*(.*)").unwrap()
    });

    if let Some(caps) = template_re.captures(line) {
        let percent = caps[1].parse::<f32>().unwrap_or(0.0);
        let speed = caps[2].trim().to_string();
        let eta = caps[3].trim().to_string();
        return Some(ProgressUpdate {
            percent,
            speed: if speed.is_empty() { "N/A".to_string() } else { speed },
            eta: if eta.is_empty() { "N/A".to_string() } else { eta },
            raw: line.to_string(),
        });
    }

    let standard_re = STANDARD_RE.get_or_init(|| {
        Regex::new(r"\[download\]\s+([0-9.]+)%\s+of\s+.*?\s+at\s+([^\s]+)\s+ETA\s+([^\s]+)").unwrap()
    });

    if let Some(caps) = standard_re.captures(line) {
        let percent = caps[1].parse::<f32>().unwrap_or(0.0);
        let speed = caps[2].trim().to_string();
        let eta = caps[3].trim().to_string();
        return Some(ProgressUpdate {
            percent,
            speed,
            eta,
            raw: line.to_string(),
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_template_progress() {
        let line = "DOWNLOAD_PROGRESS: 45.2%: 12.5MiB/s: 00:15";
        let progress = parse_progress_line(line).expect("Should parse template line");
        assert!((progress.percent - 45.2).abs() < f32::EPSILON);
        assert_eq!(progress.speed, "12.5MiB/s");
        assert_eq!(progress.eta, "00:15");
    }

    #[test]
    fn test_parse_standard_progress() {
        let line = "[download]  68.4% of 150.00MiB at 10.20MiB/s ETA 00:04";
        let progress = parse_progress_line(line).expect("Should parse standard line");
        assert!((progress.percent - 68.4).abs() < f32::EPSILON);
        assert_eq!(progress.speed, "10.20MiB/s");
        assert_eq!(progress.eta, "00:04");
    }

    #[test]
    fn test_ignore_irrelevant_line() {
        let line = "[youtube] Extracting URL: https://www.youtube.com/watch?v=123";
        assert!(parse_progress_line(line).is_none());
    }
}
