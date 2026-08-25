"use strict";

// GB Sidebar (v17) — Claude/zo.computer style navigation rail.
// Top: brand, bot selector combo (all bots of the current branch), principal
// app links (Chat, Browser, Terminal, Drive). Below: conversation history for
// the selected bot. Bottom: user profile menu (moved from the taskbar).

(function () {
  // Path segments that are suite routes/apps — never treated as bot names
  // when deriving the active bot from location.pathname.
  var ROUTE_SEGMENTS = ["suite", "cloud", "login", "signup", "chat", "app",
    "ws", "ui", "api", "auth", "desktop", "sources", "integrations", "settings", "admin"];

  var FALLBACK_APPS = {
    chat: {
      id: "chat", title: "Chat", hxGet: "/suite/partials/chat.html",
      icon: '<path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/>',
    },
    browser: {
      id: "browser", title: "Browser", hxGet: "/suite/browser/browser.html",
      icon: '<circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>',
    },
    terminal: {
      id: "terminal", title: "Terminal", hxGet: "/suite/terminal/terminal.html",
      icon: '<polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/>',
    },
    drive: {
      id: "drive", title: "Drive", hxGet: "/suite/drive/drive.html",
      icon: '<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>',
    },
  };

  function findApp(id) {
    var registry = window.APPS_REGISTRY || [];
    for (var i = 0; i < registry.length; i++) {
      if (registry[i].id === id && registry[i].hxGet) return registry[i];
    }
    return FALLBACK_APPS[id];
  }

  function appIcon(app) {
    return '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">' +
      (app.icon || "") + "</svg>";
  }

  // ── Auth helpers (same slots as chat-init.js) ──
  function authToken() {
    if (window.getGBAccessToken) {
      var t = window.getGBAccessToken();
      if (t) return t;
    }
    try {
      return localStorage.getItem("gb-access-token") ||
        sessionStorage.getItem("gb-access-token") ||
        localStorage.getItem("management_token") || "";
    } catch (e) {
      return "";
    }
  }

  function authHeaders() {
    var h = {};
    var token = authToken();
    if (token) h["Authorization"] = "Bearer " + token;
    return h;
  }

  // ── Active bot resolution ────────────────────────────────────────
  // Priority: explicit deep-link param > URL path /{bot} > stored
  // selection > server-injected default. The stored selection is kept in
  // sync so a reload of "/" reopens the last selected workspace bot.
  function isKnownAppRoute(seg) {
    var registry = window.APPS_REGISTRY || [];
    for (var i = 0; i < registry.length; i++) {
      if (registry[i].id === seg) return true;
    }
    return false;
  }

  function botFromPath() {
    try {
      var seg = window.location.pathname.split("/").filter(Boolean)[0] || "";
      if (!seg || /\.(js|css|html|json|svg|png)$/.test(seg)) return "";
      if (ROUTE_SEGMENTS.indexOf(seg) !== -1) return "";
      if (isKnownAppRoute(seg)) return "";
      return seg;
    } catch (e) {
      return "";
    }
  }

  function storedBot() {
    try {
      return localStorage.getItem("gb_selected_bot") || "";
    } catch (e) {
      return "";
    }
  }

  function saveStoredBot(name) {
    try {
      if (name) localStorage.setItem("gb_selected_bot", name);
      else localStorage.removeItem("gb_selected_bot");
    } catch (e) {}
  }

  function GBResolveActiveBot() {
    var pathBot = botFromPath();
    var active = pathBot || storedBot() || window.__INITIAL_BOT_NAME__ || "default";
    if (pathBot && pathBot !== storedBot()) saveStoredBot(pathBot);
    window.__SELECTED_BOT_NAME__ = active;
    return active;
  }
  window.GBResolveActiveBot = GBResolveActiveBot;

  // ── Principal app links ──
  // Delegates to the unified apps panel (js/sidebar-apps.js): every
  // APPS_REGISTRY app in one grid with an ON/OFF switch and filter.
  function renderApps() {
    if (window.GBSidebarApps) {
      window.GBSidebarApps.render();
    }
  }

  // ── Conversations (delegated to js/sidebar-convos.js) ────────────
  function loadHistory() {
    if (window.GBSidebarConvos) window.GBSidebarConvos.loadHistory();
  }

  function highlightActive(sessionId) {
    if (window.GBSidebarConvos) {
      window.GBSidebarConvos.highlightActive(sessionId);
    }
  }

  function openConversation(sessionId) {
    if (window.GBSidebarConvos) {
      window.GBSidebarConvos.openConversation(sessionId);
    }
  }

  function newConversation() {
    if (window.openDeepLink) {
      window.openDeepLink("chat", {});
    }
    highlightActive("");
  }

  // ── User profile block (moved from the taskbar tray) ─────────────
  function getActiveUsername() {
    try {
      var token = authToken();
      if (token) {
        var parts = token.split(".");
        if (parts.length === 3) {
          var payload = JSON.parse(atob(parts[1]));
          if (payload.username) return payload.username;
          if (payload.name) return payload.name;
          if (payload.sub) return payload.sub;
        }
      }
    } catch (e) {}
    return "User";
  }

  function openSettingsWindow() {
    var m2 = document.getElementById("user-menu");
    if (m2) m2.remove();
    fetch("/suite/admin/organization-settings.html")
      .then(function (r) { return r.text(); })
      .then(function (html) {
        // organization-settings.html is a full HTML document; extract
        // only the <body> content.  Injecting a complete <!DOCTYPE html>
        // into a WindowManager body frame produces invalid nested markup.
        var bodyMatch = html.match(/<body[^>]*>([\s\S]*)<\/body>/i);
        var content = bodyMatch ? bodyMatch[1] : html;
        if (window.WindowManager) {
          window.WindowManager.open("settings", "Settings", content);
        }
      });
  }

  function openSidebarUserMenu(displayName, email) {
    var m = document.getElementById("user-menu");
    if (m) { m.remove(); return; }

    var menu = document.createElement("div");
    menu.id = "user-menu";
    menu.style.cssText = "position:fixed;left:12px;bottom:52px;background:var(--surface,#1a1a24);border:1px solid var(--border,#333);border-radius:10px;padding:4px 0;min-width:190px;z-index:99999;box-shadow:0 4px 20px rgba(0,0,0,0.5)";

    var header = document.createElement("div");
    header.style.cssText = "padding:10px 14px;font-size:12px;color:var(--text-secondary,#888);border-bottom:1px solid var(--border,#333)";
    header.innerHTML = displayName + '<br><span style="font-size:11px;color:var(--muted,#666)">' + (email || "") + "</span>";
    menu.appendChild(header);

    var settingsBtn = document.createElement("div");
    settingsBtn.style.cssText = "padding:8px 14px;cursor:pointer;font-size:13px;color:var(--text);display:flex;align-items:center;gap:8px";
    settingsBtn.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>Settings';
    settingsBtn.addEventListener("click", openSettingsWindow);
    menu.appendChild(settingsBtn);

    var sep = document.createElement("div");
    sep.style.cssText = "border-top:1px solid var(--border,#333)";
    menu.appendChild(sep);

    var logoutBtn = document.createElement("div");
    logoutBtn.style.cssText = "padding:8px 14px;cursor:pointer;font-size:13px;color:#ef4444;display:flex;align-items:center;gap:8px";
    logoutBtn.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4M16 17l5-5-5-5M21 12H9"/></svg>Logout';
    logoutBtn.addEventListener("click", function () {
      sessionStorage.setItem("gb-signed-out", "true");
      fetch("/api/auth/logout", { method: "POST" }).finally(function () {
        localStorage.removeItem("gb-access-token");
        sessionStorage.removeItem("gb-access-token");
        localStorage.removeItem("management_token");
        localStorage.removeItem("management_email");
        localStorage.removeItem("management_name");
        localStorage.removeItem("management_is_admin");
        localStorage.removeItem("gb_selected_bot");
        var keys = [];
        for (var i = 0; i < localStorage.length; i++) {
          var k = localStorage.key(i);
          if (k && k.indexOf("gb_chat_") === 0) keys.push(k);
        }
        keys.forEach(function (k) { localStorage.removeItem(k); });
        if (window.GBSecurity && window.GBSecurity.broadcastLogout) {
          window.GBSecurity.broadcastLogout();
        }
        var m3 = document.getElementById("user-menu");
        if (m3) m3.remove();
        window.location.href = window.GB_LOGIN_URL || "/login";
      });
    });
    menu.appendChild(logoutBtn);

    document.body.appendChild(menu);
    setTimeout(function () {
      document.addEventListener("click", function closeMenu(ev) {
        if (ev.target.closest && ev.target.closest("#user-menu")) return;
        document.removeEventListener("click", closeMenu);
        var m2 = document.getElementById("user-menu");
        if (m2) m2.remove();
      }, { once: true });
    }, 10);
  }

  function avatarHtml(initial) {
    return '<span class="sidebar-user-avatar">' + initial + "</span>";
  }

  function renderUserSignedIn(user) {
    var host = document.getElementById("sidebarUser");
    var collapsedHost = document.getElementById("sidebarUserCollapsed");
    var displayName = (user && (user.display_name || user.first_name || user.username)) || getActiveUsername();
    var initial = displayName.charAt(0).toUpperCase();

    if (host) {
      host.innerHTML =
        avatarHtml(initial) +
        '<span class="sidebar-user-name"></span>' +
        '<span class="sidebar-user-chevron">▾</span>';
      host.querySelector(".sidebar-user-name").textContent = displayName;
      host.onclick = function (e) {
        e.stopPropagation();
        openSidebarUserMenu(displayName, user && user.email);
      };
    }
    if (collapsedHost) {
      collapsedHost.innerHTML = avatarHtml(initial);
      collapsedHost.title = displayName;
      collapsedHost.onclick = function (e) {
        e.stopPropagation();
        openSidebarUserMenu(displayName, user && user.email);
      };
    }
  }

  function renderUserSignedOut() {
    var host = document.getElementById("sidebarUser");
    var collapsedHost = document.getElementById("sidebarUserCollapsed");
    var goLogin = function () {
      window.location.href = (window.GB_LOGIN_URL || "/login") +
        "?redirect=" + encodeURIComponent(window.location.href);
    };
    if (host) {
      host.innerHTML =
        '<span class="sidebar-user-avatar sidebar-user-avatar-anon"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg></span>' +
        '<span class="sidebar-user-name">Sign in</span>';
      host.onclick = goLogin;
    }
    if (collapsedHost) {
      collapsedHost.innerHTML = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>';
      collapsedHost.title = "Sign in";
      collapsedHost.onclick = goLogin;
    }
  }

  function refreshUser() {
    var token = authToken();
    if (!token) { renderUserSignedOut(); return; }
    fetch("/api/auth/me", { headers: authHeaders() })
      .then(function (resp) {
        if (!resp.ok) throw new Error("Not authenticated");
        return resp.json();
      })
      .then(renderUserSignedIn)
      .catch(renderUserSignedOut);
  }
  window.updateLoginButton = refreshUser;

  // ── Events from the chat app ──
  window.addEventListener("gb-chat-session-changed", function (e) {
    var sid = e && e.detail && e.detail.session_id ? String(e.detail.session_id) : "";
    if (sid) highlightActive(sid);
    loadHistory();
  });

  window.addEventListener("gb-chat-message-sent", function () {
    clearTimeout(window.__gbSidebarRefreshTimer);
    window.__gbSidebarRefreshTimer = setTimeout(loadHistory, 1500);
  });

  var toggleBtn = document.querySelector(".chat-sidebar-toggle");
  if (toggleBtn) {
    toggleBtn.addEventListener("click", loadHistory);
  }

  var newBtn = document.getElementById("sidebarNewChatBtn");
  if (newBtn) {
    newBtn.addEventListener("click", newConversation);
  }

  // Expose for programmatic use (taskbar, collapsed icons, other apps).
  window.GBSidebar = {
    loadHistory: loadHistory,
    openConversation: openConversation,
    newConversation: newConversation,
    refreshUser: refreshUser,
  };

  // Home window helper used by the collapsed rail icons.
  window.openHomeWindow = function () {
    if (!window.WindowManager) return;
    var name = getActiveUsername();
    window.WindowManager.open("home", "Home",
      '<div style="padding:32px;text-align:center;">' +
      "<h2 style=\"font-size:22px;color:var(--text);margin-bottom:8px;\">Welcome, " + name + "</h2>" +
      '<p style="font-size:15px;color:var(--text-secondary);">General Bots OS</p>' +
      '<p style="font-size:13px;color:var(--text-secondary);margin-top:16px;">Press <kbd style="background:rgba(255,255,255,0.1);padding:2px 8px;border-radius:4px;">Ctrl+K</kbd> to open command palette</p>' +
      "</div>"
    );
  };

  // ── Init ────────────────────────────────────────────────────────
  // Sidebar visibility is toggled by the existing .chat-sidebar-toggle
  // button; no extra gripper is installed on the bar.

  function init() {
    renderApps();
    refreshUser();
    loadHistory();
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
