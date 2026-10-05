//! The typed [`Motion`] animation builder.
//!
//! A [`Motion`] value describes one declarative animation: a preset plus
//! modifiers. It renders the `data-motion*` attributes that the plugin's
//! `init.js` scans for, so animations stay type-checked on the Rust side
//! while the behavior lives in the vendored Motion build.
//!
//! # Raw attribute reference
//!
//! The builder emits these attributes; you can also write them by hand on
//! your own elements:
//!
//! - `data-motion="fade-up|fade-down|fade-left|fade-right|fade|scale|zoom-in|zoom-out|slide-left|slide-right|slide-up|slide-down|rotate-in|blur-in"`
//!   — the preset (default `fade-up`).
//! - `data-motion-delay="150"` — start delay in milliseconds (default `0`).
//! - `data-motion-duration="0.6"` — duration in seconds (default `0.6`).
//! - `data-motion-once="false"` — re-animate every time the element enters
//!   the viewport (default is once).
//! - `data-motion-stagger="60"` — on a container: cascade the preset over
//!   its direct children, `60` ms apart.
//! - `data-motion-scroll` — bare attribute: drive the keyframes by scroll
//!   progress instead of animating on entry.
//! - `data-motion-ease="ease-out|cubic-bezier(0.16,1,0.3,1)|spring(300,20,1)"`
//!   — easing curve (default: a snappy `cubic-bezier(0.16,1,0.3,1)`).
//! - `data-motion-repeat="2"` — repeat the animation `n` times after the
//!   first play.
//! - `data-motion-repeat-type="loop|reverse|mirror"` — how each repeat
//!   cycle restarts (default `loop`).
//! - `data-motion-amount="some|all|0.5"` — how much of the element must be
//!   visible before the `inView` trigger fires (default `some`).
//! - `data-motion-margin="-100px"` — grow/shrink the viewport for the
//!   `inView` trigger, in CSS margin syntax (`px` or `%`, up to four
//!   values). Negative waits until the element is further inside;
//!   positive fires early.
//! - `data-motion="parallax"` + `data-motion-parallax="0.3"` — scroll-linked
//!   drift: the element travels `0.3` × its own height as it moves through
//!   the viewport (see [`Motion::parallax`]).
//! - `data-motion="scroll-progress"` — opinionated page-scroll progress bar
//!   (see [`Motion::scroll_progress`]).
//! - `data-motion-scroll-target="#hero"` — drive the scroll-linked keyframes
//!   by *another* element's traversal (a CSS selector; falls back to the
//!   element itself when missing). Implies `data-motion-scroll`.
//! - `data-motion-scroll-offset="start end,center center"` — comma-separated
//!   scroll offsets mapping target/container edges (default
//!   `"start end,end start"`). Invalid entries are ignored.
//! - `data-motion-reduced="animate"` — opt back into animation when the
//!   user prefers reduced motion (default: skip animation, content stays
//!   fully visible).
//! - `data-motion-stagger-from="first|last|center|edges|2"` — where a
//!   staggered cascade starts (default `first`). Only applies with
//!   `data-motion-stagger`.
//! - `data-motion-stagger-ease="ease-out"` — ease the stagger distribution
//!   across children (same value syntax as `data-motion-ease`).
//! - `data-motion-hover="scale(1.05) brightness(1.1)"` — grow and/or
//!   brighten on hover (any subset).
//! - `data-motion-press="scale(0.95)"` — shrink while pressed.

use autumn_web::{Markup, html};

/// Default animation duration in seconds, used when no duration is set.
const DEFAULT_DURATION_SECS: f32 = 0.6;

/// An easing curve for a [`Motion`] animation.
///
/// Serializes to the `data-motion-ease` attribute, which `assets/init.js`
/// maps onto Motion's easing definitions. When no ease is set, `init.js`
/// applies a snappy expo-out curve (`cubic-bezier(0.16, 1, 0.3, 1)`).
///
/// ```rust
/// use autumn_plugin_motion::{Ease, Motion};
/// use autumn_web::{Markup, html};
///
/// let card: Markup = Motion::scale()
///     .ease(Ease::Spring { stiffness: 300.0, damping: 20.0, mass: 1.0 })
///     .wrap(html! { p { "Boing" } });
/// assert!(card.into_string().contains(r#"data-motion-ease="spring(300,20,1)""#));
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ease {
    /// Constant speed.
    Linear,
    /// Start slow, end fast (`cubic-bezier(0.42, 0, 1, 1)`).
    In,
    /// Start fast, end slow (`cubic-bezier(0, 0, 0.58, 1)`).
    Out,
    /// Slow at both ends (`cubic-bezier(0.42, 0, 0.58, 1)`).
    InOut,
    /// Circular ease-in.
    CircIn,
    /// Circular ease-out.
    CircOut,
    /// Circular ease-in-out.
    CircInOut,
    /// Overshoots, then settles (ease-in flavor).
    BackIn,
    /// Overshoots, then settles (ease-out flavor).
    BackOut,
    /// Overshoots, then settles (in-out flavor).
    BackInOut,
    /// Starts by moving slightly backwards, then accelerates.
    Anticipate,
    /// A custom cubic-bezier curve: `(x1, y1, x2, y2)`.
    CubicBezier(f32, f32, f32, f32),
    /// Spring physics. `stiffness` is the spring strength, `damping` the
    /// friction, `mass` the weight being moved.
    Spring {
        /// Spring strength (higher = snappier).
        stiffness: f32,
        /// Friction (higher = less oscillation).
        damping: f32,
        /// Weight being moved.
        mass: f32,
    },
}

