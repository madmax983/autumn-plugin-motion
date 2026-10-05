//! [`motion_script()`]: the `<script>` tags for the vendored Motion build.
//!
//! Put it in the page `<head>` (or at the end of `<body>`); the scripts are
//! `defer`red so they run after parsing, in document order (Motion first,
//! then the init script). Every tag points at the content-hashed URL from
//! [`MOTION_ASSETS`] and carries the Subresource Integrity hash the bundle
//! computes from the embedded bytes.

use autumn_web::{Markup, html};

use crate::assets::{INIT_JS, MOTION_ASSETS, MOTION_CSS, MOTION_JS};

/// Renders the `<script>` tags loading Motion and the plugin init script.
///
/// ```rust
/// use autumn_plugin_motion::motion_script;
/// use autumn_web::html;
///
/// let head = html! {
///     head {
///         (motion_script())
///     }
/// };
/// assert!(head.into_string().contains("/static/_plugins/motion/motion.min."));
/// ```
#[must_use]
pub fn motion_script() -> Markup {
    html! {
        (MOTION_ASSETS.deferred_script_tag(MOTION_JS))
        (MOTION_ASSETS.deferred_script_tag(INIT_JS))
    }
}

/// Renders the `<link>` tag for the plugin stylesheet.
///
/// Only needed when you use [`crate::Motion::scroll_progress`] (or want the
/// plugin's default styles); the animation behavior itself is class-free.
///
/// ```rust
/// use autumn_plugin_motion::motion_stylesheet;
///
/// let html = motion_stylesheet().into_string();
/// assert!(html.contains(r#"href="/static/_plugins/motion/motion."#), "{html}");
/// ```
#[must_use]
pub fn motion_stylesheet() -> Markup {
    MOTION_ASSETS.stylesheet_tag(MOTION_CSS)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bundle entry for `path` (test helper).
    fn asset(path: &str) -> &'static autumn_web::assets::PluginAsset {
        MOTION_ASSETS.get(path).expect("file is bundled")
    }

    #[test]
    fn script_tags_carry_sri_and_fingerprinted_urls() {
        let html = motion_script().into_string();
        let motion = asset(MOTION_JS);
        let init = asset(INIT_JS);
        assert!(
            html.contains(&format!(r#"src="{}""#, motion.url())),
            "{html}"
        );
        assert!(html.contains(&format!(r#"src="{}""#, init.url())), "{html}");
        assert!(
            html.contains(&format!(r#"integrity="{}""#, motion.integrity())),
            "motion script tag carries the SRI hash: {html}"
        );
        assert!(
            html.contains(&format!(r#"integrity="{}""#, init.integrity())),
            "init script tag carries the SRI hash: {html}"
        );
        assert_eq!(html.matches("defer").count(), 2, "{html}");
        assert_eq!(
            html.matches(r#"crossorigin="anonymous""#).count(),
            2,
            "{html}"
        );
        assert!(!html.contains("not found"), "{html}");
    }

    #[test]
    fn motion_loads_before_the_init_script() {
        let html = motion_script().into_string();
        let motion_at = html.find(asset(MOTION_JS).url()).expect("motion tag");
        let init_at = html.find(asset(INIT_JS).url()).expect("init tag");
        assert!(
            motion_at < init_at,
            "deferred scripts run in order; init.js needs window.Motion: {html}"
        );
    }

    #[test]
    fn stylesheet_link_carries_sri_and_fingerprinted_url() {
        let html = motion_stylesheet().into_string();
        let css = asset(MOTION_CSS);
        assert!(html.contains(r#"rel="stylesheet""#), "{html}");
        assert!(html.contains(&format!(r#"href="{}""#, css.url())), "{html}");
        assert!(
            html.contains(&format!(r#"integrity="{}""#, css.integrity())),
            "{html}"
        );
        assert!(html.contains(r#"crossorigin="anonymous""#), "{html}");
    }
}
