
"use strict";

// Slide transition effects — DOM-based CSS transitions, no canvas/webgl

var TRANSITIONS = {
    FADE: "fade",
    SLIDE_LEFT: "slide-left",
    SLIDE_RIGHT: "slide-right",
    ZOOM_IN: "zoom-in",
    ZOOM_OUT: "zoom-out",
    WIPE_LEFT: "wipe-left",
    WIPE_RIGHT: "wipe-right",
    WIPE_UP: "wipe-up",
    WIPE_DOWN: "wipe-down",
    NONE: "none"
};

function applyTransition(slideEl, transition, duration) {
    if (!slideEl) return;
    duration = duration || 400;
    transition = transition || "fade";
    var cls = "transition-" + transition;
    slideEl.classList.remove("transition-fade", "transition-slide-left", "transition-slide-right",
        "transition-zoom-in", "transition-zoom-out", "transition-wipe-left", "transition-wipe-right",
        "transition-wipe-up", "transition-wipe-down", "transition-none");
    slideEl.style.transition = "all " + duration + "ms ease-in-out";
    slideEl.classList.add(cls);
    slideEl.style.opacity = "1";
}

function getCurrentTransition(slideEl) {
    for (var key in TRANSITIONS) {
        var cls = "transition-" + TRANSITIONS[key];
        if (slideEl.classList.contains(cls)) return TRANSITIONS[key];
    }
    return "none";
}

function getDefaultTransition() {
    return window.slideConfig && window.slideConfig.defaultTransition || "fade";
}

// Inject transition styles once
(function() {
    if (document.getElementById("transition-styles")) return;
    var style = document.createElement("style");
    style.id = "transition-styles";
    style.textContent = [
        ".transition-fade { animation: fadeIn 0.4s ease; }",
        ".transition-slide-left { animation: slideLeft 0.4s ease; }",
        ".transition-slide-right { animation: slideRight 0.4s ease; }",
        ".transition-zoom-in { animation: zoomIn 0.4s ease; }",
        ".transition-zoom-out { animation: zoomOut 0.4s ease; }",
        ".transition-wipe-left { animation: wipeLeft 0.4s ease; }",
        ".transition-wipe-right { animation: wipeRight 0.4s ease; }",
        ".transition-wipe-up { animation: wipeUp 0.4s ease; }",
        ".transition-wipe-down { animation: wipeDown 0.4s ease; }",
        ".transition-none { animation: none; }",
        "@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }",
        "@keyframes slideLeft { from { transform: translateX(100%); opacity: 0; } to { transform: translateX(0); opacity: 1; } }",
        "@keyframes slideRight { from { transform: translateX(-100%); opacity: 0; } to { transform: translateX(0); opacity: 1; } }",
        "@keyframes zoomIn { from { transform: scale(0.5); opacity: 0; } to { transform: scale(1); opacity: 1; } }",
        "@keyframes zoomOut { from { transform: scale(1.5); opacity: 0; } to { transform: scale(1); opacity: 1; } }",
        "@keyframes wipeLeft { from { clip-path: inset(0 100% 0 0); } to { clip-path: inset(0 0 0 0); } }",
        "@keyframes wipeRight { from { clip-path: inset(0 0 0 100%); } to { clip-path: inset(0 0 0 0); } }",
        "@keyframes wipeUp { from { clip-path: inset(100% 0 0 0); } to { clip-path: inset(0 0 0 0); } }",
        "@keyframes wipeDown { from { clip-path: inset(0 0 100% 0); } to { clip-path: inset(0 0 0 0); } }"
    ].join("\n");
    document.head.appendChild(style);
})();

window.applyTransition = applyTransition;
window.getCurrentTransition = getCurrentTransition;
window.getDefaultTransition = getDefaultTransition;
window.TRANSITIONS = TRANSITIONS;
