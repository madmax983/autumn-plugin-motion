//! HTTP routes serving the vendored Motion assets.
//!
//! The plugin mounts these at `/{*path}`-free, fixed paths under the
//! `/__motion` namespace so they never collide with app routes:
//!
//! - `GET /__motion/motion.min.js` — the vendored Motion UMD build.
//! - `GET /__motion/init.js` — the plugin's declarative `data-motion`
//!   scanner with the htmx re-scan hook.
//! - `GET /__motion/motion.css` — default styles for
//!   `Motion::scroll_progress()` (include via `motion_stylesheet()`).

use autumn_web::reexports::axum::Router;
use autumn_web::reexports::axum::http::{HeaderValue, StatusCode, header};
use autumn_web::reexports::axum::response::{IntoResponse, Response};
use autumn_web::reexports::axum::routing::get;

/// `Content-Type` for the served JavaScript, mirroring the framework's
/// static-asset policy.
const JS_CONTENT_TYPE: &str = "text/javascript; charset=utf-8";

/// `Cache-Control` for the vendored assets. The bytes are pinned per plugin
/// version, so a day of public caching is safe; a plugin upgrade ships new
/// bytes under the same URL at most once per release.
const CACHE_CONTROL: &str = "public, max-age=86400";

/// The axum router the plugin merges into the app.
///
/// Generic over the router state so tests can drive it as `Router<()>` via
/// `oneshot` (axum only implements `Service` for `Router<()>`); the plugin
/// instantiates it at `AppState`. (`Router` itself is already `#[must_use]`.)
pub fn motion_routes<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/__motion/motion.min.js", get(serve_motion_js))
        .route("/__motion/init.js", get(serve_init_js))
        .route("/__motion/motion.css", get(serve_motion_css))
}

/// Serves the vendored Motion UMD build.
async fn serve_motion_js() -> Response {
    asset_response("motion.min.js")
}

/// Serves the plugin's declarative init script.
async fn serve_init_js() -> Response {
    asset_response("init.js")
}

/// `Content-Type` for the plugin stylesheet.
const CSS_CONTENT_TYPE: &str = "text/css; charset=utf-8";

/// Serves the plugin stylesheet (default styles for
/// `Motion::scroll_progress()`).
async fn serve_motion_css() -> Response {
    crate::assets::file("motion.css").map_or_else(
        || StatusCode::NOT_FOUND.into_response(),
        |bytes| {
            (
                StatusCode::OK,
                [
                    (
                        header::CONTENT_TYPE,
                        HeaderValue::from_static(CSS_CONTENT_TYPE),
                    ),
                    (
                        header::CACHE_CONTROL,
                        HeaderValue::from_static(CACHE_CONTROL),
                    ),
                ],
                bytes,
            )
                .into_response()
        },
    )
}

/// Serves one vendored asset from memory with JS content type and caching.
fn asset_response(name: &str) -> Response {
    crate::assets::file(name).map_or_else(
        || StatusCode::NOT_FOUND.into_response(),
        |bytes| {
            (
                StatusCode::OK,
                [
                    (
                        header::CONTENT_TYPE,
                        HeaderValue::from_static(JS_CONTENT_TYPE),
                    ),
                    (
                        header::CACHE_CONTROL,
                        HeaderValue::from_static(CACHE_CONTROL),
                    ),
                ],
                bytes,
            )
                .into_response()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use autumn_web::reexports::axum::body::Body;
    use autumn_web::reexports::axum::extract::Path;
    use autumn_web::reexports::axum::http::Request;
    use tower::ServiceExt as _;

    /// Extracts the `name` path segment and serves it; unknown names 404.
    /// Covers the not-found branch of `asset_response`.
    fn serve_named_asset(Path(name): Path<String>) -> Response {
        asset_response(&name)
    }

    /// Performs a GET against the plugin router, axum-level.
    async fn get(path: &str) -> Response {
        motion_routes::<()>()
            .oneshot(
                Request::builder()
                    .uri(path)
                    .body(Body::empty())
                    .expect("test request builds"),
            )
            .await
            .expect("router responds")
    }

    #[tokio::test]
    async fn motion_js_serves_with_js_content_type() {
        let response = get("/__motion/motion.min.js").await;
        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .expect("content type is set");
        assert_eq!(content_type, JS_CONTENT_TYPE);
        let body = axum_body_to_bytes(response.into_body()).await;
        assert!(
            body.starts_with(b"!function"),
            "serves the minified UMD build"
        );
        assert!(
            body.windows(b"window.Motion".len())
                .any(|w| w == b"window.Motion"),
            "UMD build targets the Motion global"
        );
    }

    #[tokio::test]
    async fn init_js_serves_with_js_content_type() {
        let response = get("/__motion/init.js").await;
        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .expect("content type is set");
        assert_eq!(content_type, JS_CONTENT_TYPE);
        let body = axum_body_to_bytes(response.into_body()).await;
        let text = std::str::from_utf8(&body).expect("init.js is UTF-8");
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
    async fn motion_css_serves_with_css_content_type() {
        let response = get("/__motion/motion.css").await;
        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .expect("content type is set");
        assert_eq!(content_type, CSS_CONTENT_TYPE);
        let body = axum_body_to_bytes(response.into_body()).await;
        let text = std::str::from_utf8(&body).expect("motion.css is UTF-8");
        assert!(
            text.contains(".motion-progress"),
            "stylesheet styles the progress bar"
        );
    }

    #[tokio::test]
    async fn unknown_asset_path_is_not_found() {
        let response = serve_named_asset(Path("nope.js".to_string()));
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    /// Drains an axum body to bytes (test helper).
    async fn axum_body_to_bytes(body: Body) -> Vec<u8> {
        autumn_web::reexports::axum::body::to_bytes(body, usize::MAX)
            .await
            .expect("body reads")
            .to_vec()
    }
}
