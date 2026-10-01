//! Motion demo: a tiny Autumn app showing off `autumn-plugin-motion`.
//!
//! Run it, then open the page in a browser:
//!
//! ```sh
//! cargo run --example motion_demo
//! ```
//!
//! Then visit <http://127.0.0.1:3000> and:
//!
//! - watch the hero cascade in on load (staggered `fade-up`),
//! - scroll down to see the cards reveal as they enter the viewport,
//! - smash the "Load more" button — each htmx batch staggers in with no
//!   extra JavaScript, thanks to the plugin's `htmx:afterSwap` re-scan,
//! - watch the thin progress bar at the top track page scroll (driven by
//!   `window.Motion`'s `scroll()`, the plugin's escape hatch for bespoke
//!   choreography).
//!
//! No database, no auth — just Maud, htmx, and Motion. All JS and CSS live
//! in `static/` (embedded with `embed_static!()` and referenced via
//! `asset_url`): Autumn's default CSP blocks inline `<script>` and inline
//! `<style>` breaks under nonce mode, so nothing is inlined. htmx itself
//! comes from the framework's built-in handler — no CDN, no vendoring.

use std::sync::atomic::{AtomicU32, Ordering};

use autumn_plugin_motion::{Ease, Motion, MotionPlugin, motion_script};
use autumn_web::assets::asset_url;
use autumn_web::{Markup, html};

/// The crate's `static/` dir, embedded in the binary. `$CARGO_MANIFEST_DIR`
/// is the crate root even for examples.
static STATIC: autumn_web::include_dir::Dir = autumn_web::embed_static!();

/// Batch counter for the `/more` partial, so every htmx batch gets its own
/// playful headline.
static BATCH: AtomicU32 = AtomicU32::new(1);

#[autumn_web::main]
async fn main() {
    autumn_web::app()
        .plugin(MotionPlugin::new())
        .embedded_static(&STATIC)
        .routes(autumn_web::routes![index, more])
        .run()
        .await;
}

/// The page shell: scripts, progress bar, and content.
fn layout(content: &Markup) -> Markup {
    html! {
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Motion demo" }
                link rel="stylesheet" href=(asset_url("css/demo.css"));
                // The plugin's vendored Motion build + declarative init
                // script (both defer + SRI-hashed).
                (motion_script())
                // htmx from the framework's built-in handler — no CDN.
                script src=(asset_url("js/htmx.min.js")) defer {}
                // Bespoke choreography via the plugin's escape hatch:
                // `window.Motion` is the vendored global. Deferred, so it
                // runs after the plugin scripts above (document order).
                script src=(asset_url("js/demo.js")) defer {}
            }
            body {
                div class="progress" {}
                div class="wrap" { (content) }
            }
        }
    }
}

#[autumn_web::get("/")]
async fn index() -> Markup {
    layout(&html! {
        // 1. Entrance: the hero cascades in on load, 130ms apart, on a
        //    spring — Phase 1 typed easing, straight from Rust.
        section class="hero" {
            (Motion::fade_up().stagger(130).ease(Ease::Spring {
                stiffness: 260.0, damping: 22.0, mass: 1.0,
            }).wrap(html! {
                div class="kicker" { "autumn-plugin-motion" }
                h1 { "Server-rendered HTML," br; "now with choreography." }
                p {
                    "This whole page is Maud + htmx. Every animation below \
                     is declared in Rust — no hand-written animation code \
                     except one tiny progress-bar script."
                }
            }))
        }
        // 2. Scroll reveals: each card animates as it enters the viewport.
        section class="cards" {
            div class="card" data-motion="fade-up" {
                h2 { "Declarative" }
                p { "data-motion attributes describe the animation; the plugin's init script does the rest." }
            }
            div class="card" data-motion="slide-left" data-motion-delay="80" {
                h2 { "Typed in Rust" }
                p { "The Motion builder mirrors every attribute, so refactors stay compiler-checked." }
            }
            div class="card" data-motion="slide-right" data-motion-delay="80" {
                h2 { "Scroll-aware" }
                p { "Elements animate when they scroll into view — below the fold waits its turn." }
            }
            div class="card" data-motion="scale" data-motion-delay="80" data-motion-ease="back-out" {
                h2 { "Kind by default" }
                p { "prefers-reduced-motion disables everything; content stays fully visible." }
            }
        }
        // 3. htmx swap: each batch animates in via the afterSwap re-scan.
        section class="htmx-zone" {
            (Motion::fade_up().wrap(html! {
                h2 { "The htmx party trick" }
                p {
                    "Hit the button. The server returns plain HTML; the \
                     plugin re-scans the swapped content and each batch \
                     staggers in. Zero animation JavaScript on your part."
                }
                button hx-get="/more" hx-target="#more-items"
                    hx-swap="beforeend" {
                    "Load more"
                }
            }))
            div id="more-items" {}
        }
        footer { "fin. — now go animate something" }
    })
}

/// htmx partial: one more batch of items. The stagger container animates
/// its direct children when the swap lands, via the init script's
/// `htmx:afterSwap` hook.
#[autumn_web::get("/more")]
async fn more() -> Markup {
    let n = BATCH.fetch_add(1, Ordering::Relaxed);
    let blurbs = [
        "A fresh batch, straight from the server.",
        "Rendered by Rust, choreographed by Motion.",
        "No JavaScript was written for this cascade.",
        "The re-scan hook did all of it.",
    ];
    Motion::fade_up().stagger(90).wrap(html! {
        @for (i, blurb) in blurbs.iter().enumerate() {
            div class="item" {
                b { "Batch " (n) " · item " (i + 1) }
                p { (blurb) }
            }
        }
    })
}
