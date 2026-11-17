/* Settings module 06b: group management.
   #2307 — split from 06_users_groups.js to stay under the 450-line limit.
   Shared helpers come from S._ugCommon (published by 06_users_groups.js). */
(function (window) {
  "use strict";
  var S = window.__GBSettingsMain;
  var UG = S._ugCommon;
  var t = UG.t;
  var byId = UG.byId;
  var setStatus = UG.setStatus;
  var fieldValue = UG.fieldValue;
  var openDialog = UG.openDialog;
  var closeDialog = UG.closeDialog;
  var parseResponse = UG.parseResponse;
  var askConfirm = UG.askConfirm;
  var bindConfirmDialog = UG.bindConfirmDialog;
  var loadingMarkup = UG.loadingMarkup;
  var errorMarkup = UG.errorMarkup;
  var resetForm = UG.resetForm;
  var withSubmitState = UG.withSubmitState;
  var showToast = UG.showToast;
  var escapeHtml = UG.escapeHtml;
  var apiUrl = UG.apiUrl;
  var getAuthToken = UG.getAuthToken;
  var API_ENDPOINTS = UG.API_ENDPOINTS;
  var debounce = S.debounce;
  var groupsData = [];

  function initGroupManagement() {
    bindConfirmDialog();
    var addGroupBtn = byId("add-group-btn");
    if (addGroupBtn) addGroupBtn.addEventListener("click", openAddGroupDialog);
    var groupSearchInput = byId("group-search");
    if (groupSearchInput) {
      groupSearchInput.addEventListener(
        "input",
        debounce(function () { loadGroups(this.value); }, 300),
      );
    }
    var addGroupForm = byId("add-group-form");
    if (addGroupForm) addGroupForm.addEventListener("submit", handleAddGroup);
    loadGroups();
  }

  function loadGroups(search) {
    var container = byId("groups-list");
    if (!container) return;
    container.setAttribute("aria-busy", "true");
    container.innerHTML = loadingMarkup();
    var params = new URLSearchParams();
    params.append("per_page", "50");
    if (search) params.append("search", search);
    fetch(apiUrl(API_ENDPOINTS.GROUPS_LIST) + "?" + params.toString(), {
      headers: { Authorization: "Bearer " + getAuthToken() },
    })
      .then(function (r) { return parseResponse(r); })
      .then(function (data) {
        groupsData = data && Array.isArray(data.groups) ? data.groups : [];
        renderGroups(container);
        container.setAttribute("aria-busy", "false");
      })
      .catch(function () {
        container.setAttribute("aria-busy", "false");
        container.innerHTML = errorMarkup(
          t("admin-failed-load-groups", "Failed to load groups."),
        );
      });
  }

  function renderGroups(container) {
    if (!container) return;
    if (groupsData.length === 0) {
      container.innerHTML = '<div class="empty-state"><p>' +
        escapeHtml(t("admin-no-groups", "No groups found")) + "</p></div>";
      return;
    }
    var members = escapeHtml(t("admin-members-suffix", "members"));
    var view = escapeHtml(t("admin-view-members", "View Members"));
    var edit = escapeHtml(t("admin-action-edit", "Edit"));
    var del = escapeHtml(t("admin-action-delete", "Delete"));
    var html = '<div class="groups-grid">';
    groupsData.forEach(function (group) {
      if (!group) return;
      var id = escapeHtml(group.id || "");
      html += '<div class="group-card" data-group-id="' + id + '">';
      html += '<div class="group-header"><h3 class="group-name">' +
        escapeHtml(group.name || "") + '</h3><span class="member-count">' +
        escapeHtml(String(group.member_count || 0)) + " " + members + "</span></div>";
      if (group.description) {
        html += '<p class="group-description">' +
          escapeHtml(group.description) + "</p>";
      }
      html += '<div class="group-actions">';
      html += '<button class="btn-secondary btn-sm" title="' + view +
        '" onclick="SettingsModule.viewGroupMembers(\'' + id + '\')">' + view + "</button>";
      html += '<button class="btn-icon" title="' + edit +
        '" onclick="SettingsModule.editGroup(\'' + id +
        '\')"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path></svg></button>';
      html += '<button class="btn-icon btn-danger" title="' + del +
        '" onclick="SettingsModule.deleteGroup(\'' + id +
        '\')"><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg></button>';
      html += "</div></div>";
    });
    html += "</div>";
    container.innerHTML = html;
  }

  function openAddGroupDialog() {
    resetForm("add-group-form");
    setStatus("add-group-status", "");
    openDialog("create-group-modal");
  }

  function closeAddGroupDialog() {
    closeDialog("create-group-modal");
    resetForm("add-group-form");
    setStatus("add-group-status", "");
  }

  function handleAddGroup(event) {
    if (event && typeof event.preventDefault === "function") event.preventDefault();
    var form = event ? event.target : null;
    if (!form) return;
    var submit = form.querySelector('button[type="submit"]');
    var original = submit ? submit.textContent : "";
    withSubmitState(form, true, original);
    setStatus("add-group-status", "");
    var payload = {
      name: fieldValue(form, "name"),
      description: fieldValue(form, "description") || "",
    };
    fetch(apiUrl(API_ENDPOINTS.GROUPS_CREATE), {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: "Bearer " + getAuthToken(),
      },
      body: JSON.stringify(payload),
    })
      .then(function (r) { return parseResponse(r); })
      .then(function () {
        var ok = t("admin-group-created", "Group created successfully");
        setStatus("add-group-status", ok);
        showToast(ok, "success");
        closeAddGroupDialog();
        loadGroups();
      })
      .catch(function (error) {
        var message = error && error.message
          ? error.message
          : t("admin-failed-create-group", "Failed to create group.");
        setStatus("add-group-status", message, true);
      })
      .finally(function () { withSubmitState(form, false, original); });
  }

  function editGroup(groupId) {
    var group = groupsData.find(function (g) { return g && g.id === groupId; });
    if (!group) return;
    openDialog("edit-group-dialog");
  }

  function deleteGroup(groupId) {
    askConfirm(t("admin-confirm-delete-group", "Delete this group?"), function () {
      fetch(apiUrl(API_ENDPOINTS.GROUPS_DELETE.replace(":group_id", groupId)), {
        method: "DELETE",
        headers: { Authorization: "Bearer " + getAuthToken() },
      })
        .then(function (r) { return parseResponse(r); })
        .then(function () {
          showToast(t("admin-group-deleted", "Group deleted"), "success");
          loadGroups();
        })
        .catch(function (error) { showToast(error.message, "error"); });
    });
  }

  function viewGroupMembers(groupId) {
    if (!groupId) return;
    openDialog("group-members-dialog");
  }

  function removeGroupMember(groupId, userId) {
    askConfirm(t("admin-confirm-remove-member", "Remove member?"), function () {
      fetch(
        apiUrl(API_ENDPOINTS.GROUPS_REMOVE_MEMBER.replace(":group_id", groupId)),
        {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: "Bearer " + getAuthToken(),
          },
          body: JSON.stringify({ user_id: userId }),
        },
      )
        .then(function (r) { return parseResponse(r); })
        .then(function () {
          showToast(t("admin-member-removed", "Member removed"), "success");
          loadGroups();
        })
        .catch(function (error) { showToast(error.message, "error"); });
    });
  }

  S.initGroupManagement = initGroupManagement;
  S.loadGroups = loadGroups;
  S.editGroup = editGroup;
  S.deleteGroup = deleteGroup;
  S.viewGroupMembers = viewGroupMembers;
  S.removeGroupMember = removeGroupMember;
  S.openAddGroupDialog = openAddGroupDialog;
  S.closeAddGroupDialog = closeAddGroupDialog;
  S.handleAddGroup = handleAddGroup;
})(window);
