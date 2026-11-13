#[derive(Debug, Clone, PartialEq)]
pub struct ProgressUpdate {
    pub percent: f32,
    pub speed: String,
    pub eta: String,
    pub raw: String,
}

fn or_na(s: &str) -> String {
    if s.is_empty() { "N/A" } else { s }.to_string()
}

/// Reads our own progress template, then yt-dlp's standard `[download]` line.
pub fn parse_progress_line(line: &str) -> Option<ProgressUpdate> {
    let raw = line.to_string();
    if let Some((_, rest)) = line.split_once("DOWNLOAD_PROGRESS:") {
        let mut parts = rest.splitn(3, ':');
        let percent = parts.next()?.trim().strip_suffix('%')?.parse().ok()?;
        let speed = or_na(parts.next()?.trim());
        let eta = or_na(parts.next()?.trim());
        return Some(ProgressUpdate {
            percent,
            speed,
            eta,
            raw,
        });
    }
    let (_, rest) = line.split_once("[download]")?;
    let words: Vec<&str> = rest.split_whitespace().collect();
    let percent = words.first()?.strip_suffix('%')?.parse().ok()?;
    let at = words.iter().position(|w| *w == "at")?;
    let (speed, eta) = match words.get(at + 1..at + 4)? {
        [speed, "ETA", eta] => (speed.to_string(), eta.to_string()),
        _ => return None,
    };
    Some(ProgressUpdate {
        percent,
        speed,
        eta,
        raw,
    })
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
