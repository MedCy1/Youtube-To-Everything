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
        "Cette vidéo n’est pas disponible.",
    ),
    (
        &["incomplete youtube id", "looks truncated"],
        "Ce lien semble incomplet. Copiez-le en entier.",
    ),
    (
        &["unsupported url", "is not a valid url"],
        "Ce lien n’est pas pris en charge.",
    ),
    (
        &[
            "sign in to confirm",
            "confirm you’re not a bot",
            "confirm you're not a bot",
        ],
        "YouTube demande une vérification.",
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
        "Connexion impossible. Vérifiez votre réseau.",
    ),
    (
        &[
            "no such file or directory",
            "os error 2",
            "program not found",
        ],
        "yt-dlp ou ffmpeg est introuvable sur cet ordinateur.",
    ),
    (
        &["no space left"],
        "Il n’y a plus d’espace disque disponible.",
    ),
];

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
            headline: "Une erreur est survenue.",
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
        assert_eq!(report.headline, "Cette vidéo n’est pas disponible.");
        assert_eq!(report.cause, None);
    }

    #[test]
    fn recognises_the_other_wording_of_an_unavailable_video() {
        let report = explain("ERROR: [youtube] abc: This video is unavailable");
        assert_eq!(report.headline, "Cette vidéo n’est pas disponible.");
    }

    #[test]
    fn recognises_a_truncated_link() {
        let report = explain("ERROR: Incomplete YouTube ID -zzzzzz. URL looks truncated.");
        assert_eq!(
            report.headline,
            "Ce lien semble incomplet. Copiez-le en entier."
        );
    }

    #[test]
    fn falls_back_to_a_generic_headline_and_bounds_the_cause() {
        let report = explain(&format!("ERROR: {}", "x".repeat(400)));
        assert_eq!(report.headline, "Une erreur est survenue.");
        assert!(report.cause.unwrap().chars().count() <= MAX_CAUSE + 1);
    }

    #[test]
    fn shows_the_last_line_of_an_ffmpeg_failure() {
        let report = explain(
            "Process execution error: ffmpeg exited with code Some(1), stderr: Stream map error\nInvalid argument",
        );
        assert_eq!(report.headline, "Une erreur est survenue.");
        assert_eq!(report.cause.as_deref(), Some("Invalid argument"));
    }
}
