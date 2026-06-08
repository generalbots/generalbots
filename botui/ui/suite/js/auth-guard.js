"use strict";
/* GBAuthGuard — front-end auth check + login redirect for Zitadel-protected apps.
 * Public API (window.GBAuthGuard):
 *   require()          — async; resolves to { id, name, email, color, token } if logged in,
 *                        or redirects to /suite/auth/login.html?return_to=<current-url> otherwise
 *   getUser()          — sync; returns parsed user from localStorage or null
 *   getToken()         — sync; returns JWT or ""
 *   isAuthenticated()  — sync; boolean
 *   logout(returnTo)   — clears storage, redirects to login
 *   loginUrl(returnTo) — builds /suite/auth/login.html URL with return_to
 *
 * Depends on auth-service.js (loaded before this file in suite_app.js order).
 */

(function (window) {
  const KEYS = {
    TOKEN: "gb-access-token",
    REFRESH: "gb-refresh-token",
    EXPIRES: "gb-token-expires",
    USER: "gb-user-data"
  };

  function read(key) {
    try { return localStorage.getItem(key) || ""; } catch (_) { return ""; }
  }

  function isExpired() {
    const exp = parseInt(read(KEYS.EXPIRES) || "0", 10);
    if (!exp) return false;
    return Date.now() > exp - 30000;
  }

  function getUser() {
    try {
      const raw = read(KEYS.USER);
      if (!raw) return null;
      const u = JSON.parse(raw);
      if (!u || !(u.id || u.sub || u.user_id)) return null;
      return {
        id: u.id || u.sub || u.user_id,
        name: u.display_name || u.name || u.email || "User",
        email: u.email || "",
        color: u.color || (window.GBCollab && window.GBCollab.helpers ? window.GBCollab.helpers.pickColor(u.id || u.sub || u.user_id) : "#3b82f6")
      };
    } catch (_) { return null; }
  }

  function getToken() { return read(KEYS.TOKEN); }

  function isAuthenticated() {
    const t = getToken();
    if (!t) return false;
    return !isExpired();
  }

  function loginUrl(returnTo) {
    const rt = returnTo || (window.location.pathname + window.location.search + window.location.hash);
    return "/suite/auth/login.html?return_to=" + encodeURIComponent(rt);
  }

  function require() {
    if (isAuthenticated()) {
      return Promise.resolve(Object.assign({ token: getToken() }, getUser()));
    }
    if (window.GBSuite && window.GBSuite.toast) {
      window.GBSuite.toast("Sessão expirada — faça login para continuar.", "warn");
    }
    window.location.href = loginUrl();
    return Promise.reject("unauthenticated");
  }

  function logout(returnTo) {
    try {
      Object.values(KEYS).forEach(function (k) { localStorage.removeItem(k); });
    } catch (_) {}
    window.location.href = loginUrl(returnTo);
  }

  function injectLoginButton(targetEl, opts) {
    if (!targetEl) return;
    if (targetEl.querySelector(".gb-login-btn")) return;
    const cfg = opts || {};
    const btnCls = cfg.cls || "gb-login-btn";
    const label = cfg.label || (isAuthenticated() ? "Conectado" : "Entrar");
    const btn = document.createElement("button");
    btn.className = btnCls;
    btn.type = "button";
    btn.style.cssText = "display:inline-flex;align-items:center;gap:6px;padding:6px 14px;border-radius:6px;border:1px solid #334155;background:#1e293b;color:#f8fafc;font-size:13px;font-weight:500;cursor:pointer;transition:background 0.15s;";
    if (isAuthenticated()) {
      const u = getUser();
      btn.innerHTML = '<span style="display:inline-block;width:8px;height:8px;border-radius:50%;background:' + (u && u.color || "#22c55e") + ';"></span>' + (u ? u.name : "Conectado");
      btn.addEventListener("mouseenter", function () { btn.style.background = "#334155"; });
      btn.addEventListener("mouseleave", function () { btn.style.background = "#1e293b"; });
      btn.addEventListener("click", function (e) {
        e.preventDefault();
        if (confirm("Sair da sessão?")) logout();
      });
    } else {
      btn.innerHTML = '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"></path><polyline points="10 17 15 12 10 7"></polyline><line x1="15" y1="12" x2="3" y2="12"></line></svg> Entrar';
      btn.addEventListener("click", function (e) {
        e.preventDefault();
        window.location.href = loginUrl();
      });
    }
    targetEl.appendChild(btn);
  }

  window.GBAuthGuard = {
    require: require,
    getUser: getUser,
    getToken: getToken,
    isAuthenticated: isAuthenticated,
    logout: logout,
    loginUrl: loginUrl,
    injectLoginButton: injectLoginButton
  };
})(window);
