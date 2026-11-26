(function (window, document) {
  "use strict";
  var N = document.__gb_mod_security_bootstrap;
  if (!N) return;
  var S = N.s;
  var GBSecurity = N.GBSecurity;
  var AUTH_KEYS = N.AUTH_KEYS;

  /* #2943 — one silent session gate for all suite fetches; the collab
   * scheduler owns the tab's single sign-in hint. */
  var NO_SESSION_PUBLIC = [
    "/api/product",
    "/api/i18n/",
    "/api/auth",
    "/api/client-errors",
  ];

  /* #2943 — the i18n prefix stays public for the catalogue endpoints, but the
   * caller-scoped preference route is denied anonymously by the server
   * (`public_routes.rs`), so it must never bypass the session gate. */
  var NO_SESSION_PRIVATE = ["/api/i18n/preference"];

  function hasSession() {
    try {
      return !!(typeof window.getGBAccessToken === "function" && window.getGBAccessToken());
    } catch (e) {
      return false;
    }
  }

  /* #2943 — only API calls can be guaranteed-401. Static suite assets and
   * HTMX fragments must load without a session (Settings fetches its index
   * through window.fetch; gating it blanked the whole window). */
  function isApiPath(url) {
    try {
      return new URL(url, window.location.href).pathname.indexOf("/api/") === 0;
    } catch (e) {
      return url.indexOf("/api/") === 0;
    }
  }

  function isPublicNoSessionPath(url) {
    if (typeof url !== "string") return false;
    var path = url;
    try {
      path = new URL(url, window.location.href).pathname;
    } catch (e) {
      // Opaque URLs still match the literal prefixes.
    }
    for (var p = 0; p < NO_SESSION_PRIVATE.length; p++) {
      var priv = NO_SESSION_PRIVATE[p];
      if (path === priv || path.indexOf(priv + "?") === 0 || path.indexOf(priv + "/") === 0) {
        return false;
      }
    }
    for (var i = 0; i < NO_SESSION_PUBLIC.length; i++) {
      var prefix = NO_SESSION_PUBLIC[i];
      // Match on a path boundary (no /api/autotask false hit).
      var suffix = prefix.charAt(prefix.length - 1) === "/" ? "" : "/";
      if (
        path === prefix ||
        path.indexOf(prefix + suffix) === 0 ||
        path.indexOf(prefix + "?") === 0
      ) {
        return true;
      }
    }
    return false;
  }

  // Minimal Response-shaped sentinel: no network, all callers keep working.
  function noSessionResponse(url) {
    return {
      ok: false,
      status: 401,
      statusText: "",
      url: url,
      headers: { get: function () { return null; } },
      json: function () { return Promise.resolve({}); },
      text: function () { return Promise.resolve(""); },
      clone: function () { return noSessionResponse(url); },
    };
  }

  Object.assign(GBSecurity, {
    init: function () {
      if (this.initialized) {
        console.warn("[GBSecurity] Already initialized");
        return;
      }

      var self = this;

      // Restore tokens from session/local storage into the memory closure
      // (Issue #575); localStorage wins over a stale sessionStorage token.
      var storedAccess =
        localStorage.getItem(AUTH_KEYS.ACCESS_TOKEN) ||
        sessionStorage.getItem(AUTH_KEYS.ACCESS_TOKEN) ||
        localStorage.getItem("management_token") || null;
      if (storedAccess) {
        S._accessToken = storedAccess;
        gbLog.debug("[GBSecurity] Access token restored from sessionStorage/localStorage");
      }
      var storedRefresh = sessionStorage.getItem(AUTH_KEYS.REFRESH_TOKEN);
      if (storedRefresh) {
        S._refreshToken = storedRefresh;
      }
      // Promote a newer localStorage token into the closure (suite-sso hop).
      var localToken = localStorage.getItem(AUTH_KEYS.ACCESS_TOKEN);
      if (localToken && localToken !== storedAccess) {
        S._accessToken = localToken;
        gbLog.debug("[GBSecurity] Promoted localStorage token over stale sessionStorage copy");
      }
      // Check stored token expiry.
      var storedExpires = sessionStorage.getItem(AUTH_KEYS.TOKEN_EXPIRES);
      if (storedExpires) {
        var expires = parseInt(storedExpires, 10);
        if (!isNaN(expires)) {
          S._tokenExpires = expires;
          // Expired: refresh now.
          if (Date.now() > expires) {
            S._accessToken = null;
            gbLog.debug("[GBSecurity] Stored token expired, will refresh");
          }
        }
      }

      this.initHTMXInterceptor();
      this.initFetchInterceptor();
      this.initXHRInterceptor();
      this.initAuthEventHandlers();

      this.initialized = true;
      gbLog.debug("[GBSecurity] Security bootstrap initialized");
      gbLog.debug(
        "[GBSecurity] Token in memory:",
        S._accessToken ? S._accessToken.substring(0, 20) + "..." : "NONE",
      );

      window.dispatchEvent(new CustomEvent("gb:security:ready"));

      // Refresh in background when possible so the access token stays fresh.
      if (S._refreshToken && !S._accessToken) {
        gbLog.debug("[GBSecurity] No access token, attempting refresh...");
        setTimeout(function () {
          self.refreshTokenFromStorage();
        }, 100);
      } else if (S._refreshToken && S._accessToken) {
        // Refresh in background even with a token.
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
        // #2943 — HTMX uses XHR, so the fetch gate cannot see it. Drop
        // guaranteed-401 private API requests before they are sent; static
        // /suite fragments stay untouched.
        if (
          !hasSession() &&
          isApiPath(event.detail.path) &&
          !self.isPublicEndpoint(event.detail.path) &&
          !isPublicNoSessionPath(event.detail.path)
        ) {
          event.preventDefault();
          return;
        }
        var token = self.getToken();
        var sessionId = self.getSessionId();

        gbLog.debug(
          "[GBSecurity] htmx:configRequest for:",
          event.detail.path,
          "token:",
          token ? token.substring(0, 20) + "..." : "NONE",
        );

        // Never overwrite an Authorization header: a second Bearer breaks
        // server-side parsing and silently degrades the request to anonymous.
        var sameOrigin = self.isSameOrigin(event.detail.path);
        if (!token) {
          console.warn(
            "[GBSecurity] NO TOKEN - request will be unauthenticated",
          );
        } else if (!sameOrigin) {
          gbLog.debug("[GBSecurity] Authorization header skipped for cross-origin request");
        } else if (!event.detail.headers["Authorization"]) {
          event.detail.headers["Authorization"] = "Bearer " + token;
          gbLog.debug("[GBSecurity] Authorization header added");
        } else {
          // Header already attached: request is authenticated.
          gbLog.debug("[GBSecurity] Authorization header already present");
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

      gbLog.debug("[GBSecurity] HTMX interceptor registered");
    },

    initFetchInterceptor: function () {
      var self = this;
      var originalFetch = window.fetch;

      window.fetch = function (input, init) {
        var url = typeof input === "string" ? input : input.url;
        init = init || {};
        init.headers = init.headers || {};

        // #2943: drop guaranteed-401 private fetches; public routes pass.
        if (
          isApiPath(url) &&
          self.isSameOrigin(url) &&
          !self.isPublicEndpoint(url) &&
          !isPublicNoSessionPath(url) &&
          !hasSession()
        ) {
          return Promise.resolve(noSessionResponse(url));
        }

        gbLog.debug(
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

      gbLog.debug("[GBSecurity] Fetch interceptor registered");
    }
  });
})(window, document);
