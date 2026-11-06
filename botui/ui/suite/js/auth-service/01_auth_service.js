/**
 * Authentication Service Module
 * Handles login, logout, token management, and user session state
 */

(function (window, document) {
  "use strict";
  var N = (document.__gb_mod_auth_service = {});
  const AUTH_STORAGE_KEYS = {
    ACCESS_TOKEN: "gb-access-token",
    REFRESH_TOKEN: "gb-refresh-token",
    TOKEN_EXPIRES: "gb-token-expires",
    USER_DATA: "gb-user-data",
    REMEMBER_ME: "gb-remember-me",
  };

  const AUTH_ENDPOINTS = {
    LOGIN: "/api/auth/login",
    LOGOUT: "/api/auth/logout",
    REFRESH: "/api/auth/refresh",
    CURRENT_USER: "/api/auth/me",
    VERIFY_2FA: "/api/auth/2fa/verify",
    RESEND_2FA: "/api/auth/2fa/resend",
  };

  const USER_ENDPOINTS = {
    LIST: "/api/directory/users/list",
    CREATE: "/api/directory/users/create",
    UPDATE: "/api/directory/users/:user_id/update",
    DELETE: "/api/directory/users/:user_id/delete",
    PROFILE: "/api/directory/users/:user_id/profile",
    ROLES: "/api/directory/users/:user_id/roles",
  };

  const GROUP_ENDPOINTS = {
    LIST: "/api/directory/groups/list",
    CREATE: "/api/directory/groups/create",
    UPDATE: "/api/directory/groups/:group_id/update",
    DELETE: "/api/directory/groups/:group_id/delete",
    MEMBERS: "/api/directory/groups/:group_id/members",
    ADD_MEMBER: "/api/directory/groups/:group_id/members/add",
    REMOVE_MEMBER: "/api/directory/groups/:group_id/members/remove",
  };
  class AuthService {
    constructor() {
      this.currentUser = null;
      this.tokenRefreshTimer = null;
      this.eventListeners = {};
      this.init();
    }

    init() {
      this.loadStoredUser();
      this.setupTokenRefresh();
      // NOTE: Interceptors are now handled centrally by security-bootstrap.js
      // No need to set up duplicate fetch interceptors here
    }

    loadStoredUser() {
      try {
        const userData = localStorage.getItem(AUTH_STORAGE_KEYS.USER_DATA);
        if (userData) {
          this.currentUser = JSON.parse(userData);
        }
      } catch (e) {
        console.warn("Failed to load stored user data:", e);
        this.clearAuth();
      }
    }

    setupTokenRefresh() {
      const expiresAt = localStorage.getItem(AUTH_STORAGE_KEYS.TOKEN_EXPIRES);
      if (expiresAt) {
        const expiresMs = parseInt(expiresAt, 10) - Date.now();
        const refreshMs = expiresMs - 5 * 60 * 1000;

        if (refreshMs > 0) {
          this.tokenRefreshTimer = setTimeout(() => {
            this.refreshToken();
          }, refreshMs);
        } else if (expiresMs > 0) {
          this.refreshToken();
        } else {
          console.log(
            "[AuthService] Token already expired at startup, clearing auth (no redirect)",
          );
          this.clearAuth();
        }
      }
    }

    // NOTE: setupInterceptors is deprecated - auth headers are now handled
    // centrally by security-bootstrap.js which loads before any app code.
    // This ensures ALL fetch, XHR, and HTMX requests get auth headers automatically.
    setupInterceptors() {
      // Interceptors handled by security-bootstrap.js
      console.log("[AuthService] Fetch interceptors delegated to GBSecurity");
    }

    async login(email, password, remember) {
      try {
        const response = await fetch(AUTH_ENDPOINTS.LOGIN, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            email: email,
            password: password,
            remember: remember || false,
          }),
        });

        const data = await response.json();

        if (!response.ok) {
          throw new Error(data.error || "Login failed");
        }

        if (data.requires_2fa) {
          return {
            success: false,
            requires_2fa: true,
            session_token: data.session_token,
          };
        }

        if (data.success && data.access_token) {
          this.storeTokens(data, remember);
          await this.fetchCurrentUser();
          this.emit("login", this.currentUser);
          return {
            success: true,
            redirect: data.redirect || "/",
          };
        }

        throw new Error(data.message || "Login failed");
      } catch (error) {
        console.error("Login error:", error);
        throw error;
      }
    }

    async verify2FA(sessionToken, code, trustDevice) {
      try {
        const response = await fetch(AUTH_ENDPOINTS.VERIFY_2FA, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            session_token: sessionToken,
            code: code,
            trust_device: trustDevice || false,
          }),
        });

        const data = await response.json();

        if (!response.ok) {
          throw new Error(data.error || "2FA verification failed");
        }

        if (data.success && data.access_token) {
          this.storeTokens(data, false);
          await this.fetchCurrentUser();
          this.emit("login", this.currentUser);
          return {
            success: true,
            redirect: data.redirect || "/",
          };
        }

        throw new Error(data.message || "2FA verification failed");
      } catch (error) {
        console.error("2FA verification error:", error);
        throw error;
      }
    }

    async resend2FA(sessionToken) {
      try {
        const response = await fetch(AUTH_ENDPOINTS.RESEND_2FA, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            session_token: sessionToken,
          }),
        });

        const data = await response.json();

        if (!response.ok) {
          throw new Error(data.error || "Failed to resend 2FA code");
        }

        return data;
      } catch (error) {
        console.error("Resend 2FA error:", error);
        throw error;
      }
    }
  }

  N.AUTH_STORAGE_KEYS = AUTH_STORAGE_KEYS;
  N.AUTH_ENDPOINTS = AUTH_ENDPOINTS;
  N.USER_ENDPOINTS = USER_ENDPOINTS;
  N.GROUP_ENDPOINTS = GROUP_ENDPOINTS;
  N.AuthService = AuthService;
})(window, document);
