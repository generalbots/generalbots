"use strict";

// GB Sidebar History (#1161) — grouped + searchable conversation history.
// Groups sidebar.js conversation items into Today / Yesterday /
// Previous 7 Days / Older, and filters them by a text search. Depends on
// `window.GBSidebarBase` (sidebar-enhance.js). Load after sidebar-sections.js.

(function () {
  if (window.GBSidebarHistory) return;

  var filterValue = "";
  var mo = null;
  var groupScheduled = false;

  function byId(id) {
    return window.GBSidebarBase.byId(id);
  }

  // #2095 — user-visible labels come from the catalogue; the authored English
  // text is the fallback when the active locale has no entry yet.
  function t(key, fallback) {
    return window.gbT ? window.gbT(key, fallback) : fallback;
  }

  // One entry per time bucket: `id` is internal (grouping/signature), `key`
  // and `label` are the catalogue key and its English fallback.
  var BUCKETS = [
    { id: "today", key: "sidebar-bucket-today", label: "Today" },
    { id: "yesterday", key: "sidebar-bucket-yesterday", label: "Yesterday" },
    { id: "last7", key: "sidebar-bucket-previous-7-days", label: "Previous 7 Days" },
    { id: "older", key: "sidebar-bucket-older", label: "Older" },
  ];

  function timeBucket(ts) {
    var then = new Date(ts).getTime();
    if (!then) return "older";
    var now = new Date();
    var startToday = new Date(
      now.getFullYear(),
      now.getMonth(),
      now.getDate()
    ).getTime();
    var msDay = 86400000;
    if (then >= startToday) return "today";
    if (then >= startToday - msDay) return "yesterday";
    if (then >= startToday - 7 * msDay) return "last7";
    return "older";
  }

  // Regrouping rewrites the host DOM (innerHTML wipe + re-append), which
  // would re-trigger our own MutationObserver in an infinite loop. The
  // observer is therefore disconnected while group() runs, and all
  // regroups — including direct calls — go through the debounced
  // scheduleGroup so bursts coalesce into a single pass.
  function observeHost() {
    var host = byId("chatConversations");
    if (!host || !mo) return;
    mo.observe(host, { childList: true, subtree: true });
  }

  function scheduleGroup() {
    if (groupScheduled) return;
    groupScheduled = true;
    setTimeout(function () {
      groupScheduled = false;
      if (mo) mo.disconnect();
      try {
        group();
      } finally {
        observeHost();
      }
    }, 50);
  }

  function install() {
    var host = byId("chatConversations");
    if (!host || host.parentNode.querySelector(".gb-conv-search")) return;
    var search = document.createElement("div");
    search.className = "gb-conv-search";
    search.innerHTML =
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="12" height="12"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>' +
      '<input type="search" id="gbConvSearch" placeholder="Find a conversation…" autocomplete="off">';
    host.parentNode.insertBefore(search, host);
    search.querySelector("input").addEventListener("input", function (e) {
      filterValue = e.target.value;
      scheduleGroup();
    });
    mo = new MutationObserver(scheduleGroup);
    observeHost();
    scheduleGroup();
  }

  function group() {
    var host = byId("chatConversations");
    if (!host) return;
    var items = Array.prototype.slice.call(
      host.querySelectorAll(".chat-sidebar-conv-item")
    );
    if (!items.length) {
      // renderHistory owns the empty state ("No conversations yet") — leave
      // it in place; just reset the grouping signature.
      host._gbLastSig = "";
      return;
    }
    // Stash the flat list once; re-stash whenever a fresh render replaced
    // the DOM nodes — the previous stash is detached and must never be
    // re-inserted, otherwise the history list freezes on the first fetch.
    var source = host._gbItems || items;
    var stale = source.length && source.some(function (item) {
      return !host.contains(item);
    });
    if (!source.length || stale || items.length !== source.length) {
      host._gbItems = items;
      source = items;
    }
    // Bucket timestamps: prefer the real ISO timestamp carried on the item
    // (data-ts, set by sidebar-convos.js) so a re-render still buckets by
    // the original updated_at instead of the relative label ("2h", …).
    source.forEach(function (item) {
      if (item._gbTs === undefined) {
        var raw = item.getAttribute && item.getAttribute("data-ts");
        if (!raw) {
          var timeEl = item.querySelector(".chat-sidebar-conv-time");
          if (timeEl) raw = timeEl.textContent.trim();
        }
        var ts = new Date(raw || "").getTime();
        item._gbTs = isNaN(ts) ? 0 : ts;
      }
    });
    var groups = {};
    BUCKETS.forEach(function (bucket) {
      groups[bucket.id] = [];
    });
    source.forEach(function (item) {
      var bucket = item._gbTs ? timeBucket(item._gbTs) : "older";
      groups[bucket].push(item);
    });
    // Commit only when the grouping actually changed. Rebuilding the same
    // list mutates the DOM and re-triggers the MutationObserver below,
    // which would call group() again forever (each commit → new mutation).
    var sig = signature(groups);
    if (sig === host._gbLastSig) {
      applyFilter(host);
      return;
    }
    host._gbLastSig = sig;
    // Preserve children that are not conversation items or group labels
    // (e.g. the "Locked conversations" toggle) across regrouping.
    var extras = Array.prototype.slice.call(host.children).filter(function (el) {
      return !el.classList.contains("chat-sidebar-conv-item") &&
             !el.classList.contains("gb-conv-group-title");
    });
    host.innerHTML = "";
    BUCKETS.forEach(function (bucket) {
      if (!groups[bucket.id].length) return;
      var label = document.createElement("div");
      label.className = "chat-sidebar-section-label gb-conv-group-title";
      label.textContent = t(bucket.key, bucket.label);
      host.appendChild(label);
      groups[bucket.id].forEach(function (item) {
        host.appendChild(item);
      });
    });
    extras.forEach(function (el) {
      if (el && el.parentNode !== host) host.appendChild(el);
    });
    applyFilter(host);
  }

  // Lightweight signature of the current grouping (bucket + item order), used
  // to skip pointless DOM rebuilds that would feed the MutationObserver.
  function signature(groups) {
    var parts = [];
    BUCKETS.forEach(function (bucket) {
      if (!groups[bucket.id].length) return;
      parts.push(
        bucket.id +
          ":" +
          groups[bucket.id].map(function (item) {
            return (
              (item.getAttribute && item.getAttribute("data-session-id")) ||
              item._gbTs
            );
          }).join(",")
      );
    });
    return parts.join("|");
  }

  function applyFilter(host) {
    var query = (filterValue || "").toLowerCase();
    host.querySelectorAll(".chat-sidebar-conv-item").forEach(function (item) {
      var name = (item.textContent || "").toLowerCase();
      item.style.display = !query || name.indexOf(query) !== -1 ? "" : "none";
    });
    host.querySelectorAll(".gb-conv-group-title").forEach(function (label) {
      var anyVisible = false;
      var node = label.nextSibling;
      while (node) {
        if (node.nodeType === 1 && node.style.display !== "none") {
          anyVisible = true;
          break;
        }
        node = node.nextSibling;
      }
      label.style.display = anyVisible ? "" : "none";
    });
  }

  function resetForBot() {
    var host = byId("chatConversations");
    if (host) {
      delete host.dataset.grouped;
      host._gbItems = undefined;
      host._gbLastSig = undefined;
      scheduleGroup();
    }
  }

  // A language switch relabels the buckets without touching the items, so the
  // grouping signature is dropped to force one regroup.
  window.addEventListener("localeChanged", function () {
    var host = byId("chatConversations");
    if (host) host._gbLastSig = undefined;
    scheduleGroup();
  });

  window.GBSidebarHistory = {
    install: install,
    resetForBot: resetForBot,
  };
})();