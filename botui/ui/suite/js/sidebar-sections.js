"use strict";

// GB Sidebar Sections (#1161) — concrete section renderers for the sidebar
// enhancements. Depends on `window.GBSidebarBase` (helpers in
// sidebar-enhance.js). Load order: sidebar-enhance.js → sidebar-sections.js
// → sidebar-history.js.

(function () {
  if (window.GBSidebarSections) return;

  var base = function () {
    return window.GBSidebarBase;
  };

  // ── 1. Pinned apps bar ───────────────────────────────────────

  function renderPins() {
    var host = base().byId("gbSidePins");
    if (!host) return;
    var state = base().state;
    host.innerHTML = "";
    var strip = document.createElement("div");
    strip.className = "gb-side-pins";
    state.pinned.forEach(function (id) {
      var app = base().findApp(id);
      if (!app) return;
      var item = document.createElement("div");
      item.className = "gb-side-pin";
      item.setAttribute("data-app-id", id);
      item.setAttribute("draggable", "true");
      item.title = app.title;
      item.innerHTML = '<span class="gb-side-pin-icon">' + base().iconOf(app) + "</span>";
      item.addEventListener("click", function () {
        // #1302 — clicking a pinned app OPENS it (launch behavior kept);
        // removal lives in the per-pin context menu and the Add-pin picker.
        if (window.openDeepLink) window.openDeepLink(id, {});
      });
      item.addEventListener("contextmenu", function (e) {
        e.preventDefault();
        openUnpinMenu(id, e.clientX, e.clientY);
      });
      item.addEventListener("dragstart", function (e) {
        e.dataTransfer.setData("text/plain", id);
        e.dataTransfer.effectAllowed = "move";
        item.classList.add("dragging");
      });
      item.addEventListener("dragend", function () {
        item.classList.remove("dragging");
      });
      strip.appendChild(item);
    });
    var plus = document.createElement("div");
    plus.className = "gb-side-pin gb-side-pin-plus";
    plus.innerHTML = '<span class="gb-side-pin-icon">+</span>';
    // #1302 — the + button opens a pin picker instead of the Start Menu:
    // users clicked it expecting to ADD a pin, not to launch an app.
    plus.title = "Add pin";
    plus.addEventListener("click", function (e) {
      e.stopPropagation();
      openPinPicker(plus, e.clientX, e.clientY);
    });
    strip.appendChild(plus);
    host.appendChild(strip);
  }

  // #1302 — picker listing every registered app not yet pinned. Choosing an
  // entry PINS it (left-click never launches from this panel anymore).
  function openPinPicker(anchor, x, y) {
    closeExistingMenus();
    var state = base().state;
    var reg = window.APPS_REGISTRY || [];
    var menu = document.createElement("div");
    menu.className = "desktop-context-menu gb-launcher-menu";
    menu.style.maxHeight = "320px";
    menu.style.overflowY = "auto";
    var pinned = state.pinned.slice();
    var added = 0;
    reg.forEach(function (app) {
      if (pinned.indexOf(app.id) !== -1) return;
      added++;
      var entry = document.createElement("div");
      entry.className = "desktop-context-item";
      entry.textContent = app.title || app.id;
      entry.addEventListener("click", function () {
        state.pinned.push(app.id);
        base().savePins();
        renderPins();
        menu.remove();
      });
      menu.appendChild(entry);
    });
    if (!added) {
      var empty = document.createElement("div");
      empty.className = "desktop-context-item";
      empty.style.opacity = "0.6";
      empty.textContent = "All apps are already pinned";
      menu.appendChild(empty);
    }
    document.body.appendChild(menu);
    var mw = menu.offsetWidth || 210;
    var mh = menu.offsetHeight || 200;
    var left = Math.max(8, Math.min(x, window.innerWidth - mw - 8));
    var top = Math.max(8, Math.min(y, window.innerHeight - mh - 8));
    menu.style.left = left + "px";
    menu.style.top = top + "px";
    document.addEventListener("click", function dismiss() { menu.remove(); }, { once: true });
  }

  function closeExistingMenus() {
    var menus = document.querySelectorAll(".gb-launcher-menu");
    for (var i = 0; i < menus.length; i++) menus[i].remove();
  }

  function openUnpinMenu(appId, x, y) {
    var menu = document.createElement("div");
    menu.className = "desktop-context-menu gb-launcher-menu";
    menu.style.left = Math.min(x, window.innerWidth - 200) + "px";
    menu.style.top = Math.min(y, window.innerHeight - 130) + "px";
    var unpin = document.createElement("div");
    unpin.className = "desktop-context-item";
    unpin.textContent = "Unpin from sidebar";
    unpin.addEventListener("click", function () {
      base().state.pinned = base().state.pinned.filter(function (id) {
        return id !== appId;
      });
      base().savePins();
      renderPins();
      menu.remove();
    });
    menu.appendChild(unpin);
    document.body.appendChild(menu);
    document.addEventListener(
      "click",
      function dismiss() {
        menu.remove();
      },
      { once: true }
    );
  }

  function bindPinReorder() {
    var host = base().byId("gbSidePins");
    if (!host) return;
    host.addEventListener("dragover", function (e) {
      e.preventDefault();
      var dragged = e.dataTransfer.getData("text/plain");
      var target = e.target.closest(".gb-side-pin");
      if (!dragged || !target) return;
      var toId = target.getAttribute("data-app-id");
      if (!toId || dragged === toId) return;
      var state = base().state;
      var from = state.pinned.indexOf(dragged);
      var to = state.pinned.indexOf(toId);
      if (from < 0 || to < 0) return;
      state.pinned.splice(from, 1);
      state.pinned.splice(to, 0, dragged);
      base().savePins();
      renderPins();
    });
  }

  // ── 4. Quick files ───────────────────────────────────────────

  function renderQuickFiles() {
    var host = base().byId("gbQuickFiles");
    if (!host) return;
    fetch("/api/files/recent", { headers: base().authHeaders() })
      .then(function (r) {
        if (!r.ok) throw new Error("recent files");
        return r.json();
      })
      .then(function (data) {
        host.innerHTML = "";
        var files = Array.isArray(data)
          ? data
          : data && Array.isArray(data.files)
            ? data.files
            : [];
        files.slice(0, 5).forEach(function (f) {
          var fileName = String(f.name || f.path || "");
          var appId = appForFile(fileName);
          var item = document.createElement("div");
          item.className = "gb-file-item";
          item.title = fileName;
          item.innerHTML =
            '<span class="gb-file-icon">' + fileIcon(fileName) + "</span>" +
            '<span class="gb-file-name"></span>';
          item.querySelector(".gb-file-name").textContent = fileName;
          item.addEventListener("click", function () {
            if (window.openDeepLink) {
              window.openDeepLink(appId, { path: fileName });
            }
          });
          host.appendChild(item);
        });
        if (!files.length) hideSection(host);
      })
      .catch(function () {
        hideSection(host);
      });
  }

  function hideSection(host) {
    var sec = host.closest(".gb-side-sec");
    if (sec) sec.style.display = "none";
  }

  function appForFile(name) {
    var lower = name.toLowerCase();
    if (/\.(xlsx?|csv)$/.test(lower)) return "sheet";
    if (/\.(docx?|odt|pdf|txt)$/.test(lower)) return "docs";
    if (/\.bas$/.test(lower)) return "bas-editor";
    if (/\.(mp4|mp3|wav|webm|avi)$/.test(lower)) return "player";
    return "drive";
  }

  function fileIcon(name) {
    var app = appForFile(name);
    var glyph =
      app === "sheet" ? "▦" : app === "docs" ? "▧" : app === "bas-editor" ? "▶" : app === "player" ? "▶" : "▧";
    return glyph;
  }

  // ── 5. Quick actions ─────────────────────────────────────────

  function renderActions() {
    var host = base().byId("gbQuickActions");
    if (!host) return;
    var actions = [
      {
        label: "New conversation",
        run: function () {
          var btn = base().byId("sidebarNewChatBtn");
          if (btn) btn.click();
        },
      },
      {
        // #1303 — Explorer replaces Terminal in the quick actions.
        label: "Explorer",
        run: function () {
          if (window.openDeepLink) window.openDeepLink("drive", {});
        },
      },
      {
        label: "Settings",
        run: function () {
          if (window.openDeepLink) window.openDeepLink("settings", {});
        },
      },
      {
        // #1303 — Vibe replaces the New bot shortcut.
        label: "Vibe",
        run: function () {
          if (window.openDeepLink) window.openDeepLink("vibe", {});
        },
      },
    ];
    host.innerHTML = "";
    actions.forEach(function (action) {
      var chip = document.createElement("button");
      chip.className = "gb-side-action";
      chip.textContent = action.label;
      chip.addEventListener("click", action.run);
      host.appendChild(chip);
    });
  }

  // ── 6. Rich user card ────────────────────────────────────────

  function enhanceUserCard() {
    var card = document.querySelector(".chat-sidebar-user");
    if (!card || card.dataset.enhanced) return;
    card.dataset.enhanced = "1";
    var meta = document.createElement("div");
    meta.className = "gb-user-meta";
    var userInfo = readUserInfo();
    meta.textContent = userInfo.email || "Signed in";
    card.appendChild(meta);
    var dot = document.createElement("span");
    dot.className = "gb-user-dot";
    dot.title = "Online";
    if (userInfo.role) dot.textContent = userInfo.role;
    card.appendChild(dot);
    card.addEventListener("click", function () {
      toggleUserMenu(card, userInfo);
    });
  }

  function readUserInfo() {
    try {
      var raw = localStorage.getItem("gb-user") || "";
      if (raw) return JSON.parse(raw);
    } catch (e) {}
    return {};
  }

  function toggleUserMenu(card, info) {
    var existing = base().getById("gbUserMenu");
    if (existing) {
      existing.remove();
      return;
    }
    var menu = document.createElement("div");
    menu.id = "gbUserMenu";
    menu.className = "desktop-context-menu gb-launcher-menu";
    var items = [
      {
        label: "Settings",
        run: function () {
          if (window.openDeepLink) window.openDeepLink("settings", {});
        },
      },
      {
        label: "Billing",
        run: function () {
          if (window.openDeepLink) window.openDeepLink("billing", {});
        },
      },
      {
        label: "Logout",
        run: function () {
          // Full logout: same surface as sidebar.js — clears the suite
          // token, the cloud management session and the GBSecurity
          // closure, or the next page load re-authenticates from the
          // surviving management_token (login → logout → login bug).
          sessionStorage.setItem("gb-signed-out", "true");
          fetch("/api/auth/logout", { method: "POST" }).finally(function () {
            ["gb-access-token", "management_token", "management_email",
             "management_name", "management_is_admin", "gb-selected-bot",
             "gb_selected_bot", "token", "id_token", "gb_token"]
              .forEach(function (k) {
                localStorage.removeItem(k);
                sessionStorage.removeItem(k);
              });
            var keys = [];
            for (var i = 0; i < localStorage.length; i++) {
              var k = localStorage.key(i);
              if (k && k.indexOf("gb_chat_") === 0) keys.push(k);
            }
            keys.forEach(function (k) { localStorage.removeItem(k); });
            if (window.GBSecurity) {
              if (window.GBSecurity.clearTokens) window.GBSecurity.clearTokens();
              if (window.GBSecurity.broadcastLogout) window.GBSecurity.broadcastLogout();
            }
            var login = window.GB_LOGIN_URL || "/login";
            window.location.href =
              login + "?redirect=" + encodeURIComponent(window.location.href);
          });
        },
      },
    ];
    items.forEach(function (entry) {
      var item = document.createElement("div");
      item.className = "desktop-context-item";
      item.textContent = entry.label;
      item.addEventListener("click", function () {
        menu.remove();
        entry.run();
      });
      menu.appendChild(item);
    });
    // #1294 — anchor the menu to the avatar/user card instead of a fixed
    // bottom-left offset: on tall or narrow viewports the fixed position
    // drifted far away from the card that was clicked. Measured after the
    // items are appended so the real menu size is known.
    (function positionMenu() {
      menu.style.visibility = "hidden";
      menu.style.left = "0px";
      menu.style.top = "0px";
      document.body.appendChild(menu);
      var rect = card.getBoundingClientRect();
      var realW = menu.offsetWidth || 200;
      var realH = menu.offsetHeight || 120;
      var left = Math.max(8, Math.min(rect.left, window.innerWidth - realW - 8));
      var top = rect.top - realH - 8; // default: open above the card
      if (top < 8) top = rect.bottom + 8; // not enough room above → below
      top = Math.max(8, Math.min(top, window.innerHeight - realH - 8));
      menu.style.left = left + "px";
      menu.style.top = top + "px";
      menu.style.visibility = "";
    })();
    document.addEventListener(
      "click",
      function dismiss() {
        menu.remove();
      },
      { once: true }
    );
  }

  // ── Public surface ───────────────────────────────────────────

  window.GBSidebarSections = {
    renderPins: renderPins,
    bindPinReorder: bindPinReorder,
    renderQuickFiles: renderQuickFiles,
    renderActions: renderActions,
    enhanceUserCard: enhanceUserCard,
  };
})();