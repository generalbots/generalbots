"use strict";
/* Slides animation — transitions gallery panel (#2431).
 * Duration/direction/easing/advance, apply-to-all and per-slide override.
 * All strings via saT(); no native dialogs; focus-trapped, Esc to close. */

(function (window) {
  var A = window.SlidesAnimation;
  var E = window.SlidesAnimationEngine;
  if (!A || !E) return;
  var panel = null;
  var previousFocus = null;
  var draft = null;

  function t(key, fallback) { return A.saT(key, fallback); }
  function el(tag, cls, text) {
    var n = document.createElement(tag);
    if (cls) n.className = cls;
    if (text !== undefined) n.textContent = text;
    return n;
  }
  function button(label, ariaKey, ariaFallback, cls) {
    var b = el("button", "sl-trans-btn" + (cls ? " " + cls : ""), label);
    b.type = "button";
    if (ariaKey) b.setAttribute("aria-label", t(ariaKey, ariaFallback));
    return b;
  }
  function selectField(labelKey, labelFallback, options) {
    var wrap = el("label", "sl-trans-field");
    wrap.appendChild(el("span", "sl-trans-label", t(labelKey, labelFallback)));
    var sel = el("select", "sl-trans-select");
    for (var i = 0; i < options.length; i++) {
      var o = el("option", "", t(options[i].key, options[i].fallback));
      o.value = options[i].value;
      sel.appendChild(o);
    }
    wrap.appendChild(sel);
    wrap._select = sel;
    return wrap;
  }
  function focusables() {
    if (!panel) return [];
    return Array.prototype.slice.call(panel.querySelectorAll("button,select,input"))
      .filter(function (n) { return !n.disabled && n.offsetParent !== null; });
  }
  function syncDraftFromModel() {
    var cfg = A.transition(A.slideIndex());
    draft = {};
    for (var k in cfg) if (Object.prototype.hasOwnProperty.call(cfg, k)) draft[k] = cfg[k];
  }
  function build() {
    if (panel) return panel;
    panel = el("div", "sl-trans-panel");
    panel.id = "sl-trans-panel";
    panel.setAttribute("role", "dialog");
    panel.setAttribute("aria-label", t("slides-trans-title", "Slide transitions"));
    panel.hidden = true;

    var head = el("div", "sl-trans-head");
    head.appendChild(el("h2", "sl-trans-title", t("slides-trans-title", "Slide transitions")));
    var close = button("×", "slides-anim-close", "Close", "sl-trans-close");
    head.appendChild(close);
    panel.appendChild(head);

    var body = el("div", "sl-trans-body");
    body.appendChild(el("div", "sl-trans-section-title", t("slides-trans-gallery", "Transition gallery")));
    var gallery = el("div", "sl-trans-grid");
    gallery.id = "sl-trans-gallery";
    body.appendChild(gallery);

    var duration = el("label", "sl-trans-field sl-trans-num");
    duration.appendChild(el("span", "sl-trans-label", t("slides-anim-duration", "Duration")));
    var durationInput = el("input", "sl-trans-input");
    durationInput.type = "number"; durationInput.min = "0"; durationInput.max = "10000";
    durationInput.id = "sl-trans-duration";
    duration.appendChild(durationInput);
    duration.appendChild(el("span", "sl-trans-unit", t("slides-anim-ms", "ms")));
    body.appendChild(duration);

    var direction = selectField("slides-anim-direction", "Direction", [
      { value: "left", key: "slides-anim-dir-left", fallback: "Left" },
      { value: "right", key: "slides-anim-dir-right", fallback: "Right" },
      { value: "up", key: "slides-anim-dir-up", fallback: "Up" },
      { value: "down", key: "slides-anim-dir-down", fallback: "Down" }
    ]);
    direction.id = "sl-trans-direction";
    body.appendChild(direction);

    var easing = selectField("slides-anim-easing", "Easing", [
      { value: "linear", key: "slides-anim-ease-linear", fallback: "Linear" },
      { value: "ease", key: "slides-anim-ease-ease", fallback: "Ease" },
      { value: "ease-in", key: "slides-anim-ease-in", fallback: "Ease in" },
      { value: "ease-out", key: "slides-anim-ease-out", fallback: "Ease out" },
      { value: "ease-in-out", key: "slides-anim-ease-in-out", fallback: "Ease in and out" }
    ]);
    easing.id = "sl-trans-easing";
    body.appendChild(easing);

    var advance = selectField("slides-trans-advance", "Advance slide", [
      { value: "click", key: "slides-trans-advance-click", fallback: "On click" },
      { value: "time", key: "slides-trans-advance-time", fallback: "After time" }
    ]);
    advance.id = "sl-trans-advance";
    body.appendChild(advance);

    var seconds = el("label", "sl-trans-field sl-trans-num");
    seconds.appendChild(el("span", "sl-trans-label", t("slides-trans-advance-time", "After time")));
    var secondsInput = el("input", "sl-trans-input");
    secondsInput.type = "number"; secondsInput.min = "0"; secondsInput.max = "3600";
    secondsInput.id = "sl-trans-seconds";
    seconds.appendChild(secondsInput);
    body.appendChild(seconds);

    var flags = el("div", "sl-trans-flags");
    var all = el("label", "sl-anim-check");
    var allInput = el("input"); allInput.type = "checkbox"; allInput.id = "sl-trans-all";
    all.appendChild(allInput);
    all.appendChild(el("span", "", t("slides-trans-apply-all", "Apply to all slides")));
    var override = el("label", "sl-anim-check");
    var overrideInput = el("input"); overrideInput.type = "checkbox"; overrideInput.id = "sl-trans-override";
    override.appendChild(overrideInput);
    override.appendChild(el("span", "", t("slides-trans-override", "Override for this slide")));
    flags.appendChild(all); flags.appendChild(override);
    body.appendChild(flags);

    var actions = el("div", "sl-trans-actions");
    var preview = button(t("slides-trans-preview", "Preview transition"), null, null);
    preview.id = "sl-trans-preview";
    actions.appendChild(preview);
    body.appendChild(actions);
    panel.appendChild(body);

    var status = el("div", "sl-anim-status");
    status.id = "sl-trans-status";
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    status.textContent = t("slides-anim-status-ready", "Ready");
    panel.appendChild(status);
    document.body.appendChild(panel);

    close.addEventListener("click", closePanel);
    panel.addEventListener("keydown", function (ev) {
      if (ev.key === "Escape") { ev.preventDefault(); closePanel(); return; }
      if (ev.key !== "Tab") return;
      var f = focusables();
      if (!f.length) return;
      if (ev.shiftKey && document.activeElement === f[0]) { ev.preventDefault(); f[f.length - 1].focus(); }
      else if (!ev.shiftKey && document.activeElement === f[f.length - 1]) { ev.preventDefault(); f[0].focus(); }
    });

    function changed() {
      if (!draft) return;
      draft.duration = Number(durationInput.value) || 0;
      draft.direction = direction._select.value;
      draft.easing = easing._select.value;
      draft.advance = advance._select.value;
      draft.seconds = Number(secondsInput.value) || 0;
      seconds.hidden = draft.advance !== "time";
      A.setTransition(draft, A.slideIndex(), allInput.checked);
      E.status(t(allInput.checked ? "slides-trans-status-all" : "slides-trans-status-applied",
        allInput.checked ? "Transition applied to all slides" : "Transition applied to this slide"));
    }
    durationInput.addEventListener("change", changed);
    direction._select.addEventListener("change", changed);
    easing._select.addEventListener("change", changed);
    advance._select.addEventListener("change", changed);
    secondsInput.addEventListener("change", changed);
    allInput.addEventListener("change", changed);
    overrideInput.addEventListener("change", function () {
      if (!overrideInput.checked) { delete A.slideData(A.slideIndex(), true).transition; A.save(); }
      else changed();
      E.status(t("slides-trans-status-applied", "Transition applied to this slide"));
    });
    preview.addEventListener("click", function () {
      E.playTransition(A.canvas(), draft);
      E.status(t("slides-trans-status-preview", "Previewing transition"));
    });
    return panel;
  }

  function renderGallery() {
    var host = document.getElementById("sl-trans-gallery");
    if (!host) return;
    host.textContent = "";
    for (var i = 0; i < A.TRANSITIONS.length; i++) {
      (function (def) {
        var b = button(t(def.key, def.fallback), null, null, def.id === draft.type ? "selected" : "");
        b.dataset.type = def.id;
        b.addEventListener("click", function () {
          draft.type = def.id;
          var allInput = document.getElementById("sl-trans-all");
          A.setTransition(draft, A.slideIndex(), allInput && allInput.checked);
          renderGallery();
          E.status(t(allInput && allInput.checked ? "slides-trans-status-all" : "slides-trans-status-applied",
            allInput && allInput.checked ? "Transition applied to all slides" : "Transition applied to this slide"));
        });
        host.appendChild(b);
      })(A.TRANSITIONS[i]);
    }
  }
  function refresh() {
    if (!panel || panel.hidden) return;
    syncDraftFromModel();
    var d = document.getElementById("sl-trans-duration");
    if (d) d.value = String(draft.duration);
    var dir = document.getElementById("sl-trans-direction");
    if (dir && dir._select) dir._select.value = draft.direction;
    var ez = document.getElementById("sl-trans-easing");
    if (ez && ez._select) ez._select.value = draft.easing;
    var adv = document.getElementById("sl-trans-advance");
    if (adv && adv._select) adv._select.value = draft.advance;
    var sec = document.getElementById("sl-trans-seconds");
    if (sec) { sec.value = String(draft.seconds); sec.hidden = draft.advance !== "time"; }
    var ov = document.getElementById("sl-trans-override");
    if (ov) ov.checked = !!(A.slideData(A.slideIndex(), false).transition);
    renderGallery();
  }
  function openPanel() {
    build();
    if (!panel) return;
    previousFocus = document.activeElement;
    syncDraftFromModel();
    panel.hidden = false;
    refresh();
    var f = focusables()[0];
    if (f && f.focus) f.focus();
    E.status(t("slides-anim-status-local-only", "Stored locally — server persistence is not available yet"));
  }
  function closePanel() {
    if (!panel) return;
    panel.hidden = true;
    if (previousFocus && previousFocus.focus) previousFocus.focus();
    previousFocus = null;
  }
  A.on("transition", refresh);
  window.addEventListener("localeChanged", function () {
    if (!panel) return;
    var h = panel.querySelector(".sl-trans-title");
    if (h) h.textContent = t("slides-trans-title", "Slide transitions");
    refresh();
  });

  window.SlidesAnimationTransitions = { open: openPanel, close: closePanel, refresh: refresh, get isOpen() { return !!panel && !panel.hidden; } };
})(window);
