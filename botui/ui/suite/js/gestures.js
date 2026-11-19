/* Shared pointer-event input layer (#2419).
 *
 * Plain script, loaded before app scripts. Exposes window.gbGestures with
 * onTap / onLongPress / onSwipe plus two small geometry helpers. Every helper
 * returns a cleanup function so a re-rendered app can detach its listeners.
 * Mouse behaviour is left to the legacy click/contextmenu handlers: long-press
 * deliberately ignores mouse pointers so desktop interactions stay identical.
 */
"use strict";

(function () {
    const LONG_PRESS_MS = 500;
    const MOVE_TOLERANCE = 12;
    const SWIPE_THRESHOLD = 40;

    function isCoarsePointer() {
        return typeof window.matchMedia === "function" && window.matchMedia("(pointer: coarse)").matches;
    }

    function isPrimary(e) {
        return e.isPrimary !== false && (e.pointerType !== "mouse" || e.button === 0);
    }

    function capture(el, e) {
        const target = el && el.setPointerCapture ? el : (e.target && e.target.setPointerCapture ? e.target : null);
        if (target) {
            try {
                target.setPointerCapture(e.pointerId);
            } catch (error) {
                /* Pointer can be gone between events; tracking still works. */
                if (window.gbLogError) window.gbLogError(error, "gestures.capture");
            }
        }
    }

    function distance(a, b) {
        return Math.hypot(b.clientX - a.x, b.clientY - a.y);
    }

    function onTap(el, handler, options) {
        const opts = options || {};
        const tolerance = typeof opts.tolerance === "number" ? opts.tolerance : MOVE_TOLERANCE;
        let start = null;

        function down(e) {
            if (!isPrimary(e)) return;
            start = { id: e.pointerId, x: e.clientX, y: e.clientY };
            capture(el, e);
        }
        function move(e) {
            if (!start || e.pointerId !== start.id) return;
            if (distance(start, e) > tolerance) start = null;
        }
        function up(e) {
            if (!start || e.pointerId !== start.id) return;
            const origin = start;
            start = null;
            handler.call(el, e, origin);
        }
        function cancel(e) {
            if (start && e.pointerId === start.id) start = null;
        }

        el.addEventListener("pointerdown", down);
        el.addEventListener("pointermove", move);
        el.addEventListener("pointerup", up);
        el.addEventListener("pointercancel", cancel);
        return function detach() {
            el.removeEventListener("pointerdown", down);
            el.removeEventListener("pointermove", move);
            el.removeEventListener("pointerup", up);
            el.removeEventListener("pointercancel", cancel);
        };
    }

    function onLongPress(el, handler, options) {
        const opts = options || {};
        const delay = typeof opts.delay === "number" ? opts.delay : LONG_PRESS_MS;
        const tolerance = typeof opts.tolerance === "number" ? opts.tolerance : MOVE_TOLERANCE;
        const includeMouse = !!opts.includeMouse;
        let timer = null;
        let start = null;

        function clear() {
            if (timer !== null) clearTimeout(timer);
            timer = null;
            start = null;
        }
        function down(e) {
            if (!isPrimary(e) || (e.pointerType === "mouse" && !includeMouse)) return;
            start = { id: e.pointerId, x: e.clientX, y: e.clientY, pointerType: e.pointerType };
            capture(el, e);
            timer = setTimeout(function () {
                const origin = start;
                timer = null;
                start = null;
                if (origin) handler.call(el, e, origin);
            }, delay);
        }
        function move(e) {
            if (!start || e.pointerId !== start.id) return;
            if (distance(start, e) > tolerance) clear();
        }
        function cancel(e) {
            if (start && e.pointerId === start.id) clear();
        }

        el.addEventListener("pointerdown", down);
        el.addEventListener("pointermove", move);
        el.addEventListener("pointerup", cancel);
        el.addEventListener("pointercancel", cancel);
        return function detach() {
            clear();
            el.removeEventListener("pointerdown", down);
            el.removeEventListener("pointermove", move);
            el.removeEventListener("pointerup", cancel);
            el.removeEventListener("pointercancel", cancel);
        };
    }

    function onSwipe(el, handler, options) {
        const opts = options || {};
        const threshold = typeof opts.threshold === "number" ? opts.threshold : SWIPE_THRESHOLD;
        let start = null;

        function down(e) {
            if (!isPrimary(e)) return;
            start = { id: e.pointerId, x: e.clientX, y: e.clientY };
            capture(el, e);
        }
        function up(e) {
            if (!start || e.pointerId !== start.id) return;
            const origin = start;
            start = null;
            const dx = e.clientX - origin.x;
            const dy = e.clientY - origin.y;
            const horizontal = Math.abs(dx) >= Math.abs(dy);
            const length = horizontal ? Math.abs(dx) : Math.abs(dy);
            if (length < threshold) return;
            handler.call(el, e, {
                direction: horizontal ? (dx > 0 ? "right" : "left") : (dy > 0 ? "down" : "up"),
                distance: length,
                dx: dx,
                dy: dy,
                origin: origin
            });
        }
        function cancel(e) {
            if (start && e.pointerId === start.id) start = null;
        }

        el.addEventListener("pointerdown", down);
        el.addEventListener("pointerup", up);
        el.addEventListener("pointercancel", cancel);
        return function detach() {
            el.removeEventListener("pointerdown", down);
            el.removeEventListener("pointerup", up);
            el.removeEventListener("pointercancel", cancel);
        };
    }

    function clampToViewport(el, x, y, margin) {
        if (!el) return;
        const m = typeof margin === "number" ? margin : 8;
        const view = window.visualViewport;
        const width = view ? view.width : window.innerWidth;
        const height = view ? view.height : window.innerHeight;
        const offsetX = view ? view.offsetLeft : 0;
        const offsetY = view ? view.offsetTop : 0;
        const rect = el.getBoundingClientRect();
        const left = Math.max(m, Math.min(x, width - rect.width - m)) + offsetX;
        const top = Math.max(m, Math.min(y, height - rect.height - m)) + offsetY;
        el.style.left = Math.round(left) + "px";
        el.style.top = Math.round(top) + "px";
    }

    window.gbGestures = {
        isCoarsePointer: isCoarsePointer,
        onTap: onTap,
        onLongPress: onLongPress,
        onSwipe: onSwipe,
        clampToViewport: clampToViewport
    };
})();
