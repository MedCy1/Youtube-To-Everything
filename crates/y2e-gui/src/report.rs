//! Turns raw tool errors into something a person can act on.

pub struct Report {
    pub headline: &'static str,
    /// Technical detail, only when the headline could not say what went wrong.
    pub cause: Option<String>,
}

const MAX_CAUSE: usize = 140;

const KNOWN: &[(&[&str], &str)] = &[
    (
        &[
            "unavailable",
            "private video",
            "has been removed",
            "is not available",
        ],
        "This video is not available.",
    ),
    (
        &["incomplete youtube id", "looks truncated"],
        "This link looks incomplete. Copy the whole link.",
    ),
    (
        &["unsupported url", "is not a valid url"],
        "This link is not supported.",
    ),
    (
        &[
            "sign in to confirm",
            "confirm you’re not a bot",
            "confirm you're not a bot",
        ],
        "YouTube is asking for a verification.",
    ),
    (
        &[
            "name or service not known",
            "temporary failure",
            "network is unreachable",
            "timed out",
            "unable to download",
            "connection",
        ],
        "Could not connect. Check your network.",
    ),
    (
        &[
            "no such file or directory",
            "os error 2",
            "program not found",
        ],
        "yt-dlp or ffmpeg was not found on this computer.",
    ),
    (
        &["no space left"],
        "Il n’y a plus d’espace disque disponible.",
    ),
];

/// The landing card for tools the app cannot run without. `can_install`: the
/// app can fetch them itself, so say what that costs.
pub fn missing_tools(tools: &[&str], can_install: bool) -> Report {
    let headline = match tools {
        ["yt-dlp"] => "yt-dlp was not found.",
        ["ffmpeg"] => "ffmpeg was not found.",
        _ => "yt-dlp and ffmpeg were not found.",
    };
    let cause = if can_install {
        let mb: u32 = tools
            .iter()
            .map(|t| if *t == "ffmpeg" { 150 } else { 40 })
            .sum();
        format!("y2e can install them for you (about {mb} MB to download, once).")
    } else {
        "Install them with your package manager, then restart y2e.".to_string()
    };
    Report {
        headline,
        cause: Some(cause),
    }
}

/// The landing card offering newer versions of the tools. What the user's
/// package manager owns is announced, not offered: only they can update it.
pub fn updates(list: &[y2e_core::tools::Update]) -> Report {
    let names: Vec<&str> = list.iter().map(|u| u.tool).collect();
    let details: Vec<&str> = list.iter().map(|u| u.detail.as_str()).collect();
    let theirs: Vec<&str> = list.iter().filter(|u| !u.managed).map(|u| u.tool).collect();
    let advice = if theirs.is_empty() {
        "Staying up to date avoids errors when YouTube changes.".to_string()
    } else {
        format!("Update with your package manager: {}.", theirs.join(" et "))
    };
    Report {
        headline: if names == ["yt-dlp"] {
            "A yt-dlp update is available."
        } else if names == ["ffmpeg"] {
            "An ffmpeg update is available."
        } else {
            "Updates are available."
        },
        cause: Some(format!("{}. {advice}", details.join(" · "))),
    }
}

pub fn explain(raw: &str) -> Report {
    let lower = raw.to_lowercase();
    match KNOWN
        .iter()
        .find(|(needles, _)| needles.iter().any(|needle| lower.contains(needle)))
    {
        Some((_, headline)) => Report {
            headline,
            cause: None,
        },
        None => Report {
            headline: "Something went wrong.",
            cause: Some(short_cause(raw)),
        },
    }
}

/// The part of the message after `ERROR:`, without the extractor tag and the
/// video id yt-dlp puts in front of it, cut to one readable line.
fn short_cause(raw: &str) -> String {
    // yt-dlp states its cause on the line after `ERROR:`; ffmpeg's is the last line of its output.
    let from_ffmpeg = !raw.contains("ERROR:") && raw.contains("stderr:");
    let raw = if from_ffmpeg {
        raw.rsplit("stderr:").next().unwrap_or(raw).trim()
    } else {
        raw
    };
    let mut cause = raw.rsplit("ERROR:").next().unwrap_or(raw).trim();
    if let Some(rest) = cause.strip_prefix('[').and_then(|c| c.split_once(']')) {
        cause = rest.1.trim();
    }
    if let Some((id, rest)) = cause.split_once(": ") {
        if !id.contains(' ') {
            cause = rest.trim();
        }
    }
    let mut lines = cause.lines().filter(|l| !l.trim().is_empty());
    let line = if from_ffmpeg {
        lines.next_back()
    } else {
        lines.next()
    }
    .unwrap_or("")
    .trim();
    if line.chars().count() > MAX_CAUSE {
        let cut: String = line.chars().take(MAX_CAUSE).collect();
        format!("{}…", cut.trim_end())
    } else {
        line.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNAVAILABLE: &str = "yt-dlp --dump-json https://www.youtube.com/watch?v=zzz exited with code Some(1), stderr: ERROR: [youtube] zzz: Video unavailable";

    #[test]
    fn names_a_known_problem_without_the_technical_noise() {
        let report = explain(UNAVAILABLE);
        assert_eq!(report.headline, "This video is not available.");
        assert_eq!(report.cause, None);
    }

    #[test]
    fn recognises_the_other_wording_of_an_unavailable_video() {
        let report = explain("ERROR: [youtube] abc: This video is unavailable");
        assert_eq!(report.headline, "This video is not available.");
    }

    #[test]
    fn recognises_a_truncated_link() {
        let report = explain("ERROR: Incomplete YouTube ID -zzzzzz. URL looks truncated.");
        assert_eq!(
            report.headline,
            "This link looks incomplete. Copy the whole link."
        );
    }

    #[test]
    fn falls_back_to_a_generic_headline_and_bounds_the_cause() {
        let report = explain(&format!("ERROR: {}", "x".repeat(400)));
        assert_eq!(report.headline, "Something went wrong.");
        assert!(report.cause.unwrap().chars().count() <= MAX_CAUSE + 1);
    }

    #[test]
    fn shows_the_last_line_of_an_ffmpeg_failure() {
        let report = explain(
            "Process execution error: ffmpeg exited with code Some(1), stderr: Stream map error\nInvalid argument",
        );
        assert_eq!(report.headline, "Something went wrong.");
        assert_eq!(report.cause.as_deref(), Some("Invalid argument"));
    }

    #[test]
    fn names_the_missing_tools() {
        assert_eq!(
            missing_tools(&["ffmpeg"], false).headline,
            "ffmpeg was not found."
        );
        assert_eq!(
            missing_tools(&["yt-dlp", "ffmpeg"], true).headline,
            "yt-dlp and ffmpeg were not found."
        );
    }

    #[test]
    fn leaves_the_package_managers_tools_to_the_user() {
        let update = |tool, managed| y2e_core::tools::Update {
            tool,
            detail: "1 → 2".to_string(),
            managed,
        };
        let report = updates(&[update("yt-dlp", false)]);
        assert!(report.cause.unwrap().contains("package manager: yt-dlp"));
        let report = updates(&[update("yt-dlp", true)]);
        assert!(!report.cause.unwrap().contains("package manager"));
    }
}
