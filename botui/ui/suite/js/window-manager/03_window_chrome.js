/* WindowManager — window chrome, taskbar docking and recents (3/7). */
(function (window) {
  "use strict";
  var WM = window.__GBWindowManager;
  if (!WM) return;
  var WindowManager = WM.WindowManager;
  var APPS_REGISTRY = WM.APPS_REGISTRY;

  WM.defineMethods(WindowManager.prototype, {

  // Keep an open window inside the viewport. If layout shifted between
  // computing the cascade position and the window being appended
  // (auto-opened apps on load are the usual victim), the window can sit
  // off-screen (negative rect) even though its inline top/left look sane.
  // Shift it back into view using pure viewport-rect math: whatever the
  // offsetParent is doing, the window must end up fully visible.
  _clampWindowIntoView(id) {
    // A maximized window fills the workspace by design; never clamp it.
    if (document.body.classList.contains("window-maximized")) return;
    const win = document.getElementById(`window-${id}`);
    if (!win) return;
    const rect = win.getBoundingClientRect();
    const margin = 8;
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    // #1278 — the real lower bound is the taskbar's top edge, not the
    // viewport bottom: a window whose footer slides under the taskbar
    // (e.g. the New Project popup after its content grows) leaves its
    // primary buttons unclickable. Measure the taskbar when present and
    // keep windows above it.
    let bottomBound = vh - margin;
    const taskbar = document.getElementById("taskbar");
    if (taskbar) {
      const tb = taskbar.getBoundingClientRect();
      if (tb.top > 0 && tb.top < vh) bottomBound = tb.top - margin;
    }
    // Already fully inside — nothing to do.
    if (rect.top >= margin && rect.left >= margin &&
        rect.bottom <= bottomBound && rect.right <= vw - margin) return;
    const curTop = parseInt(win.style.top || "0", 10) || 0;
    const curLeft = parseInt(win.style.left || "0", 10) || 0;
    let top = curTop;
    let left = curLeft;
    if (rect.top < margin) top = curTop + (margin - rect.top);
    if (rect.left < margin) left = curLeft + (margin - rect.left);
    if (rect.right > vw - margin) left = curLeft - (rect.right - (vw - margin));
    if (rect.bottom > bottomBound) top = curTop - (rect.bottom - bottomBound);
    win.style.top = `${Math.max(margin, top)}px`;
    win.style.left = `${Math.max(margin, left)}px`;
  },

  _glassHeader(id, title, opts) {
    const maximize = opts && opts.noMaximize !== true
      ? `<div class="window-dot window-dot-maximize" role="button" tabindex="0" title="Maximize" data-i18n-title="window-maximize" aria-label="Maximize" data-i18n-aria-label="window-maximize" onclick="window.WindowManager.toggleMaximize('${id}')"></div>`
      : "";
    return `<div class="window-header-glass">
      <div class="window-title">${title}</div>
      <div class="window-dot-controls">
        <div class="window-dot window-dot-minimize" role="button" tabindex="0" title="Minimize" data-i18n-title="window-minimize" aria-label="Minimize" data-i18n-aria-label="window-minimize" onclick="window.WindowManager.toggleMinimize('${id}')"></div>
        ${maximize}
        <div class="window-dot window-dot-close" role="button" tabindex="0" title="Close" data-i18n-title="window-close" aria-label="Close" data-i18n-aria-label="window-close" onclick="window.WindowManager.close('${id}')"></div>
      </div>
    </div>`;
  },

  _glassBody(id) {
    return `<div id="window-body-${id}" class="window-body-glass"></div>`;
  },

  _legacyHeader(id, title, opts) {
    const maximize = opts && opts.noMaximize !== true
      ? `<button class="btn-maximize hover:text-gray-600" onclick="window.WindowManager.toggleMaximize('${id}')"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/></svg></button>`
      : "";
    return `<div class="window-header"><div class="font-mono text-xs font-bold text-brand-600 tracking-wide">${title}</div><div class="flex space-x-3 text-gray-400"><button class="btn-minimize hover:text-gray-600" onclick="window.WindowManager.toggleMinimize('${id}')"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="5" y1="12" x2="19" y2="12"/></svg></button>${maximize}<button class="btn-close hover:text-red-500" onclick="window.WindowManager.close('${id}')"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg></button></div></div>`;
  },

  _legacyBody(id) {
    return `<div id="window-body-${id}" class="window-body relative flex-1 overflow-y-auto"></div>`;
  },

  // Issue #725/#1849: every app window carries an inverted 3D bevel status
  // bar. The markup is the shared live contract renderer sink (see
  // 07_status.js): state LED + primary text, progress, metrics, selection,
  // connection and the window title. Apps publish through GBStatus and never
  // write these nodes directly.
  _statusBar(id) {
    const title = ((this.openWindows.find((w) => w.id === id)) || {}).title || "";
    const safeTitle = String(title).replace(/[&<>"']/g, (ch) => ({
      "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
    })[ch]);
    return `<div class="window-statusbar" data-window-id="${id}" data-state="ready" role="group" aria-label="Window status">
      <span class="window-statusbar-status" data-state="ready" role="status" aria-live="polite" tabindex="0" data-help="status-ready" data-tooltip="Ready" title="Ready">Ready</span>
      <span class="window-statusbar-progress" role="progressbar" aria-label="Progress" aria-valuemin="0" aria-valuemax="100" hidden>
        <span class="window-statusbar-progress-track" aria-hidden="true"><span class="window-statusbar-progress-fill"></span></span>
        <span class="window-statusbar-progress-label"></span>
      </span>
      <span class="window-statusbar-metrics" hidden></span>
      <span class="window-statusbar-selection" hidden></span>
      <span class="window-statusbar-spacer"></span>
      <span class="window-statusbar-connection" hidden></span>
      <span class="window-statusbar-title" title="${safeTitle}">${safeTitle}</span>
    </div>`;
  },

  _injectBodyContent(id, htmlContent) {
    const body = document.getElementById(`window-body-${id}`);
    if (!body) return;
    body.dataset.windowId = id;
    const tempDiv = document.createElement("div");
    tempDiv.innerHTML = htmlContent;
    const scripts = Array.from(tempDiv.querySelectorAll("script")).map((s) => {
      const clone = document.createElement("script");
      Array.from(s.attributes).forEach((a) => clone.setAttribute(a.name, a.value));
      clone.textContent = s.textContent;
      // Dynamic scripts are async by default and execute in arbitrary
      // order, which breaks modules that depend on earlier ones (e.g.
      // vibe-dialog-*.js registering into vibe-dialogs.js). Force
      // insertion-order execution so the fragment's script order holds.
      if (clone.hasAttribute("src")) clone.async = false;
      s.remove();
      return clone;
    });
    // Vendor scripts define globals (Terminal, _amdLoaderGlobal, etc.).
    // Re-injecting them in a second window breaks with "already declared"
    // errors, so skip src-based scripts that were already loaded once.
    // NOTE: dedup is scoped PER WINDOW (not page-global) — app modules
    // (vibe-*, chat-*, etc.) bind listeners to the fresh injected DOM and
    // MUST re-run on every window open; only vendor bundles are skipped
    // after their first load anywhere.
    //
    // Loading gate: the fragment is hidden behind a spinner until its own
    // stylesheets have applied, so windows never flash unstyled markup
    // (FOUC). Apps without local stylesheets reveal immediately.
    body.classList.remove("gb-window-ready");
    body.innerHTML =
      '<div class="gb-window-loading" role="status" aria-label="Loading app" data-i18n-aria-label="window-loading">' +
      '<div class="gb-window-loading-spinner"></div></div>' +
      '<div class="gb-window-content"></div>';
    const content = body.querySelector(".gb-window-content");
    content.innerHTML = tempDiv.innerHTML;
    const VENDOR = /(xterm|vendor\/|amd-loader|monaco|@|three\.)/;
    window.__gbLoadedScripts = window.__gbLoadedScripts || {};
    scripts.forEach((s) => {
      const src = s.getAttribute("src");
      if (src && VENDOR.test(src)) {
        if (window.__gbLoadedScripts[src]) return;
        window.__gbLoadedScripts[src] = true;
      }
      content.appendChild(s);
    });
    if (window.htmx) htmx.process(content);
    // Localize the fragment: the i18n boot pass ran before this window
    // existed, so its [data-i18n] nodes otherwise keep authored text or a
    // raw key (seen as "drive-starred"/"drive-recent" in the Drive nav).
    if (window.i18n && typeof window.i18n.translatePage === "function") {
      window.i18n.translatePage(content);
    }
    this._revealWhenReady(body);
  },

  // Reveal the app body once every local stylesheet of the fragment has
  // loaded (or failed). A timeout guards apps whose CSS 404s or hangs.
  _revealWhenReady(body) {
    const links = Array.from(body.querySelectorAll('link[rel="stylesheet"]'));
    const reclampAfterReveal = () => {
      // #1278 — the open-time clamp runs before the app content settles;
      // a popup whose body grows after reveal (New Project modal) can end
      // up extending under the taskbar. Clamp once more now that the
      // window has its final height.
      const wid = body.dataset && body.dataset.windowId;
      if (wid && this._clampWindowIntoView) this._clampWindowIntoView(wid);
    };
    if (!links.length) {
      body.classList.add("gb-window-ready");
      reclampAfterReveal();
      return;
    }
    const sheetReady = (link) => {
      try {
        return !!(link.sheet && link.sheet.cssRules && link.sheet.cssRules.length);
      } catch (e) {
        return false; // cross-origin sheet: fall through to load event
      }
    };
    let remaining = links.length;
    let done = false;
    const finish = () => {
      if (done) return;
      done = true;
      clearTimeout(timer);
      body.classList.add("gb-window-ready");
      reclampAfterReveal();
    };
    const onOne = () => {
      remaining -= 1;
      if (remaining <= 0) finish();
    };
    const timer = setTimeout(finish, 3000);
    links.forEach((link) => {
      if (sheetReady(link)) {
        onOne();
        return;
      }
      link.addEventListener("load", onOne);
      link.addEventListener("error", onOne);
    });
  },

  _addTaskbarDockItem(id) {
    const center = this.getTaskbarCenter();
    if (!center) return;
    const app = (window.APPS_REGISTRY || APPS_REGISTRY).find((a) => a.id === id);
    const dockItem = document.createElement("div");
    dockItem.id = `dock-item-${id}`;
    dockItem.className = "taskbar-dock-item";
    dockItem.title = app ? app.title : id;
    dockItem.onclick = () => this.toggleMinimize(id);
    dockItem.innerHTML = this.getIconSvg(id);
    center.appendChild(dockItem);
  },

  _trackRecent(id, title) {
    // #1632 item 6 — Vibe is opened from its own entry and must not show up
    // in the launcher's quick-access (Recent) grid.
    if (id === "vibe" || String(id).indexOf("vibe-") === 0) return;
    try {
      let recent = JSON.parse(localStorage.getItem("gb-recent-apps") || "[]");
      recent = recent.filter((r) => r.id !== id);
      recent.unshift({ id, title, ts: Date.now() });
      recent = recent.slice(0, 10);
      localStorage.setItem("gb-recent-apps", JSON.stringify(recent));
    } catch (e) {}
  },

  getRecentApps() {
    try {
      const recent = JSON.parse(localStorage.getItem("gb-recent-apps") || "[]");
      const registry = window.APPS_REGISTRY || APPS_REGISTRY;
      // #2015 — an app that left the registry (e.g. the duplicate "Sales"
      // surface folded into CRM) must not linger as a Recent tile with an
      // empty hxGet: it looks like a launcher entry but opens nothing. Keep
      // only entries the registry still resolves and drop the rest from
      // storage so the stale ids do not accumulate.
      const resolved = recent
        .filter((r) => r && r.id !== "vibe" && String(r.id).indexOf("vibe-") !== 0)
        .map((r) => registry.find((a) => a.id === r.id))
        .filter(Boolean);
      if (resolved.length !== recent.length) {
        try {
          localStorage.setItem("gb-recent-apps", JSON.stringify(
            recent.filter((r) => resolved.some((a) => a.id === (r && r.id)))
          ));
        } catch (e) {}
      }
      return resolved;
    } catch (e) {
      return [];
    }
  },

  });
})(window);