impl Ease {
    /// The `data-motion-ease` attribute value for this ease.
    fn attr_value(self) -> String {
        match self {
            Self::Linear => "linear".to_owned(),
            Self::In => "ease-in".to_owned(),
            Self::Out => "ease-out".to_owned(),
            Self::InOut => "ease-in-out".to_owned(),
            Self::CircIn => "circ-in".to_owned(),
            Self::CircOut => "circ-out".to_owned(),
            Self::CircInOut => "circ-in-out".to_owned(),
            Self::BackIn => "back-in".to_owned(),
            Self::BackOut => "back-out".to_owned(),
            Self::BackInOut => "back-in-out".to_owned(),
            Self::Anticipate => "anticipate".to_owned(),
            Self::CubicBezier(x1, y1, x2, y2) => {
                format!("cubic-bezier({x1},{y1},{x2},{y2})")
            }
            Self::Spring {
                stiffness,
                damping,
                mass,
            } => format!("spring({stiffness},{damping},{mass})"),
        }
    }
}

/// How a repeated [`Motion`] animation restarts each cycle.
///
/// Serializes to the `data-motion-repeat-type` attribute; mirrors Motion's
/// `repeatType` transition option.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatType {
    /// Jump back to the start each cycle (default).
    Loop,
    /// Play backwards on alternate cycles.
    Reverse,
    /// Alternate direction each cycle (ping-pong).
    Mirror,
}

impl RepeatType {
    /// The `data-motion-repeat-type` attribute value.
    const fn attr(self) -> &'static str {
        match self {
            Self::Loop => "loop",
            Self::Reverse => "reverse",
            Self::Mirror => "mirror",
        }
    }
}

/// How much of the element must be visible before its `inView` animation
/// triggers.
///
/// Serializes to the `data-motion-amount` attribute; mirrors Motion's
/// `inView` `amount` option (`"some" | "all" | number`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InViewAmount {
    /// Trigger when any part of the element enters the viewport
    /// (Motion's default).
    Some,
    /// Trigger only when the whole element is visible.
    All,
    /// Trigger when this fraction (0.0–1.0) of the element is visible.
    /// Values outside the range are clamped.
    Fraction(f32),
}

impl InViewAmount {
    /// The `data-motion-amount` attribute value.
    fn attr_value(self) -> String {
        match self {
            Self::Some => "some".to_string(),
            Self::All => "all".to_string(),
            Self::Fraction(v) => format!("{}", v.clamp(0.0, 1.0)),
        }
    }
}

impl From<f32> for InViewAmount {
    /// Lets `.amount(0.5)` mean `.amount(InViewAmount::Fraction(0.5))`.
    fn from(v: f32) -> Self {
        Self::Fraction(v)
    }
}

/// Where a staggered cascade starts.
///
/// Serializes to `data-motion-stagger-from`; mirrors Motion's `stagger`
/// `from` option (`"first" | "last" | "center" | "edges" | index`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaggerFrom {
    /// From the first child (default).
    First,
    /// From the last child.
    Last,
    /// From the middle outward.
    Center,
    /// From the edges inward.
    Edges,
    /// From the child at this index.
    Index(u32),
}

impl StaggerFrom {
    /// The `data-motion-stagger-from` attribute value.
    fn attr_value(self) -> String {
        match self {
            Self::First => "first".to_string(),
            Self::Last => "last".to_string(),
            Self::Center => "center".to_string(),
            Self::Edges => "edges".to_string(),
            Self::Index(i) => i.to_string(),
        }
    }
}

