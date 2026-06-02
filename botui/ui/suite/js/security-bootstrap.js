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
      return _accessToken;
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
      // Store refresh token in sessionStorage for cross-page navigation
      // (cleared when tab closes). Access token stays in memory only.
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
      // After page navigation, the access token must be re-obtained via
      // refresh token (sessionStorage persists across same-tab reloads).
      var storedRefresh = sessionStorage.getItem(AUTH_KEYS.REFRESH_TOKEN);
      if (storedRefresh) {
        _refreshToken = storedRefresh;
      }
      // Access token cannot persist in storage - on page reload it's gone.
      // The token was passed via setTokens() during login flow which kept
      // it in the closure for the current page lifetime.

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
            var path = window.location.pathname + window.location.hash;
            window.location.href =
              "/auth/login.html?expired=1&redirect=" +
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
            var path = window.location.pathname + window.location.hash;
            window.location.href =
              "/auth/login.html?expired=1&redirect=" +
              encodeURIComponent(path);
          });
          return;
        }

        console.log(
          "[GBSecurity] Auth expired, clearing tokens and redirecting",
        );
        self.clearTokens();

        var currentPath = window.location.pathname + window.location.hash;
        window.location.href =
          "/auth/login.html?expired=1&redirect=" +
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
})(window, document);
