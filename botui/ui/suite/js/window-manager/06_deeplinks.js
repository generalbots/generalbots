/* WindowManager — deep links, tool windows, dialogs and queries (6/9). */
(function (window) {
  "use strict";
  var WM = window.__GBWindowManager;
  if (!WM) return;
  var WindowManager = WM.WindowManager;
  var APPS_REGISTRY = WM.APPS_REGISTRY;
  var isPreviewId = WM.isPreviewId;

  WM.defineMethods(WindowManager.prototype, {
  openDeepLink(appId, params, opts) {
    opts = opts || {};
    // #2013 — "admin" deep link routes into Settings' admin view.
    if (appId === "admin") {
      params = Object.assign({ view: "admin" }, params || {});
      appId = "settings";
    }
    // `clockapp` is a legacy alias of `clock`.
    if (appId === "clockapp") {
      appId = "clock";
    }
    // #2015 — "sales" is now CRM's Quotes & Orders view.
    if (appId === "sales") {
      params = Object.assign({ view: "quotes" }, params || {});
      appId = "crm";
    }
    if (!window.GBPreviewMode.isOn() && isPreviewId(appId)) {
      // #2226 — offer Preview mode instead of silently falling back.
      this.offerPreviewApp(appId, params, opts);
      return;
    }
    const app = (window.APPS_REGISTRY || APPS_REGISTRY).find((a) => a.id === appId);
    const known = APPS_REGISTRY.find((a) => a.id === appId);
    // Known-but-withheld app: offer login, do not fetch a missing partial.
    if (!app && known && typeof WM.isAuthWithheldApp === "function" && WM.isAuthWithheldApp(appId)) {
      this._openLoginRequired(appId, known.title || appId, opts);
      return;
    }
    const title = app
      ? window.gbAppTitle
        ? window.gbAppTitle(app)
        : app.title
      : appId;
    const hxGet = app ? app.hxGet : `/suite/partials/${appId}.html`;
    this.closeStartMenu();
    // #1432/#1439 — new:true forces a fresh window instance.
    const forceNew = opts.new === true;
    const targetId = forceNew ? `${appId}-${Date.now().toString(36)}-${Math.floor(Math.random() * 1e4)}` : appId;
    const existed = !forceNew && this.getWindow(targetId) !== null;
    // Vibe keeps glass chrome but no status bar.
    this.open(targetId, title, "", {
      noMaximize: appId === "vibe",
      ownerId: opts.ownerId || null,
    });
    // #2184 — a fresh window starts with only this open's params.
    window.__gbAppParams__ = existed
      ? Object.assign({}, window.__gbAppParams__ || {}, params || {})
      : Object.assign({}, params || {});
    if (existed) {
      // #1288 — a re-targeted window adopts the caller's ownership.
      if (opts.ownerId) {
        const rec = this.openWindows.find((w) => w.id === targetId);
        if (rec) rec.ownerId = opts.ownerId;
      }
      // Deep-links re-target an open window, not only first opens.
      document.dispatchEvent(new CustomEvent("gb:deep-link", {
        detail: { appId: targetId, params: params || {} },
      }));
      return;
    }
    const qs = Object.keys(params || {}).map((k) => `${encodeURIComponent(k)}=${encodeURIComponent(params[k])}`).join("&");
    const sep = hxGet.indexOf("?") === -1 ? "?" : "&";
    fetch(hxGet + sep + (qs ? qs + "&" : "") + "_=" + Date.now()).then((r) => r.text()).then((html) => {
      const body = document.getElementById(`window-body-${targetId}`);
      if (body) this._injectBodyContent(targetId, html);
      // #2181 — the first batch still needs the deep-link event.
      document.dispatchEvent(new CustomEvent("gb:deep-link", {
        detail: { appId: targetId, params: params || {} },
      }));
      // Re-clamp after content settles.
      this._clampWindowIntoView(targetId);
    }).catch(() => {
      const body = document.getElementById(`window-body-${targetId}`);
      if (body) this._injectBodyContent(targetId, `<div style="padding:20px"><h3>${title}</h3><p>Application loading...</p></div>`);
    });
  },

  // Withheld app: localized login prompt instead of a 404.
  _openLoginRequired(appId, title, opts) {
    this.closeStartMenu();
    this.open(appId, title, "", { ownerId: (opts && opts.ownerId) || null });
    const body = document.getElementById(`window-body-${appId}`);
    if (!body) return;
    const t = (k, f) => window.gbT ? window.gbT(k, f) : f;
    const url = (window.GB_LOGIN_URL || "/login") + "?return_to=" + encodeURIComponent(window.location.href);
    this._injectBodyContent(appId, '<div style="padding:20px"><p>' +
      t("cloud-appstores-login-first", "Please login first") + '</p><a href="' + url + '">' +
      t("auth-go-to-login", "Go to Login") + '</a></div>');
  },

  // #2226 — a deep link (chat button, __ui_plan__, start menu) to a preview
  // app asks the user once to turn Preview mode on, then opens exactly the app
  // that was requested. Declining leaves the desktop untouched.
  offerPreviewApp(appId, params, opts) {
    const openIt = () => {
      window.GBPreviewMode.set(true);
      if (window.WindowManager && typeof window.WindowManager.refreshAppsCatalog === "function") {
        window.WindowManager.refreshAppsCatalog();
      }
      this.openDeepLink(appId, params, opts);
    };
    const label = appId;
    if (window.GBModal && typeof window.GBModal.confirm === "function") {
      window.GBModal.confirm(
        `"${label}" is a preview app. Turn on Preview mode and open it?`,
        "Preview mode",
        openIt,
        function () {}
      );
      return;
    }
    // No modal available (isolated runs): honour the request directly.
    openIt();
  },

  // VB6/Adobe-style floating tool window: open (or focus) a window under a
  // unique id and fetch a partial into its body. Unlike openDeepLink, the
  // id is caller-controlled (e.g. "vibe-run", "vibe-terminal") so several
  // accessory windows can float beside the main app window.
  openToolWindow(id, title, hxGet, params, opts) {
    this.closeStartMenu();
    const existed = this.getWindow(id) !== null;
    opts = opts || {};
    this.open(id, title, "", { tool: true, ownerId: opts.ownerId || null, noMaximize: opts.noMaximize === true });
    if (existed) return;
    window.__gbAppParams__ = Object.assign({}, window.__gbAppParams__ || {}, params || {});
    const qs = Object.keys(params || {}).map((k) => `${encodeURIComponent(k)}=${encodeURIComponent(params[k])}`).join("&");
    const sep = hxGet.indexOf("?") === -1 ? "?" : "&";
    fetch(hxGet + sep + (qs ? qs + "&" : "") + "_=" + Date.now())
      .then((r) => r.text())
      .then((html) => {
        const body = document.getElementById(`window-body-${id}`);
        if (body) this._injectBodyContent(id, html);
      })
      .catch(() => {
        const body = document.getElementById(`window-body-${id}`);
        if (body) this._injectBodyContent(id, `<div style="padding:20px"><h3>${title}</h3><p>Tool window failed to load.</p></div>`);
      });
  },

  // Open (or focus) a tool window and return its body element so callers can
  // build DOM directly (dialog modules, panels, modals). Content is cleared
  // on every call so re-opening a dialog never stacks stale markup.
  openToolWindowBody(id, title, opts) {
    opts = opts || {};
    this.closeStartMenu();
    const existed = this.getWindow(id) !== null;
    this.open(id, title, "", {
      tool: true,
      popup: !!opts.popup,
      ownerId: opts.ownerId || null,
      noMaximize: opts.noMaximize === true,
      top: typeof opts.top === "number" ? opts.top : undefined,
      left: typeof opts.left === "number" ? opts.left : undefined,
    });
    const body = document.getElementById(`window-body-${id}`);
    if (!existed && body && opts.htmlContent) {
      this._injectBodyContent(id, opts.htmlContent);
      return body;
    }
    return body;
  },

  // VB6-style floating confirmation (no native confirm/alert modals).
  // Opens a small tool window; onYes/onNo are called on button press.
  confirmFloating(title, message, onYes, onNo, yesLabel) {
    const html =
      '<div class="gb-confirm-floating">' +
      "<p>" + String(message == null ? "" : message) + "</p>" +
      '<div class="gb-confirm-actions">' +
      '<button data-c-no class="gb-confirm-btn">Cancel</button>' +
      '<button data-c-yes class="gb-confirm-btn primary">' + (yesLabel || "OK") + "</button>" +
      "</div></div>";
    const body = this.openToolWindowBody("gb-confirm", title || "Confirm", { htmlContent: html });
    if (!body) return;
    body.querySelector("[data-c-no]").addEventListener("click", () => {
      this.close("gb-confirm");
      if (onNo) onNo();
    });
    body.querySelector("[data-c-yes]").addEventListener("click", () => {
      this.close("gb-confirm");
      if (onYes) onYes();
    });
  },

  // VB6-style floating text input (replaces native window.prompt).
  promptFloating(title, message, defaultValue, onOk) {
    const safe = String(defaultValue == null ? "" : defaultValue);
    const html =
      '<div class="gb-confirm-floating">' +
      (message ? "<p>" + String(message) + "</p>" : "") +
      '<input type="text" class="gb-prompt-input" value="' + safe.replace(/"/g, "&quot;") + '" />' +
      '<div class="gb-confirm-actions">' +
      '<button data-c-no class="gb-confirm-btn">Cancel</button>' +
      '<button data-c-yes class="gb-confirm-btn primary">OK</button>' +
      "</div></div>";
    const body = this.openToolWindowBody("gb-prompt", title || "Input", { htmlContent: html });
    if (!body) return;
    const input = body.querySelector(".gb-prompt-input");
    input.focus();
    input.select();
    const finish = (value) => {
      this.close("gb-prompt");
      if (onOk) onOk(value);
    };
    body.querySelector("[data-c-no]").addEventListener("click", () => finish(null));
    body.querySelector("[data-c-yes]").addEventListener("click", () => finish(input.value.trim()));
    input.addEventListener("keydown", (e) => {
      if (e.key === "Enter") finish(input.value.trim());
      if (e.key === "Escape") finish(null);
    });
  },

  // Issue #1160: isolated launch — open the app in its own top-level tab
  // with a fresh context. Deep-link params travel via the URL query string.
  openIsolated(appId, params) {
    const app = (window.APPS_REGISTRY || APPS_REGISTRY).find((a) => a.id === appId);
    let url = app ? app.hxGet : `/suite/partials/${appId}.html`;
    const qs = Object.keys(params || {}).map((k) => `${encodeURIComponent(k)}=${encodeURIComponent(params[k])}`).join("&");
    const sep = url.indexOf("?") === -1 ? "?" : "&";
    url = url + sep + (qs ? qs + "&" : "") + "isolated=1";
    window.open(url, "_blank", "noopener");
  },

  // Registry lookup helper shared by launchers and the widget pane.
  getApp(appId) {
    return (window.APPS_REGISTRY || APPS_REGISTRY).find((a) => a.id === appId) || null;
  },

  /* ─── Multitasking helpers (#1155) ─── */
  listWindows() {
    return this.openWindows.slice();
  },

  getWindow(id) {
    return this.openWindows.find((w) => w.id === id) || null;
  },

  minimizeWindow(id) {
    this.toggleMinimize(id);
  },

  maximizeWindow(id) {
    this.toggleMaximize(id);
  },

  focusWindow(id) {
    this.focus(id);
  },

  restoreWindow(id) {
    const obj = this.openWindows.find((w) => w.id === id);
    if (!obj) return;
    if (obj.isMinimized) this.toggleMinimize(id);
    if (obj.isMaximized) this.toggleMaximize(id);
    if (obj.snapLayout) {
      const el = document.getElementById(`window-${id}`);
      if (el && obj.previousState) {
        el.style.left = obj.previousState.left;
        el.style.top = obj.previousState.top;
        el.style.width = obj.previousState.width;
        el.style.height = obj.previousState.height;
        delete obj.snapLayout;
      }
    }
    this.focus(id);
  },

  // Snap-assist geometry (#1155): lay the window out per a named layout
  // (halves, quarters, thirds) relative to the viewport.
  snapWindow(id, layout) {
    const obj = this.openWindows.find((w) => w.id === id);
    if (!obj) return;
    const el = document.getElementById(`window-${id}`);
    if (!el) return;
    const L = {
      left: { x: 0, y: 0, w: 0.5, h: 1 },
      right: { x: 0.5, y: 0, w: 0.5, h: 1 },
      top: { x: 0, y: 0, w: 1, h: 0.5 },
      bottom: { x: 0, y: 0.5, w: 1, h: 0.5 },
      "top-left": { x: 0, y: 0, w: 0.5, h: 0.5 },
      "top-right": { x: 0.5, y: 0, w: 0.5, h: 0.5 },
      "bottom-left": { x: 0, y: 0.5, w: 0.5, h: 0.5 },
      "bottom-right": { x: 0.5, y: 0.5, w: 0.5, h: 0.5 },
      "third-left": { x: 0, y: 0, w: 1 / 3, h: 1 },
      "third-right": { x: 1 / 3, y: 0, w: 2 / 3, h: 1 },
    };
    const l = L[layout];
    if (!l) return;
    if (obj.isMaximized) this.toggleMaximize(id);
    const pad = 4;
    el.style.left = `${l.x * window.innerWidth + pad}px`;
    el.style.top = `${l.y * window.innerHeight + pad}px`;
    el.style.width = `${l.w * window.innerWidth - pad * 2}px`;
    el.style.height = `${l.h * window.innerHeight - pad * 2}px`;
    el.style.borderRadius = "0";
    obj.isMaximized = false;
    obj.snapLayout = layout;
    obj.previousState = { width: el.style.width, height: el.style.height, top: el.style.top, left: el.style.left };
    this.focus(id);
  },
  });
})(window);

// #2172 — plumbing for VM streaming deltas. The WS layer translates a
// `gb-doc-delta` server event into this CustomEvent; here we forward it to
// the target app window so its renderer can apply ops live. Apps opt in by
// listening for `gb:deep-link` with a `delta` param (existing contract).
(function () {
  document.addEventListener("gb-doc-delta", function (e) {
    var d = (e && e.detail) || {};
    if (!d.app) return;
    if (window.WindowManager && typeof window.WindowManager.openDeepLink === "function") {
      // Reuse the open window; params carry the delta batch.
      window.WindowManager.openDeepLink(d.app, { delta: JSON.stringify(d), file: d.file, bucket: d.bucket }, { new: false });
    }
  });
})();
