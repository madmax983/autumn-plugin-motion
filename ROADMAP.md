# Motion plugin API roadmap

Goal: a **typed, declarative Rust API** covering Motion 12's vanilla-JS surface
(`motion@12.43.0`, vendored UMD at `assets/motion.min.js`), so Autumn apps get
the whole library without writing JavaScript.

Design rules (carry these through every phase):

- Every helper is a typed Rust builder method or enum — no raw strings for
  things the compiler can check.
- Every helper maps to `data-motion-*` attributes; `assets/init.js` is the
  single interpreter. Rust and JS stay in lockstep (same preset names,
  same attribute names).
- Declarative first. Imperative escape hatches (`animate()` on any selector)
  come later, if at all — Autumn renders HTML, so the attribute model wins.

## Where we are (v0.1.0)

Presets: `fade_up`, `fade`, `scale`, `slide_left`, `slide_right`.
Modifiers: `delay`, `duration`, `once`, `stagger`, `scroll`, `wrap`.
Attributes: `data-motion`, `data-motion-delay`, `data-motion-duration`,
`data-motion-once`, `data-motion-stagger`, `data-motion-scroll`.

## Autumn 0.8 / plugin 0.2.0 ✅ done 2026-10-05

autumn-web 0.8.0 shipped the plugin asset seam (`PluginAssets` +
`AppBuilder::plugin_assets`), closing the "no plugin asset-overlay seam"
gap that forced the hand-rolled `/__motion/*` router.

- `MOTION_ASSETS` (`PluginAssets::from_files("motion", …)`) bundles
  `motion.min.js`, `init.js`, `motion.css`; `MotionPlugin::build` is now
  `app.plugin_assets(&MOTION_ASSETS)`.
- URLs moved to `/static/_plugins/motion/<name>.<hash>.<ext>` (immutable,
  ETag/304, Range); `motion_script()`/`motion_stylesheet()` use the
  bundle's tag helpers, so SRI is computed, not hand-kept.
- Removed: `motion_routes()`, `INIT_JS_INTEGRITY`, `MOTION_CSS_INTEGRITY`.
  `MOTION_JS_INTEGRITY` remains as the upstream provenance pin.
- Dropped the forced `embed-assets` feature on autumn-web.
- Decision record: `docs/adr/0001-plugin-assets-seam.md`.

## The upstream surface (dist/motion.js exports, 283 total)

Public vanilla API worth covering, grouped:

| Group | Exports | Status |
|---|---|---|
| Core animate | `animate` | partial (presets only) |
| Viewport trigger | `inView` | partial (once only) |
| Scroll-linked | `scroll`, `scrollInfo` | partial (progress-bar demo is hand JS) |
| Stagger | `stagger` | partial (fixed ms; no `from`/`ease`) |
| Springs | `spring`, `inertia` | not covered |
| Easings | `easeIn/Out/InOut`, `circIn/Out/InOut`, `backIn/Out/InOut`, `anticipate`, `cubicBezier`, `mirrorEasing`, `reverseEasing`, `steps` | not covered |
| Gestures | `hover`, `press` | not covered |
| Values | `motionValue`, `interpolate`, `transform`, `pipe`, `clamp`, `mix`, `wrap`, `progress` | not covered (JS-side) |
| Frame loop | `frame`, `cancelFrame` | not covered (JS-side) |
| Observers | `resize` | not covered |
| A11y | `prefersReducedMotion` | not covered |

Out of scope (React/Vue-only or internals): `useScroll`, `useSpring`,
`useMotionValue`, `useTransform`, `useInView`, `useAnimationFrame`,
`animateMini` internals, projection/layout internals, `animateView`.

## Phases

### Phase 1 — Typed easing ✅ done 2026-10-01
The biggest expressiveness gap: every preset currently runs on Motion's
default easing.

- New `Ease` enum: `Linear`, `In`, `Out`, `InOut`,
  `CircIn`, `CircOut`, `CircInOut`, `BackIn`, `BackOut`, `BackInOut`,
  `Anticipate`, `CubicBezier(f32, f32, f32, f32)`,
  `Spring { stiffness, damping, mass }`.
- Builder: `.ease(Ease::CircOut)` → `data-motion-ease="circ-out"`,
  `data-motion-ease="cubic-bezier(0.16,1,0.3,1)"`,
  `data-motion-ease="spring(300,25,1)"`.
- `init.js`: map names to Motion easing definitions / transition objects.
- Sensible per-preset defaults (e.g. `fade_up` defaults to an
  expo-ish `CubicBezier(0.16, 1, 0.3, 1)`).

### Phase 2 — More presets ✅ done 2026-10-01
Directions and combos Motion keyframes express trivially:

- `fade_down`, `fade_left`, `fade_right` (complete the fade family)
- `slide_up`, `slide_down`
- `zoom_in`, `zoom_out` (scale + fade, larger range than `scale`)
- `rotate_in` (small rotate + fade), `blur_in` (blur + fade)
- `.repeat(n)` / `.mirror(true)` → `data-motion-repeat`,
  `data-motion-repeat-type` (loop/mirror) for pulsing badges etc.