/// An animation preset, mirroring the keyframes in `assets/init.js`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    /// Fade in while rising 24px.
    FadeUp,
    /// Fade in while sinking 24px.
    FadeDown,
    /// Fade in while drifting 24px left.
    FadeLeft,
    /// Fade in while drifting 24px right.
    FadeRight,
    /// Fade in place.
    Fade,
    /// Fade in while scaling from 0.94.
    Scale,
    /// Fade in while zooming from 0.8 scale.
    ZoomIn,
    /// Fade in while settling from 1.15 scale.
    ZoomOut,
    /// Fade in while sliding 32px from the right.
    SlideLeft,
    /// Fade in while sliding 32px from the left.
    SlideRight,
    /// Fade in while rising 32px from below.
    SlideUp,
    /// Fade in while sinking 32px from above.
    SlideDown,
    /// Fade in while straightening from -8 degrees.
    RotateIn,
    /// Fade in while de-blurring from 8px.
    BlurIn,
    /// Scroll-linked drift (see [`Motion::parallax`]).
    Parallax,
}

impl Preset {
    /// The `data-motion` attribute value for this preset.
    const fn attr(self) -> &'static str {
        match self {
            Self::FadeUp => "fade-up",
            Self::FadeDown => "fade-down",
            Self::FadeLeft => "fade-left",
            Self::FadeRight => "fade-right",
            Self::Fade => "fade",
            Self::Scale => "scale",
            Self::ZoomIn => "zoom-in",
            Self::ZoomOut => "zoom-out",
            Self::SlideLeft => "slide-left",
            Self::SlideRight => "slide-right",
            Self::SlideUp => "slide-up",
            Self::SlideDown => "slide-down",
            Self::RotateIn => "rotate-in",
            Self::BlurIn => "blur-in",
            Self::Parallax => "parallax",
        }
    }
}

/// A declarative animation: preset plus modifiers.
///
/// Build with a preset constructor ([`Motion::fade_up`], [`Motion::fade`],
/// [`Motion::scale`], [`Motion::slide_left`], … — see [`Preset`]),
/// tune with the modifier methods, then either [`Motion::wrap`] the markup
/// or write the attributes by hand (see the module docs).
///
/// ```rust
/// use autumn_plugin_motion::Motion;
/// use autumn_web::{Markup, html};
///
/// let card: Markup = Motion::fade_up().delay(150).wrap(html! {
///     h2 { "Hello" }
/// });
/// assert!(card.into_string().contains(r#"data-motion="fade-up""#));
/// ```
#[derive(Debug, Clone)]
pub struct Motion {
    preset: Preset,
    delay_ms: u32,
    duration_secs: Option<f32>,
    ease: Option<Ease>,
    repeat: Option<u32>,
    repeat_type: Option<RepeatType>,
    amount: Option<InViewAmount>,
    margin: Option<String>,
    scroll_target: Option<String>,
    scroll_offset: Option<[String; 2]>,
    parallax_factor: Option<f32>,
    stagger_from: Option<StaggerFrom>,
    stagger_ease: Option<Ease>,
    hover_scale: Option<f32>,
    hover_brightness: Option<f32>,
    press_scale: Option<f32>,
    once: bool,
    stagger_ms: Option<u32>,
    scroll: bool,
}

impl Motion {
    /// Fade in while rising 24px.
    #[must_use]
    pub const fn fade_up() -> Self {
        Self::preset(Preset::FadeUp)
    }

    /// Fade in while sinking 24px.
    #[must_use]
    pub const fn fade_down() -> Self {
        Self::preset(Preset::FadeDown)
    }

    /// Fade in while drifting 24px left.
    #[must_use]
    pub const fn fade_left() -> Self {
        Self::preset(Preset::FadeLeft)
    }

    /// Fade in while drifting 24px right.
    #[must_use]
    pub const fn fade_right() -> Self {
        Self::preset(Preset::FadeRight)
    }

    /// Fade in place.
    #[must_use]
    pub const fn fade() -> Self {
        Self::preset(Preset::Fade)
    }

    /// Fade in while scaling from 0.94.
    #[must_use]
    pub const fn scale() -> Self {
        Self::preset(Preset::Scale)
    }

    /// Fade in while zooming from 0.8 scale.
    #[must_use]
    pub const fn zoom_in() -> Self {
        Self::preset(Preset::ZoomIn)
    }

    /// Fade in while settling from 1.15 scale.
    #[must_use]
    pub const fn zoom_out() -> Self {
        Self::preset(Preset::ZoomOut)
    }

    /// Fade in while sliding 32px from the right.
    #[must_use]
    pub const fn slide_left() -> Self {
        Self::preset(Preset::SlideLeft)
    }

    /// Fade in while sliding 32px from the left.
    #[must_use]
    pub const fn slide_right() -> Self {
        Self::preset(Preset::SlideRight)
    }

    /// Fade in while rising 32px from below.
    #[must_use]
    pub const fn slide_up() -> Self {
        Self::preset(Preset::SlideUp)
    }

    /// Fade in while sinking 32px from above.
    #[must_use]
    pub const fn slide_down() -> Self {
        Self::preset(Preset::SlideDown)
    }

