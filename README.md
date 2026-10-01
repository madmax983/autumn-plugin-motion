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
    div class="progress-bar" { }
}))
```

## Attribute reference

| Attribute | Values | Default | Notes |
|---|---|---|---|
| `data-motion` | `fade-up`, `fade`, `scale`, `slide-left`, `slide-right` | `fade-up` | The preset. Unknown values fall back to `fade-up`. |
| `data-motion-delay` | milliseconds, e.g. `150` | `0` | Start delay. |
| `data-motion-duration` | seconds, e.g. `1.2` | `0.6` | Animation duration. |
| `data-motion-once` | `false` | _(once)_ | Set to `false` to re-animate every time the element enters the viewport. |
| `data-motion-stagger` | milliseconds, e.g. `60` | — | On a container: cascade the preset over its direct children. |
| `data-motion-scroll` | _(bare attribute)_ | — | Drive the keyframes by scroll progress instead of animating on entry. |

The Rust [`Motion`](https://docs.rs/autumn-plugin-motion) builder mirrors
all of these: `Motion::fade_up()`, `.fade()`, `.scale()`,
`.slide_left()`, `.slide_right()`, plus `.delay(ms)`,
`.duration(secs)`, `.once(bool)`, `.stagger(ms)`, `.scroll()`, and
`.wrap(markup)`.

Details:

- Elements animate when they scroll into view (`inView`), not on page
  load — below-the-fold content waits its turn.
- The scanner marks handled elements with `data-motion-init`, so htmx
  re-scans never double-animate.
- `prefers-reduced-motion: reduce` disables all animations; content stays
  fully visible. If Motion fails to load, the page is likewise untouched —
  animations are progressive enhancement, never a dependency.
- Stagger containers animate their *direct children*; don't put
  `data-motion` on the children too.

## How it works

- `assets/motion.min.js` — the upstream minified Motion UMD build
  (12.43.0, from jsDelivr), renamed from `dist/motion.js`. Sets
  `window.Motion`.
- `assets/init.js` — the plugin-authored scanner (~6 KiB). Reads the
  attributes above, calls `Motion.animate` / `inView` / `scroll` /
  `stagger`.
- `assets/manifest.json` — version pin, source URL, and `sha384`
  integrity hashes for both files.
- The plugin merges an axum router serving both at `/__motion/*` with
  `text/javascript` content type and a day of public caching. The
  `<script>` tags carry the SRI hashes.

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
- Plugin assets live under the `/__motion` namespace rather than the
  app's `/static/*` fingerprinting — Autumn has no plugin asset-overlay
  seam yet, so the plugin serves its own files.
