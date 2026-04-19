/**
 * Authentication Service Module
 * Handles login, logout, token management, and user session state
 */

(function (window) {
  "use strict";

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
        window.location.href = "/auth/login.html";
      }
    }

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
    }

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
    }

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
    }

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
    }

    handleTokenExpired() {
      this.clearAuth();
      this.emit("tokenExpired");

      const currentPath = window.location.pathname + window.location.hash;
      if (!window.location.pathname.startsWith("/auth/")) {
        window.location.href =
          "/auth/login.html?expired=1&redirect=" +
          encodeURIComponent(currentPath);
      }
    }

    getAccessToken() {
      return (
        localStorage.getItem(AUTH_STORAGE_KEYS.ACCESS_TOKEN) ||
        sessionStorage.getItem(AUTH_STORAGE_KEYS.ACCESS_TOKEN)
      );
    }

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
    }

    getCurrentUser() {
      return this.currentUser;
    }

    hasRole(role) {
      if (!this.currentUser || !this.currentUser.roles) {
        return false;
      }
      return this.currentUser.roles.includes(role);
    }

    hasAnyRole(roles) {
      if (!this.currentUser || !this.currentUser.roles) {
        return false;
      }
      return roles.some((role) => this.currentUser.roles.includes(role));
    }

    isAdmin() {
      return this.hasAnyRole(["admin", "super_admin", "superadmin"]);
    }

    on(event, callback) {
      if (!this.eventListeners[event]) {
        this.eventListeners[event] = [];
      }
      this.eventListeners[event].push(callback);
    }

    off(event, callback) {
      if (!this.eventListeners[event]) {
        return;
      }
      this.eventListeners[event] = this.eventListeners[event].filter(
        (cb) => cb !== callback,
      );
    }

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
    }
  }

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

  class GroupService {
    constructor(authService) {
      this.authService = authService;
    }

    async listGroups(page, perPage, search) {
      const params = new URLSearchParams();
      if (page) params.append("page", page);
      if (perPage) params.append("per_page", perPage);
      if (search) params.append("search", search);

      const url =
        GROUP_ENDPOINTS.LIST +
        (params.toString() ? "?" + params.toString() : "");

      const response = await fetch(url);
      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to list groups");
      }
      return response.json();
    }

    async createGroup(groupData) {
      const response = await fetch(GROUP_ENDPOINTS.CREATE, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(groupData),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to create group");
      }
      return response.json();
    }

    async updateGroup(groupId, groupData) {
      const url = GROUP_ENDPOINTS.UPDATE.replace(":group_id", groupId);
      const response = await fetch(url, {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(groupData),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to update group");
      }
      return response.json();
    }

    async deleteGroup(groupId) {
      const url = GROUP_ENDPOINTS.DELETE.replace(":group_id", groupId);
      const response = await fetch(url, {
        method: "DELETE",
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to delete group");
      }
      return response.json();
    }

    async getGroupMembers(groupId) {
      const url = GROUP_ENDPOINTS.MEMBERS.replace(":group_id", groupId);
      const response = await fetch(url);

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to get group members");
      }
      return response.json();
    }

    async addGroupMember(groupId, userId, roles) {
      const url = GROUP_ENDPOINTS.ADD_MEMBER.replace(":group_id", groupId);
      const response = await fetch(url, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          user_id: userId,
          roles: roles || [],
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to add group member");
      }
      return response.json();
    }

    async removeGroupMember(groupId, userId) {
      const url = GROUP_ENDPOINTS.REMOVE_MEMBER.replace(":group_id", groupId);
      const response = await fetch(url, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          user_id: userId,
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || "Failed to remove group member");
      }
      return response.json();
    }
  }

  const authService = new AuthService();
  const userService = new UserService(authService);
  const groupService = new GroupService(authService);

  window.AuthService = authService;
  window.UserService = userService;
  window.GroupService = groupService;

  window.GBAuth = {
    service: authService,
    users: userService,
    groups: groupService,

    login: function (email, password, remember) {
      return authService.login(email, password, remember);
    },

    logout: function () {
      return authService.logout();
    },

    isAuthenticated: function () {
      return authService.isAuthenticated();
    },

    getCurrentUser: function () {
      return authService.getCurrentUser();
    },

    hasRole: function (role) {
      return authService.hasRole(role);
    },

    isAdmin: function () {
      return authService.isAdmin();
    },

    on: function (event, callback) {
      authService.on(event, callback);
    },

    off: function (event, callback) {
      authService.off(event, callback);
    },
  };

  document.addEventListener("DOMContentLoaded", function () {
    if (authService.isAuthenticated()) {
      authService.fetchCurrentUser().then(function (user) {
        if (user) {
          document.body.classList.add("authenticated");
          if (authService.isAdmin()) {
            document.body.classList.add("is-admin");
          }
        }
      });
    }
  });
})(window);
