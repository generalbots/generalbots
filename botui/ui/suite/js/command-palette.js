"use strict";

// Unified command palette (Ctrl+K) for the desktop shell.
// Supersedes the app-only Start Menu: lists every app from the backend
// catalog together with its declarative `commands[]`, so a user can open an
// app, jump straight into a deep link, or send a chat `__api_call__`.
//
// Loaded AFTER window-manager.js so it can read `window.APPS_REGISTRY` and
// reuse `window.WindowManager.launchFromMenu` / `openDeepLink`.

(function () {
  if (window.GBCommandPalette) return;
  window.GBCommandPalette = {};

  var state = {
    catalog: [],
    query: "",
    selected: 0,
    results: [],
  };

  function tr(key, fallback, params) {
    var text = window.gbT ? window.gbT(key, fallback, params) : fallback;
    // A key missing from the catalogue makes gbT return the authored fallback
    // verbatim, without substituting its {placeholders}. Do it here so a new
    // command never renders a brace in the list.
    if (params && typeof params === "object") {
      Object.keys(params).forEach(function (name) {
        text = String(text).split("{" + name + "}").join(params[name]);
      });
    }
    return text;
  }

  function escapeHtml(s) {
    return String(s == null ? "" : s)
      .replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  }

  function loadCatalog() {
    if (state.catalog.length) return Promise.resolve(state.catalog);
    return fetch("/api/apps/catalog")
      .then(function (r) { if (!r.ok) throw new Error("catalog"); return r.json(); })
      .then(function (data) {
        state.catalog = (data && Array.isArray(data.apps)) ? data.apps : [];
        return state.catalog;
      })
      .catch(function () { return state.catalog; });
  }

  function appsForQuery() {
    var q = state.query.toLowerCase().trim();
    var apps = (window.APPS_REGISTRY || []).filter(function (a) {
      // #1312 — Vibe toolwindows launch from the Vibe toolbar, not the palette.
      return a.enabled !== false && a.compiled !== false && !a.toolwindow && String(a.id).indexOf("vibe-") !== 0;
    });
    if (!apps.length) {
      apps = state.catalog.filter(function (a) { return a.enabled !== false && a.compiled !== false; });
    }
    return apps;
  }

  // Build a unified result list: matching apps and their commands.
  function computeResults(q) {
    var needle = q.toLowerCase().trim();
    var apps = appsForQuery();
    var cmdByApp = {};
    (state.catalog || []).forEach(function (a) {
      cmdByApp[a.id] = a.commands || [];
    });

    var results = [];

    apps.forEach(function (app) {
      var label = (app.title || "").toLowerCase();
      var keywords = ((app.keywords || "") + " " + (app.description || "")).toLowerCase();
      var appMatch = !needle || label.includes(needle) || keywords.includes(needle) || app.id.includes(needle);
      if (appMatch) {
        results.push({
          type: "app",
          appId: app.id,
          title: app.title || app.id,
          category: app.category,
          icon: app.icon,
          color: app.color,
          hint: tr("palette-open-app", "Open {app}", { app: app.title || app.id }),
        });
      }

      (cmdByApp[app.id] || []).forEach(function (c) {
        var cName = (c.name || "").toLowerCase();
        var cLabel = (c.label || "").toLowerCase();
        var cSummary = (c.summary || "").toLowerCase();
        var match = !needle
          || cName.includes(needle)
          || cLabel.includes(needle)
          || cSummary.indexOf(needle) !== -1
          || (app.id + " " + app.title).toLowerCase().indexOf(needle) !== -1;
        if (!match) return;
        results.push({
          type: "command",
          appId: app.id,
          appTitle: app.title || app.id,
          command: c,
          title: c.label || c.name,
          summary: c.summary,
          deepLink: c.deep_link || null,
        });
      });
    });

    // #1861 — replay surface: "Show tour: <App>" for every app that ships a
    // guided tour. Tours are local only (no network, no telemetry).
    var tourIds = (window.GBTour && typeof window.GBTour.definitionIds === "function")
      ? window.GBTour.definitionIds()
      : [];
    tourIds.forEach(function (id) {
      var app = (window.APPS_REGISTRY || []).filter(function (a) { return a.id === id; })[0] ||
        { id: id, title: id };
      var title = tr("palette-show-tour", "Show tour: {app}", { app: app.title || id });
      var haystack = (title + " " + id).toLowerCase();
      if (needle && haystack.indexOf(needle) === -1) return;
      results.push({
        type: "tour",
        appId: id,
        appTitle: app.title || id,
        title: title,
        summary: tr("palette-show-tour-hint", "Replay the guided tour"),
      });
    });

    // Sort: apps first (by title), then commands (by title).
    results.sort(function (a, b) {
      if (a.type !== b.type) return a.type === "app" ? -1 : 1;
      return (a.title || "").localeCompare(b.title || "");
    });
    return results.slice(0, 60);
  }

  function renderPalette(root) {
    var needle = state.query.toLowerCase().trim();
    var results = computeResults(needle);
    state.results = results;
    state.selected = results.length ? 0 : null;

    var html = '';
    html += '<div class="gb-palette-head">';
    html += '  <input id="gb-palette-input" type="text" placeholder="' + tr('palette-placeholder', 'Search apps and commands. Or type @ <command> to send to the assistant...') + '" autocomplete="off"/>';
    html += '</div>';
    html += '<div class="gb-palette-body" id="gb-palette-body">';
    if (!results.length) {
      html += '<div class="gb-palette-empty">' + tr("palette-empty", "No matching apps or commands") + "</div>";
    } else {
      results.forEach(function (r, i) {
        var active = (i === state.selected) ? " gb-palette-item-active" : "";
        html += '<div class="gb-palette-item' + active + '" data-index="' + i + '">';
        if (r.type === "app") {
          html += '<span class="gb-palette-icon" style="color:' + escapeHtml(r.color) + '">' + (r.icon ? '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">' + r.icon + '</svg>' : "") + '</span>';
          html += '<span class="gb-palette-label">' + escapeHtml(r.title) + '</span>';
          html += '<span class="gb-palette-kind">' + tr("palette-kind-app", "App") + "</span>";
        } else if (r.type === "tour") {
          // The title already names the app ("Show tour: Drive").
          html += '<span class="gb-palette-label"><b>' + escapeHtml(r.title) + '</b></span>';
          html += '<span class="gb-palette-kind">' + tr("palette-kind-tour", "Tour") + '</span>';
        } else {
          html += '<span class="gb-palette-icon" style="color:' + escapeHtml(r.color) + '">' + (r.icon ? '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">' + r.icon + '</svg>' : "") + '</span>';
          html += '<span class="gb-palette-label"><b>' + escapeHtml(r.title) + '</b> <span class="gb-palette-app">' + escapeHtml(r.appTitle) + '</span></span>';
          html += '<span class="gb-palette-kind">' + escapeHtml(r.command.name) + '</span>';
        }
        html += '</div>';
      });
    }
    html += '</div>';
    html += '<div class="gb-palette-footer">';
    html += tr('palette-hint', '↑↓ navigate · Enter run · Esc close · Ctrl+K toggle');
    html += '</div>';

    root.innerHTML = html;
    var input = root.querySelector("#gb-palette-input");
    if (input) {
      input.focus();
      input.value = state.query;
      var len = input.value.length;
      input.setSelectionRange(len, len);
      input.addEventListener("input", function (e) {
        state.query = e.target.value;
        renderPalette(root);
      });
      input.addEventListener("keydown", function (e) {
        onKey(e, root);
      });
    }
    root.querySelectorAll(".gb-palette-item").forEach(function (item) {
      item.addEventListener("click", function () {
        var idx = parseInt(item.getAttribute("data-index"), 10);
        runResult(idx, root);
      });
    });
  }

  function runResult(idx, overlay) {
    var r = state.results[idx];
    if (!r) return;
    closePalette();
    if (r.type === "app") {
      var app = appsForQuery().find(function (a) { return a.id === r.appId; });
      if (app && window.WindowManager) {
        window.WindowManager.launchFromMenu(app.id, app.title || app.id, app.hxGet || "/(suite/" + app.id + ".html)");
      }
      return;
    }
    if (r.type === "tour") {
      // #1861 — replay. The engine honours `force`, so a completed tour runs
      // again instead of being skipped by the persistence guard.
      if (window.GBTour && typeof window.GBTour.start === "function") {
        window.GBTour.start(r.appId, { force: true });
      }
      return;
    }
    // command: if it declares a deep link template, collect placeholders and open
    // the app contextualized; if it is a harvested endpoint (derived), send it to
    // the assistant as an api.exec command (works on web + other channels); else
    // ask the assistant in chat.
    if (r.deepLink) {
      var placeholders = (r.deepLink.match(/\{([^}]+)\}/g) || []).map(function (p) {
        return p.slice(1, -1);
      });
      if (!placeholders.length) {
        window.openDeepLink(r.appId, {});
        return;
      }
      var params = {};
      var proceed = true;
      placeholders.forEach(function (key) {
        var promptText = tr("palette-value-prompt", "{name}: {summary} — value for {key}:", {
          name: r.command.name || "",
          summary: r.summary || "",
          key: key,
        });
        var value = prompt(promptText);
        if (value === null) { proceed = false; }
        else if (value.trim() !== "") { params[key] = value.trim(); }
      });
      if (!proceed) return;
      window.openDeepLink(r.appId, params);
      return;
    }
    if (r.command && (r.command.method || r.command.derived)) {
      sendToChat(r.command);
      return;
    }
    sendToChat(r.command);
  }

  function sendToChat(cmd) {
    var input = document.getElementById("messageInput");
    if (!input) return;
    // Derived endpoint commands run through api.exec (cross-channel);
    // curated commands are asked to the assistant by name.
    var text;
    if (cmd.method && cmd.path) {
      text = tr("palette-execute", "Execute {method} {path} and summarize the result.", {
        method: cmd.method,
        path: cmd.path,
      });
    } else {
      text = tr("palette-use-command", "Use the {command} command to {summary}.", {
        command: cmd.name || "command",
        summary: cmd.summary || cmd.label || "",
      });
    }
    input.value = text;
    input.dispatchEvent(new Event("input"));
  }

  function openPalette() {
    if (window.GBCommandPalette.overlay) return; // already open
    var overlay = document.createElement("div");
    overlay.id = "gb-palette";
    overlay.className = "gb-palette";
    overlay.addEventListener("mousedown", function (e) { if (e.target === overlay) closePalette(); });
    document.body.appendChild(overlay);
    window.GBCommandPalette.overlay = overlay;
    loadCatalog().then(function () { renderPalette(overlay); });
  }

  function closePalette() {
    var overlay = window.GBCommandPalette && window.GBCommandPalette.overlay;
    if (overlay) { overlay.remove(); }
    window.GBCommandPalette.overlay = null;
  }

  function toggle() {
    if (state.results.length && window.GBCommandPalette.overlay) {
      closePalette();
    } else {
      openPalette();
    }
  }

  function onKey(e, overlay) {
    if (e.key === "Escape") { e.preventDefault(); closePalette(); }
    else if (e.key === "ArrowDown") { e.preventDefault(); move(1, overlay); }
    else if (e.key === "ArrowUp") { e.preventDefault(); move(-1, overlay); }
    else if (e.key === "Enter") { e.preventDefault(); if (state.selected != null) runResult(state.selected, overlay); }
  }

  function move(dir, overlay) {
    if (!state.results.length) return;
    state.selected = (state.selected + dir + state.results.length) % state.results.length;
    overlay.querySelectorAll(".gb-palette-item").forEach(function (item) {
      var idx = parseInt(item.getAttribute("data-index"), 10);
      item.classList.toggle("gb-palette-item-active", idx === state.selected);
    });
    overlay.querySelectorAll(".gb-palette-item-active")[0]?.scrollIntoView({ block: "nearest" });
  }

  // Guard: reuse the active palette state from the shell (window-manager) if present.

  document.addEventListener("keydown", function (e) {
    if ((e.metaKey || e.ctrlKey) && !e.shiftKey && e.key.toLowerCase() === "k") {
      e.preventDefault();
      e.stopImmediatePropagation();
      toggle();
    }
  });

  window.addEventListener("localeChanged", function () {
    if (window.GBCommandPalette.overlay) renderPalette(window.GBCommandPalette.overlay);
  });

  window.GBCommandPalette.toggle = toggle;
  window.GBCommandPalette.open = openPalette;
  window.GBCommandPalette.close = closePalette;
})();