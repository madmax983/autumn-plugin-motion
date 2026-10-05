//! Vendored Motion assets, embedded at compile time.
//!
//! The crate vendors the Motion UMD build (`assets/motion.min.js`, pinned in
//! `assets/manifest.json`), the plugin-authored declarative scanner
//! (`assets/init.js`), and the default stylesheet (`assets/motion.css`).
//!
//! All three form the [`MOTION_ASSETS`] bundle, which
//! [`MotionPlugin`](crate::MotionPlugin) installs through Autumn's
//! `AppBuilder::plugin_assets` seam. The framework serves each file under
//! `/static/_plugins/motion/` at a content-hashed URL (`immutable` for a
//! year) and at its plain URL (`must-revalidate`), with `ETag`/`304` and
//! `Range` support, and computes each file's `sha384` Subresource Integrity
//! hash from the embedded bytes, so nothing here keeps hashes by hand.
//!
//! The bundle lists its files explicitly instead of embedding the whole
//! `assets/` directory, so `manifest.json` (vendoring provenance) is never
//! served.

use autumn_web::assets::PluginAssets;

/// URL namespace of the bundle: files are served under
/// `/static/_plugins/motion/`.
pub const ASSETS_NAMESPACE: &str = "motion";

/// Logical path of the vendored Motion UMD build inside [`MOTION_ASSETS`].
pub(crate) const MOTION_JS: &str = "motion.min.js";

/// Logical path of the declarative init script inside [`MOTION_ASSETS`].
pub(crate) const INIT_JS: &str = "init.js";

/// Logical path of the default stylesheet inside [`MOTION_ASSETS`].
pub(crate) const MOTION_CSS: &str = "motion.css";

/// The plugin's asset bundle: Motion, the init script, and the stylesheet.
///
/// [`MotionPlugin`](crate::MotionPlugin) installs it; you only need it
/// directly to build URLs or tags yourself:
///
/// ```rust
/// use autumn_plugin_motion::MOTION_ASSETS;
///
/// let url = MOTION_ASSETS.url("init.js");
/// assert!(url.starts_with("/static/_plugins/motion/init."), "{url}");
/// let sri = MOTION_ASSETS.integrity("init.js").expect("init.js is bundled");
/// assert!(sri.starts_with("sha384-"));
/// ```
///
/// Once the plugin is installed, templates can also resolve the hashed URL
/// with `autumn_web::assets::asset_url("_plugins/motion/init.js")`.
pub static MOTION_ASSETS: PluginAssets = PluginAssets::from_files(
    ASSETS_NAMESPACE,
    &[
        (MOTION_JS, include_bytes!("../assets/motion.min.js")),
        (INIT_JS, include_bytes!("../assets/init.js")),
        (MOTION_CSS, include_bytes!("../assets/motion.css")),
    ],
);

/// Pinned Motion version vendored in `assets/motion.min.js`.
pub const MOTION_VERSION: &str = "12.43.0";

/// jsDelivr source URL of the vendored UMD build.
pub const MOTION_SOURCE: &str = "https://cdn.jsdelivr.net/npm/motion@12.43.0/dist/motion.js";

/// `sha384` Subresource Integrity hash of the upstream file vendored as
/// `assets/motion.min.js`.
///
/// This is a provenance pin, not something the `<script>` tag reads (the
/// bundle computes its own hashes): it records exactly which upstream bytes
/// were vendored, and a test fails if `assets/motion.min.js` drifts from it.
pub const MOTION_JS_INTEGRITY: &str =
    "sha384-hgVFh5YKDMdpcWEjpRqk2kPXOKBH7I9NBCKe3uR6dHKgy/BWPe8kyL01kiYuRn1u";

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    use sha2::{Digest as _, Sha384};

    /// Recomputes the `sha384` SRI of embedded bytes.
    fn sri(bytes: &[u8]) -> String {
        let digest = Sha384::digest(bytes);
        format!(
            "sha384-{}",
            base64::engine::general_purpose::STANDARD.encode(digest)
        )
    }

    #[test]
    fn bundle_holds_exactly_the_three_served_files() {
        let files: Vec<&str> = MOTION_ASSETS
            .iter()
            .map(autumn_web::assets::PluginAsset::logical_path)
            .collect();
        // Sorted by logical path; `manifest.json` is deliberately absent.
        assert_eq!(files, [INIT_JS, MOTION_CSS, MOTION_JS]);
        assert_eq!(MOTION_ASSETS.namespace(), ASSETS_NAMESPACE);
        assert_eq!(MOTION_ASSETS.mount_path(), "/static/_plugins/motion");
    }

    #[test]
    fn bundle_integrity_matches_embedded_bytes() {
        for asset in MOTION_ASSETS.iter() {
            assert_eq!(
                asset.integrity(),
                sri(asset.bytes()),
                "{} SRI is computed from its bytes",
                asset.logical_path()
            );
        }
    }

    #[test]
    fn vendored_motion_matches_the_pinned_upstream_hash() {
        let motion = MOTION_ASSETS
            .get(MOTION_JS)
            .expect("motion.min.js is bundled");
        assert_eq!(sri(motion.bytes()), MOTION_JS_INTEGRITY);
        assert_eq!(motion.integrity(), MOTION_JS_INTEGRITY);
    }

    #[test]
    fn urls_are_fingerprinted_under_the_plugin_mount() {
        for asset in MOTION_ASSETS.iter() {
            let path = asset.logical_path();
            assert_eq!(asset.plain_url(), format!("/static/_plugins/motion/{path}"));
            let (stem, ext) = path
                .rsplit_once('.')
                .expect("bundled files have extensions");
            let url = asset.url();
            let hash = url
                .strip_prefix(&format!("/static/_plugins/motion/{stem}."))
                .and_then(|rest| rest.strip_suffix(&format!(".{ext}")))
                .unwrap_or_else(|| panic!("{url} is the fingerprinted form of {path}"));
            assert_eq!(hash.len(), 8, "{url}");
            assert!(hash.bytes().all(|b| b.is_ascii_hexdigit()), "{url}");
        }
    }

    #[test]
    fn content_types_match_the_files() {
        let content_type = |path| {
            MOTION_ASSETS
                .get(path)
                .expect("file is bundled")
                .content_type()
        };
        assert_eq!(content_type(MOTION_JS), "text/javascript; charset=utf-8");
        assert_eq!(content_type(INIT_JS), "text/javascript; charset=utf-8");
        assert_eq!(content_type(MOTION_CSS), "text/css; charset=utf-8");
    }

    #[test]
    fn manifest_agrees_with_constants() {
        let manifest = include_str!("../assets/manifest.json");
        assert!(
            manifest.contains(MOTION_VERSION),
            "manifest records the pinned version"
        );
        assert!(
            manifest.contains(MOTION_SOURCE),
            "manifest records the source URL"
        );
        assert!(
            manifest.contains(MOTION_JS_INTEGRITY),
            "manifest records the motion.min.js integrity"
        );
    }
}
