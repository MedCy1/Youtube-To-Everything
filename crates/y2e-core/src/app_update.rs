//! Finding out whether a newer release of the app itself exists.

use crate::tools::fetch_text;

const LATEST_RELEASE: &str =
    "https://api.github.com/repos/MedCy1/Youtube-To-Everything/releases/latest";

/// A release newer than the running app.
#[derive(Debug, Clone, PartialEq)]
pub struct AppUpdate {
    /// Without the leading `v`: `0.3.0`.
    pub version: String,
    /// The release page, where the packages are.
    pub url: String,
}

/// `0.10.2` as numbers, so that `0.10.0` is later than `0.9.0`. A pre-release
/// suffix (`-rc1`) is dropped.
fn numbers(version: &str) -> Option<Vec<u64>> {
    version
        .trim_start_matches('v')
        .split('-')
        .next()?
        .split('.')
        .map(|part| part.parse().ok())
        .collect()
}

/// The update described by a GitHub "latest release" answer, if it is newer
/// than `current`.
fn from_release(json: &str, current: &str) -> Option<AppUpdate> {
    let release: serde_json::Value = serde_json::from_str(json).ok()?;
    let tag = release.get("tag_name")?.as_str()?;
    let url = release.get("html_url")?.as_str()?;
    (numbers(tag)? > numbers(current)?).then(|| AppUpdate {
        version: tag.trim_start_matches('v').to_string(),
        url: url.to_string(),
    })
}

/// Asks GitHub for the latest release. Quiet on any failure (no network, rate
/// limit, a repository that is not public yet): this check must never get in
/// the way.
pub async fn check(current: &str) -> Option<AppUpdate> {
    from_release(&fetch_text(LATEST_RELEASE).await.ok()?, current)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RELEASE: &str = r#"{"tag_name":"v0.10.0","html_url":"https://example.test/r/v0.10.0"}"#;

    #[test]
    fn a_later_release_is_offered() {
        let update = from_release(RELEASE, "0.9.3").unwrap();
        assert_eq!(update.version, "0.10.0");
        assert_eq!(update.url, "https://example.test/r/v0.10.0");
    }

    #[test]
    fn the_same_or_an_older_release_is_not() {
        assert_eq!(from_release(RELEASE, "0.10.0"), None);
        assert_eq!(from_release(RELEASE, "1.0.0"), None);
    }

    #[test]
    fn unreadable_answers_are_ignored() {
        assert_eq!(from_release("{\"message\":\"Not Found\"}", "0.1.0"), None);
        assert_eq!(from_release("not json", "0.1.0"), None);
    }
}
