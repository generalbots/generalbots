"use strict";

/**
 * Workspace tabs (#1168-fe) — event wiring.
 * Pointer-drag reorder, inline dblclick rename, pin context menu, the "+"
 * right-click picker (recent conversations only) and the zero-edit
 * history integration: a document-level contextmenu listener on
 * .chat-sidebar-conv-item items dispatches "gb-open-history-tab".
 *
 * The tab strip is for CONVERSATIONS only — apps never appear here; they
 * open as desktop windows via the launcher/openDeepLink.
 */

(function () {
  function stripEl() { return document.getElementById("gbTabStrip"); }

  function tabIdFromEvent(e) {
    var el = e.target.closest(".gb-tab");
    return el ? el.getAttribute("data-tab-id") : null;
  }

  // Every close path (×, middle-click, Ctrl+W, context menu, overflow menu)
  // called this function, but it was never defined: the click threw a
  // ReferenceError and the tab stayed open. Refuse only the cases the strip
  // must protect: a pinned tab and the last remaining tab.
  function closeTabGuarded(id) {
    if (!id || !window.GBTabs || !GBTabs.state) return;
    var tabs = GBTabs.state.tabs || [];
    var tab = null;
    for (var i = 0; i < tabs.length; i++) {
      if (tabs[i].id === id) { tab = tabs[i]; break; }
    }
    if (!tab) return;
    if (tab.pinned) {
      if (typeof notify === "function") notify("Pinned tabs cannot be closed", "info");
      return;
    }
    if (tabs.length <= 1) {
      if (typeof notify === "function") notify("Keep at least one conversation open", "info");
      return;
    }
    GBTabs.closeTab(id);
  }

  // ── Pointer-drag reorder ──

  var drag = null;

  function onPointerDown(e) {
    if (e.button !== 0) return;
    if (e.target.closest(".gb-tab-close") || e.target.closest(".gb-tab-new")) return;
    var id = tabIdFromEvent(e);
    if (!id) return;
    drag = { id: id, moved: false, startX: e.clientX };
    window.addEventListener("pointermove", onPointerMove);
    window.addEventListener("pointerup", onPointerUp);
  }

  function onPointerMove(e) {
    if (!drag) return;
    if (!drag.moved && Math.abs(e.clientX - drag.startX) < 6) return;
    drag.moved = true;
    var over = document.elementFromPoint(e.clientX, e.clientY);
    var target = over && over.closest ? over.closest(".gb-tab") : null;
    if (!target || target.getAttribute("data-tab-id") === drag.id) return;
    var fromIdx = parseInt(
      stripEl().querySelector('[data-tab-id="' + drag.id + '"]').getAttribute("data-tab-idx"), 10);
    var toIdx = parseInt(target.getAttribute("data-tab-idx"), 10);
    GBTabs.moveTab(fromIdx, toIdx);
    GBTabs.renderStrip();
  }

  function onPointerUp() {
    window.removeEventListener("pointermove", onPointerMove);
    window.removeEventListener("pointerup", onPointerUp);
    if (drag && drag.moved) suppressNextClick();
    drag = null;
  }

  function suppressNextClick() {
    var armed = true;
    function swallow(e) {
      if (armed && e.target.closest && e.target.closest(".gb-tab")) {
        e.stopPropagation();
        e.preventDefault();
      }
      armed = false;
      stripEl().removeEventListener("click", swallow, true);
    }
    var s = stripEl();
    if (s) s.addEventListener("click", swallow, true);
  }

  // ── Click / close / dblclick rename ──

  function onClick(e) {
    if (drag && drag.moved) return;
    if (e.target.closest("#gbTabNew")) {
      // Handled by 22_multichat's capture listener: every plain click on
      // "+" opens a NEW CHAT tab. History picker lives on right-click.
      return;
    }
    var id = tabIdFromEvent(e);
    if (!id) return;
    if (e.target.closest(".gb-tab-close")) {
      e.stopPropagation();
      closeTabGuarded(id);
      return;
    }
    // Multi-chat: focus switches the visible conversation (its own pane +
    // session); without multi-chat the stock history-tab path applies.
    if (window.GBMultiChat && GBMultiChat.tabs[id]) {
      GBMultiChat.switchTo(id);
      return;
    }
    GBTabs.focusTab(id);
  }

  function onDblClick(e) {
    var id = tabIdFromEvent(e);
    if (!id || e.target.closest(".gb-tab-close")) return;
    startInlineRename(id, e.target.closest(".gb-tab"));
  }

  function startInlineRename(id, tabEl) {
    var titleSpan = tabEl.querySelector(".gb-tab-title");
    if (!titleSpan || titleSpan.querySelector("input")) return;
    var tab = null;
    GBTabs.state.tabs.forEach(function (t) { if (t.id === id) tab = t; });
    if (!tab) return;
    var input = document.createElement("input");
    input.type = "text";
    input.className = "gb-tab-rename";
    input.value = tab.title;
    input.maxLength = 60;
    titleSpan.textContent = "";
    titleSpan.appendChild(input);
    input.focus();
    input.select();
    input.addEventListener("keydown", function (ev) {
      if (ev.key === "Enter") { ev.preventDefault(); commit(); }
      if (ev.key === "Escape") { ev.preventDefault(); cancel(); }
    });
    input.addEventListener("blur", commit);
    input.addEventListener("click", function (ev) { ev.stopPropagation(); });
    function done() {
      setTimeout(function () { GBTabs.renderStrip(); }, 0);
    }
    function commit() {
      input.removeEventListener("blur", commit);
      GBTabs.renameTab(id, input.value);
      done();
    }
    function cancel() {
      input.removeEventListener("blur", commit);
      done();
    }
  }

  // ── Tab context menu (pin / rename / close) ──

  function closeMenus() {
    document.querySelectorAll(".gb-context-menu").forEach(function (m) { m.remove(); });
  }

  function openTabMenu(x, y, id) {
    closeMenus();
    var menu = document.createElement("div");
    menu.className = "gb-context-menu";
    var isPinned = false;
    GBTabs.state.tabs.forEach(function (t) { if (t.id === id) isPinned = t.pinned; });
    menu.innerHTML =
      '<button type="button" data-act="pin">' + (isPinned ? "Unpin" : "Pin") + "</button>" +
      '<button type="button" data-act="rename">Rename</button>' +
      '<button type="button" data-act="close">Close</button>';
    document.body.appendChild(menu);
    positionMenu(menu, x, y);
    menu.addEventListener("click", function (e) {
      var act = e.target.getAttribute("data-act");
      closeMenus();
      if (!act) return;
      if (act === "pin") GBTabs.togglePin(id);
      if (act === "rename") {
        var el = stripEl().querySelector('[data-tab-id="' + id + '"]');
        if (el) startInlineRename(id, el);
      }
      if (act === "close") closeTabGuarded(id);
    });
  }

  function positionMenu(menu, x, y) {
    var r = menu.getBoundingClientRect();
    menu.style.left = Math.min(x, window.innerWidth - r.width - 8) + "px";
    menu.style.top = Math.min(y, window.innerHeight - r.height - 8) + "px";
  }

  // ── "+" right-click history picker (recent sessions only) ──

  function resolveBotName() {
    return (window.GBResolveActiveBot && GBResolveActiveBot()) ||
      (window.ChatState && window.ChatState.currentBotName) || "";
  }

  function fetchRecentSessions() {
    return fetch(
      "/api/chat/history/sessions?limit=8&bot_name=" +
      encodeURIComponent(resolveBotName()),
      { headers: GBTabs.authHeaders() }
    )
      .then(function (r) { return r.ok ? r.json() : null; })
      .then(function (d) { return (d && d.sessions) || []; })
      .catch(function () { return []; });
  }

  function openPicker(clickEvent) {
    closeMenus();
    var picker = document.createElement("div");
    picker.className = "gb-context-menu gb-tab-picker";
    picker.innerHTML = '<div class="gb-tab-picker-title">Open conversation in new tab</div>' +
      '<div class="gb-tab-picker-body"><div class="gb-tab-picker-empty">Loading…</div></div>';
    document.body.appendChild(picker);
    var anchor = clickEvent && clickEvent.target.getBoundingClientRect
      ? clickEvent.target.getBoundingClientRect()
      : { left: 24, bottom: 60 };
    positionMenu(picker, anchor.left, anchor.bottom + 4);

    var body = picker.querySelector(".gb-tab-picker-body");

    function renderRows(rows) {
      body.innerHTML = rows.join("") ||
        '<div class="gb-tab-picker-empty">Nothing to open</div>';
    }

    Promise.all([fetchRecentSessions()]).then(function (results) {
      var sessions = results[0] || [];
      var rows = [];
      if (sessions.length) {
        rows.push('<div class="gb-tab-picker-section">Recent conversations</div>');
        sessions.forEach(function (s, i) {
          rows.push('<button type="button" class="gb-tab-picker-row" data-session-idx="' + i + '">' +
            '<span class="gb-tab-picker-glyph">\u{1F4AC}</span>' +
            "<span>" + escapeHtml(s.title || "Conversation") + "</span></button>");
        });
      }
      renderRows(rows);

      picker.addEventListener("click", function (e) {
        var row = e.target.closest(".gb-tab-picker-row");
        closeMenus();
        if (!row) return;
        var sessIdx = row.getAttribute("data-session-idx");
        if (sessIdx !== null) {
          var s = sessions[parseInt(sessIdx, 10)];
          if (s && s.session_id) {
            GBTabs.createTab({
              kind: "history",
              sessionId: s.session_id,
              botId: (window.ChatState && window.ChatState.currentBotId) || undefined,
              title: s.title || "Conversation",
              faviconGlyph: "\u{1F4AC}",
            });
          }
        }
      });
    });
  }

  // ── History integration (zero edit to sidebar-convos.js) ──

  function bindHistoryContextMenu() {
    document.addEventListener("contextmenu", function (e) {
      var item = e.target.closest(".chat-sidebar-conv-item[data-session-id]");
      if (!item) return;
      e.preventDefault();
      closeMenus();
      var menu = document.createElement("div");
      menu.className = "gb-context-menu";
      menu.innerHTML = '<button type="button" data-act="open">Open in new tab</button>' +
        '<button type="button" data-act="delete" class="gb-menu-danger">Delete conversation</button>';
      document.body.appendChild(menu);
      positionMenu(menu, e.clientX, e.clientY);
      menu.addEventListener("click", function (ev) {
        var act = ev.target.getAttribute("data-act");
        if (!act) return;
        closeMenus();
        if (act === "open") {
          window.dispatchEvent(new CustomEvent("gb-open-history-tab", {
            detail: {
              sessionId: item.getAttribute("data-session-id"),
              title: (item.querySelector(".chat-sidebar-conv-name") || {}).textContent || "Conversation",
            },
          }));
          return;
        }
        if (act === "delete") {
          var sid = item.getAttribute("data-session-id");
          if (!sid) return;
          if (window.GBSidebarConvos && window.GBSidebarConvos.deleteConversation) {
            window.GBSidebarConvos.deleteConversation(sid, item);
          }
        }
      });
    });

    window.addEventListener("gb-open-history-tab", function (e) {
      var d = e.detail || {};
      if (!d.sessionId) return;
      // Opening a conversation that already has a tab must activate that tab.
      // Creating a second tab for the same session made pruneTabs drop it as a
      // duplicate, leaving activeTabId pointing at a tile that no longer
      // existed and replaying the history into the wrong pane (#1693).
      var existing = (GBTabs.state.tabs || []).find(function (tab) {
        return String(tab.sessionId || "") === String(d.sessionId);
      });
      if (existing) {
        GBTabs.focusTab(existing.id);
        return;
      }
      GBTabs.createTab({
        kind: "history",
        sessionId: d.sessionId,
        botId: (window.ChatState && window.ChatState.currentBotId) || undefined,
        title: d.title || "Conversation",
        faviconGlyph: "\u{1F4AC}",
      });
    });
  }

  // Helpers shared with 21_tabs_events_boot.js (same split for #1731).
  // A namespaced object avoids new globals; load order keeps it populated
  // before the boot file binds the document listeners.
  var E = (GBTabs.events = GBTabs.events || {});
  E.tabIdFromEvent = tabIdFromEvent;
  E.closeTabGuarded = closeTabGuarded;
  E.closeMenus = closeMenus;
  E.positionMenu = positionMenu;
  E.onPointerDown = onPointerDown;
  E.onClick = onClick;
  E.onDblClick = onDblClick;
  E.openPicker = openPicker;
  E.openTabMenu = openTabMenu;
  E.bindHistoryContextMenu = bindHistoryContextMenu;
})();
