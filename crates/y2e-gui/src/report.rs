//! Turns raw tool errors into something a person can act on.

pub struct Report {
    pub headline: &'static str,
    pub cause: String,
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
    let headline = KNOWN
        .iter()
        .find(|(needles, _)| needles.iter().any(|needle| lower.contains(needle)))
        .map_or("Une erreur est survenue.", |(_, headline)| headline);
    Report {
        headline,
        cause: short_cause(raw),
    }
}

/// The part of the message after `ERROR:`, without the extractor tag and the
/// video id yt-dlp puts in front of it, cut to one readable line.
fn short_cause(raw: &str) -> String {
    let mut cause = raw.rsplit("ERROR:").next().unwrap_or(raw).trim();
    if let Some(rest) = cause.strip_prefix('[').and_then(|c| c.split_once(']')) {
        cause = rest.1.trim();
    }
    if let Some((id, rest)) = cause.split_once(": ") {
        if !id.contains(' ') {
            cause = rest.trim();
        }
    }
    let line = cause.lines().next().unwrap_or("").trim();
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
    fn names_the_problem_and_keeps_only_the_cause() {
        let report = explain(UNAVAILABLE);
        assert_eq!(report.headline, "Cette vidéo n’est pas disponible.");
        assert_eq!(report.cause, "Video unavailable");
    }

    #[test]
    fn recognises_the_other_wording_of_an_unavailable_video() {
        let report = explain("ERROR: [youtube] abc: This video is unavailable");
        assert_eq!(report.headline, "Cette vidéo n’est pas disponible.");
    }

    #[test]
    fn falls_back_to_a_generic_headline_and_bounds_the_cause() {
        let report = explain(&format!("ERROR: {}", "x".repeat(400)));
        assert_eq!(report.headline, "Une erreur est survenue.");
        assert!(report.cause.chars().count() <= MAX_CAUSE + 1);
    }
}
