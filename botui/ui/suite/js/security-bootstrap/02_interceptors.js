(function (window, document) {
  "use strict";
  var N = document.__gb_mod_security_bootstrap;
  if (!N) return;
  var S = N.s;
  var GBSecurity = N.GBSecurity;
  var AUTH_KEYS = N.AUTH_KEYS;
  Object.assign(GBSecurity, {
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
      // Prefer localStorage over sessionStorage: freshly minted tokens
      // (suite-sso hop, login redirect) land in localStorage while
      // sessionStorage can still hold the previous (stale) session token.
      var storedAccess =
        localStorage.getItem(AUTH_KEYS.ACCESS_TOKEN) ||
        sessionStorage.getItem(AUTH_KEYS.ACCESS_TOKEN) ||
        localStorage.getItem("management_token") || null;
      if (storedAccess) {
        S._accessToken = storedAccess;
        console.log("[GBSecurity] Access token restored from sessionStorage/localStorage");
      }
      var storedRefresh = sessionStorage.getItem(AUTH_KEYS.REFRESH_TOKEN);
      if (storedRefresh) {
        S._refreshToken = storedRefresh;
      }
      // If localStorage holds a newer token than the one restored above
      // (e.g. suite-sso hop wrote localStorage after a stale sessionStorage
      // token from a previous login), promote it into the closure so every
      // request uses the freshly acquired credential.
      var localToken = localStorage.getItem(AUTH_KEYS.ACCESS_TOKEN);
      if (localToken && localToken !== storedAccess) {
        S._accessToken = localToken;
        console.log("[GBSecurity] Promoted localStorage token over stale sessionStorage copy");
      }
      // Also check token expiry from sessionStorage
      var storedExpires = sessionStorage.getItem(AUTH_KEYS.TOKEN_EXPIRES);
      if (storedExpires) {
        var expires = parseInt(storedExpires, 10);
        if (!isNaN(expires)) {
          S._tokenExpires = expires;
          // If already expired, refresh immediately
          if (Date.now() > expires) {
            S._accessToken = null;
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
        S._accessToken ? S._accessToken.substring(0, 20) + "..." : "NONE",
      );

      window.dispatchEvent(new CustomEvent("gb:security:ready"));

      // Asynchronously refresh token in background if refresh token exists.
      // This ensures we have a fresh access token even if the stored one
      // is expired or about to expire.
      if (S._refreshToken && !S._accessToken) {
        console.log("[GBSecurity] No access token, attempting refresh...");
        setTimeout(function () {
          self.refreshTokenFromStorage();
        }, 100);
      } else if (S._refreshToken && S._accessToken) {
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

        // Do NOT overwrite an existing Authorization header: htmx passes
        // event.detail.headers straight into the XHR, and the XHR interceptor
        // below would then attach a second copy (Bearer A, Bearer A), which
        // breaks server-side token parsing and silently degrades the request
        // to anonymous. Only attach when absent.
        var sameOrigin = self.isSameOrigin(event.detail.path);
        if (!token) {
          console.warn(
            "[GBSecurity] NO TOKEN - request will be unauthenticated",
          );
        } else if (!sameOrigin) {
          console.log("[GBSecurity] Authorization header skipped for cross-origin request");
        } else if (!event.detail.headers["Authorization"]) {
          event.detail.headers["Authorization"] = "Bearer " + token;
          console.log("[GBSecurity] Authorization header added");
        } else {
          // Token exists but the header is already attached (previous pass)
          // — the request is authenticated, not anonymous.
          console.log("[GBSecurity] Authorization header already present");
        }
        if (sessionId && sameOrigin) {
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

        // Public endpoints must never carry a stale Bearer token from a
        // previous login.  A mismatched/expired JWT causes a spurious 401
        // that kills the bot-public check and cascades into a freeze.
        var resolvedUrl = typeof input === "string" ? input : input.url;
        if (!self.isSameOrigin(resolvedUrl)) {
          // Strip credentials a caller may have pre-attached: they must never
          // reach a third-party origin.
          ["Authorization", "authorization", "X-Session-ID", "x-session-id",
            "X-CSRF-Token", "x-csrf-token"].forEach(function (name) {
            delete init.headers[name];
          });
        } else if (!self.isPublicEndpoint(resolvedUrl)) {
          init.headers = self.buildAuthHeaders(init.headers);
        }

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
    }
  });
})(window, document);
