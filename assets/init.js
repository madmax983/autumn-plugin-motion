/* Autumn Motion plugin — declarative animation init.
 *
 * Scans `[data-motion]` elements and wires Motion (motion.dev) animations:
 * in-view reveals, stagger cascades, and scroll-linked effects. Re-scans on
 * `htmx:afterSwap` so htmx-swapped content animates declaratively too.
 *
 * Progressive enhancement: if Motion fails to load, or the user prefers
 * reduced motion, this script does nothing and all content stays visible.
 */
(function () {
    "use strict";

    // Preset name -> Motion keyframes. Keep in sync with the Rust `Motion`
    // builder presets in `src/motion.rs`.
    var PRESETS = {
        "fade-up": { opacity: [0, 1], y: [24, 0] },
        fade: { opacity: [0, 1] },
        scale: { opacity: [0, 1], scale: [0.94, 1] },
        "slide-left": { opacity: [0, 1], x: [32, 0] },
        "slide-right": { opacity: [0, 1], x: [-32, 0] },
    };

    var DEFAULT_DURATION = 0.6;
    // Snappy expo-out-ish easing shared by every preset unless the element
    // sets data-motion-ease.
    var DEFAULT_EASING = [0.16, 1, 0.3, 1];

    // data-motion-ease kebab names -> Motion easing names.
    var NAMED_EASINGS = {
        linear: "linear",
        "ease-in": "easeIn",
        "ease-out": "easeOut",
        "ease-in-out": "easeInOut",
        "circ-in": "circIn",
        "circ-out": "circOut",
        "circ-in-out": "circInOut",
        "back-in": "backIn",
        "back-out": "backOut",
        "back-in-out": "backInOut",
        anticipate: "anticipate",
    };

    function motion() {
        return window.Motion || null;
    }

    function reducedMotion() {
        return (
            typeof window.matchMedia === "function" &&
            window.matchMedia("(prefers-reduced-motion: reduce)").matches
        );
    }

    // Apply the "from" keyframe values as inline styles so elements do not
    // flash fully-visible before their in-view animation starts.
    function hideFrom(el, keyframes) {
        if (keyframes.opacity) {
            el.style.opacity = String(keyframes.opacity[0]);
        }
        var x = keyframes.x ? keyframes.x[0] : 0;
        var y = keyframes.y ? keyframes.y[0] : 0;
        var scale = keyframes.scale ? keyframes.scale[0] : 1;
        var t = "";
        if (x) {
            t += "translateX(" + x + "px) ";
        }
        if (y) {
            t += "translateY(" + y + "px) ";
        }
        if (scale !== 1) {
            t += "scale(" + scale + ") ";
        }
        if (t) {
            el.style.transform = t.trim();
        }
    }

    // Parse data-motion-ease into a Motion transition fragment.
    // Accepts kebab-case names ("ease-out"), "cubic-bezier(a,b,c,d)", and
    // "spring(stiffness,damping,mass)". Unknown values fall back to the
    // default easing.
    function readEase(el) {
        var raw = el.getAttribute("data-motion-ease");
        if (!raw) {
            return { ease: DEFAULT_EASING };
        }
        var m = raw.match(/^cubic-bezier\(([^)]+)\)$/);
        if (m) {
            var curve = m[1].split(",").map(Number);
            if (
                curve.length === 4 &&
                curve.every(function (n) {
                    return isFinite(n);
                })
            ) {
                return { ease: curve };
            }
            return { ease: DEFAULT_EASING };
        }
        m = raw.match(/^spring\(([^)]+)\)$/);
        if (m) {
            var s = m[1].split(",").map(Number);
            if (
                s.length === 3 &&
                s.every(function (n) {
                    return isFinite(n);
                })
            ) {
                return {
                    type: "spring",
                    stiffness: s[0],
                    damping: s[1],
                    mass: s[2],
                };
            }
            return { ease: DEFAULT_EASING };
        }
        if (
            Object.prototype.hasOwnProperty.call(NAMED_EASINGS, raw)
        ) {
            return { ease: NAMED_EASINGS[raw] };
        }
        return { ease: DEFAULT_EASING };
    }

    function readOpts(el) {
        var delayMs = parseInt(el.getAttribute("data-motion-delay") || "0", 10);
        var duration = parseFloat(el.getAttribute("data-motion-duration") || "");
        return {
            delay: (isNaN(delayMs) ? 0 : delayMs) / 1000,
            duration: isNaN(duration) ? DEFAULT_DURATION : duration,
            once: el.getAttribute("data-motion-once") !== "false",
            scroll: el.hasAttribute("data-motion-scroll"),
        };
    }

    function animateEl(M, el) {
        var kind = el.getAttribute("data-motion") || "fade-up";
        var keyframes = PRESETS[kind] || PRESETS["fade-up"];
        var opts = readOpts(el);
        // NOTE: Motion's transition option is `ease`, not `easing` — the
        // unknown `easing` key was silently ignored before Phase 1.
        var timing = Object.assign(
            {
                duration: opts.duration,
                delay: opts.delay,
            },
            readEase(el),
        );

        if (opts.scroll) {
            // Scroll-linked: drive the keyframes by scroll progress as the
            // element travels through the viewport. Always linear — the
            // scroll position is the clock.
            M.scroll(M.animate(el, keyframes, { ease: "linear" }), {
                target: el,
                offset: ["start end", "end start"],
            });
            return;
        }

        var staggerAttr = el.getAttribute("data-motion-stagger");
        if (staggerAttr !== null) {
            // Stagger container: cascade the preset over direct children.
            var kids = Array.prototype.filter.call(el.children, function (c) {
                return c.nodeType === 1 && !c.hasAttribute("data-motion-init");
            });
            kids.forEach(function (c) {
                c.setAttribute("data-motion-init", "true");
                hideFrom(c, keyframes);
            });
            if (!kids.length) {
                return;
            }
            var step = (parseInt(staggerAttr, 10) || 0) / 1000;
            M.inView(
                el,
                function () {
                    M.animate(
                        kids,
                        keyframes,
                        Object.assign({}, timing, {
                            delay: M.stagger(step, { startDelay: opts.delay }),
                        }),
                    );
                },
                { once: opts.once },
            );
            return;
        }

        M.inView(
            el,
            function () {
                M.animate(el, keyframes, timing);
            },
            { once: opts.once },
        );
    }

    // Scan `root` for uninitialized `[data-motion]` elements. `root` may be
    // the document or an htmx swap target.
    function initIn(root) {
        var M = motion();
        if (!M || reducedMotion()) {
            return;
        }
        var scope = root && root.querySelectorAll ? root : document;
        var els = [];
        if (
            scope !== document &&
            scope.matches &&
            scope.matches("[data-motion]:not([data-motion-init])")
        ) {
            els.push(scope);
        }
        var found = scope.querySelectorAll("[data-motion]:not([data-motion-init])");
        for (var i = 0; i < found.length; i++) {
            els.push(found[i]);
        }
        els.forEach(function (el) {
            el.setAttribute("data-motion-init", "true");
            var stagger = el.hasAttribute("data-motion-stagger");
            if (!stagger) {
                var kind = el.getAttribute("data-motion") || "fade-up";
                hideFrom(el, PRESETS[kind] || PRESETS["fade-up"]);
            }
            animateEl(M, el);
        });
    }

    function boot() {
        initIn(document);
    }

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", boot);
    } else {
        boot();
    }

    // htmx: re-scan swapped-in content so every partial animates
    // declaratively. `e.detail.target` scopes the scan to the swap target.
    document.body.addEventListener("htmx:afterSwap", function (e) {
        initIn(e.detail && e.detail.target ? e.detail.target : document);
    });
})();
