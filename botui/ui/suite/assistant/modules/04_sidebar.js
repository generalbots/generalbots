"use strict";
/**
 * 04_sidebar.js - Sidebar: wordmark, conversations, user menu, status.
 */

(function () {
    var A = window.GBAssistant;

    function escText(s) {
        var d = document.createElement("span"); d.textContent = s; return d.innerHTML;
    }

    /** Transform a paperclip-marker title into "Anexo: <filename>". */
    function formatTitle(title) {
        if (!title) return A.t("assistant.untitled", "Untitled");
        var clip = title.match(/^\u{1F4CE}\s*(.+)$/u);
        if (clip) {
            var name = clip[1].trim();
            var label = A.locale && A.locale.startsWith("pt") ? "Anexo" : "Attachment";
            return label + ": " + name;
        }
        return title;
    }

    /* ---- Wordmark ---- */
    A.renderWordmark = function () {
        var el = document.getElementById("gbWordmark");
        if (!el) { return; }
        var name = (A.productConfig && A.productConfig.name) || "General Bots";
        if (name.charAt(name.length - 1) === ".") {
            el.innerHTML = escText(name.slice(0, -1)) +
                '<span class="gb-wordmark-dot">.</span>';
        } else {
            el.innerHTML = escText(name) + '<span class="gb-wordmark-dot">.</span>';
        }
    };

    /* ---- Conversation list ---- */
    A.refreshSidebar = function () {
        A.loadSessions().then(function () { renderConversationList(); });
    };

    function renderConversationList() {
        var container = document.getElementById("gbConversations");
        if (!container) { return; }
        container.innerHTML = "";
        if (!A.sessions || A.sessions.length === 0) { return; }

        var groups = groupByDate(A.sessions);
        var order = ["today", "yesterday", "earlier"];
        var labels = {
            today: A.t("assistant.today", "Today"),
            yesterday: A.t("assistant.yesterday", "Yesterday"),
            earlier: A.t("assistant.earlier", "Earlier"),
        };
        for (var g = 0; g < order.length; g++) {
            var key = order[g];
            var items = groups[key];
            if (!items || items.length === 0) { continue; }
            var lbl = document.createElement("div");
            lbl.className = "gb-conv-group-label";
            lbl.textContent = labels[key];
            container.appendChild(lbl);
            for (var i = 0; i < items.length; i++) {
                container.appendChild(createConvItem(items[i]));
            }
        }
    }

    function groupByDate(sessions) {
        var now = new Date();
        var todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
        var yesterdayStart = todayStart - 86400000;
        var groups = { today: [], yesterday: [], earlier: [] };
        for (var i = 0; i < sessions.length; i++) {
            var s = sessions[i];
            var ts = s.updated_at ? new Date(s.updated_at).getTime() : 0;
            if (ts >= todayStart) { groups.today.push(s); }
            else if (ts >= yesterdayStart) { groups.yesterday.push(s); }
            else { groups.earlier.push(s); }
        }
        return groups;
    }

    function createConvItem(session) {
        var btn = document.createElement("button");
        btn.className = "gb-conv-item" +
            (session.id === A.currentSessionId ? " active" : "");
        btn.setAttribute("data-session-id", session.id);

        var textSpan = document.createElement("span");
        textSpan.className = "gb-conv-item-text";
        textSpan.textContent = formatTitle(session.title);
        btn.appendChild(textSpan);

        var actions = document.createElement("span");
        actions.className = "gb-conv-item-actions";

        var renameBtn = document.createElement("button");
        renameBtn.className = "gb-conv-action-btn";
        renameBtn.setAttribute("aria-label", A.t("assistant.rename", "Rename"));
        renameBtn.title = A.t("assistant.rename", "Rename");
        renameBtn.innerHTML = '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 3a2.83 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"/></svg>';
        renameBtn.addEventListener("click", function (e) {
            e.stopPropagation();
            handleRename(session.id, textSpan);
        });
        actions.appendChild(renameBtn);

        var deleteBtn = document.createElement("button");
        deleteBtn.className = "gb-conv-action-btn";
        deleteBtn.setAttribute("aria-label", A.t("assistant.delete", "Delete"));
        deleteBtn.title = A.t("assistant.delete", "Delete");
        deleteBtn.innerHTML = '<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>';
        deleteBtn.addEventListener("click", function (e) {
            e.stopPropagation();
            handleDelete(session.id);
        });
        actions.appendChild(deleteBtn);
        btn.appendChild(actions);

        btn.addEventListener("click", function () { loadConversation(session.id); });
        return btn;
    }

    function loadConversation(sessId) {
        A.currentSessionId = sessId;
        renderConversationList();
        /* Update titlebar with the conversation title */
        var sess = A.sessions.filter(function (s) { return s.id === sessId; })[0];
        if (typeof window.gbSetTitlebarTitle === "function") {
            window.gbSetTitlebarTitle(sess ? formatTitle(sess.title) : "");
        }
        A.loadSessionMessages(sessId).then(function (msgs) {
            A.renderHistory(msgs);
            A.reconnectWs();
        });
    }

    function handleRename(sessId, textSpan) {
        var current = textSpan.textContent;
        var newName = prompt(A.t("assistant.rename-prompt", "Rename conversation:"), current);
        if (newName && newName !== current) {
            A.renameSession(sessId, newName).then(function () {
                textSpan.textContent = newName;
            });
        }
    }

    function handleDelete(sessId) {
        if (!confirm(A.t("assistant.delete-confirm", "Delete this conversation?"))) {
            return;
        }
        A.deleteSession(sessId).then(function () {
            if (A.currentSessionId === sessId) {
                A.currentSessionId = null;
                A.showEmptyState();
                if (typeof A.renderGreeting === "function") { A.renderGreeting(); }
                A.newSession().then(function () { A.reconnectWs(); });
            }
            A.refreshSidebar();
        });
    }

    /* ---- Search ---- */
    A.initSearch = function () {
        var input = document.getElementById("gbSearchInput");
        if (!input) { return; }
        input.addEventListener("input", function () {
            var q = input.value.toLowerCase().trim();
            var items = document.querySelectorAll(".gb-conv-item");
            for (var i = 0; i < items.length; i++) {
                var text = items[i].querySelector(".gb-conv-item-text");
                var match = !q || (text && text.textContent.toLowerCase().indexOf(q) !== -1);
                items[i].style.display = match ? "" : "none";
            }
            var labels = document.querySelectorAll(".gb-conv-group-label");
            for (var j = 0; j < labels.length; j++) {
                var next = labels[j].nextElementSibling;
                var hasVisible = false;
                while (next && !next.classList.contains("gb-conv-group-label")) {
                    if (next.style.display !== "none") { hasVisible = true; }
                    next = next.nextElementSibling;
                }
                labels[j].style.display = hasVisible ? "" : "none";
            }
        });
    };

    /* ---- User section with menu ---- */
    A.renderUser = function () {
        var nameEl = document.getElementById("gbUserName");
        var avatarEl = document.getElementById("gbUserAvatar");
        var name = A.userName || A.userEmail || "";
        if (nameEl) { nameEl.textContent = name || A.t("assistant.user", "User"); }
        if (avatarEl) {
            avatarEl.textContent = (name || "U").charAt(0).toUpperCase();
        }
    };

    A.initUserMenu = function () {
        var userSection = document.getElementById("gbUserSection");
        if (!userSection) { return; }
        userSection.style.cursor = "pointer";
        userSection.addEventListener("click", function (e) {
            e.stopPropagation();
            toggleUserMenu(userSection);
        });
    };

    var userMenuEl = null;
    function toggleUserMenu(anchor) {
        if (userMenuEl) { userMenuEl.remove(); userMenuEl = null; return; }
        userMenuEl = document.createElement("div");
        userMenuEl.className = "gb-user-menu";
        var settingsLabel = A.t("assistant.settings", "Settings");
        var signOutLabel = A.t("assistant.sign-out", "Sign out");
        userMenuEl.innerHTML =
            '<button class="gb-user-menu-item" data-action="settings">' +
                '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.6 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.6a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>' +
                '<span>' + escText(settingsLabel) + '</span></button>' +
            '<button class="gb-user-menu-item gb-user-menu-danger" data-action="signout">' +
                '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><polyline points="16 17 21 12 16 7"/><line x1="21" y1="12" x2="9" y2="12"/></svg>' +
                '<span>' + escText(signOutLabel) + '</span></button>';
        anchor.parentElement.insertBefore(userMenuEl, anchor.nextSibling);
        userMenuEl.addEventListener("click", function (ev) {
            var item = ev.target.closest("[data-action]");
            if (!item) { return; }
            var action = item.getAttribute("data-action");
            if (action === "settings") { A.switchView("settings"); }
            if (action === "signout") { A.signOut(); }
            userMenuEl.remove(); userMenuEl = null;
        });
        document.addEventListener("click", function close() {
            if (userMenuEl) { userMenuEl.remove(); userMenuEl = null; }
            document.removeEventListener("click", close);
        }, { once: true });
    }

    /* ---- Status line ---- */
    A.renderStatus = function () {
        var textEl = document.getElementById("gbStatusText");
        if (!textEl) { return; }

        /* Find the display name of the current model */
        var modelName = A.defaultModel || A.selectedModel || "";
        /* Shorten long model IDs: after the last slash or colon */
        var short = modelName;
        var slashIdx = short.lastIndexOf("/");
        if (slashIdx >= 0) { short = short.substring(slashIdx + 1); }
        var colonIdx = short.indexOf(":");
        if (colonIdx > 0) { short = short.substring(0, colonIdx); }
        if (!short) { short = modelName; }

        var dest = A.destination;
        if (!dest || !dest.scope || dest.scope === "unknown") {
            textEl.textContent = short;
            textEl.className = "gb-status-text";
            return;
        }

        var scope = dest.scope;
        var host = dest.host || "";
        var label = "";
        var isExternal = false;

        if (scope === "machine") {
            label = short + " · " + A.t("assistant.on-this-computer", "on this computer");
        } else if (scope === "network") {
            label = short + " · " + A.t("assistant.on-your-network", "on your network") +
                (host ? " (" + host + ")" : "");
        } else if (scope === "external") {
            isExternal = true;
            label = short + " · " + A.t("assistant.external", "external") +
                (host ? ": " + host : "");
        } else {
            label = short;
        }

        if (isExternal) {
            textEl.innerHTML =
                '<svg class="gb-status-external-icon" width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>' +
                '<span class="gb-status-external">' + escText(label) + '</span>';
        } else {
            textEl.textContent = label;
            textEl.className = "gb-status-text";
        }
    };

    /* ---- Sidebar toggle ---- */
    A.initSidebarToggle = function () {
        var btn = document.getElementById("gbSidebarToggle");
        if (!btn) { return; }
        btn.addEventListener("click", function () {
            var sidebar = document.getElementById("gb-sidebar");
            if (!sidebar) { return; }
            sidebar.classList.toggle("collapsed");
            A.sidebarCollapsed = sidebar.classList.contains("collapsed");
        });
        var expand = document.createElement("button");
        expand.className = "gb-btn-icon gb-expand-sidebar";
        expand.setAttribute("aria-label", A.t("assistant.expand-sidebar", "Expand sidebar"));
        expand.innerHTML = '<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="9" y1="3" x2="9" y2="21"/></svg>';
        expand.addEventListener("click", function () {
            var sidebar = document.getElementById("gb-sidebar");
            if (sidebar) { sidebar.classList.remove("collapsed"); }
            A.sidebarCollapsed = false;
        });
        var main = document.getElementById("gb-main");
        if (main) { main.insertBefore(expand, main.firstChild); }
    };
})();
