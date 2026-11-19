"use strict";
/* Slides masters (#2429) — layer 3b: theme-token editor, inheritance
 * inspector and apply-to-selection/all with a before/after diff list. */
(function (window) {
  var M = (window.GBSlidesMasters = window.GBSlidesMasters || {});
  var t = M.tr;

  M.tokenLabelKey = function (key) {
    if (key === "font.major") return "slides-masters-font-major";
    if (key === "font.minor") return "slides-masters-font-minor";
    if (key === "effect.style") return "slides-masters-effect-style";
    if (key === "layout") return "slides-masters-change-layout";
    return "slides-masters-color-" + key.slice("color.".length);
  };
  M.sourceLabel = function (source) {
    if (source === "master") return t("slides-masters-source-master", "Master");
    if (source === "layout") return t("slides-masters-source-layout", "Layout");
    if (source === "slide") return t("slides-masters-source-slide", "Slide (override)");
    return t("slides-masters-source-none", "Not set");
  };
  function currentIndex() {
    var cv = (window.SlideCanvas && window.SlideCanvas.canvas) || document.getElementById("slide-canvas");
    if (!cv) return 0;
    var raw = cv.dataset ? cv.dataset.slideId : cv.getAttribute("data-slide-id");
    var n = parseInt(raw, 10);
    return isFinite(n) && n >= 0 ? n : 0;
  }
  function themeOptions(state) {
    return Object.keys(state.themes).map(function (id) {
      return { value: id, label: state.themes[id].name };
    });
  }
  function layoutOptions(state) {
    return state.layouts.map(function (l) { return { value: l.id, label: l.name }; });
  }

  M.buildThemePanel = function (api, state) {
    var box = M.section("slides-masters-section-theme", "Theme tokens");
    var picker = M.selectField("slides-masters-theme-select", "Theme", themeOptions(state), state.master.defaultThemeId);
    picker.control.className = "sm-theme-select";
    box.appendChild(picker);
    var actions = document.createElement("div");
    actions.className = "sm-row";
    var nameField = M.textField("slides-masters-theme-name", "Theme name", "", "slides-masters-theme-name", "Theme name");
    nameField.className = "sm-inline-field";
    actions.appendChild(nameField);
    actions.appendChild(M.button("slides-masters-theme-add", "New theme", function () {
      var name = nameField.control.value.trim();
      if (!name) { M.status(api, "slides-masters-status-name-required", "Please enter a name"); return; }
      var id = M.uid("theme");
      M.mutate(function (s) { s.themes[id] = { id: id, name: name, flat: M.defaultFlat() }; });
      M.status(api, "slides-masters-status-theme-new", "Theme created");
      api.rebuild();
    }, "primary"));
    actions.appendChild(M.button("slides-masters-theme-dup", "Duplicate theme", function () {
      var id = picker.control.value;
      M.mutate(function (s) {
        var src = s.themes[id];
        var copyId = M.uid("theme");
        s.themes[copyId] = { id: copyId, name: src.name + " copy", flat: M.clone(src.flat) };
      });
      M.status(api, "slides-masters-status-theme-dup", "Theme duplicated");
      api.rebuild();
    }));
    box.appendChild(actions);

    var active = state.themes[picker.control.value] || state.themes[state.master.defaultThemeId];
    var grid = document.createElement("div");
    grid.className = "sm-token-grid";
    M.TOKEN_KEYS.forEach(function (key) {
      var label = M.el("label", "sm-token", M.tokenLabelKey(key), key);
      var input = document.createElement("input");
      input.type = key === "effect.style" ? "text" : "text";
      input.className = "sm-token-input";
      input.value = active.flat[key] === undefined ? "" : String(active.flat[key]);
      input.setAttribute("data-i18n-aria-label", M.tokenLabelKey(key));
      input.setAttribute("aria-label", t(M.tokenLabelKey(key), key));
      input.addEventListener("change", function () {
        var affected = M.setMasterToken(active.id, key, input.value);
        M.status(api, "slides-masters-status-theme-token", "Token {key} updated on {n} slide(s)",
          { key: key, n: affected.length });
        api.rebuild();
      });
      label.appendChild(input);
      if (key.indexOf("color.") === 0) {
        var swatch = document.createElement("span");
        swatch.className = "sm-swatch";
        swatch.style.backgroundColor = active.flat[key];
        label.appendChild(swatch);
      }
      grid.appendChild(label);
    });
    box.appendChild(grid);
    return box;
  };

  M.buildInheritPanel = function (api, state) {
    var box = M.section("slides-masters-section-inherit", "Inheritance inspector");
    box.appendChild(M.el("p", "sm-hint", "slides-masters-inherit-hint", "Select a slide to inspect inherited values"));
    var keys = Object.keys(state.slides);
    var options = (keys.length ? keys : ["0"]).map(function (k) {
      return { value: k, label: t("slides-masters-apply-row", "Slide {n}", { n: Number(k) + 1 }) };
    });
    var picker = M.selectField("slides-masters-apply-scope", "Apply to", options, String(currentIndex()));
    box.appendChild(picker);
    var table = document.createElement("div");
    table.className = "sm-inherit-grid";
    M.TOKEN_KEYS.forEach(function (key) {
      var res = M.resolve(state, picker.control.value, key);
      var row = document.createElement("div");
      row.className = "sm-inherit-row";
      row.appendChild(M.el("span", "sm-inherit-prop", M.tokenLabelKey(key), key));
      var val = document.createElement("span");
      val.className = "sm-inherit-value";
      val.textContent = res.value === null ? t("slides-masters-change-none", "—") : String(res.value);
      row.appendChild(val);
      row.appendChild(M.el("span", "sm-inherit-from", "slides-masters-inherit-from", "Inherited from:"));
      var from = document.createElement("span");
      from.className = "sm-inherit-source";
      from.textContent = M.sourceLabel(res.source);
      row.appendChild(from);
      var input = document.createElement("input");
      input.type = "text";
      input.className = "sm-inherit-input";
      input.value = res.value === null ? "" : String(res.value);
      M.labelled(input, "data-i18n-aria-label", M.tokenLabelKey(key), key);
      row.appendChild(input);
      row.appendChild(M.button("slides-masters-inherit-set", "Set override", function () {
        M.setOverride(picker.control.value, key, input.value);
        M.status(api, "slides-masters-status-override-set", "Override saved on slide {n}",
          { n: Number(picker.control.value) + 1 });
        api.rebuild();
      }));
      if (res.explicit) {
        row.appendChild(M.button("slides-masters-inherit-clear", "Clear override", function () {
          M.clearOverride(picker.control.value, key);
          M.status(api, "slides-masters-status-override-cleared", "Override cleared on slide {n}",
            { n: Number(picker.control.value) + 1 });
          api.rebuild();
        }));
      }
      table.appendChild(row);
    });
    box.appendChild(table);
    return box;
  };

  M.buildApplyPanel = function (api, state) {
    var box = M.section("slides-masters-section-apply", "Apply to slides");
    var scope = M.selectField("slides-masters-apply-scope", "Apply to", [
      { value: "selection", label: t("slides-masters-apply-selection", "Selected slide") },
      { value: "all", label: t("slides-masters-apply-all", "All slides") }
    ], "selection");
    box.appendChild(scope);
    var themePick = M.selectField("slides-masters-theme-select", "Theme", themeOptions(state), state.master.defaultThemeId);
    var layoutPick = M.selectField("slides-masters-layout-list", "Layouts", layoutOptions(state), state.layouts[0].id);
    themePick.control.className = "sm-theme-select";
    layoutPick.control.className = "sm-layout-select";
    box.appendChild(themePick);
    box.appendChild(layoutPick);

    function targets() {
      var all = Object.keys(state.slides);
      if (scope.control.value === "all") return all;
      var idx = String(currentIndex());
      return all.indexOf(idx) >= 0 ? [idx] : (all.length ? [all[0]] : []);
    }
    function refreshDiff() {
      diff.innerHTML = "";
      var rows = M.diffApply(M.init(), {
        slideKeys: targets(), themeId: themePick.control.value, layoutId: layoutPick.control.value
      });
      if (!rows.length) {
        diff.appendChild(M.el("p", "sm-hint", "slides-masters-apply-nochange", "No slides would change"));
        return;
      }
      rows.forEach(function (row) {
        var line = M.el("div", "sm-diff-row", "slides-masters-apply-row",
          "Slide {n}: {count} change(s)");
        line.textContent = t("slides-masters-apply-row", "Slide {n}: {count} change(s)",
          { n: Number(row.slideKey) + 1, count: row.changes.length });
        row.changes.forEach(function (c) {
          var detail = document.createElement("span");
          detail.className = "sm-diff-detail";
          detail.textContent = "  " + t(M.tokenLabelKey(c.key), c.key) + ": " +
            String(c.before) + " \u2192 " + String(c.after) + " (" + M.sourceLabel(c.source) + ")";
          line.appendChild(detail);
        });
        diff.appendChild(line);
      });
    }
    var diff = document.createElement("div");
    diff.className = "sm-diff";
    box.appendChild(M.el("h3", "sm-subtitle", "slides-masters-apply-preview", "Changes preview"));
    box.appendChild(diff);
    scope.control.addEventListener("change", refreshDiff);
    themePick.control.addEventListener("change", refreshDiff);
    layoutPick.control.addEventListener("change", refreshDiff);
    refreshDiff();

    var buttons = document.createElement("div");
    buttons.className = "sm-row";
    buttons.appendChild(M.button("slides-masters-apply-theme", "Apply theme", function () {
      var rows = M.applyTheme({ slideKeys: targets(), themeId: themePick.control.value });
      M.status(api, "slides-masters-status-applied", "Applied to {n} slide(s)", { n: rows.length });
      api.rebuild();
    }, "primary"));
    buttons.appendChild(M.button("slides-masters-apply-layout", "Apply layout", function () {
      var rows = M.applyLayout({ slideKeys: targets(), layoutId: layoutPick.control.value });
      M.status(api, "slides-masters-status-applied", "Applied to {n} slide(s)", { n: rows.length });
      api.rebuild();
    }));
    box.appendChild(buttons);
    return box;
  };
})(typeof window !== "undefined" ? window : globalThis);
