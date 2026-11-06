(function (window, document) {
  "use strict";
  var N = document.__gb_mod_auth_service;
  if (!N) return;
  var AuthService = N.AuthService;
  var UserService = N.UserService;
  var GROUP_ENDPOINTS = N.GROUP_ENDPOINTS;

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

  (function(){ var __cb = function () {
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
  }; if (document.readyState === "loading") { document.addEventListener("DOMContentLoaded", __cb); } else { __cb(); } })();

  delete document.__gb_mod_auth_service;
})(window, document);
