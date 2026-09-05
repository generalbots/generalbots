"use strict";

/**
 * Workspace tabs (#1168-fe) — event wiring.
 * Pointer-drag reorder, inline dblclick rename, pin context menu, the "+"
 * mini picker (recent sessions + static app entries) and the zero-edit
 * history integration: a document-level contextmenu listener on
 * .chat-sidebar-conv-item items dispatches "gb-open-history-tab".
 */

(function () {
  var APP_ENTRIES = [
    { id: "vibe", title: "Vibe", glyph: "\u26A1", path: "/vibe" },
    { id: "research", title: "Research", glyph: "\u{1F52C}", path: "/research" },
    { id: "drive", title: "Drive", glyph: "\u{1F4C1}", path: "/drive" },
  ];

  function stripEl() { return document.getElementById("gbTabStrip"); }

  function tabIdFromEvent(e) {
    var el = e.target.closest(".gb-tab");
    return el ? el.getAttribute("data-tab-id") : null;
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
      // Shift/alt click = parallel multi-chat tab (handled by the capture
      // listener in 22_multichat.js); plain click = stock picker.
      if (e.shiftKey || e.altKey) return;
      openPicker(e);
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

  // ── "+" mini picker: recent sessions + static app entries ──

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
    picker.innerHTML = '<div class="gb-tab-picker-title">Open in new tab</div>' +
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

    var appRows = ['<div class="gb-tab-picker-section">Apps</div>'];
    APP_ENTRIES.forEach(function (app, i) {
      appRows.push('<button type="button" class="gb-tab-picker-row" data-app-idx="' + i + '">' +
        '<span class="gb-tab-picker-glyph">' + app.glyph + "</span>" +
        "<span>" + escapeHtml(app.title) + "</span></button>");
    });

    Promise.all([fetchRecentSessions()]).then(function (results) {
      var sessions = results[0] || [];
      var rows = appRows.slice();
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
        var appIdx = row.getAttribute("data-app-idx");
        var sessIdx = row.getAttribute("data-session-idx");
        if (appIdx !== null) {
          var app = APP_ENTRIES[parseInt(appIdx, 10)];
          // Decision: apps open in a NEW browser tab via the desktop shell
          // route so the chat workspace stays intact. Raw /suite/{app}/…
          // paths are fragments and would render an unbootstrapped shell.
          window.open(app.path, "_blank", "noopener");
          return;
        }
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
      menu.innerHTML = '<button type="button" data-act="open">Open in new tab</button>';
      document.body.appendChild(menu);
      positionMenu(menu, e.clientX, e.clientY);
      menu.addEventListener("click", function (ev) {
        if (ev.target.getAttribute("data-act") !== "open") return;
        closeMenus();
        window.dispatchEvent(new CustomEvent("gb-open-history-tab", {
          detail: {
            sessionId: item.getAttribute("data-session-id"),
            title: (item.querySelector(".chat-sidebar-conv-name") || {}).textContent || "Conversation",
          },
        }));
      });
    });

    window.addEventListener("gb-open-history-tab", function (e) {
      var d = e.detail || {};
      if (!d.sessionId) return;
      GBTabs.createTab({
        kind: "history",
        sessionId: d.sessionId,
        botId: (window.ChatState && window.ChatState.currentBotId) || undefined,
        title: d.title || "Conversation",
        faviconGlyph: "\u{1F4AC}",
      });
    });
  }

  // ── Boot ──

  function init() {
    // Event DELEGATION on document: the strip is created dynamically by
    // GBTabs.renderStrip(), and this module's init can run before that (or
    // before every WM re-injection). Binding to a strip element that does
    // not exist yet (or to a node that is later replaced) silently killed
    // every tab click — clicking a tab did nothing. document-level capture
    // survives all strip re-creations.
    document.addEventListener("pointerdown", function (e) {
      if (!e.target.closest || !e.target.closest("#gbTabStrip")) return;
      onPointerDown(e);
    }, true);
    document.addEventListener("click", function (e) {
      if (!e.target.closest || !e.target.closest("#gbTabStrip")) return;
      onClick(e);
    });
    document.addEventListener("dblclick", function (e) {
      if (!e.target.closest || !e.target.closest("#gbTabStrip")) return;
      onDblClick(e);
    });
    document.addEventListener("contextmenu", function (e) {
      if (!e.target.closest || !e.target.closest("#gbTabStrip")) return;
      var id = tabIdFromEvent(e);
      if (!id || e.target.closest(".gb-tab-close")) return;
      e.preventDefault();
      openTabMenu(e.clientX, e.clientY, id);
    });
    // #1283 — middle-click (auxclick button 1) closes the tab.
    document.addEventListener("auxclick", function (e) {
      if (e.button !== 1) return;
      if (!e.target.closest || !e.target.closest("#gbTabStrip")) return;
      var id = tabIdFromEvent(e);
      if (id) { e.preventDefault(); closeTabGuarded(id); }
    });
    document.addEventListener("click", function (e) {
      if (!e.target.closest(".gb-context-menu")) closeMenus();
    });
    document.addEventListener("keydown", function (e) {
      if (e.key === "Escape") closeMenus();
    });
    bindHistoryContextMenu();
    bindKeyboardCycle();
    if (window.GBMultiChat) GBMultiChat.start();
  }

  // #1283 — Ctrl+Tab / Ctrl+Shift+Tab cycle tabs; Ctrl+W closes the active
  // tab (guarded). Plain browser-tab muscle memory, zero new chrome.
  function bindKeyboardCycle() {
    document.addEventListener("keydown", function (e) {
      if (!GBTabs.state.enabled || GBTabs.state.tabs.length < 2) return;
      if (e.ctrlKey && !e.altKey && !e.metaKey && e.key === "Tab") {
        e.preventDefault();
        var tabs = GBTabs.state.tabs;
        var idx = tabs.findIndex(function (t) { return t.id === GBTabs.state.activeTabId; });
        var next = e.shiftKey
          ? tabs[(idx - 1 + tabs.length) % tabs.length]
          : tabs[(idx + 1) % tabs.length];
        if (window.GBMultiChat && GBMultiChat.tabs[next.id]) GBMultiChat.switchTo(next.id);
        else GBTabs.focusTab(next.id);
      } else if (e.ctrlKey && !e.shiftKey && !e.altKey && !e.metaKey && (e.key === "w" || e.key === "W")) {
        var active = GBTabs.activeTab();
        if (active && !active.pinned && GBTabs.state.tabs.length > 1) {
          e.preventDefault();
          closeTabGuarded(active.id);
        }
      }
    });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
