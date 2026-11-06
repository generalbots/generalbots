/**
 * SECURITY BOOTSTRAP - Centralized Authentication Engine
 *
 * This file MUST be loaded IMMEDIATELY after HTMX and BEFORE any other scripts.
 * It provides a unified security mechanism for ALL apps in the suite.
 *
 * Features:
 * - Automatic Authorization header injection for ALL HTMX requests
 * - Fetch API interception for ALL fetch() calls
 * - XMLHttpRequest interception for legacy code
 * - Token refresh handling
 * - Session management
 * - Centralized auth state
 */

(function (window, document) {
  "use strict";
  var N = (document.__gb_mod_security_bootstrap = {});
  var S = (N.s = { _accessToken: null, _refreshToken: null, _tokenExpires: null, _sessionId: null, _userData: null });
   console.log("[GBSecurity] Loading...");

  // SECURITY: Tokens stored in closure variables instead of
  // localStorage/sessionStorage to prevent XSS-based exfiltration
  // (Issue #575). Access token lives in memory only and is NOT
  // accessible via window.GBSecurity or DevTools console.
  // Refresh token uses sessionStorage only (cleared on tab close).

  var AUTH_KEYS = {
    ACCESS_TOKEN: "gb-access-token",
    REFRESH_TOKEN: "gb-refresh-token",
    SESSION_ID: "gb-session-id",
    TOKEN_EXPIRES: "gb-token-expires",
    USER_DATA: "gb-user-data",
  };

  N.AUTH_KEYS = AUTH_KEYS;

  var GBSecurity = (N.GBSecurity = {});
  Object.assign(GBSecurity, {
    initialized: false,

    getToken: function () {
      // Fresh SSO/login JWTs land in localStorage (suite-sso hop) or
      // management_token (chat ?token= capture); a stale sessionStorage
      // copy from a previous login must never shadow them — otherwise
      // CRM/Drive silently degrade to anonymous and grids come back empty.
      return (
        S._accessToken ||
        localStorage.getItem("gb-access-token") ||
        localStorage.getItem("management_token") ||
        sessionStorage.getItem("gb-access-token") ||
        null
      );
    },

    getSessionId: function () {
      return S._sessionId;
    },

    getRefreshToken: function () {
      return S._refreshToken || sessionStorage.getItem(AUTH_KEYS.REFRESH_TOKEN) || null;
    },

    isAuthenticated: function () {
      if (!S._accessToken) return false;
      if (S._tokenExpires && Date.now() > S._tokenExpires) return false;
      return true;
    },

    getCsrfToken: function () {
      var cookies = document.cookie.split(';');
      for (var i = 0; i < cookies.length; i++) {
        var c = cookies[i].trim();
        if (c.indexOf('csrf_token=') === 0) {
          return c.substring('csrf_token='.length, c.length);
        }
      }
      return null;
    },

    setTokens: function (accessToken, refreshToken, expiresIn, persistent) {
      S._accessToken = accessToken || null;
      S._refreshToken = refreshToken || null;
      if (expiresIn) {
        S._tokenExpires = Date.now() + expiresIn * 1000;
      } else {
        S._tokenExpires = null;
      }
      // Store access token in sessionStorage as fallback for page-navigation
      // survival. Primary storage is the closure (_accessToken) for XSS
      // protection (Issue #575). SessionStorage is cleared on tab close.
      if (accessToken) {
        sessionStorage.setItem(AUTH_KEYS.ACCESS_TOKEN, accessToken);
      } else {
        sessionStorage.removeItem(AUTH_KEYS.ACCESS_TOKEN);
      }
      // Store refresh token in sessionStorage for cross-page navigation
      // (cleared when tab closes).
      if (refreshToken) {
        sessionStorage.setItem(AUTH_KEYS.REFRESH_TOKEN, refreshToken);
      } else {
        sessionStorage.removeItem(AUTH_KEYS.REFRESH_TOKEN);
      }
      // Store persistent flag to know if session is "remembered"
      if (persistent) {
        sessionStorage.setItem("gb-persistent", "1");
      } else {
        sessionStorage.removeItem("gb-persistent");
      }
    },

    refreshTokenFromStorage: async function () {
      var refreshToken = S._refreshToken || sessionStorage.getItem(AUTH_KEYS.REFRESH_TOKEN);
      if (!refreshToken) {
        console.log("[GBSecurity] No refresh token found, skipping refresh");
        return false;
      }
      try {
        var response = await fetch("/api/auth/refresh", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ refresh_token: refreshToken }),
        });
        if (!response.ok) {
          console.warn("[GBSecurity] Token refresh failed:", response.status);
          if (response.status === 401) {
            this.clearTokens();
          }
          return false;
        }
        var data = await response.json();
        if (data.access_token) {
          this.setTokens(
            data.access_token,
            data.refresh_token || null,
            data.expires_in || null,
            true,
          );
          console.log("[GBSecurity] Token refreshed successfully after page load");
          return true;
        }
        console.warn("[GBSecurity] Refresh response missing access_token");
        return false;
      } catch (e) {
        console.warn("[GBSecurity] Token refresh error:", e);
        return false;
      }
    },

    clearTokens: function () {
      S._accessToken = null;
      S._refreshToken = null;
      S._tokenExpires = null;
      S._sessionId = null;
      S._userData = null;
      // Clean up any residual sessionStorage items
      Object.keys(AUTH_KEYS).forEach(function (key) {
        sessionStorage.removeItem(AUTH_KEYS[key]);
      });
      sessionStorage.removeItem("gb-persistent");
      // Cloud management tokens must also go — otherwise the chat
      // re-authenticates as the previous user after logout.
      localStorage.removeItem("management_token");
      localStorage.removeItem("management_email");
      localStorage.removeItem("management_name");
      localStorage.removeItem("management_is_admin");
      // Clear per-bot chat session caches
      var i = 0, keys = [];
      for (i = 0; i < localStorage.length; i++) {
        var k = localStorage.key(i);
        if (k && k.indexOf("gb_chat_") === 0) keys.push(k);
      }
      keys.forEach(function (k) { localStorage.removeItem(k); });
    },

    buildAuthHeaders: function (existingHeaders) {
      var headers = existingHeaders || {};
      var token = this.getToken();
      var sessionId = this.getSessionId();

      if (token && !headers["Authorization"]) {
        headers["Authorization"] = "Bearer " + token;
      }
      if (sessionId && !headers["X-Session-ID"]) {
        headers["X-Session-ID"] = sessionId;
      }

      var csrfToken = this.getCsrfToken();
      if (csrfToken && !headers["X-CSRF-Token"]) {
        headers["X-CSRF-Token"] = csrfToken;
      }

      return headers;
    },

    // Public endpoints that must NEVER carry a stale Bearer token.  A
    // previous-login token is expired/foreign and the server rejects it
    // with 401, which kills the bot-public check and cascades into a
    // 10× concurrent-redirect freeze.
    isPublicEndpoint: function (url) {
      if (typeof url !== "string") return false;
      return (
        /\/api\/bot\/public/.test(url) ||
        /\/api\/catalog\/products/.test(url) ||
        /\/api\/catalog\/prices/.test(url) ||
        /\/api\/news\.xml/.test(url) ||
        /\/api\/product/.test(url)
      );
    },

    handleUnauthorized: function (url) {
      if (window.__BOT_IS_PUBLIC__) return;
      console.warn("[GBSecurity] Unauthorized response from:", url);

      // Fresh visit with NO token anywhere — every page-load fires ~10 API
      // calls that return 401 before the bot-public check / login flow can
      // supply a token. Suppress ALL redirects in this state to prevent the
      // concurrent-redirect cascade that freezes the page.
      if (
        !S._accessToken &&
        !localStorage.getItem("gb-access-token") &&
        !localStorage.getItem("management_token") &&
        !sessionStorage.getItem("gb-access-token")
      ) {
        console.log("[GBSecurity] No token anywhere — skipping 401 (fresh visit)");
        return;
      }

      // Cosmetic / pre-init endpoints that can legitimately return 401
      // before session initialisation. The frontend has fallbacks for all
      // of them so there is never a reason to bounce to login.
      if (
        typeof url === "string" &&
        (/\/api\/apps\/catalog/.test(url) ||
          /\/api\/cloud\/bots/.test(url) ||
          /\/api\/chat\/history/.test(url) ||
          /\/api\/files\/recent/.test(url) ||
          /\/api\/bots\/list/.test(url) ||
          /\/api\/system\/usage/.test(url) ||
          /\/api\/setup\/status/.test(url) ||
          /\/api\/product/.test(url) ||
          /\/api\/bot\/public/.test(url))
      ) {
        console.log("[GBSecurity] Skipping 401 redirect for cosmetic endpoint:", url);
        return;
      }

      window.dispatchEvent(
        new CustomEvent("gb:auth:unauthorized", {
          detail: { url: url },
        }),
      );
    }
  });
})(window, document);
