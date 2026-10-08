//! URL safety check for `src` and `href` values.

/// True when `url` is not empty and does not run script.
///
/// Browsers drop tab and newline characters in a URL and skip leading control
/// characters, so the check does too before it reads the scheme.
pub(crate) fn is_safe_url(url: &str) -> bool {
    let cleaned: String = url
        .chars()
        .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
        .collect();
    let trimmed = cleaned.trim_matches(|c: char| c <= ' ');
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    !["javascript:", "vbscript:", "data:text/html"]
        .iter()
        .any(|scheme| lower.starts_with(scheme))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_script_urls() {
        for bad in [
            "",
            "   ",
            "javascript:alert(1)",
            "JAVASCRIPT:alert(1)",
            " javascript:x",
            "java\tscript:x",
            "java\nscript:x",
            "\u{1}javascript:x",
            "vbscript:x",
            "data:text/html,<b>",
            "DATA:TEXT/HTML;base64,AA",
        ] {
            assert!(!is_safe_url(bad), "{bad:?}");
        }
    }

    #[test]
    fn accepts_normal_urls() {
        for good in [
            "/static/a.png",
            "a.png",
            "./a.png",
            "https://example.com/a.mp4",
            "data:image/png;base64,AAAA",
            "blob:https://example.com/1",
            "/javascript:not-a-scheme",
        ] {
            assert!(is_safe_url(good), "{good:?}");
        }
    }
}
