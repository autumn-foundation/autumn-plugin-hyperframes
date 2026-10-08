//! Vendored HyperFrames files and plugin files, embedded at compile time.
//!
//! [`HYPERFRAMES_ASSETS`] holds five files:
//!
//! - `hyperframes-player.global.js`: the upstream `@hyperframes/player` IIFE build.
//! - `hyperframe.runtime.iife.js`: the upstream `@hyperframes/core` runtime IIFE build.
//! - `init.js`: the plugin scanner (controls, in-view play, reduced motion, htmx).
//! - `hyperframes.css`: host page styles (player size).
//! - `composition.css`: composition styles (root box, full-frame clips).
//!
//! [`HyperframesPlugin`](crate::HyperframesPlugin) installs the bundle. Autumn
//! serves each file under `/static/_plugins/hyperframes/` at a hashed URL and
//! computes its SRI hash. The bundle lists its files one by one, so
//! `manifest.json` (provenance only) is not served.

use autumn_web::assets::PluginAssets;

/// URL namespace of the bundle. Files serve under `/static/_plugins/hyperframes/`.
pub const ASSETS_NAMESPACE: &str = "hyperframes";

/// Logical path of the vendored player.
pub(crate) const PLAYER_JS: &str = "hyperframes-player.global.js";

/// Logical path of the vendored runtime.
pub(crate) const RUNTIME_JS: &str = "hyperframe.runtime.iife.js";

/// Logical path of the plugin scanner.
pub(crate) const INIT_JS: &str = "init.js";

/// Logical path of the host page stylesheet.
pub(crate) const HOST_CSS: &str = "hyperframes.css";

/// Logical path of the composition stylesheet.
pub(crate) const COMPOSITION_CSS: &str = "composition.css";

/// The plugin asset bundle.
///
/// [`HyperframesPlugin`](crate::HyperframesPlugin) installs it. Use it directly
/// only to make URLs or tags yourself:
///
/// ```rust
/// use autumn_plugin_hyperframes::HYPERFRAMES_ASSETS;
///
/// let url = HYPERFRAMES_ASSETS.url("init.js");
/// assert!(url.starts_with("/static/_plugins/hyperframes/init."), "{url}");
/// let sri = HYPERFRAMES_ASSETS.integrity("init.js").expect("init.js is bundled");
/// assert!(sri.starts_with("sha384-"));
/// ```
pub static HYPERFRAMES_ASSETS: PluginAssets = PluginAssets::from_files(
    ASSETS_NAMESPACE,
    &[
        (
            PLAYER_JS,
            include_bytes!("../assets/hyperframes-player.global.js"),
        ),
        (
            RUNTIME_JS,
            include_bytes!("../assets/hyperframe.runtime.iife.js"),
        ),
        (INIT_JS, include_bytes!("../assets/init.js")),
        (HOST_CSS, include_bytes!("../assets/hyperframes.css")),
        (COMPOSITION_CSS, include_bytes!("../assets/composition.css")),
    ],
);

/// Pinned HyperFrames version of the vendored player and runtime.
pub const HYPERFRAMES_VERSION: &str = "0.8.140";

/// jsDelivr source URL of the vendored player.
pub const PLAYER_SOURCE: &str =
    "https://cdn.jsdelivr.net/npm/@hyperframes/player@0.8.140/dist/hyperframes-player.global.js";

/// jsDelivr source URL of the vendored runtime.
pub const RUNTIME_SOURCE: &str =
    "https://cdn.jsdelivr.net/npm/@hyperframes/core@0.8.140/dist/hyperframe.runtime.iife.js";

/// `sha384` SRI hash of the upstream player file.
///
/// This is a provenance pin. A test fails when the vendored bytes change.
pub const PLAYER_JS_INTEGRITY: &str =
    "sha384-wYfdLwjuZ+Vt5lRqB1bMMSjwF9iOQLjbZtz6D+Zql2IYDuxnxc80aKnq/4xnRxrA";

/// `sha384` SRI hash of the upstream runtime file.
///
/// This is a provenance pin. A test fails when the vendored bytes change.
pub const RUNTIME_JS_INTEGRITY: &str =
    "sha384-UCnstVlWMl9wteIvcOzh30jSJFGbRklSCBsCo0OcM21ffq2lBFOWkUeiH/la0l/L";

