//! Recognises a YouTube link in arbitrary text (the clipboard).

const HOSTS: &[&str] = &[
    "youtube.com",
    "www.youtube.com",
    "m.youtube.com",
    "music.youtube.com",
    "youtu.be",
];

/// The trimmed link when `text` is exactly one http(s) link to YouTube.
pub fn youtube_link(text: &str) -> Option<String> {
    let text = text.trim();
    if text.is_empty() || text.len() > 300 || text.contains(char::is_whitespace) {
        return None;
    }
    let rest = text
        .strip_prefix("https://")
        .or_else(|| text.strip_prefix("http://"))?;
    let host = rest.split(['/', '?', '#']).next()?.to_lowercase();
    HOSTS.contains(&host.as_str()).then(|| text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_the_usual_youtube_links() {
        for link in [
            "https://www.youtube.com/watch?v=aqz-KE-bpKQ",
            "  https://youtu.be/jNQXAC9IVRw\n",
            "http://m.youtube.com/watch?v=abc",
            "https://music.youtube.com/watch?v=abc",
        ] {
            assert!(youtube_link(link).is_some(), "{link}");
        }
    }

    #[test]
    fn rejects_everything_else() {
        for text in [
            "",
            "hello world",
            "https://example.com/watch?v=abc",
            "https://www.youtube.com.evil.test/watch?v=abc",
            "voici https://youtu.be/abc",
            "youtube.com/watch?v=abc",
        ] {
            assert_eq!(youtube_link(text), None, "{text}");
        }
    }
}
