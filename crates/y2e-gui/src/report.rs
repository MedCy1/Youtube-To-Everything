//! Turns raw tool errors into something a person can act on.

use crate::i18n::{t, Strings};
use y2e_core::tools::Change;
use y2e_core::InstallProblem;

pub struct Report {
    pub headline: &'static str,
    /// Technical detail, only when the headline could not say what went wrong.
    pub cause: Option<String>,
}

const MAX_CAUSE: usize = 140;

/// Picks the sentence for a known problem out of a language's texts.
type Say = fn(&Strings) -> &'static str;

/// Phrases in a raw error, and what to say about it.
const KNOWN: &[(&[&str], Say)] = &[
    (
        &[
            "unavailable",
            "private video",
            "has been removed",
            "is not available",
        ],
        |t| t.err_unavailable,
    ),
    (&["incomplete youtube id", "looks truncated"], |t| {
        t.err_truncated
    }),
    (&["unsupported url", "is not a valid url"], |t| {
        t.err_unsupported
    }),
    (
        &[
            "sign in to confirm",
            "confirm you’re not a bot",
            "confirm you're not a bot",
        ],
        |t| t.err_verification,
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
        |t| t.err_network,
    ),
    (
        &[
            "no such file or directory",
            "os error 2",
            "program not found",
        ],
        |t| t.err_tools_missing,
    ),
    (&["no space left"], |t| t.err_disk_full),
    (
        &["paste or type a youtube link", "missing or invalid url"],
        |t| t.err_paste_link,
    ),
    (&["invalid time range"], |t| t.err_bad_range),
];

/// How to get tools the app cannot fetch itself, on this system.
fn install_by_hand(tools: &[&str], t: &Strings) -> String {
    let list = tools.join(" ");
    if cfg!(target_os = "macos") {
        (t.install_brew)(&list)
    } else if cfg!(windows) {
        (t.install_winget)(&list)
    } else {
        t.install_by_package_manager.to_string()
    }
}

/// The landing card for tools the app cannot run without. `can_install`: the
/// app can fetch them itself, so say what that costs.
pub fn missing_tools(tools: &[&str], can_install: bool) -> Report {
    missing_tools_in(tools, can_install, t())
}

fn missing_tools_in(tools: &[&str], can_install: bool, t: &Strings) -> Report {
    let headline = match tools {
        ["yt-dlp"] => t.missing_ytdlp,
        ["ffmpeg"] => t.missing_ffmpeg,
        _ => t.missing_both,
    };
    let cause = if can_install {
        let mb: u32 = tools
            .iter()
            .map(|tool| match *tool {
                "ffmpeg" if cfg!(target_os = "macos") => 60,
                "ffmpeg" => 150,
                _ => 40,
            })
            .sum();
        (t.can_install)(mb)
    } else {
        install_by_hand(tools, t)
    };
    Report {
        headline,
        cause: Some(cause),
    }
}

/// The landing card offering newer versions of the tools. What the user's
/// package manager owns is announced, not offered: only they can update it.
pub fn updates(list: &[y2e_core::tools::Update]) -> Report {
    updates_in(list, t())
}

fn updates_in(list: &[y2e_core::tools::Update], t: &Strings) -> Report {
    let names: Vec<&str> = list.iter().map(|u| u.tool).collect();
    let details: Vec<String> = list
        .iter()
        .map(|u| match &u.change {
            Change::Version { from, to } => format!("{from} → {to}"),
            Change::Aged => t.ffmpeg_is_old.to_string(),
        })
        .collect();
    let theirs: Vec<&str> = list.iter().filter(|u| !u.managed).map(|u| u.tool).collect();
    let advice = if theirs.is_empty() {
        t.stay_up_to_date.to_string()
    } else {
        (t.update_with_package_manager)(&theirs.join(t.and))
    };
    Report {
        headline: if names == ["yt-dlp"] {
            t.update_ytdlp
        } else if names == ["ffmpeg"] {
            t.update_ffmpeg
        } else {
            t.update_both
        },
        cause: Some(format!("{}. {advice}", details.join(" · "))),
    }
}

pub fn explain(raw: &str) -> Report {
    explain_in(raw, t())
}

