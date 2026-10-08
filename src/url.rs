//! URL allowlist for `src` and `href` values.

/// Where a URL goes. Each use has its own allowlist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UrlUse {
    /// `img`, `video`, `audio` and `poster`: relative, `http(s):`, `blob:`, and
    /// `data:` images (not SVG), videos and audio.
    Media,
    /// Pages, nested compositions, stylesheets and scripts: relative and `http(s):` only.
    Document,
}

/// True when `url` is not empty and its scheme is on the allowlist for `use_`.
///
/// Browsers drop tab and newline characters in a URL and skip leading and
/// trailing control characters and spaces, so the check does too.
pub(crate) fn is_safe_url(url: &str, use_: UrlUse) -> bool {
    let cleaned: String = url
        .chars()
        .filter(|c| !matches!(c, '\t' | '\n' | '\r'))
        .collect();
    let trimmed = cleaned.trim_matches(|c: char| c <= ' ');
    if trimmed.is_empty() {
        return false;
    }
    let Some(scheme) = scheme(trimmed) else {
        // A relative URL (path, `?query`, `#fragment` or `//host`).
        return true;
    };
    match scheme.as_str() {
        "http" | "https" => true,
        "blob" => use_ == UrlUse::Media,
        "data" => use_ == UrlUse::Media && is_media_data(&trimmed[5..]),
        _ => false,
    }
}

/// The lowercase scheme, if `url` has one.
fn scheme(url: &str) -> Option<String> {
    let end = url.find([':', '/', '?', '#'])?;
    if !url[end..].starts_with(':') {
        return None;
    }
    let name = &url[..end];
    let mut chars = name.chars();
    let valid = chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'));
    valid.then(|| name.to_ascii_lowercase())
}

/// True for a `data:` body with an image (not SVG), video or audio type.
fn is_media_data(body: &str) -> bool {
    let mime = body.trim_start().to_ascii_lowercase();
    (mime.starts_with("image/") && !mime.starts_with("image/svg"))
        || mime.starts_with("video/")
        || mime.starts_with("audio/")
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOTH: [UrlUse; 2] = [UrlUse::Media, UrlUse::Document];

    #[test]
    fn rejects_script_and_unknown_schemes() {
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
            "data: text/html,<b>",
            "data:image/svg+xml,<svg/>",
            "data:application/xhtml+xml,x",
            "data:text/xml,x",
            "ftp://x/y",
            "file:///etc/passwd",
        ] {
            for use_ in BOTH {
                assert!(!is_safe_url(bad, use_), "{bad:?} as {use_:?}");
            }
        }
    }

    #[test]
    fn accepts_relative_and_http_urls_everywhere() {
        for good in [
            "/static/a.png",
            "a.png",
            "./a.png",
            "../a.png",
            "?x=1",
            "#top",
            "//cdn.example.com/a.js",
            "https://example.com/a.mp4",
            "HTTP://example.com/a",
            "/javascript:not-a-scheme",
            "a/b:c",
        ] {
            for use_ in BOTH {
                assert!(is_safe_url(good, use_), "{good:?} as {use_:?}");
            }
        }
    }

    #[test]
    fn media_also_accepts_blob_and_media_data() {
        for good in [
            "blob:https://example.com/1",
            "data:image/png;base64,AAAA",
            "data:video/mp4;base64,AAAA",
            "data:audio/mpeg;base64,AAAA",
        ] {
            assert!(is_safe_url(good, UrlUse::Media), "{good:?}");
            assert!(!is_safe_url(good, UrlUse::Document), "{good:?}");
        }
    }
}
