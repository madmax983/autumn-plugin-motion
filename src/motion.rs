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

    /// Builds a [`Motion`] from a raw preset.
    const fn preset(preset: Preset) -> Self {
        Self {
            preset,
            delay_ms: 0,
            duration_secs: None,
            ease: None,
            repeat: None,
            repeat_type: None,
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

    /// Drive the keyframes by scroll progress as the element travels through
    /// the viewport, instead of animating on entry.
    #[must_use]
    pub const fn scroll(mut self) -> Self {
        self.scroll = true;
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
        html! {
            div data-motion=(preset)
                data-motion-delay=[delay.as_deref()]
                data-motion-duration=[duration.as_deref()]
                data-motion-ease=[ease.as_deref()]
                data-motion-repeat=[repeat.as_deref()]
                data-motion-repeat-type=[repeat_type]
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
