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
//! No database, no auth — just Maud, htmx, and Motion.

use std::sync::atomic::{AtomicU32, Ordering};

use autumn_plugin_motion::{Motion, MotionPlugin, motion_script};
use autumn_web::{Markup, html};

/// Batch counter for the `/more` partial, so every htmx batch gets its own
/// playful headline.
static BATCH: AtomicU32 = AtomicU32::new(1);

#[autumn_web::main]
async fn main() {
    autumn_web::app()
        .plugin(MotionPlugin::new())
        .routes(autumn_web::routes![index, more])
        .run()
        .await;
}

/// Minimal page CSS. The interesting bits are the `data-motion` attributes;
/// the styles just give the animations room to breathe.
const CSS: &str = r"
    * { box-sizing: border-box; }
    body {
        margin: 0; font-family: system-ui, sans-serif; color: #f4f1ea;
        background: #14121a;
    }
    .progress {
        position: fixed; top: 0; left: 0; height: 4px; width: 100%;
        background: linear-gradient(90deg, #c084fc, #f472b6);
        transform: scaleX(0); transform-origin: left; z-index: 50;
    }
    .wrap { max-width: 720px; margin: 0 auto; padding: 0 24px; }
    .hero { min-height: 92vh; display: flex; flex-direction: column;
            justify-content: center; }
    .hero h1 { font-size: 3.2rem; margin: 0 0 12px; }
    .hero p { font-size: 1.25rem; color: #b9b3c7; max-width: 34rem; }
    .kicker { text-transform: uppercase; letter-spacing: .2em; font-size: .8rem;
              color: #c084fc; margin-bottom: 16px; }
    .cards { display: grid; gap: 20px; padding: 80px 0; }
    .card { background: #1e1b28; border: 1px solid #352f47; border-radius: 14px;
            padding: 28px; }
    .card h2 { margin: 0 0 8px; font-size: 1.4rem; }
    .card p { margin: 0; color: #b9b3c7; }
    .htmx-zone { padding: 40px 0 120px; }
    .htmx-zone button {
        font-size: 1.1rem; padding: 12px 28px; border-radius: 999px; border: 0;
        background: #c084fc; color: #14121a; font-weight: 700; cursor: pointer;
    }
    .htmx-zone button:hover { background: #d8b4fe; }
    #more-items { display: grid; gap: 12px; margin-top: 24px; }
    .item { background: #1e1b28; border: 1px solid #352f47; border-radius: 10px;
            padding: 16px 20px; }
    .item b { color: #f472b6; }
    footer { padding: 60px 0; color: #6f6a80; text-align: center; }
";

/// The page shell: scripts, progress bar, and content.
fn layout(content: &Markup) -> Markup {
    html! {
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Motion demo" }
                style { (CSS) }
                // The plugin's vendored Motion build + declarative init
                // script (both defer + SRI-hashed).
                (motion_script())
                // htmx from a CDN — the demo's only external dependency.
                script src="https://cdn.jsdelivr.net/npm/htmx.org@2/dist/htmx.min.js"
                    defer {}
                // Bespoke choreography via the plugin's escape hatch:
                // `window.Motion` is the vendored global. Deferred, so it
                // runs after the plugin scripts above (document order).
                script defer {
                    (maud::PreEscaped(r"
                    (function () {
                        var M = window.Motion;
                        if (!M) return;
                        var bar = document.querySelector('.progress');
                        if (!bar) return;
                        // Page-scroll-driven progress: scaleX 0 -> 1.
                        M.scroll(M.animate(bar, { scaleX: [0, 1] }, { easing: 'linear' }));
                    })();"))
                }
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
        // 1. Entrance: the hero cascades in on load, 130ms apart.
        section class="hero" {
            (Motion::fade_up().stagger(130).wrap(html! {
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
            div class="card" data-motion="scale" data-motion-delay="80" {
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
