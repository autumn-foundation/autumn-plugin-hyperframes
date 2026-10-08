//! Tags for the host page: [`hyperframes_script()`] and [`hyperframes_stylesheet()`].

use autumn_web::{Markup, html};

use crate::assets::{HOST_CSS, HYPERFRAMES_ASSETS, INIT_JS, PLAYER_JS};

/// Renders the `<script>` tags for the player and the plugin scanner.
///
/// Put it in `<head>`. The tags are deferred, so they run in order after the
/// parser: first the player (it defines `<hyperframes-player>`), then `init.js`.
/// Each tag has a hashed URL and an SRI hash.
///
/// The host page does not load the HyperFrames runtime. Only compositions load it.
///
/// ```rust
/// use autumn_plugin_hyperframes::hyperframes_script;
///
/// let html = hyperframes_script().into_string();
/// assert!(html.contains("/static/_plugins/hyperframes/hyperframes-player.global."));
/// ```
#[must_use]
pub fn hyperframes_script() -> Markup {
    html! {
        (HYPERFRAMES_ASSETS.deferred_script_tag(PLAYER_JS))
        (HYPERFRAMES_ASSETS.deferred_script_tag(INIT_JS))
    }
}

/// Renders the `<link>` tag for the host page stylesheet.
///
/// The stylesheet gives the player a block layout and an aspect ratio.
///
/// ```rust
/// use autumn_plugin_hyperframes::hyperframes_stylesheet;
///
/// let html = hyperframes_stylesheet().into_string();
/// assert!(html.contains(r#"href="/static/_plugins/hyperframes/hyperframes."#), "{html}");
/// ```
#[must_use]
pub fn hyperframes_stylesheet() -> Markup {
    HYPERFRAMES_ASSETS.stylesheet_tag(HOST_CSS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::RUNTIME_JS;

    fn asset(path: &str) -> &'static autumn_web::assets::PluginAsset {
        HYPERFRAMES_ASSETS.get(path).expect("bundled")
    }

    #[test]
    fn script_tags_are_deferred_with_sri() {
        let html = hyperframes_script().into_string();
        for path in [PLAYER_JS, INIT_JS] {
            let a = asset(path);
            assert!(html.contains(&format!(r#"src="{}""#, a.url())), "{html}");
            assert!(
                html.contains(&format!(r#"integrity="{}""#, a.integrity())),
                "{html}"
            );
        }
        assert_eq!(html.matches("defer").count(), 2, "{html}");
        assert_eq!(
            html.matches(r#"crossorigin="anonymous""#).count(),
            2,
            "{html}"
        );
    }

    #[test]
    fn player_loads_before_the_scanner() {
        let html = hyperframes_script().into_string();
        let player = html.find(asset(PLAYER_JS).url()).expect("player tag");
        let init = html.find(asset(INIT_JS).url()).expect("init tag");
        assert!(player < init, "{html}");
    }

    #[test]
    fn host_page_never_loads_the_runtime() {
        let html = hyperframes_script().into_string();
        assert!(!html.contains(asset(RUNTIME_JS).url()), "{html}");
    }

    #[test]
    fn stylesheet_link_carries_sri() {
        let html = hyperframes_stylesheet().into_string();
        let css = asset(HOST_CSS);
        assert!(html.contains(r#"rel="stylesheet""#), "{html}");
        assert!(html.contains(&format!(r#"href="{}""#, css.url())), "{html}");
        assert!(
            html.contains(&format!(r#"integrity="{}""#, css.integrity())),
            "{html}"
        );
    }
}
