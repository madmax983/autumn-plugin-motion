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
//! - watch the hero cascade in on load (staggered `fade-up`), then scroll
//!   — the whole hero drifts against you (parallax),
//! - scroll down to see the cards reveal as they enter the viewport —
//!   the last one fires early via `data-motion-margin="80px"`, and one is
//!   scrubbed by the scroll position itself,
//! - smash the "Load more" button — each htmx batch staggers in with no
//!   extra JavaScript, thanks to the plugin's `htmx:afterSwap` re-scan,
//! - watch the thin progress bar at the top track page scroll
//!   (`Motion::scroll_progress()` — one line, zero hand-written JS).
//!
//! No database, no auth — just Maud, htmx, and Motion. All JS and CSS live
//! in `static/` (embedded with `embed_static!()` and referenced via
//! `asset_url`): Autumn's default CSP blocks inline `<script>` and inline
//! `<style>` breaks under nonce mode, so nothing is inlined. htmx itself
//! comes from the framework's built-in handler — no CDN, no vendoring.

use std::sync::atomic::{AtomicU32, Ordering};

use autumn_plugin_motion::{Ease, Motion, MotionPlugin, motion_script, motion_stylesheet};
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
                // script (both defer + SRI-hashed), plus the default
                // stylesheet for Motion::scroll_progress().
                (motion_script())
                (motion_stylesheet())
                // htmx from the framework's built-in handler — no CDN.
                script src=(asset_url("js/htmx.min.js")) defer {}
            }
            body {
                // Phase 4: the hand-written progress-bar script is gone —
                // this one line renders the bar and init.js drives it.
                (Motion::scroll_progress())
                div class="wrap" { (content) }
            }
        }
    }
}

#[autumn_web::get("/")]
async fn index() -> Markup {
    layout(&html! {
        // 1. Entrance: the hero cascades in on load, 130ms apart, on a
        //    spring — Phase 1 typed easing, straight from Rust. The whole
        //    hero also drifts against the scroll (Phase 4 parallax).
        (Motion::parallax(-0.12).wrap(html! {
            section class="hero" {
                (Motion::fade_up().stagger(130).ease(Ease::Spring {
                    stiffness: 260.0, damping: 22.0, mass: 1.0,
                }).wrap(html! {
                    div class="kicker" { "autumn-plugin-motion" }
                    h1 { "Server-rendered HTML," br; "now with choreography." }
                    p {
                        "This whole page is Maud + htmx. Every animation below \
                         is declared in Rust — no hand-written animation code."
                    }
                }))
            }
        }))
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
            // Phase 4: scroll-scrubbed — the fade is driven by the scroll
            // position itself, finishing as the card reaches mid-viewport.
            (Motion::fade()
                .scroll_offset(["start end", "center center"])
                .wrap(html! {
                    div class="card" {
                        h2 { "Scroll-scrubbed" }
                        p { "No trigger here — drag the page and watch this card fade with its own traversal." }
                    }
                }))
            div class="card" data-motion="zoom-in" data-motion-delay="80" data-motion-ease="back-out" data-motion-repeat="2" data-motion-repeat-type="mirror" data-motion-margin="80px" {
                h2 { "Kind by default" }
                p { "prefers-reduced-motion disables everything; content stays fully visible." }
            }
        }
        // 3. htmx swap: each batch animates in via the afterSwap re-scan.
        section class="htmx-zone" {
            (Motion::fade_up().amount(0.6).wrap(html! {
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
    Motion::slide_up().stagger(90).wrap(html! {
        @for (i, blurb) in blurbs.iter().enumerate() {
            div class="item" {
                b { "Batch " (n) " · item " (i + 1) }
                p { (blurb) }
            }
        }
    })
}