### Phase 3 — Viewport options ✅ done 2026-10-01
`inView` takes `{ once, amount, margin }`; we only expose `once`.

- `.amount(0.5)` → `data-motion-amount` (fraction visible to trigger)
- `.margin("-100px")` → `data-motion-margin` (root margin shrink/grow)
- Keep `once` as-is.

Also fixed while here: the vendored `inView` takes `{ root, margin, amount }` — there is no `once` option, so the `{ once }` object init.js passed was silently ignored (same class of bug as the `easing`→`ease` fix) and `data-motion-once="false"` never re-triggered. Now the callback returns a cleanup when `once` is false, keeping the element observed so it re-animates on every entry; the dead `{ once }` object is gone.

### Phase 4 — Scroll-linked, typed ✅ done 2026-10-01
Today `data-motion-scroll` is boolean-ish and the demo's progress bar is
hand-written JS. Make scroll a first-class typed concept:

- `Motion::scroll_progress()` → opinionated progress-bar element
  (replaces the demo's hand JS).
- `.scroll_target("#hero")`, `.scroll_offset(["start end", "end start"])`
  → `data-motion-scroll-target`, `data-motion-scroll-offset`, wired to
  Motion's `scroll()` with `target`/`offset`.
- `.parallax(0.3)` → scroll-linked `y` transform helper for hero art.

Shipped as: `Motion::scroll_progress()` renders
`<div class="motion-progress" data-motion="scroll-progress">`, styled by the
new `assets/motion.css` (served at `/__motion/motion.css`, SRI-pinned,
opt-in via `motion_stylesheet()`); init.js drives `scaleX` 0→1 with page
scroll. `.scroll_target()`/`.scroll_offset()` imply `.scroll()`; offsets are
validated in init.js (invalid entries ignored). `.parallax(f)` drifts the
element `f` × its own height over its viewport traversal (positive lags the
scroll). Demo: hand `demo.js` progress script deleted, hero wrapped in
`parallax(-0.12)`, one card scrubbed via `scroll_offset`.

### Phase 5 — Stagger v2 ✅ done 2026-10-01
`stagger(duration, { start, from, ease })` — we only do flat ms.

- `.stagger_from(StaggerFrom::Center)` → `data-motion-stagger-from`
  (`first` | `last` | `center` | index).
- `.stagger_ease(Ease::Out)` → eased distribution across children.
- Stagger `start` delay folds into existing `delay`.

Shipped as: `StaggerFrom::{First, Last, Center, Edges, Index(u32)}`
(`edges` is a bonus beyond the roadmap — Motion supports it);
`.stagger_from()` / `.stagger_ease()` are `const fn`; init.js builds
`{ startDelay, from, ease }` for `M.stagger`, validating `from` and
reusing the easing parser for the distribution ease.

### Phase 6 — Gestures ✅ done 2026-10-01
Motion's `hover`/`press` are tiny and high-value for buttons/cards:

- `.hover_scale(1.05)`, `.hover_brightness(1.1)` → `data-motion-hover`
- `.press_scale(0.95)` → `data-motion-press`
- `init.js` wires `Motion.hover` / `Motion.press` on these elements.

Shipped as: `data-motion-hover="scale(1.05) brightness(1.1)"`
(space-separated function syntax, any subset) and
`data-motion-press="scale(0.95)"`; init.js parses them defensively
(positive finite numbers only) and animates in/out via the hover/press
start→release callback contract. Demo: first card lifts on hover and
dips on press.

### Phase 7 — Reduced motion ✅ done 2026-10-01
`prefersReducedMotion` exists upstream; we ignore it.

- `init.js` checks `matchMedia("(prefers-reduced-motion: reduce)")` and
  skips animation (show final state) unless
  `data-motion-reduced="animate"` opts back in.
- Zero Rust API needed beyond the opt-out attribute; document it.

Shipped as: the guard already existed (`initIn` bailed when the media
query matched); now it's per-element — reduced-motion users get static
content unless an element carries `data-motion-reduced="animate"`.
Unclaimed elements stay unmarked so later scans can still pick them up.
Documented in the module docs, README attribute table, and gotchas.

### Phase 8 — Escape hatches (only if phases 1–7 leave real gaps)
- `Motion::custom(keyframes_json)` — typed-ish raw keyframes for the
  5% presets can't express.
- `timeline!` macro — sequenced multi-element stories. Declarative
  timelines fight the attribute model; defer until someone asks.

## Sequencing

Phases 1–3 are independent and small — good first increments.
Phase 4 builds on 3 (offset/target are viewport concepts).
Phases 5–6 are independent. Phase 7 is a one-line-ish JS change, do it
with whichever phase touches `init.js` next. Phase 8 is demand-driven.
