"use strict";
/* Docs large .docx load path (#2533), focus-trapped dialog + inspector.
 *
 * Escape closes, Tab is trapped, focus is restored. Content is built with
 * createElement + textContent only (no innerHTML, no native dialogs). All
 * user-facing text resolves through the docs-large FTL catalog. */
(function (window) {
  var NS = window.DocsLarge;
  if (!NS) return;
  var UI = (NS.dialog = NS.dialog || {});
  var refs = null;
  var verdict = null;
  var range = null;
  var VIEWPORT_ROWS = 6;

  UI.shell = function () {
    var host = document.getElementById("docs-modal-container") || document.body;
    var restore = document.activeElement;
    var backdrop = NS.el("div", "gb-large-backdrop");
    var dialog = NS.el("div", "gb-large-dialog");
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-modal", "true");
    var titleId = "gb-large-title-" + Math.random().toString(36).slice(2, 9);
    dialog.setAttribute("aria-labelledby", titleId);
    var heading = NS.i18n(NS.el("h3", "gb-large-dialog-title"), "docs-large-title", "Large document load path");
    heading.id = titleId;
    dialog.appendChild(heading);
    var note = NS.el("p", "gb-large-local-note");
    note.textContent = NS.localNote();
    dialog.appendChild(note);
    var body = NS.el("div", "gb-large-dialog-body");
    dialog.appendChild(body);
    var status = NS.el("div", "gb-large-status");
    status.id = "docs-large-status";
    status.setAttribute("role", "status");
    status.setAttribute("aria-live", "polite");
    dialog.appendChild(status);
    var footer = NS.el("div", "gb-large-dialog-footer");
    var close = NS.i18n(NS.el("button", "gb-large-btn"), "docs-large-close", "Close");
    close.setAttribute("type", "button");
    footer.appendChild(close);
    dialog.appendChild(footer);
    backdrop.appendChild(dialog);

    var selector = "button,input,select,textarea,[tabindex]:not([tabindex='-1'])";
    function focusables() {
      var list = dialog.querySelectorAll ? dialog.querySelectorAll(selector) : [];
      return Array.prototype.filter.call(list, function (node) { return !node.disabled; });
    }
    function closeDialog() {
      document.removeEventListener("keydown", onKey, true);
      if (backdrop.parentNode) backdrop.parentNode.removeChild(backdrop);
      if (restore && restore.focus) restore.focus();
    }
    function onKey(event) {
      var action = NS.focusMove(focusables(), document.activeElement, event);
      if (action.action === "close") { event.preventDefault(); closeDialog(); return; }
      if (action.action === "focus") { event.preventDefault(); focusables()[action.index].focus(); }
    }
    close.addEventListener("click", closeDialog);
    backdrop.addEventListener("click", function (event) { if (event.target === backdrop) closeDialog(); });
    document.addEventListener("keydown", onKey, true);
    host.appendChild(backdrop);
    var first = focusables()[0];
    if (first) first.focus();
    return { body: body, status: status, close: closeDialog, root: dialog };
  };

  function button(key, fallback, cls) {
    var node = NS.i18n(NS.el("button", cls || "gb-large-btn"), key, fallback);
    node.setAttribute("type", "button");
    return node;
  }

  function section(titleKey, titleFallback) {
    var wrap = NS.el("section", "gb-large-section");
    wrap.appendChild(NS.i18n(NS.el("h4", "gb-large-section-title"), titleKey, titleFallback));
    return wrap;
  }

  function row(labelKey, labelFallback, node) {
    var line = NS.el("div", "gb-large-row");
    line.appendChild(NS.i18n(NS.el("span", "gb-large-label"), labelKey, labelFallback));
    if (node) line.appendChild(node);
    return line;
  }

  function value(text) { var node = NS.el("span", "gb-large-value"); node.textContent = text; return node; }

  function renderControls(shell) {
    var state = NS.state;
    var wrap = section("docs-large-controls", "Fixture and save controls");

    var size = NS.el("select", "gb-large-input");
    size.setAttribute("data-i18n-aria-label", "docs-large-size-label");
    size.setAttribute("aria-label", NS.tr("docs-large-size-label", "Simulated size"));
    NS.SIZES_MB.forEach(function (mb) {
      var option = NS.i18n(NS.el("option"), "docs-large-size-" + mb, mb + " MB");
      option.value = String(mb);
      size.appendChild(option);
    });
    size.value = String(state.sizeMb);
    wrap.appendChild(row("docs-large-size-label", "Simulated size", size));

    var load = button("docs-large-load", "Load fixture");
    wrap.appendChild(load);

    var mode = button("docs-large-toggle-mode", "Toggle automatic/manual");
    wrap.appendChild(row("docs-large-mode-label", "Save mode", mode));

    var etag = NS.el("input", "gb-large-input");
    etag.type = "text";
    etag.value = state.etagSent;
    etag.setAttribute("data-i18n-aria-label", "docs-large-etag-label");
    etag.setAttribute("aria-label", NS.tr("docs-large-etag-label", "ETag (If-Match)"));
    wrap.appendChild(row("docs-large-etag-label", "ETag (If-Match)", etag));

    var incremental = button("docs-large-save-incremental", "Save changed blocks");
    var full = button("docs-large-save-full", "Save full body");
    var actions = NS.el("div", "gb-large-actions");
    actions.appendChild(incremental);
    actions.appendChild(full);
    wrap.appendChild(actions);

    var failure = NS.el("p", "gb-large-failure");
    failure.setAttribute("role", "status");
    failure.setAttribute("aria-live", "polite");
    wrap.appendChild(failure);

    var bytes = value("");
    wrap.appendChild(row("docs-large-bytes-label", "Wire bytes (estimate)", bytes));
    shell.body.appendChild(wrap);
    return {
      size: size, load: load, mode: mode, etag: etag,
      incremental: incremental, full: full, failure: failure, bytes: bytes
    };
  }

  function renderDegradation(shell) {
    var wrap = section("docs-large-degradation", "Editing state");
    var banner = NS.el("p", "gb-large-banner");
    var enable = button("docs-large-enable-anyway", "Enable editing anyway");
    wrap.appendChild(banner);
    wrap.appendChild(enable);
    shell.body.appendChild(wrap);
    return { banner: banner, enable: enable };
  }

  function renderWindow(shell) {
    var wrap = section("docs-large-window-title", "Windowed editing");
    var scroll = NS.el("input", "gb-large-input");
    scroll.type = "range";
    scroll.min = "0";
    scroll.max = "100";
    scroll.value = "0";
    scroll.setAttribute("data-i18n-aria-label", "docs-large-scroll-label");
    scroll.setAttribute("aria-label", NS.tr("docs-large-scroll-label", "Scroll position"));
    wrap.appendChild(scroll);
    var host = NS.el("div", "gb-large-window-host");
    wrap.appendChild(host);
    var mounted = value("");
    wrap.appendChild(row("docs-large-scope", "Scoped contenteditable: visible window only", mounted));
    shell.body.appendChild(wrap);
    return { scroll: scroll, host: host, mounted: mounted };
  }

  function renderInspector(shell) {
    var wrap = section("docs-large-inspector", "Inspector");
    var rangeNode = value("");
    var depth = value("");
    wrap.appendChild(row("docs-large-range-label", "Mounted range", rangeNode));
    wrap.appendChild(row("docs-large-queue-depth", "Queue depth", depth));
    shell.body.appendChild(wrap);
    return { range: rangeNode, depth: depth };
  }

  function refresh() {
    if (!refs) return;
    var state = NS.state;
    var manifest = NS.Fixture.manifest(state.sizeMb);
    state.fullBytes = manifest.nominalBytes;

    verdict = NS.Degradation.evaluate(state.sizeMb, NS.DEFAULT_THRESHOLD_MB);
    refs.deg.banner.textContent = NS.tr(verdict.bannerKey, verdict.bannerFallback);
    refs.deg.enable.style.display = verdict.readOnly && !state.editingEnabled ? "" : "none";
    var modeKey = state.mode === "manual" ? "docs-large-mode-manual" : "docs-large-mode-auto";
    refs.controls.mode.setAttribute("data-i18n", modeKey);
    refs.controls.mode.textContent = NS.tr(modeKey, state.mode === "manual" ? "Manual" : "Automatic");
    refs.deg.banner.className = "gb-large-banner" + (verdict.readOnly ? " gb-large-banner-warn" : "");

    var total = manifest.blockCount;
    var maxScroll = Math.max(0, total - VIEWPORT_ROWS);
    var scrollBlock = Math.round(Number(refs.win.scroll.value) / 100 * maxScroll);
    range = NS.Window.visibleRange(scrollBlock * NS.Window.ROW_HEIGHT,
      VIEWPORT_ROWS * NS.Window.ROW_HEIGHT, total, NS.Window.ROW_HEIGHT, NS.Window.OVERSCAN);
    var plan = NS.Window.mountPlan(state.sizeMb, range);
    NS.Window.mount(refs.win.host, plan, NS.Degradation.editable(verdict, state));
    refs.win.mounted.textContent = plan.blocks.length + " blocks";
    refs.insp.range.textContent = NS.Window.label(plan, NS.tr);
    refs.insp.depth.textContent = String(state.queue.length);
    refs.controls.bytes.textContent = NS.tr("docs-large-bytes-incremental",
      "Incremental bytes: { $n }", { n: NS.Save.incrementalBytes(state) });
    NS.persist();
  }

  function doLoad() {
    var state = NS.state;
    state.sizeMb = Number(refs.controls.size.value) || NS.SIZES_MB[0];
    state.queue = [];
    state.etag = "v1";
    state.etagSent = "v1";
    verdict = NS.Degradation.start(state.sizeMb, NS.DEFAULT_THRESHOLD_MB, state);
    refs.controls.etag.value = state.etagSent;
    var manifest = NS.Fixture.manifest(state.sizeMb);
    NS.statusKey("docs-large-status-loaded", "Fixture loaded: { $n } blocks", { n: manifest.blockCount });
    refresh();
  }

  function doSave(kind) {
    var state = NS.state;
    state.etagSent = refs.controls.etag.value;
    state.fullBytes = NS.Fixture.manifest(state.sizeMb).nominalBytes;
    if (kind === "incremental" && !state.queue.length) {
      NS.Save.enqueue(state, NS.Fixture.blockId(0), NS.Fixture.blockText(0).length);
    }
    var result = NS.Save.plan(state);
    var outcome = NS.Save.commit(state, result);
    refs.controls.failure.textContent = outcome.lostData ? "" : NS.Save.message(result);
    refs.controls.failure.className = "gb-large-failure" + (result.ok ? " gb-large-failure-ok" : " gb-large-failure-error");
    NS.status(NS.Save.message(result));
    refresh();
  }

  UI.render = function (shell) {
    refs = {
      deg: renderDegradation(shell),
      controls: renderControls(shell),
      win: renderWindow(shell),
      insp: renderInspector(shell),
      baseline: NS.el("div", "gb-large-baseline")
    };
    shell.body.appendChild(refs.baseline);
    NS.Baseline.render(refs.baseline);

    refs.controls.load.addEventListener("click", doLoad);
    refs.controls.size.addEventListener("change", doLoad);
    refs.controls.mode.addEventListener("click", function () {
      NS.Save.toggleMode(NS.state);
      NS.statusKey("docs-large-status-mode", "Save mode: { $s }", { s: NS.state.mode });
      refresh();
    });
    refs.controls.etag.addEventListener("change", function () { NS.state.etagSent = refs.controls.etag.value; refresh(); });
    refs.controls.incremental.addEventListener("click", function () { doSave("incremental"); });
    refs.controls.full.addEventListener("click", function () { doSave("full"); });
    refs.deg.enable.addEventListener("click", function () {
      NS.Degradation.enableAnyway(NS.state);
      NS.statusKey("docs-large-status-enabled", "Editing enabled");
      refresh();
    });
    refs.win.scroll.addEventListener("input", refresh);
    return refs;
  };

  UI.refresh = function () { refresh(); };

  UI.open = function () {
    NS.loadState();
    var shell = UI.shell();
    UI.render(shell);
    refresh();
    return shell;
  };
})(typeof window !== "undefined" ? window : globalThis);
