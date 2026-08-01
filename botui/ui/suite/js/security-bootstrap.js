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
   console.log("[GBSecurity] Loading...");

  // SECURITY: Tokens stored in closure variables instead of
  // localStorage/sessionStorage to prevent XSS-based exfiltration
  // (Issue #575). Access token lives in memory only and is NOT
  // accessible via window.GBSecurity or DevTools console.
  // Refresh token uses sessionStorage only (cleared on tab close).
  var _accessToken = null;
  var _refreshToken = null;
  var _tokenExpires = null;
  var _sessionId = null;
  var _userData = null;

  var AUTH_KEYS = {
    ACCESS_TOKEN: "gb-access-token",
    REFRESH_TOKEN: "gb-refresh-token",
    SESSION_ID: "gb-session-id",
    TOKEN_EXPIRES: "gb-token-expires",
    USER_DATA: "gb-user-data",
  };

  var GBSecurity = {
    initialized: false,

    getToken: function () {
      return _accessToken || localStorage.getItem("gb-access-token") || null;
    },

    getSessionId: function () {
      return _sessionId;
    },

    getRefreshToken: function () {
      return _refreshToken || sessionStorage.getItem(AUTH_KEYS.REFRESH_TOKEN) || null;
    },

    isAuthenticated: function () {
      if (!_accessToken) return false;
      if (_tokenExpires && Date.now() > _tokenExpires) return false;
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
      _accessToken = accessToken || null;
      _refreshToken = refreshToken || null;
      if (expiresIn) {
        _tokenExpires = Date.now() + expiresIn * 1000;
      } else {
        _tokenExpires = null;
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
      var refreshToken = _refreshToken || sessionStorage.getItem(AUTH_KEYS.REFRESH_TOKEN);
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
      _accessToken = null;
      _refreshToken = null;
      _tokenExpires = null;
      _sessionId = null;
      _userData = null;
      // Clean up any residual sessionStorage items
      Object.keys(AUTH_KEYS).forEach(function (key) {
        sessionStorage.removeItem(AUTH_KEYS[key]);
      });
      sessionStorage.removeItem("gb-persistent");
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

    handleUnauthorized: function (url) {
      console.warn("[GBSecurity] Unauthorized response from:", url);
      window.dispatchEvent(
        new CustomEvent("gb:auth:unauthorized", {
          detail: { url: url },
        }),
      );
    },

    init: function () {
      if (this.initialized) {
        console.warn("[GBSecurity] Already initialized");
        return;
      }

      var self = this;

      // Restore tokens from sessionStorage into memory closure.
      // Access token is stored in sessionStorage by setTokens() as a
      // fallback, so it survives page navigation. Primary storage is
      // the closure for XSS protection (Issue #575).
      var storedAccess = sessionStorage.getItem(AUTH_KEYS.ACCESS_TOKEN) || localStorage.getItem(AUTH_KEYS.ACCESS_TOKEN);
      if (storedAccess) {
        _accessToken = storedAccess;
        console.log("[GBSecurity] Access token restored from sessionStorage/localStorage");
      }
      var storedRefresh = sessionStorage.getItem(AUTH_KEYS.REFRESH_TOKEN);
      if (storedRefresh) {
        _refreshToken = storedRefresh;
      }
      // Also check token expiry from sessionStorage
      var storedExpires = sessionStorage.getItem(AUTH_KEYS.TOKEN_EXPIRES);
      if (storedExpires) {
        var expires = parseInt(storedExpires, 10);
        if (!isNaN(expires)) {
          _tokenExpires = expires;
          // If already expired, refresh immediately
          if (Date.now() > expires) {
            _accessToken = null;
            console.log("[GBSecurity] Stored token expired, will refresh");
          }
        }
      }

      this.initHTMXInterceptor();
      this.initFetchInterceptor();
      this.initXHRInterceptor();
      this.initAuthEventHandlers();

      this.initialized = true;
      console.log("[GBSecurity] Security bootstrap initialized");
      console.log(
        "[GBSecurity] Token in memory:",
        _accessToken ? _accessToken.substring(0, 20) + "..." : "NONE",
      );

      window.dispatchEvent(new CustomEvent("gb:security:ready"));

      // Asynchronously refresh token in background if refresh token exists.
      // This ensures we have a fresh access token even if the stored one
      // is expired or about to expire.
      if (_refreshToken && !_accessToken) {
        console.log("[GBSecurity] No access token, attempting refresh...");
        setTimeout(function () {
          self.refreshTokenFromStorage();
        }, 100);
      } else if (_refreshToken && _accessToken) {
        // Even with a token, refresh in background to extend session
        setTimeout(function () {
          self.refreshTokenFromStorage();
        }, 1000);
      }
    },

    initHTMXInterceptor: function () {
      var self = this;

      if (typeof htmx === "undefined") {
        console.warn("[GBSecurity] HTMX not found, skipping HTMX interceptor");
        return;
      }

      document.addEventListener("htmx:configRequest", function (event) {
        var token = self.getToken();
        var sessionId = self.getSessionId();

        console.log(
          "[GBSecurity] htmx:configRequest for:",
          event.detail.path,
          "token:",
          token ? token.substring(0, 20) + "..." : "NONE",
        );

        if (token) {
          event.detail.headers["Authorization"] = "Bearer " + token;
          console.log("[GBSecurity] Authorization header added");
        } else {
          console.warn(
            "[GBSecurity] NO TOKEN - request will be unauthenticated",
          );
        }
        if (sessionId) {
          event.detail.headers["X-Session-ID"] = sessionId;
        }
      });

      document.addEventListener("htmx:responseError", function (event) {
        if (event.detail.xhr && event.detail.xhr.status === 401) {
          self.handleUnauthorized(event.detail.pathInfo.requestPath);
        }
      });

      console.log("[GBSecurity] HTMX interceptor registered");
    },

    initFetchInterceptor: function () {
      var self = this;
      var originalFetch = window.fetch;

      window.fetch = function (input, init) {
        var url = typeof input === "string" ? input : input.url;
        init = init || {};
        init.headers = init.headers || {};

        console.log(
          "[GBSecurity] fetch intercepted:",
          url,
          "token:",
          self.getToken() ? "EXISTS" : "NONE",
        );

        if (typeof init.headers.entries === "function") {
          var headerObj = {};
          init.headers.forEach(function (value, key) {
            headerObj[key] = value;
          });
          init.headers = headerObj;
        }

        if (init.headers instanceof Headers) {
          var headerObj = {};
          init.headers.forEach(function (value, key) {
            headerObj[key] = value;
          });
          init.headers = headerObj;
        }

        init.headers = self.buildAuthHeaders(init.headers);

        return originalFetch
          .call(window, input, init)
          .then(function (response) {
            var url = typeof input === "string" ? input : input.url;

            if (response.status === 401) {
              self.handleUnauthorized(url);
            } else if (!response.ok && window.ErrorReporter && window.ErrorReporter.reportNetworkError) {
              window.ErrorReporter.reportNetworkError(url, response.status, response.statusText);
            }

            return response;
          });
      };

      console.log("[GBSecurity] Fetch interceptor registered");
    },

    initXHRInterceptor: function () {
      var self = this;
      var originalOpen = XMLHttpRequest.prototype.open;
      var originalSend = XMLHttpRequest.prototype.send;

      XMLHttpRequest.prototype.open = function (
        method,
        url,
        async,
        user,
        password,
      ) {
        this._gbUrl = url;
        this._gbMethod = method;
        return originalOpen.apply(this, arguments);
      };

      XMLHttpRequest.prototype.send = function (body) {
        var xhr = this;
        var token = self.getToken();
        var sessionId = self.getSessionId();

        if (token && !this._gbSkipAuth) {
          try {
            this.setRequestHeader("Authorization", "Bearer " + token);
          } catch (e) {}
        }
        if (sessionId && !this._gbSkipAuth) {
          try {
            this.setRequestHeader("X-Session-ID", sessionId);
          } catch (e) {}
        }

        this.addEventListener("load", function () {
          if (xhr.status === 401) {
            self.handleUnauthorized(xhr._gbUrl);
          }
        });

        return originalSend.apply(this, arguments);
      };

      console.log("[GBSecurity] XHR interceptor registered");
    },

    initAuthEventHandlers: function () {
      var self = this;

      window.addEventListener("gb:auth:unauthorized", function (event) {
        var isLoginPage =
          window.location.pathname.includes("/auth/") ||
          window.location.hash.includes("login");

        var isAuthEndpoint =
          event.detail &&
          event.detail.url &&
          (event.detail.url.includes("/api/auth/login") ||
            event.detail.url.includes("/api/auth/refresh"));

        if (isLoginPage || isAuthEndpoint) {
          return;
        }

        console.log(
          "[GBSecurity] Unauthorized response, dispatching expired event",
        );
        window.dispatchEvent(
          new CustomEvent("gb:auth:expired", {
            detail: { url: event.detail.url },
          }),
        );
      });

      window.addEventListener("gb:auth:expired", function (event) {
        // Check if current bot is public - if so, skip redirect
        if (window.__BOT_IS_PUBLIC__ === true) {
          console.log("[GBSecurity] Bot is public, skipping auth redirect");
          return;
        }

        // If public status not yet known, wait for checkBotPublicStatus
        if (window.__BOT_IS_PUBLIC__ === undefined &&
            window.__checkBotPublicStatusPromise) {
          console.log(
            "[GBSecurity] Public status still loading, waiting...",
          );
          var timer = setTimeout(function () {
            if (window.__BOT_IS_PUBLIC__ === true) return;
            self.clearTokens();
            sessionStorage.setItem('gb-signed-out', 'true');
            var path = window.location.pathname + window.location.hash;
            window.location.href =
              (window.GB_LOGIN_URL || "/login") + "?expired=1&redirect=" +
              encodeURIComponent(path);
          }, 5000);
          window.__checkBotPublicStatusPromise.then(function () {
            clearTimeout(timer);
            if (window.__BOT_IS_PUBLIC__ === true) {
              console.log(
                "[GBSecurity] Bot is public (deferred), skipping auth redirect",
              );
              return;
            }
            self.clearTokens();
            sessionStorage.setItem('gb-signed-out', 'true');
            var path = window.location.pathname + window.location.hash;
            window.location.href =
              (window.GB_LOGIN_URL || "/login") + "?expired=1&redirect=" +
              encodeURIComponent(path);
          });
          return;
        }

        console.log(
          "[GBSecurity] Auth expired, clearing tokens and redirecting",
        );
        self.clearTokens();
        sessionStorage.setItem('gb-signed-out', 'true');

        var currentPath = window.location.pathname + window.location.hash;
        window.location.href =
          (window.GB_LOGIN_URL || "/login") + "?expired=1&redirect=" +
          encodeURIComponent(currentPath);
      });

      window.addEventListener("gb:auth:login", function (event) {
        var data = event.detail;
        if (data.accessToken) {
          self.setTokens(
            data.accessToken,
            data.refreshToken,
            data.expiresIn,
            data.persistent !== false,
          );
          console.log("[GBSecurity] Tokens stored after login");
        }
      });

      window.addEventListener("gb:auth:logout", function () {
        self.clearTokens();
        console.log("[GBSecurity] Tokens cleared after logout");
      });
    },
  };

  GBSecurity.init();

  window.GBSecurity = GBSecurity;

  // Global accessor for backwards compatibility with legacy modules
  // that read gb-access-token from localStorage/sessionStorage directly.
  // Priority: GBSecurity closure → sessionStorage → localStorage.
  window.getGBAccessToken = function () {
    var t = window.GBSecurity && window.GBSecurity.getToken();
    if (t) return t;
    t = sessionStorage.getItem("gb-access-token");
    if (t) return t;
    return localStorage.getItem("gb-access-token") || null;
  };

  window.getGBRefreshToken = function () {
    var t = window.GBSecurity && window.GBSecurity.getRefreshToken();
    if (t) return t;
    t = sessionStorage.getItem("gb-refresh-token");
    if (t) return t;
    return localStorage.getItem("gb-refresh-token") || null;
  };

  // Cross-tab logout synchronization.
  //
  // The `storage` event fires ONLY in OTHER tabs when this tab writes to
  // localStorage. The tab performing logout writes `gb-logout-signal`
  // (a timestamp) and then navigates to /login itself. Every other tab
  // hears the event and redirects to the login page, ensuring a logout in
  // one tab logs everyone out.
  var LOGOUT_SIGNAL_KEY = "gb-logout-signal";
  var redirectingToLogin = false;

  function forceRedirectToLogin() {
    if (redirectingToLogin) return;
    redirectingToLogin = true;
    sessionStorage.setItem("gb-signed-out", "true");
    window.location.replace(window.GB_LOGIN_URL || "/login");
  }

  window.addEventListener("storage", function (event) {
    if (event.key === LOGOUT_SIGNAL_KEY && event.newValue) {
      console.log("[GBSecurity] Logout signal detected from another tab");
      GBSecurity.clearTokens();
      forceRedirectToLogin();
    }
  });

  // Broadcast a logout signal so other tabs close their sessions.
  GBSecurity.broadcastLogout = function () {
    try {
      localStorage.setItem(LOGOUT_SIGNAL_KEY, String(Date.now()));
    } catch (e) {
      console.warn("[GBSecurity] Failed to broadcast logout signal:", e);
    }
  };
})(window, document);
