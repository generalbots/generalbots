/* WindowManager — window chrome, taskbar docking and recents (3/9). */
(function (window) {
  "use strict";
  var WM = window.__GBWindowManager;
  if (!WM) return;
  var WindowManager = WM.WindowManager;
  var APPS_REGISTRY = WM.APPS_REGISTRY;

  // #2933 — all non-critical chrome load work joins ONE coalesced idle/rAF pass.
  var chromeWork = [], chromeWorkQueued = false;
  // Boot used to drain every job in one ~160 ms task: flag first, then bounded slices; geometry stays in the clamp callback.
  var chromeBootFlipped = false, chromeCursor = 0, CHROME_CHUNK = 8;
  function scheduleChromeWork() {
    if (chromeWorkQueued) return;
    chromeWorkQueued = true;
    if (typeof window.requestIdleCallback === "function") window.requestIdleCallback(drainChromeWork, { timeout: 800 });
    else window.requestAnimationFrame(drainChromeWork);
  }
  function drainChromeWork() {
    chromeWorkQueued = false;
    if (!chromeBootFlipped) {
      // Cheap boot task: flag/class flip only.
      chromeBootFlipped = true;
      document.documentElement.classList.add("gb-chrome-idle-ready");
      if (chromeWork.length) scheduleChromeWork();
      return;
    }
    var jobs = chromeWork;
    // Bounded write-only slice; reclamps keep their own idle task below.
    var end = Math.min(jobs.length, chromeCursor + CHROME_CHUNK);
    for (; chromeCursor < end; chromeCursor += 1) jobs[chromeCursor]();
    if (chromeCursor >= jobs.length) {
      chromeWork = [];
      chromeCursor = 0;
    } else {
      scheduleChromeWork();
    }
  }
  function queueChromeWork(job) {
    chromeWork.push(job);
    scheduleChromeWork();
  }

  WM.defineMethods(WindowManager.prototype, {

  // #2410 — one geometry source; visualViewport.height survives the iOS keyboard (innerHeight does not).
  _viewportBounds() {
    const margin = 8;
    const vv = window.visualViewport;
    const vw = Math.max(160, window.innerWidth);
    const vh = (vv && vv.height) ? vv.height : window.innerHeight;
    const offsetTop = (vv && vv.offsetTop) ? vv.offsetTop : 0;
    let bottom = vh + offsetTop - margin;
    // Keep windows above the taskbar (also honoured by --gb-chrome-bottom).
    const taskbar = document.getElementById("taskbar") || document.querySelector(".toolbar-v15");
    if (taskbar) {
      const tb = taskbar.getBoundingClientRect();
      if (tb.top > 0 && tb.top < vh + offsetTop) bottom = Math.min(bottom, tb.top - margin);
    }
    if (bottom < offsetTop + margin + 120) bottom = offsetTop + margin + 120;
    return {
      margin,
      offsetTop,
      width: vw,
      top: margin + offsetTop,
      bottom,
      height: Math.max(120, bottom - margin - offsetTop),
    };
  },

  // Pointer device, not width alone, decides the mode: coarse phone = pinned sheet; coarse tablet keeps drag.
  _isCoarsePointer() {
    return !!(window.matchMedia && window.matchMedia("(pointer: coarse)").matches);
  },

  _isSheetMode() {
    return this._isCoarsePointer() && window.innerWidth < 768;
  },

  // Keep an open window inside the viewport with pure rect math; layout can
  // shift between cascade and append (auto-opened apps on load).
  _clampWindowIntoView(id) {
    const win = document.getElementById(`window-${id}`);
    if (!win) return;
    if (typeof this._syncResizeGrip === "function") this._syncResizeGrip(win);
    // A maximized window fills the workspace by design; never clamp it.
    if (document.body.classList.contains("window-maximized")) return;
    const b = this._viewportBounds();

    if (this._isSheetMode()) {
      // Reduced phone mode: the window is a full-height sheet pinned above
      // the taskbar. No free drag/resize; the app inside still scrolls.
      win.classList.add("gb-window-sheet");
      const sheetWidth = Math.max(160, b.width - b.margin * 2);
      win.style.left = `${b.margin}px`;
      win.style.top = `${b.top}px`;
      win.style.width = `${sheetWidth}px`;
      win.style.height = `${Math.floor(b.height)}px`;
      win.style.maxWidth = `${sheetWidth}px`;
      win.style.maxHeight = `${Math.floor(b.height)}px`;
      return;
    }
    win.classList.remove("gb-window-sheet");
    // Leaving sheet mode (e.g. on rotate) must not keep the phone caps.
    win.style.maxWidth = "";
    win.style.maxHeight = "";

    // READ PHASE — one box read; the resized rect is derived, not re-read.
    const rect = win.getBoundingClientRect();
    const curTop = parseInt(win.style.top || "0", 10) || 0;
    const curLeft = parseInt(win.style.left || "0", 10) || 0;
    const availW = b.width - b.margin * 2;
    const clampH = rect.height > b.height ? Math.floor(b.height) : null;
    const clampW = rect.width > availW ? Math.floor(availW) : null;
    const finalH = clampH === null ? rect.height : clampH;
    const finalW = clampW === null ? rect.width : clampW;
    let top = curTop;
    let left = curLeft;
    if (rect.top < b.top) top = curTop + (b.top - rect.top);
    if (rect.left < b.margin) left = curLeft + (b.margin - rect.left);
    if (rect.left + finalW > b.width - b.margin) left = curLeft - (rect.left + finalW - (b.width - b.margin));
    if (rect.top + finalH > b.bottom) top = curTop - (rect.top + finalH - b.bottom);

    // WRITE PHASE
    if (clampH !== null) {
      win.style.height = `${clampH}px`;
      win.style.maxHeight = `${clampH}px`;
    }
    if (clampW !== null) {
      win.style.width = `${clampW}px`;
      win.style.maxWidth = `${clampW}px`;
    }
    win.style.top = `${Math.max(b.top, top)}px`;
    win.style.left = `${Math.max(b.margin, left)}px`;
  },

  // #2341 — universal per-app sanity affordance.
  // Menu and diagnostics are built on demand by 08_app_actions.js.
  _sanityButton(id) {
    const label = window.gbT ? window.gbT("app-actions", "App actions") : "App actions";
    return `<button type="button" class="window-sanity-btn" data-app-id="${id}" ` +
      `title="${label}" aria-label="${label}" data-i18n-title="app-actions" ` +
      `data-i18n-aria-label="app-actions" aria-haspopup="menu" aria-expanded="false" ` +
      `onclick="window.GBAppActions && window.GBAppActions.open('${id}', this)">` +
      `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">` +
      `<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .34 1.88l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.7 1.7 0 0 0-1.88-.34 1.7 1.7 0 0 0-1.03 1.56V21a2 2 0 1 1-4 0v-.09a1.7 1.7 0 0 0-1.11-1.56 1.7 1.7 0 0 0-1.88.34l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.7 1.7 0 0 0 .34-1.88 1.7 1.7 0 0 0-1.56-1.03H3a2 2 0 1 1 0-4h.09A1.7 1.7 0 0 0 4.65 8.6a1.7 1.7 0 0 0-.34-1.88l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.7 1.7 0 0 0 1.88.34H9a1.7 1.7 0 0 0 1.03-1.56V3a2 2 0 1 1 4 0v.09a1.7 1.7 0 0 0 1.03 1.56 1.7 1.7 0 0 0 1.88-.34l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.7 1.7 0 0 0-.34 1.88V9a1.7 1.7 0 0 0 1.56 1.03H21a2 2 0 1 1 0 4h-.09a1.7 1.7 0 0 0-1.51 1.03z"/></svg>` +
      `</button>`;
  },

  _glassHeader(id, title, opts) {
    // #2223 — Maximize removed; Fullscreen is the single "fill the screen" action.
    const fullscreen = opts && opts.noMaximize !== true
      ? `<div class="window-dot window-dot-fullscreen" role="button" tabindex="0" title="Fullscreen" data-i18n-title="window-fullscreen" aria-label="Fullscreen" data-i18n-aria-label="window-fullscreen" aria-pressed="false" onclick="window.WindowManager.toggleFullscreen('${id}')"></div>`
      : "";
    return `<div class="window-header-glass">
      <div class="window-title">${title}</div>
      <div class="window-dot-controls">
        ${this._sanityButton(id)}
        <div class="window-dot window-dot-minimize" role="button" tabindex="0" title="Minimize" data-i18n-title="window-minimize" aria-label="Minimize" data-i18n-aria-label="window-minimize" onclick="window.WindowManager.toggleMinimize('${id}')"></div>
        ${fullscreen}
        <div class="window-dot window-dot-close" role="button" tabindex="0" title="Close" data-i18n-title="window-close" aria-label="Close" data-i18n-aria-label="window-close" onclick="window.WindowManager.close('${id}')"></div>
      </div>
    </div>`;
  },

  _glassBody(id) {
    return `<div id="window-body-${id}" class="window-body-glass"></div>`;
  },

  _legacyHeader(id, title, opts) {
    // #2223 — Maximize removed; Fullscreen is the single "fill the screen" action.
    const fullscreen = opts && opts.noMaximize !== true
      ? `<button class="btn-fullscreen hover:text-gray-600" title="Fullscreen" data-i18n-title="window-fullscreen" aria-label="Fullscreen" data-i18n-aria-label="window-fullscreen" aria-pressed="false" onclick="window.WindowManager.toggleFullscreen('${id}')"><svg class="icon-enter" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="15 3 21 3 21 9"/><polyline points="9 21 3 21 3 15"/><line x1="21" y1="3" x2="14" y2="10"/><line x1="3" y1="21" x2="10" y2="14"/></svg><svg class="icon-exit" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="4 14 10 14 10 20"/><polyline points="20 10 14 10 14 4"/><line x1="14" y1="10" x2="21" y2="3"/><line x1="3" y1="21" x2="10" y2="14"/></svg></button>`
      : "";
    return `<div class="window-header"><div class="font-mono text-xs font-bold text-brand-600 tracking-wide">${title}</div><div class="flex space-x-3 text-gray-400">${this._sanityButton(id)}<button class="btn-minimize hover:text-gray-600" onclick="window.WindowManager.toggleMinimize('${id}')"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="5" y1="12" x2="19" y2="12"/></svg></button>${fullscreen}<button class="btn-close hover:text-red-500" onclick="window.WindowManager.close('${id}')"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg></button></div></div>`;
  },

  _legacyBody(id) {
    return `<div id="window-body-${id}" class="window-body relative flex-1 overflow-y-auto"></div>`;
  },

  // #725/#1849 — status-bar sink for GBStatus (see 07_status.js); apps never write these nodes.
  _statusBar(id) {
    const title = ((this.openWindows.find((w) => w.id === id)) || {}).title || "";
    const statusText = window.gbT ? window.gbT("status-ready", "Ready") : "Ready";
    const safeTitle = String(title).replace(/[&<>"']/g, (ch) => ({
      "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
    })[ch]);
    return `<div class="window-statusbar" data-window-id="${id}" data-state="ready" role="group" aria-label="Window status">
      <span class="window-statusbar-status" data-state="ready" role="status" aria-live="polite" tabindex="0" data-help="status-ready" data-i18n="status-ready" data-i18n-title="status-ready" data-i18n-aria-label="status-ready" data-tooltip="${statusText}" title="${statusText}">${statusText}</span>
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
    // #2356 — never let a missing fragment render the word "undefined".
    const html = (htmlContent == null) ? "" : String(htmlContent);
    const tempDiv = document.createElement("div");
    tempDiv.innerHTML = html;
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
    // Vendor globals break on re-injection; dedup src scripts once. App
    // modules must re-run per window. A spinner hides the fragment until its
    // stylesheets apply (no FOUC).
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
    // Localize late: the boot i18n pass ran before this window existed.
    if (window.i18n && typeof window.i18n.translatePage === "function") {
      window.i18n.translatePage(content);
    }
    this._revealWhenReady(body);
  },

  // Reveal the body once its stylesheets settle; timeout guards missing CSS.
  _revealWhenReady(body) {
    const links = Array.from(body.querySelectorAll('link[rel="stylesheet"]'));
    const reclampAfterReveal = () => {
      // #1278 — clamp again after reveal; content can grow under the taskbar.
      const wid = body.dataset && body.dataset.windowId;
      if (!wid || !this._clampWindowIntoView) return;
      // #2933 — clamp reads getBoundingClientRect; reveal first, clamp on idle.
      if (typeof window.requestIdleCallback === "function") {
        window.requestIdleCallback(() => this._clampWindowIntoView(wid), { timeout: 800 });
      } else {
        window.setTimeout(() => this._clampWindowIntoView(wid), 1);
      }
    };
    if (!links.length) {
      queueChromeWork(function () {
        body.classList.add("gb-window-ready");
        reclampAfterReveal();
      });
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
      if (remaining <= 0) queueChromeWork(finish);
    };
    const timer = setTimeout(function () { queueChromeWork(finish); }, 3000);
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
    queueChromeWork(() => {
      const center = this.getTaskbarCenter();
      if (!center) return;
      const app = (window.APPS_REGISTRY || APPS_REGISTRY).find((a) => a.id === id);
      const title = app ? (window.gbAppTitle ? window.gbAppTitle(app) : app.title) : id;
      const icon = this.getIconSvg(id);
      // READ PHASE done
      const dockItem = document.createElement("div");
      dockItem.id = `dock-item-${id}`;
      dockItem.className = "taskbar-dock-item";
      dockItem.title = title;
      dockItem.onclick = () => this.toggleMinimize(id);
      dockItem.innerHTML = icon;
      center.appendChild(dockItem);
    });
  },

  _trackRecent(id, title) {
    // #1632 — Vibe has its own entry; never put it in Recent.
    if (id === "vibe" || String(id).indexOf("vibe-") === 0) return;
    try {
      let recent = gbStoreGet("gb-recent-apps", []);
      recent = recent.filter((r) => r.id !== id);
      recent.unshift({ id, title, ts: Date.now() });
      recent = recent.slice(0, 10);
      localStorage.setItem("gb-recent-apps", JSON.stringify(recent));
    } catch (e) { window.gbLogError ? gbLogError(e, "_trackRecent") : console.error("_trackRecent", e); }
  },

  getRecentApps() {
    try {
      const recent = gbStoreGet("gb-recent-apps", []);
      const registry = window.APPS_REGISTRY || APPS_REGISTRY;
      // #2015 — drop stale Recent ids the registry no longer resolves.
      const resolved = recent
        .filter((r) => r && r.id !== "vibe" && String(r.id).indexOf("vibe-") !== 0)
        .map((r) => registry.find((a) => a.id === r.id))
        .filter(Boolean);
      if (resolved.length !== recent.length) {
        try {
          localStorage.setItem("gb-recent-apps", JSON.stringify(
            recent.filter((r) => resolved.some((a) => a.id === (r && r.id)))
          ));
        } catch (e) { window.gbLogError ? gbLogError(e, "getRecentApps") : console.error("getRecentApps", e); }
      }
      return resolved;
    } catch (e) {
      return [];
    }
  },

  });
})(window);
