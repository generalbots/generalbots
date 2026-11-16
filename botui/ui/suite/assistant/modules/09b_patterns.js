"use strict";
/**
 * 09b_patterns.js - Chat background pattern data and logic.
 * Split from 09_settings.js to stay under the 450-line limit.
 * Exposes A._patterns, A._getCurrentPattern, A._setCurrentPattern,
 * A._applyPattern, A._applyPatternCSS, A.bootPattern on GBAssistant.
 */

(function () {
    var A = window.GBAssistant;

    var PATTERNS = [
        { id: "none", css: "" },
        { id: "dots", css: "background-image:radial-gradient(var(--id-accent) 0.8px, var(--id-paper) 0.8px);background-size:8px 8px;" },
        { id: "microdots", css: "background:radial-gradient(var(--id-accent) 15%,transparent 16%) 0 0,radial-gradient(var(--id-accent) 15%,transparent 16%) 4px 4px,radial-gradient(var(--pat-accent20) 15%,transparent 20%) 0 1px,radial-gradient(var(--pat-accent20) 15%,transparent 20%) 4px 5px;background-color:var(--id-paper);background-size:8px 8px;" },
        { id: "grid", css: "background-image:linear-gradient(var(--id-accent) 0.8px,transparent 0.8px),linear-gradient(90deg,var(--id-accent) 0.8px,transparent 0.8px),linear-gradient(var(--id-accent) 0.4px,transparent 0.4px),linear-gradient(90deg,var(--id-accent) 0.4px,var(--id-paper) 0.4px);background-size:40px 40px,40px 40px,8px 8px,8px 8px;background-position:-0.8px -0.8px,-0.8px -0.8px,-0.4px -0.4px,-0.4px -0.4px;" },
        { id: "blueprint", css: "background-image:repeating-linear-gradient(to right,transparent 0,transparent 1.333px,var(--id-paper) 1.333px,var(--id-paper) 4px,transparent 4px,transparent 5.333px),repeating-linear-gradient(to bottom,transparent 0,transparent 1.333px,var(--id-paper) 1.333px,var(--id-paper) 4px,transparent 4px,transparent 5.333px),linear-gradient(to bottom,var(--id-accent) 0.48px,transparent 0.48px),linear-gradient(to right,var(--id-accent) 0.48px,transparent 0.48px);background-size:100% 100%,100% 100%,16px 16px,16px 16px;background-position:0 0,0 0,0 -0.16px,-0.16px 0;" },
        { id: "weave", css: "background:radial-gradient(circle,transparent 20%,var(--id-paper) 20%,var(--id-paper) 80%,transparent 80%,transparent),radial-gradient(circle,transparent 20%,var(--id-paper) 20%,var(--id-paper) 80%,transparent 80%,transparent) 15px 15px,linear-gradient(var(--id-accent) 1.2px,transparent 1.2px) 0 -0.6px,linear-gradient(90deg,var(--id-accent) 1.2px,var(--id-paper) 1.2px) -0.6px 0;background-size:30px 30px,30px 30px,15px 15px,15px 15px;" },
        { id: "mosaic", css: "background-image:repeating-linear-gradient(0deg,transparent,transparent 11px,var(--id-accent) 11px,var(--id-accent) 12px,transparent 12px,transparent 23px,var(--id-accent) 23px,var(--id-accent) 24px),repeating-linear-gradient(90deg,transparent,transparent 11px,var(--id-accent) 11px,var(--id-accent) 12px,transparent 12px,transparent 23px,var(--id-accent) 23px,var(--id-accent) 24px),radial-gradient(circle at 12px 12px,var(--id-accent) 0.96px,transparent 1.46px),radial-gradient(circle at 0px 0px,var(--id-accent) 0.72px,transparent 1.22px),radial-gradient(circle at 24px 0px,var(--id-accent) 0.72px,transparent 1.22px),radial-gradient(circle at 0px 24px,var(--id-accent) 0.72px,transparent 1.22px),radial-gradient(circle at 24px 24px,var(--id-accent) 0.72px,transparent 1.22px);background-size:24px 24px;background-color:var(--id-paper);" },
    ];
    /* "image" = the shipped pattern.png (dot composition) */
    PATTERNS.splice(1, 0, { id: "image", css: "__IMAGE__" });

    function getCurrentPattern() {
        try { return localStorage.getItem("gb-chat-pattern") || "image"; } catch (e) { return "image"; }
    }
    function setCurrentPattern(id) {
        try { localStorage.setItem("gb-chat-pattern", id); } catch (e) { /* */ }
    }

    function applyPattern(id) {
        var main = document.querySelector(".gb-main");
        if (!main) { return; }
        main.classList.remove("gb-pattern-none", "gb-pattern-css", "gb-pattern-image");
        main.style.removeProperty("--gb-pat-css");
        main.removeAttribute("data-pattern");

        if (id === "none") {
            main.classList.add("gb-pattern-none");
        } else if (id === "image") {
            main.classList.add("gb-pattern-image");
        } else {
            var pat = PATTERNS.filter(function (p) { return p.id === id; })[0];
            if (pat && pat.css) {
                main.classList.add("gb-pattern-css");
                main.setAttribute("data-pattern", id);
                applyPatternCSS(pat.css);
            }
        }
    }

    var patStyleEl = null;
    function applyPatternCSS(css) {
        if (!patStyleEl) {
            patStyleEl = document.createElement("style");
            patStyleEl.id = "gbPatternStyle";
            document.head.appendChild(patStyleEl);
        }
        patStyleEl.textContent =
            ':root { --pat-accent20: color-mix(in srgb, var(--id-accent) 20%, transparent); }' +
            '.gb-pattern-css.gb-main::before { background: none; ' + css + ' opacity: 0.15; -webkit-mask: none !important; mask: none !important; animation: none; z-index: 1; pointer-events: none; }';
    }

    A.bootPattern = function () {
        var id = getCurrentPattern();
        applyPattern(id);
    };

    /* Expose for 09_settings.js */
    A._patterns = PATTERNS;
    A._getCurrentPattern = getCurrentPattern;
    A._setCurrentPattern = setCurrentPattern;
    A._applyPattern = applyPattern;
    A._applyPatternCSS = applyPatternCSS;
})();