    /// Fade in while straightening from -8 degrees.
    #[must_use]
    pub const fn rotate_in() -> Self {
        Self::preset(Preset::RotateIn)
    }

    /// Fade in while de-blurring from 8px.
    #[must_use]
    pub const fn blur_in() -> Self {
        Self::preset(Preset::BlurIn)
    }

    /// Scroll-linked drift for hero art and backgrounds.
    ///
    /// As the element travels through the viewport, it translates by
    /// `factor` × its own height: positive lags the scroll (drifts down as
    /// you scroll down), negative moves against it. `0.3` is a good
    /// default; keep `|factor|` modest — large values fling content away.
    ///
    /// Parallax is inherently scroll-linked and ignores the `scroll`,
    /// `scroll_target`, and `scroll_offset` modifiers.
    ///
    /// ```rust
    /// use autumn_plugin_motion::Motion;
    /// use autumn_web::{Markup, html};
    ///
    /// let html: Markup = Motion::parallax(0.3).wrap(html! { p { "Hi" } });
    /// let s = html.into_string();
    /// assert!(s.contains(r#"data-motion="parallax""#), "{s}");
    /// assert!(s.contains(r#"data-motion-parallax="0.3""#), "{s}");
    /// ```
    #[must_use]
    pub const fn parallax(factor: f32) -> Self {
        let mut motion = Self::preset(Preset::Parallax);
        motion.parallax_factor = Some(factor);
        motion
    }

    /// Opinionated page-scroll progress bar element.
    ///
    /// Renders a fixed top bar that fills left-to-right as the page
    /// scrolls — no hand-written JavaScript. Include the plugin stylesheet
    /// once per page for the default styling:
    ///
    /// ```rust
    /// use autumn_plugin_motion::{Motion, motion_stylesheet};
    /// use autumn_web::{Markup, html};
    ///
    /// let head: Markup = html! { (motion_stylesheet()) };
    /// assert!(head.into_string().contains("/static/_plugins/motion/motion."));
    /// let bar: Markup = Motion::scroll_progress();
    /// let s = bar.into_string();
    /// assert!(s.contains(r#"data-motion="scroll-progress""#), "{s}");
    /// assert!(s.contains(r#"class="motion-progress""#), "{s}");
    /// ```
    #[must_use]
    pub fn scroll_progress() -> Markup {
        html! {
            div class="motion-progress" data-motion="scroll-progress" {}
        }
    }

    /// Builds a [`Motion`] from a raw preset.
    const fn preset(preset: Preset) -> Self {
        Self {
            preset,
            delay_ms: 0,
            duration_secs: None,
            ease: None,
            repeat: None,
            repeat_type: None,
            amount: None,
            margin: None,
            scroll_target: None,
            scroll_offset: None,
            parallax_factor: None,
            stagger_from: None,
            stagger_ease: None,
            hover_scale: None,
            hover_brightness: None,
            press_scale: None,
            once: true,
            stagger_ms: None,
            scroll: false,
        }
    }

    /// Start delay in milliseconds.
    #[must_use]
    pub const fn delay(mut self, ms: u32) -> Self {
        self.delay_ms = ms;
        self
    }

    /// Duration in seconds (default 0.6).
    #[must_use]
    pub const fn duration(mut self, secs: f32) -> Self {
        self.duration_secs = Some(secs);
        self
    }

    /// Easing curve (default: a snappy `cubic-bezier(0.16,1,0.3,1)` applied
    /// by `init.js` when the attribute is absent).
    ///
    /// ```rust
    /// use autumn_plugin_motion::{Ease, Motion};
    /// use autumn_web::{Markup, html};
    ///
    /// let html: Markup = Motion::fade_up()
    ///     .ease(Ease::Out)
    ///     .wrap(html! { p { "Hi" } });
    /// assert!(html.into_string().contains(r#"data-motion-ease="ease-out""#));
    /// ```
    #[must_use]
    pub const fn ease(mut self, ease: Ease) -> Self {
        self.ease = Some(ease);
        self
    }

    /// Repeat the animation `n` times after the first play (so `repeat(2)`
    /// plays three times total). Useful for pulsing badges and attention
    /// loops.
    ///
    /// ```rust
    /// use autumn_plugin_motion::Motion;
    /// use autumn_web::{Markup, html};
    ///
    /// let html: Markup = Motion::scale().repeat(2).wrap(html! { p { "Hi" } });
    /// assert!(html.into_string().contains(r#"data-motion-repeat="2""#));
    /// ```
    #[must_use]
    pub const fn repeat(mut self, n: u32) -> Self {
        self.repeat = Some(n);
        self
    }

    /// How each repeat cycle restarts: [`RepeatType::Loop`] (default),
    /// [`RepeatType::Reverse`], or [`RepeatType::Mirror`] (ping-pong).
    #[must_use]
    pub const fn repeat_type(mut self, repeat_type: RepeatType) -> Self {
        self.repeat_type = Some(repeat_type);
        self
    }

