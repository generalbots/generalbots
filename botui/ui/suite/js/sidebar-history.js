"use strict";

// GB Sidebar History (#1161) — grouped + searchable conversation history.
// Groups sidebar.js conversation items into Today / Yesterday /
// Previous 7 Days / Older, and filters them by a text search. Depends on
// `window.GBSidebarBase` (sidebar-enhance.js). Load after sidebar-sections.js.

(function () {
  if (window.GBSidebarHistory) return;

  var filterValue = "";
  var repainting = false;

  function byId(id) {
    return window.GBSidebarBase.byId(id);
  }

  function timeBucket(ts) {
    var then = new Date(ts).getTime();
    if (!then) return "Older";
    var now = new Date();
    var startToday = new Date(
      now.getFullYear(),
      now.getMonth(),
      now.getDate()
    ).getTime();
    var msDay = 86400000;
    if (then >= startToday) return "Today";
    if (then >= startToday - msDay) return "Yesterday";
    if (then >= startToday - 7 * msDay) return "Previous 7 Days";
    return "Older";
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
      group();
    });
    new MutationObserver(function () {
      if (repainting) return;
      repainting = true;
      group();
      repainting = false;
    }).observe(host, { childList: true, subtree: true });
    group();
  }

  function group() {
    var host = byId("chatConversations");
    if (!host) return;
    var items = Array.prototype.slice.call(
      host.querySelectorAll(".chat-sidebar-conv-item")
    );
    if (!items.length) {
      host.innerHTML = "";
      return;
    }
    // First pass: stash the flat list once; subsequent renders regroup it.
    if (!host.dataset.grouped) {
      host.dataset.grouped = "1";
      items.forEach(function (item) {
        var timeEl = item.querySelector(".chat-sidebar-conv-time");
        if (timeEl) {
          var ts = new Date(timeEl.textContent.trim()).getTime();
          item._gbTs = isNaN(ts) ? 0 : ts;
        } else {
          item._gbTs = 0;
        }
      });
      host._gbItems = items;
    }
    // Re-stash when the source list changes length (new session added).
    var source = host._gbItems || items;
    if (items.length !== source.length) {
      host._gbItems = items;
      source = items;
    }
    var groups = {
      Today: [],
      Yesterday: [],
      "Previous 7 Days": [],
      Older: [],
    };
    source.forEach(function (item) {
      var bucket = item._gbTs ? timeBucket(item._gbTs) : "Older";
      groups[bucket].push(item);
    });
    host.innerHTML = "";
    ["Today", "Yesterday", "Previous 7 Days", "Older"].forEach(function (name) {
      if (!groups[name].length) return;
      var label = document.createElement("div");
      label.className = "chat-sidebar-section-label gb-conv-group-title";
      label.textContent = name;
      host.appendChild(label);
      groups[name].forEach(function (item) {
        host.appendChild(item);
      });
    });
    applyFilter(host);
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
      group();
    }
  }

  window.GBSidebarHistory = {
    install: install,
    resetForBot: resetForBot,
  };
})();