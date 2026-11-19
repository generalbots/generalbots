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
        { id: "blueprint", css: "background-image:radial-gradient(circle at 1px 1px,var(--id-accent) 2.5px,transparent 3px),linear-gradient(var(--id-accent) 1.5px,transparent 1.5px),linear-gradient(90deg,var(--id-accent) 1.5px,transparent 1.5px),linear-gradient(color-mix(in srgb,var(--id-accent) 45%,transparent) 1px,transparent 1px),linear-gradient(90deg,color-mix(in srgb,var(--id-accent) 45%,transparent) 1px,transparent 1px);background-size:96px 96px,96px 96px,96px 96px,24px 24px,24px 24px;" },
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
            '.gb-pattern-css.gb-main::before { background: none; ' + css + ' opacity: calc(0.15 * var(--gb-pat-strength, 1)); -webkit-mask: none !important; mask: none !important; animation: none; z-index: 1; pointer-events: none;' +
            ' right: auto; bottom: auto; width: calc(100% / var(--gb-pat-scale, 1)); height: calc(100% / var(--gb-pat-scale, 1)); transform: scale(var(--gb-pat-scale, 1)); transform-origin: 0 0; }';
    }

    /* ---- Sliders: pattern opacity and scale, frost layer (percent values) ---- */
    var SLIDERS = {
        strength: { store: "gb-pat-strength", min: 20, max: 300, step: 5, def: 100, key: "settings.bg-opacity", en: "Opacity" },
        scale:    { store: "gb-pat-scale", min: 50, max: 300, step: 5, def: 100, key: "settings.bg-scale", en: "Scale" },
        frost:    { store: "gb-frost", min: 0, max: 100, step: 5, def: 0, key: "settings.frost", en: "Frost layer (blur)" }
    };
    function getSlider(name) {
        var cfg = SLIDERS[name], v = NaN;
        try { v = parseInt(localStorage.getItem(cfg.store), 10); } catch (e) { /* */ }
        return isNaN(v) ? cfg.def : Math.min(cfg.max, Math.max(cfg.min, v));
    }
    function applySliders() {
        var main = document.querySelector(".gb-main");
        if (!main) { return; }
        var scale = getSlider("scale"), frost = getSlider("frost");
        main.style.setProperty("--gb-pat-strength", String(getSlider("strength") / 100));
        main.style.setProperty("--gb-pat-scale", String(scale / 100));
        main.style.setProperty("--gb-frost", String(frost / 100));
        /* The shipped image keeps its exact default framing until the scale is changed. */
        main.classList.toggle("gb-pat-scaled", scale !== 100);
        var layer = document.getElementById("gbFrost");
        if (!layer && frost > 0) {
            layer = document.createElement("div");
            layer.id = "gbFrost"; layer.className = "gb-frost"; layer.setAttribute("aria-hidden", "true");
            main.insertBefore(layer, main.firstChild);
        }
        if (layer) { layer.hidden = frost <= 0; }
    }
    A._sliderRow = function (name) {
        var cfg = SLIDERS[name], v = getSlider(name);
        return '<div class="gb-set-range-row"><label class="gb-set-label" for="gbRange_' + name + '">' + A.t(cfg.key, cfg.en) + '</label>' +
            '<input type="range" class="gb-set-range" id="gbRange_' + name + '" data-slider="' + name + '" min="' + cfg.min +
            '" max="' + cfg.max + '" step="' + cfg.step + '" value="' + v + '" />' +
            '<output class="gb-set-range-value" for="gbRange_' + name + '">' + v + '%</output>' +
            '<button type="button" class="gb-set-range-reset" data-reset="' + name + '" title="' + A.t("settings.slider-reset", "Default") + '">&#8634;</button></div>';
    };
    A._syncSliders = function (panel) {
        var none = getCurrentPattern() === "none";
        ["strength", "scale"].forEach(function (name) {
            var input = panel.querySelector('[data-slider="' + name + '"]');
            if (input) { input.disabled = none; }
        });
    };
    A._bindSliders = function (panel) {
        function set(input, value) {
            var name = input.getAttribute("data-slider");
            try { localStorage.setItem(SLIDERS[name].store, String(value)); } catch (e) { /* */ }
            input.value = value;
            var out = panel.querySelector('output[for="' + input.id + '"]');
            if (out) { out.textContent = value + "%"; }
            applySliders();
        }
        panel.querySelectorAll(".gb-set-range").forEach(function (input) {
            input.addEventListener("input", function () { set(input, parseInt(input.value, 10)); });
        });
        panel.querySelectorAll("[data-reset]").forEach(function (btn) {
            btn.addEventListener("click", function () {
                var name = btn.getAttribute("data-reset");
                set(panel.querySelector('[data-slider="' + name + '"]'), SLIDERS[name].def);
            });
        });
        A._syncSliders(panel);
    };

    /* The small preview inside a pattern tile of Settings > Appearance. */
    A._previewPattern = function (div) {
        var patId = div.getAttribute("data-pat-id");
        if (!patId || patId === "none") { return; }
        if (patId === "image") {
            /* The shipped dot composition is too detailed for a 64x40 tile: a dot cluster stands for it. */
            var dots = [["1.5px", "20% 30%"], ["2px", "55% 20%"], ["1px", "40% 60%"], ["2.5px", "70% 55%"], ["1.5px", "30% 80%"],
                ["1px", "80% 35%"], ["2px", "15% 55%"], ["1.5px", "60% 75%"], ["1px", "85% 70%"]];
            div.style.cssText = "background:" + dots.map(function (d) {
                return "radial-gradient(" + d[0] + " " + d[0] + " at " + d[1] + ", var(--id-accent) 60%, transparent 100%)";
            }).join(",") + ",var(--id-paper);opacity:0.6;";
            return;
        }
        var pat = PATTERNS.filter(function (p) { return p.id === patId; })[0];
        if (pat && pat.css) { div.style.cssText = pat.css + "opacity:0.3;"; }
    };

    A.bootPattern = function () {
        var id = getCurrentPattern();
        applyPattern(id);
        applySliders();
    };

    /* Expose for 09_settings.js */
    A._patterns = PATTERNS;
    A._getCurrentPattern = getCurrentPattern;
    A._setCurrentPattern = setCurrentPattern;
    A._applyPattern = applyPattern;
    A._applyPatternCSS = applyPatternCSS;
})();
