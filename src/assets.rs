//! Vendored Motion assets, embedded at compile time.
//!
//! The crate vendors the Motion UMD build (`assets/motion.min.js`, pinned in
//! `assets/manifest.json`), the plugin-authored declarative scanner
//! (`assets/init.js`), and the default stylesheet (`assets/motion.css`).
//! All three are served from memory by [`crate::routes`].

use autumn_web::include_dir;
use autumn_web::include_dir::Dir;

/// The embedded `assets/` directory.
///
/// The `include_dir` module import stays under that name because the
/// `include_dir!` expansion references it, mirroring
/// `autumn_web::embed_static!`.
pub(crate) static ASSETS: Dir<'static> = include_dir::include_dir!("$CARGO_MANIFEST_DIR/assets");

/// Pinned Motion version vendored in `assets/motion.min.js`.
pub const MOTION_VERSION: &str = "12.43.0";

/// jsDelivr source URL of the vendored UMD build.
pub const MOTION_SOURCE: &str = "https://cdn.jsdelivr.net/npm/motion@12.43.0/dist/motion.js";

/// `sha384` Subresource Integrity hash of `assets/motion.min.js`.
pub const MOTION_JS_INTEGRITY: &str =
    "sha384-hgVFh5YKDMdpcWEjpRqk2kPXOKBH7I9NBCKe3uR6dHKgy/BWPe8kyL01kiYuRn1u";

/// `sha384` Subresource Integrity hash of `assets/init.js`.
///
/// If `init.js` changes, update this constant (and `assets/manifest.json`);
/// [`integrity_hashes_match_embedded_bytes`] fails otherwise.
pub const INIT_JS_INTEGRITY: &str =
    "sha384-dUWEfCOpBgSzbVwX8EAHPbpeWW0lscEnu4hCBuH1F3TM3xPSQXXZqyvPd25+UoC9";

/// `sha384` Subresource Integrity hash of `assets/motion.css`.
///
/// If `motion.css` changes, update this constant (and
/// `assets/manifest.json`); [`integrity_hashes_match_embedded_bytes`] fails
/// otherwise.
pub const MOTION_CSS_INTEGRITY: &str =
    "sha384-yfZM6OaZCXTqD3cFPs2vvMxMH8ZWzeSiY04VwJ79Kj9GIj2IX8CBht7/Avi0OMNr";

/// Raw bytes of a vendored asset, or `None` when the name is unknown.
pub(crate) fn file(name: &str) -> Option<&'static [u8]> {
    ASSETS.get_file(name).map(include_dir::File::contents)
}

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
    fn integrity_hashes_match_embedded_bytes() {
        let motion = file("motion.min.js").expect("motion.min.js is embedded");
        assert_eq!(sri(motion), MOTION_JS_INTEGRITY);
        let init = file("init.js").expect("init.js is embedded");
        assert_eq!(sri(init), INIT_JS_INTEGRITY);
        let css = file("motion.css").expect("motion.css is embedded");
        assert_eq!(sri(css), MOTION_CSS_INTEGRITY);
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
        assert!(
            manifest.contains(INIT_JS_INTEGRITY),
            "manifest records the init.js integrity"
        );
        assert!(
            manifest.contains(MOTION_CSS_INTEGRITY),
            "manifest records the motion.css integrity"
        );
    }
}
