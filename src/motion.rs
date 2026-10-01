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
//! - `data-motion="fade-up|fade|scale|slide-left|slide-right"` — the preset
//!   (default `fade-up`).
//! - `data-motion-delay="150"` — start delay in milliseconds (default `0`).
//! - `data-motion-duration="0.6"` — duration in seconds (default `0.6`).
//! - `data-motion-once="false"` — re-animate every time the element enters
//!   the viewport (default is once).
//! - `data-motion-stagger="60"` — on a container: cascade the preset over
//!   its direct children, `60` ms apart.
//! - `data-motion-scroll` — bare attribute: drive the keyframes by scroll
//!   progress instead of animating on entry.

use autumn_web::{Markup, html};

/// Default animation duration in seconds, used when no duration is set.
const DEFAULT_DURATION_SECS: f32 = 0.6;

/// An animation preset, mirroring the keyframes in `assets/init.js`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    /// Fade in while rising 24px.
    FadeUp,
    /// Fade in place.
    Fade,
    /// Fade in while scaling from 0.94.
    Scale,
    /// Fade in while sliding 32px from the right.
    SlideLeft,
    /// Fade in while sliding 32px from the left.
    SlideRight,
}

impl Preset {
    /// The `data-motion` attribute value for this preset.
    const fn attr(self) -> &'static str {
        match self {
            Self::FadeUp => "fade-up",
            Self::Fade => "fade",
            Self::Scale => "scale",
            Self::SlideLeft => "slide-left",
            Self::SlideRight => "slide-right",
        }
    }
}

/// A declarative animation: preset plus modifiers.
///
/// Build with a preset constructor ([`Motion::fade_up`], [`Motion::fade`],
/// [`Motion::scale`], [`Motion::slide_left`], [`Motion::slide_right`]),
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

    /// Builds a [`Motion`] from a raw preset.
    const fn preset(preset: Preset) -> Self {
        Self {
            preset,
            delay_ms: 0,
            duration_secs: None,
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
        html! {
            div data-motion=(preset)
                data-motion-delay=[delay.as_deref()]
                data-motion-duration=[duration.as_deref()]
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
            (Motion::fade(), "fade"),
            (Motion::scale(), "scale"),
            (Motion::slide_left(), "slide-left"),
            (Motion::slide_right(), "slide-right"),
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