    /// How much of the element must be visible before the `inView`
    /// animation triggers: [`InViewAmount::Some`] (default),
    /// [`InViewAmount::All`], or [`InViewAmount::Fraction`]. A bare `f32`
    /// works too, via `From`:
    ///
    /// ```rust
    /// use autumn_plugin_motion::{InViewAmount, Motion};
    /// use autumn_web::{Markup, html};
    ///
    /// let a: Markup = Motion::fade_up().amount(0.5).wrap(html! { p { "Hi" } });
    /// assert!(a.into_string().contains(r#"data-motion-amount="0.5""#));
    /// let b: Markup = Motion::fade_up()
    ///     .amount(InViewAmount::All)
    ///     .wrap(html! { p { "Hi" } });
    /// assert!(b.into_string().contains(r#"data-motion-amount="all""#));
    /// ```
    #[must_use]
    pub fn amount(mut self, amount: impl Into<InViewAmount>) -> Self {
        self.amount = Some(amount.into());
        self
    }

    /// Grow or shrink the viewport used for the `inView` trigger, in CSS
    /// margin syntax with `px` or `%` values — e.g. `"-100px"` waits until
    /// the element is 100px inside the viewport, `"80px"` fires early.
    /// Up to four space-separated values, like CSS `margin`.
    ///
    /// ```rust
    /// use autumn_plugin_motion::Motion;
    /// use autumn_web::{Markup, html};
    ///
    /// let html: Markup = Motion::fade_up().margin("-100px").wrap(html! { p { "Hi" } });
    /// assert!(html.into_string().contains(r#"data-motion-margin="-100px""#));
    /// ```
    #[must_use]
    pub fn margin(mut self, margin: impl Into<String>) -> Self {
        self.margin = Some(margin.into());
        self
    }

    /// When `false`, re-animate every time the element enters the viewport
    /// instead of only the first time.
    #[must_use]
    pub const fn once(mut self, once: bool) -> Self {
        self.once = once;
        self
    }

    /// Cascade the preset over the wrapped element's direct children, `ms`
    /// milliseconds apart.
    #[must_use]
    pub const fn stagger(mut self, ms: u32) -> Self {
        self.stagger_ms = Some(ms);
        self
    }

    /// Where the staggered cascade starts: [`StaggerFrom::First`] (default),
    /// [`StaggerFrom::Last`], [`StaggerFrom::Center`],
    /// [`StaggerFrom::Edges`], or [`StaggerFrom::Index`]. Only applies with
    /// [`.stagger(ms)`](Motion::stagger).
    ///
    /// ```rust
    /// use autumn_plugin_motion::{Motion, StaggerFrom};
    /// use autumn_web::{Markup, html};
    ///
    /// let html: Markup = Motion::fade_up()
    ///     .stagger(90)
    ///     .stagger_from(StaggerFrom::Center)
    ///     .wrap(html! { p { "Hi" } });
    /// assert!(
    ///     html.into_string().contains(r#"data-motion-stagger-from="center""#)
    /// );
    /// ```
    #[must_use]
    pub const fn stagger_from(mut self, from: StaggerFrom) -> Self {
        self.stagger_from = Some(from);
        self
    }

    /// Ease the stagger distribution across children — e.g. [`Ease::Out`]
    /// bunches later children together. Same value syntax as
    /// [`.ease()`](Motion::ease); only applies with
    /// [`.stagger(ms)`](Motion::stagger).
    #[must_use]
    pub const fn stagger_ease(mut self, ease: Ease) -> Self {
        self.stagger_ease = Some(ease);
        self
    }

    /// Drive the keyframes by scroll progress as the element travels through
    /// the viewport, instead of animating on entry.
    #[must_use]
    pub const fn scroll(mut self) -> Self {
        self.scroll = true;
        self
    }

    /// Drive the scroll-linked keyframes by *another* element's traversal
    /// of the viewport — a CSS selector like `"#hero"`. When the selector
    /// matches nothing, the element itself is used.
    ///
    /// Implies [`Motion::scroll`].
    ///
    /// ```rust
    /// use autumn_plugin_motion::Motion;
    /// use autumn_web::{Markup, html};
    ///
    /// let html: Markup = Motion::fade()
    ///     .scroll_target("#hero")
    ///     .wrap(html! { p { "Hi" } });
    /// let s = html.into_string();
    /// assert!(s.contains(r#"data-motion-scroll-target="#), "{s}");
    /// assert!(s.contains("#hero"), "{s}");
    /// assert!(s.contains("data-motion-scroll"), "{s}");
    /// ```
    #[must_use]
    pub fn scroll_target(mut self, selector: impl Into<String>) -> Self {
        self.scroll_target = Some(selector.into());
        self.scroll = true;
        self
    }