fn explain_in(raw: &str, t: &Strings) -> Report {
    let lower = raw.to_lowercase();
    match KNOWN
        .iter()
        .find(|(needles, _)| needles.iter().any(|needle| lower.contains(needle)))
    {
        Some((_, say)) => Report {
            headline: say(t),
            cause: None,
        },
        None => Report {
            headline: t.err_generic,
            cause: Some(short_cause(raw)),
        },
    }
}

/// An installation problem, worded in the selected language.
pub fn install_problem(problem: &InstallProblem) -> String {
    let t = t();
    match problem {
        InstallProblem::NoDataFolder => t.problem_no_data_folder.to_string(),
        InstallProblem::NoBuild(tool) => (t.problem_no_build)(tool),
        InstallProblem::MissingHelper => t.problem_missing_helper.to_string(),
        InstallProblem::ServerSilent => t.problem_server_silent.to_string(),
        InstallProblem::Unreadable(url) => (t.problem_unreadable)(url),
        InstallProblem::ChecksumMissing(file) => (t.problem_checksum_missing)(file),
        InstallProblem::Corrupted(file) => (t.problem_corrupted)(file),
        InstallProblem::Other(text) => text.clone(),
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
    use crate::i18n::{ENGLISH, FRENCH};

    fn english(raw: &str) -> Report {
        explain_in(raw, &ENGLISH)
    }

    const UNAVAILABLE: &str = "yt-dlp --dump-json https://www.youtube.com/watch?v=zzz exited with code Some(1), stderr: ERROR: [youtube] zzz: Video unavailable";

    #[test]
    fn names_a_known_problem_without_the_technical_noise() {
        let report = english(UNAVAILABLE);
        assert_eq!(report.headline, "This video is not available.");
        assert_eq!(report.cause, None);
    }

    #[test]
    fn recognises_the_other_wording_of_an_unavailable_video() {
        let report = english("ERROR: [youtube] abc: This video is unavailable");
        assert_eq!(report.headline, "This video is not available.");
    }

    #[test]
    fn recognises_a_truncated_link() {
        let report = english("ERROR: Incomplete YouTube ID -zzzzzz. URL looks truncated.");
        assert_eq!(
            report.headline,
            "This link looks incomplete. Copy the whole link."
        );
    }

    #[test]
    fn falls_back_to_a_generic_headline_and_bounds_the_cause() {
        let report = english(&format!("ERROR: {}", "x".repeat(400)));
        assert_eq!(report.headline, "Something went wrong.");
        assert!(report.cause.unwrap().chars().count() <= MAX_CAUSE + 1);
    }

    #[test]
    fn shows_the_last_line_of_an_ffmpeg_failure() {
        let report = english(
            "Process execution error: ffmpeg exited with code Some(1), stderr: Stream map error\nInvalid argument",
        );
        assert_eq!(report.headline, "Something went wrong.");
        assert_eq!(report.cause.as_deref(), Some("Invalid argument"));
    }

    #[test]
    fn names_the_missing_tools() {
        assert_eq!(
            missing_tools_in(&["ffmpeg"], false, &ENGLISH).headline,
            "ffmpeg was not found."
        );
        assert_eq!(
            missing_tools_in(&["yt-dlp", "ffmpeg"], true, &ENGLISH).headline,
            "yt-dlp and ffmpeg were not found."
        );
    }

    #[test]
    fn leaves_the_package_managers_tools_to_the_user() {
        let update = |tool, managed| y2e_core::tools::Update {
            tool,
            change: Change::Version {
                from: "1".to_string(),
                to: "2".to_string(),
            },
            managed,
        };
        let report = updates_in(&[update("yt-dlp", false)], &ENGLISH);
        assert!(report.cause.unwrap().contains("package manager: yt-dlp"));
        let report = updates_in(&[update("yt-dlp", true)], &ENGLISH);
        assert!(!report.cause.unwrap().contains("package manager"));
    }

    #[test]
    fn speaks_french_when_asked_to() {
        assert_eq!(
            explain_in(UNAVAILABLE, &FRENCH).headline,
            "Cette vidéo n’est pas disponible."
        );
        assert_eq!(
            missing_tools_in(&["ffmpeg"], false, &FRENCH).headline,
            "ffmpeg est introuvable."
        );
    }

    #[test]
    fn the_interfaces_own_messages_get_a_headline() {
        assert_eq!(
            english("Missing or invalid URL.").headline,
            "Paste or type a YouTube link."
        );
        assert_eq!(
            english("Invalid time range: start must be less").headline,
            "The start of the clip must come before its end."
        );
    }
}
