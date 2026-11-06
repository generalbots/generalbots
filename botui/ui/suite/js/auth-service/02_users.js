(function (window, document) {
  "use strict";
  var N = document.__gb_mod_auth_service;
  if (!N) return;
  var AuthService = N.AuthService;
  var AUTH_STORAGE_KEYS = N.AUTH_STORAGE_KEYS;
  var AUTH_ENDPOINTS = N.AUTH_ENDPOINTS;
  var USER_ENDPOINTS = N.USER_ENDPOINTS;

  Object.assign(AuthService.prototype, {
    async logout() {
      try {
        const token = this.getAccessToken();
        if (token) {
          await fetch(AUTH_ENDPOINTS.LOGOUT, {
            method: "POST",
            headers: {
              Authorization: "Bearer " + token,
              "Content-Type": "application/json",
            },
          });
        }
      } catch (error) {
        console.warn("Logout API call failed:", error);
      } finally {
        this.clearAuth();
        this.emit("logout");
        window.location.href = window.GB_LOGIN_URL || "/login";
      }
    },

    async fetchCurrentUser() {
      try {
        const token = this.getAccessToken();
        if (!token) {
          return null;
        }

        const response = await fetch(AUTH_ENDPOINTS.CURRENT_USER, {
          headers: {
            Authorization: "Bearer " + token,
          },
        });

        if (!response.ok) {
          if (response.status === 401) {
            console.log(
              "[AuthService] fetchCurrentUser got 401, clearing auth (no redirect)",
            );
            this.clearAuth();
          }
          return null;
        }

        const userData = await response.json();
        this.currentUser = userData;
        localStorage.setItem(
          AUTH_STORAGE_KEYS.USER_DATA,
          JSON.stringify(userData),
        );
        this.emit("userUpdated", userData);
        return userData;
      } catch (error) {
        console.error("Failed to fetch current user:", error);
        return null;
      }
    },

    async refreshToken() {
      const refreshToken = localStorage.getItem(
        AUTH_STORAGE_KEYS.REFRESH_TOKEN,
      );
      if (!refreshToken) {
        return false;
      }

      try {
        const response = await fetch(AUTH_ENDPOINTS.REFRESH, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            refresh_token: refreshToken,
          }),
        });

        if (!response.ok) {
          this.handleTokenExpired();
          return false;
        }

        const data = await response.json();
        if (data.access_token) {
          this.storeTokens(
            data,
            !!localStorage.getItem(AUTH_STORAGE_KEYS.REMEMBER_ME),
          );
          return true;
        }

        return false;
      } catch (error) {
        console.error("Token refresh failed:", error);
        this.handleTokenExpired();
        return false;
      }
    },

    storeTokens(data, remember) {
      const storage = remember ? localStorage : sessionStorage;

      if (data.access_token) {
        localStorage.setItem(AUTH_STORAGE_KEYS.ACCESS_TOKEN, data.access_token);
      }

      if (data.refresh_token) {
        localStorage.setItem(
          AUTH_STORAGE_KEYS.REFRESH_TOKEN,
          data.refresh_token,
        );
      }

      if (data.expires_in) {
        const expiresAt = Date.now() + data.expires_in * 1000;
        localStorage.setItem(
          AUTH_STORAGE_KEYS.TOKEN_EXPIRES,
          expiresAt.toString(),
        );
      }

      if (remember) {
        localStorage.setItem(AUTH_STORAGE_KEYS.REMEMBER_ME, "true");
      }

      this.setupTokenRefresh();
    },

    clearAuth() {
      if (this.tokenRefreshTimer) {
        clearTimeout(this.tokenRefreshTimer);
        this.tokenRefreshTimer = null;
      }

      Object.values(AUTH_STORAGE_KEYS).forEach((key) => {
        localStorage.removeItem(key);
        sessionStorage.removeItem(key);
      });

      this.currentUser = null;
    },

    handleTokenExpired() {
      // Already on a login page? Redirecting would nest
      // login?expired=1&redirect=<itself> into an infinite loop.
      if (
        window.location.pathname === "/login" ||
        window.location.pathname.startsWith("/auth/")
      ) {
        return;
      }
      this.clearAuth();
      this.emit("tokenExpired");

      // Full URL, not pathname: a relative redirect would resolve against
      // the login host after auth and bounce back to login (login loop).
      if (!window.location.pathname.startsWith("/auth/")) {
        window.location.href =
          (window.GB_LOGIN_URL || "/login") + "?expired=1&redirect=" +
          encodeURIComponent(window.location.href);
      }
    },

    getAccessToken() {
      return (
        localStorage.getItem(AUTH_STORAGE_KEYS.ACCESS_TOKEN) ||
        sessionStorage.getItem(AUTH_STORAGE_KEYS.ACCESS_TOKEN)
      );
    },

    isAuthenticated() {
      const token = this.getAccessToken();
      const expiresAt = localStorage.getItem(AUTH_STORAGE_KEYS.TOKEN_EXPIRES);

      if (!token) {
        return false;
      }

      if (expiresAt && Date.now() > parseInt(expiresAt, 10)) {
        return false;
      }

      return true;
    },

    getCurrentUser() {
      return this.currentUser;
    },

    hasRole(role) {
      if (!this.currentUser || !this.currentUser.roles) {
        return false;
      }
      return this.currentUser.roles.includes(role);
    },

    hasAnyRole(roles) {
      if (!this.currentUser || !this.currentUser.roles) {
        return false;
      }
      return roles.some((role) => this.currentUser.roles.includes(role));
    },

    isAdmin() {
      return this.hasAnyRole(["admin", "super_admin", "superadmin"]);
    },

    on(event, callback) {
      if (!this.eventListeners[event]) {
        this.eventListeners[event] = [];
      }
      this.eventListeners[event].push(callback);
    },

    off(event, callback) {
      if (!this.eventListeners[event]) {
        return;
      }
      this.eventListeners[event] = this.eventListeners[event].filter(
        (cb) => cb !== callback,
      );
    },

    emit(event, data) {
      if (!this.eventListeners[event]) {
        return;
      }
      this.eventListeners[event].forEach((callback) => {
        try {
          callback(data);
        } catch (e) {
          console.error("Event listener error:", e);
        }
      });
    },
  });

  class UserService {
    constructor(authService) {
      this.authService = authService;
    }

    async listUsers(page, perPage, search) {
      const params = new URLSearchParams();
      if (page) params.append("page", page);
      if (perPage) params.append("per_page", perPage);
      if (search) params.append("search", search);

      const url =
        USER_ENDPOINTS.LIST +
        (params.toString() ? "?" + params.toString() : "");

      const response = await fetch(url);
      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to list users");
      }
      return response.json();
    }

    async createUser(userData) {
      const response = await fetch(USER_ENDPOINTS.CREATE, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(userData),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to create user");
      }
      return response.json();
    }

    async updateUser(userId, userData) {
      const url = USER_ENDPOINTS.UPDATE.replace(":user_id", userId);
      const response = await fetch(url, {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(userData),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to update user");
      }
      return response.json();
    }

    async deleteUser(userId) {
      const url = USER_ENDPOINTS.DELETE.replace(":user_id", userId);
      const response = await fetch(url, {
        method: "DELETE",
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to delete user");
      }
      return response.json();
    }

    async getUserProfile(userId) {
      const url = USER_ENDPOINTS.PROFILE.replace(":user_id", userId);
      const response = await fetch(url);

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to get user profile");
      }
      return response.json();
    }

    async getUserRoles(userId) {
      const url = USER_ENDPOINTS.ROLES.replace(":user_id", userId);
      const response = await fetch(url);

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to get user roles");
      }
      return response.json();
    }
  }

  N.UserService = UserService;
})(window, document);
