//! [`HyperframesPlugin`]: installs the HyperFrames assets in an Autumn app.

use std::borrow::Cow;

use autumn_web::app::AppBuilder;
use autumn_web::plugin::Plugin;

use crate::assets::HYPERFRAMES_ASSETS;

/// The plugin name in Autumn diagnostics.
pub const PLUGIN_NAME: &str = "autumn-plugin-hyperframes";

/// Installs the HyperFrames assets in an Autumn app.
///
/// The plugin installs [`HYPERFRAMES_ASSETS`](crate::HYPERFRAMES_ASSETS) through
/// the Autumn `plugin_assets` seam. It adds no other routes and no startup hooks.
///
/// ```rust,no_run
/// use autumn_plugin_hyperframes::HyperframesPlugin;
///
/// # async fn run() {
/// autumn_web::app()
///     .plugin(HyperframesPlugin::new())
///     .run()
///     .await;
/// # }
/// ```
#[derive(Debug, Default)]
#[must_use]
pub struct HyperframesPlugin;

impl HyperframesPlugin {
    /// Makes the plugin. It reads no configuration.
    pub const fn new() -> Self {
        Self
    }
}

impl Plugin for HyperframesPlugin {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed(PLUGIN_NAME)
    }

    fn build(self, app: AppBuilder) -> AppBuilder {
        app.plugin_assets(&HYPERFRAMES_ASSETS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::{COMPOSITION_CSS, HOST_CSS, INIT_JS, PLAYER_JS, RUNTIME_JS};
    use autumn_web::assets::{PLUGIN_ASSETS_ROUTE_MARKER, asset_url};
    use autumn_web::plugin_conformance::{ConformanceConfig, run_conformance};
    use autumn_web::route_listing::{RouteClassification, RouteSource};
    use autumn_web::test::{TestApp, TestClient};

    const ALL: [&str; 5] = [PLAYER_JS, RUNTIME_JS, INIT_JS, HOST_CSS, COMPOSITION_CSS];
    const IMMUTABLE: &str = "public, max-age=31536000, immutable";
    const REVALIDATE: &str = "public, max-age=0, must-revalidate";

    fn client() -> TestClient {
        TestApp::new().plugin(HyperframesPlugin::new()).build()
    }

    #[tokio::test]
    async fn every_file_serves_at_its_hashed_url() {
        let client = client();
        for path in ALL {
            let asset = HYPERFRAMES_ASSETS.get(path).expect("bundled");
            let response = client.get(asset.url()).send().await;
            response
                .assert_ok()
                .assert_header("content-type", asset.content_type())
                .assert_header("cache-control", IMMUTABLE);
            assert_eq!(response.body.as_slice(), asset.bytes(), "{path}");
        }
    }

    #[tokio::test]
    async fn plain_urls_serve_with_revalidation_and_etags() {
        let client = client();
        for path in ALL {
            let plain = format!("/static/_plugins/hyperframes/{path}");
            let response = client.get(&plain).send().await;
            response
                .assert_ok()
                .assert_header("cache-control", REVALIDATE);
            let etag = response.header("etag").expect("etag").to_owned();
            client
                .get(&plain)
                .header("if-none-match", &etag)
                .send()
                .await
                .assert_status(304);
        }
    }

    #[tokio::test]
    async fn unbundled_and_stale_paths_are_not_found() {
        let client = client();
        for path in [
            "/static/_plugins/hyperframes/manifest.json",
            "/static/_plugins/hyperframes/init.00000000.js",
            "/static/_plugins/hyperframes/nope.js",
        ] {
            client.get(path).send().await.assert_status(404);
        }
    }

    #[tokio::test]
    async fn asset_url_resolves_the_installed_bundle() {
        let _client = client();
        for path in ALL {
            assert_eq!(
                asset_url(&format!("_plugins/hyperframes/{path}")),
                HYPERFRAMES_ASSETS.url(path)
            );
        }
    }

    #[test]
    fn bundle_routes_are_public_plugin_routes() {
        let app = autumn_web::app().plugin(HyperframesPlugin::new());
        let infos = app.plugin_route_infos().expect("route infos");
        let routes: Vec<_> = infos
            .iter()
            .filter(|i| i.path.starts_with("/static/_plugins/hyperframes/"))
            .collect();
        assert_eq!(routes.len(), 10, "five files, two URLs each: {infos:?}");
        for info in routes {
            assert_eq!(info.method, "GET");
            assert_eq!(info.classification, RouteClassification::Public);
            assert_eq!(info.middleware, [PLUGIN_ASSETS_ROUTE_MARKER]);
            assert_eq!(info.source, RouteSource::Plugin(PLUGIN_NAME.to_owned()));
        }
    }

    #[test]
    fn plugin_passes_conformance() {
        let app = autumn_web::app().plugin(HyperframesPlugin::new());
        let infos = app.plugin_route_infos().expect("route infos");
        let report = run_conformance(&ConformanceConfig::new(PLUGIN_NAME), &infos);
        assert!(report.passed(), "{}", report.to_text_report());
    }

    #[tokio::test]
    async fn installing_the_plugin_twice_is_harmless() {
        let client = TestApp::new()
            .plugin(HyperframesPlugin::new())
            .plugin(HyperframesPlugin::new())
            .build();
        client
            .get(&HYPERFRAMES_ASSETS.url(INIT_JS))
            .send()
            .await
            .assert_ok();
    }
}