    /// Remap the scroll-linked keyframes onto custom viewport edges, e.g.
    /// `["start end", "center center"]` finishes the animation when the
    /// element's center reaches the viewport's center (default
    /// `["start end", "end start"]`: the full traversal).
    ///
    /// Each entry names a target edge and a container edge
    /// (`start`/`center`/`end`, or a number with optional `px`/`%`).
    /// Implies [`Motion::scroll`].
    ///
    /// ```rust
    /// use autumn_plugin_motion::Motion;
    /// use autumn_web::{Markup, html};
    ///
    /// let html: Markup = Motion::fade()
    ///     .scroll_offset(["start end", "center center"])
    ///     .wrap(html! { p { "Hi" } });
    /// assert!(
    ///     html.into_string().contains(
    ///         r#"data-motion-scroll-offset="start end,center center""#
    ///     )
    /// );
    /// ```
    #[must_use]
    pub fn scroll_offset(mut self, offset: [&str; 2]) -> Self {
        self.scroll_offset = Some([offset[0].into(), offset[1].into()]);
        self.scroll = true;
        self
    }

    /// Grow to `scale` while hovered (e.g. `1.05` for a subtle lift).
    /// The element eases back on pointer leave. Pairs with entrance
    /// presets; combines with [`Motion::hover_brightness`].
    ///
    /// ```rust
    /// use autumn_plugin_motion::Motion;
    /// use autumn_web::{Markup, html};
    ///
    /// let html: Markup = Motion::fade_up()
    ///     .hover_scale(1.05)
    ///     .hover_brightness(1.1)
    ///     .wrap(html! { p { "Hi" } });
    /// let s = html.into_string();
    /// assert!(s.contains(r#"data-motion-hover="scale(1.05) brightness(1.1)""#), "{s}");
    /// ```
    #[must_use]
    pub const fn hover_scale(mut self, scale: f32) -> Self {
        self.hover_scale = Some(scale);
        self
    }

    /// Brighten to `brightness` while hovered (e.g. `1.1`). Combines with
    /// [`Motion::hover_scale`].
    #[must_use]
    pub const fn hover_brightness(mut self, brightness: f32) -> Self {
        self.hover_brightness = Some(brightness);
        self
    }

    /// Shrink to `scale` while pressed (e.g. `0.97` for a tactile button).
    /// Releases back on pointer up.
    ///
    /// ```rust
    /// use autumn_plugin_motion::Motion;
    /// use autumn_web::{Markup, html};
    ///
    /// let html: Markup = Motion::fade_up().press_scale(0.97).wrap(html! { p { "Hi" } });
    /// assert!(
    ///     html.into_string().contains(r#"data-motion-press="scale(0.97)""#)
    /// );
    /// ```
    #[must_use]
    pub const fn press_scale(mut self, scale: f32) -> Self {
        self.press_scale = Some(scale);
        self
    }

