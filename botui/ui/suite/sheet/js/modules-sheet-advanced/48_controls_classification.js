"use strict";
/* Sheets enterprise controls (#2437) — classification banner, export watermark
 * preview and a local export gate. No file is actually exported and no real
 * watermark is embedded in any server-generated document. */

(function () {
  var C = window.SheetControls;

  var LEVELS = [
    { key: "public", label: "sheet-ctrl-class-public", fallback: "Public" },
    { key: "internal", label: "sheet-ctrl-class-internal", fallback: "Internal" },
    { key: "confidential", label: "sheet-ctrl-class-confidential", fallback: "Confidential" },
    { key: "restricted", label: "sheet-ctrl-class-restricted", fallback: "Restricted" },
  ];

  var POLICIES = [
    { key: "allow", label: "sheet-ctrl-class-policy-allow", fallback: "Allow export" },
    { key: "deny", label: "sheet-ctrl-class-policy-deny", fallback: "Deny export" },
    { key: "approve", label: "sheet-ctrl-class-policy-approve", fallback: "Require approval" },
  ];

  function labelOf(list, key) {
    var found = list.filter(function (item) { return item.key === key; })[0];
    return found ? C.t(found.label, found.fallback) : String(key);
  }

  function levelLabel() { return labelOf(LEVELS, C.state().classification.level); }

  function bannerText() {
    return C.t("sheet-ctrl-class-banner", "Classification: {level}", { level: levelLabel() });
  }

  function watermarkText() {
    return C.t("sheet-ctrl-class-watermark-text", "{level} - local preview only", { level: levelLabel() });
  }

  function exportStatusKey() {
    var policy = C.state().classification.exportPolicy;
    if (policy === "deny") return "sheet-ctrl-class-export-denied";
    if (policy === "approve") return "sheet-ctrl-class-export-approval";
    return "sheet-ctrl-class-export-allowed";
  }

  function exportStatusText() {
    var key = exportStatusKey();
    var fallbacks = {
      "sheet-ctrl-class-export-allowed": "Export is allowed by the local policy.",
      "sheet-ctrl-class-export-denied": "Export is denied by the local policy.",
      "sheet-ctrl-class-export-approval": "Approval is required; the local export button is disabled.",
    };
    return C.t(key, fallbacks[key]);
  }

  function canExport() {
    return C.state().classification.exportPolicy === "allow";
  }

  function set(level, policy) {
    var validLevel = LEVELS.filter(function (item) { return item.key === level; })[0];
    var validPolicy = POLICIES.filter(function (item) { return item.key === policy; })[0];
    if (!validLevel || !validPolicy) return false;
    C.mutate(function (state) {
      state.classification.level = level;
      state.classification.exportPolicy = policy;
    });
    paintBanner();
    return true;
  }

  function paintBanner() {
    var host = document.getElementById("sheet-content") || document.body;
    if (!host) return null;
    var banner = document.getElementById("ss-ctrl-class-banner");
    if (!banner) {
      banner = C.element("div", "ss-ctrl-class-banner");
      banner.id = "ss-ctrl-class-banner";
      banner.setAttribute("role", "status");
      banner.setAttribute("aria-live", "polite");
      if (host.insertBefore) host.insertBefore(banner, host.firstChild);
      else host.appendChild(banner);
    }
    var level = C.state().classification.level;
    banner.textContent = bannerText();
    banner.setAttribute("data-level", level);
    return banner;
  }

  function render(container, ctx) {
    var doc = container.ownerDocument || document;
    function make(tag, className, text) {
      var node = doc.createElement(tag);
      if (className) node.className = className;
      if (text != null) node.textContent = text;
      return node;
    }
    var classification = C.state().classification;
    var grid = make("div", "ss-ctrl-form");

    var levelRow = make("div", "ss-ctrl-row");
    var levelLabelNode = make("label", "ss-ctrl-label", C.t("sheet-ctrl-class-level", "Classification label"));
    levelLabelNode.setAttribute("data-i18n", "sheet-ctrl-class-level");
    var level = make("select", "ss-ctrl-input");
    level.id = "ss-ctrl-class-level";
    LEVELS.forEach(function (item) {
      var option = make("option", "", C.t(item.label, item.fallback));
      option.value = item.key;
      option.setAttribute("data-i18n", item.label);
      if (classification.level === item.key) option.selected = true;
      level.appendChild(option);
    });
    levelRow.appendChild(levelLabelNode);
    levelRow.appendChild(level);
    grid.appendChild(levelRow);

    var policyRow = make("div", "ss-ctrl-row");
    var policyLabelNode = make("label", "ss-ctrl-label", C.t("sheet-ctrl-class-policy", "Export policy"));
    policyLabelNode.setAttribute("data-i18n", "sheet-ctrl-class-policy");
    var policy = make("select", "ss-ctrl-input");
    policy.id = "ss-ctrl-class-policy";
    POLICIES.forEach(function (item) {
      var option = make("option", "", C.t(item.label, item.fallback));
      option.value = item.key;
      option.setAttribute("data-i18n", item.label);
      if (classification.exportPolicy === item.key) option.selected = true;
      policy.appendChild(option);
    });
    policyRow.appendChild(policyLabelNode);
    policyRow.appendChild(policy);
    grid.appendChild(policyRow);

    var watermark = make("div", "ss-ctrl-watermark", watermarkText());
    watermark.id = "ss-ctrl-watermark";
    var watermarkLabel = make("p", "ss-ctrl-label", C.t("sheet-ctrl-class-watermark", "Watermark preview"));
    watermarkLabel.setAttribute("data-i18n", "sheet-ctrl-class-watermark");
    grid.appendChild(watermarkLabel);
    grid.appendChild(watermark);

    var exportButton = make("button", "ss-ctrl-button", C.t("sheet-ctrl-class-export", "Export preview"));
    exportButton.type = "button";
    exportButton.id = "ss-ctrl-class-export";
    exportButton.setAttribute("data-i18n", "sheet-ctrl-class-export");
    exportButton.disabled = !canExport();
    grid.appendChild(exportButton);
    container.appendChild(grid);

    function refresh() {
      watermark.textContent = watermarkText();
      exportButton.disabled = !canExport();
      paintBanner();
    }
    level.addEventListener("change", function () {
      var ok = set(level.value, policy.value);
      ctx.setStatus(ok ? C.t("sheet-ctrl-class-saved", "Classification policy saved.") : C.t("sheet-ctrl-status-ready", "Ready."), !ok);
      refresh();
    });
    policy.addEventListener("change", function () {
      var ok = set(level.value, policy.value);
      ctx.setStatus(ok ? C.t("sheet-ctrl-class-saved", "Classification policy saved.") : C.t("sheet-ctrl-status-ready", "Ready."), !ok);
      refresh();
      ctx.setStatus(exportStatusText(), false);
    });
    exportButton.addEventListener("click", function () {
      if (!canExport()) return;
      if (window.SheetControlsAudit && window.SheetControlsAudit.recordEvent) {
        window.SheetControlsAudit.recordEvent("export", "sheet-ctrl-class-export");
      }
      ctx.setStatus(exportStatusText(), false);
    });
    refresh();
  }

  window.SheetControlsClassification = {
    LEVELS: LEVELS,
    POLICIES: POLICIES,
    levelLabel: levelLabel,
    bannerText: bannerText,
    watermarkText: watermarkText,
    exportStatusKey: exportStatusKey,
    canExport: canExport,
    set: set,
    paintBanner: paintBanner,
    render: render,
  };
})();
