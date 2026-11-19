"use strict";
/* Slides object model — accessible local editor shell (#2430). Clicking the
 * toolbar entry opens a focus-trapped dialog with Esc/focus restore, a polite
 * status region and no window.alert/confirm/prompt. All labels resolve via gbT.
 * Media sections live in 49_om_dialog_media.js and register through Sections. */

(function () {
  var OM = window.GBSlidesOM;
  var Ops = window.GBSlidesOMOps;
  var panel = null;
  var lastFocus = null;
  var status = null;
  var controls = {};
  var Sections = {};

  function label(key, fallback, params) { return OM.t(key, fallback, params); }
  function el(tag, cls, text, key) {
    var node = document.createElement(tag);
    if (cls) node.className = cls;
    if (key) node.setAttribute("data-i18n", key);
    if (text !== undefined && text !== null) node.textContent = text;
    return node;
  }
  function button(key, fallback, handler, cls) {
    var node = el("button", cls || "sl-om-btn", label(key, fallback), key);
    node.type = "button";
    if (typeof handler === "function") node.addEventListener("click", handler);
    return node;
  }
  function fieldset(key, fallback) {
    var box = el("fieldset", "sl-om-section");
    box.appendChild(el("legend", "sl-om-legend", label(key, fallback), key));
    return box;
  }
  function announce(message) {
    if (status) status.textContent = message || "";
  }

  function focusables(box) {
    if (!box || !box.querySelectorAll) return [];
    var nodes = box.querySelectorAll("button:not([disabled]),input:not([disabled]),select:not([disabled]),textarea:not([disabled]),[tabindex]:not([tabindex='-1'])");
    return Array.prototype.filter.call(nodes || [], function (node) {
      return node.offsetParent !== null && node.getAttribute("aria-hidden") !== "true";
    });
  }
  function makeDialog(titleKey, titleFallback) {
    var previous = document.activeElement;
    var overlay = el("div", "sl-om-overlay");
    var box = el("div", "sl-om-dialog");
    box.setAttribute("role", "dialog");
    box.setAttribute("aria-modal", "true");
    box.tabIndex = -1;
    box.setAttribute("aria-label", label(titleKey, titleFallback));
    var head = el("div", "sl-om-dialog-head");
    head.appendChild(el("h2", "sl-om-dialog-title", label(titleKey, titleFallback), titleKey));
    var badge = el("span", "sl-om-local-badge", label("slides-om-local-only", "Local only — not saved to the server"), "slides-om-local-only");
    badge.setAttribute("role", "status");
    badge.setAttribute("aria-live", "polite");
    head.appendChild(badge);
    head.appendChild(button("slides-om-close", "Close", function () { dismiss(false); }, "sl-om-close"));
    box.appendChild(head);
    var live = el("div", "sl-om-status");
    live.setAttribute("role", "status");
    live.setAttribute("aria-live", "polite");
    box.appendChild(live);
    var body = el("div", "sl-om-dialog-body");
    var foot = el("div", "sl-om-dialog-foot");
    box.appendChild(body); box.appendChild(foot);
    overlay.appendChild(box);
    (document.body || document.documentElement).appendChild(overlay);

    function onKey(event) {
      if (event.key === "Escape") { event.preventDefault(); dismiss(false); return; }
      if (event.key !== "Tab") return;
      var list = focusables(box);
      if (!list.length) return;
      var first = list[0], last = list[list.length - 1];
      if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus(); }
      else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus(); }
    }
    function dismiss(result) {
      document.removeEventListener("keydown", onKey, true);
      if (overlay.parentNode) overlay.parentNode.removeChild(overlay);
      if (previous && typeof previous.focus === "function") previous.focus();
      if (typeof config.onClose === "function") config.onClose(result);
    }
    var config = { onClose: null };
    document.addEventListener("keydown", onKey, true);
    var first = focusables(box)[0];
    if (first && typeof first.focus === "function") first.focus();
    else if (typeof box.focus === "function") box.focus();
    return {
      overlay: overlay, box: box, body: body, foot: foot, live: live,
      dismiss: dismiss, setOnClose: function (fn) { config.onClose = fn; }
    };
  }

  function selectionSection(body) {
    var box = fieldset("slides-om-selection-label", "Selection");
    controls.selectAll = button("slides-om-select-all", "Select all", function () {
      OM.selectAll(); announce(label("slides-om-status-selected", "{n} object(s) selected", { n: String(OM.selection.length) }));
    });
    controls.clear = button("slides-om-clear-selection", "Clear selection", function () {
      OM.clearSelection(); announce(label("slides-om-status-empty", "Nothing selected"));
    });
    controls.group = button("slides-om-group", "Group", function () {
      var count = Ops.group(OM.selection.slice());
      announce(count ? label("slides-om-status-grouped", "{n} object(s) grouped", { n: "2" }) : label("slides-om-status-select-two", "Select at least two objects"));
    });
    controls.ungroup = button("slides-om-ungroup", "Ungroup", function () {
      announce(Ops.ungroup() ? label("slides-om-status-ungrouped", "Group ungrouped") : label("slides-om-status-empty", "Nothing selected"));
    });
    controls.lock = button("slides-om-lock", "Lock", function () {
      Ops.lockHide("locked", true); announce(label("slides-om-status-locked", "Object locked"));
    });
    controls.hide = button("slides-om-hide", "Hide", function () {
      Ops.lockHide("hidden", true); announce(label("slides-om-status-hidden", "Object hidden"));
    });
    controls.duplicate = button("slides-om-duplicate", "Duplicate", function () { Ops.duplicateSelected(); });
    controls.delete = button("slides-om-delete", "Delete", function () {
      var removed = Ops.deleteSelected();
      announce(label("slides-om-status-deleted", "Deleted {n} object(s)", { n: String(removed) }));
    }, "sl-om-btn-danger");
    [controls.selectAll, controls.clear, controls.group, controls.ungroup,
      controls.lock, controls.hide, controls.duplicate, controls.delete].forEach(function (b) { box.appendChild(b); });
    body.appendChild(box);
  }

  function zOrderSection(body) {
    var box = fieldset("slides-om-zorder-label", "Layer order");
    [["front", "slides-om-bring-front", "Bring to front"],
      ["forward", "slides-om-bring-forward", "Bring forward"],
      ["backward", "slides-om-send-backward", "Send backward"],
      ["back", "slides-om-send-back", "Send to back"]].forEach(function (item) {
      box.appendChild(button(item[1], item[2], function () { Ops.zOrder(item[0]); }));
    });
    body.appendChild(box);
  }

  function precisionSection(body) {
    var box = fieldset("slides-om-align-label", "Align");
    [["left", "slides-om-align-left", "Align left"],
      ["hcenter", "slides-om-align-hcenter", "Align centre horizontally"],
      ["right", "slides-om-align-right", "Align right"],
      ["top", "slides-om-align-top", "Align top"],
      ["vcenter", "slides-om-align-vcenter", "Align centre vertically"],
      ["bottom", "slides-om-align-bottom", "Align bottom"]].forEach(function (item) {
      box.appendChild(button(item[1], item[2], function () {
        if (!OM.selection.length) { announce(label("slides-om-status-empty", "Nothing selected")); return; }
        Ops.align(item[0]);
      }));
    });
    var dist = fieldset("slides-om-distribute-label", "Distribute");
    [["h", "slides-om-distribute-h", "Distribute horizontally"],
      ["v", "slides-om-distribute-v", "Distribute vertically"]].forEach(function (item) {
      dist.appendChild(button(item[1], item[2], function () {
        if (!Ops.distribute(item[0])) announce(label("slides-om-status-need-three", "Select at least three objects"));
      }));
    });
    [["h", "slides-om-space-h", "Space evenly across"],
      ["v", "slides-om-space-v", "Space evenly down"]].forEach(function (item) {
      dist.appendChild(button(item[1], item[2], function () { Ops.spaceEvenly(item[0]); }));
    });
    [["w", "slides-om-equal-width", "Equal width"],
      ["h", "slides-om-equal-height", "Equal height"],
      ["both", "slides-om-equal-size", "Equal size"]].forEach(function (item) {
      dist.appendChild(button(item[1], item[2], function () { Ops.equalSize(item[0]); }));
    });
    body.appendChild(box); body.appendChild(dist);
  }

  function guidesSection(body) {
    var box = fieldset("slides-om-guides-label", "Guides & snapping");
    var settings = OM.settings();
    [["rulers", "slides-om-guide-rulers", "Rulers"],
      ["grid", "slides-om-guide-grid", "Grid"],
      ["snap", "slides-om-guide-snap", "Snap to objects"],
      ["smartGuides", "slides-om-guide-smart", "Smart guides"]].forEach(function (item) {
      var wrap = el("label", "sl-om-check");
      var input = document.createElement("input");
      input.type = "checkbox"; input.checked = !!settings[item[0]];
      input.addEventListener("change", function () { Ops.setting(item[0], input.checked); });
      wrap.appendChild(input);
      wrap.appendChild(el("span", "", label(item[1], item[2]), item[1]));
      box.appendChild(wrap);
    });
    var axis = document.createElement("select");
    axis.setAttribute("aria-label", label("slides-om-guide-axis", "Guide axis"));
    axis.setAttribute("data-i18n-aria-label", "slides-om-guide-axis");
    ["x", "y"].forEach(function (value, index) {
      var option = document.createElement("option");
      option.value = value;
      option.textContent = label(index === 0 ? "slides-om-guide-vertical" : "slides-om-guide-horizontal",
        index === 0 ? "Vertical" : "Horizontal");
      axis.appendChild(option);
    });
    var at = document.createElement("input");
    at.type = "number"; at.value = "120";
    at.setAttribute("aria-label", label("slides-om-guide-position", "Guide position"));
    at.setAttribute("data-i18n-aria-label", "slides-om-guide-position");
    box.appendChild(axis); box.appendChild(at);
    box.appendChild(button("slides-om-guide-add", "Add guide", function () {
      Ops.addGuide(axis.value, OM.num(at.value, 0));
      announce(label("slides-om-status-guide-added", "Guide added"));
    }));
    box.appendChild(button("slides-om-guide-remove", "Remove last guide", function () {
      var guides = OM.slide().guides || [];
      if (!guides.length) return;
      Ops.removeGuide(guides[guides.length - 1].id);
      announce(label("slides-om-status-guide-removed", "Guide removed"));
    }));
    body.appendChild(box);
  }

  function syncSelection() {
    if (!panel) return;
    var count = OM.selection.length;
    Object.keys(controls).forEach(function (name) {
      if (name === "selectAll" || name === "clear" || name === "pen" || name === "connect") return;
      if (controls[name] && controls[name].disabled !== undefined) controls[name].disabled = count === 0;
    });
  }

  function open() {
    if (panel) return;
    lastFocus = document.activeElement;
    var dialog = makeDialog("slides-om-title", "Object model");
    panel = dialog;
    status = dialog.live;
    Object.keys(controls).forEach(function (key) { delete controls[key]; });
    selectionSection(dialog.body);
    zOrderSection(dialog.body);
    precisionSection(dialog.body);
    guidesSection(dialog.body);
    if (Sections.objects) Sections.objects(dialog.body);
    if (Sections.size) Sections.size(dialog.body);
    dialog.foot.appendChild(button("slides-om-undo", "Undo", function () {
      if (!OM.undo()) announce(label("slides-om-status-nothing-undo", "Nothing to undo"));
    }));
    dialog.foot.appendChild(button("slides-om-redo", "Redo", function () {
      if (!OM.redo()) announce(label("slides-om-status-nothing-redo", "Nothing to redo"));
    }));
    var close = button("slides-om-close", "Close", function () { dialog.dismiss(false); });
    dialog.foot.appendChild(close);
    dialog.setOnClose(function () {
      panel = null; status = null;
      OM.penMode = false; OM.connectorMode = null;
      if (window.GBSlidesOMCanvas) window.GBSlidesOMCanvas.render();
      if (lastFocus && typeof lastFocus.focus === "function") lastFocus.focus();
      lastFocus = null;
    });
    OM.on("selection", syncSelection);
    syncSelection();
    announce(label("slides-om-status-ready", "Select objects on the canvas to begin."));
  }

  function probe() {
    if (!panel) return null;
    var box = panel.box;
    return {
      role: box.getAttribute("role"),
      ariaModal: box.getAttribute("aria-modal"),
      ariaLabel: box.getAttribute("aria-label"),
      focusTrap: !!panel.overlay,
      esc: true,
      restoresFocus: !!lastFocus,
      statusRole: status ? status.getAttribute("role") : null,
      statusLive: status ? status.getAttribute("aria-live") : null,
      localBadge: !!panel.box.querySelector(".sl-om-local-badge")
    };
  }

  window.GBSlidesOMDialog = {
    open: open,
    close: function () { if (panel) panel.dismiss(false); },
    isOpen: function () { return !!panel; },
    syncSelection: syncSelection,
    probe: probe,
    makeDialog: makeDialog
  };
  window.GBSlidesOMDialogUI = {
    el: el, button: button, fieldset: fieldset, label: label,
    announce: announce, controls: controls, sections: Sections
  };
}());
