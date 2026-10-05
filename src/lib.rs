//! Motion animations for Autumn, with Maud + htmx ergonomics.
//!
//! Add [`MotionPlugin`] to the app, put [`motion_script()`] in the page, and
//! animate declaratively:
//!
//! ```rust,no_run
//! use autumn_plugin_motion::{Motion, MotionPlugin, motion_script};
//! use autumn_web::prelude::*;
//!
//! #[get("/")]
//! async fn index() -> Markup {
//!     html! {
//!         html {
//!             head { (motion_script()) }
//!             body {
//!                 // Typed builder: wraps the markup in an animated div.
//!                 (Motion::fade_up().delay(100).wrap(html! {
//!                     h1 { "Hello" }
//!                 }))
//!                 // Or raw attributes on your own elements:
//!                 p data-motion="fade" data-motion-delay="200" { "World" }
//!             }
//!         }
//!     }
//! }
//!
//! # async fn run() {
//! autumn_web::app()
//!     .plugin(MotionPlugin::new())
//!     .routes(routes![index])
//!     .run()
//!     .await;
//! # }
//! ```
//!
//! The plugin vendors [Motion](https://motion.dev) 12.x (MIT) — no npm, no
//! bundler — and installs it as an Autumn plugin asset bundle
//! ([`MOTION_ASSETS`]), served from memory under `/static/_plugins/motion/`
//! at content-hashed, immutably cached URLs with SRI hashes computed from the
//! embedded bytes. The init script
//! scans `[data-motion]` on load and re-scans on `htmx:afterSwap`, so every
//! htmx partial animates declaratively too. `prefers-reduced-motion` is
//! respected: no animations, content stays visible.
//!
//! # Limits
//!
//! - Motion+ (the paid motion.dev tier: `splitText`, `Cursor`, `Carousel`,
//!   `Ticker`, `AnimateNumber`, …) is not included — only the MIT core.
//! - The Motion version is pinned per plugin release (see
//!   [`assets::MOTION_VERSION`]); upgrading Motion means upgrading the plugin.
//! - Animations are enter/scroll effects on server-rendered HTML. There is no
//!   layout-animation / shared-element support in the declarative layer.
//! - Without JavaScript (or with reduced motion) everything renders
//!   statically — by design.

mod assets;
mod motion;
mod plugin;
mod script;

pub use assets::{
    ASSETS_NAMESPACE, MOTION_ASSETS, MOTION_JS_INTEGRITY, MOTION_SOURCE, MOTION_VERSION,
};
pub use motion::{Ease, InViewAmount, Motion, Preset, RepeatType, StaggerFrom};
pub use plugin::{MotionPlugin, PLUGIN_NAME};
pub use script::{motion_script, motion_stylesheet};
