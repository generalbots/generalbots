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

    /* ---- Conversation list ---- */
    A.refreshSidebar = function () {
        A.loadSessions().then(function () { renderConversationList(); });
    };

    function renderConversationList() {
        var container = document.getElementById("gbConversations");
        if (!container) { return; }
        container.innerHTML = "";
        var count = document.getElementById("gbConvCount");
        if (count) { count.textContent = String((A.sessions || []).length); }
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
        var displayTitle = formatTitle(session.title);
        textSpan.textContent = displayTitle;
        textSpan.title = displayTitle;
        btn.appendChild(textSpan);

        var actions = document.createElement("span");
        actions.className = "gb-conv-item-actions";

        var renameBtn = document.createElement("button");
        renameBtn.className = "gb-conv-action-btn";
        renameBtn.setAttribute("aria-label", A.t("assistant.rename", "Rename"));
        renameBtn.title = A.t("assistant.rename", "Rename");
        renameBtn.innerHTML = '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linejoin="round"><path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/></svg>';
        renameBtn.addEventListener("click", function (e) {
            e.stopPropagation();
            handleRename(session.id, textSpan);
        });
        actions.appendChild(renameBtn);

        var deleteBtn = document.createElement("button");
        deleteBtn.className = "gb-conv-action-btn";
        deleteBtn.setAttribute("aria-label", A.t("assistant.delete", "Delete"));
        deleteBtn.title = A.t("assistant.delete", "Delete");
        deleteBtn.innerHTML = '<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round"><path d="M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></svg>';
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
        /* Settings or Drive may be open: a conversation picked in the
           sidebar is shown, so the chat view comes back first. */
        if (A.currentView !== "chat" && typeof A.switchView === "function") { A.switchView("chat"); }
        A.currentSessionId = sessId;
        renderConversationList();
        /* Sync the agent toggle for this conversation's stored state */
        if (typeof A.syncAgentToggle === "function") { A.syncAgentToggle(); }
        /* Update titlebar with the conversation title */
        var sess = A.sessions.filter(function (s) { return s.id === sessId; })[0];
        if (typeof window.gbSetTitlebarTitle === "function") {
            window.gbSetTitlebarTitle(sess ? formatTitle(sess.title) : "");
        }
        A.loadSessionMessages(sessId).then(function (msgs) {
            A.renderHistory(msgs);
            A.reconnectWs();
        }).catch(function (err) {
            console.warn("[assistant] loadConversation failed:", err);
            showLoadError(sessId);
        });
        /* Narrow viewports: auto-close the slide-over sidebar */
        if (isNarrow() && !A.sidebarCollapsed) { collapseSidebar(); }
    }

    function showLoadError(sessId) {
        A.showConversation();
        var msgs = document.getElementById("gbMessages"); if (!msgs) return;
        msgs.innerHTML = '<div class="gb-load-error" role="alert">' +
            '<span>' + escText(A.t("assistant.load-error", "Could not load this conversation.")) + '</span>' +
            '<button class="gb-btn-retry">' + escText(A.t("assistant.retry", "Tentar novamente")) + '</button></div>';
        var btn = msgs.querySelector(".gb-btn-retry");
        if (btn) btn.addEventListener("click", function () { loadConversation(sessId); });
    }

    /* ---- Inline rename (edits title in place) ---- */
    function handleRename(sessId, textSpan) {
        if (textSpan.querySelector("input")) { return; } /* already editing */
        var current = textSpan.textContent;
        var input = document.createElement("input");
        input.type = "text";
        input.className = "gb-conv-rename-input";
        input.value = current;
        textSpan.textContent = "";
        textSpan.appendChild(input);
        input.focus();
        input.select();

        function commit() {
            var val = input.value.trim();
            cleanup();
            if (val && val !== current) {
                textSpan.textContent = val;
                A.renameSession(sessId, val);
            } else {
                textSpan.textContent = current;
            }
        }
        function cleanup() {
            input.removeEventListener("keydown", onKey);
            input.removeEventListener("blur", onBlur);
        }
        function onKey(e) {
            if (e.key === "Enter") { e.preventDefault(); commit(); }
            if (e.key === "Escape") { e.preventDefault(); cleanup(); textSpan.textContent = current; }
        }
        function onBlur() { commit(); }
        input.addEventListener("keydown", onKey);
        input.addEventListener("blur", onBlur);
    }

    /* ---- Inline delete confirmation ---- */
    function handleDelete(sessId) {
        var row = document.querySelector('[data-session-id="' + sessId + '"]');
        if (!row) { return; }
        /* If already showing confirmation, ignore */
        if (row.querySelector(".gb-conv-confirm")) { return; }
        var originalHtml = row.innerHTML;

        var bar = document.createElement("div");
        bar.className = "gb-conv-confirm";
        bar.innerHTML =
            '<span class="gb-conv-confirm-text">' + escText(A.t("assistant.delete-confirm", "Delete this conversation?")) + '</span>' +
            '<button class="gb-conv-confirm-btn gb-conv-confirm-danger" data-act="yes">' + escText(A.t("assistant.delete", "Delete")) + '</button>' +
            '<button class="gb-conv-confirm-btn" data-act="no">' + escText(A.t("assistant.cancel", "Cancel")) + '</button>';
        row.innerHTML = "";
        row.appendChild(bar);

        var cancelBtn = bar.querySelector('[data-act="no"]');
        if (cancelBtn) { cancelBtn.focus(); }

        function restore() {
            row.innerHTML = originalHtml;
            /* Re-bind the click handler */
            row.addEventListener("click", function () { loadConversation(sessId); });
            document.removeEventListener("keydown", escHandler);
        }
        function escHandler(e) {
            if (e.key === "Escape") { e.preventDefault(); restore(); }
        }
        document.addEventListener("keydown", escHandler);

        bar.addEventListener("click", function (e) {
            e.stopPropagation();
            var act = e.target.closest("[data-act]");
            if (!act) { return; }
            if (act.getAttribute("data-act") === "no") { restore(); return; }
            /* Perform delete */
            act.disabled = true;
            act.textContent = "...";
            A.apiFetch(
                "/api/chat/history/sessions/" + encodeURIComponent(sessId),
                { method: "DELETE" }
            ).then(function (r) {
                if (!r.ok) {
                    return r.text().then(function (body) {
                        var reason = "";
                        try { var j = JSON.parse(body); reason = j.error || j.message || body; } catch (e) { reason = body; }
                        bar.innerHTML = '<span class="gb-conv-confirm-text gb-conv-confirm-error">' +
                            escText(reason || "Error " + r.status) + '</span>' +
                            '<button class="gb-conv-confirm-btn" data-act="no">OK</button>';
                        bar.querySelector('[data-act="no"]').addEventListener("click", function (ev) {
                            ev.stopPropagation(); restore();
                        });
                    });
                }
                document.removeEventListener("keydown", escHandler);
                if (A.currentSessionId === sessId) {
                    A.currentSessionId = null;
                    A.showEmptyState();
                    if (typeof A.renderGreeting === "function") { A.renderGreeting(); }
                    A.newSession().then(function () {
                        A.reconnectWs();
                        if (typeof A.focusComposer === "function") { A.focusComposer(); }
                    });
                }
                A.refreshSidebar();
            }).catch(function (err) {
                bar.innerHTML = '<span class="gb-conv-confirm-text gb-conv-confirm-error">' +
                    escText(String(err)) + '</span>' +
                    '<button class="gb-conv-confirm-btn" data-act="no">OK</button>';
                bar.querySelector('[data-act="no"]').addEventListener("click", function (ev) {
                    ev.stopPropagation(); restore();
                });
            });
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
        var roleEl = document.getElementById("gbUserRole"), roles = A.userRoles || [];
        if (roleEl) {
            var pt = A.locale && A.locale.startsWith("pt");
            var admin = roles.some(function (r) { return /admin/i.test(r); });
            roleEl.textContent = !roles.length ? "" : admin ? (pt ? "Administrador" : "Administrator") : (pt ? "Usuário" : "User");
        }
        if (avatarEl) {
            var own = /^#[0-9a-f]{6}$/i.test(A.avatarColor || "") && A.avatarColor.toLowerCase() !== "#171717";
            avatarEl.style.backgroundColor = own ? A.avatarColor : "";
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
        if (userMenuEl) { userMenuEl.remove(); userMenuEl = null; anchor.setAttribute("aria-expanded", "false"); return; }
        userMenuEl = document.createElement("div");
        userMenuEl.className = "gb-user-menu";
        anchor.setAttribute("aria-expanded", "true");
        var signOutLabel = A.t("assistant.sign-out", "Sign out");
        userMenuEl.innerHTML = '<button class="gb-user-menu-item gb-user-menu-danger" data-action="signout">' +
            GBIcon("sign-out", 16) + escText(signOutLabel) + '</button>';
        anchor.parentElement.insertBefore(userMenuEl, anchor.nextSibling);
        userMenuEl.addEventListener("click", function (ev) {
            var item = ev.target.closest("[data-action]");
            if (!item) { return; }
            var action = item.getAttribute("data-action");
            if (action === "settings") { A.switchView("settings"); }
            if (action === "signout") { A.signOut(); }
            userMenuEl.remove(); userMenuEl = null; anchor.setAttribute("aria-expanded", "false");
        });
        document.addEventListener("click", function close() {
            if (userMenuEl) { userMenuEl.remove(); userMenuEl = null; anchor.setAttribute("aria-expanded", "false"); }
            document.removeEventListener("click", close);
        }, { once: true });
    }

    /* ---- Sidebar toggle ---- */
    function isNarrow() { return window.innerWidth <= 960; }

    function collapseSidebar() { setCollapsed(true); }
    function setCollapsed(value) {
        A.sidebarCollapsed = value;
        document.getElementById("gb-sidebar").classList.toggle("collapsed", value);
        document.getElementById("gbSidebarToggle").setAttribute("aria-expanded", String(!value));
    }
    A.toggleSidebar = function () { setCollapsed(!A.sidebarCollapsed); };
    A.initSidebarToggle = function () {
        var narrow = window.matchMedia("(max-width: 960px)");
        setCollapsed(narrow.matches);
        document.getElementById("gbSidebarToggle").addEventListener("click", A.toggleSidebar);
        narrow.addEventListener("change", function (e) { setCollapsed(e.matches); });
    };
})();
