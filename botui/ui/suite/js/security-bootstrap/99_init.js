(function (window, document) {
  "use strict";
  var N = document.__gb_mod_security_bootstrap;
  if (!N) return;
  var GBSecurity = N.GBSecurity;
  Object.assign(GBSecurity, {
    initXHRInterceptor: function () {
      var self = this;
      var originalOpen = XMLHttpRequest.prototype.open;
      var originalSend = XMLHttpRequest.prototype.send;
      var originalSetHeader = XMLHttpRequest.prototype.setRequestHeader;

      XMLHttpRequest.prototype.setRequestHeader = function (name, value) {
        // Do not let explicit credentials reach a third-party origin.
        var sensitive = ["authorization", "x-session-id", "x-csrf-token"];
        if (this._gbUrl && !self.isSameOrigin(this._gbUrl) &&
            sensitive.indexOf(String(name).toLowerCase()) !== -1) {
          return;
        }
        this._gbHeaders = this._gbHeaders || {};
        this._gbHeaders[String(name).toLowerCase()] = value;
        return originalSetHeader.apply(this, arguments);
      };

      XMLHttpRequest.prototype.open = function (method, url, async, user, password) {
        this._gbUrl = url;
        this._gbMethod = method;
        return originalOpen.apply(this, arguments);
      };

      XMLHttpRequest.prototype.send = function (body) {
        var xhr = this;
        var token = self.getToken();
        var sessionId = self.getSessionId();
        var headers = xhr._gbHeaders || {};
        // Never send credentials to a third-party origin.
        var sameOrigin = self.isSameOrigin(xhr._gbUrl);

        // Never duplicate an Authorization header already attached by the
        // htmx:configRequest interceptor ("Bearer A, Bearer A" breaks server
        // token parsing and silently downgrades requests to anonymous).
        if (sameOrigin && token && !this._gbSkipAuth && !headers["authorization"]) {
          try {
            this.setRequestHeader("Authorization", "Bearer " + token);
          } catch (e) {}
        }
        if (sameOrigin && sessionId && !this._gbSkipAuth && !headers["x-session-id"]) {
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

      // True when the current document IS a login page. The login host
      // serves "/login" (not "/auth/"), so both must be checked — without
      // this, expiry handlers bounce login?expired=1&redirect=<itself>
      // against themselves and build a self-nesting redirect loop.
      self.isOnAuthPage = function () {
        return (
          window.location.pathname === "/login" ||
          window.location.pathname.startsWith("/auth/")
        );
      };

      window.addEventListener("gb:auth:unauthorized", function (event) {
        var isLoginPage =
          window.location.pathname.includes("/auth/") ||
          window.location.pathname === "/login" ||
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
        if (window.__BOT_IS_PUBLIC__ === false) {
        console.log("[GBSecurity] Private bot — suppressing auth:expired dispatch");
        return;
      }
      window.dispatchEvent(
          new CustomEvent("gb:auth:expired", {
            detail: { url: event.detail.url },
          }),
        );
      });

      // Debounce guard: ~10 API calls fire during boot and each 401
      // dispatches gb:auth:expired separately.  Without a gate every
      // handler registers its own timer + promise callback; when the
      // promise resolves ALL fire simultaneously, each calling
      // clearTokens() + window.location.href → page freeze.
      var _expiredDebounce = false;

      window.addEventListener("gb:auth:expired", function (event) {
        // Never redirect away from the login page itself: re-entering
        // login?expired=1&redirect=<login url> nests the loop.
        if (self.isOnAuthPage()) {
          return;
        }

        // Check if current bot is public - if so, skip redirect.
        // If the bot is private (false), the desktop.html
        // DOMContentLoaded handler already verified the user is
        // authenticated or redirected them.  We never redirect
        // away from a private bot's desktop — stale tokens just
        // mean the user sees empty grids, not a loop.
        if (window.__BOT_IS_PUBLIC__ === true) {
          console.log("[GBSecurity] Bot is public, skipping auth redirect");
          return;
        }
        if (window.__BOT_IS_PUBLIC__ === false) {
          console.log("[GBSecurity] Bot is private, skipping auth redirect (desktop handles auth)");
          return;
        }

        // Debounce: only queue ONE redirect decision, regardless of how
        // many 401s arrived.  The bot-public check resolves once; we
        // act exactly once on its outcome.
        if (_expiredDebounce) {
          console.log("[GBSecurity] Expired event already queued, dropping duplicate");
          return;
        }
        _expiredDebounce = true;

        // If public status not yet known, wait for checkBotPublicStatus
        if (window.__BOT_IS_PUBLIC__ === undefined &&
            window.__checkBotPublicStatusPromise) {
          console.log(
            "[GBSecurity] Public status still loading, waiting...",
          );
          var timer = setTimeout(function () {
            if (window.__BOT_IS_PUBLIC__ === true || window.__BOT_IS_PUBLIC__ === false) return;
            self.clearTokens();
            sessionStorage.setItem('gb-signed-out', 'true');
            window.location.href =
              (window.GB_LOGIN_URL || "/login") + "?expired=1&redirect=" +
              encodeURIComponent(window.location.href);
          }, 5000);
          window.__checkBotPublicStatusPromise.then(function () {
            clearTimeout(timer);
            if (window.__BOT_IS_PUBLIC__ === true || window.__BOT_IS_PUBLIC__ === false) {
              console.log(
                "[GBSecurity] Bot public status resolved, skipping auth redirect",
              );
              return;
            }
            self.clearTokens();
            sessionStorage.setItem('gb-signed-out', 'true');
            window.location.href =
              (window.GB_LOGIN_URL || "/login") + "?expired=1&redirect=" +
              encodeURIComponent(window.location.href);
          });
          return;
        }

        if (window.__BOT_IS_PUBLIC__ === false) {
          console.log(
            "[GBSecurity] Bot is private, skipping expired redirect (desktop handles auth)",
          );
          return;
        }
        console.log(
          "[GBSecurity] Auth expired, clearing tokens and redirecting",
        );
        self.clearTokens();
        sessionStorage.setItem('gb-signed-out', 'true');
        window.location.href =
          (window.GB_LOGIN_URL || "/login") + "?expired=1&redirect=" +
          encodeURIComponent(window.location.href);
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
    }
  });
  GBSecurity.init();

  window.GBSecurity = GBSecurity;

  // Global accessor for backwards compatibility with legacy modules
  // that read gb-access-token from localStorage/sessionStorage directly.
  // Priority: GBSecurity closure → sessionStorage → localStorage.
  window.getGBAccessToken = function () {
    var t = window.GBSecurity && window.GBSecurity.getToken();
    if (t) return t;
    // Fresh SSO credentials and chat ?token= captures always win over
    // older session copies.
    t = localStorage.getItem("gb-access-token");
    if (t) return t;
    // The login flow stores the JWT under `token` (and sometimes `id_token`
    // or `gb_token`); the vibe/chat sub-windows read through this accessor,
    // so those keys must be honored or every sub-window API 401s.
    t = localStorage.getItem("token") || localStorage.getItem("id_token") || localStorage.getItem("gb_token");
    if (t) return t;
    t = localStorage.getItem("management_token");
    if (t) return t;
    return sessionStorage.getItem("gb-access-token") || sessionStorage.getItem("token") || null;
  };

  // Normalize the JWT into the legacy gb-access-token key on every page
  // load. Many suite apps (Drive, collab, Settings, shared Terminal/Browser,
  // automations, memory, …) read only localStorage["gb-access-token"] and
  // would otherwise 401 their APIs even though the session is valid — the
  // login flow stores the token under `token`/`id_token` and GBSecurity
  // keeps it in its closure/sessionStorage. Mirroring once here makes every
  // legacy reader work without touching each app.
  (function normalizeLegacyToken() {
    if (localStorage.getItem("gb-access-token")) return;
    var t =
      (window.GBSecurity && window.GBSecurity.getToken()) ||
      localStorage.getItem("token") ||
      localStorage.getItem("id_token") ||
      localStorage.getItem("gb_token") ||
      localStorage.getItem("management_token") ||
      sessionStorage.getItem("gb-access-token") ||
      sessionStorage.getItem("token");
    if (t) {
      try {
        localStorage.setItem("gb-access-token", t);
      } catch (e) {
        /* storage may be disabled */
      }
    }
  })();

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
  delete document.__gb_mod_security_bootstrap;
})(window, document);
