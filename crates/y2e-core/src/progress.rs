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

/// Reads ffmpeg's own stats line, such as `size=256KiB time=00:00:30.03 speed=1.54x`.
/// yt-dlp hands a cut to ffmpeg and prints nothing itself until it is done.
/// `total`: the length of the cut, in seconds.
pub fn parse_ffmpeg_stats(line: &str, total: u64) -> Option<ProgressUpdate> {
    let field = |name: &str| line.split_once(name)?.1.split_whitespace().next();
    let mut clock = field("time=")?.split(':');
    let hours: f32 = clock.next()?.parse().ok()?;
    let minutes: f32 = clock.next()?.parse().ok()?;
    let seconds: f32 = clock.next()?.parse().ok()?;
    let done = hours * 3600.0 + minutes * 60.0 + seconds;
    let total = total.max(1) as f32;
    let speed = field("speed=").unwrap_or("N/A");
    let eta = match speed.strip_suffix('x').and_then(|x| x.parse::<f32>().ok()) {
        Some(x) if x > 0.0 => {
            let left = ((total - done) / x).max(0.0) as u64;
            format!("{:02}:{:02}", left / 60, left % 60)
        }
        _ => "N/A".to_string(),
    };
    Some(ProgressUpdate {
        // Only the end of the export says it is done.
        percent: (done / total * 100.0).min(99.0),
        speed: speed.to_string(),
        eta,
        raw: line.trim().to_string(),
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

    const STATS: &str =
        "size=     256KiB time=00:00:30.03 bitrate=  69.8kbits/s speed=1.54x elapsed=0:00:19.50";

    #[test]
    fn test_parse_ffmpeg_stats() {
        let progress = parse_ffmpeg_stats(STATS, 40).expect("Should parse ffmpeg stats");
        assert!((progress.percent - 75.075).abs() < 0.01);
        assert_eq!(progress.speed, "1.54x");
        assert_eq!(progress.eta, "00:06");
    }

    #[test]
    fn test_ffmpeg_stats_never_claim_to_be_done() {
        let progress = parse_ffmpeg_stats(STATS, 20).expect("Should parse ffmpeg stats");
        assert!((progress.percent - 99.0).abs() < f32::EPSILON);
        assert_eq!(progress.eta, "00:00");
    }

    #[test]
    fn test_ffmpeg_stats_ignore_other_lines() {
        assert!(parse_ffmpeg_stats("Output #0, webm, to 'file:x.part':", 40).is_none());
        assert!(parse_ffmpeg_stats("size=N/A time=N/A bitrate=N/A", 40).is_none());
    }
}