    /// Wraps `markup` in a `<div>` carrying the animation attributes.
    ///
    /// This is the Maud-friendly composition path — no attribute splicing,
    /// just a wrapper element:
    ///
    /// ```rust
    /// use autumn_plugin_motion::Motion;
    /// use autumn_web::html;
    ///
    /// let page = html! {
    ///     (Motion::fade_up().delay(100).wrap(html! { p { "Hi" } }))
    /// };
    /// ```
    ///
    /// Takes `markup` by value even though Maud only borrows it: the owned
    /// form is the ergonomic call shape (`.wrap(html! { ... })`).
    #[allow(clippy::needless_pass_by_value)]
    #[must_use]
    pub fn wrap(self, markup: Markup) -> Markup {
        let preset = self.preset.attr();
        let delay = self.delay_ms.ne(&0).then(|| self.delay_ms.to_string());
        let duration = self
            .duration_secs
            .filter(|d| *d != DEFAULT_DURATION_SECS)
            .map(|d| d.to_string());
        let once = (!self.once).then_some("false");
        let stagger = self.stagger_ms.map(|ms| ms.to_string());
        let scroll = self.scroll;
        let ease = self.ease.map(Ease::attr_value);
        let repeat = self.repeat.map(|n| n.to_string());
        let repeat_type = self.repeat_type.map(RepeatType::attr);
        let amount = self.amount.map(InViewAmount::attr_value);
        let margin = self.margin;
        let scroll_target = self.scroll_target;
        let scroll_offset = self.scroll_offset.map(|[a, b]| format!("{a},{b}"));
        let parallax = self.parallax_factor.map(|f| format!("{f}"));
        let stagger_from = self.stagger_from.map(StaggerFrom::attr_value);
        let stagger_ease = self.stagger_ease.map(Ease::attr_value);
        let mut hover_parts = Vec::new();
        if let Some(scale) = self.hover_scale {
            hover_parts.push(format!("scale({scale})"));
        }
        if let Some(brightness) = self.hover_brightness {
            hover_parts.push(format!("brightness({brightness})"));
        }
        let hover = (!hover_parts.is_empty()).then(|| hover_parts.join(" "));
        let press = self.press_scale.map(|scale| format!("scale({scale})"));
        html! {
            div data-motion=(preset)
                data-motion-delay=[delay.as_deref()]
                data-motion-duration=[duration.as_deref()]
                data-motion-ease=[ease.as_deref()]
                data-motion-repeat=[repeat.as_deref()]
                data-motion-repeat-type=[repeat_type]
                data-motion-amount=[amount.as_deref()]
                data-motion-margin=[margin.as_deref()]
                data-motion-parallax=[parallax.as_deref()]
                data-motion-scroll-target=[scroll_target.as_deref()]
                data-motion-scroll-offset=[scroll_offset.as_deref()]
                data-motion-stagger-from=[stagger_from.as_deref()]
                data-motion-stagger-ease=[stagger_ease.as_deref()]
                data-motion-hover=[hover.as_deref()]
                data-motion-press=[press.as_deref()]
                data-motion-once=[once]
                data-motion-stagger=[stagger.as_deref()]
                data-motion-scroll[scroll]
            {
                (markup)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Renders a [`Motion`] wrapping empty markup.
    fn render(motion: Motion) -> String {
        motion.wrap(html! {}).into_string()
    }

    #[test]
    fn all_presets_render_their_attribute() {
        let cases = [
            (Motion::fade_up(), "fade-up"),
            (Motion::fade_down(), "fade-down"),
            (Motion::fade_left(), "fade-left"),
            (Motion::fade_right(), "fade-right"),
            (Motion::fade(), "fade"),
            (Motion::scale(), "scale"),
            (Motion::zoom_in(), "zoom-in"),
            (Motion::zoom_out(), "zoom-out"),
            (Motion::slide_left(), "slide-left"),
            (Motion::slide_right(), "slide-right"),
            (Motion::slide_up(), "slide-up"),
            (Motion::slide_down(), "slide-down"),
            (Motion::rotate_in(), "rotate-in"),
            (Motion::blur_in(), "blur-in"),
        ];
        for (motion, attr) in cases {
            let html = render(motion);
            assert!(
                html.contains(&format!(r#"data-motion="{attr}""#)),
                "preset renders data-motion=\"{attr}\": {html}"
            );
        }
    }

    #[test]
    fn defaults_omit_optional_attributes() {
        let html = render(Motion::fade_up());
        for attr in [
            "data-motion-delay",
            "data-motion-duration",
            "data-motion-ease",
            "data-motion-repeat",
            "data-motion-amount",
            "data-motion-margin",
            "data-motion-parallax",
            "data-motion-scroll-target",
            "data-motion-scroll-offset",
            "data-motion-stagger-from",
            "data-motion-stagger-ease",
            "data-motion-hover",
            "data-motion-press",
            "data-motion-once",
            "data-motion-stagger",
            "data-motion-scroll",
        ] {
            assert!(!html.contains(attr), "default omits {attr}: {html}");
        }
    }

    #[test]
    fn modifiers_render_their_attributes() {
        let html = render(
            Motion::scale()
                .delay(150)
                .duration(1.2)
                .once(false)
                .stagger(60),
        );
        assert!(html.contains(r#"data-motion-delay="150""#), "{html}");
        assert!(html.contains(r#"data-motion-duration="1.2""#), "{html}");
        assert!(html.contains(r#"data-motion-once="false""#), "{html}");
        assert!(html.contains(r#"data-motion-stagger="60""#), "{html}");
    }

    #[test]
    fn default_duration_is_omitted() {
        let html = render(Motion::fade().duration(DEFAULT_DURATION_SECS));
        assert!(
            !html.contains("data-motion-duration"),
            "explicit default duration is omitted: {html}"
        );
    }

    #[test]
    fn ease_renders_its_attribute() {
        let cases = [
            (Ease::Linear, r#"data-motion-ease="linear""#),
            (Ease::Out, r#"data-motion-ease="ease-out""#),
            (Ease::InOut, r#"data-motion-ease="ease-in-out""#),
            (Ease::CircOut, r#"data-motion-ease="circ-out""#),
            (Ease::BackIn, r#"data-motion-ease="back-in""#),
            (Ease::Anticipate, r#"data-motion-ease="anticipate""#),
            (
                Ease::CubicBezier(0.16, 1.0, 0.3, 1.0),
                r#"data-motion-ease="cubic-bezier(0.16,1,0.3,1)""#,
            ),
            (
                Ease::Spring {
                    stiffness: 300.0,
                    damping: 20.0,
                    mass: 1.0,
                },
                r#"data-motion-ease="spring(300,20,1)""#,
            ),
        ];
        for (ease, attr) in cases {
            let html = render(Motion::fade_up().ease(ease));
            assert!(html.contains(attr), "ease renders {attr}: {html}");
        }
    }

    #[test]
    fn repeat_renders_its_attributes() {
        let html = render(Motion::scale().repeat(2).repeat_type(RepeatType::Mirror));
        assert!(html.contains(r#"data-motion-repeat="2""#), "{html}");
        assert!(
            html.contains(r#"data-motion-repeat-type="mirror""#),
            "{html}"
        );
    }

    #[test]
    fn amount_variants_render() {
        let html = render(Motion::fade_up().amount(0.5));
        assert!(html.contains(r#"data-motion-amount="0.5""#), "{html}");
        let html = render(Motion::fade_up().amount(InViewAmount::All));
        assert!(html.contains(r#"data-motion-amount="all""#), "{html}");
        let html = render(Motion::fade_up().amount(InViewAmount::Some));
        assert!(html.contains(r#"data-motion-amount="some""#), "{html}");
    }

    #[test]
    fn amount_fraction_clamps_to_unit_range() {
        let html = render(Motion::fade_up().amount(1.5));
        assert!(html.contains(r#"data-motion-amount="1""#), "{html}");
        let html = render(Motion::fade_up().amount(-0.2));
        assert!(html.contains(r#"data-motion-amount="0""#), "{html}");
    }

    #[test]
    fn margin_renders() {
        let html = render(Motion::fade_up().margin("-100px"));
        assert!(html.contains(r#"data-motion-margin="-100px""#), "{html}");
    }

    #[test]
    fn parallax_renders_kind_and_factor() {
        let html = render(Motion::parallax(0.3));
        assert!(html.contains(r#"data-motion="parallax""#), "{html}");
        assert!(html.contains(r#"data-motion-parallax="0.3""#), "{html}");
    }

    #[test]
    fn scroll_target_renders_and_implies_scroll() {
        let html = render(Motion::fade().scroll_target("#hero"));
        assert!(html.contains("data-motion-scroll-target"), "{html}");
        assert!(html.contains("#hero"), "{html}");
        assert!(html.contains("data-motion-scroll"), "{html}");
    }

    #[test]
    fn scroll_offset_renders_comma_separated() {
        let html = render(Motion::fade().scroll_offset(["start end", "center center"]));
        assert!(
            html.contains(r#"data-motion-scroll-offset="start end,center center""#),
            "{html}"
        );
    }

    #[test]
    fn scroll_progress_renders_opinionated_bar() {
        let html = Motion::scroll_progress().into_string();
        assert!(html.contains(r#"data-motion="scroll-progress""#), "{html}");
        assert!(html.contains(r#"class="motion-progress""#), "{html}");
    }

    #[test]
    fn stagger_from_and_ease_render() {
        let html = render(
            Motion::fade_up()
                .stagger(90)
                .stagger_from(StaggerFrom::Center)
                .stagger_ease(Ease::Out),
        );
        assert!(html.contains(r#"data-motion-stagger="90""#), "{html}");
        assert!(
            html.contains(r#"data-motion-stagger-from="center""#),
            "{html}"
        );
        assert!(
            html.contains(r#"data-motion-stagger-ease="ease-out""#),
            "{html}"
        );
    }

    #[test]
    fn stagger_from_index_renders_number() {
        let html = render(Motion::fade_up().stagger_from(StaggerFrom::Index(2)));
        assert!(html.contains(r#"data-motion-stagger-from="2""#), "{html}");
    }

    #[test]
    fn hover_and_press_render() {
        let html = render(
            Motion::fade_up()
                .hover_scale(1.05)
                .hover_brightness(1.1)
                .press_scale(0.97),
        );
        assert!(
            html.contains(r#"data-motion-hover="scale(1.05) brightness(1.1)""#),
            "{html}"
        );
        assert!(
            html.contains(r#"data-motion-press="scale(0.97)""#),
            "{html}"
        );
    }

    #[test]
    fn scroll_renders_bare_attribute() {
        let html = render(Motion::fade_up().scroll());
        assert!(
            html.contains("data-motion-scroll"),
            "scroll renders the bare attribute: {html}"
        );
    }

    #[test]
    fn wrap_keeps_inner_markup() {
        let html = Motion::fade_up()
            .wrap(html! { p { "inner text" } })
            .into_string();
        assert!(html.starts_with("<div"), "{html}");
        assert!(html.contains("<p>inner text</p>"), "{html}");
        assert!(html.contains(r#"data-motion="fade-up""#), "{html}");
    }
}
