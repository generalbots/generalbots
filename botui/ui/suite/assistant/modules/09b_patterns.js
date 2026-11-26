"use strict";
/**
 * 09b_patterns.js - Chat background pattern data and logic.
 * Split from 09_settings.js to stay under the 450-line limit.
 * Exposes A._patterns, A._getCurrentPattern, A._setCurrentPattern,
 * A._applyPattern, A._applyPatternCSS, A.bootPattern on GBAssistant.
 */

(function () {
    var A = window.GBAssistant;
    var THEME_KEYS = { light: "settings-theme-light", dark: "settings-theme-dark", system: "settings-theme-system" };
    var TEXT_KEYS = { small: "settings-text-small", "default": "settings-text-default", large: "settings-text-large", xlarge: "settings-text-xlarge" };
    var root = document.documentElement;
    var mantis = root.dataset.identity === "mantis";

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

    var MANTIS_PATTERNS = [
        { id: "none", en: "None", pt: "Nenhum" },
        { id: "dots", en: "Dots", pt: "Pontos" },
        { id: "microdots", en: "Micro", pt: "Micro" },
        { id: "grid", en: "Grid", pt: "Grade" },
        { id: "blueprint", en: "Blueprint", pt: "Planta" },
        { id: "weave", en: "Weave", pt: "Tecido" },
        { id: "mosaic", en: "Mosaic", pt: "Mosaico" }
    ];

    function getCurrentPattern() {
        var stored;
        try { stored = localStorage.getItem("gb-chat-pattern"); } catch (e) { /* storage unavailable */ }
        if (mantis) {
            return MANTIS_PATTERNS.some(function (pattern) { return pattern.id === stored; }) ? stored : "dots";
        }
        return stored || "image";
    }
    function setCurrentPattern(id) {
        try { localStorage.setItem("gb-chat-pattern", id); } catch (e) { /* */ }
    }

    function applyPattern(id) {
        if (mantis) {
            root.dataset.gbPattern = id === "image" ? "dots" : id;
            return;
        }
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
        if (mantis) {
            root.style.setProperty("--gb-pat-strength", String(getSlider("strength") / 100));
            root.style.setProperty("--gb-pat-scale", String(getSlider("scale") / 100));
            return;
        }
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
        panel.querySelectorAll(".gb-set-range[data-slider]").forEach(function (input) {
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

    function seg(name, items, selected) {
        return '<div class="gb-ap-seg" style="grid-template-columns:repeat(' + items.length + ',1fr)">' + items.map(function (item) {
            return '<button type="button" data-' + name + '="' + item[0] + '" aria-pressed="' + (selected === item[0]) + '">' + item[1] + '</button>';
        }).join("") + '</div>';
    }

    function syncChoices(panel, type, selected) {
        panel.querySelectorAll("[data-" + type + "]").forEach(function (button) {
            button.setAttribute("aria-pressed", String(button.getAttribute("data-" + type) === selected));
        });
    }

    function saveAccent(hex) {
        try { localStorage.setItem("gb-accent", hex); } catch (e) { /* storage unavailable */ }
        if (A.apiFetch) {
            A.apiFetch("/api/appearance", {
                method: "PUT",
                body: JSON.stringify({ accent: hex })
            }).catch(function () { /* The device preference is already saved. */ });
        }
    }

    function sliderRow(name, label) {
        var cfg = SLIDERS[name], v = getSlider(name), id = "gbAppearanceRange_" + name;
        return '<div class="gb-ap-slider"><label class="gb-ap-label" for="' + id + '">' + label + '</label>' +
            '<input type="range" class="gb-set-range" id="' + id + '" data-slider="' + name + '" min="' + cfg.min + '" max="' + cfg.max +
            '" step="' + cfg.step + '" value="' + v + '"><output for="' + id + '">' + v + '%</output><button type="button" data-reset="' + name +
            '" title="' + A.t("settings.slider-reset", "Default") + '" aria-label="' + A.t("settings.slider-reset", "Default") + '">' +
            RESET_ICON + '</button></div>';
    }
    var RESET_ICON = '<svg aria-hidden="true" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 1 0 3-6.7L3 8M3 3v5h5"/></svg>';

    /* App zoom (identity/zoom.js): the value follows the slider while it moves
       and the page rescales when it is released, so the thumb stays under the pointer. */
    function zoomRow(label) {
        var z = window.GBZoom, reset = A.t("settings.slider-reset", "Default");
        return '<div class="gb-ap-sec gb-ap-zoom"><label class="gb-ap-label" for="gbAppearanceZoom">' + label + '</label>' +
            '<div class="gb-ap-slider"><input type="range" class="gb-set-range" id="gbAppearanceZoom" min="' + z.min + '" max="' + z.max +
            '" step="' + z.increment + '" value="' + z.get() + '"><output for="gbAppearanceZoom">' + z.get() + '%</output>' +
            '<button type="button" data-zoom-reset title="' + reset + '" aria-label="' + reset + '">' + RESET_ICON + '</button></div></div>';
    }
    function showZoom(value) {
        var input = document.getElementById("gbAppearanceZoom");
        if (!input) { return; }
        input.value = value;
        input.nextElementSibling.textContent = value + "%";
    }
    function bindZoom(panel) {
        var input = panel.querySelector("#gbAppearanceZoom");
        if (!input) { return; }
        input.addEventListener("input", function () { input.nextElementSibling.textContent = input.value + "%"; });
        input.addEventListener("change", function () { window.GBZoom.set(input.value); });
        panel.querySelector("[data-zoom-reset]").addEventListener("click", function () { window.GBZoom.reset(); });
    }
    window.addEventListener("gb-zoom-change", function (event) { showZoom(event.detail.value); });

    A.renderMantisAppearance = function (container) {
        var theme = window.GBTheme;
        var size = window.GBTextSize;
        var identity = window.GBIdentity;
        var portuguese = root.lang.toLowerCase().startsWith("pt");
        function tr(pt, en) { return portuguese ? pt : en; }
        var activeAccent = root.style.getPropertyValue("--id-accent").trim().toUpperCase();
        var hue = Math.round(identity.hueOf(activeAccent)) % 360;
        var pattern = getCurrentPattern();
        function accentName(hex) {
            var preset = identity.presets.find(function (item) { return item.hex.toUpperCase() === String(hex).toUpperCase(); });
            return preset ? (portuguese ? preset.pt : preset.en) : tr("Personalizado", "Custom");
        }
        var track = "linear-gradient(90deg, " + Array.from({ length: 13 }, function (_, i) {
            return identity.fromHue(i * 30) + " " + (i / 12 * 100).toFixed(1) + "%";
        }).join(", ") + ")";
        var px = { small: 14, "default": 16, large: 18, xlarge: 20 };
        var short = { small: tr("P", "S"), "default": tr("M", "M"), large: tr("G", "L"), xlarge: tr("XG", "XL") };
        container.innerHTML = '<div class="gb-ap">' +
            '<div class="gb-ap-sec"><span class="gb-ap-label">' + tr("Tema", "Theme") + '</span>' +
            seg("appearance-theme", ["light", "dark", "system"].map(function (id) { return [id, A.escapeHtml(A.t(THEME_KEYS[id] || "settings-theme-light", id))]; }), theme.get()) + '</div>' +
            (window.GBZoom ? zoomRow(tr("Zoom do aplicativo", "App zoom")) : '') +
            '<div class="gb-ap-sec"><div class="gb-ap-label gb-ap-pair"><span>' + tr("Acento", "Accent") + '</span><span class="gb-ap-ink" id="gbAppearanceAccentName">' +
            A.escapeHtml(accentName(activeAccent)) + '</span></div><div class="gb-ap-swatches">' + identity.presets.map(function (preset) {
                var label = A.escapeHtml(portuguese ? preset.pt : preset.en);
                return '<button type="button" data-acc="' + preset.hex + '" title="' + label + '" aria-label="' + label + '" aria-pressed="' +
                    (preset.hex.toUpperCase() === activeAccent) + '"><i style="background:' + preset.hex + '"></i></button>';
            }).join("") + '</div><div class="gb-ap-hue"><div class="gb-ap-label gb-ap-pair gb-ap-small"><span>' + tr("Matiz", "Hue") +
            '</span><span class="gb-ap-ink"><output for="gbAppearanceHue">' + hue + '°</output></span></div><div class="gb-ap-track" style="--gb-hue-track:' +
            track + ';--gb-hue-pct:' + (hue / 359 * 100).toFixed(1) + '%"><i class="gb-ap-rail"></i><i class="gb-ap-thumb"></i>' +
            '<input type="range" id="gbAppearanceHue" min="0" max="359" value="' + hue + '" aria-label="' + tr("Matiz do acento", "Accent hue") + '"></div>' +
            '<span class="gb-ap-hint">' + tr("Muda só a cor. O brilho e a intensidade ficam fixos para manter o contraste.",
                "Changes only the colour. Brightness and intensity stay fixed to keep the contrast.") + '</span></div></div>' +
            '<div class="gb-ap-sec"><span class="gb-ap-label">' + tr("Fundo", "Background") + '</span><div class="gb-ap-pats">' +
            MANTIS_PATTERNS.map(function (item) {
                return '<button type="button" data-gb-pick="' + item.id + '" aria-pressed="' + (pattern === item.id) + '"><i class="gb-ap-tile" data-gb-pattern="' +
                    item.id + '"></i><span>' + A.escapeHtml(portuguese ? item.pt : item.en) + '</span></button>';
            }).join("") + '</div></div>' +
            '<div class="gb-ap-sliders">' + sliderRow("strength", tr("Opacidade", "Opacity")) + sliderRow("scale", tr("Escala", "Scale")) + '</div>' +
            '<div class="gb-ap-sec"><span class="gb-ap-label">' + tr("Tamanho do texto", "Text size") + '</span>' +
            seg("appearance-size", size.steps.map(function (id) {
                return [id, '<span aria-hidden="true">' + short[id] + '</span><small aria-hidden="true">' + (px[id] || "") +
                    '</small><span class="gb-sr-only">' + A.escapeHtml(A.t(TEXT_KEYS[id] || "settings-text-default", id)) + '</span>'];
            }), size.get()) + '</div></div>';

        var panel = container.querySelector(".gb-ap");
        var hueInput = panel.querySelector("#gbAppearanceHue");
        var hueOutput = panel.querySelector('output[for="gbAppearanceHue"]');
        var hueTrack = panel.querySelector(".gb-ap-track");
        function showAccent(accent) {
            hueInput.value = Math.round(identity.hueOf(accent)) % 360;
            hueOutput.value = hueInput.value + "°";
            hueTrack.style.setProperty("--gb-hue-pct", (hueInput.value / 359 * 100).toFixed(1) + "%");
            panel.querySelector("#gbAppearanceAccentName").textContent = accentName(accent);
            syncChoices(panel, "acc", accent);
        }
        panel.addEventListener("click", function (event) {
            var button = event.target.closest("button");
            if (!button) { return; }
            var data = button.dataset;
            if (data.appearanceTheme) {
                theme.set(data.appearanceTheme);
                syncChoices(panel, "appearance-theme", data.appearanceTheme);
            } else if (data.appearanceSize) {
                size.set(data.appearanceSize);
                syncChoices(panel, "appearance-size", data.appearanceSize);
            } else if (data.acc) {
                var accent = identity.apply(data.acc).accent;
                showAccent(accent);
                saveAccent(accent);
            } else if (data.gbPick) {
                setCurrentPattern(data.gbPick);
                applyPattern(data.gbPick);
                syncChoices(panel, "gb-pick", data.gbPick);
                A._syncSliders(panel);
            }
        });
        hueInput.addEventListener("input", function () {
            var accent = identity.apply(identity.fromHue(Number(hueInput.value))).accent;
            showAccent(accent);
            try { localStorage.setItem("gb-accent", accent); } catch (e) { /* storage unavailable */ }
        });
        hueInput.addEventListener("change", function () {
            saveAccent(root.style.getPropertyValue("--id-accent"));
        });
        A._bindSliders(panel);
        A._syncSliders(panel);
        bindZoom(panel);
    };

    if (mantis) { A.bootPattern(); }
})();
