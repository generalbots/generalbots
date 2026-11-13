"use strict";

// Vibe multi-chat (#1282/#1283/#1285-fe) — namespace, DOM helpers and tab
// persistence. Split from the former 22_multichat module for #1731.
//
// Each tab is an INDEPENDENT conversation: its own WebSocket session, its own
// message container, its own streaming state. This file only defines the
// shared namespace (window.GBMultiChat), element/token helpers and the
// localStorage fallback; the pane pipeline lives in the sibling files.

(function () {
  var MC = (window.GBMultiChat = {
    tabs: {}, // tabId -> record
    activeId: null,
    seq: 0,
  });

  var STORAGE_KEY = "gb.multichat.v1";

  function q(id) {
    return document.getElementById(id);
  }

  function wrapper() {
    return q("chatContentWrapper");
  }

  function strip() {
    return q("gbTabStrip");
  }

  function tabRecord(id) {
    return MC.tabs[id] || null;
  }

  function token() {
    return (
      (window.GBTabs && GBTabs.token && GBTabs.token()) ||
      (typeof window.getChatAccessToken === "function" && window.getChatAccessToken()) ||
      (window.GBSecurity && window.GBSecurity.getToken && window.GBSecurity.getToken()) ||
      sessionStorage.getItem("gb-access-token") ||
      ""
    );
  }

  function esc(s) {
    var d = document.createElement("div");
    d.textContent = s == null ? "" : String(s);
    return d.innerHTML;
  }

  /* ── Persistence (localStorage fallback, #1285 AC3) ── */

  function persist() {
    try {
      var rec = { tabs: {}, activeId: MC.activeId };
      Object.keys(MC.tabs).forEach(function (id) {
        var t = MC.tabs[id];
        rec.tabs[id] = {
          title: t.title,
          sessionId: t.sessionId,
          scrollback: t.scrollback.slice(-100),
          unread: t.unread || 0,
        };
      });
      localStorage.setItem(STORAGE_KEY, JSON.stringify(rec));
    } catch (e) { /* storage unavailable */ }
  }

  var persistTimer = null;
  function schedulePersist() {
    if (persistTimer) return;
    persistTimer = setTimeout(function () {
      persistTimer = null;
      persist();
    }, 500);
  }

  function restore() {
    var rec = null;
    try {
      rec = gbStoreGet(STORAGE_KEY, null);
    } catch (e) { rec = null; }
    if (!rec || !rec.tabs) return false;
    var ids = Object.keys(rec.tabs);
    if (!ids.length) return false;
    ids.forEach(function (id) {
      var saved = rec.tabs[id];
      var tab = MC.ensureTab(id, saved.title || "Chat", saved.sessionId || null);
      // The stock tab's conversation IS the stock DOM — never resurrect a
      // persisted scrollback for it (pre-fix versions accumulated one
      // greeting copy per window open here).
      if (tab.usesStockDom || id === "tab-default") {
        tab.scrollback = [];
      } else {
        tab.scrollback = Array.isArray(saved.scrollback) ? saved.scrollback.slice(-100) : [];
      }
      tab.needsHistoryReplay = tab.scrollback.length === 0 && !!tab.sessionId;
      tab.scrollback.forEach(function (m) {
        MC.appendToContainer(tab.pane, m.sender === "user" ? "user" : "bot", m.content);
      });
    });
    // Restore the last active tab (default: the first non-original tab).
    var target = rec.activeId && MC.tabs[rec.activeId] ? rec.activeId : null;
    if (!target) {
      var ids2 = Object.keys(MC.tabs);
      target = ids2.length ? ids2[ids2.length - 1] : null;
    }
    if (target) MC.switchTo(target);
    return true;
  }
  /* ── Published for the sibling multichat modules ── */

  MC.q = q;
  MC.wrapper = wrapper;
  MC.token = token;
  MC.esc = esc;
  MC.schedulePersist = schedulePersist;
  MC.restore = restore;
})();
