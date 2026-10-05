# autumn-plugin-motion

Motion ([motion.dev](https://motion.dev), vanilla JS, MIT) animations for
Autumn apps, with Maud + htmx ergonomics. No npm, no bundler, no React: the
plugin vendors the Motion UMD build into the crate and serves it from
memory, and gives you a typed Rust builder plus a declarative
`data-motion` attribute system.

## Quickstart

Add the plugin:

```rust
use autumn_plugin_motion::MotionPlugin;

autumn_web::app()
    .plugin(MotionPlugin::new())
    .run()
    .await;
```

Put the scripts in your layout (both are `defer`red and SRI-hashed):

```rust
use autumn_plugin_motion::motion_script;
use autumn_web::{Markup, html};

fn layout(content: Markup) -> Markup {
    html! {
        html {
            head { (motion_script()) }
            body { (content) }
        }
    }
}
```

Animate with the typed builder — it wraps your markup in an animated
`<div>`, no attribute splicing:

```rust
use autumn_plugin_motion::Motion;

let page = layout(html! {
    (Motion::fade_up().delay(100).wrap(html! {
        h1 { "Hello" }
    }))
});
```

Or write the attributes directly on your own elements:

```rust
let list = html! {
    ul data-motion="fade-up" data-motion-stagger="60" {
        li { "one" }
        li { "two" }
        li { "three" }
    }
};
```

## htmx

This is where it shines. The init script re-scans on `htmx:afterSwap`,
scoped to the swap target, so every htmx partial animates declaratively
with zero extra JS:

```rust
#[get("/feed")]
async fn feed() -> Markup {
    html! {
        div id="feed" {
            // Each row fades up, staggered — including rows added later
            // by an htmx swap into #feed.
            (Motion::fade_up().stagger(50).wrap(rows()))
        }
    }
}
```

```html
<button hx-get="/feed" hx-target="#feed" hx-swap="afterbegin">
    Load more
</button>
```

Scroll-linked effects work the same way:

```rust
(Motion::fade().scroll().wrap(html! {
    div class="scrub-me" { }
}))
```

Target another element's traversal, or remap the offsets:

```rust
// Fade as #hero crosses the viewport; finish when the element's center
// hits the viewport's center.
(Motion::fade()
    .scroll_target("#hero")
    .scroll_offset(["start end", "center center"])
    .wrap(markup))
```

Parallax drift and the opinionated progress bar:

```rust
use autumn_plugin_motion::{Motion, motion_stylesheet};

// In <head>, once per page:
(motion_stylesheet())

// Hero art that lags the scroll:
(Motion::parallax(0.3).wrap(markup))

// A fixed top bar that fills with page scroll — zero hand-written JS:
(Motion::scroll_progress())
```

## Demo

`examples/motion_demo.rs` is a tiny runnable app showing all of it:

```sh
cargo run --example motion_demo
# then open http://127.0.0.1:3000
```

The page demonstrates a staggered hero entrance with parallax drift,
scroll-reveal cards (one scrubbed by the scroll position itself), an
htmx "Load more" button whose server-rendered batches stagger in via the
`htmx:afterSwap` re-scan hook, and a scroll-driven progress bar
(`Motion::scroll_progress()` — one line, zero hand-written JS).

## Gotchas

Learned the hard way while building the demo — the rules for JS/CSS in
Autumn apps:

- **Never inline `<script>`.** The default CSP is `script-src 'self'` (no
  `unsafe-inline`), so inline scripts are blocked. Put JS in external
  files under the app's `static/` dir and reference them with
  `asset_url("js/app.js")`.
- **Never inline `<style>` either.** In nonce mode
  (`security.headers.csp_nonce`) `style-src` becomes `'self'` + nonce
  only, so inline styles break. Same answer: external stylesheet via
  `asset_url("css/app.css")`.
- **htmx ships with the framework.** Don't pull it from a CDN:
  `asset_url("js/htmx.min.js")` resolves to Autumn's built-in embedded
  htmx handler — no vendoring needed (it's skipped automatically if you
  pin your own copy with `autumn assets add htmx@…`).
- **Embed the app's `static/` dir** with `autumn_web::embed_static!()`
  and hand it to `AppBuilder::embedded_static(&STATIC)`; `$CARGO_MANIFEST_DIR`
  is the crate root even for examples, so this works from `examples/` too.

## Attribute reference

| Attribute | Values | Default | Notes |
|---|---|---|---|
| `data-motion` | `fade-up`, `fade-down`, `fade-left`, `fade-right`, `fade`, `scale`, `zoom-in`, `zoom-out`, `slide-left`, `slide-right`, `slide-up`, `slide-down`, `rotate-in`, `blur-in` | `fade-up` | The preset. Unknown values fall back to `fade-up`. `parallax` and `scroll-progress` are special scroll-driven kinds (see below), not entrance presets. |
| `data-motion-delay` | milliseconds, e.g. `150` | `0` | Start delay. |
| `data-motion-duration` | seconds, e.g. `1.2` | `0.6` | Animation duration. |
| `data-motion-once` | `false` | _(once)_ | Set to `false` to re-animate every time the element enters the viewport. |
| `data-motion-stagger` | milliseconds, e.g. `60` | — | On a container: cascade the preset over its direct children. |
| `data-motion-stagger-from` | `first`, `last`, `center`, `edges`, or a child index, e.g. `2` | `first` | Where the staggered cascade starts. Only applies with `data-motion-stagger`. |
| `data-motion-stagger-ease` | same syntax as `data-motion-ease` | — | Ease the stagger distribution across children. |
| `data-motion-hover` | e.g. `scale(1.05) brightness(1.1)` | — | Grow and/or brighten while hovered; eases back on leave. |
| `data-motion-press` | e.g. `scale(0.95)` | — | Shrink while pressed; releases on pointer up. |
| `data-motion-reduced` | `animate` | — | Opt back into animation when the user prefers reduced motion. Default: skip animation, content stays fully visible. |
| `data-motion-scroll` | _(bare attribute)_ | — | Drive the keyframes by scroll progress instead of animating on entry. |
| `data-motion-scroll-target` | CSS selector, e.g. `#hero` | _(the element itself)_ | Drive the scroll-linked keyframes by another element's traversal. Implies `data-motion-scroll`. |
| `data-motion-scroll-offset` | comma-separated edges, e.g. `start end,center center` | `start end,end start` | Remap the scroll-linked keyframes onto custom viewport edges. Invalid entries are ignored. |
| `data-motion-parallax` | factor, e.g. `0.3` | `0.3` | With `data-motion="parallax"`: scroll-linked drift of `factor` × the element's own height. Positive lags the scroll; negative opposes it. |
| `data-motion-ease` | `linear`, `ease-in`, `ease-out`, `ease-in-out`, `circ-in`, `circ-out`, `circ-in-out`, `back-in`, `back-out`, `back-in-out`, `anticipate`, `cubic-bezier(0.16,1,0.3,1)`, `spring(300,20,1)` | `cubic-bezier(0.16,1,0.3,1)` | Easing curve. `spring(stiffness,damping,mass)` uses spring physics. |
| `data-motion-repeat` | times, e.g. `2` | — | Repeat the animation `n` times after the first play. |
| `data-motion-repeat-type` | `loop`, `reverse`, `mirror` | `loop` | How each repeat cycle restarts. |
| `data-motion-amount` | `some`, `all`, or a fraction `0`–`1`, e.g. `0.5` | `some` | How much of the element must be visible before the `inView` trigger fires. |
| `data-motion-margin` | CSS margin syntax in `px`/`%`, e.g. `-100px`, `80px` | — | Grow/shrink the viewport for the `inView` trigger. Negative waits until the element is further inside; positive fires early. Invalid values are ignored. |

The Rust [`Motion`](https://docs.rs/autumn-plugin-motion) builder mirrors
all of these: `Motion::fade_up()`, `fade_down()`, `fade_left()`,
`fade_right()`, `fade()`, `scale()`, `zoom_in()`, `zoom_out()`,
`slide_left()`, `slide_right()`, `slide_up()`, `slide_down()`,
`rotate_in()`, `blur_in()`, plus `.delay(ms)`, `.duration(secs)`,
`.ease(Ease::Out)`, `.repeat(n)`, `.repeat_type(RepeatType::Mirror)`,
`.amount(0.5)` (or `.amount(InViewAmount::All)`), `.margin("-100px")`,
`.scroll_target("#hero")`, `.scroll_offset(["start end", "center center"])`,
`.stagger_from(StaggerFrom::Center)`, `.stagger_ease(Ease::Out)`,
`.hover_scale(1.05)`, `.hover_brightness(1.1)`, `.press_scale(0.97)`,
`.once(bool)`, `.stagger(ms)`, `.scroll()`, and `.wrap(markup)`.
`Motion::parallax(0.3)` builds a scroll-drift wrapper and
`Motion::scroll_progress()` renders the opinionated progress-bar element
(pair it with `motion_stylesheet()` in `<head>`). The
[`Ease`] enum covers named easings, `Ease::CubicBezier(x1, y1, x2, y2)`,
and `Ease::Spring { stiffness, damping, mass }`:

```rust
use autumn_plugin_motion::{Ease, Motion};

let bouncy = Motion::scale()
    .ease(Ease::Spring { stiffness: 300.0, damping: 20.0, mass: 1.0 })
    .wrap(markup);
```

Details:

- Elements animate when they scroll into view (`inView`), not on page
  load — below-the-fold content waits its turn.
- The scanner marks handled elements with `data-motion-init`, so htmx
  re-scans never double-animate.
- `prefers-reduced-motion: reduce` disables all animations; content stays
  fully visible. A single element can opt back in with
  `data-motion-reduced="animate"`. If Motion fails to load, the page is
  likewise untouched — animations are progressive enhancement, never a
  dependency.
- Stagger containers animate their *direct children*; don't put
  `data-motion` on the children too.

## How it works

- `assets/motion.min.js` — the upstream minified Motion UMD build
  (12.43.0, from jsDelivr), renamed from `dist/motion.js`. Sets
  `window.Motion`.
- `assets/init.js` — the plugin-authored scanner. Reads the attributes
  above, calls `Motion.animate` / `inView` / `scroll` / `stagger` /
  `hover` / `press`.
- `assets/motion.css` — default styles for `Motion::scroll_progress()`.
- `assets/manifest.json` — vendoring provenance: Motion version pin,
  source URL, and the `sha384` of the upstream file. Not served.
- The three served files form one `PluginAssets` bundle
  (`MOTION_ASSETS`), which `MotionPlugin` installs with Autumn's
  `AppBuilder::plugin_assets` seam (autumn-web 0.8+). The framework
  serves each file under `/static/_plugins/motion/`:
  - at a content-hashed URL (`init.<sha256-prefix>.js`) with
    `Cache-Control: public, max-age=31536000, immutable`, and
  - at its plain URL (`init.js`) with `must-revalidate`,
  - both with `ETag` / `304` and `Range` support.
- `motion_script()` / `motion_stylesheet()` emit the hashed URLs with
  `integrity=` hashes the framework computes from the embedded bytes —
  no hand-kept SRI constants. The routes show up in `autumn routes` as
  public, plugin-attributed static files.
- Need a URL yourself? `MOTION_ASSETS.url("init.js")`, or
  `asset_url("_plugins/motion/init.js")` once the plugin is installed.

## Upgrading from 0.1

0.2 requires autumn-web 0.8 and moves the assets onto the framework's
plugin-asset seam:

- Asset URLs moved from `/__motion/*` to `/static/_plugins/motion/*`
  (content-hashed). If you only use `motion_script()` /
  `motion_stylesheet()`, nothing changes for you. Hand-written
  `/__motion/...` references must switch to those helpers,
  `MOTION_ASSETS.url(..)`, or `asset_url("_plugins/motion/..")`.
- `motion_routes()` is gone — `MotionPlugin` installs `MOTION_ASSETS`.
- `INIT_JS_INTEGRITY` and `MOTION_CSS_INTEGRITY` are gone — use
  `MOTION_ASSETS.integrity("init.js")` / `("motion.css")`.
  `MOTION_JS_INTEGRITY` stays as the upstream provenance pin.
- The plugin no longer turns on autumn-web's `embed-assets` feature;
  enable it in your app if you rely on it.

## Limits

- **Motion+ is not included.** The paid motion.dev tier (`splitText`,
  `Cursor`, `Carousel`, `Ticker`, `AnimateNumber`, ScrambleText,
  Typewriter, …) needs a private-registry token and is deliberately out
  of scope — this plugin ships the MIT core only.
- **The Motion version is pinned per plugin release** (see
  `assets/manifest.json`). Upgrading Motion means upgrading the plugin;
  there is no `autumn assets`-style version switch.
- **Declarative enter/scroll effects only.** No layout animations, no
  shared-element transitions, no gesture handlers in the attribute
  layer — for bespoke choreography, call `window.Motion` directly; the
  global is there.
- **No live-browser animation verification.** The init script's logic is
  covered by content/string tests only (hook presence, preset table,
  reduced-motion guard). Motion's own test suite covers the animation
  engine; this plugin covers the wiring.
