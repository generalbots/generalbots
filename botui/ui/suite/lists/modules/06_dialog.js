"use strict";
/* #2537 - accessible segment dialog: role=dialog, focus trap, Esc, focus
   restore, inline validation, no native alert/confirm/prompt. */
(function (global) {
  var ns = global.GBSegments;
  if (!ns) return;

  var listening = false;
  var debouncedPreview = ns.debounce(function () { ns.buildPreview(); }, 350);

  ns.attachListsApp = function (api) {
    if (api && typeof api.refresh === "function") ns.state.refresh = api.refresh;
  };

  function dialog() { return ns.byId("gb-segment-dialog"); }
  function backdrop() { return ns.byId("gb-segment-backdrop"); }

  function setPathValue(root, path, mutate) {
    var node = ns.nodeAt(root, path);
    if (!node) return;
    mutate(node);
  }

  function open(source, initialModel, name) {
    var back = backdrop();
    if (!back) return;
    ns.state.source = source || "header";
    ns.state.opener = document.activeElement;
    ns.state.model = initialModel ? ns.normalize(initialModel) : ns.emptyModel();
    ns.state.errors = [];
    var nameInput = ns.byId("gb-segment-save-name");
    if (nameInput) nameInput.value = name || "";
    var nameError = ns.byId("gb-segment-name-error");
    if (nameError) nameError.hidden = true;
    var apply = ns.byId("gb-segment-apply");
    if (apply) apply.hidden = ns.state.source !== "list-modal";
    back.hidden = false;
    ns.renderBuilder();
    ns.renderPreview(null);
    ns.setStatus("");
    var builder = ns.byId("gb-segment-builder");
    var first = builder.querySelector("select") || builder.querySelector('[data-action="add-condition"]');
    if (first && typeof first.focus === "function") first.focus();
    else if (ns.byId("gb-segment-close")) ns.byId("gb-segment-close").focus();
  }
  ns.open = open;

  function close() {
    var back = backdrop();
    if (!back || back.hidden) return;
    ns.cancelPreview();
    back.hidden = true;
    var opener = ns.state.opener;
    ns.state.opener = null;
    if (opener && typeof opener.focus === "function") opener.focus();
  }
  ns.close = close;

  function trapKeydown(event) {
    var dlg = dialog();
    if (!dlg) return;
    if (event.key === "Escape") {
      event.preventDefault();
      close();
      return;
    }
    if (event.key !== "Tab") return;
    var focusable = Array.prototype.slice.call(dlg.querySelectorAll(ns.FOCUSABLE)).filter(function (el) {
      return !el.hidden && (typeof el.offsetParent === "undefined" || el.offsetParent !== null);
    });
    if (!focusable.length) return;
    var first = focusable[0];
    var last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  function changed(event) {
    var target = event.target;
    var path = parseBuilderPath(target);
    if (!target || !path) return;
    var action = target.getAttribute("data-action");
    var part = target.getAttribute("data-part");
    if (action === "match") {
      setPathValue(ns.state.model, path, function (node) { node.match = target.value === "any" ? "any" : "all"; });
      ns.state.errors = [];
      ns.renderBuilder();
      debouncedPreview();
      return;
    }
    if (part === "field") {
      setPathValue(ns.state.model, path, function (node) {
        node.field = target.value;
        node.op = "";
        node.value = "";
      });
      ns.state.errors = [];
      ns.renderBuilder();
      debouncedPreview();
      return;
    }
    if (part === "op") {
      setPathValue(ns.state.model, path, function (node) {
        node.op = target.value;
        var field = ns.fieldById(node.field);
        if (field && field.options && !["in", "not_in"].includes(node.op)) node.value = "";
      });
      ns.state.errors = [];
      ns.renderBuilder();
      debouncedPreview();
      return;
    }
    if (part === "value") {
      setPathValue(ns.state.model, path, function (node) { node.value = target.value; });
      debouncedPreview();
    }
  }

  function parseBuilderPath(el) {
    if (!el || !el.getAttribute) return null;
    var raw = el.getAttribute("data-path");
    if (raw == null) return null;
    try { return JSON.parse(raw); } catch (e) { return null; }
  }

  function clicked(event) {
    var button = event.target.closest ? event.target.closest("[data-action]") : null;
    if (!button) return;
    var action = button.getAttribute("data-action");
    var path = parseBuilderPath(button);
    if (!action || !path) return;
    var root = ns.state.model;
    if (action === "add-condition") {
      ns.addTo(root, path, ns.newCondition());
    } else if (action === "add-group") {
      ns.addTo(root, path, ns.newGroup("all"));
    } else if (action === "remove-condition" || action === "remove-group") {
      ns.removeAt(root, path);
    } else {
      return;
    }
    ns.state.errors = [];
    ns.renderBuilder();
    debouncedPreview();
  }

  function focusFirstError() {
    var invalid = dialog().querySelector(".gb-segment-invalid select, .gb-segment-invalid input");
    if (invalid && typeof invalid.focus === "function") invalid.focus();
  }

  function validatedModel() {
    var errors = ns.validate(ns.state.model);
    if (!errors.length) return ns.normalize(ns.state.model);
    ns.setErrors(errors);
    ns.setStatus(ns.t("segments-status-validation-error", "Fix the highlighted conditions before saving."));
    focusFirstError();
    return null;
  }

  function listName() {
    var input = ns.byId("gb-segment-save-name");
    var error = ns.byId("gb-segment-name-error");
    var value = input ? input.value.trim() : "";
    if (error) error.hidden = !!value;
    if (!value && input) input.focus();
    return value;
  }

  function applyToList() {
    var model = validatedModel();
    if (!model) return;
    var json = ns.toJson(model);
    var hidden = ns.byId("list-form-segment-json");
    var textarea = ns.byId("list-form-query");
    var type = ns.byId("list-form-type");
    if (hidden) hidden.value = json;
    if (textarea) textarea.value = json;
    if (type && type.value !== "dynamic") {
      type.value = "dynamic";
      type.dispatchEvent(new Event("change"));
    }
    ns.renderSummary(json);
    close();
  }

  async function saveAsList() {
    var model = validatedModel();
    if (!model) return;
    var name = listName();
    if (!name) return;
    var button = ns.byId("gb-segment-save");
    if (button) button.disabled = true;
    try {
      var response = await fetch("/api/crm/lists", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ name: name, list_type: "dynamic", segment: model, query_text: ns.toJson(model) })
      });
      if (!response.ok) throw new Error("HTTP " + response.status);
      ns.setStatus(ns.t("segments-save-success", "List saved."));
      close();
      if (ns.state.refresh) ns.state.refresh();
    } catch (e) {
      ns.setStatus(ns.t("segments-save-error", "Could not save the list. Check the details and try again."));
    } finally {
      if (button) button.disabled = false;
    }
  }

  ns.renderSummary = function (json) {
    var summary = ns.byId("lists-segment-summary");
    if (!summary) return;
    var hidden = ns.byId("list-form-segment-json");
    var model = ns.fromJson(json || (hidden ? hidden.value : ""));
    summary.textContent = model
      ? ns.t("segments-summary-label", "Current segment") + ": " + ns.countConditions(model)
      : ns.t("segments-summary-empty", "No structured segment yet.");
  };

  function openFromModal() {
    var textarea = ns.byId("list-form-query");
    var hidden = ns.byId("list-form-segment-json");
    var raw = hidden && hidden.value ? hidden.value : (textarea ? textarea.value : "");
    var model = ns.fromJson(raw) || ns.emptyModel();
    open("list-modal", model, ns.byId("list-form-name") ? ns.byId("list-form-name").value : "");
  }

  ns.init = function () {
    if (!backdrop() || !dialog()) return;
    if (typeof global.GBListsRefresh === "function") {
      ns.attachListsApp({ refresh: global.GBListsRefresh });
    }
    // HTMX can re-swap the fragment; rewire only when the nodes changed.
    if (listening && ns.dialogNodes && ns.dialogNodes.backdrop === backdrop() && ns.dialogNodes.dialog === dialog()) return;
    listening = false;
    ns.dialogNodes = { backdrop: backdrop(), dialog: dialog() };
    listening = true;
    var builder = ns.byId("gb-segment-builder");
    var dlg = dialog();
    if (builder) {
      builder.addEventListener("change", changed);
      builder.addEventListener("input", function (e) {
        if (e.target && e.target.getAttribute("data-part") === "value") changed(e);
      });
      builder.addEventListener("click", clicked);
    }
    dlg.addEventListener("change", function (e) {
      if (e.target && e.target.getAttribute("data-action") === "match") changed(e);
    });
    dlg.addEventListener("keydown", trapKeydown);
    backdrop().addEventListener("click", function (e) {
      if (e.target === backdrop()) close();
    });
    var closeBtn = ns.byId("gb-segment-close");
    if (closeBtn) closeBtn.addEventListener("click", close);
    var cancel = ns.byId("gb-segment-cancel");
    if (cancel) cancel.addEventListener("click", close);
    var apply = ns.byId("gb-segment-apply");
    if (apply) apply.addEventListener("click", applyToList);
    var save = ns.byId("gb-segment-save");
    if (save) save.addEventListener("click", saveAsList);
    var header = ns.byId("segments-open-builder");
    if (header) header.addEventListener("click", function () { open("header"); });
    var fromModal = ns.byId("lists-build-segment");
    if (fromModal) fromModal.addEventListener("click", openFromModal);
    ns.renderSummary();
  };
})(window);
