(function (window, document) {
  "use strict";
  var N = document.__gb_mod_collab_comments;
  if (!N) return;
  var S = N.s;
  var ensureCss = N.ensureCss;
  var currentMentionToken = N.currentMentionToken;
  var renderMentions = N.renderMentions;
  var hideMentions = N.hideMentions;
  var highlightMention = N.highlightMention;
  var selectMentionIndex = N.selectMentionIndex;
  var post = N.post;
  var sendPresence = N.sendPresence;
  var react = N.react;
  var del = N.del;
  var reply = N.reply;
  var resolve = N.resolve;
  var state = N.state;
  var load = N.load;
  var markRead = N.markRead;
  var heartbeat = N.heartbeat;
  var stopHeartbeat = N.stopHeartbeat;
  var req = N.req;
  function build() {
    if (S.panel) return;
    ensureCss();
    S.panel = document.createElement("div");
    S.panel.id = "gb-comments-panel";
    S.panel.setAttribute("role", "dialog");
    S.panel.setAttribute("aria-label", "Comments");
    S.panel.innerHTML =
      '<div class="gbc-header">' +
        '<span class="gbc-title" id="gbc-title">Comments</span>' +
        '<span class="gbc-presence" id="gbc-presence"></span>' +
        '<button class="gbc-close" onclick="GBCollabComments.close()" title="Close">\u00D7</button>' +
      "</div>" +
      '<div class="gbc-filter" id="gbc-filter">' +
        '<button class="gbc-filter-btn active" data-filter="open">Open</button>' +
        '<button class="gbc-filter-btn" data-filter="all">All</button>' +
        '<button class="gbc-filter-btn" data-filter="resolved">Resolved</button>' +
      "</div>" +
      '<div class="gbc-list" id="gbc-list"></div>' +
      '<div class="gbc-input-row">' +
        '<input class="gbc-input" id="gbc-input" type="text" placeholder="Add a comment\u2026 use @name to mention" autocomplete="off" />' +
        '<button class="gbc-send" onclick="GBCollabComments.post()">Post</button>' +
      "</div>";
    document.body.appendChild(S.panel);
    S.listEl = document.getElementById("gbc-list");
    S.inputEl = document.getElementById("gbc-input");
    S.presenceEl = document.getElementById("gbc-presence");

    var inputRow = S.panel.querySelector(".gbc-input-row");
    if (inputRow) inputRow.style.position = "relative";
    S.mentionBox = document.createElement("div");
    S.mentionBox.className = "gbc-mention-box";
    S.mentionBox.style.cssText = "position:absolute;left:14px;right:14px;bottom:54px;background:#1e293b;border:1px solid #334155;border-radius:8px;box-shadow:0 8px 24px rgba(0,0,0,.45);max-height:220px;overflow-y:auto;display:none;z-index:10;";
    if (inputRow) inputRow.appendChild(S.mentionBox);

    S.inputEl.addEventListener("input", function () {
      var token = currentMentionToken();
      if (token) renderMentions(token);
      else hideMentions();
    });
    S.inputEl.addEventListener("keydown", function (e) {
      var open = S.mentionBox && S.mentionBox.style.display !== "none" && S.mentionItems.length > 0;
      if (open) {
        if (e.key === "ArrowDown") {
          e.preventDefault();
          S.mentionIndex = (S.mentionIndex + 1) % S.mentionItems.length;
          highlightMention(S.mentionIndex);
          return;
        }
        if (e.key === "ArrowUp") {
          e.preventDefault();
          S.mentionIndex = (S.mentionIndex - 1 + S.mentionItems.length) % S.mentionItems.length;
          highlightMention(S.mentionIndex);
          return;
        }
        if (e.key === "Enter" || e.key === "Tab") {
          if (e.key === "Enter") e.preventDefault();
          if (selectMentionIndex()) return;
        }
        if (e.key === "Escape") {
          e.preventDefault();
          hideMentions();
          return;
        }
      }
      if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); hideMentions(); post(); return; }
      S.typingSent = true;
      sendPresence(true);
    });

    S.listEl.addEventListener("click", function (e) {
      var reactEl = e.target.closest(".gbc-react");
      if (reactEl) { react(reactEl.dataset.comment, reactEl.dataset.emoji); return; }
      var chipEl = e.target.closest(".gbc-chip");
      if (chipEl) { react(chipEl.dataset.comment, chipEl.dataset.emoji); return; }
      var delEl = e.target.closest(".gbc-del");
      if (delEl) { del(delEl.dataset.comment); return; }
      var replyEl = e.target.closest(".gbc-reply-btn");
      if (replyEl) { reply(replyEl.dataset.comment); return; }
      var resolveEl = e.target.closest(".gbc-resolve-btn");
      if (resolveEl) { resolve(resolveEl.dataset.comment, resolveEl.dataset.resolved !== "1"); return; }
    });

    var filterEl = document.getElementById("gbc-filter");
    if (filterEl) {
      filterEl.addEventListener("click", function (e) {
        var btn = e.target.closest(".gbc-filter-btn");
        if (!btn) return;
        state.filter = btn.dataset.filter;
        filterEl.querySelectorAll(".gbc-filter-btn").forEach(function (b) { b.classList.toggle("active", b === btn); });
        load();
      });
    }
  }

  function open(opts) {
    opts = opts || {};
    build();
    state.resourceType = opts.resourceType || null;
    state.resourceId = opts.resourceId || null;
    state.title = opts.title || "Comments";
    state.notify = opts.notify || null;
    state.includeChildren = !!opts.includeChildren;
    state.filter = "open";
    var filterEl = document.getElementById("gbc-filter");
    if (filterEl) filterEl.querySelectorAll(".gbc-filter-btn").forEach(function (b) { b.classList.toggle("active", b.dataset.filter === "open"); });
    document.getElementById("gbc-title").textContent = state.title;
    if (S.inputEl) S.inputEl.value = "";
    S.panel.classList.add("gbc-open");
    load();
    markRead();
    heartbeat();
  }

  function close() {
    if (S.panel) S.panel.classList.remove("gbc-open");
    stopHeartbeat();
  }

  // Mount a button that opens the panel for a resolved resource.
  function mountButton(container, opts) {
    if (!container || !opts || typeof opts.resourceId !== "function") return null;
    build();
    var btn = document.createElement("button");
    btn.type = "button";
    btn.textContent = opts.label || "\uD83D\uDCAC Comments";
    btn.className = "gbc-mount-btn";
    btn.style.cssText = "background:#1e293b;color:#f8fafc;border:1px solid #334155;border-radius:6px;padding:6px 12px;font-size:13px;cursor:pointer;";
    btn.addEventListener("click", function () {
      open({ resourceType: opts.resourceType, resourceId: opts.resourceId(), title: opts.title || "Comments", notify: opts.notify });
    });
    container.appendChild(btn);
    return btn;
  }

  // Bind a live unread-count badge to a button/container. Polls the unread
  // endpoint and shows a red count pill; opening the panel (markRead) clears it.
  function bindBadge(container, opts) {
    if (!container || !opts || !opts.resourceType) return null;
    if (container.__gbcBadgeBound) return container.__gbcBadgeBound;
    build();
    if (container.style && !container.style.position) container.style.position = "relative";
    var badge = document.createElement("span");
    badge.className = "gbc-badge";
    badge.style.cssText = "display:none;position:absolute;top:-4px;right:-4px;min-width:16px;height:16px;border-radius:999px;background:#ef4444;color:#fff;font-size:10px;font-weight:700;line-height:16px;text-align:center;padding:0 4px;pointer-events:none;";
    container.appendChild(badge);
    container.__gbcBadgeBound = badge;
    function refresh() {
      var qs = "/comments/unread?resource_type=" + encodeURIComponent(opts.resourceType) + "&resource_id=" + encodeURIComponent(opts.resourceId);
      if (opts.includeChildren) qs += "&include_children=true";
      req(qs).then(function (r) {
        var n = (r && r.count) || 0;
        badge.textContent = n > 99 ? "99+" : String(n);
        badge.style.display = n > 0 ? "inline-block" : "none";
      }).catch(function () {});
    }
    S.badgeRefreshFn = refresh;
    refresh();
    if (!container.__gbcBadgeTimer) container.__gbcBadgeTimer = setInterval(refresh, 30000);
    return badge;
  }

  // In-app @mention notification: poll the mentions feed and announce new
  // mentions via the screen-reader live region (no SMTP dependency).
  var lastMentionId = null;
  function pollMentions() {
    if (!window.GBCollabA11y) return;
    req("/mentions/me?limit=5")
      .then(function (items) {
        if (!items || !items.length) return;
        var newest = items[0];
        if (newest && newest.id !== lastMentionId) {
          lastMentionId = newest.id;
          window.GBCollabA11y.announce(
            "You were mentioned in a comment by " + (newest.author_name || "someone"),
            true
          );
        }
      })
      .catch(function () {});
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", function () { pollMentions(); setInterval(pollMentions, 60000); });
  } else {
    pollMentions();
    setInterval(pollMentions, 60000);
  }

  window.GBCollabComments = { open: open, close: close, post: post, mountButton: mountButton, bindBadge: bindBadge, resolve: resolve };

  N.build = build;
  N.open = open;
  N.close = close;
  N.mountButton = mountButton;
  N.bindBadge = bindBadge;
  N.pollMentions = pollMentions;
  delete document.__gb_mod_collab_comments;
})(window, document);
