"use strict";
/* Slides masters (#2429) — layer 3c: layout editor, brand kit with lock
 * rules, and the local versioned template library. */
(function (window) {
  var M = (window.GBSlidesMasters = window.GBSlidesMasters || {});
  var t = M.tr;

  function layoutOptions(state) {
    return state.layouts.map(function (l) { return { value: l.id, label: l.name }; });
  }

  M.buildLayoutPanel = function (api, state) {
    var box = M.section("slides-masters-section-layouts", "Layouts");
    var picker = M.selectField("slides-masters-layout-list", "Layouts", layoutOptions(state), state.layouts[0].id);
    box.appendChild(picker);
    var actions = document.createElement("div");
    actions.className = "sm-row";
    var nameField = M.textField("slides-masters-layout-name", "Layout name", "", "slides-masters-layout-name", "Layout name");
    nameField.className = "sm-inline-field";
    actions.appendChild(nameField);
    actions.appendChild(M.button("slides-masters-layout-add", "New layout", function () {
      var name = nameField.control.value.trim();
      if (!name) { M.status(api, "slides-masters-status-name-required", "Please enter a name"); return; }
      M.addLayout(name);
      M.status(api, "slides-masters-status-layout-added", "Layout created");
      api.rebuild();
    }, "primary"));
    actions.appendChild(M.button("slides-masters-layout-duplicate", "Duplicate layout", function () {
      M.duplicateLayout(picker.control.value);
      M.status(api, "slides-masters-status-layout-dup", "Layout duplicated");
      api.rebuild();
    }));
    actions.appendChild(M.button("slides-masters-layout-rename", "Rename layout", function () {
      var name = nameField.control.value.trim();
      if (!name) { M.status(api, "slides-masters-status-name-required", "Please enter a name"); return; }
      M.renameLayout(picker.control.value, name);
      M.status(api, "slides-masters-status-layout-renamed", "Layout renamed");
      api.rebuild();
    }));
    actions.appendChild(M.button("slides-masters-layout-delete", "Delete layout", function () {
      var ok = M.deleteLayout(picker.control.value);
      M.status(api, ok ? "slides-masters-status-layout-deleted" : "slides-masters-status-layout-last",
        ok ? "Layout deleted" : "At least one layout is required");
      api.rebuild();
    }));
    box.appendChild(actions);

    var layout = null;
    state.layouts.forEach(function (l) { if (l.id === picker.control.value) layout = l; });
    if (!layout) return box;
    var props = document.createElement("div");
    props.className = "sm-row";
    var bg = M.textField("slides-masters-layout-background", "Background colour", layout.background);
    var footer = M.textField("slides-masters-layout-footer", "Footer text", layout.footer);
    props.appendChild(bg); props.appendChild(footer);
    box.appendChild(props);
    var numLabel = document.createElement("label");
    numLabel.className = "sm-check";
    var num = document.createElement("input");
    num.type = "checkbox";
    num.checked = !!layout.number;
    M.labelled(numLabel, "data-i18n", "slides-masters-layout-number", "Show slide number");
    numLabel.insertBefore(num, numLabel.firstChild);
    box.appendChild(numLabel);
    box.appendChild(M.button("slides-masters-action-save", "Save", function () {
      M.setLayoutField(layout.id, "background", bg.control.value);
      M.setLayoutField(layout.id, "footer", footer.control.value);
      M.setLayoutField(layout.id, "number", num.checked);
      M.status(api, "slides-masters-status-layout-saved", "Layout saved");
    }, "primary"));

    box.appendChild(M.el("h3", "sm-subtitle", "slides-masters-layout-placeholder", "Placeholder geometry"));
    var geo = document.createElement("div");
    geo.className = "sm-geo-grid";
    M.ROLES.forEach(function (role) {
      var current = layout.placeholders[role] || { x: 0, y: 0, w: 100, h: 40 };
      var line = document.createElement("div");
      line.className = "sm-geo-row";
      line.appendChild(M.el("span", "sm-geo-role", "slides-masters-role-" + role, role));
      var fields = {};
      ["x", "y", "w", "h"].forEach(function (axis) {
        var input = document.createElement("input");
        input.type = "number";
        input.className = "sm-geo-input";
        input.value = current[axis];
        M.labelled(input, "data-i18n-aria-label", "slides-masters-geo-" + axis, axis);
        line.appendChild(input);
        fields[axis] = input;
      });
      line.appendChild(M.button("slides-masters-geo-save", "Save", function () {
        var geometry = {};
        ["x", "y", "w", "h"].forEach(function (axis) { geometry[axis] = Number(fields[axis].value) || 0; });
        M.setPlaceholder(layout.id, role, geometry);
        M.status(api, "slides-masters-status-layout-geo", "Placeholder geometry saved");
      }));
      geo.appendChild(line);
    });
    box.appendChild(geo);
    return box;
  };

  M.buildBrandPanel = function (api, state) {
    var box = M.section("slides-masters-section-brand", "Brand kit");
    var brand = M.clone(state.brand);
    var fonts = M.textField("slides-masters-brand-fonts", "Approved fonts", brand.fonts.join(", "));
    var palette = M.textField("slides-masters-brand-palette", "Approved colours", brand.palette.join(", "));
    var logo = M.textField("slides-masters-brand-logo", "Logo URL", brand.logoUrl);
    box.appendChild(fonts); box.appendChild(palette); box.appendChild(logo);
    var lockLabel = document.createElement("label");
    lockLabel.className = "sm-check";
    var lock = document.createElement("input");
    lock.type = "checkbox";
    lock.checked = !!brand.locked;
    M.labelled(lockLabel, "data-i18n", "slides-masters-brand-locked", "Brand lock");
    lockLabel.insertBefore(lock, lockLabel.firstChild);
    box.appendChild(lockLabel);
    var mode = M.selectField("slides-masters-brand-mode", "Lock mode", [
      { value: "warn", label: t("slides-masters-brand-mode-warn", "Warn only") },
      { value: "block", label: t("slides-masters-brand-mode-block", "Block off-brand content") }
    ], brand.mode);
    box.appendChild(mode);
    box.appendChild(M.button("slides-masters-action-save", "Save", function () {
      brand.fonts = fonts.control.value.split(",").map(function (s) { return s.trim(); }).filter(Boolean);
      brand.palette = palette.control.value.split(",").map(function (s) { return s.trim(); }).filter(Boolean);
      brand.logoUrl = logo.control.value.trim();
      brand.locked = lock.checked;
      brand.mode = mode.control.value;
      M.saveBrand(brand);
      M.status(api, "slides-masters-status-brand-saved", "Brand kit saved");
      api.rebuild();
    }, "primary"));

    box.appendChild(M.el("h3", "sm-subtitle", "slides-masters-brand-test", "Test an insertion"));
    var test = document.createElement("div");
    test.className = "sm-row";
    var fontTest = M.textField("slides-masters-brand-test-font", "Font to insert", "");
    var colorTest = M.textField("slides-masters-brand-test-color", "Colour to insert", "");
    fontTest.className = "sm-inline-field"; colorTest.className = "sm-inline-field";
    test.appendChild(fontTest); test.appendChild(colorTest);
    test.appendChild(M.button("slides-masters-brand-test-run", "Check", function () {
      var result = M.checkInsert(M.init(), { font: fontTest.control.value.trim(), color: colorTest.control.value.trim() });
      if (result.allowed && !result.reasons.length) {
        M.status(api, "slides-masters-brand-ok", "Insertion allowed");
        return;
      }
      var reason = result.reasons[0];
      var kindKey = "slides-masters-brand-kind-" + reason.kind;
      var kind = t(kindKey, reason.kind);
      if (result.mode === "block") {
        M.status(api, "slides-masters-brand-blocked", "Off-brand {kind} \u201c{value}\u201d was blocked",
          { kind: kind, value: reason.value });
      } else {
        M.status(api, "slides-masters-brand-warned", "Off-brand {kind} \u201c{value}\u201d is not in the brand kit",
          { kind: kind, value: reason.value });
      }
    }));
    box.appendChild(test);
    return box;
  };

  M.buildTemplatesPanel = function (api, state) {
    var box = M.section("slides-masters-section-templates", "Template library");
    var nameField = M.textField("slides-masters-template-name", "Template name", "", "slides-masters-template-name", "Template name");
    nameField.className = "sm-inline-field";
    var row = document.createElement("div");
    row.className = "sm-row";
    row.appendChild(nameField);
    row.appendChild(M.button("slides-masters-template-save", "Save template", function () {
      var name = nameField.control.value.trim();
      if (!name) { M.status(api, "slides-masters-status-name-required", "Please enter a name"); return; }
      var entry = M.saveTemplate(name);
      M.status(api, "slides-masters-status-template-saved", "Template {name} saved", { name: entry.name });
      api.rebuild();
    }, "primary"));
    box.appendChild(row);
    if (!state.templates.length) {
      box.appendChild(M.el("p", "sm-hint", "slides-masters-template-empty", "No saved templates"));
      return box;
    }
    var list = document.createElement("div");
    list.className = "sm-template-list";
    state.templates.slice().reverse().forEach(function (entry) {
      var line = document.createElement("div");
      line.className = "sm-template-row";
      var label = document.createElement("span");
      label.textContent = entry.name + " \u00b7 " + entry.savedAt;
      line.appendChild(label);
      line.appendChild(M.button("slides-masters-template-restore", "Restore", function () {
        M.restoreTemplate(entry.versionId);
        M.status(api, "slides-masters-status-template-restored", "Template {name} restored", { name: entry.name });
        api.rebuild();
      }));
      line.appendChild(M.button("slides-masters-template-delete", "Delete", function () {
        M.deleteTemplate(entry.versionId);
        M.status(api, "slides-masters-status-template-deleted", "Template {name} deleted", { name: entry.name });
        api.rebuild();
      }));
      list.appendChild(line);
    });
    box.appendChild(list);
    return box;
  };
})(typeof window !== "undefined" ? window : globalThis);
