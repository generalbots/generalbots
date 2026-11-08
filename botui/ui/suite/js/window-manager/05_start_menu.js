/* WindowManager — start menu, filtering and launcher (5/7). */
(function (window) {
  "use strict";
  var WM = window.__GBWindowManager;
  if (!WM) return;
  var WindowManager = WM.WindowManager;
  var APPS_REGISTRY = WM.APPS_REGISTRY;
  var CATEGORY_LABELS = WM.CATEGORY_LABELS;
  var isCoreAppId = WM.isCoreAppId;

  WM.defineMethods(WindowManager.prototype, {
  /* ─── START MENU ─── */
  toggleStartMenu() {
    if (this.startMenuOpen) {
      this.closeStartMenu();
    } else {
      this.openStartMenu();
    }
  },

  openStartMenu() {
    if (this.startMenuOpen) return;
    this.closeStartMenu();
    const overlay = document.createElement("div");
    overlay.id = "startMenuOverlay";
    overlay.className = "start-menu-overlay";
    overlay.onclick = (e) => { if (e.target === overlay) this.closeStartMenu(); };

    const menu = document.createElement("div");
    menu.id = "startMenu";
    menu.className = "start-menu";
    menu.innerHTML = this._buildStartMenuHTML();
    menu.addEventListener("click", (e) => e.stopPropagation());

    overlay.appendChild(menu);
    document.body.appendChild(overlay);
    document.body.classList.add("start-menu-open");
    this.startMenuOpen = true;

    // #1632 item 1 — anchor the launcher to its trigger (the Start button)
    // and clamp it to the viewport instead of relying on the overlay's
    // centering, which pushed the menu off-screen in short windows.
    const startBtn = document.getElementById("startBtn");
    if (startBtn) {
      const anchorRect = startBtn.getBoundingClientRect();
      const menuW = menu.offsetWidth || 660;
      const menuH = menu.offsetHeight || 560;
      const margin = 8;
      const left = Math.min(
        Math.max(margin, anchorRect.left),
        Math.max(margin, window.innerWidth - menuW - margin),
      );
      let top = anchorRect.top - menuH - margin;
      if (top < margin) {
        top = Math.max(margin, window.innerHeight - menuH - margin);
      }
      menu.style.position = "fixed";
      menu.style.margin = "0";
      menu.style.left = `${left}px`;
      menu.style.top = `${top}px`;
    }

    const input = menu.querySelector("#startMenuSearchInput");
    if (input) {
      setTimeout(() => input.focus(), 100);
      input.addEventListener("input", () => this._filterStartMenu());
    }

    const previewSwitch = menu.querySelector("#startMenuPreviewSwitch");
    if (previewSwitch) {
      previewSwitch.addEventListener("change", () => {
        // Persisting calls refreshAppsCatalog(), which re-dispatches
        // `gb-apps-catalog-loaded`; _refreshStartMenuBody() runs from that
        // event so the menu is not rebuilt twice.
        if (window.GBPreviewMode) window.GBPreviewMode.set(previewSwitch.checked);
        else this._refreshStartMenuBody();
      });
    }
    if (!this._startMenuPreviewHandler) {
      this._startMenuPreviewHandler = (e) => {
        if (!this.startMenuOpen) return;
        const sw = document.querySelector("#startMenuPreviewSwitch");
        const on = window.GBPreviewMode ? window.GBPreviewMode.isOn() : !!(e && e.detail && e.detail.preview);
        if (sw) sw.checked = on;
        this._refreshStartMenuBody();
      };
      window.addEventListener("gb-preview-changed", this._startMenuPreviewHandler);
      window.addEventListener("gb-apps-catalog-loaded", this._startMenuPreviewHandler);
    }

    document.addEventListener("keydown", this._startMenuKeyHandler);
  },

  _refreshStartMenuBody() {
    if (!this.startMenuOpen) return;
    const body = document.getElementById("startMenuBody");
    if (!body) return;
    const input = document.getElementById("startMenuSearchInput");
    const query = input ? input.value : "";
    // Rebuild the generated menu body so category/recent lists reflect the
    // current preview/compiled/catalog state, then restore the filter.
    const full = this._buildStartMenuHTML();
    const tmp = document.createElement("div");
    tmp.innerHTML = full;
    const nextBody = tmp.querySelector("#startMenuBody");
    if (nextBody) body.innerHTML = nextBody.innerHTML;
    if (input && query) {
      input.value = query;
      this._filterStartMenu();
    }
  },

  closeStartMenu() {
    const overlay = document.getElementById("startMenuOverlay");
    if (overlay) overlay.remove();
    document.body.classList.remove("start-menu-open");
    this.startMenuOpen = false;
    document.removeEventListener("keydown", this._startMenuKeyHandler);
  },

  _buildStartMenuHTML() {
    const tr = (key, fallback) =>
      window.gbT ? window.gbT(key, fallback) : fallback;
    const recent = this.getRecentApps();
    const categories = ["ai", "business", "office", "dev", "system"];

    // Filter apps by product config if available
    const enabledApps = window.productConfig && window.productConfig.apps
      ? new Set(window.productConfig.apps
          .map(function (a) { return a.toLowerCase(); })
          .filter(function (a) {
            return !(typeof window.isAppHidden === "function" && window.isAppHidden(a));
          }))
      : null;
    const previewOn = !!(window.GBPreviewMode && window.GBPreviewMode.isOn());

    let html = `<div class="start-menu-search-wrap"><div class="start-menu-search">
      <svg viewBox="0 0 24 24" fill="none" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
      <input type="text" id="startMenuSearchInput" placeholder="${tr('start-menu-search-placeholder', 'Type to find an app…')}"/>
    </div><label class="gb-side-switch start-menu-preview-switch" title="${tr('sidebar-preview-hint', 'Show the applications that are still in preview')}"><span class="gb-side-preview-label">${tr('sidebar-preview', 'Preview mode')}</span><input type="checkbox" id="startMenuPreviewSwitch"${previewOn ? " checked" : ""} aria-label="${tr('sidebar-preview-toggle', 'Toggle preview mode')}"><span class="gb-side-switch-slider"></span></label></div>`;

    html += `<div class="start-menu-body" id="startMenuBody">`;

    if (recent.length > 0) {
      html += `<div class="start-menu-recent"><div class="start-menu-section-title">${tr('start-menu-recent', 'Recent')}</div><div class="start-menu-grid">`;
      recent.forEach(function (app) {
        // APPS_REGISTRY is the rebuilt catalog view (#1811); allow apps it
        // surfaces (including preview apps when preview mode is on).
        var inRegistry = (window.APPS_REGISTRY || APPS_REGISTRY).some(function (a) { return a.id === app.id; });
        if (!enabledApps || enabledApps.has(app.id) || inRegistry || app.preview) {
          html += this._appTileHTML(app);
        }
      }.bind(this));
      html += `</div></div>`;
    }

    categories.forEach(function (cat) {
      var apps = (window.APPS_REGISTRY || APPS_REGISTRY).filter(function (a) {
        // #1312 — Vibe toolwindows open from the Vibe toolbar only.
        // #1386c — only apps pinned for the launcher are listed.
        return !a.toolwindow && String(a.id).indexOf("vibe-") !== 0 &&
          a.launcher_default !== false && a.category === cat;
      });
      if (enabledApps) {
        apps = apps.filter(function (a) {
          // #1289/#1291 — dynamic catalog tiles (user bots `bot-*`,
          // vibe-published apps `vibeapp-*`) never appear in the static
          // product `apps=` list; their presence in the catalog already IS
          // the server's visibility decision, so exempt them here.
          // Core surfaces show even when not listed in `apps=`; every other
          // app follows the product file (#1811). Preview apps surfaced by
          // the rebuilt registry are allowed when preview mode is on.
          if (a.id.startsWith("bot-") || a.id.startsWith("vibeapp-") || isCoreAppId(a.id)) return true;
          if (a.preview) return true;
          return enabledApps.has(a.id);
        });
      }
      if (!apps.length) return;
      html += `<div class="start-menu-category" data-category="${cat}"><div class="start-menu-section-title">${tr('category-' + cat, CATEGORY_LABELS[cat] || cat)}</div><div class="start-menu-grid">`;
      apps.forEach(function (app) { html += this._appTileHTML(app); }.bind(this));
      html += `</div></div>`;
    }.bind(this));

    html += `<div id="startMenuEmpty" class="start-menu-filter-empty" style="display:none">${tr('start-menu-no-apps', 'No apps found')}</div>`;
    html += `</div>`;

    html += `<div class="start-menu-footer">
      <div class="start-menu-user"><div class="start-menu-avatar">U</div><div class="start-menu-user-name">${tr('start-menu-user', 'User')}</div></div>
      <div class="start-menu-power" onclick="window.location.href=(window.GB_LOGIN_URL||'/login')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>
      </div>
    </div>`;

    return html;
  },

  _appTileHTML(app) {
    const iconColor = app.color || "#88ccff";
    return `<div class="start-menu-app" data-app-id="${app.id}" onclick="window.WindowManager.launchFromMenu('${app.id}', '${app.title.replace(/'/g, "\\'")}', '${app.hxGet}')">
      <div class="start-menu-app-icon" style="background:${iconColor}22;color:${iconColor}"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${app.icon}</svg></div>
      <div class="start-menu-app-label">${app.title}</div>
    </div>`;
  },

  _filterStartMenu() {
    const input = document.getElementById("startMenuSearchInput");
    const query = (input ? input.value : "").toLowerCase();
    const body = document.getElementById("startMenuBody");
    if (!body) return;
    const tiles = body.querySelectorAll(".start-menu-app");
    const categories = body.querySelectorAll(".start-menu-category");
    const recent = body.querySelector(".start-menu-recent");
    const empty = document.getElementById("startMenuEmpty");
    let anyVisible = false;

    tiles.forEach((tile) => {
      const label = (tile.querySelector(".start-menu-app-label")?.textContent || "").toLowerCase();
      const match = !query || label.includes(query);
      tile.style.display = match ? "" : "none";
      if (match) anyVisible = true;
    });

    categories.forEach((cat) => {
      const visTiles = cat.querySelectorAll(".start-menu-app[style*='display: none'], .start-menu-app:not([style])");
      let allHidden = true;
      cat.querySelectorAll(".start-menu-app").forEach((t) => {
        if (t.style.display !== "none") allHidden = false;
      });
      cat.style.display = (query && allHidden) ? "none" : "";
    });

    if (recent) {
      let recentHidden = true;
      recent.querySelectorAll(".start-menu-app").forEach((t) => {
        if (t.style.display !== "none") recentHidden = false;
      });
      recent.style.display = (query && recentHidden) ? "none" : "";
    }

    if (empty) empty.style.display = anyVisible ? "none" : "";
  },

  launchFromMenu(id, title, hxGet) {
    this.closeStartMenu();
    // #1289/#1291 — catalog tiles may carry deep-link params (chat bot
    // binding, vibe app URL). Route through openDeepLink so the params
    // reach the app window (__gbAppParams__ + gb:deep-link retarget).
    const app = (window.APPS_REGISTRY || APPS_REGISTRY).find((a) => a.id === id);
    if (app && app.deep_link_params && Object.keys(app.deep_link_params).length) {
      this.openDeepLink(id, app.deep_link_params);
      return;
    }
    const existed = this.getWindow(id) !== null;
    // The Vibe workbench keeps the glass chrome but no status bar.
    this.open(id, title, "", {
      noMaximize: id === "vibe" });
    // Never re-inject into an existing window: the app HTML declares
    // top-level consts (e.g. drive's API_BASE) and re-running it throws
    // "Identifier ... has already been declared".
    if (existed) return;
    const sep = hxGet.indexOf("?") === -1 ? "?" : "&";
    fetch(hxGet + sep + "_=" + Date.now()).then((r) => r.text()).then((html) => {
      const body = document.getElementById(`window-body-${id}`);
      if (body) this._injectBodyContent(id, html);
    }).catch(() => {
      const body = document.getElementById(`window-body-${id}`);
      if (body) this._injectBodyContent(id, `<div style="padding:20px"><h3>${title}</h3><p>${window.gbT ? window.gbT('app-loading', 'Application loading…') : 'Application loading…'}</p></div>`);
    });
  },

  // Deep-link: open an app window contextualized with query params. The app
  // HTML receives the params both as URL query args and via
  // window.__gbAppParams__ so it can select/filter the referenced record.
  // Only available in the desktop shell (web channel).
  });

  // Rebuild the launcher while it is open so a language switch applies
  // immediately (the menu is generated, not part of the translated DOM).
  window.addEventListener("localeChanged", function () {
    var wm = window.WindowManager;
    if (!wm || !wm.startMenuOpen) return;
    var current = document.getElementById("startMenuSearchInput");
    var query = current ? current.value : "";
    // Reopen instead of replacing innerHTML: the search input's listener is
    // attached in openStartMenu, so a raw rewrite would leave it dead.
    wm.closeStartMenu();
    wm.openStartMenu();
    var input = document.getElementById("startMenuSearchInput");
    if (input && query) {
      input.value = query;
      wm._filterStartMenu();
    }
  });
})(window);
