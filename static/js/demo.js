/* Scroll-driven progress bar for the motion demo.
 *
 * External file on purpose: Autumn's default CSP is `script-src 'self'`
 * (no `unsafe-inline`), so inline `<script>` blocks are blocked. Loaded
 * with `defer`, after the plugin's vendored Motion build, so
 * `window.Motion` is ready. Served via `asset_url("js/demo.js")`. */

(function () {
    var M = window.Motion;
    if (!M) return;
    var bar = document.querySelector('.progress');
    if (!bar) return;
    // Page-scroll-driven progress: scaleX 0 -> 1 across the whole document.
    M.scroll(M.animate(bar, { scaleX: [0, 1] }, { easing: 'linear' }));
})();
