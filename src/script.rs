//! [`motion_script()`]: the `<script>` tags for the vendored Motion build.
//!
//! Put it in the page `<head>` (or at the end of `<body>`); the scripts are
//! `defer`red so they run after parsing. Both tags carry Subresource
//! Integrity hashes pinned in [`crate::assets`].

use autumn_web::{Markup, html};

use crate::assets::{INIT_JS_INTEGRITY, MOTION_CSS_INTEGRITY, MOTION_JS_INTEGRITY};

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
/// assert!(head.into_string().contains("/__motion/motion.min.js"));
/// ```
#[must_use]
pub fn motion_script() -> Markup {
    html! {
        script src="/__motion/motion.min.js"
            integrity=(MOTION_JS_INTEGRITY)
            crossorigin="anonymous"
            defer {}
        script src="/__motion/init.js"
            integrity=(INIT_JS_INTEGRITY)
            crossorigin="anonymous"
            defer {}
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
/// assert!(html.contains(r#"href="/__motion/motion.css""#), "{html}");
/// ```
#[must_use]
pub fn motion_stylesheet() -> Markup {
    html! {
        link rel="stylesheet" href="/__motion/motion.css"
            integrity=(MOTION_CSS_INTEGRITY)
            crossorigin="anonymous";
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_tags_carry_sri_and_namespaced_paths() {
        let html = motion_script().into_string();
        assert!(html.contains(r#"src="/__motion/motion.min.js""#), "{html}");
        assert!(html.contains(r#"src="/__motion/init.js""#), "{html}");
        assert!(
            html.contains(&format!(r#"integrity="{MOTION_JS_INTEGRITY}""#)),
            "motion script tag carries the SRI hash: {html}"
        );
        assert!(
            html.contains(&format!(r#"integrity="{INIT_JS_INTEGRITY}""#)),
            "init script tag carries the SRI hash: {html}"
        );
        assert!(html.contains("defer"), "{html}");
    }
}
