//! Tags for the host page: [`hyperframes_script()`] and [`hyperframes_stylesheet()`].

use autumn_web::Markup;

/// Renders the `<script>` tags for the player and the plugin scanner.
#[must_use]
pub fn hyperframes_script() -> Markup {
    Markup::default()
}

/// Renders the `<link>` tag for the host page stylesheet.
#[must_use]
pub fn hyperframes_stylesheet() -> Markup {
    Markup::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::{HOST_CSS, HYPERFRAMES_ASSETS, INIT_JS, PLAYER_JS, RUNTIME_JS};

    fn asset(path: &str) -> &'static autumn_web::assets::PluginAsset {
        HYPERFRAMES_ASSETS.get(path).expect("bundled")
    }

    #[test]
    fn script_tags_are_deferred_with_sri() {
        let html = hyperframes_script().into_string();
        for path in [PLAYER_JS, INIT_JS] {
            let a = asset(path);
            assert!(html.contains(&format!(r#"src="{}""#, a.url())), "{html}");
            assert!(html.contains(&format!(r#"integrity="{}""#, a.integrity())), "{html}");
        }
        assert_eq!(html.matches("defer").count(), 2, "{html}");
        assert_eq!(html.matches(r#"crossorigin="anonymous""#).count(), 2, "{html}");
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
        assert!(html.contains(&format!(r#"integrity="{}""#, css.integrity())), "{html}");
    }
}
