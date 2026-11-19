"use strict";
/* Slides animation — per-object timeline panel (#2431).
 * Ordered effect list, drag/keyboard reorder, parameter editor, preview scrubber.
 * Plain script; all visible strings come from saT(). */

(function (window) {
  var A = window.SlidesAnimation;
  var E = window.SlidesAnimationEngine;
  if (!A || !E) return;
  var panel = null;
  var previousFocus = null;
  var selectedId = null;
  var activeCategory = "entrance";
  var objectId = null;
  var START_KEYS = {
    click: "slides-anim-start-click",
    with: "slides-anim-start-with",
    after: "slides-anim-start-after"
  };

  function t(key, fallback, params) { return A.saT(key, fallback, params); }
  function el(tag, cls, text) {
    var n = document.createElement(tag);
    if (cls) n.className = cls;
    if (text !== undefined) n.textContent = text;
    return n;
  }
  function button(label, ariaKey, ariaFallback, cls) {
    var b = el("button", "sl-anim-btn" + (cls ? " " + cls : ""), label);
    b.type = "button";
    if (ariaKey) b.setAttribute("aria-label", t(ariaKey, ariaFallback));
    return b;
  }
  function currentFx() {
    var list = A.effects();
    for (var i = 0; i < list.length; i++) if (list[i].id === selectedId) return list[i];
    return null;
  }
  function focusables() {
    if (!panel) return [];
    return Array.prototype.slice.call(panel.querySelectorAll("button,select,input,[tabindex='0']"))
      .filter(function (n) { return !n.disabled && n.offsetParent !== null; });
  }

  function build() {
    if (panel) return panel;
    panel = el("div", "sl-anim-panel");
    panel.id = "sl-anim-panel";
    panel.setAttribute("role", "dialog");
    panel.setAttribute("aria-label", t("slides-anim-title", "Animation timeline"));
    panel.hidden = true;

    var head = el("div", "sl-anim-head");
    head.appendChild(el("h2", "sl-anim-title", t("slides-anim-title", "Animation timeline")));
    var reduce = el("label", "sl-anim-check sl-anim-reduce");
    var reduceInput = el("input");
    reduceInput.type = "checkbox";
    reduceInput.id = "sl-anim-reduce";
    reduceInput.checked = !!A.state.reduceMotion;
    reduce.appendChild(reduceInput);
    reduce.appendChild(el("span", "", t("slides-anim-reduce-deck", "Reduce motion for this deck")));
    head.appendChild(reduce);
    var close = button("×", "slides-anim-close", "Close", "sl-anim-close");
    close.id = "sl-anim-close";
    head.appendChild(close);
    panel.appendChild(head);

    var body = el("div", "sl-anim-body");
    var objRow = el("div", "sl-anim-row");
    objRow.appendChild(el("span", "sl-anim-label", t("slides-anim-object", "Object")));
    var objSel = el("select", "sl-anim-select");
    objSel.id = "sl-anim-object";
    objSel.setAttribute("aria-label", t("slides-anim-object", "Object"));
    objRow.appendChild(objSel);
    body.appendChild(objRow);

    var listHead = el("div", "sl-anim-section-title", t("slides-anim-effects", "Effects"));
    body.appendChild(listHead);
    var list = el("div", "sl-anim-list");
    list.id = "sl-anim-list";
    list.setAttribute("role", "listbox");
    list.setAttribute("aria-label", t("slides-anim-effects", "Effects"));
    body.appendChild(list);

    var preview = el("div", "sl-anim-preview");
    var controls = el("div", "sl-anim-controls");
    var play = button(t("slides-anim-play", "Play"), null, null);
    play.id = "sl-anim-play";
    var pause = button(t("slides-anim-pause", "Pause"), null, null);
    pause.id = "sl-anim-pause";
    var reset = button(t("slides-anim-reset", "Reset"), null, null);
    reset.id = "sl-anim-reset";
    controls.appendChild(play); controls.appendChild(pause); controls.appendChild(reset);
    preview.appendChild(controls);
    var scrub = el("input", "sl-anim-scrubber");
    scrub.type = "range";
    scrub.id = "sl-anim-scrubber";
    scrub.min = "0"; scrub.max = "100"; scrub.value = "0";
    scrub.setAttribute("aria-label", t("slides-anim-scrubber", "Build scrubber"));
    preview.appendChild(scrub);
    var time = el("span", "sl-anim-time", "0 / 0");
    time.id = "sl-anim-time";
    preview.appendChild(time);
    body.appendChild(preview);

    var tabs = el("div", "sl-anim-tabs");
    tabs.setAttribute("role", "tablist");
    tabs.id = "sl-anim-tabs";
    body.appendChild(tabs);
    var grid = el("div", "sl-anim-grid");
    grid.id = "sl-anim-grid";
    body.appendChild(grid);
    var editor = el("div", "sl-anim-editor");
    editor.id = "sl-anim-editor";
    editor.hidden = true;
    body.appendChild(editor);
    panel.appendChild(body);

    var status = el("div", "sl-anim-status");
    status.id = "sl-anim-status";
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    status.textContent = t("slides-anim-status-ready", "Ready");
    panel.appendChild(status);
    document.body.appendChild(panel);

    reduceInput.addEventListener("change", function () {
      A.setReduceMotion(reduceInput.checked);
      E.status(t("slides-anim-status-local-only", "Stored locally — server persistence is not available yet"));
    });
    close.addEventListener("click", closePanel);
    panel.addEventListener("keydown", function (ev) {
      if (ev.key === "Escape") { ev.preventDefault(); closePanel(); return; }
      if (ev.key !== "Tab") return;
      var f = focusables();
      if (!f.length) return;
      var first = f[0], last = f[f.length - 1];
      if (ev.shiftKey && document.activeElement === first) { ev.preventDefault(); last.focus(); }
      else if (!ev.shiftKey && document.activeElement === last) { ev.preventDefault(); first.focus(); }
    });
    objSel.addEventListener("change", function () { objectId = objSel.value || null; selectedId = null; refresh(); });
    play.addEventListener("click", function () { E.play(A.effects()).then(function () { updateTime(E.total); }); });
    pause.addEventListener("click", function () { E.stop(); });
    reset.addEventListener("click", function () { E.stop(); E.seek(0, A.effects()); E.revealAll(A.effects()); updateTime(0); });
    scrub.addEventListener("input", function () {
      E.stop();
      var total = E.totalOf(A.effects());
      var ms = total * (Number(scrub.value) / 100);
      E.seek(ms, A.effects());
      updateTime(ms);
    });
    return panel;
  }

  function updateTime(ms) {
    var node = document.getElementById("sl-anim-time");
    if (!node) return;
    var unit = t("slides-anim-ms", "ms");
    node.textContent = Math.round(ms || 0) + " / " + Math.round(E.totalOf(A.effects())) + " " + unit;
  }
  function renderObjects() {
    var sel = document.getElementById("sl-anim-object");
    if (!sel) return;
    var old = sel.value;
    sel.textContent = "";
    var els = A.elements();
    if (!els.length) {
      var empty = el("option", "", t("slides-anim-object-none", "No objects on this slide"));
      empty.value = "";
      sel.appendChild(empty);
      objectId = null;
      return;
    }
    for (var i = 0; i < els.length; i++) {
      var o = el("option", "", (els[i].dataset.type || t("slides-anim-object", "Object")) + " · " + els[i].dataset.id);
      o.value = els[i].dataset.id;
      sel.appendChild(o);
    }
    if (old && sel.querySelector('option[value="' + old + '"]')) sel.value = old;
    else if (objectId && sel.querySelector('option[value="' + objectId + '"]')) sel.value = objectId;
    else { objectId = sel.value; selectedId = null; }
  }
  function renderList() {
    var host = document.getElementById("sl-anim-list");
    if (!host) return;
    host.textContent = "";
    var list = A.effects();
    if (!list.length) { host.appendChild(el("p", "sl-anim-empty", t("slides-anim-effects-empty", "No effects yet — add one below"))); return; }
    for (var i = 0; i < list.length; i++) {
      (function (fx, idx) {
        var row = el("div", "sl-anim-row-item");
        row.setAttribute("role", "option");
        row.setAttribute("tabindex", "0");
        row.dataset.id = fx.id;
        row.draggable = true;
        var name = A.effectName(fx);
        row.setAttribute("aria-label", name + " — " + t(START_KEYS[fx.start] || START_KEYS.click, "On click"));
        var grip = el("span", "sl-anim-grip", "⋮⋮");
        grip.setAttribute("aria-hidden", "true");
        row.appendChild(grip);
        row.appendChild(el("span", "sl-anim-fx-name", name));
        row.appendChild(el("span", "sl-anim-fx-meta", Math.round(fx.duration) + " " + t("slides-anim-ms", "ms")));
        var up = button("↑", "slides-anim-reorder-up", "Move up", "sl-anim-mini");
        var down = button("↓", "slides-anim-reorder-down", "Move down", "sl-anim-mini");
        var del = button("×", "slides-anim-remove", "Remove effect", "sl-anim-mini");
        up.addEventListener("click", function (e) { e.stopPropagation(); if (A.moveEffect(fx.id, -1)) E.status(t("slides-anim-status-moved", "Effect moved")); });
        down.addEventListener("click", function (e) { e.stopPropagation(); if (A.moveEffect(fx.id, 1)) E.status(t("slides-anim-status-moved", "Effect moved")); });
        del.addEventListener("click", function (e) { e.stopPropagation(); A.removeEffect(fx.id); E.status(t("slides-anim-status-removed", "Effect removed")); });
        row.appendChild(up); row.appendChild(down); row.appendChild(del);
        row.addEventListener("click", function () { selectedId = fx.id; renderEditor(); });
        row.addEventListener("keydown", function (e) {
          if (e.key === "Enter") { selectedId = fx.id; renderEditor(); return; }
          if (!e.altKey || (e.key !== "ArrowUp" && e.key !== "ArrowDown")) return;
          e.preventDefault();
          if (A.moveEffect(fx.id, e.key === "ArrowUp" ? -1 : 1)) {
            E.status(t("slides-anim-status-moved", "Effect moved"));
            var next = document.querySelector('[data-id="' + fx.id + '"]');
            if (next && next.focus) next.focus();
          }
        });
        row.addEventListener("dragstart", function (e) { e.dataTransfer.setData("text/plain", fx.id); e.dataTransfer.effectAllowed = "move"; });
        row.addEventListener("dragover", function (e) { e.preventDefault(); e.dataTransfer.dropEffect = "move"; });
        row.addEventListener("drop", function (e) {
          e.preventDefault();
          var fromId = e.dataTransfer.getData("text/plain");
          var reordered = A.effects().slice();
          var fi = -1, ti = -1;
          for (var k = 0; k < reordered.length; k++) {
            if (reordered[k].id === fromId) fi = k;
            if (reordered[k].id === fx.id) ti = k;
          }
          if (fi < 0 || ti < 0 || fi === ti) return;
          var moved = reordered.splice(fi, 1)[0];
          reordered.splice(ti, 0, moved);
          A.setEffects(reordered);
          E.status(t("slides-anim-status-moved", "Effect moved"));
        });
        host.appendChild(row);
      })(list[i], i);
    }
  }
  function renderTabs() {
    var host = document.getElementById("sl-anim-tabs");
    if (!host) return;
    host.textContent = "";
    for (var i = 0; i < A.CATEGORIES.length; i++) {
      (function (cat) {
        var b = button(t(cat.key, cat.fallback), null, null);
        b.setAttribute("role", "tab");
        b.dataset.cat = cat.id;
        b.setAttribute("aria-selected", cat.id === activeCategory ? "true" : "false");
        b.addEventListener("click", function () { activeCategory = cat.id; renderTabs(); renderGrid(); });
        host.appendChild(b);
      })(A.CATEGORIES[i]);
    }
  }
  function renderGrid() {
    var host = document.getElementById("sl-anim-grid");
    if (!host) return;
    host.textContent = "";
    for (var i = 0; i < A.EFFECTS.length; i++) {
      (function (def) {
        if (def.cat !== activeCategory) return;
        var b = button(t(def.key, def.fallback), null, null);
        b.addEventListener("click", function () {
          var fx = A.addEffect(def.id, objectId);
          selectedId = fx.id;
          E.status(t("slides-anim-status-added", "{ $name } added", { name: A.effectName(fx) }));
          refresh();
        });
        host.appendChild(b);
      })(A.EFFECTS[i]);
    }
  }
  function renderEditor() {
    var host = document.getElementById("sl-anim-editor");
    if (!host) return;
    var fx = currentFx();
    if (window.SlidesAnimationEditor) window.SlidesAnimationEditor.render(host, fx);
    else { host.hidden = true; }
  }

  function refresh() {
    if (!panel || panel.hidden) return;
    renderObjects();
    renderList();
    renderTabs();
    renderGrid();
    renderEditor();
    updateTime(E.time);
  }
  function openPanel() {
    build();
    if (!panel) return;
    previousFocus = document.activeElement;
    panel.hidden = false;
    renderObjects();
    renderList();
    renderTabs();
    renderGrid();
    renderEditor();
    updateTime(0);
    var first = focusables()[0];
    if (first && first.focus) first.focus();
    E.status(t("slides-anim-status-local-only", "Stored locally — server persistence is not available yet"));
  }
  function closePanel() {
    if (!panel) return;
    panel.hidden = true;
    E.stop();
    E.revealAll(A.effects());
    if (previousFocus && previousFocus.focus) previousFocus.focus();
    previousFocus = null;
  }
  A.on("effects", refresh);
  A.on("reduce", refresh);
  window.addEventListener("localeChanged", function () {
    if (!panel) return;
    var h = panel.querySelector(".sl-anim-title");
    if (h) h.textContent = t("slides-anim-title", "Animation timeline");
    refresh();
  });

  window.SlidesAnimationTimeline = { open: openPanel, close: closePanel, refresh: refresh, get isOpen() { return !!panel && !panel.hidden; } };
})(window);
