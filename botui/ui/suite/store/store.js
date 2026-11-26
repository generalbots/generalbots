"use strict";
/* App Store (#1156): catalog browsing, install tracking (localStorage +
   server-side popularity), and one-click launch. */

(function () {
  if (window.GBAppStore) return;

  const STORAGE_KEY = "gb-installed-apps";
  let catalog = [];
  let serverInstalls = {};
  let activeCat = "all";
  let query = "";

  function readInstalled() {
    try {
      return gbStoreGet(STORAGE_KEY, []);
    } catch (e) {
      return [];
    }
  }

  function writeInstalled(list) {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(list));
    } catch (e) { window.gbLogError ? gbLogError(e, "writeInstalled") : console.error("writeInstalled", e); }
  }

  function isInstalled(id) {
    return readInstalled().indexOf(id) !== -1;
  }

  function install(id, title) {
    const list = readInstalled();
    if (list.indexOf(id) === -1) {
      list.push(id);
      writeInstalled(list);
    }
    render();
    // Server-side popularity counter; best-effort sync
    fetch("/api/apps/install", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ app_id: id }),
    }).then(function(r){
      if (!r.ok) throw new Error('HTTP ' + r.status);
    }).catch(function (err) {
      console.error("[gb-store] App install request failed:", err);
      if (window.GBToasts && window.GBToasts.show) window.GBToasts.show('Store', 'Install saved locally; server sync failed.', 'error');
    });
    window.dispatchEvent(new CustomEvent("gb-app-installed", { detail: { id, title } }));
    window.dispatchEvent(new CustomEvent("gb-launcher-pin-request", { detail: { kind: "app", appId: id } }));
  }

  function uninstall(id, title) {
    const list = readInstalled().filter(function (x) { return x !== id; });
    writeInstalled(list);
    render();
    fetch("/api/apps/uninstall", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ app_id: id }),
    }).catch(function (err) {
      console.error("[gb-store] App uninstall request failed:", err);
    });
    window.dispatchEvent(new CustomEvent("gb-app-uninstalled", { detail: { id, title } }));
  }

  function openApp(app) {
    if (window.WindowManager) {
      window.WindowManager.open(app.id, app.title, "");
      const sep = app.url.indexOf("?") === -1 ? "?" : "&";
      fetch(app.url + sep + "_=" + Date.now())
        .then(function (r) {
          if (!r.ok) throw new Error('HTTP ' + r.status);
          return r.text();
        })
        .then(function (html) {
          const body = document.getElementById("window-body-" + app.id);
          if (body && window.WindowManager._injectBodyContent) {
            window.WindowManager._injectBodyContent(app.id, html);
          }
        })
        .catch(function (err) {
          console.error("[gb-store] Failed to load app content:", err);
          if (window.GBToasts && window.GBToasts.show) window.GBToasts.show('Store', 'Failed to open app: ' + (err && err.message ? err.message : 'error'), 'error');
        });
    } else {
      var w = window.open(app.url, "_blank", "noopener,noreferrer"); if (w) { try { w.opener = null; } catch (e) { window.gbLogError ? gbLogError(e, "openApp") : console.error("openApp", e); } }
    }
  }

  function loadCatalog() {
    const grid = document.getElementById("storeGrid");
    if (!grid) return;
    grid.innerHTML = '<div class="store-loading">Loading the store…</div>';
    Promise.all([
      fetch("/api/apps/catalog").then(function (r) { return r.json(); }),
      fetch("/api/apps/install/stats").then(function (r) { return r.json(); }).catch(function () { return { installs: {} }; }),
    ])
      .then(function (results) {
        const data = results[0];
        const stats = results[1] || {};
        serverInstalls = stats.installs || {};
        catalog = (data.apps || [])
          .filter(function (a) { return a.enabled !== false && a.compiled !== false && a.kind !== "widget"; })
          .map(function (a) {
            return {
              id: a.id,
              title: a.title,
              category: a.category,
              color: a.color,
              url: a.url,
              description: a.description,
              icon: a.icon,
            };
          });
        renderCats();
        render();
      })
      .catch(function () {
        grid.innerHTML = '<div class="store-empty">Store unavailable — check the API.</div>';
      });
  }

  function renderCats() {
    const nav = document.getElementById("storeCats");
    if (!nav) return;
    const cats = { all: "All" };
    catalog.forEach(function (a) {
      if (!cats[a.category]) cats[a.category] = a.category;
    });
    nav.innerHTML = Object.keys(cats)
      .map(function (key) {
        return '<button class="store-cat" data-cat="' + key + '">' + cats[key] + "</button>";
      })
      .join("");
    Array.from(nav.querySelectorAll(".store-cat")).forEach(function (btn) {
      // #2965 — never dereference classList on a missing/stale node.
      safeToggleClass(btn, "active", btn.dataset.cat === activeCat);
      btn.addEventListener("click", function () {
        activeCat = btn.dataset.cat;
        renderCats();
        render();
      });
    });
  }

  function render() {
    const grid = document.getElementById("storeGrid");
    if (!grid) return;
    const q = query.toLowerCase();
    const list = catalog.filter(function (a) {
      if (activeCat !== "all" && a.category !== activeCat) return false;
      if (!q) return true;
      return (a.title + " " + a.description + " " + a.keywords).toLowerCase().indexOf(q) !== -1;
    });
    if (!list.length) {
      grid.innerHTML = '<div class="store-empty">No apps match your search.</div>';
      return;
    }
    grid.innerHTML = list
      .map(function (a) {
        const installed = isInstalled(a.id);
        const count = serverInstalls[a.id] || 0;
        const iconColor = a.color || "#88ccff";
        return (
          '<div class="store-card">' +
          '<div class="store-card-top">' +
          '<div class="store-app-icon" style="background:' + iconColor + "22;color:" + iconColor + '"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">' + (a.icon || "") + "</svg></div>" +
          '<div><div class="store-app-name">' + escapeHtml(a.title) + '</div><div class="store-app-cat">' + escapeHtml(a.category) + "</div></div>" +
          "</div>" +
          '<div class="store-app-desc">' + escapeHtml(a.description || "") + "</div>" +
          '<div class="store-card-actions">' +
          (installed
            ? '<button class="store-install-btn installed" data-uninstall="' + escapeHtml(a.id) + '" title="Click to uninstall">Installed</button>'
            : '<button class="store-install-btn" data-install="' + escapeHtml(a.id) + '">Install</button>') +
          '<button class="store-open-btn" data-open="' + escapeHtml(a.id) + '">Open</button>' +
          '<span class="store-installs">' + (count || "") + (count ? " installs" : "") + "</span>" +
          "</div>" +
          "</div>"
        );
      })
      .join("");
    Array.from(grid.querySelectorAll("[data-install]")).forEach(function (btn) {
      btn.addEventListener("click", function () {
        const id = btn.dataset.install;
        const app = catalog.find(function (a) { return a.id === id; });
        if (app) install(id, app.title);
      });
    });
    Array.from(grid.querySelectorAll("[data-uninstall]")).forEach(function (btn) {
      btn.addEventListener("click", function () {
        const id = btn.dataset.uninstall;
        const app = catalog.find(function (a) { return a.id === id; });
        uninstall(id, app ? app.title : id);
      });
    });
    Array.from(grid.querySelectorAll("[data-open]")).forEach(function (btn) {
      btn.addEventListener("click", function () {
        const app = catalog.find(function (a) { return a.id === btn.dataset.open; });
        if (app) openApp(app);
      });
    });
  }

  function escapeHtml(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
  }

  // #2965 — the store is injected as an HTMX fragment, so a shell
  // re-render can leave a stale element reference behind. A direct
  // `el.classList.*` on that node throws "Cannot read properties of null
  // (reading 'classList')". Every class toggle goes through this guard.
  function safeToggleClass(el, cls, on) {
    if (!el || !el.classList) return;
    if (on) el.classList.add(cls);
    else el.classList.remove(cls);
  }

  function bootStore() {
    const search = document.getElementById("storeSearch");
    if (search && !search.dataset.gbStoreInit) {
      search.dataset.gbStoreInit = "1";
      search.addEventListener("input", function () {
        query = search.value.trim();
        render();
      });
    }
    loadCatalog();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", bootStore, { once: true });
  } else {
    bootStore();
  }

  window.GBAppStore = { loadCatalog: loadCatalog, render: render, install: install, uninstall: uninstall, isInstalled: isInstalled };
})();