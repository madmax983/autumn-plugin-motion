//! [`MotionPlugin`]: installs the Motion animation assets in an Autumn app.
//!
//! The plugin merges the [`motion_routes`](crate::motion_routes) router,
//! serving the vendored Motion UMD build and the declarative init script
//! under `/__motion/*`. No configuration, no startup hooks.

use std::borrow::Cow;

use autumn_web::app::AppBuilder;
use autumn_web::plugin::Plugin;

use crate::routes::motion_routes;

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
        app.merge(motion_routes())
    }
}
