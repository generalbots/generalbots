"use strict";
/* Slides animation/transition — bootstrap (#2431).
 * Adds two toolbar entry points without replacing the committed Insert hub or
 * the existing htmx animation/transition buttons. */

(function (window) {
  var A = window.SlidesAnimation;
  var E = window.SlidesAnimationEngine;
  var TL = window.SlidesAnimationTimeline;
  var TR = window.SlidesAnimationTransitions;
  if (!A || !E) return;

  function t(key, fallback) { return A.saT(key, fallback); }
  function icon(path) {
    var svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("width", "16");
    svg.setAttribute("height", "16");
    svg.setAttribute("viewBox", "0 0 24 24");
    svg.setAttribute("fill", "none");
    svg.setAttribute("stroke", "currentColor");
    svg.setAttribute("stroke-width", "2");
    svg.setAttribute("aria-hidden", "true");
    for (var i = 0; i < path.length; i++) {
      var p = document.createElementNS("http://www.w3.org/2000/svg", "path");
      p.setAttribute("d", path[i]);
      svg.appendChild(p);
    }
    return svg;
  }
  function makeButton(id, labelKey, labelFallback, ariaKey, ariaFallback, glyph, onClick) {
    var b = document.createElement("button");
    b.id = id;
    b.type = "button";
    b.className = "btn-icon sl-anim-launch";
    b.title = t(ariaKey, ariaFallback);
    b.setAttribute("aria-label", t(ariaKey, ariaFallback));
    b.setAttribute("data-i18n-title", ariaKey);
    b.setAttribute("data-i18n-aria-label", ariaKey);
    b.appendChild(icon(glyph));
    var label = document.createElement("span");
    label.setAttribute("data-i18n", labelKey);
    label.textContent = t(labelKey, labelFallback);
    b.appendChild(label);
    b.addEventListener("click", onClick);
    return b;
  }
  function installLaunchers() {
    if (document.getElementById("sl-anim-launch")) return;
    var center = document.querySelector(".slides-toolbar-row-main .toolbar-center");
    if (!center) return;
    var group = document.createElement("div");
    group.className = "toolbar-group sl-anim-launch-group";
    var anim = makeButton("sl-anim-launch", "slides-anim-toolbar", "Animation",
      "slides-anim-open-aria", "Open the animation timeline",
      ["M5 3l14 9-14 9V3z"], function () { if (TL) TL.open(); });
    var trans = makeButton("sl-trans-launch", "slides-trans-toolbar", "Transitions",
      "slides-trans-open-aria", "Open slide transitions",
      ["M13 17l5-5-5-5", "M6 17l5-5-5-5"], function () { if (TR) TR.open(); });
    group.appendChild(anim);
    group.appendChild(trans);
    var right = center.querySelector(".toolbar-right");
    if (right) center.insertBefore(group, right);
    else center.appendChild(group);
  }
  function hookPresenter() {
    var btn = document.getElementById("presenterBtn");
    if (!btn || btn.dataset.saPresentHook) return;
    btn.dataset.saPresentHook = "1";
    btn.addEventListener("click", function () {
      window.setTimeout(function () {
        E.play(A.effects()).catch(function () { E.revealAll(A.effects()); });
      }, 50);
    });
  }
  function boot() {
    installLaunchers();
    E.hookSlideChanges();
    hookPresenter();
  }
  window.SlidesAnimationUI = {
    openTimeline: function () { if (TL) TL.open(); },
    openTransitions: function () { if (TR) TR.open(); }
  };
  window.addEventListener("localeChanged", function () {
    var a = document.getElementById("sl-anim-launch");
    var tr = document.getElementById("sl-trans-launch");
    if (a) {
      a.title = t("slides-anim-open-aria", "Open the animation timeline");
      a.setAttribute("aria-label", a.title);
    }
    if (tr) {
      tr.title = t("slides-trans-open-aria", "Open slide transitions");
      tr.setAttribute("aria-label", tr.title);
    }
  });
  if (document.readyState === "loading") document.addEventListener("DOMContentLoaded", boot);
  else boot();
})(window);
