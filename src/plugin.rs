//! [`MotionPlugin`]: installs the Motion animation assets in an Autumn app.
//!
//! The plugin installs the [`MOTION_ASSETS`] bundle through Autumn's
//! `AppBuilder::plugin_assets` seam: the vendored Motion UMD build, the
//! declarative init script, and the default stylesheet are served under
//! `/static/_plugins/motion/` with content-hashed URLs. No configuration, no
//! startup hooks.

use std::borrow::Cow;

use autumn_web::app::AppBuilder;
use autumn_web::plugin::Plugin;

use crate::assets::MOTION_ASSETS;

/// The plugin name in Autumn diagnostics.
pub const PLUGIN_NAME: &str = "autumn-plugin-motion";

/// Installs the Motion animation assets in an Autumn app.
///
/// ```rust,no_run
/// use autumn_plugin_motion::MotionPlugin;
///
/// # async fn run() {
/// autumn_web::app()
///     .plugin(MotionPlugin::new())
///     .run()
///     .await;
/// # }
/// ```
///
/// Then include [`motion_script`](crate::motion_script) in the page and
/// animate with [`Motion`](crate::Motion) or raw `data-motion` attributes.
#[derive(Debug, Default)]
#[must_use]
pub struct MotionPlugin;

impl MotionPlugin {
    /// Makes the plugin. It reads no configuration.
    pub const fn new() -> Self {
        Self
    }
}

impl Plugin for MotionPlugin {
    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed(PLUGIN_NAME)
    }

    fn build(self, app: AppBuilder) -> AppBuilder {
        app.plugin_assets(&MOTION_ASSETS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::{INIT_JS, MOTION_CSS, MOTION_JS};
    use autumn_web::assets::{PLUGIN_ASSETS_ROUTE_MARKER, asset_url};
    use autumn_web::plugin_conformance::{ConformanceConfig, run_conformance};
    use autumn_web::route_listing::{RouteClassification, RouteSource};
    use autumn_web::test::{TestApp, TestClient};

    const JS: &str = "text/javascript; charset=utf-8";
    const CSS: &str = "text/css; charset=utf-8";
    const IMMUTABLE: &str = "public, max-age=31536000, immutable";
    const REVALIDATE: &str = "public, max-age=0, must-revalidate";

    /// A test client for an app with only the plugin installed.
    fn client() -> TestClient {
        TestApp::new().plugin(MotionPlugin::new()).build()
    }

    /// The fingerprinted URL of a bundled file (test helper).
    fn url(path: &str) -> String {
        MOTION_ASSETS.url(path)
    }

    #[tokio::test]
    async fn motion_js_serves_at_its_fingerprinted_url() {
        let response = client().get(&url(MOTION_JS)).send().await;
        response
            .assert_ok()
            .assert_header("content-type", JS)
            .assert_header("cache-control", IMMUTABLE);
        assert!(
            response.body.starts_with(b"!function"),
            "serves the minified UMD build"
        );
        assert!(
            response
                .body
                .windows(b"window.Motion".len())
                .any(|w| w == b"window.Motion"),
            "UMD build targets the Motion global"
        );
    }

    #[tokio::test]
    async fn init_js_serves_at_its_fingerprinted_url() {
        let response = client().get(&url(INIT_JS)).send().await;
        response
            .assert_ok()
            .assert_header("content-type", JS)
            .assert_header("cache-control", IMMUTABLE);
        let text = response.text();
        assert!(
            text.contains("htmx:afterSwap"),
            "init script re-scans on htmx swaps"
        );
        assert!(
            text.contains("prefers-reduced-motion"),
            "init script respects reduced motion"
        );
    }

    #[tokio::test]
    async fn motion_css_serves_at_its_fingerprinted_url() {
        let response = client().get(&url(MOTION_CSS)).send().await;
        response
            .assert_ok()
            .assert_header("content-type", CSS)
            .assert_header("cache-control", IMMUTABLE);
        assert!(
            response.text().contains(".motion-progress"),
            "stylesheet styles the progress bar"
        );
    }

    #[tokio::test]
    async fn plain_urls_serve_with_revalidation_and_etags() {
        let client = client();
        for path in [MOTION_JS, INIT_JS, MOTION_CSS] {
            let plain = format!("/static/_plugins/motion/{path}");
            let response = client.get(&plain).send().await;
            response
                .assert_ok()
                .assert_header("cache-control", REVALIDATE);
            let etag = response.header("etag").expect("etag is set").to_owned();
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
            // Vendoring provenance is never served.
            "/static/_plugins/motion/manifest.json",
            // An older release's hash.
            "/static/_plugins/motion/init.00000000.js",
            "/static/_plugins/motion/nope.js",
            // The pre-0.2 namespace is gone.
            "/__motion/init.js",
        ] {
            client.get(path).send().await.assert_status(404);
        }
    }

    #[tokio::test]
    async fn asset_url_resolves_the_installed_bundle() {
        let _client = client();
        for path in [MOTION_JS, INIT_JS, MOTION_CSS] {
            assert_eq!(asset_url(&format!("_plugins/motion/{path}")), url(path));
        }
    }

    #[test]
    fn bundle_routes_are_public_plugin_routes() {
        let app = autumn_web::app().plugin(MotionPlugin::new());
        let infos = app.plugin_route_infos().expect("route infos");
        let asset_routes: Vec<_> = infos
            .iter()
            .filter(|info| info.path.starts_with("/static/_plugins/motion/"))
            .collect();
        assert_eq!(
            asset_routes.len(),
            6,
            "three files, two URLs each: {infos:?}"
        );
        for info in asset_routes {
            assert_eq!(info.method, "GET");
            assert_eq!(info.classification, RouteClassification::Public);
            assert_eq!(info.middleware, [PLUGIN_ASSETS_ROUTE_MARKER]);
            assert_eq!(info.source, RouteSource::Plugin(PLUGIN_NAME.to_owned()));
        }
    }

    #[test]
    fn plugin_passes_conformance() {
        let app = autumn_web::app().plugin(MotionPlugin::new());
        let infos = app.plugin_route_infos().expect("route infos");
        let report = run_conformance(&ConformanceConfig::new(PLUGIN_NAME), &infos);
        assert!(report.passed(), "{}", report.to_text_report());
    }

    #[tokio::test]
    async fn installing_the_plugin_twice_is_harmless() {
        let client = TestApp::new()
            .plugin(MotionPlugin::new())
            .plugin(MotionPlugin::new())
            .build();
        client.get(&url(INIT_JS)).send().await.assert_ok();
    }
}
