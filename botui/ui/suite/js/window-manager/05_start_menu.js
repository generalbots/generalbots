/* WindowManager — start menu, filtering and launcher (5/7). */
(function (window) {
  "use strict";
  var WM = window.__GBWindowManager;
  if (!WM) return;
  var WindowManager = WM.WindowManager;
  var APPS_REGISTRY = WM.APPS_REGISTRY;
  var CATEGORY_LABELS = WM.CATEGORY_LABELS;
  var isAlwaysAppId = WM.isAlwaysAppId;

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

    document.addEventListener("keydown", this._startMenuKeyHandler);
  },

  closeStartMenu() {
    const overlay = document.getElementById("startMenuOverlay");
    if (overlay) overlay.remove();
    document.body.classList.remove("start-menu-open");
    this.startMenuOpen = false;
    document.removeEventListener("keydown", this._startMenuKeyHandler);
  },

  _buildStartMenuHTML() {
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

    let html = `<div class="start-menu-search-wrap"><div class="start-menu-search">
      <svg viewBox="0 0 24 24" fill="none" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
      <input type="text" id="startMenuSearchInput" placeholder="Type to find an app..."/>
    </div></div>`;

    html += `<div class="start-menu-body" id="startMenuBody">`;

    if (recent.length > 0) {
      html += `<div class="start-menu-recent"><div class="start-menu-section-title">Recent</div><div class="start-menu-grid">`;
      recent.forEach(function (app) {
        if (!enabledApps || enabledApps.has(app.id)) {
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
          // #1409 — the permanently-visible apps must show regardless of
          // the product file too.
          if (a.id.startsWith("bot-") || a.id.startsWith("vibeapp-") || isAlwaysAppId(a.id)) return true;
          return enabledApps.has(a.id);
        });
      }
      if (!apps.length) return;
      html += `<div class="start-menu-category" data-category="${cat}"><div class="start-menu-section-title">${CATEGORY_LABELS[cat] || cat}</div><div class="start-menu-grid">`;
      apps.forEach(function (app) { html += this._appTileHTML(app); }.bind(this));
      html += `</div></div>`;
    }.bind(this));

    html += `<div id="startMenuEmpty" class="start-menu-filter-empty" style="display:none">No apps found</div>`;
    html += `</div>`;

    html += `<div class="start-menu-footer">
      <div class="start-menu-user"><div class="start-menu-avatar">U</div><div class="start-menu-user-name">User</div></div>
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
      if (body) this._injectBodyContent(id, `<div style="padding:20px"><h3>${title}</h3><p>Application loading...</p></div>`);
    });
  },

  // Deep-link: open an app window contextualized with query params. The app
  // HTML receives the params both as URL query args and via
  // window.__gbAppParams__ so it can select/filter the referenced record.
  // Only available in the desktop shell (web channel).
  });
})(window);
