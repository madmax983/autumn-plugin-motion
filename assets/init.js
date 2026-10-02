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
        "fade-down": { opacity: [0, 1], y: [-24, 0] },
        "fade-left": { opacity: [0, 1], x: [24, 0] },
        "fade-right": { opacity: [0, 1], x: [-24, 0] },
        fade: { opacity: [0, 1] },
        scale: { opacity: [0, 1], scale: [0.94, 1] },
        "zoom-in": { opacity: [0, 1], scale: [0.8, 1] },
        "zoom-out": { opacity: [0, 1], scale: [1.15, 1] },
        "slide-left": { opacity: [0, 1], x: [32, 0] },
        "slide-right": { opacity: [0, 1], x: [-32, 0] },
        "slide-up": { opacity: [0, 1], y: [32, 0] },
        "slide-down": { opacity: [0, 1], y: [-32, 0] },
        "rotate-in": { opacity: [0, 1], rotate: [-8, 0] },
        "blur-in": { opacity: [0, 1], filter: ["blur(8px)", "blur(0px)"] },
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
        var rotate = keyframes.rotate ? keyframes.rotate[0] : 0;
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
        if (rotate) {
            t += "rotate(" + rotate + "deg) ";
        }
        if (t) {
            el.style.transform = t.trim();
        }
        if (keyframes.filter) {
            el.style.filter = keyframes.filter[0];
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

    var REPEAT_TYPES = { loop: true, reverse: true, mirror: true };
    var AMOUNT_KEYWORDS = { some: true, all: true };
    // rootMargin only accepts px/% lengths, 1-4 of them like CSS margin.
    // An invalid rootMargin makes IntersectionObserver throw, which would
    // kill the whole scan — so validate and drop bad values instead.
    var MARGIN_RE =
        /^-?\d+(\.\d+)?(px|%)(\s+-?\d+(\.\d+)?(px|%)){0,3}$/;

    function readAmount(el) {
        var raw = el.getAttribute("data-motion-amount");
        if (raw === null || raw === "") {
            return null;
        }
        var word = raw.trim().toLowerCase();
        if (AMOUNT_KEYWORDS[word]) {
            return word;
        }
        var n = parseFloat(word);
        if (isNaN(n)) {
            return null;
        }
        return Math.min(1, Math.max(0, n));
    }

    function readMargin(el) {
        var raw = el.getAttribute("data-motion-margin");
        if (raw === null) {
            return null;
        }
        var margin = raw.trim();
        return MARGIN_RE.test(margin) ? margin : null;
    }

    function readOpts(el) {
        var delayMs = parseInt(el.getAttribute("data-motion-delay") || "0", 10);
        var duration = parseFloat(el.getAttribute("data-motion-duration") || "");
        var repeatRaw = el.getAttribute("data-motion-repeat");
        var repeat = repeatRaw === null ? null : parseInt(repeatRaw, 10);
        var repeatType = el.getAttribute("data-motion-repeat-type");
        return {
            delay: (isNaN(delayMs) ? 0 : delayMs) / 1000,
            duration: isNaN(duration) ? DEFAULT_DURATION : duration,
            repeat: repeat !== null && isFinite(repeat) && repeat > 0 ? repeat : null,
            repeatType:
                repeatType && REPEAT_TYPES[repeatType] ? repeatType : null,
            amount: readAmount(el),
            margin: readMargin(el),
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
        if (opts.repeat !== null) {
            timing.repeat = opts.repeat;
            timing.repeatType = opts.repeatType || "loop";
        }
        // inView's `margin` maps to IntersectionObserver's rootMargin and
        // `amount` to its threshold ("some"/"all"/number).
        // NOTE: this Motion build's inView takes { root, margin, amount } —
        // there is no `once` option. Once-ness comes from the callback:
        // return nothing and the observer disconnects that element after
        // the first trigger; return a cleanup and it stays observed.
        var viewOpts = { amount: opts.amount !== null ? opts.amount : "some" };
        if (opts.margin !== null) {
            viewOpts.margin = opts.margin;
        }

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
                    var controls = M.animate(
                        kids,
                        keyframes,
                        Object.assign({}, timing, {
                            delay: M.stagger(step, { startDelay: opts.delay }),
                        }),
                    );
                    // Returning a cleanup keeps the element observed, so the
                    // cascade replays on every entry when once is false.
                    if (!opts.once) {
                        return function () {
                            controls.stop();
                        };
                    }
                },
                viewOpts,
            );
            return;
        }

        M.inView(
            el,
            function () {
                var controls = M.animate(el, keyframes, timing);
                if (!opts.once) {
                    return function () {
                        controls.stop();
                    };
                }
            },
            viewOpts,
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
