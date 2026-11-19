"use strict";
/* Slides animation — selected-effect parameter editor (#2431).
 * Split out of the timeline panel to keep modules focused. Loaded after
 * 16b_anim_motion.js, before 16_anim_timeline.js. */

(function (window) {
  var A = window.SlidesAnimation;
  var E = window.SlidesAnimationEngine;
  var M = window.SlidesAnimationMotion;
  if (!A || !E) return;

  function t(key, fallback, params) { return A.saT(key, fallback, params); }
  function el(tag, cls, text) {
    var n = document.createElement(tag);
    if (cls) n.className = cls;
    if (text !== undefined) n.textContent = text;
    return n;
  }
  function button(label, cls) {
    var b = el("button", "sl-anim-btn" + (cls ? " " + cls : ""), label);
    b.type = "button";
    return b;
  }
  function labelledSelect(labelKey, labelFallback, options) {
    var wrap = el("label", "sl-anim-field");
    wrap.appendChild(el("span", "sl-anim-label", t(labelKey, labelFallback)));
    var sel = el("select", "sl-anim-select");
    for (var i = 0; i < options.length; i++) {
      var o = el("option", "", t(options[i].key, options[i].fallback));
      o.value = options[i].value;
      sel.appendChild(o);
    }
    wrap.appendChild(sel);
    wrap._select = sel;
    return wrap;
  }
  function numberField(key, fallback, value, min, max) {
    var wrap = el("label", "sl-anim-field sl-anim-num");
    wrap.appendChild(el("span", "sl-anim-label", t(key, fallback)));
    var inp = el("input", "sl-anim-input");
    inp.type = "number";
    inp.min = String(min);
    inp.max = String(max);
    inp.value = String(value);
    wrap.appendChild(inp);
    wrap.appendChild(el("span", "sl-anim-unit", t("slides-anim-ms", "ms")));
    wrap._input = inp;
    return wrap;
  }
  var EASINGS = [
    { value: "linear", key: "slides-anim-ease-linear", fallback: "Linear" },
    { value: "ease", key: "slides-anim-ease-ease", fallback: "Ease" },
    { value: "ease-in", key: "slides-anim-ease-in", fallback: "Ease in" },
    { value: "ease-out", key: "slides-anim-ease-out", fallback: "Ease out" },
    { value: "ease-in-out", key: "slides-anim-ease-in-out", fallback: "Ease in and out" }
  ];
  var STARTS = [
    { value: "click", key: "slides-anim-start-click", fallback: "On click" },
    { value: "with", key: "slides-anim-start-with", fallback: "With previous" },
    { value: "after", key: "slides-anim-start-after", fallback: "After previous" }
  ];
  var DIRECTIONS = [
    { value: "left", key: "slides-anim-dir-left", fallback: "Left" },
    { value: "right", key: "slides-anim-dir-right", fallback: "Right" },
    { value: "up", key: "slides-anim-dir-up", fallback: "Up" },
    { value: "down", key: "slides-anim-dir-down", fallback: "Down" }
  ];

  function render(host, fx) {
    if (!host) return;
    host.textContent = "";
    if (!fx) { host.hidden = true; return; }
    host.hidden = false;
    host.appendChild(el("h3", "sl-anim-editor-title", A.effectName(fx)));
    function patch(key, value) { var p = {}; p[key] = value; A.updateEffect(fx.id, p); }
    var duration = numberField("slides-anim-duration", "Duration", fx.duration, 0, 10000);
    duration._input.addEventListener("change", function () { patch("duration", Number(duration._input.value) || 0); });
    var delay = numberField("slides-anim-delay", "Delay", fx.delay, 0, 10000);
    delay._input.addEventListener("change", function () { patch("delay", Number(delay._input.value) || 0); });
    var easing = labelledSelect("slides-anim-easing", "Easing", EASINGS);
    easing._select.value = fx.easing;
    easing._select.addEventListener("change", function () { patch("easing", easing._select.value); });
    var start = labelledSelect("slides-anim-start", "Start", STARTS);
    start._select.value = fx.start;
    start._select.addEventListener("change", function () { patch("start", start._select.value); });
    host.appendChild(duration); host.appendChild(delay); host.appendChild(easing); host.appendChild(start);
    if (fx.effect === "fly" || fx.effect === "fly-out" || fx.effect === "wipe" || fx.category === "motion") {
      var dir = labelledSelect("slides-anim-direction", "Direction", DIRECTIONS);
      dir._select.value = fx.direction || "left";
      dir._select.addEventListener("change", function () { patch("direction", dir._select.value); });
      host.appendChild(dir);
    }
    if (fx.category === "motion" && M) host.appendChild(M.editor(fx, function (path) { patch("path", path); }));
    if (fx.category === "build") {
      var levels = el("label", "sl-anim-field");
      levels.appendChild(el("span", "sl-anim-label", t("slides-anim-levels", "Paragraph levels")));
      var levelsInput = el("input", "sl-anim-input");
      levelsInput.type = "text";
      levelsInput.value = (fx.levels || []).join(",");
      levelsInput.addEventListener("change", function () {
        var parts = String(levelsInput.value).split(",");
        var nums = [];
        for (var i = 0; i < parts.length; i++) {
          var n = parseInt(parts[i], 10);
          if (isFinite(n) && n >= 0) nums.push(n);
        }
        patch("levels", nums);
      });
      levels.appendChild(levelsInput);
      host.appendChild(levels);
    }
    var preview = button(t("slides-anim-preview", "Preview"), "sl-anim-preview-btn");
    preview.addEventListener("click", function () { E.play([fx]); });
    host.appendChild(preview);
  }

  window.SlidesAnimationEditor = { render: render };
})(window);
