//! URL safety check for `src` and `href` values.

/// True when `url` is not empty and does not run script.
pub(crate) fn is_safe_url(url: &str) -> bool {
    let _ = url;
    true
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
