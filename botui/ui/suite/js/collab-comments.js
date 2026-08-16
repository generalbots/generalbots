"use strict";
/* GBCollabComments — shared threaded-comments panel for every collaboration app
 * (drive, sheet, docs, tasks, calendar, slides, plan...). Backed by the generic
 * REST API at /api/collab/* (threaded comments, @mentions, emoji reactions,
 * presence). This file is app-agnostic: it ships its own styles and only needs
 * a resource_type + resource_id to address a document, cell, task or event.
 *
 * Public API (window.GBCollabComments):
 *   open({ resourceType, resourceId, title, notify })  — open the panel
 *   close()                                            — close + stop heartbeat
 *   post()                                             — submit the composer
 *   mountButton(container, { resourceType, resourceId, label, notify })
 *                                                      — inject a button that opens
 *                                                        the panel for a resource
 *
 * Auth: reads the JWT from localStorage/sessionStorage (gb-access-token) and
 * sends it as a Bearer header. All endpoints are authenticated server-side.
 */
(function (window) {
  var CSS_ID = "gb-collab-comments-css";
  var HEARTBEAT_MS = 30000;
  var panel = null;
  var listEl = null;
  var inputEl = null;
  var presenceEl = null;
  var heartbeatTimer = null;
  var typingSent = false;
  var state = { resourceType: null, resourceId: null, title: "Comments", notify: null, includeChildren: false };
  var mentionBox = null;      // floating @mention autocomplete dropdown
  var mentionCandidates = []; // [{ id, name }] merged from presence + thread authors
  var presenceUsers = [];     // raw /api/collab/presence items
  var commentAuthors = [];    // raw { author_id, author_name } from the thread
  var mentionItems = [];      // visible candidate buttons (keyboard nav)
  var mentionIndex = -1;      // highlighted candidate index
  var activeMention = null;   // { at, partial } token currently open

  function ensureCss() {
    if (document.getElementById(CSS_ID)) return;
    var css = [
      "#gb-comments-panel{position:fixed;top:0;right:0;bottom:0;width:340px;max-width:92vw;",
      "background:#0f172a;border-left:1px solid #334155;z-index:100000;display:flex;flex-direction:column;",
      "box-shadow:-8px 0 24px rgba(0,0,0,.4);transform:translateX(100%);transition:transform .2s ease;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;}",
      "#gb-comments-panel.gbc-open{transform:translateX(0);}",
      "#gb-comments-panel .gbc-header{display:flex;align-items:center;gap:8px;padding:12px 14px;border-bottom:1px solid #334155;background:#1e293b;}",
      "#gb-comments-panel .gbc-title{flex:1;color:#f8fafc;font-size:14px;font-weight:600;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;}",
      "#gb-comments-panel .gbc-presence{display:none;font-size:11px;color:#94a3b8;max-width:120px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;}",
      "#gb-comments-panel .gbc-close{background:none;border:none;color:#94a3b8;font-size:20px;line-height:1;cursor:pointer;padding:0 4px;}",
      "#gb-comments-panel .gbc-close:hover{color:#f8fafc;}",
      "#gb-comments-panel .gbc-list{flex:1;overflow-y:auto;padding:12px 14px;display:flex;flex-direction:column;gap:12px;}",
      "#gb-comments-panel .gbc-empty,#gb-comments-panel .gbc-loading,#gb-comments-panel .gbc-error{color:#94a3b8;font-size:13px;text-align:center;padding:24px 8px;}",
      "#gb-comments-panel .gbc-error{color:#f87171;}",
      "#gb-comments-panel .gbc-input-row{display:flex;gap:8px;padding:10px 14px;border-top:1px solid #334155;background:#1e293b;}",
      "#gb-comments-panel .gbc-input{flex:1;background:#0f172a;border:1px solid #334155;border-radius:6px;color:#f8fafc;padding:8px 10px;font-size:13px;font-family:inherit;}",
      "#gb-comments-panel .gbc-send{background:#3b82f6;color:#fff;border:none;border-radius:6px;padding:8px 14px;font-size:13px;font-weight:600;cursor:pointer;}",
      "#gb-comments-panel .gbc-send:hover{background:#2563eb;}",
      "#gb-comments-panel .gbc-item{background:#1e293b;border:1px solid #334155;border-radius:8px;padding:10px 12px;}",
      "#gb-comments-panel .gbc-item.gbc-reply{margin-left:18px;background:#16202f;}",
      "#gb-comments-panel .gbc-meta{display:flex;align-items:baseline;gap:8px;margin-bottom:4px;}",
      "#gb-comments-panel .gbc-author{color:#f8fafc;font-weight:600;font-size:12.5px;}",
      "#gb-comments-panel .gbc-time{color:#64748b;font-size:11px;}",
      "#gb-comments-panel .gbc-body{color:#e2e8f0;font-size:13px;line-height:1.5;white-space:pre-wrap;word-break:break-word;}",
      "#gb-comments-panel .gbc-body .gbc-mention{color:#60a5fa;font-weight:600;background:rgba(59,130,246,.15);border-radius:3px;padding:0 2px;}",
      "#gb-comments-panel .gbc-actions{display:flex;align-items:center;gap:4px;flex-wrap:wrap;margin-top:8px;}",
      "#gb-comments-panel .gbc-react{font-size:14px;padding:2px 6px;border-radius:4px;cursor:pointer;user-select:none;}",
      "#gb-comments-panel .gbc-react:hover{background:#334155;}",
      "#gb-comments-panel .gbc-chip{display:inline-flex;align-items:center;gap:2px;font-size:12px;background:#334155;border-radius:999px;padding:2px 8px;cursor:pointer;}",
      "#gb-comments-panel .gbc-chip:hover{background:#475569;}",
      "#gb-comments-panel .gbc-reply-btn,#gb-comments-panel .gbc-del{font-size:12px;color:#94a3b8;cursor:pointer;padding:2px 4px;border-radius:4px;}",
      "#gb-comments-panel .gbc-reply-btn:hover{color:#60a5fa;background:#1e293b;}",
      "#gb-comments-panel .gbc-del:hover{color:#f87171;background:#1e293b;}",
      "#gb-comments-panel .gbc-replies{display:flex;flex-direction:column;gap:8px;margin-top:10px;}"
    ].join("");
    var style = document.createElement("style");
    style.id = CSS_ID;
    style.textContent = css;
    document.head.appendChild(style);
  }

  function esc(s) {
    return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;").replace(/'/g, "&#39;");
  }

  function token() {
    try {
      return localStorage.getItem("gb-access-token") || sessionStorage.getItem("gb-access-token") || "";
    } catch (_) { return ""; }
  }

  function req(endpoint, options) {
    var headers = { "Content-Type": "application/json" };
    var t = token();
    if (t) headers["Authorization"] = "Bearer " + t;
    return fetch("/api/collab" + endpoint, Object.assign({ headers: headers }, options || {}))
      .then(function (res) {
        return res.json().catch(function () { return { error: res.statusText }; })
          .then(function (body) {
            if (!res.ok) throw new Error(body.error || "Request failed (" + res.status + ")");
            return body;
          });
      });
  }

  function renderBody(body) {
    return esc(body).replace(/@([\w.@-]+)/g, '<span class="gbc-mention">@$1</span>');
  }

  function timeFmt(iso) {
    try { return new Date(iso).toLocaleString(); } catch (_) { return iso; }
  }

  function chips(comment) {
    if (!comment.reactions || !comment.reactions.length) return "";
    var counts = {};
    comment.reactions.forEach(function (r) { counts[r.emoji] = (counts[r.emoji] || 0) + 1; });
    return Object.keys(counts).map(function (e) {
      return '<span class="gbc-chip" data-comment="' + esc(comment.id) + '" data-emoji="' + esc(e) + '">' + esc(e) + " " + counts[e] + "</span>";
    }).join("");
  }

  // When the panel aggregates child resources (includeChildren), surface the
  // anchor (e.g. cell A1) of a comment whose resource is nested under the
  // panel's resource, so users can tell where each comment belongs.
  function anchorBadge(comment) {
    if (!state.includeChildren || !comment.resource_type || comment.resource_type === state.resourceType) return "";
    var suffix = comment.resource_id || "";
    if (state.resourceId && suffix.indexOf(state.resourceId + ":") === 0) {
      suffix = suffix.slice(state.resourceId.length + 1);
    }
    var parts = suffix.split(":");
    var cell = parts[parts.length - 1];
    if (!cell) return "";
    return '<span class="gbc-anchor" style="font-size:10px;color:#60a5fa;background:rgba(59,130,246,.12);border-radius:3px;padding:0 5px;line-height:16px;font-weight:600;">' + esc(cell) + "</span>";
  }

  function node(comment, isReply) {
    var n = document.createElement("div");
    n.className = "gbc-item" + (isReply ? " gbc-reply" : "");
    n.innerHTML =
      '<div class="gbc-meta"><span class="gbc-author">' + esc(comment.author_name || comment.author_id) + "</span>" +
      anchorBadge(comment) +
      '<span class="gbc-time">' + timeFmt(comment.created_at) + "</span></div>" +
      '<div class="gbc-body">' + renderBody(comment.body) + "</div>" +
      '<div class="gbc-actions">' +
        ["👍", "❤️", "😄", "🎉"].map(function (e) {
          return '<span class="gbc-react" data-comment="' + esc(comment.id) + '" data-emoji="' + e + '">' + e + "</span>";
        }).join("") +
        chips(comment) +
        '<span class="gbc-reply-btn" data-comment="' + esc(comment.id) + '">Reply</span>' +
        '<span class="gbc-del" data-comment="' + esc(comment.id) + '">Delete</span>' +
      "</div>";
    if (comment.replies && comment.replies.length) {
      var wrap = document.createElement("div");
      wrap.className = "gbc-replies";
      comment.replies.forEach(function (r) { wrap.appendChild(node(r, true)); });
      n.appendChild(wrap);
    }
    return n;
  }

  function load() {
    if (!listEl || !state.resourceId) return;
    listEl.innerHTML = '<div class="gbc-loading">Loading comments\u2026</div>';
    var qs = "/comments?resource_type=" + encodeURIComponent(state.resourceType) + "&resource_id=" + encodeURIComponent(state.resourceId);
    if (state.includeChildren) qs += "&include_children=true";
    req(qs)
      .then(function (items) {
        listEl.innerHTML = "";
        commentAuthors = [];
        if (!items || !items.length) {
          listEl.innerHTML = '<div class="gbc-empty">No comments yet. Use @name to mention someone.</div>';
        } else {
          var seen = {};
          items.forEach(function (c) {
            listEl.appendChild(node(c, false));
            if (c.author_name && !seen[c.author_name]) {
              seen[c.author_name] = true;
              commentAuthors.push({ author_id: c.author_id, author_name: c.author_name });
            }
          });
        }
        refreshMentionCandidates();
        loadPresence();
      })
      .catch(function (e) {
        listEl.innerHTML = '<div class="gbc-error">Failed to load comments: ' + esc(e.message) + "</div>";
      });
  }

  function loadPresence() {
    if (!presenceEl || !state.resourceId) return;
    req("/presence?resource_type=" + encodeURIComponent(state.resourceType) + "&resource_id=" + encodeURIComponent(state.resourceId))
      .then(function (items) {
        presenceUsers = items || [];
        refreshMentionCandidates();
        if (!items || !items.length) { presenceEl.style.display = "none"; presenceEl.textContent = ""; return; }
        var typing = items.filter(function (p) { return p.typing; });
        var names = items.map(function (p) { return p.user_name; });
        presenceEl.textContent = typing.length ? "typing: " + names.join(", ") : "viewing: " + names.join(", ");
        presenceEl.style.display = "inline-block";
      })
      .catch(function () {});
  }

  function sendPresence(typing) {
    if (!state.resourceId) return;
    req("/presence", {
      method: "POST",
      body: JSON.stringify({ resource_type: state.resourceType, resource_id: state.resourceId, typing: !!typing })
    }).catch(function () { /* presence is best-effort */ });
  }

  function heartbeat() {
    stopHeartbeat();
    if (!state.resourceId) return;
    sendPresence(false);
    heartbeatTimer = setInterval(function () { sendPresence(typingSent); }, HEARTBEAT_MS);
  }

  function stopHeartbeat() {
    if (heartbeatTimer) { clearInterval(heartbeatTimer); heartbeatTimer = null; }
    typingSent = false;
  }

  function addComment(body, parentId) {
    var payload = { resource_type: state.resourceType, resource_id: state.resourceId, body: body };
    if (parentId) payload.parent_id = parentId;
    return req("/comments", { method: "POST", body: JSON.stringify(payload) });
  }

  function post() {
    if (!inputEl) return;
    var body = inputEl.value.trim();
    if (!body) return;
    inputEl.value = "";
    typingSent = false;
    addComment(body, null).then(load).catch(function (e) { notify("Comment failed: " + e.message, "error"); });
  }

  function react(commentId, emoji) {
    req("/comments/" + commentId + "/reactions", { method: "POST", body: JSON.stringify({ emoji: emoji }) })
      .then(load)
      .catch(function (e) { notify("Reaction failed: " + e.message, "error"); });
  }

  function del(commentId) {
    if (!window.confirm("Delete this comment?")) return;
    req("/comments/" + commentId, { method: "DELETE" })
      .then(load)
      .catch(function (e) { notify("Delete failed: " + e.message, "error"); });
  }

  function reply(commentId) {
    var body = window.prompt("Reply to comment:");
    if (!body) return;
    addComment(body.trim(), commentId).then(load).catch(function (e) { notify("Reply failed: " + e.message, "error"); });
  }

  function notify(msg, type) {
    if (typeof state.notify === "function") { state.notify(msg, type); return; }
    // Minimal fallback toast.
    var t = document.createElement("div");
    t.style.cssText = "position:fixed;bottom:24px;right:24px;z-index:100001;padding:10px 16px;border-radius:8px;color:#fff;font-size:13px;box-shadow:0 4px 12px rgba(0,0,0,.3);background:" + (type === "error" ? "#ef4444" : "#3b82f6");
    t.textContent = msg;
    document.body.appendChild(t);
    setTimeout(function () { t.remove(); }, 4000);
  }

  // Merge active presence users with the thread's authors into the
  // autocomplete candidate list (deduplicated by display name).
  function refreshMentionCandidates() {
    var seen = {};
    var out = [];
    function add(id, name) {
      if (!name || seen[name]) return;
      seen[name] = true;
      out.push({ id: id || name, name: name });
    }
    presenceUsers.forEach(function (p) { add(p.user_id, p.user_name); });
    commentAuthors.forEach(function (a) { add(a.author_id, a.author_name); });
    mentionCandidates = out;
  }

  // The partial @token being typed at the caret, or null when not mentioning.
  function currentMentionToken() {
    if (!inputEl) return null;
    var value = inputEl.value;
    var pos = inputEl.selectionStart == null ? value.length : inputEl.selectionStart;
    var before = value.slice(0, pos);
    var at = before.lastIndexOf("@");
    if (at === -1) return null;
    var partial = before.slice(at + 1);
    if (!/^[\w.@-]*$/.test(partial)) return null;
    return { at: at, partial: partial };
  }

  function hideMentions() {
    if (mentionBox) { mentionBox.style.display = "none"; mentionBox.innerHTML = ""; }
    mentionItems = [];
    mentionIndex = -1;
    activeMention = null;
  }

  function highlightMention(idx) {
    if (!mentionBox) return;
    var children = mentionBox.children;
    for (var i = 0; i < children.length; i++) {
      children[i].style.background = i === idx ? "#334155" : "none";
    }
  }

  function renderMentions(token) {
    if (!mentionBox) return;
    var lower = token.partial.toLowerCase();
    var matches = mentionCandidates.filter(function (c) {
      return c.name.toLowerCase().indexOf(lower) === 0;
    }).slice(0, 8);
    if (!matches.length) { hideMentions(); return; }
    mentionBox.innerHTML = "";
    mentionItems = [];
    matches.forEach(function (c) {
      var b = document.createElement("button");
      b.type = "button";
      b.className = "gbc-mention-item";
      b.textContent = c.name;
      b.style.cssText = "display:block;width:100%;text-align:left;background:none;border:none;color:#f8fafc;font-size:13px;padding:8px 12px;cursor:pointer;";
      b.addEventListener("mouseover", function () {
        mentionIndex = mentionItems.indexOf(b);
        highlightMention(mentionIndex);
      });
      b.addEventListener("mousedown", function (e) { e.preventDefault(); applyMention(token.at, c.name); });
      mentionBox.appendChild(b);
      mentionItems.push(b);
    });
    activeMention = token;
    mentionIndex = 0;
    highlightMention(0);
    mentionBox.style.display = "block";
  }

  function selectMentionIndex() {
    if (mentionIndex < 0 || mentionIndex >= mentionItems.length || !activeMention) return false;
    applyMention(activeMention.at, mentionItems[mentionIndex].textContent);
    return true;
  }

  function applyMention(at, name) {
    if (!inputEl) return;
    var value = inputEl.value;
    var pos = inputEl.selectionStart == null ? value.length : inputEl.selectionStart;
    var newValue = value.slice(0, at) + "@" + name + " " + value.slice(pos);
    inputEl.value = newValue;
    var caret = at + name.length + 2; // after "@name "
    inputEl.focus();
    try { inputEl.setSelectionRange(caret, caret); } catch (_) {}
    hideMentions();
  }

  function build() {
    if (panel) return;
    ensureCss();
    panel = document.createElement("div");
    panel.id = "gb-comments-panel";
    panel.innerHTML =
      '<div class="gbc-header">' +
        '<span class="gbc-title" id="gbc-title">Comments</span>' +
        '<span class="gbc-presence" id="gbc-presence"></span>' +
        '<button class="gbc-close" onclick="GBCollabComments.close()" title="Close">\u00D7</button>' +
      "</div>" +
      '<div class="gbc-list" id="gbc-list"></div>' +
      '<div class="gbc-input-row">' +
        '<input class="gbc-input" id="gbc-input" type="text" placeholder="Add a comment\u2026 use @name to mention" autocomplete="off" />' +
        '<button class="gbc-send" onclick="GBCollabComments.post()">Post</button>' +
      "</div>";
    document.body.appendChild(panel);
    listEl = document.getElementById("gbc-list");
    inputEl = document.getElementById("gbc-input");
    presenceEl = document.getElementById("gbc-presence");

    var inputRow = panel.querySelector(".gbc-input-row");
    if (inputRow) inputRow.style.position = "relative";
    mentionBox = document.createElement("div");
    mentionBox.className = "gbc-mention-box";
    mentionBox.style.cssText = "position:absolute;left:14px;right:14px;bottom:54px;background:#1e293b;border:1px solid #334155;border-radius:8px;box-shadow:0 8px 24px rgba(0,0,0,.45);max-height:220px;overflow-y:auto;display:none;z-index:10;";
    if (inputRow) inputRow.appendChild(mentionBox);

    inputEl.addEventListener("input", function () {
      var token = currentMentionToken();
      if (token) renderMentions(token);
      else hideMentions();
    });
    inputEl.addEventListener("keydown", function (e) {
      var open = mentionBox && mentionBox.style.display !== "none" && mentionItems.length > 0;
      if (open) {
        if (e.key === "ArrowDown") {
          e.preventDefault();
          mentionIndex = (mentionIndex + 1) % mentionItems.length;
          highlightMention(mentionIndex);
          return;
        }
        if (e.key === "ArrowUp") {
          e.preventDefault();
          mentionIndex = (mentionIndex - 1 + mentionItems.length) % mentionItems.length;
          highlightMention(mentionIndex);
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
      typingSent = true;
      sendPresence(true);
    });

    listEl.addEventListener("click", function (e) {
      var reactEl = e.target.closest(".gbc-react");
      if (reactEl) { react(reactEl.dataset.comment, reactEl.dataset.emoji); return; }
      var chipEl = e.target.closest(".gbc-chip");
      if (chipEl) { react(chipEl.dataset.comment, chipEl.dataset.emoji); return; }
      var delEl = e.target.closest(".gbc-del");
      if (delEl) { del(delEl.dataset.comment); return; }
      var replyEl = e.target.closest(".gbc-reply-btn");
      if (replyEl) { reply(replyEl.dataset.comment); return; }
    });
  }

  function open(opts) {
    opts = opts || {};
    build();
    state.resourceType = opts.resourceType || null;
    state.resourceId = opts.resourceId || null;
    state.title = opts.title || "Comments";
    state.notify = opts.notify || null;
    state.includeChildren = !!opts.includeChildren;
    document.getElementById("gbc-title").textContent = state.title;
    if (inputEl) inputEl.value = "";
    panel.classList.add("gbc-open");
    load();
    heartbeat();
  }

  function close() {
    if (panel) panel.classList.remove("gbc-open");
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

  window.GBCollabComments = { open: open, close: close, post: post, mountButton: mountButton };
})(window);