/// The hashed URL of the vendored runtime.
pub(crate) fn runtime_url() -> String {
    HYPERFRAMES_ASSETS.url(RUNTIME_JS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    use sha2::{Digest as _, Sha384};

    fn sri(bytes: &[u8]) -> String {
        format!(
            "sha384-{}",
            base64::engine::general_purpose::STANDARD.encode(Sha384::digest(bytes))
        )
    }

    fn text(path: &str) -> &'static str {
        std::str::from_utf8(HYPERFRAMES_ASSETS.get(path).expect("bundled").bytes()).expect("utf-8")
    }

    #[test]
    fn bundle_holds_exactly_the_five_served_files() {
        let files: Vec<&str> = HYPERFRAMES_ASSETS
            .iter()
            .map(autumn_web::assets::PluginAsset::logical_path)
            .collect();
        assert_eq!(
            files,
            // Sorted by logical path. `manifest.json` is not in the bundle.
            [COMPOSITION_CSS, RUNTIME_JS, PLAYER_JS, HOST_CSS, INIT_JS]
        );
        assert_eq!(
            HYPERFRAMES_ASSETS.mount_path(),
            "/static/_plugins/hyperframes"
        );
    }

    #[test]
    fn vendored_files_match_the_pinned_upstream_hashes() {
        let player = HYPERFRAMES_ASSETS.get(PLAYER_JS).expect("player");
        let runtime = HYPERFRAMES_ASSETS.get(RUNTIME_JS).expect("runtime");
        assert_eq!(sri(player.bytes()), PLAYER_JS_INTEGRITY);
        assert_eq!(sri(runtime.bytes()), RUNTIME_JS_INTEGRITY);
        assert_eq!(player.integrity(), PLAYER_JS_INTEGRITY);
        assert_eq!(runtime.integrity(), RUNTIME_JS_INTEGRITY);
    }

    #[test]
    fn bundle_integrity_matches_embedded_bytes() {
        for asset in HYPERFRAMES_ASSETS.iter() {
            assert_eq!(
                asset.integrity(),
                sri(asset.bytes()),
                "{}",
                asset.logical_path()
            );
        }
    }

    #[test]
    fn content_types_match_the_files() {
        let ct = |p| HYPERFRAMES_ASSETS.get(p).expect("bundled").content_type();
        for js in [PLAYER_JS, RUNTIME_JS, INIT_JS] {
            assert_eq!(ct(js), "text/javascript; charset=utf-8");
        }
        for css in [HOST_CSS, COMPOSITION_CSS] {
            assert_eq!(ct(css), "text/css; charset=utf-8");
        }
    }

    #[test]
    fn manifest_agrees_with_constants() {
        let manifest = include_str!("../assets/manifest.json");
        for pin in [
            HYPERFRAMES_VERSION,
            PLAYER_SOURCE,
            RUNTIME_SOURCE,
            PLAYER_JS_INTEGRITY,
            RUNTIME_JS_INTEGRITY,
            "Apache-2.0",
        ] {
            assert!(manifest.contains(pin), "manifest records {pin}");
        }
    }

    #[test]
    fn upstream_player_defines_the_element_and_knows_runtime_src() {
        let player = text(PLAYER_JS);
        assert!(player.contains(r#"customElements.define("hyperframes-player""#));
        assert!(player.contains("runtime-src"), "player reads runtime-src");
        assert!(player.contains(&format!("@hyperframes/core@{HYPERFRAMES_VERSION}/")));
    }

    #[test]
    fn plugin_files_are_not_empty() {
        assert!(text(HOST_CSS).contains("hyperframes-player"));
        let init = text(INIT_JS);
        for hook in [
            "htmx:load",
            "htmx:beforeCleanupElement",
            "prefers-reduced-motion",
            "IntersectionObserver",
            "window.AutumnHyperframes",
        ] {
            assert!(init.contains(hook), "init.js has {hook}");
        }
        assert!(text(COMPOSITION_CSS).contains(".clip"));
    }

    #[test]
    fn init_js_seek_grammar_matches_rust_seconds() {
        // `time::seconds` writes at most three decimals and no exponent.
        assert!(
            text(INIT_JS).contains(r"var RE_SEEK = /^\d+(?:\.\d{1,3})?$/;"),
            "init.js RE_SEEK must match the Rust time format"
        );
    }

    #[test]
    fn runtime_url_is_hashed() {
        let url = runtime_url();
        assert!(
            url.starts_with("/static/_plugins/hyperframes/hyperframe.runtime.iife."),
            "{url}"
        );
        assert_ne!(
            url,
            HYPERFRAMES_ASSETS.get(RUNTIME_JS).expect("rt").plain_url()
        );
    }
}
